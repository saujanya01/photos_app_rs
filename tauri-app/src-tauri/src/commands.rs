use photo_app_core::db;
use photo_app_core::indexer;
use photo_app_core::models::{IndexStats, MediaFile, SearchFilter, TimelineGroup};

/// Get the database path (~/.photo_app_rs/photo_app.db)
fn get_db_path() -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let db_dir = home.join(".photo_app_rs");

    std::fs::create_dir_all(&db_dir)
        .map_err(|e| format!("Failed to create database directory: {}", e))?;

    let db_path = db_dir.join("photo_app.db");
    db_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid path".to_string())
}

/// Open a database connection
fn open_connection() -> Result<rusqlite::Connection, String> {
    let db_path = get_db_path()?;
    db::open_db(&db_path).map_err(|e| format!("Failed to open database: {}", e))
}

/// Scan a directory and index all media files
#[tauri::command]
pub async fn scan_directory(path: String) -> Result<u64, String> {
    // Run scanning in a blocking task since it does heavy I/O
    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_connection()?;
        indexer::scan_directory(&conn, &path, |progress| {
            log::info!(
                "Scan progress: {}/{} files, {} new ({})",
                progress.scanned,
                progress.total_files,
                progress.new_files,
                progress.current_file
            );
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get timeline data with pagination and filtering
#[tauri::command]
pub async fn get_timeline(
    offset: i64,
    limit: i64,
    filter: SearchFilter,
) -> Result<Vec<TimelineGroup>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_connection()?;
        db::get_timeline(&conn, offset, limit, &filter)
            .map_err(|e| format!("Failed to get timeline: {}", e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get full details for a single media file
#[tauri::command]
pub async fn get_media_detail(id: i64) -> Result<MediaFile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_connection()?;
        db::get_media_detail(&conn, id).map_err(|e| format!("Failed to get media detail: {}", e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get the file path for a media file (for full-res viewing)
#[tauri::command]
pub async fn get_full_path(id: i64) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_connection()?;
        db::get_file_path(&conn, id).map_err(|e| format!("Failed to get file path: {}", e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get library statistics
#[tauri::command]
pub async fn get_stats() -> Result<IndexStats, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_connection()?;
        db::get_stats(&conn).map_err(|e| format!("Failed to get stats: {}", e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Search media with filters
#[tauri::command]
pub async fn search_media(filter: SearchFilter) -> Result<Vec<TimelineGroup>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_connection()?;
        db::get_timeline(&conn, 0, 500, &filter)
            .map_err(|e| format!("Failed to search media: {}", e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// ============================================================
// Preferences — Phase 3
// Persisted as JSON at ~/.photo_app_rs/preferences.json. The shape
// is opaque to Rust; the frontend owns the schema (see prefs.ts).
// ============================================================
fn get_prefs_path() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let dir = home.join(".photo_app_rs");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create app directory: {}", e))?;
    Ok(dir.join("preferences.json"))
}

#[tauri::command]
pub async fn read_preferences() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let path = get_prefs_path()?;
        match std::fs::read_to_string(&path) {
            Ok(s) => Ok(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("{}".to_string()),
            Err(e) => Err(format!("Failed to read preferences: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn write_preferences(json: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        // Validate that what we're being asked to persist is JSON.
        serde_json::from_str::<serde_json::Value>(&json)
            .map_err(|e| format!("Invalid JSON: {}", e))?;
        let path = get_prefs_path()?;
        // Atomic-ish write: tmp file then rename, so a crash mid-write
        // can never leave a half-written preferences file.
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json.as_bytes())
            .map_err(|e| format!("Failed to write preferences: {}", e))?;
        std::fs::rename(&tmp, &path)
            .map_err(|e| format!("Failed to commit preferences: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
