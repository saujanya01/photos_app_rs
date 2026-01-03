use std::fs;
use std::time::SystemTime;

use rusqlite::Connection;

use crate::database::models::MediaFileRow;
use crate::utils::core::Media;

pub fn new_backup_session(
    conn: &Connection,
    source_path: &str,
    destination_path: &str,
) -> rusqlite::Result<i64> {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Use RETURNING clause to get the ID directly from the INSERT
    // This eliminates race conditions in multi-threaded/async contexts
    let session_id = conn.query_row(
        "INSERT INTO backup_sessions (source_path, destination_path, started_at, status) VALUES (?1, ?2, ?3, 'running') RETURNING id",
        (source_path, destination_path, now as i64),
        |row| row.get::<_, i64>(0),
    )?;

    Ok(session_id)
}

pub fn update_backup_session_completed(conn: &Connection, session_id: i64) -> rusqlite::Result<()> {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    conn.execute(
        "UPDATE backup_sessions SET status = 'completed', completed_at = ?1 WHERE id = ?2",
        (now as i64, session_id),
    )?;

    Ok(())
}
pub fn insert_media_file(conn: &Connection, media: &Media) -> rusqlite::Result<()> {
    // We know the file exists since we just created Media from it, so unwrap is safe
    let metadata = fs::metadata(&media.file_path).unwrap();

    let media_file_row = MediaFileRow {
        id: None,
        hash: media.hash.clone(),
        file_size_bytes: media.file_size as i64,
        media_type: media.file_type.to_string(),
        extension: media
            .file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string(),
        camera_make: media.exif_data.as_ref().and_then(|e| e.camera_make.clone()),
        camera_model: media
            .exif_data
            .as_ref()
            .and_then(|e| e.camera_model.clone()),
        lens_model: media.exif_data.as_ref().and_then(|e| e.lens_model.clone()),
        date_taken: media.exif_data.as_ref().and_then(|e| e.date_taken.clone()),
        iso: media.exif_data.as_ref().and_then(|e| e.iso.clone()),
        aperture: media.exif_data.as_ref().and_then(|e| e.aperture.clone()),
        shutter_speed: media
            .exif_data
            .as_ref()
            .and_then(|e| e.shutter_speed.clone()),
        focal_length: media
            .exif_data
            .as_ref()
            .and_then(|e| e.focal_length.clone()),
        software: media.exif_data.as_ref().and_then(|e| e.software.clone()),
        duration_seconds: None,
        resolution_width: None,
        resolution_height: None,
        path: media.file_path.to_str().unwrap().to_string(),
        date_added: metadata
            .created()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        date_modified: metadata
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    };

    conn.execute(
        "INSERT INTO media_files (hash, file_size_bytes, media_type, extension, camera_make, camera_model, lens_model, date_taken, iso, aperture, shutter_speed, focal_length, software, duration_seconds, resolution_width, resolution_height, path, date_added, date_modified) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        rusqlite::params![
            media_file_row.hash,
            media_file_row.file_size_bytes as i64,
            media_file_row.media_type,
            media_file_row.extension,
            media_file_row.camera_make,
            media_file_row.camera_model,
            media_file_row.lens_model,
            media_file_row.date_taken,
            media_file_row.iso,
            media_file_row.aperture,
            media_file_row.shutter_speed,
            media_file_row.focal_length,
            media_file_row.software,
            media_file_row.duration_seconds,
            media_file_row.resolution_width,
            media_file_row.resolution_height,
            media_file_row.path,
            media_file_row.date_added as i64,
            media_file_row.date_modified as i64
        ],
    )?;
    Ok(())
}

pub fn insert_duplicate_group(conn: &Connection, media: &Media) -> rusqlite::Result<()> {
    // Get the media_file_id by querying with hash
    let media_file_id: i64 = conn.query_row(
        "SELECT id FROM media_files WHERE hash = ?1",
        [&media.hash],
        |row| row.get::<_, i64>(0),
    )?;

    // Count how many files have the same hash (total_copies)
    let total_copies: i32 = conn.query_row(
        "SELECT COUNT(*) FROM media_files WHERE hash = ?1",
        [&media.hash],
        |row| row.get::<_, i32>(0),
    )?;

    // Calculate total_size_bytes = file_size * total_copies
    let total_size_bytes = media.file_size as i64 * total_copies as i64;

    // Calculate wasted_space_bytes = file_size * (total_copies - 1)
    // This matches the formula from main.rs: duplicate.file_size * (duplicate.count as u64 - 1)
    let wasted_space_bytes = media.file_size as i64 * (total_copies as i64 - 1);

    // Insert into duplicate_groups table
    conn.execute(
        "INSERT INTO duplicate_groups (media_file_id, total_copies, total_size_bytes, wasted_space_bytes) 
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            media_file_id,
            total_copies,
            total_size_bytes,
            wasted_space_bytes
        ],
    )?;

    Ok(())
}
