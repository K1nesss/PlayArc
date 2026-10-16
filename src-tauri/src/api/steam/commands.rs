use reqwest::Client;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use crate::storage::db::get_db_connection;
use crate::storage::meta::{
    delete_meta, get_meta, set_meta, STEAM_LAST_ERROR, STEAM_LAST_ERROR_AT, STEAM_LAST_SYNC_AT,
};
use crate::system::logging::{log_error_event, log_info_event, log_warn_event};

const MEDIA_FAILURE_RETRY_SECONDS: i64 = 24 * 60 * 60;
const STEAM_MEDIA_EVENT: &str = "STEAM_MEDIA_SYNC";
static STEAM_MEDIA_SYNC_RUNNING: AtomicBool = AtomicBool::new(false);

/// HTTP client for all Steam requests.
/// - Trusts the operating system's certificate store in addition to the bundled roots
///   (feature `rustls-tls-native-roots`), so tools that proxy Steam traffic with a locally
///   installed certificate (e.g. Watt Toolkit) work.
/// - Times out instead of hanging when api.steampowered.com is unreachable.
fn steam_http_client() -> Client {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// A readable description of a failed request: what went wrong plus the underlying cause
/// (DNS, connect, TLS, timeout…). The URL is always dropped — Steam API URLs carry the key.
fn describe_request_error(error: reqwest::Error) -> String {
    let hint = if error.is_timeout() {
        Some("timed out")
    } else if error.is_connect() {
        Some("could not connect")
    } else {
        None
    };
    let error = error.without_url();
    let mut parts: Vec<String> = Vec::new();
    if let Some(hint) = hint {
        parts.push(hint.to_string());
    }
    let mut current: Option<&dyn std::error::Error> = Some(&error);
    while let Some(err) = current {
        let text = err.to_string();
        if !text.is_empty() && !parts.iter().any(|part| part.contains(&text)) {
            parts.push(text);
        }
        current = err.source();
    }
    parts.join(": ")
}

/// Last line of defence: never let the API key reach a message, log line or the UI.
fn redact_key(message: String, api_key: &str) -> String {
    if api_key.len() >= 8 {
        message.replace(api_key, "***")
    } else {
        message
    }
}

#[derive(Serialize)]
pub struct SteamIconSyncSummary {
    pub matched_games: usize,
    pub downloaded_icons: usize,
    pub downloaded_covers: usize,
    pub downloaded_heroes: usize,
    pub updated_games: usize,
    pub skipped_reason: Option<String>,
}

#[derive(Deserialize)]
struct OwnedGamesPayload {
    response: OwnedGamesResponse,
}

#[derive(Deserialize)]
struct OwnedGamesResponse {
    games: Option<Vec<OwnedGame>>,
}

#[derive(Deserialize)]
struct OwnedGame {
    appid: i32,
    img_icon_url: Option<String>,
}

#[derive(Clone)]
struct LocalGameMediaTarget {
    id: i32,
    appid: i32,
    name: String,
    needs_icon: bool,
    needs_cover: bool,
    needs_hero: bool,
}

struct SteamMediaSyncGuard;

impl Drop for SteamMediaSyncGuard {
    fn drop(&mut self) {
        STEAM_MEDIA_SYNC_RUNNING.store(false, Ordering::Release);
    }
}

#[tauri::command]
pub async fn sync_steam_owned_game_icons(
    app: AppHandle,
    api_key: String,
    steam64_id: String,
    trigger: Option<String>,
) -> Result<SteamIconSyncSummary, String> {
    sync_steam_media(app, api_key, steam64_id, false, trigger).await
}

#[tauri::command]
pub async fn sync_all_steam_game_media(
    app: AppHandle,
    api_key: String,
    steam64_id: String,
    trigger: Option<String>,
) -> Result<SteamIconSyncSummary, String> {
    sync_steam_media(app, api_key, steam64_id, true, trigger).await
}

/// Last Steam sync outcome, persisted in `app_meta` (DOIT.md §11).
#[derive(Serialize)]
pub struct SteamSyncStatus {
    pub last_synced_at: Option<i64>,
    pub last_error: Option<String>,
    pub last_error_at: Option<i64>,
    pub running: bool,
}

#[tauri::command]
pub fn steam_sync_status() -> Result<SteamSyncStatus, String> {
    let conn = get_db_connection()?;
    let number = |key: &str| -> Result<Option<i64>, String> {
        Ok(get_meta(&conn, key)?.and_then(|value| value.parse().ok()))
    };
    Ok(SteamSyncStatus {
        last_synced_at: number(STEAM_LAST_SYNC_AT)?,
        last_error: get_meta(&conn, STEAM_LAST_ERROR)?,
        last_error_at: number(STEAM_LAST_ERROR_AT)?,
        running: STEAM_MEDIA_SYNC_RUNNING.load(Ordering::Acquire),
    })
}

/// Record the outcome of a sync that actually ran (skips are not recorded).
fn record_sync_outcome(result: &Result<SteamIconSyncSummary, String>) {
    let Ok(conn) = get_db_connection() else {
        return;
    };
    let now = current_unix_seconds().to_string();
    match result {
        Ok(summary) if summary.skipped_reason.is_none() => {
            let _ = set_meta(&conn, STEAM_LAST_SYNC_AT, &now);
            let _ = delete_meta(&conn, STEAM_LAST_ERROR);
            let _ = delete_meta(&conn, STEAM_LAST_ERROR_AT);
        }
        Ok(_) => {}
        Err(error) => {
            let _ = set_meta(&conn, STEAM_LAST_ERROR, error);
            let _ = set_meta(&conn, STEAM_LAST_ERROR_AT, &now);
        }
    }
}

async fn sync_steam_media(
    app: AppHandle,
    api_key: String,
    steam64_id: String,
    force_refresh: bool,
    trigger: Option<String>,
) -> Result<SteamIconSyncSummary, String> {
    let result = sync_steam_media_inner(app, api_key, steam64_id, force_refresh, trigger).await;
    record_sync_outcome(&result);
    result
}

async fn sync_steam_media_inner(
    app: AppHandle,
    api_key: String,
    steam64_id: String,
    force_refresh: bool,
    trigger: Option<String>,
) -> Result<SteamIconSyncSummary, String> {
    let api_key = api_key.trim();
    let steam64_id = steam64_id.trim();
    let trigger = trigger.unwrap_or_else(|| "unknown".to_string());

    if api_key.is_empty() || steam64_id.is_empty() {
        log_warn_event(
            STEAM_MEDIA_EVENT,
            format!(
                "trigger={} action=skip reason=missing_api_key_or_steam64_id",
                trigger
            ),
        );
        return Ok(SteamIconSyncSummary {
            matched_games: 0,
            downloaded_icons: 0,
            downloaded_covers: 0,
            downloaded_heroes: 0,
            updated_games: 0,
            skipped_reason: Some("missing_api_key_or_steam64_id".to_string()),
        });
    }

    if STEAM_MEDIA_SYNC_RUNNING
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        log_warn_event(
            STEAM_MEDIA_EVENT,
            format!(
                "trigger={} action=skip reason=sync_already_running",
                trigger
            ),
        );
        return Ok(SteamIconSyncSummary {
            matched_games: 0,
            downloaded_icons: 0,
            downloaded_covers: 0,
            downloaded_heroes: 0,
            updated_games: 0,
            skipped_reason: Some("sync_already_running".to_string()),
        });
    }
    let _sync_guard = SteamMediaSyncGuard;

    let app_cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to resolve app cache directory: {}", e))?;
    let icon_cache_dir = app_cache_dir.join("steam-icons");
    let library_cache_dir = app_cache_dir.join("steam-library");
    log_info_event(
        STEAM_MEDIA_EVENT,
        format!(
            "trigger={} action=start force_refresh={} cache_dir={} icon_dir={} library_dir={}",
            trigger,
            force_refresh,
            app_cache_dir.display(),
            icon_cache_dir.display(),
            library_cache_dir.display()
        ),
    );
    fs::create_dir_all(&icon_cache_dir)
        .map_err(|e| format!("Failed to create Steam icon cache directory: {}", e))?;
    fs::create_dir_all(&library_cache_dir)
        .map_err(|e| format!("Failed to create Steam library cache directory: {}", e))?;

    let targets = load_games_missing_media(force_refresh)?;
    log_info_event(
        STEAM_MEDIA_EVENT,
        format!(
            "trigger={} action=load_targets target_count={}",
            trigger,
            targets.len()
        ),
    );
    if targets.is_empty() {
        log_info_event(
            STEAM_MEDIA_EVENT,
            format!("trigger={} action=complete reason=no_targets", trigger),
        );
        return Ok(SteamIconSyncSummary {
            matched_games: 0,
            downloaded_icons: 0,
            downloaded_covers: 0,
            downloaded_heroes: 0,
            updated_games: 0,
            skipped_reason: Some("no_targets".to_string()),
        });
    }

    let target_appids = targets
        .iter()
        .map(|target| target.appid)
        .collect::<HashSet<_>>();
    let owned_games = fetch_owned_games(api_key, steam64_id).await?;
    log_info_event(
        STEAM_MEDIA_EVENT,
        format!(
            "trigger={} action=owned_games_response owned_game_count={} target_count={}",
            trigger,
            owned_games.len(),
            target_appids.len()
        ),
    );
    let icon_hashes = owned_games
        .into_iter()
        .filter(|game| target_appids.contains(&game.appid))
        .filter_map(|game| {
            let icon_hash = game.img_icon_url?.trim().to_string();
            if icon_hash.is_empty() {
                None
            } else {
                Some((game.appid, icon_hash))
            }
        })
        .collect::<HashMap<_, _>>();
    log_info_event(
        STEAM_MEDIA_EVENT,
        format!(
            "trigger={} action=match_owned_games matched_icon_hash_count={} target_count={}",
            trigger,
            icon_hashes.len(),
            target_appids.len()
        ),
    );

    let client = steam_http_client();
    let mut downloaded_icons = 0;
    let mut downloaded_covers = 0;
    let mut downloaded_heroes = 0;
    let mut updates = Vec::new();

    for target in targets {
        log_info_event(
            STEAM_MEDIA_EVENT,
            format!(
                "trigger={} action=process_game game_id={} appid={} name=\"{}\" needs_icon={} needs_cover={} needs_hero={}",
                trigger,
                target.id,
                target.appid,
                sanitize_log_value(&target.name),
                target.needs_icon,
                target.needs_cover,
                target.needs_hero
            ),
        );

        let mut icon_path = None;
        let mut cover_path = None;
        let mut hero_path = None;

        if target.needs_icon {
            if let Some(icon_hash) = icon_hashes.get(&target.appid) {
                let next_icon_path =
                    icon_cache_dir.join(format!("{}_{}.jpg", target.appid, icon_hash));
                let should_skip_icon = should_skip_media_download(target.appid, "icon")?;
                if next_icon_path.exists() && !force_refresh {
                    log_info_event(
                        STEAM_MEDIA_EVENT,
                        format!(
                            "trigger={} action=asset_skip appid={} name=\"{}\" asset=icon reason=file_already_cached path={}",
                            trigger,
                            target.appid,
                            sanitize_log_value(&target.name),
                            next_icon_path.display()
                        ),
                    );
                } else if should_skip_icon && !force_refresh {
                    log_warn_event(
                        STEAM_MEDIA_EVENT,
                        format!(
                            "trigger={} action=asset_skip appid={} name=\"{}\" asset=icon reason=failure_cooldown",
                            trigger,
                            target.appid,
                            sanitize_log_value(&target.name)
                        ),
                    );
                } else {
                    let url = format!(
                        "https://media.steampowered.com/steamcommunity/public/images/apps/{}/{}.jpg",
                        target.appid, icon_hash
                    );
                    log_info_event(
                        STEAM_MEDIA_EVENT,
                        format!(
                            "trigger={} action=asset_download_start appid={} name=\"{}\" asset=icon url={} path={}",
                            trigger,
                            target.appid,
                            sanitize_log_value(&target.name),
                            url,
                            next_icon_path.display()
                        ),
                    );
                    match download_media(&client, &url, &next_icon_path).await {
                        Err(error) => {
                            eprintln!(
                                "Failed to cache Steam icon for appid {}: {}",
                                target.appid, error
                            );
                            log_error_event(
                                STEAM_MEDIA_EVENT,
                                format!(
                                    "trigger={} action=asset_download_failed appid={} name=\"{}\" asset=icon url={} error={}",
                                    trigger,
                                    target.appid,
                                    sanitize_log_value(&target.name),
                                    url,
                                    error
                                ),
                            );
                            record_media_failure(target.appid, "icon", &error)?;
                        }
                        Ok(bytes_written) => {
                            clear_media_failure(target.appid, "icon")?;
                            log_info_event(
                                STEAM_MEDIA_EVENT,
                                format!(
                                    "trigger={} action=asset_download_success appid={} name=\"{}\" asset=icon bytes={} path={}",
                                    trigger,
                                    target.appid,
                                    sanitize_log_value(&target.name),
                                    bytes_written,
                                    next_icon_path.display()
                                ),
                            );
                            downloaded_icons += 1;
                        }
                    }
                }

                if next_icon_path.exists() {
                    icon_path = Some(next_icon_path.to_string_lossy().to_string());
                }
            } else {
                log_warn_event(
                    STEAM_MEDIA_EVENT,
                    format!(
                        "trigger={} action=asset_skip appid={} name=\"{}\" asset=icon reason=missing_icon_hash_from_owned_games_response",
                        trigger,
                        target.appid,
                        sanitize_log_value(&target.name)
                    ),
                );
            }
        }

        if target.needs_cover {
            let next_cover_path =
                library_cache_dir.join(format!("{}_library_600x900.jpg", target.appid));
            let should_skip_cover = should_skip_media_download(target.appid, "cover")?;
            if next_cover_path.exists() && !force_refresh {
                log_info_event(
                    STEAM_MEDIA_EVENT,
                    format!(
                        "trigger={} action=asset_skip appid={} name=\"{}\" asset=cover reason=file_already_cached path={}",
                        trigger,
                        target.appid,
                        sanitize_log_value(&target.name),
                        next_cover_path.display()
                    ),
                );
            } else if should_skip_cover && !force_refresh {
                log_warn_event(
                    STEAM_MEDIA_EVENT,
                    format!(
                        "trigger={} action=asset_skip appid={} name=\"{}\" asset=cover reason=failure_cooldown",
                        trigger,
                        target.appid,
                        sanitize_log_value(&target.name)
                    ),
                );
            } else {
                let url = format!(
                    "https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900.jpg",
                    target.appid
                );
                log_info_event(
                    STEAM_MEDIA_EVENT,
                    format!(
                        "trigger={} action=asset_download_start appid={} name=\"{}\" asset=cover url={} path={}",
                        trigger,
                        target.appid,
                        sanitize_log_value(&target.name),
                        url,
                        next_cover_path.display()
                    ),
                );
                match download_media(&client, &url, &next_cover_path).await {
                    Err(error) => {
                        eprintln!(
                            "Failed to cache Steam library image for appid {}: {}",
                            target.appid, error
                        );
                        log_error_event(
                            STEAM_MEDIA_EVENT,
                            format!(
                                "trigger={} action=asset_download_failed appid={} name=\"{}\" asset=cover url={} error={}",
                                trigger,
                                target.appid,
                                sanitize_log_value(&target.name),
                                url,
                                error
                            ),
                        );
                        record_media_failure(target.appid, "cover", &error)?;
                    }
                    Ok(bytes_written) => {
                        clear_media_failure(target.appid, "cover")?;
                        log_info_event(
                            STEAM_MEDIA_EVENT,
                            format!(
                                "trigger={} action=asset_download_success appid={} name=\"{}\" asset=cover bytes={} path={}",
                                trigger,
                                target.appid,
                                sanitize_log_value(&target.name),
                                bytes_written,
                                next_cover_path.display()
                            ),
                        );
                        downloaded_covers += 1;
                    }
                }
            }

            if next_cover_path.exists() {
                cover_path = Some(next_cover_path.to_string_lossy().to_string());
            }
        }

        if target.needs_hero {
            let next_hero_path =
                library_cache_dir.join(format!("{}_library_hero.jpg", target.appid));
            let should_skip_hero = should_skip_media_download(target.appid, "hero")?;
            if next_hero_path.exists() && !force_refresh {
                log_info_event(
                    STEAM_MEDIA_EVENT,
                    format!(
                        "trigger={} action=asset_skip appid={} name=\"{}\" asset=hero reason=file_already_cached path={}",
                        trigger,
                        target.appid,
                        sanitize_log_value(&target.name),
                        next_hero_path.display()
                    ),
                );
            } else if should_skip_hero && !force_refresh {
                log_warn_event(
                    STEAM_MEDIA_EVENT,
                    format!(
                        "trigger={} action=asset_skip appid={} name=\"{}\" asset=hero reason=failure_cooldown",
                        trigger,
                        target.appid,
                        sanitize_log_value(&target.name)
                    ),
                );
            } else {
                let url = format!(
                    "https://cdn.akamai.steamstatic.com/steam/apps/{}/library_hero.jpg",
                    target.appid
                );
                log_info_event(
                    STEAM_MEDIA_EVENT,
                    format!(
                        "trigger={} action=asset_download_start appid={} name=\"{}\" asset=hero url={} path={}",
                        trigger,
                        target.appid,
                        sanitize_log_value(&target.name),
                        url,
                        next_hero_path.display()
                    ),
                );
                match download_media(&client, &url, &next_hero_path).await {
                    Err(error) => {
                        eprintln!(
                            "Failed to cache Steam hero image for appid {}: {}",
                            target.appid, error
                        );
                        log_error_event(
                            STEAM_MEDIA_EVENT,
                            format!(
                                "trigger={} action=asset_download_failed appid={} name=\"{}\" asset=hero url={} error={}",
                                trigger,
                                target.appid,
                                sanitize_log_value(&target.name),
                                url,
                                error
                            ),
                        );
                        record_media_failure(target.appid, "hero", &error)?;
                    }
                    Ok(bytes_written) => {
                        clear_media_failure(target.appid, "hero")?;
                        log_info_event(
                            STEAM_MEDIA_EVENT,
                            format!(
                                "trigger={} action=asset_download_success appid={} name=\"{}\" asset=hero bytes={} path={}",
                                trigger,
                                target.appid,
                                sanitize_log_value(&target.name),
                                bytes_written,
                                next_hero_path.display()
                            ),
                        );
                        downloaded_heroes += 1;
                    }
                }
            }

            if next_hero_path.exists() {
                hero_path = Some(next_hero_path.to_string_lossy().to_string());
            }
        }

        if icon_path.is_some() || cover_path.is_some() || hero_path.is_some() {
            log_info_event(
                STEAM_MEDIA_EVENT,
                format!(
                    "trigger={} action=game_update_queued appid={} name=\"{}\" has_icon_path={} has_cover_path={} has_hero_path={}",
                    trigger,
                    target.appid,
                    sanitize_log_value(&target.name),
                    icon_path.is_some(),
                    cover_path.is_some(),
                    hero_path.is_some()
                ),
            );
            updates.push(GameMediaUpdate {
                id: target.id,
                icon_path,
                cover_path,
                hero_path,
            });
        }
    }

    let updated_games = update_game_media_paths(&updates)?;
    log_info_event(
        STEAM_MEDIA_EVENT,
        format!(
        "trigger={} action=complete matched={} downloaded_icons={} downloaded_covers={} downloaded_heroes={} queued_game_updates={} updated_db_rows={}",
        trigger,
        icon_hashes.len(),
        downloaded_icons,
        downloaded_covers,
        downloaded_heroes,
        updates.len(),
        updated_games
    ));

    Ok(SteamIconSyncSummary {
        matched_games: icon_hashes.len(),
        downloaded_icons,
        downloaded_covers,
        downloaded_heroes,
        updated_games,
        skipped_reason: None,
    })
}

fn load_games_missing_media(force_refresh: bool) -> Result<Vec<LocalGameMediaTarget>, String> {
    let conn = get_db_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, appid, name, icon_path, cover_path, hero_path
             FROM games
             WHERE appid IS NOT NULL
               AND is_hidden = 0
               AND NOT EXISTS (
                   SELECT 1 FROM ignored_games ignored WHERE ignored.appid = games.appid
               )",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut targets = Vec::new();
    for row in rows {
        let (id, appid, name, icon_path, cover_path, hero_path) = row.map_err(|e| e.to_string())?;
        let has_icon = icon_path
            .as_deref()
            .map(|path| Path::new(path).exists())
            .unwrap_or(false);
        let has_cover = cover_path
            .as_deref()
            .map(|path| Path::new(path).exists())
            .unwrap_or(false);
        let has_hero = hero_path
            .as_deref()
            .map(|path| Path::new(path).exists())
            .unwrap_or(false);

        if force_refresh || !has_icon || !has_cover || !has_hero {
            targets.push(LocalGameMediaTarget {
                id,
                appid,
                name,
                needs_icon: force_refresh || !has_icon,
                needs_cover: force_refresh || !has_cover,
                needs_hero: force_refresh || !has_hero,
            });
        }
    }

    Ok(targets)
}

async fn fetch_owned_games(api_key: &str, steam64_id: &str) -> Result<Vec<OwnedGame>, String> {
    log_info_event(
        STEAM_MEDIA_EVENT,
        format!(
            "action=owned_games_request steam64_id_suffix={}",
            mask_steam64_id(steam64_id)
        ),
    );
    let response = steam_http_client()
        .get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/")
        .query(&[
            ("key", api_key),
            ("steamid", steam64_id),
            ("include_appinfo", "true"),
            ("include_played_free_games", "true"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| {
            redact_key(
                format!("Failed to reach Steam: {}", describe_request_error(e)),
                api_key,
            )
        })?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(format!(
            "Steam rejected the API key ({}). Check the key in Settings.",
            status.as_u16()
        ));
    }
    if !status.is_success() {
        return Err(format!("Steam returned an error ({})", status));
    }

    let payload = response.json::<OwnedGamesPayload>().await.map_err(|e| {
        redact_key(
            format!(
                "Failed to read Steam's response: {}",
                describe_request_error(e)
            ),
            api_key,
        )
    })?;

    Ok(payload.response.games.unwrap_or_default())
}

async fn download_media(client: &Client, url: &str, path: &Path) -> Result<usize, String> {
    let response = client.get(url).send().await.map_err(|e| {
        format!(
            "Failed to request Steam media {}: {}",
            url,
            describe_request_error(e)
        )
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Steam media download failed with HTTP {}", status));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read Steam media bytes from {}: {}", url, e))?;

    let byte_count = bytes.len();
    fs::write(path, bytes).map_err(|e| format!("Failed to write Steam media cache: {}", e))?;
    Ok(byte_count)
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn should_skip_media_download(appid: i32, asset_type: &str) -> Result<bool, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    let retry_after = conn
        .query_row(
            "SELECT retry_after FROM steam_media_failures WHERE appid = ?1 AND asset_type = ?2",
            params![appid, asset_type],
            |row| row.get::<_, i64>(0),
        )
        .ok();

    Ok(retry_after.map(|value| value > now).unwrap_or(false))
}

fn record_media_failure(appid: i32, asset_type: &str, error: &str) -> Result<(), String> {
    let conn = get_db_connection()?;
    let failed_at = current_unix_seconds();
    let retry_after = failed_at + MEDIA_FAILURE_RETRY_SECONDS;

    conn.execute(
        "INSERT INTO steam_media_failures (appid, asset_type, failed_at, retry_after, last_error)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(appid, asset_type) DO UPDATE SET
         failed_at = excluded.failed_at,
         retry_after = excluded.retry_after,
         last_error = excluded.last_error",
        params![appid, asset_type, failed_at, retry_after, error],
    )
    .map_err(|e| e.to_string())?;

    log_warn_event(
        STEAM_MEDIA_EVENT,
        format!(
            "action=record_failure appid={} asset={} retry_after={} error={}",
            appid, asset_type, retry_after, error
        ),
    );

    Ok(())
}

fn clear_media_failure(appid: i32, asset_type: &str) -> Result<(), String> {
    let conn = get_db_connection()?;
    conn.execute(
        "DELETE FROM steam_media_failures WHERE appid = ?1 AND asset_type = ?2",
        params![appid, asset_type],
    )
    .map_err(|e| e.to_string())?;

    log_info_event(
        STEAM_MEDIA_EVENT,
        format!("action=clear_failure appid={} asset={}", appid, asset_type),
    );

    Ok(())
}

struct GameMediaUpdate {
    id: i32,
    icon_path: Option<String>,
    cover_path: Option<String>,
    hero_path: Option<String>,
}

fn update_game_media_paths(updates: &[GameMediaUpdate]) -> Result<usize, String> {
    if updates.is_empty() {
        return Ok(0);
    }

    let mut conn = get_db_connection()?;
    let transaction = conn.transaction().map_err(|e| e.to_string())?;
    let mut updated = 0;

    for update in updates {
        if let Some(icon_path) = &update.icon_path {
            log_info_event(
                STEAM_MEDIA_EVENT,
                format!(
                    "action=db_update_start game_id={} asset=icon path={}",
                    update.id, icon_path
                ),
            );
            updated += transaction
                .execute(
                    "UPDATE games SET icon_path = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
                    params![icon_path, update.id],
                )
                .map_err(|e| e.to_string())?;
        }

        if let Some(cover_path) = &update.cover_path {
            log_info_event(
                STEAM_MEDIA_EVENT,
                format!(
                    "action=db_update_start game_id={} asset=cover path={}",
                    update.id, cover_path
                ),
            );
            updated += transaction
                .execute(
                    "UPDATE games SET cover_path = ?1, accent_color = NULL, updated_at = strftime('%s', 'now') WHERE id = ?2",
                    params![cover_path, update.id],
                )
                .map_err(|e| e.to_string())?;
        }

        if let Some(hero_path) = &update.hero_path {
            log_info_event(
                STEAM_MEDIA_EVENT,
                format!(
                    "action=db_update_start game_id={} asset=hero path={}",
                    update.id, hero_path
                ),
            );
            updated += transaction
                .execute(
                    "UPDATE games SET hero_path = ?1, accent_color = NULL, updated_at = strftime('%s', 'now') WHERE id = ?2",
                    params![hero_path, update.id],
                )
                .map_err(|e| e.to_string())?;
        }
    }

    transaction.commit().map_err(|e| e.to_string())?;
    Ok(updated)
}

fn sanitize_log_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn mask_steam64_id(steam64_id: &str) -> String {
    let suffix = steam64_id
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("***{}", suffix)
}

#[cfg(test)]
mod request_error_tests {
    use super::*;

    #[test]
    fn connect_failure_names_the_cause_and_never_the_key() {
        let key = "0123456789ABCDEF0123456789ABCDEF";
        let message = tauri::async_runtime::block_on(async {
            let error = steam_http_client()
                .get("http://127.0.0.1:9/IPlayerService/GetOwnedGames/v1/")
                .query(&[("key", key)])
                .send()
                .await
                .unwrap_err();
            redact_key(describe_request_error(error), key)
        });
        assert!(!message.contains(key), "{message}");
        assert!(message.contains("could not connect"), "{message}");
        println!("{message}");
    }

    #[test]
    fn redact_key_masks_the_key() {
        let key = "0123456789ABCDEF0123456789ABCDEF";
        assert_eq!(redact_key(format!("x key={key} y"), key), "x key=*** y");
    }
}
