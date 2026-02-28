-- Add profile fields to users
ALTER TABLE users ADD COLUMN full_name   TEXT;
ALTER TABLE users ADD COLUMN age         INTEGER;
ALTER TABLE users ADD COLUMN is_rso      INTEGER NOT NULL DEFAULT 0; -- 0 = false, 1 = true
ALTER TABLE users ADD COLUMN major       TEXT;

-- Interests lookup table
CREATE TABLE IF NOT EXISTS interests (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT    NOT NULL UNIQUE
);

-- Many-to-many: users <-> interests
CREATE TABLE IF NOT EXISTS user_interests (
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    interest_id INTEGER NOT NULL REFERENCES interests(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, interest_id)
);

-- Matches between two users
CREATE TABLE IF NOT EXISTS matches (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user1_id   INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user2_id   INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT    NOT NULL DEFAULT (datetime('now')),
    -- ensure the pair is stored in a canonical order to avoid duplicates
    CHECK (user1_id < user2_id),
    UNIQUE (user1_id, user2_id)
);

-- Chat messages
CREATE TABLE IF NOT EXISTS messages (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    sender_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content      TEXT    NOT NULL,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_messages_sender    ON messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient_id);
CREATE INDEX IF NOT EXISTS idx_messages_pair      ON messages(sender_id, recipient_id);
