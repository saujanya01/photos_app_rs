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
