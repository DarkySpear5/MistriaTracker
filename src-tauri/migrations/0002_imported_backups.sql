CREATE TABLE IF NOT EXISTS imported_backups (
    source_sha256 TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE RESTRICT,
    game_version TEXT NOT NULL,
    imported_at TEXT NOT NULL
);
