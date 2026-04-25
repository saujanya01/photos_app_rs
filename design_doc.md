# photo_app_rs — Design Document

**A Tauri-based Photo & Video Management System**

Version 1.0 · March 2026 · Author: Saujanya

*Extends the existing photo_app_rs Rust workspace*

---

## Table of Contents

1. [Problem Statement & Vision](#1-problem-statement--vision)
2. [Architecture Overview](#2-architecture-overview)
3. [Database Design](#3-database-design)
4. [Core Components](#4-core-components)
5. [Tauri Frontend](#5-tauri-frontend)
6. [Rust Crate Dependencies](#6-rust-crate-dependencies)
7. [Development Phases](#7-development-phases)
8. [Key Design Decisions & Tradeoffs](#8-key-design-decisions--tradeoffs)
9. [Performance Considerations](#9-performance-considerations)
10. [Future Possibilities](#10-future-possibilities-post-mvp)

---

## 1. Problem Statement & Vision

### 1.1 The Problem

No existing solution satisfies the requirements of a photographer who wants Apple Photos-like browsing but with full control over storage.

| Existing Tool | What It Gets Right | Where It Fails |
|---|---|---|
| **Apple Photos** | Beautiful timeline UI, smart grouping, instant search | Requires local import; no external drive support; locked to Apple ecosystem |
| **Digikam** | Powerful metadata, tagging, non-destructive edits | Treats video thumbnails as photos (destroys timeline); clunky UX; heavy resource usage |
| **Immich** | Modern web UI, face recognition, map view | Self-hosted server; copies all files into its own storage; not a desktop app |
| **photo_app_rs (current)** | SHA256 dedup, EXIF extraction, structured backup | CLI only; no visual browsing; no search UI; no thumbnail cache |

### 1.2 The Vision

photo_app_rs becomes a lightweight Tauri desktop app that:

- Browses photos and videos in a **timeline view** directly from external drives **without copying files locally**
- Generates and caches **thumbnails locally in SQLite** for instant scrolling (full-res loaded on demand from mounted drive)
- Treats photos and videos as **distinct media types** — video thumbnails never pollute the photo timeline
- Backs up to any target device in **yy/mm/dd structure** with a **portable metadata database** on the target
- Searches by date, location (reverse geocoded), camera, tags, and file properties

### 1.3 Design Principles

- **Non-destructive:** Never modify source files. All metadata, tags, and edits stored in the database only.
- **Source stays put:** Files remain on external drives. Only thumbnails are cached locally.
- **Portable database:** Backup targets carry their own SQLite DB — readable and useful even without the app.
- **Media type separation:** Photos and videos are indexed separately. Timeline can filter or interleave them.
- **Offline-first:** Everything works without internet. No cloud dependency.

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

The system is split into three layers, each with a clear responsibility:

```
┌─────────────────────────────────────────────────────────┐
│                    TAURI FRONTEND (WebView)              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │ Timeline  │  │  Search   │  │  Media   │  │ Backup │  │
│  │  View     │  │  Panel    │  │  Viewer  │  │ Config │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘  │
└────────────────────────┬────────────────────────────────┘
                         │ Tauri Commands (IPC)
┌────────────────────────┴────────────────────────────────┐
│                   RUST BACKEND (Tauri Core)              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │ Indexer   │  │ Thumb    │  │ Backup   │  │ Query  │  │
│  │ Service   │  │ Generator│  │ Engine   │  │ Engine │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘  │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐  │
│  │ EXIF      │  │ Video    │  │ Device Manager       │  │
│  │ Extractor │  │ Metadata │  │ (mount detection)    │  │
│  └──────────┘  └──────────┘  └──────────────────────┘  │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│                   STORAGE LAYER                          │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Local SQLite │  │ Thumb Cache  │  │ External     │   │
│  │ (app DB)     │  │ (~/.photo_app│  │ Drives       │   │
│  │              │  │  /thumbs/)   │  │ (source)     │   │
│  └─────────────┘  └──────────────┘  └──────────────┘   │
│  ┌──────────────────────────────────────────────────┐   │
│  │ Backup Target (ext drive with yy/mm/dd + DB)     │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Workspace Structure

Extending the existing photo_app_rs workspace with new crates:

```
photo_app_rs/
├── Cargo.toml                    # Workspace root
├── analytics/                    # Existing: EXIF extraction, scanning
│   ├── Cargo.toml
│   └── src/
├── core/                         # NEW: shared types, DB, business logic
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── db/
│       │   ├── mod.rs            # Connection pool, migrations
│       │   ├── models.rs         # Structs (reuse existing)
│       │   ├── queries.rs        # All SQL queries
│       │   └── schema.rs         # Migration definitions
│       ├── indexer/
│       │   ├── mod.rs            # Orchestrates scanning
│       │   ├── scanner.rs        # File discovery + filtering
│       │   ├── hasher.rs         # SHA256 (partial + full)
│       │   └── metadata.rs       # EXIF + video metadata extraction
│       ├── thumbnailer/
│       │   ├── mod.rs
│       │   ├── image_thumb.rs    # image crate resize
│       │   └── video_thumb.rs    # ffmpeg frame extraction
│       ├── backup/
│       │   ├── mod.rs
│       │   ├── engine.rs         # Copy + organize yy/mm/dd
│       │   └── portable_db.rs    # Write DB to backup target
│       ├── search/
│       │   ├── mod.rs
│       │   └── query_builder.rs  # Composable search filters
│       └── device/
│           ├── mod.rs
│           └── watcher.rs        # Mount/unmount detection
├── tauri-app/                    # NEW: Tauri application
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs               # Tauri entry point
│   │   └── commands/              # Tauri IPC commands
│   │       ├── mod.rs
│   │       ├── indexing.rs
│   │       ├── browsing.rs
│   │       ├── search.rs
│   │       └── backup.rs
│   └── frontend/                  # Web UI
│       ├── package.json
│       ├── src/
│       │   ├── App.tsx
│       │   ├── components/
│       │   │   ├── Timeline.tsx
│       │   │   ├── MediaGrid.tsx
│       │   │   ├── MediaViewer.tsx
│       │   │   ├── SearchBar.tsx
│       │   │   ├── Sidebar.tsx
│       │   │   └── BackupPanel.tsx
│       │   ├── hooks/
│       │   └── stores/
│       └── index.html
└── migrations/                    # Shared SQL migrations
    ├── 001_initial_schema.sql     # Existing
    └── 002_thumbnail_cache.sql    # NEW
```

---

## 3. Database Design

### 3.1 Dual Database Strategy

The system maintains two SQLite databases with distinct purposes:

| Database | Location | Purpose |
|---|---|---|
| **App Database** | `~/.photo_app_rs/photo_app.db` | Primary index of all known media, thumbnails, tags, search. Stays on the local machine. |
| **Portable Database** | `<backup_target>/.photo_index/backup.db` | Subset of metadata for files on that specific backup drive. Travels with the drive. |

### 3.2 Schema Extensions (Migration 002)

These tables extend the existing schema from migration 001. The existing `media_files`, `file_locations`, `storage_devices`, `backup_sessions`, and `duplicate_stats` tables remain unchanged.

#### 3.2.1 Thumbnail Cache

```sql
CREATE TABLE thumbnails (
    media_file_id INTEGER PRIMARY KEY,
    thumb_small   BLOB NOT NULL,          -- 200px, JPEG ~5-15KB
    thumb_medium  BLOB,                   -- 600px, JPEG ~30-60KB
    generated_at  INTEGER NOT NULL,
    FOREIGN KEY (media_file_id) REFERENCES media_files(id)
);
```

Thumbnails are stored as BLOBs in SQLite rather than individual files. For a library of 50K photos, the small thumbnails add roughly 500MB to the database — entirely manageable and avoids filesystem overhead of 50K tiny files.

#### 3.2.2 Tags and Albums

```sql
CREATE TABLE tags (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE media_tags (
    media_file_id INTEGER NOT NULL,
    tag_id        INTEGER NOT NULL,
    PRIMARY KEY (media_file_id, tag_id),
    FOREIGN KEY (media_file_id) REFERENCES media_files(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TABLE albums (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT NOT NULL,
    description    TEXT,
    cover_media_id INTEGER,
    created_at     INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE album_media (
    album_id      INTEGER NOT NULL,
    media_file_id INTEGER NOT NULL,
    sort_order    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (album_id, media_file_id),
    FOREIGN KEY (album_id) REFERENCES albums(id),
    FOREIGN KEY (media_file_id) REFERENCES media_files(id)
);
```

#### 3.2.3 Location Index

```sql
-- Reverse geocoded location cache
CREATE TABLE locations (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    latitude  REAL NOT NULL,
    longitude REAL NOT NULL,
    city      TEXT,
    state     TEXT,
    country   TEXT,
    label     TEXT,                   -- e.g. 'Goa, India'
    UNIQUE(latitude, longitude)
);

-- Add location columns to media_files (migration ALTER TABLE)
ALTER TABLE media_files ADD COLUMN location_id INTEGER REFERENCES locations(id);
ALTER TABLE media_files ADD COLUMN gps_latitude REAL;
ALTER TABLE media_files ADD COLUMN gps_longitude REAL;
```

#### 3.2.4 Search Indexes

```sql
-- Full-text search on metadata
CREATE VIRTUAL TABLE media_fts USING fts5(
    camera_make, camera_model, lens_model,
    extension, tags_text, location_label,
    content=media_files, content_rowid=id
);

-- Composite indexes for common queries
CREATE INDEX idx_media_date_type ON media_files(date_taken, media_type);
CREATE INDEX idx_media_location ON media_files(location_id);
CREATE INDEX idx_media_camera ON media_files(camera_model);
```

### 3.3 Portable Database Schema

The backup target database is a self-contained subset. It includes `media_files` (all columns), `file_locations` (only paths on this device), `storage_devices` (only this device), and `backup_sessions` (only sessions targeting this device). It does NOT include thumbnails, tags, albums, or FTS indexes — those stay in the app database.

This means if you plug the backup drive into any machine with SQLite, you can query your entire photo metadata:

```bash
sqlite3 /Volumes/BACKUP/.photo_index/backup.db \
  "SELECT file_path, date_taken, camera_model FROM media_files ORDER BY date_taken"
```

---

## 4. Core Components

### 4.1 Indexer Service

The indexer scans source directories (external drives, SD cards) and populates the database. It reuses the existing scanning logic from the `analytics` crate and extends it.

#### 4.1.1 Scanning Pipeline

```
Source Dir ──▶ File Discovery ──▶ Filter (extensions) ──▶ Partial Hash
                                                              │
                                                    ┌─────────┴─────────┐
                                                    │ Hash exists in DB? │
                                                    └─────────┬─────────┘
                                                      Yes │         │ No
                                                          │         │
                                                    Skip (log)  Full Hash
                                                                │
                                                          ┌─────┴─────┐
                                                          │ Extract   │
                                                          │ Metadata  │
                                                          └─────┬─────┘
                                                                │
                                                          ┌─────┴─────┐
                                                          │ Generate  │
                                                          │ Thumbnail │
                                                          └─────┬─────┘
                                                                │
                                                          Insert into DB
```

#### 4.1.2 Supported Extensions

| Media Type | Extensions | Metadata Source |
|---|---|---|
| **Image** | jpg, jpeg, png, tiff, arw, cr2, nef, dng, heic, heif, webp | kamadak-exif |
| **Video** | mp4, mov, avi, mkv, mts | ffprobe + Sony XML sidecars |

#### 4.1.3 Media Type Separation

This is critical to avoiding the Digikam problem. The indexer classifies every file at scan time by extension into `image` or `video`. The `media_type` column in `media_files` drives all downstream behavior:

- Timeline view groups by date but can filter to photos-only, videos-only, or both
- Video thumbnails are generated with ffmpeg (single frame at 10% duration) and stored separately
- Video thumbnails are NEVER mixed into the photo grid unless explicitly requested

### 4.2 Thumbnail Generator

Generates two sizes of JPEG thumbnails per media file:

| Size | Dimension | Use Case | Approx Size |
|---|---|---|---|
| Small | 200px longest edge | Grid view, timeline scrolling | 5-15 KB |
| Medium | 600px longest edge | Detail preview, hover | 30-60 KB |

#### 4.2.1 Image Thumbnails

Uses the `image` crate with the Lanczos3 filter for quality downscaling. For RAW files (ARW, CR2, NEF), extracts the embedded JPEG preview from EXIF data first (much faster than decoding the full RAW).

#### 4.2.2 Video Thumbnails

Calls ffmpeg to extract a single frame:

```bash
ffmpeg -ss <10% of duration> -i input.mp4 -frames:v 1 -q:v 2 thumb.jpg
```

The 10% offset avoids black frames common at the start of videos. If ffmpeg is not installed, video thumbnails are skipped gracefully (placeholder icon shown in UI).

### 4.3 Backup Engine

Extends the existing backup logic with portable database generation.

#### 4.3.1 Backup Flow

```
1. User selects: Source (indexed drive) ──▶ Target (backup drive)
2. Engine compares source DB hashes against target DB hashes
3. New files are copied to target in yy/mm/dd structure:
   /BACKUP/media/25/03/08/IMG_1234.ARW
   /BACKUP/media/25/03/08/VID_5678.MP4
4. Target portable DB updated with new file entries
5. Backup session logged (files scanned/copied/skipped, duration)
```

#### 4.3.2 Target Directory Structure

```
/Volumes/BACKUP_DRIVE/
├── .photo_index/
│   ├── backup.db               # Portable SQLite database
│   └── device_info.json        # Drive metadata (UUID, label, capacity)
└── media/
    ├── 24/
    │   ├── 12/
    │   │   ├── 27/
    │   │   │   ├── IMG_0001.ARW
    │   │   │   ├── IMG_0001.JPG
    │   │   │   └── VID_0002.MP4
    │   │   └── 28/
    │   │       └── ...
    │   └── ...
    └── 25/
        └── ...
```

### 4.4 Search & Query Engine

Composable query builder that constructs SQL from user-facing filters:

| Filter | Maps To | Example |
|---|---|---|
| Date range | `date_taken BETWEEN ? AND ?` | "March 2025" |
| Location | `location_id IN (SELECT id FROM locations WHERE ...)` | "Goa" or "India" |
| Camera | `camera_model LIKE ?` | "A6400" |
| Media type | `media_type = ?` | "image" or "video" |
| Tags | `id IN (SELECT media_file_id FROM media_tags ...)` | "sunset, landscape" |
| Full text | `media_fts MATCH ?` | "Sony portrait" |
| File size | `file_size_bytes > ?` | "> 10MB" |
| Extension | `extension = ?` | "arw" |

Filters are AND-combined. The query builder produces parameterized SQL to avoid injection. FTS5 handles fuzzy text matching across concatenated metadata fields.

---

## 5. Tauri Frontend

### 5.1 Technology Stack

| Layer | Choice | Rationale |
|---|---|---|
| Framework | Tauri v2 | Rust backend, ~10MB binary, native performance |
| UI Framework | React + TypeScript | Rich ecosystem, good virtualization libraries |
| Styling | Tailwind CSS | Rapid iteration, consistent design |
| State Management | Zustand | Lightweight, no boilerplate |
| Virtualization | react-virtuoso or react-window | Essential for rendering 50K+ thumbnails smoothly |
| Build Tool | Vite | Fast HMR, works well with Tauri |

### 5.2 UI Layout

```
┌──────────────────────────────────────────────────────────┐
│ ┌─────────┐  ┌───────────────────────────────────────┐   │
│ │         │  │ Search: [________________________] 🔍  │   │
│ │ Sidebar  │  ├───────────────────────────────────────┤   │
│ │         │  │                                        │   │
│ │ 📷 All   │  │  ── March 2025 ────────────────────   │   │
│ │ 🖼 Photos│  │  [img] [img] [img] [img] [img] [img] │   │
│ │ 🎬 Videos│  │  [img] [img] [img] [img]             │   │
│ │ 📁 Albums│  │                                        │   │
│ │ 🏷 Tags  │  │  ── February 2025 ────────────────    │   │
│ │ 💾 Devices│  │  [img] [img] [img] [img] [img]      │   │
│ │ ⚙ Backup │  │  [vid] [img] [img]                   │   │
│ │         │  │                                        │   │
│ └─────────┘  └───────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

### 5.3 Key UI Components

#### 5.3.1 Timeline View

The primary view. Groups media by date (day/month/year) with section headers. Uses virtualized scrolling — only thumbnails in the viewport are rendered. Clicking a thumbnail opens the MediaViewer which loads the full-resolution file from the mounted drive.

#### 5.3.2 Media Viewer

Full-screen overlay with left/right navigation. Shows the full-resolution image loaded directly from the external drive path stored in `file_locations`. Displays EXIF data in an expandable sidebar panel. For videos, uses an embedded player (HTML5 video element pointing to the file path via Tauri's asset protocol).

#### 5.3.3 Backup Panel

Shows connected devices, last backup time, and a diff summary (X new files, Y GB to transfer). Progress bar with file-level detail during backup. History of past backup sessions from the `backup_sessions` table.

### 5.4 Tauri IPC Commands

All communication between the frontend and Rust backend goes through Tauri's command system:

| Command | Direction | Purpose |
|---|---|---|
| `scan_directory` | Frontend → Backend | Start indexing a directory path |
| `get_timeline` | Frontend → Backend | Fetch paginated timeline (date range, offset, limit) |
| `get_thumbnail` | Frontend → Backend | Retrieve thumbnail BLOB for a media_file_id |
| `get_full_media` | Frontend → Backend | Get file path for full-res viewing (asset protocol) |
| `search_media` | Frontend → Backend | Execute search with filter object |
| `start_backup` | Frontend → Backend | Initiate backup from source to target device |
| `get_devices` | Frontend → Backend | List known storage devices and connection status |
| `add_tags` | Frontend → Backend | Add tags to selected media files |
| `scan_progress` | Backend → Frontend | Event: indexing progress (files scanned, total, ETA) |
| `backup_progress` | Backend → Frontend | Event: backup progress (files copied, speed, ETA) |
| `device_connected` | Backend → Frontend | Event: external drive mounted/unmounted |

---

## 6. Rust Crate Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `tauri` | 2.x | Application framework, IPC, asset protocol |
| `rusqlite` | 0.31+ | SQLite with bundled feature (no system dep) |
| `kamadak-exif` | 0.5 | EXIF extraction (already in use) |
| `image` | 0.25+ | Image decoding, resizing, JPEG encoding |
| `sha2` | 0.10+ | SHA256 hashing |
| `serde` / `serde_json` | 1.x | Serialization for IPC and JSON |
| `tokio` | 1.x | Async runtime (already in use) |
| `notify` | 6.x | Filesystem watcher for device mount events |
| `walkdir` | 2.x | Recursive directory traversal |
| `chrono` | 0.4 | Date parsing and formatting |
| `rayon` | 1.x | Parallel thumbnail generation |
| `log` + `env_logger` | latest | Structured logging |

For video metadata extraction, the app shells out to `ffprobe`/`ffmpeg` (same approach as the existing analytics crate). This avoids pulling in heavy C bindings and keeps the binary small.

---

## 7. Development Phases

Each phase produces a usable increment. You can stop after any phase and have something functional.

### Phase 1: Core Library Extraction (1-2 weeks)

Refactor existing `analytics` crate code into the new `core` crate. Extract the scanning, hashing, and metadata logic into reusable modules. Set up the migration system to run both 001 and 002. Write unit tests for the hasher and metadata extractors. The goal is a clean library that both the CLI and Tauri app can depend on.

### Phase 2: Thumbnail Generation (1 week)

Implement the `thumbnailer` module. Image thumbnails via the `image` crate, video thumbnails via ffmpeg. Store in SQLite BLOBs. Test with a mix of JPEG, ARW, and MP4 files from your A6400. Verify that RAW thumbnail extraction uses the embedded JPEG preview for speed.

### Phase 3: Tauri App Shell + Timeline (2-3 weeks)

Set up the Tauri v2 project with React + Vite + Tailwind. Implement the Timeline view with virtualized scrolling. Wire up `get_timeline` and `get_thumbnail` commands. At this point you can scan a directory and browse photos in a timeline. **This is the first "wow moment".**

### Phase 4: Search + Tags (1-2 weeks)

Add the FTS5 index and query builder. Implement the SearchBar component with filter chips. Add tag management (create, assign, bulk tag). Build the sidebar with filter-by-camera, filter-by-location panels.

### Phase 5: Backup Engine + Portable DB (1-2 weeks)

Extend the existing backup logic with portable database generation. Implement the BackupPanel UI with progress tracking. Add device detection (mount watcher). Test the full flow: scan SD card → browse timeline → backup to external drive → verify portable DB on target.

### Phase 6: Polish (ongoing)

Albums, keyboard shortcuts, drag-select in grid, dark mode, performance optimization for 50K+ file libraries, optional reverse geocoding via offline database (cities500 from GeoNames).

### Phase Summary

| Phase | Milestone | Estimated Time |
|---|---|---|
| 1 | Core library with tests, migrations running | 1-2 weeks |
| 2 | Thumbnails generating for images + videos | 1 week |
| 3 | Tauri app with browsable timeline | 2-3 weeks |
| 4 | Search working with FTS + filters | 1-2 weeks |
| 5 | Backup with portable DB on target | 1-2 weeks |
| 6 | Albums, polish, dark mode | Ongoing |

---

## 8. Key Design Decisions & Tradeoffs

### 8.1 Thumbnails in SQLite vs Filesystem

**Decision:** Store thumbnails as BLOBs in SQLite.

- **Pro:** Single file to manage, atomic operations, no orphaned thumbnails, simpler backup of the cache.
- **Pro:** SQLite handles 50K BLOBs of 10-60KB each without issues (tested up to millions of rows).
- **Con:** Database file grows to ~500MB-1GB for large libraries. Acceptable for a local cache.
- **Alternative considered:** Filesystem thumbnails (like Digikam). Rejected due to cache invalidation complexity and filesystem overhead.

### 8.2 No Local Copy of Full Files

**Decision:** Full-resolution files are always read from the external drive at view time.

- **Implication:** If the drive is disconnected, you can still browse thumbnails and metadata, but cannot view full resolution. The UI shows a clear "drive disconnected" indicator.
- **Why not cache full-res locally?** A library of 10K RAW files at 25MB each = 250GB. Defeats the purpose of keeping files on external storage.

### 8.3 Reverse Geocoding Strategy

**Decision:** Use an offline city database (GeoNames cities500, ~10MB) rather than an API.

- **Rationale:** Offline-first principle. No API keys, no rate limits, no internet required. Accuracy to city level is sufficient for photo organization.
- **Implementation:** Load `cities500.txt` into SQLite at first run. Nearest-city lookup via Haversine distance. Cache results in the `locations` table.

### 8.4 React vs Leptos/Yew for Frontend

**Decision:** React + TypeScript, despite the Rust theme.

- **Rationale:** React has battle-tested virtualization libraries (`react-virtuoso`, `react-window`) essential for smooth scrolling through thousands of thumbnails. Leptos/Yew ecosystem is not mature enough for this use case yet. The Rust backend is where performance matters; the frontend is rendering HTML.

---

## 9. Performance Considerations

| Operation | Target | Strategy |
|---|---|---|
| Initial scan of 10K files | < 5 minutes | Parallel hashing with rayon, partial hash shortcut |
| Timeline scroll (50K items) | 60 fps | Virtualized list, only render visible rows, small thumb BLOBs |
| Thumbnail load | < 10ms per thumb | SQLite BLOB read by primary key is O(1) |
| Search query | < 100ms | FTS5 index, composite indexes on common filter columns |
| Full-res image load | < 500ms | Direct file read from mounted drive, no processing |
| Backup of 1K new files | I/O bound | Sequential copy with progress events, no CPU bottleneck |

The main performance risk is the initial indexing of a large library. The partial hash optimization (128KB read instead of full file) reduces this significantly — for a 25MB ARW file, you skip 99.5% of the data on duplicate checks.

---

## 10. Future Possibilities (Post-MVP)

These are explicitly out of scope for the initial build but designed to be addable without architectural changes:

- **Face detection and grouping** — using a local ML model like ONNX-based RetinaFace
- **Map view** — using Leaflet with OpenStreetMap tiles, GPS data already in DB
- **Duplicate detection UI** — the `duplicate_stats` table already exists; just needs a frontend
- **Multi-device sync** — if you ever want two machines to share the same index, a CRDT-based sync layer on top of SQLite
- **Mobile companion app** — Tauri v2 supports iOS/Android, could reuse the same frontend
- **RAW processing preview** — embed a lightweight RAW processor like `rawloader` for quick adjustments