-- Likes: records when one user likes (or passes on) another user's profile.
-- When both user A likes user B AND user B likes user A, a match can be created.
CREATE TABLE IF NOT EXISTS likes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    liker_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    liked_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_like     INTEGER NOT NULL DEFAULT 1, -- 1 = liked, 0 = passed
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (liker_id, liked_id)
);

CREATE INDEX IF NOT EXISTS idx_likes_liker ON likes(liker_id);
CREATE INDEX IF NOT EXISTS idx_likes_liked ON likes(liked_id);
