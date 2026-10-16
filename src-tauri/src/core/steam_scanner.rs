use rusqlite::Result;
use std::collections::HashSet;
use std::time::SystemTime;
use steamlocate::SteamDir;

use crate::core::process_watcher::request_process_match_index_refresh;
use crate::storage::db::get_db_connection;

#[tauri::command]
pub fn steam_scan_print() -> Result<(), String> {
    println!("=== Starting Steam scan ===");

    let steam_dir = SteamDir::locate().map_err(|e| format!("Failed to locate Steam: {}", e))?;
    println!(
        "Steam installation found at - {}",
        steam_dir.path().display()
    );

    // 打开数据库连接
    let mut conn = get_db_connection()?;
    println!("Database connection established");

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    let ignored_appids = {
        let mut stmt = tx
            .prepare("SELECT appid FROM ignored_games")
            .map_err(|e| format!("Failed to prepare ignored games query: {}", e))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, i32>(0))
            .map_err(|e| format!("Failed to query ignored games: {}", e))?;
        rows.collect::<Result<HashSet<_>, _>>()
            .map_err(|e| format!("Failed to read ignored games: {}", e))?
    };

    // 先将所有游戏标记为未安装，扫描时会重新标记为已安装
    tx.execute(
        "UPDATE games SET is_installed = 0, install_path = NULL WHERE source = 'steam';",
        [],
    )
    .map_err(|e| format!("Failed to mark games as not installed: {}", e))?;
    println!("Marked all games as not installed");

    let mut upsert_stmt = tx
        .prepare(
            "INSERT INTO games (appid, source, name, install_path, is_installed, created_at, updated_at)
             VALUES (?1, 'steam', ?2, ?3, 1, ?4, ?5)
             ON CONFLICT(appid) DO UPDATE SET
             source = 'steam',
             name = excluded.name,
             install_path = excluded.install_path,
             is_installed = excluded.is_installed,
             updated_at = excluded.updated_at",
        )
        .map_err(|e| format!("Failed to prepare upsert statement: {}", e))?;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;

    let libraries = steam_dir
        .libraries()
        .map_err(|e| format!("Failed to get libraries: {}", e))?;

    let mut game_count = 0;
    let mut library_count = 0;

    for library in libraries {
        library_count += 1;
        let library = library.map_err(|e| format!("Failed to get library: {}", e))?;
        println!("Processing library - {}", library.path().display());

        let apps = library.apps();
        for app in apps {
            let app = app.map_err(|e| format!("Failed to get app: {}", e))?;
            let appid = app.app_id as i32;
            if ignored_appids.contains(&appid) {
                continue;
            }

            let name = app.name.as_deref().unwrap_or("<unknown>");
            let install_path = library.resolve_app_dir(&app);
            let install_path_str = install_path.to_str().unwrap_or("");

            game_count += 1;

            // 插入或更新游戏信息到数据库
            let result = upsert_stmt.execute((appid, name, install_path_str, timestamp, timestamp));

            if let Err(e) = result {
                println!("    ✗ Error saving to database: {}", e);
            }
        }
    }

    drop(upsert_stmt);
    tx.commit()
        .map_err(|e| format!("Failed to commit Steam scan transaction: {}", e))?;
    request_process_match_index_refresh();

    println!("=== Steam scan completed ===");
    println!(
        "Found {} libraries and {} games\n",
        library_count, game_count
    );

    Ok(())
}
