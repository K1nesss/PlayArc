//! One-time carry-over from the app's previous name (Velaris → PlayArc).
//!
//! User data lived in `<data dir>/Velaris`. On Windows that folder can also be
//! the old per-user install folder (NSIS installs to `%LOCALAPPDATA%\<productName>`),
//! so it is never moved or deleted: only the data files are copied into
//! `<data dir>/PlayArc`, and only when PlayArc has no database yet.

use std::fs;
use std::path::Path;

use crate::storage::db::resolve_app_data_dir;

const LEGACY_DIR_NAME: &str = "Velaris";
const CURRENT_DIR_NAME: &str = "PlayArc";

/// Files that make up the user's data. The database's WAL/SHM siblings are
/// copied with it so uncheckpointed writes are not lost.
const DATA_FILES: &[&str] = &[
    "playtime-tracker.db",
    "playtime-tracker.db-wal",
    "playtime-tracker.db-shm",
    "settings.json",
    "window-state.json",
];

/// Copy data from the legacy folder if this is the first launch under the new
/// name. Never fails the startup: problems are reported and the app continues
/// (with a fresh data folder, the old data still intact for a later attempt).
pub fn migrate_legacy_data_dir() {
    let Ok(base) = resolve_app_data_dir() else {
        return;
    };
    if let Err(error) = migrate(&base.join(LEGACY_DIR_NAME), &base.join(CURRENT_DIR_NAME)) {
        eprintln!(
            "Failed to carry over data from {}: {}",
            LEGACY_DIR_NAME, error
        );
    }
}

fn migrate(legacy: &Path, current: &Path) -> Result<(), String> {
    if !legacy.join(DATA_FILES[0]).is_file() || current.join(DATA_FILES[0]).exists() {
        return Ok(());
    }
    fs::create_dir_all(current).map_err(|e| e.to_string())?;

    // Copy into temporary names first, then rename, so a half-finished copy is
    // never mistaken for a migrated database on the next launch.
    let mut copied = Vec::new();
    for name in DATA_FILES {
        let source = legacy.join(name);
        if !source.is_file() {
            continue;
        }
        let staging = current.join(format!("{}.migrating", name));
        if let Err(error) = fs::copy(&source, &staging) {
            for (staged, _) in &copied {
                let _ = fs::remove_file(staged);
            }
            let _ = fs::remove_file(&staging);
            return Err(format!("copy {}: {}", name, error));
        }
        copied.push((staging, current.join(name)));
    }

    // The database goes last: its presence is what marks the migration as done.
    copied.sort_by_key(|(_, target)| target.ends_with(DATA_FILES[0]));
    for (staging, target) in copied {
        fs::rename(&staging, &target).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("playarc-migrate-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copies_data_and_leaves_legacy_untouched() {
        let base = temp_dir("copy");
        let legacy = base.join(LEGACY_DIR_NAME);
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("playtime-tracker.db"), b"db").unwrap();
        fs::write(legacy.join("settings.json"), b"{}").unwrap();
        fs::write(legacy.join("Velaris.exe"), b"exe").unwrap();

        migrate(&legacy, &base.join(CURRENT_DIR_NAME)).unwrap();

        let current = base.join(CURRENT_DIR_NAME);
        assert_eq!(
            fs::read(current.join("playtime-tracker.db")).unwrap(),
            b"db"
        );
        assert_eq!(fs::read(current.join("settings.json")).unwrap(), b"{}");
        assert!(!current.join("Velaris.exe").exists());
        assert!(!current.join("settings.json.migrating").exists());
        assert!(legacy.join("playtime-tracker.db").exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn never_overwrites_existing_data() {
        let base = temp_dir("keep");
        let legacy = base.join(LEGACY_DIR_NAME);
        let current = base.join(CURRENT_DIR_NAME);
        fs::create_dir_all(&legacy).unwrap();
        fs::create_dir_all(&current).unwrap();
        fs::write(legacy.join("playtime-tracker.db"), b"old").unwrap();
        fs::write(current.join("playtime-tracker.db"), b"new").unwrap();

        migrate(&legacy, &current).unwrap();

        assert_eq!(
            fs::read(current.join("playtime-tracker.db")).unwrap(),
            b"new"
        );
        let _ = fs::remove_dir_all(&base);
    }
}
