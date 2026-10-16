use rusqlite::{Connection, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::storage::db::get_db_path;
use crate::system::settings::{
    hydrate_runtime_settings, process_scan_intervals, settings_runtime_generation,
};

const MATCH_RELOAD_INTERVAL: Duration = Duration::from_secs(60);

static PROCESS_MATCH_INDEX_REFRESH_GENERATION: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct GameSession {
    pub game_id: i32,
    pub start_time: i64,
    pub session_id: Option<i32>,
}

#[derive(Debug, Clone)]
struct GameProcessMatch {
    game_id: i32,
    path: PathBuf,
    exact: bool,
}

pub struct ProcessWatcher {
    db_path: PathBuf,
    game_matches: Arc<Mutex<Vec<GameProcessMatch>>>,
    active_sessions: Arc<Mutex<HashMap<i32, GameSession>>>,
}

pub fn request_process_match_index_refresh() {
    PROCESS_MATCH_INDEX_REFRESH_GENERATION.fetch_add(1, Ordering::Release);
}

impl ProcessWatcher {
    fn open_connection(db_path: &Path) -> Result<Connection, String> {
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| e.to_string())?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| e.to_string())?;
        conn.busy_timeout(Duration::from_secs(3))
            .map_err(|e| e.to_string())?;
        Ok(conn)
    }

    pub fn new(db_path: &Path) -> Result<Self, String> {
        let _ = ProcessWatcher::open_connection(db_path)?;

        Ok(Self {
            db_path: db_path.to_path_buf(),
            game_matches: Arc::new(Mutex::new(Vec::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn init_game_paths(&mut self) -> Result<(), String> {
        println!("Initializing game process match index...");

        let conn = ProcessWatcher::open_connection(&self.db_path)?;
        let next_matches = ProcessWatcher::load_game_process_matches(&conn)?;
        let mut game_matches = self.game_matches.lock().unwrap();
        *game_matches = next_matches;

        println!(
            "Game process match index initialized with {} rules",
            game_matches.len()
        );
        Ok(())
    }

    fn load_game_process_matches(conn: &Connection) -> Result<Vec<GameProcessMatch>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, install_path, executable_path
                 FROM games
                 WHERE is_installed = 1 AND is_hidden = 0",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let id: i32 = row.get(0)?;
                let install_path: Option<String> = row.get(1)?;
                let executable_path: Option<String> = row.get(2)?;
                Ok((id, install_path, executable_path))
            })
            .map_err(|e| e.to_string())?;

        let mut matches = Vec::new();
        for row in rows {
            let (game_id, install_path, executable_path) = row.map_err(|e| e.to_string())?;

            if let Some(executable_path) = executable_path.filter(|value| !value.trim().is_empty())
            {
                matches.push(GameProcessMatch {
                    game_id,
                    path: ProcessWatcher::normalize_path_for_match(&PathBuf::from(executable_path)),
                    exact: true,
                });
                continue;
            }

            if let Some(install_path) = install_path.filter(|value| !value.trim().is_empty()) {
                matches.push(GameProcessMatch {
                    game_id,
                    path: ProcessWatcher::normalize_path_for_match(&PathBuf::from(install_path)),
                    exact: false,
                });
            }
        }

        Ok(matches)
    }

    pub fn recover_active_sessions(&mut self) -> Result<(), String> {
        println!("Recovering open game sessions...");

        let conn = ProcessWatcher::open_connection(&self.db_path)?;
        let mut active_sessions = self.active_sessions.lock().unwrap();
        active_sessions.clear();

        let mut stmt = conn
            .prepare(
                "SELECT id, game_id, start_time
                 FROM game_sessions
                 WHERE end_time IS NULL
                 ORDER BY game_id ASC, start_time ASC, id ASC",
            )
            .map_err(|e| e.to_string())?;

        let open_sessions = stmt
            .query_map([], |row| {
                Ok(GameSession {
                    session_id: Some(row.get(0)?),
                    game_id: row.get(1)?,
                    start_time: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for session in open_sessions {
            let session = session.map_err(|e| e.to_string())?;

            if active_sessions.contains_key(&session.game_id) {
                ProcessWatcher::close_duplicate_open_session(&conn, &session)?;
                println!(
                    "[Process Watcher] Closed duplicate open session: game_id={}, session_id={:?}",
                    session.game_id, session.session_id
                );
                continue;
            }

            println!(
                "[Process Watcher] Recovered open session: game_id={}, session_id={:?}",
                session.game_id, session.session_id
            );
            active_sessions.insert(session.game_id, session);
        }

        println!(
            "Recovered {} active sessions from database",
            active_sessions.len()
        );
        Ok(())
    }

    pub fn start_monitoring(&mut self) {
        println!("Starting process monitor...");

        let db_path = self.db_path.clone();
        let game_matches = self.game_matches.clone();
        let active_sessions = self.active_sessions.clone();

        thread::spawn(move || {
            let mut system = System::new();
            // A process's executable never changes: ask the OS for it once per process.
            let refresh_kind = ProcessRefreshKind::nothing()
                .with_exe(UpdateKind::OnlyIfNotSet)
                .without_tasks();
            // pid -> (exe as reported, normalised exe). Normalising canonicalises the path
            // (a filesystem call), so it is done once per process, not on every scan.
            let mut exe_cache: HashMap<Pid, (PathBuf, PathBuf)> = HashMap::new();
            let mut last_match_reload = Instant::now();
            let mut observed_refresh_generation =
                PROCESS_MATCH_INDEX_REFRESH_GENERATION.load(Ordering::Acquire);
            let mut observed_settings_generation = settings_runtime_generation();
            let mut fast_idle_checks_remaining = 0_u8;
            let (mut idle_scan_seconds, mut active_scan_seconds) = process_scan_intervals();

            loop {
                let current_refresh_generation =
                    PROCESS_MATCH_INDEX_REFRESH_GENERATION.load(Ordering::Acquire);
                let should_reload_matches = last_match_reload.elapsed() >= MATCH_RELOAD_INTERVAL
                    || current_refresh_generation != observed_refresh_generation;

                let has_active_sessions = ProcessWatcher::check_processes(
                    &db_path,
                    &game_matches,
                    &active_sessions,
                    &mut system,
                    &mut exe_cache,
                    refresh_kind,
                    should_reload_matches,
                );

                if should_reload_matches {
                    last_match_reload = Instant::now();
                    observed_refresh_generation = current_refresh_generation;
                }

                let current_settings_generation = settings_runtime_generation();
                if current_settings_generation != observed_settings_generation {
                    (idle_scan_seconds, active_scan_seconds) = process_scan_intervals();
                    observed_settings_generation = current_settings_generation;
                }

                let next_interval = if has_active_sessions {
                    fast_idle_checks_remaining = 2;
                    Duration::from_secs(active_scan_seconds as u64)
                } else if fast_idle_checks_remaining > 0 {
                    fast_idle_checks_remaining -= 1;
                    Duration::from_secs(active_scan_seconds as u64)
                } else {
                    Duration::from_secs(idle_scan_seconds as u64)
                };

                thread::sleep(next_interval);
            }
        });
    }

    fn check_processes(
        db_path: &Path,
        game_matches: &Arc<Mutex<Vec<GameProcessMatch>>>,
        active_sessions: &Arc<Mutex<HashMap<i32, GameSession>>>,
        system: &mut System,
        exe_cache: &mut HashMap<Pid, (PathBuf, PathBuf)>,
        refresh_kind: ProcessRefreshKind,
        should_reload_matches: bool,
    ) -> bool {
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind);

        if should_reload_matches {
            match ProcessWatcher::open_connection(db_path)
                .and_then(|conn| ProcessWatcher::load_game_process_matches(&conn))
            {
                Ok(next_matches) => {
                    let mut game_matches = game_matches.lock().unwrap();
                    *game_matches = next_matches;
                }
                Err(error) => {
                    println!(
                        "[Process Watcher] Failed to refresh game process match index: {}",
                        error
                    );
                }
            }
        }

        let indexed_matches = {
            let game_matches = game_matches.lock().unwrap();
            game_matches.clone()
        };

        let mut running_games = HashSet::new();
        let processes = system.processes();
        // Forget processes that have exited (and keep the cache bounded).
        exe_cache.retain(|pid, _| processes.contains_key(pid));
        if !indexed_matches.is_empty() {
            for (pid, process) in processes {
                let Some(exe_path) = process.exe() else {
                    continue;
                };
                let cached = exe_cache.entry(*pid).or_insert_with(|| {
                    (
                        exe_path.to_path_buf(),
                        ProcessWatcher::normalize_path_for_match(exe_path),
                    )
                });
                // A reused pid belongs to a different program: normalise again.
                if cached.0 != exe_path {
                    *cached = (
                        exe_path.to_path_buf(),
                        ProcessWatcher::normalize_path_for_match(exe_path),
                    );
                }
                if let Some(rule) = indexed_matches
                    .iter()
                    .find(|rule| ProcessWatcher::normalized_path_matches(&cached.1, rule))
                {
                    running_games.insert(rule.game_id);
                }
            }
        }

        let (new_games, ended_game_ids) = {
            let active_sessions = active_sessions.lock().unwrap();
            let new_games = running_games
                .iter()
                .copied()
                .filter(|game_id| !active_sessions.contains_key(game_id))
                .collect::<Vec<_>>();

            let ended_game_ids = active_sessions
                .keys()
                .copied()
                .filter(|game_id| !running_games.contains(game_id))
                .collect::<Vec<_>>();

            (new_games, ended_game_ids)
        };

        if new_games.is_empty() && ended_game_ids.is_empty() {
            return !running_games.is_empty()
                || active_sessions
                    .lock()
                    .map(|sessions| !sessions.is_empty())
                    .unwrap_or(false);
        }

        let conn = match ProcessWatcher::open_connection(db_path) {
            Ok(conn) => conn,
            Err(error) => {
                println!("[Process Watcher] Failed to open database: {}", error);
                return !running_games.is_empty()
                    || active_sessions
                        .lock()
                        .map(|sessions| !sessions.is_empty())
                        .unwrap_or(false);
            }
        };

        for game_id in new_games {
            match ProcessWatcher::start_session(&conn, game_id) {
                Ok(session_id) => {
                    let start_time = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    let mut active_sessions = active_sessions.lock().unwrap();
                    active_sessions.insert(
                        game_id,
                        GameSession {
                            game_id,
                            start_time: start_time as i64,
                            session_id: Some(session_id),
                        },
                    );
                }
                Err(e) => {
                    println!(
                        "[Process Watcher] Failed to save session (ID: {}) - {}",
                        game_id, e
                    );
                }
            }
        }

        let ended_sessions = {
            let mut active_sessions = active_sessions.lock().unwrap();
            let mut ended_sessions = Vec::with_capacity(ended_game_ids.len());

            for game_id in ended_game_ids {
                if let Some(session) = active_sessions.remove(&game_id) {
                    ended_sessions.push(session);
                }
            }

            ended_sessions
        };

        for session in ended_sessions {
            ProcessWatcher::end_session(&conn, session);
        }

        active_sessions
            .lock()
            .map(|sessions| !sessions.is_empty())
            .unwrap_or(false)
    }

    /// `exe` must already be normalised with `normalize_path_for_match`.
    fn normalized_path_matches(exe: &Path, rule: &GameProcessMatch) -> bool {
        if rule.exact {
            exe == rule.path
        } else {
            exe.starts_with(&rule.path)
        }
    }

    fn normalize_path_for_match(path: &Path) -> PathBuf {
        let normalized = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .replace('/', "\\");

        if cfg!(target_os = "windows") {
            PathBuf::from(normalized.to_lowercase())
        } else {
            PathBuf::from(normalized)
        }
    }

    fn start_session(conn: &Connection, game_id: i32) -> Result<i32, String> {
        let start_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut stmt = conn
            .prepare("INSERT INTO game_sessions (game_id, start_time) VALUES (?1, ?2)")
            .map_err(|e| e.to_string())?;
        stmt.execute((game_id, start_time as i64))
            .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid() as i32)
    }

    fn close_duplicate_open_session(
        conn: &Connection,
        session: &GameSession,
    ) -> Result<(), String> {
        if let Some(session_id) = session.session_id {
            conn.execute(
                "UPDATE game_sessions
                 SET end_time = ?1, duration_seconds = 0
                 WHERE id = ?2 AND end_time IS NULL",
                (session.start_time, session_id),
            )
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn end_session(conn: &Connection, session: GameSession) {
        let end_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;

        let duration_seconds = (end_time - session.start_time).max(0) as i32;

        if let Some(session_id) = session.session_id {
            let _ = conn.execute(
                "UPDATE game_sessions SET end_time = ?1, duration_seconds = ?2 WHERE id = ?3",
                (end_time, duration_seconds, session_id),
            );
        }

        ProcessWatcher::update_game_stats(conn, session.game_id, duration_seconds, end_time);
    }

    fn update_game_stats(
        conn: &Connection,
        game_id: i32,
        duration_seconds: i32,
        last_played_at: i64,
    ) {
        let mut stmt = match conn
            .prepare("SELECT total_playtime_seconds FROM game_stats WHERE game_id = ?1")
        {
            Ok(stmt) => stmt,
            Err(error) => {
                println!(
                    "[Process Watcher] Failed to prepare game stats query: {}",
                    error
                );
                return;
            }
        };
        let result = stmt.query_row((game_id,), |row| row.get::<_, i32>(0)).ok();

        match result {
            Some(current_playtime) => {
                let new_playtime = current_playtime + duration_seconds;
                let _ = conn.execute(
                    "UPDATE game_stats SET total_playtime_seconds = ?1, last_played_at = ?2 WHERE game_id = ?3",
                    (new_playtime, last_played_at, game_id),
                );
            }
            None => {
                let _ = conn.execute(
                    "INSERT INTO game_stats (game_id, total_playtime_seconds, last_played_at) VALUES (?1, ?2, ?3)",
                    (game_id, duration_seconds, last_played_at),
                );
            }
        }
    }
}

#[tauri::command]
pub fn start_process_monitor() -> Result<(), String> {
    let db_path = get_db_path()?;
    if let Err(error) = hydrate_runtime_settings() {
        println!(
            "[Process Watcher] Failed to load process scan interval settings, using defaults: {}",
            error
        );
    }

    match ProcessWatcher::new(&db_path) {
        Ok(mut watcher) => {
            if let Err(e) = watcher.init_game_paths() {
                return Err(format!("Failed to initialize game paths: {}", e));
            }

            if let Err(e) = watcher.recover_active_sessions() {
                return Err(format!("Failed to recover active sessions: {}", e));
            }

            watcher.start_monitoring();
            Ok(())
        }
        Err(e) => Err(format!("Failed to start process monitor: {}", e)),
    }
}
