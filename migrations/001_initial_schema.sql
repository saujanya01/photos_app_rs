-- ============================================================
-- schema_migrations (required for safe upgrades)
-- ============================================================
CREATE TABLE IF NOT EXISTS __schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- ============================================================
-- media_files
-- ============================================================
CREATE TABLE IF NOT EXISTS media_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Identity
    hash TEXT NOT NULL UNIQUE,                    -- SHA256
    file_size_bytes INTEGER NOT NULL,
    media_type TEXT NOT NULL CHECK (media_type IN ('image', 'video')),
    extension TEXT NOT NULL,

    -- EXIF / Metadata
    camera_make TEXT,
    camera_model TEXT,
    lens_model TEXT,
    date_taken TEXT,                              -- ISO 8601
    iso TEXT,
    aperture TEXT,
    shutter_speed TEXT,
    focal_length TEXT,
    software TEXT,

    -- Video-specific
    duration_seconds REAL,
    resolution_width INTEGER,
    resolution_height INTEGER,

    -- File system
    path TEXT NOT NULL,

    -- Timestamps (unix seconds)
    date_added INTEGER NOT NULL,
    date_modified INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_media_files_hash
    ON media_files(hash);

CREATE INDEX IF NOT EXISTS idx_media_files_date_taken
    ON media_files(date_taken);

CREATE INDEX IF NOT EXISTS idx_media_files_media_type
    ON media_files(media_type);

CREATE INDEX IF NOT EXISTS idx_media_files_path
    ON media_files(path);

-- ============================================================
-- Auto-update updated_at on change
-- ============================================================
CREATE TRIGGER IF NOT EXISTS trg_media_files_updated_at
AFTER UPDATE ON media_files
FOR EACH ROW
BEGIN
    UPDATE media_files
    SET updated_at = strftime('%s', 'now')
    WHERE id = NEW.id;
END;

-- ============================================================
-- duplicate_groups
-- ============================================================
CREATE TABLE IF NOT EXISTS duplicate_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    media_file_id INTEGER NOT NULL,
    total_copies INTEGER NOT NULL CHECK (total_copies >= 1),
    total_size_bytes INTEGER NOT NULL,
    wasted_space_bytes INTEGER NOT NULL,

    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    FOREIGN KEY (media_file_id)
        REFERENCES media_files(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_duplicate_groups_media_file_id
    ON duplicate_groups(media_file_id);

-- ============================================================
-- backup_sessions
-- ============================================================
CREATE TABLE IF NOT EXISTS backup_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    source_path TEXT NOT NULL,
    destination_path TEXT NOT NULL,

    -- Stats
    files_scanned INTEGER NOT NULL DEFAULT 0,
    files_copied INTEGER NOT NULL DEFAULT 0,
    files_skipped INTEGER NOT NULL DEFAULT 0,
    bytes_copied INTEGER NOT NULL DEFAULT 0,

    -- Timing
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    duration_seconds INTEGER,

    status TEXT NOT NULL
        CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),

    error_message TEXT,

    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_backup_sessions_status
    ON backup_sessions(status);

CREATE INDEX IF NOT EXISTS idx_backup_sessions_started_at
    ON backup_sessions(started_at);
