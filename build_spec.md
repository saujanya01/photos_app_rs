# BUILD_SPEC.md — photo_app_rs Tauri Viewer

> **For Claude Code**: Read this entire file before writing any code. This is the spec for the `tauri-viewer` branch. Build everything described here. Do not build backup/sync features — only the viewer.

## What exists already

This is a Rust workspace at `photo_app_rs/` with the following structure:

```
photo_app_rs/
├── Cargo.toml              # Workspace root: [workspace] members = ["analytics"]
├── analytics/              # Existing crate (binary + lib)
│   ├── Cargo.toml          # deps: kamadak-exif (imported as `exif`), rusqlite, serde, serde_json, sha2, walkdir, chrono
│   └── src/
│       ├── main.rs         # Entry point, calls scan_directory_with_db()
│       ├── lib.rs
│       ├── utils/
│       │   └── mod.rs      # scan_directory(), Media struct, FileType enum, EXIF extraction with kamadak-exif
│       └── db/
│           ├── mod.rs      # Database struct, initialize_database(), migration runner
│           ├── models.rs   # MediaFile, FileLocation, StorageDevice, BackupSession, DuplicateInfo structs
│           ├── queries.rs  # hash_exists(), insert_media_file(), find_duplicate_hashes(), etc.
│           └── migrations.rs
├── migrations/
│   └── 001_initial_schema.sql  # media_files, file_locations, storage_devices, backup_sessions, duplicate_groups tables
└── .gitignore
```

### Existing schema (001_initial_schema.sql) — key tables:

```sql
-- media_files: id, hash, file_size_bytes, file_size_human, media_type ('image'|'video'),
--   extension, camera_make, camera_model, lens_model, date_taken, iso, aperture,
--   shutter_speed, focal_length, software, duration_seconds, resolution_width,
--   resolution_height, date_added, date_modified, created_at, updated_at

-- file_locations: id, media_file_id, file_path, storage_device_id, is_primary,
--   original_filename, discovered_at, status ('active'|'missing'|'moved')

-- storage_devices: id, device_type, volume_uuid, volume_label, fingerprint,
--   mount_path, capacity_bytes, first_seen, last_seen, last_backup
```

### Existing Rust types in analytics/src/db/models.rs:

```rust
pub struct MediaFile {
    pub id: Option<i64>,
    pub hash: String,
    pub file_size_bytes: i64,
    pub file_size_human: String,
    pub media_type: String,        // "image" or "video"
    pub extension: String,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub date_taken: Option<String>, // ISO 8601
    pub iso: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<String>,
    pub software: Option<String>,
    pub duration_seconds: Option<f64>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub date_added: i64,
    pub date_modified: i64,
}
```

### Existing EXIF extraction in analytics/src/utils/mod.rs:

- Uses `exif::{In, Reader, Tag}` (crate name `kamadak-exif`)
- Scans directories recursively with `walkdir`
- Classifies files by extension into `FileType::Image` or `FileType::Video`
- Extracts EXIF: camera make/model, lens, date, ISO, aperture, shutter speed, focal length, GPS lat/lon
- Computes SHA256 hash (partial 128KB first, full if needed)
- Video metadata via `ffprobe` shell command

---

## What to build

A Tauri v2 desktop app that lets you browse photos/videos from any directory (including external drives) in a timeline view, without copying files locally. Only thumbnails are cached.

### Architecture

```
photo_app_rs/
├── Cargo.toml              # UPDATE: add "core" and "tauri-app" to workspace members
├── analytics/              # KEEP UNCHANGED — do not modify existing code
├── core/                   # NEW: shared library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── db.rs           # DB connection, migrations (reuse pattern from analytics/src/db/)
│       ├── models.rs       # Re-export + extend analytics models
│       ├── indexer.rs      # Scan + hash + extract metadata + generate thumbnails
│       ├── thumbnailer.rs  # Image resize (image crate), video frame (ffmpeg)
│       └── search.rs       # Query builder for filters
├── tauri-app/              # NEW: Tauri v2 application
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── src-tauri/
│   │   └── src/
│   │       ├── main.rs     # Tauri entry, register commands
│   │       └── commands.rs # All #[tauri::command] functions
│   ├── src/                # Frontend (React + TypeScript + Vite)
│   │   ├── App.tsx
│   │   ├── main.tsx
│   │   ├── index.css       # Tailwind
│   │   ├── components/
│   │   │   ├── Timeline.tsx
│   │   │   ├── MediaGrid.tsx
│   │   │   ├── MediaViewer.tsx
│   │   │   ├── SearchBar.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── DeviceStatus.tsx
│   │   ├── hooks/
│   │   │   └── useTauri.ts # Typed wrappers around invoke()
│   │   └── stores/
│   │       └── appStore.ts # Zustand store
│   ├── index.html
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   └── postcss.config.js
└── migrations/
    ├── 001_initial_schema.sql   # KEEP UNCHANGED
    └── 002_thumbnails_and_search.sql  # NEW
```

---

## Step-by-step build instructions

### Step 1: Migration 002

Create `migrations/002_thumbnails_and_search.sql`:

```sql
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
```

### Step 2: `core` crate

**`core/Cargo.toml`:**
```toml
[package]
name = "photo-app-core"
version = "0.1.0"
edition = "2021"

[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
walkdir = "2"
chrono = "0.4"
image = "0.25"
rayon = "1"
log = "0.4"
env_logger = "0.11"
exif = "0.5"                # This is kamadak-exif, import as `exif`
```

**`core/src/lib.rs`:**
```rust
pub mod db;
pub mod models;
pub mod indexer;
pub mod thumbnailer;
pub mod search;
```

**`core/src/models.rs`** — Define these structs with `Serialize, Deserialize, Clone, Debug`:
- `MediaFile` — same fields as analytics/src/db/models.rs (copy them)
- `TimelineGroup` — `{ date: String, media: Vec<MediaItem> }`
- `MediaItem` — `{ id: i64, media_type: String, extension: String, date_taken: Option<String>, camera_model: Option<String>, file_path: String, thumb_small_b64: Option<String> }` (thumbnail as base64 for IPC)
- `SearchFilter` — `{ date_from: Option<String>, date_to: Option<String>, media_type: Option<String>, camera_model: Option<String>, extension: Option<String>, query: Option<String> }`
- `ScanProgress` — `{ total_files: u64, scanned: u64, new_files: u64, current_file: String }`
- `IndexStats` — `{ total_media: i64, total_images: i64, total_videos: i64, total_size_bytes: i64, cameras: Vec<String>, date_range: (Option<String>, Option<String>) }`

**`core/src/db.rs`:**
- `pub fn open_db(path: &str) -> Result<Connection>` — opens SQLite, enables foreign keys, WAL mode, runs migrations 001 + 002 (use `include_str!` for migration files)
- `pub fn get_timeline(conn: &Connection, offset: i64, limit: i64, filter: &SearchFilter) -> Result<Vec<TimelineGroup>>` — query media_files grouped by date (GROUP BY date(date_taken)), join thumbnails table for thumb_small, encode as base64. Order by date_taken DESC. Apply filter WHERE clauses.
- `pub fn get_media_detail(conn: &Connection, id: i64) -> Result<MediaFile>` — full row from media_files
- `pub fn get_thumbnail(conn: &Connection, media_file_id: i64, size: &str) -> Result<Option<Vec<u8>>>` — return BLOB from thumbnails table
- `pub fn get_stats(conn: &Connection) -> Result<IndexStats>` — aggregate query
- `pub fn get_file_path(conn: &Connection, media_file_id: i64) -> Result<String>` — return file_path from file_locations WHERE is_primary = 1

**`core/src/indexer.rs`:**
- `pub fn scan_directory(conn: &Connection, dir_path: &str, progress_callback: impl Fn(ScanProgress)) -> Result<u64>`
- Walk directory with `walkdir`. For each file:
  1. Check extension → classify as image/video (same extensions as analytics crate: jpg, jpeg, png, tiff, arw, cr2, nef, dng, heic, heif, webp for images; mp4, mov, avi, mkv, mts for videos)
  2. Compute partial SHA256 hash (first 128KB)
  3. Check if hash exists in DB → skip if yes
  4. Compute full hash
  5. Extract EXIF metadata (reuse same approach as analytics/src/utils/mod.rs using `exif` crate)
  6. For videos: shell out to `ffprobe -v quiet -print_format json -show_format -show_streams <path>` and parse JSON for duration, resolution
  7. Insert into media_files table
  8. Insert into file_locations table (is_primary = 1)
  9. Generate thumbnail (call thumbnailer)
  10. Insert thumbnail into thumbnails table
  11. Call progress_callback with updated ScanProgress
- Use `rayon` for parallel hashing where possible, but keep DB writes sequential

**`core/src/thumbnailer.rs`:**
- `pub fn generate_image_thumbnail(path: &str, max_dim: u32) -> Result<Vec<u8>>` — use `image` crate: open file, resize with `image::imageops::FilterType::Lanczos3`, encode as JPEG (quality 80) into Vec<u8>. For RAW files (.arw, .cr2, .nef, .dng), try to extract embedded JPEG from EXIF first using `exif` crate; if that fails, return a placeholder or skip.
- `pub fn generate_video_thumbnail(path: &str, max_dim: u32) -> Result<Vec<u8>>` — shell out: `ffmpeg -ss 2 -i <path> -frames:v 1 -q:v 2 -f image2pipe -vcodec mjpeg pipe:1`. Capture stdout bytes, then resize with image crate. If ffmpeg not found, return Err (caller handles gracefully).
- Small thumbnail: max_dim = 200
- Medium thumbnail: max_dim = 600

**`core/src/search.rs`:**
- `pub fn build_where_clause(filter: &SearchFilter) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>)` — construct parameterized WHERE clause from SearchFilter. Each non-None field adds an AND condition. `query` field does LIKE match against camera_model, camera_make, lens_model, extension.

### Step 3: Tauri app

**Initialize with:**
```bash
cd photo_app_rs
npm create tauri-app@latest tauri-app -- --template react-ts
```

But since you can't run interactive commands, set it up manually:

**`tauri-app/Cargo.toml`** (this goes inside `tauri-app/src-tauri/Cargo.toml`):
```toml
[package]
name = "photo-app-tauri"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = [] }
tauri-build = { version = "2", features = [] }
photo-app-core = { path = "../../core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
log = "0.4"
env_logger = "0.11"
base64 = "0.22"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**`tauri-app/src-tauri/tauri.conf.json`:**
```json
{
  "productName": "photo_app_rs",
  "version": "0.1.0",
  "identifier": "com.photo-app-rs.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "title": "photo_app_rs",
    "windows": [
      {
        "title": "photo_app_rs",
        "width": 1280,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
```

**Important**: Enable the `fs` and `dialog` plugins for Tauri v2 so the app can access files on external drives and open directory pickers. Add `tauri-plugin-dialog` and `tauri-plugin-fs` to the Cargo.toml and register them in main.rs.

**`tauri-app/src-tauri/src/main.rs`:**
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_directory,
            commands::get_timeline,
            commands::get_media_detail,
            commands::get_full_path,
            commands::get_stats,
            commands::search_media,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**`tauri-app/src-tauri/src/commands.rs`:**
- Each command opens a DB connection using `photo_app_core::db::open_db()`. Use a hardcoded default path for now: `~/.photo_app_rs/photo_app.db` (expand `~` with `dirs` crate or `std::env::var("HOME")`).
- `#[tauri::command] async fn scan_directory(path: String) -> Result<u64, String>` — calls `photo_app_core::indexer::scan_directory()`. For now, progress_callback can just log.
- `#[tauri::command] async fn get_timeline(offset: i64, limit: i64, filter: photo_app_core::models::SearchFilter) -> Result<Vec<photo_app_core::models::TimelineGroup>, String>`
- `#[tauri::command] async fn get_media_detail(id: i64) -> Result<photo_app_core::models::MediaFile, String>`
- `#[tauri::command] async fn get_full_path(id: i64) -> Result<String, String>` — returns the file path from file_locations. Frontend uses Tauri's asset protocol or `convertFileSrc()` to display it.
- `#[tauri::command] async fn get_stats() -> Result<photo_app_core::models::IndexStats, String>`
- `#[tauri::command] async fn search_media(filter: photo_app_core::models::SearchFilter) -> Result<Vec<photo_app_core::models::TimelineGroup>, String>`

### Step 4: Frontend

**Stack**: React 18 + TypeScript + Vite + Tailwind CSS + Zustand

**`package.json` dependencies:**
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-fs": "^2",
    "react": "^18",
    "react-dom": "^18",
    "react-virtuoso": "^4",
    "zustand": "^4"
  },
  "devDependencies": {
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "@vitejs/plugin-react": "^4",
    "autoprefixer": "^10",
    "postcss": "^8",
    "tailwindcss": "^3",
    "typescript": "^5",
    "vite": "^5"
  }
}
```

**Design**: Dark theme (bg-neutral-900, text-white). Minimal, clean. Think Apple Photos but dark.

**App layout:**
```
┌──────────────────────────────────────────────────┐
│ ┌────────┐ ┌──────────────────────────────────┐  │
│ │Sidebar │ │ Toolbar: [Search...] [Scan] Stats │  │
│ │        │ ├──────────────────────────────────┤  │
│ │ All    │ │                                  │  │
│ │ Photos │ │  ── March 8, 2025 ────────────── │  │
│ │ Videos │ │  [thumb] [thumb] [thumb] [thumb]  │  │
│ │        │ │  [thumb] [thumb]                  │  │
│ │ Stats  │ │                                  │  │
│ │        │ │  ── March 7, 2025 ────────────── │  │
│ │        │ │  [thumb] [thumb] [thumb]          │  │
│ │        │ │                                  │  │
│ └────────┘ └──────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

**Components:**

**`Sidebar.tsx`** — Fixed width 200px left panel. Buttons for: All Media, Photos Only, Videos Only. Shows IndexStats at the bottom (total count, size, date range). "Scan Directory" button that opens a native directory picker (`@tauri-apps/plugin-dialog` → `open({ directory: true })`) and calls `scan_directory` command.

**`SearchBar.tsx`** — Top bar with a search input. Debounce 300ms. Dropdown filters for camera model and extension (populated from stats). Dispatches filter changes to Zustand store.

**`Timeline.tsx`** — The main content area. Uses `react-virtuoso` `GroupedVirtuoso` component. Groups = date headers (e.g. "March 8, 2025"). Items = `MediaGrid` rows within each group. Loads data via `get_timeline` command with pagination (load 50 items at a time, load more on scroll). Each thumbnail is rendered as an `<img>` with `src="data:image/jpeg;base64,{thumb_small_b64}"`. Click on thumbnail → open `MediaViewer`.

**`MediaGrid.tsx`** — Renders a responsive grid of thumbnails for a single date group. CSS grid: `grid-template-columns: repeat(auto-fill, minmax(150px, 1fr))`. Each cell shows the thumbnail with a small overlay badge if it's a video (e.g., a play icon + duration). Aspect ratio: square crop (`object-fit: cover`).

**`MediaViewer.tsx`** — Full-screen overlay (fixed, z-50, bg-black/95). Shows the full-resolution image loaded from disk. Get the file path via `get_full_path(id)`, then convert to Tauri asset URL using `convertFileSrc()` from `@tauri-apps/api`. Left/right arrow key navigation. Escape to close. EXIF info panel on the right (toggleable with `i` key). For videos: `<video>` element with controls, src from convertFileSrc.

**`DeviceStatus.tsx`** — Small indicator showing whether the source directory is still accessible (just check if the root path exists by calling a simple Tauri command or using `@tauri-apps/plugin-fs`). Green dot = accessible, red = disconnected.

**`stores/appStore.ts`** (Zustand):
```typescript
interface AppState {
  filter: SearchFilter;
  setFilter: (f: Partial<SearchFilter>) => void;
  stats: IndexStats | null;
  setStats: (s: IndexStats) => void;
  viewingMedia: number | null; // media_file_id or null
  setViewingMedia: (id: number | null) => void;
  scanning: boolean;
  setScanning: (s: boolean) => void;
}
```

**`hooks/useTauri.ts`** — Typed wrappers:
```typescript
import { invoke } from '@tauri-apps/api/core';

export async function scanDirectory(path: string): Promise<number> {
  return invoke('scan_directory', { path });
}
export async function getTimeline(offset: number, limit: number, filter: SearchFilter): Promise<TimelineGroup[]> {
  return invoke('get_timeline', { offset, limit, filter });
}
// ... etc for each command
```

### Step 5: Root workspace update

Update root `Cargo.toml`:
```toml
[workspace]
members = ["analytics", "core", "tauri-app/src-tauri"]
resolver = "2"
```

---

## How to verify it works

1. `cd tauri-app && npm install`
2. `cd tauri-app && npm run tauri dev`
3. App window opens → click "Scan Directory" → select a folder with photos
4. Scanning progress appears → after scan, timeline populates with thumbnails
5. Scroll the timeline → virtualized, smooth, grouped by date
6. Click a thumbnail → full-res viewer opens → arrow keys navigate → Escape closes
7. Type in search bar → filters results by camera model or text
8. Filter by Photos/Videos in sidebar → timeline updates

---

## Constraints

- **Do NOT modify anything in `analytics/`** — it's the existing crate, leave it alone
- **Do NOT build backup features** — no backup engine, no portable DB, no backup UI
- **Do NOT add face detection, map view, or albums** — those are post-MVP
- macOS is the primary target (the developer uses macOS with external drives)
- Use Tauri v2 (not v1)
- Database path: `~/.photo_app_rs/photo_app.db` (create directory if not exists)
- Thumbnails stored as BLOBs in SQLite, NOT as files
- Full-res images are NEVER copied locally — always read from source path
- If a file extension is not recognized, skip it silently
- If ffmpeg/ffprobe is not installed, skip video thumbnails and video metadata gracefully (log warning, don't crash)
- All Tauri commands must be async and return `Result<T, String>`

## Error handling

- All Rust functions return `Result<T, E>` — no `.unwrap()` in library code (`.unwrap()` only in main.rs if truly unrecoverable)
- Frontend shows toast/notification for errors from Tauri commands
- If scan encounters an unreadable file, log and skip — do not abort the whole scan

## Performance targets

- Scan 1000 files: under 2 minutes (parallel hashing with rayon)
- Timeline scroll: 60fps (react-virtuoso handles this, just don't block the main thread)
- Thumbnail load: under 10ms (SQLite BLOB read by primary key)
- App binary size: under 30MB