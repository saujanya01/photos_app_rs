-- ============================================================
-- Migration 004: v0.6 extensions
--   · dominant_color (OKLCH string, computed during scan, drives viewer tint)
--   · rating (0–5 star ratings, X-key in action bar)
--   · albums + album_items (sidebar Albums section, AlbumView reorder)
-- Source: photo_app_rs v0.6 design direction doc · Round 3 §Implementation notes
-- ============================================================

-- ----------------------------------------
-- media_files extensions
-- ----------------------------------------
ALTER TABLE media_files ADD COLUMN dominant_color TEXT;
ALTER TABLE media_files ADD COLUMN rating INTEGER NOT NULL DEFAULT 0
    CHECK (rating BETWEEN 0 AND 5);

CREATE INDEX IF NOT EXISTS idx_media_files_rating
    ON media_files(rating)
    WHERE rating > 0;

-- ----------------------------------------
-- albums (TEXT id matches the prototype's slug-style identifiers,
--         and avoids autoinc collisions across portable sidecar DBs)
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS albums (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    cover_media_id  INTEGER REFERENCES media_files(id) ON DELETE SET NULL,
    created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TRIGGER IF NOT EXISTS trg_albums_updated_at
AFTER UPDATE ON albums
FOR EACH ROW
BEGIN
    UPDATE albums SET updated_at = strftime('%s', 'now') WHERE id = NEW.id;
END;

-- ----------------------------------------
-- album_items
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS album_items (
    album_id       TEXT NOT NULL,
    media_file_id  INTEGER NOT NULL,
    position       INTEGER NOT NULL DEFAULT 0,
    added_at       INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    PRIMARY KEY (album_id, media_file_id),
    FOREIGN KEY (album_id)      REFERENCES albums(id)     ON DELETE CASCADE,
    FOREIGN KEY (media_file_id) REFERENCES media_files(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_album_items_position
    ON album_items(album_id, position);
