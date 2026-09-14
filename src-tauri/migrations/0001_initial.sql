PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS accepted_events (
    session_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE RESTRICT,
    event_json TEXT NOT NULL,
    PRIMARY KEY (session_id, sequence)
);

CREATE TABLE IF NOT EXISTS import_batches (
    id INTEGER PRIMARY KEY,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS imported_evidence (
    id INTEGER PRIMARY KEY,
    batch_id INTEGER NOT NULL REFERENCES import_batches(id) ON DELETE RESTRICT,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE RESTRICT,
    evidence_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS manual_overrides (
    id INTEGER PRIMARY KEY,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE RESTRICT,
    override_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE RESTRICT,
    entity_id TEXT NOT NULL,
    text TEXT NOT NULL CHECK(length(text) <= 20000),
    PRIMARY KEY (profile_id, entity_id)
);

CREATE TABLE IF NOT EXISTS journal_cursors (
    profile_id TEXT PRIMARY KEY REFERENCES profiles(id) ON DELETE RESTRICT,
    cursor TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS catalog_versions (
    game_version TEXT PRIMARY KEY,
    extracted_at TEXT NOT NULL
);
