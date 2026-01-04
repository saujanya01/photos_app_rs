mod database;
mod utils;

use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::database::connection::Database;
use crate::database::operations;
use crate::utils::core::{export_images_to_new_destination, scan_directory};
use crate::utils::duplicates::{Duplicates, find_duplicates};

const ROOT_PROJECT_PATH: &str = "/Users/saujanya/sandisk_media";
// const ROOT_PROJECT_PATH: &str = "./test_folder";
fn main() -> io::Result<()> {
    let source_path = ROOT_PROJECT_PATH;

    let destination_path = format!("{}/final_export", ROOT_PROJECT_PATH);

    let db_path = format!("{}/.photo_app_rs/sqlite.db", ".");
    let db = Database::new(&db_path).unwrap_or_else(|e| {
        panic!("Error connecting to database: {}", e);
    });

    let backup_session_id = database::operations::new_backup_session(
        &db.conn(),
        source_path.as_ref(),
        destination_path.as_ref(),
    )
    .unwrap();

    // Set up graceful shutdown handler
    let session_id_shared = Arc::new(Mutex::new(Some(backup_session_id)));
    let db_path_shared = Arc::new(db_path.clone());
    let session_id_for_handler = session_id_shared.clone();
    let db_path_for_handler = db_path_shared.clone();

    ctrlc::set_handler(move || {
        println!("\nReceived interrupt signal. Cancelling backup session...");

        if let Ok(session_id_guard) = session_id_for_handler.lock() {
            if let Some(session_id) = *session_id_guard {
                if let Ok(db) = Database::new(&*db_path_for_handler) {
                    if let Err(e) =
                        operations::update_backup_session_cancelled(db.conn(), session_id)
                    {
                        eprintln!("Error updating backup session to cancelled: {}", e);
                    } else {
                        println!("Backup session marked as cancelled.");
                    }
                } else {
                    eprintln!("Error connecting to database for cleanup.");
                }
            }
        }

        std::process::exit(130); // Standard exit code for SIGINT
    })
    .expect("Error setting Ctrl+C handler");

    let media_items = scan_directory(&db.conn(), Path::new(source_path))?;

    let duplicates = find_duplicates(&db.conn(), media_items, Path::new(&destination_path))?;

    export_to_csv(duplicates.clone(), "./csv_exports/final_export.csv")?;

    let waste_space = calculate_waste_space(duplicates.clone());

    println!("Waste Space : {:?}", format_size(waste_space));

    export_images_to_new_destination(duplicates)?;

    // Clear the session_id from shared state since we're completing normally
    *session_id_shared.lock().unwrap() = None;

    operations::update_backup_session_completed(&db.conn(), backup_session_id).unwrap();

    Ok(())
}

fn calculate_waste_space(data: Vec<Duplicates>) -> u64 {
    let mut total_waste_space = 0;

    for duplicate in data {
        total_waste_space += duplicate.file_size * (duplicate.count as u64 - 1);
    }

    total_waste_space
}

fn export_to_csv(data: Vec<Duplicates>, output_path: &str) -> io::Result<()> {
    let mut wrt =
        csv::Writer::from_path(output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    wrt.write_record(&[
        "Hash",
        "File Name",
        "Media Type",
        "Count",
        "Final File Path",
        "Old File Path(s)",
        "File Size (Bytes)",
        "File Size (Human)",
        "Camera Make",
        "Camera Model",
        "Lens Model",
        "Date Taken",
        "ISO",
        "Aperture",
        "Shutter Speed",
        "Focal Length",
        "Software",
    ])
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    for media in data {
        // Store temporary String values to avoid dangling references
        let count_str = media.count.to_string();
        let final_path = media.final_path;
        let old_paths = media
            .files
            .iter()
            .filter_map(|f| f.to_str())
            .collect::<Vec<_>>()
            .join(";");
        let file_size_str = media.file_size.to_string();
        let file_size_human = format_size(media.file_size);

        wrt.write_record(&[
            &media.hash,
            &media.media.file_name,
            &media.file_type.to_string(),
            &count_str,
            final_path.to_str().unwrap_or(""),
            &old_paths,
            &file_size_str,
            &file_size_human,
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.camera_make.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.camera_model.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.lens_model.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.date_taken.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.iso.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.aperture.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.shutter_speed.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.focal_length.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .media
                .exif_data
                .as_ref()
                .and_then(|e| e.software.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
        ])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    wrt.flush()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

pub fn format_size(size: u64) -> String {
    let size = size as f64;
    if size < 1024.0 {
        format!("{} B", size)
    } else if size < 1024.0 * 1024.0 {
        format!("{:.2} KB", size / 1024.0)
    } else if size < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} MB", size / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
    }
}
