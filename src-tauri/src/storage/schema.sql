CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    appid INTEGER UNIQUE,
    source TEXT NOT NULL DEFAULT 'steam',
    name TEXT NOT NULL,
    install_path TEXT,
    executable_path TEXT,
    cover_path TEXT,
    icon_path TEXT,
    hero_path TEXT,
    accent_color TEXT,
    is_installed INTEGER NOT NULL DEFAULT 1,
    is_hidden INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE game_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    duration_seconds INTEGER,
    FOREIGN KEY (game_id) REFERENCES games(id)
);

CREATE TABLE game_stats (
    game_id INTEGER PRIMARY KEY,
    total_playtime_seconds INTEGER NOT NULL DEFAULT 0,
    last_played_at INTEGER,
    FOREIGN KEY (game_id) REFERENCES games(id)
);

CREATE TABLE steam_media_failures (
    appid INTEGER NOT NULL,
    asset_type TEXT NOT NULL,
    failed_at INTEGER NOT NULL,
    retry_after INTEGER NOT NULL,
    last_error TEXT,
    PRIMARY KEY (appid, asset_type)
);

CREATE TABLE ignored_games (
    appid INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    ignored_at INTEGER NOT NULL,
    reason TEXT
);

CREATE TABLE app_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
