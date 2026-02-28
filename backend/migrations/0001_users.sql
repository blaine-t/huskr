CREATE TABLE IF NOT EXISTS users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    oid           TEXT    NOT NULL UNIQUE,
    email         TEXT,
    display_name  TEXT,
    tenant_id     TEXT,
    access_token  TEXT,
    refresh_token TEXT,
    id_token      TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
