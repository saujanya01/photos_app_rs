-- ============================================================
-- Migration 005: FTS5 search index + backup resume state
--   · media_fts   — full-text search across metadata + tags_text
--   · backup_sessions_v2 + backup_session_items — per-file resume
-- Source: photo_app_rs phases plan · Phase 1 (FTS5 + backup_v2)
-- ============================================================

-- ----------------------------------------
-- media_fts — FTS5 virtual table.
-- Non-external content (we manage sync via triggers below) so the
-- derived `tags_text` column can live alongside indexed metadata.
-- ----------------------------------------
CREATE VIRTUAL TABLE IF NOT EXISTS media_fts USING fts5(
    camera_make,
    camera_model,
    lens_model,
    extension,
    software,
    path,
    tags_text,
    tokenize = "unicode61 remove_diacritics 2"
);

-- Backfill existing rows (idempotent: INSERT OR REPLACE by rowid).
INSERT OR REPLACE INTO media_fts (
    rowid, camera_make, camera_model, lens_model,
    extension, software, path, tags_text
)
SELECT
    m.id,
    COALESCE(m.camera_make, ''),
    COALESCE(m.camera_model, ''),
    COALESCE(m.lens_model, ''),
    COALESCE(m.extension, ''),
    COALESCE(m.software, ''),
    COALESCE(m.path, ''),
    COALESCE((
        SELECT GROUP_CONCAT(t.name, ' ')
        FROM media_tags mt
        JOIN tags t ON t.id = mt.tag_id
        WHERE mt.media_file_id = m.id
    ), '')
FROM media_files m;

-- ----------------------------------------
-- Triggers to keep media_fts in sync with media_files.
-- tags_text is preserved on UPDATE so we don't have to re-aggregate
-- tags every time an EXIF column changes.
-- ----------------------------------------
CREATE TRIGGER IF NOT EXISTS trg_media_files_fts_ai
AFTER INSERT ON media_files
BEGIN
    INSERT INTO media_fts (
        rowid, camera_make, camera_model, lens_model,
        extension, software, path, tags_text
    ) VALUES (
        NEW.id,
        COALESCE(NEW.camera_make, ''),
        COALESCE(NEW.camera_model, ''),
        COALESCE(NEW.lens_model, ''),
        COALESCE(NEW.extension, ''),
        COALESCE(NEW.software, ''),
        COALESCE(NEW.path, ''),
        ''
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_media_files_fts_au
AFTER UPDATE ON media_files
BEGIN
    UPDATE media_fts
    SET camera_make  = COALESCE(NEW.camera_make, ''),
        camera_model = COALESCE(NEW.camera_model, ''),
        lens_model   = COALESCE(NEW.lens_model, ''),
        extension    = COALESCE(NEW.extension, ''),
        software     = COALESCE(NEW.software, ''),
        path         = COALESCE(NEW.path, '')
    WHERE rowid = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_media_files_fts_ad
AFTER DELETE ON media_files
BEGIN
    DELETE FROM media_fts WHERE rowid = OLD.id;
END;

-- ----------------------------------------
-- Triggers to keep tags_text in sync with the tags ↔ media_tags graph.
-- A tag join/unjoin re-aggregates tags_text for that single media file.
-- ----------------------------------------
CREATE TRIGGER IF NOT EXISTS trg_media_tags_fts_ai
AFTER INSERT ON media_tags
BEGIN
    UPDATE media_fts
    SET tags_text = COALESCE((
        SELECT GROUP_CONCAT(t.name, ' ')
        FROM media_tags mt
        JOIN tags t ON t.id = mt.tag_id
        WHERE mt.media_file_id = NEW.media_file_id
    ), '')
    WHERE rowid = NEW.media_file_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_media_tags_fts_ad
AFTER DELETE ON media_tags
BEGIN
    UPDATE media_fts
    SET tags_text = COALESCE((
        SELECT GROUP_CONCAT(t.name, ' ')
        FROM media_tags mt
        JOIN tags t ON t.id = mt.tag_id
        WHERE mt.media_file_id = OLD.media_file_id
    ), '')
    WHERE rowid = OLD.media_file_id;
END;

-- A tag rename should propagate to every media file carrying it.
CREATE TRIGGER IF NOT EXISTS trg_tags_fts_au
AFTER UPDATE OF name ON tags
BEGIN
    UPDATE media_fts
    SET tags_text = COALESCE((
        SELECT GROUP_CONCAT(t.name, ' ')
        FROM media_tags mt
        JOIN tags t ON t.id = mt.tag_id
        WHERE mt.media_file_id = media_fts.rowid
    ), '')
    WHERE rowid IN (SELECT media_file_id FROM media_tags WHERE tag_id = NEW.id);
END;

-- ============================================================
-- backup_sessions_v2 — replaces the per-session row from 001 with
-- a richer model that supports pause / resume + per-file state.
-- The original `backup_sessions` table is kept for historical reads.
-- ============================================================
CREATE TABLE IF NOT EXISTS backup_sessions_v2 (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,

    source_scope      TEXT NOT NULL
        CHECK (source_scope IN ('library', 'selection', 'album')),
    source_ref        TEXT,                  -- album id, selection token, or NULL for library
    destination_path  TEXT NOT NULL,

    conflict_policy   TEXT NOT NULL DEFAULT 'skip'
        CHECK (conflict_policy IN ('skip', 'overwrite', 'rename')),
    sidecar_enabled   INTEGER NOT NULL DEFAULT 1
        CHECK (sidecar_enabled IN (0, 1)),

    status            TEXT NOT NULL
        CHECK (status IN ('planning', 'running', 'paused', 'completed', 'failed', 'cancelled')),

    -- Aggregate counters (updated as items move through states)
    total_files       INTEGER NOT NULL DEFAULT 0,
    total_bytes       INTEGER NOT NULL DEFAULT 0,
    files_done        INTEGER NOT NULL DEFAULT 0,
    files_skipped     INTEGER NOT NULL DEFAULT 0,
    files_failed      INTEGER NOT NULL DEFAULT 0,
    bytes_done        INTEGER NOT NULL DEFAULT 0,

    started_at        INTEGER NOT NULL,
    last_progress_at  INTEGER,
    completed_at      INTEGER,
    error_message     TEXT,

    created_at        INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at        INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_backup_v2_status
    ON backup_sessions_v2(status);

CREATE TRIGGER IF NOT EXISTS trg_backup_v2_updated_at
AFTER UPDATE ON backup_sessions_v2
FOR EACH ROW
BEGIN
    UPDATE backup_sessions_v2
    SET updated_at = strftime('%s', 'now')
    WHERE id = NEW.id;
END;

-- ----------------------------------------
-- backup_session_items — per-file plan + state. The presence of this
-- table is what makes resume cheap: on `start_backup` we either reuse
-- a paused session's items, or plan a fresh set.
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS backup_session_items (
    session_id    INTEGER NOT NULL,
    media_file_id INTEGER NOT NULL,

    source_path   TEXT NOT NULL,
    dest_path     TEXT NOT NULL,
    file_hash     TEXT NOT NULL,           -- SHA-256, used for dedup against existing dest
    file_size     INTEGER NOT NULL,

    status        TEXT NOT NULL DEFAULT 'planned'
        CHECK (status IN ('planned', 'copying', 'done', 'skipped', 'failed')),

    bytes_copied  INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    completed_at  INTEGER,

    PRIMARY KEY (session_id, media_file_id),
    FOREIGN KEY (session_id) REFERENCES backup_sessions_v2(id) ON DELETE CASCADE,
    FOREIGN KEY (media_file_id) REFERENCES media_files(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_backup_items_session_status
    ON backup_session_items(session_id, status);

CREATE INDEX IF NOT EXISTS idx_backup_items_hash
    ON backup_session_items(file_hash);
