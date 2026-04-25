-- ============================================================
-- Migration 002: Thumbnails and Search
-- ============================================================

CREATE TABLE IF NOT EXISTS thumbnails (
    media_file_id INTEGER PRIMARY KEY,
    thumb_small BLOB NOT NULL,
    thumb_medium BLOB,
    generated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (media_file_id) REFERENCES media_files(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS media_tags (
    media_file_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (media_file_id, tag_id),
    FOREIGN KEY (media_file_id) REFERENCES media_files(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_media_date_type ON media_files(date_taken, media_type);
CREATE INDEX IF NOT EXISTS idx_media_camera ON media_files(camera_model);
CREATE INDEX IF NOT EXISTS idx_media_extension ON media_files(extension);
