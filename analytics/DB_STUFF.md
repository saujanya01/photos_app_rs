# DB tables and operations and details

## Tables

### current

1. media_files -> Stores the media file data.
2. duplicate_groups -> Tracks the duplicates of file etc (might not be needed later on, just for initial analysis).
3. backup_sessions -> Stats of every backup session.

### for later

1. file_locations -> Stores the locations of each file with device details (maybe useful when coding mutli ssd sync feature).
2. backup_errors -> Errors from every backup session.

## Schemas

### media_files

```
CREATE TABLE media_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hash TEXT NOT NULL UNIQUE,                    -- SHA256 hash (unique identifier)
    file_size_bytes INTEGER NOT NULL,             -- Size in bytes
    file_size_human TEXT NOT NULL,                -- e.g., "2.5 MB"
    media_type TEXT NOT NULL CHECK(media_type IN ('image', 'video')),
    extension TEXT NOT NULL,                      -- e.g., "jpg", "mp4"

    -- EXIF / Metadata
    camera_make TEXT,
    camera_model TEXT,
    lens_model TEXT,
    date_taken TEXT,                              -- ISO 8601 format
    iso TEXT,
    aperture TEXT,
    shutter_speed TEXT,
    focal_length TEXT,
    software TEXT,

    -- Video specific
    duration_seconds REAL,                        -- For videos
    resolution_width INTEGER,
    resolution_height INTEGER,

    -- Timestamps
    date_added INTEGER NOT NULL,                  -- Unix timestamp when first backed up
    date_modified INTEGER NOT NULL,               -- Unix timestamp of last modification

    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    path TEXT,
);
```

### duplicate_groups

```
CREATE TABLE duplicate_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    total_copies INTEGER NOT NULL,                -- How many copies exist
    total_size_bytes INTEGER NOT NULL,            -- size × copies
    wasted_space_bytes INTEGER NOT NULL,          -- size × (copies - 1)

    FOREIGN KEY (media_file_id) REFERENCES media_files(id) ON DELETE CASCADE,
    FOREIGN KEY (media_file_hash) REFERENCES media_files(hash) ON DELETE CASCADE,
);
```

### backup_sessions

```
CREATE TABLE backup_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_path TEXT NOT NULL,            -- SD card
    destination_path TEXT NOT NULL,       -- SSD

    -- Stats
    files_scanned INTEGER NOT NULL,
    files_copied INTEGER NOT NULL,
    files_skipped INTEGER NOT NULL,               -- Already existed
    bytes_copied INTEGER NOT NULL,

    -- Timing
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    duration_seconds INTEGER,

    status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'cancelled')),
    error_message TEXT,
);
```
