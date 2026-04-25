use rayon::prelude::*;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::db::{hash_exists, insert_media_file, insert_thumbnail};
use crate::models::{ExifData, MediaFile, ScanProgress, VideoMetadata};
use crate::thumbnailer::{generate_image_thumbnail, generate_video_thumbnail, THUMB_MEDIUM, THUMB_SMALL};

const PARTIAL_HASH_SIZE: usize = 128 * 1024; // 128 KB

// Image extensions
const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "tiff", "tif", "arw", "cr2", "nef", "dng", "heic", "heif", "webp", "bmp",
    "gif",
];

// Video extensions
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "mts", "m4v"];

/// Scan a directory, index all media files, and generate thumbnails
pub fn scan_directory<F>(
    conn: &Connection,
    dir_path: &str,
    progress_callback: F,
) -> Result<u64, String>
where
    F: Fn(ScanProgress) + Send + Sync,
{
    let dir = Path::new(dir_path);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir_path));
    }

    // Collect all media files first
    let mut media_paths: Vec<(String, String)> = Vec::new(); // (path, media_type)

    for entry in WalkDir::new(dir).follow_links(true).into_iter().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            let media_type = if IMAGE_EXTENSIONS.contains(&ext_lower.as_str()) {
                "image"
            } else if VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
                "video"
            } else {
                continue;
            };

            if let Some(path_str) = path.to_str() {
                media_paths.push((path_str.to_string(), media_type.to_string()));
            }
        }
    }

    let total_files = media_paths.len() as u64;
    let scanned = Arc::new(AtomicU64::new(0));
    let new_files = Arc::new(AtomicU64::new(0));

    // Initial progress
    progress_callback(ScanProgress {
        total_files,
        scanned: 0,
        new_files: 0,
        current_file: "Starting scan...".to_string(),
    });

    // Process files - we compute hashes in parallel but write to DB sequentially
    // First, compute hashes in parallel
    let file_data: Vec<_> = media_paths
        .par_iter()
        .filter_map(|(path, media_type)| {
            let scanned_count = scanned.fetch_add(1, Ordering::SeqCst) + 1;

            // Compute partial hash first
            let partial_hash = match compute_partial_hash(path) {
                Ok(h) => h,
                Err(e) => {
                    log::warn!("Failed to hash {}: {}", path, e);
                    return None;
                }
            };

            // Report progress
            progress_callback(ScanProgress {
                total_files,
                scanned: scanned_count,
                new_files: new_files.load(Ordering::SeqCst),
                current_file: path.clone(),
            });

            Some((path.clone(), media_type.clone(), partial_hash))
        })
        .collect();

    // Now process sequentially for DB operations
    for (path, media_type, hash) in file_data {
        // Check if hash exists
        match hash_exists(conn, &hash) {
            Ok(true) => {
                log::debug!("Skipping duplicate: {}", path);
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                log::warn!("DB error checking hash: {}", e);
                continue;
            }
        }

        // Process new file
        match process_new_file(conn, &path, &media_type, &hash) {
            Ok(_) => {
                new_files.fetch_add(1, Ordering::SeqCst);
            }
            Err(e) => {
                log::warn!("Failed to process {}: {}", path, e);
            }
        }
    }

    let final_new = new_files.load(Ordering::SeqCst);

    progress_callback(ScanProgress {
        total_files,
        scanned: total_files,
        new_files: final_new,
        current_file: "Scan complete".to_string(),
    });

    Ok(final_new)
}

fn compute_partial_hash(path: &str) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut buffer = vec![0u8; PARTIAL_HASH_SIZE];
    let bytes_read = file
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer[..bytes_read]);

    Ok(format!("{:x}", hasher.finalize()))
}

fn process_new_file(
    conn: &Connection,
    path: &str,
    media_type: &str,
    hash: &str,
) -> Result<(), String> {
    let file_path = Path::new(path);

    // Get file metadata
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get metadata: {}", e))?;
    let file_size = metadata.len() as i64;
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Extract EXIF/metadata
    let (exif_data, video_meta) = if media_type == "image" {
        (extract_exif(file_path), None)
    } else {
        (None, extract_video_metadata(path))
    };

    // Get timestamps
    let date_added = metadata
        .created()
        .unwrap_or(SystemTime::now())
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let date_modified = metadata
        .modified()
        .unwrap_or(SystemTime::now())
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Build MediaFile struct
    let media_file = MediaFile {
        id: None,
        hash: hash.to_string(),
        file_size_bytes: file_size,
        media_type: media_type.to_string(),
        extension,
        camera_make: exif_data.as_ref().and_then(|e| e.camera_make.clone()),
        camera_model: exif_data.as_ref().and_then(|e| e.camera_model.clone()),
        lens_model: exif_data.as_ref().and_then(|e| e.lens_model.clone()),
        date_taken: exif_data
            .as_ref()
            .and_then(|e| e.date_taken.clone())
            .or_else(|| video_meta.as_ref().and_then(|v| v.creation_time.clone())),
        iso: exif_data.as_ref().and_then(|e| e.iso.clone()),
        aperture: exif_data.as_ref().and_then(|e| e.aperture.clone()),
        shutter_speed: exif_data.as_ref().and_then(|e| e.shutter_speed.clone()),
        focal_length: exif_data.as_ref().and_then(|e| e.focal_length.clone()),
        software: exif_data.as_ref().and_then(|e| e.software.clone()),
        duration_seconds: video_meta.as_ref().and_then(|v| v.duration_seconds),
        resolution_width: video_meta.as_ref().and_then(|v| v.resolution_width),
        resolution_height: video_meta.as_ref().and_then(|v| v.resolution_height),
        path: path.to_string(),
        date_added,
        date_modified,
    };

    // Insert into database
    let media_id =
        insert_media_file(conn, &media_file).map_err(|e| format!("DB insert failed: {}", e))?;

    // Generate thumbnails
    let thumb_result = if media_type == "image" {
        generate_image_thumbnail(path, THUMB_SMALL)
    } else {
        generate_video_thumbnail(path, THUMB_SMALL)
    };

    if let Ok(thumb_small) = thumb_result {
        // Try to generate medium thumbnail
        let thumb_medium = if media_type == "image" {
            generate_image_thumbnail(path, THUMB_MEDIUM).ok()
        } else {
            generate_video_thumbnail(path, THUMB_MEDIUM).ok()
        };

        if let Err(e) = insert_thumbnail(
            conn,
            media_id,
            &thumb_small,
            thumb_medium.as_deref(),
        ) {
            log::warn!("Failed to save thumbnail: {}", e);
        }
    } else if let Err(e) = thumb_result {
        log::warn!("Failed to generate thumbnail for {}: {}", path, e);
    }

    Ok(())
}

fn extract_exif(path: &Path) -> Option<ExifData> {
    let file = File::open(path).ok()?;
    let mut bufreader = BufReader::new(file);
    let exif = exif::Reader::new()
        .read_from_container(&mut bufreader)
        .ok()?;

    Some(ExifData {
        camera_make: exif
            .get_field(exif::Tag::Make, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string()),
        camera_model: exif
            .get_field(exif::Tag::Model, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string()),
        lens_model: exif
            .get_field(exif::Tag::LensModel, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string()),
        date_taken: exif
            .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
            .or_else(|| exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY))
            .map(|f| f.display_value().to_string()),
        iso: exif
            .get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string()),
        aperture: exif
            .get_field(exif::Tag::FNumber, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string()),
        shutter_speed: exif
            .get_field(exif::Tag::ExposureTime, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string()),
        focal_length: exif
            .get_field(exif::Tag::FocalLength, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string()),
        software: exif
            .get_field(exif::Tag::Software, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string()),
    })
}

fn extract_video_metadata(path: &str) -> Option<VideoMetadata> {
    // Use ffprobe to get video metadata
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;

    // Extract duration from format
    let duration = json
        .get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    // Extract resolution from first video stream
    let mut width = None;
    let mut height = None;

    if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
        for stream in streams {
            if stream.get("codec_type").and_then(|c| c.as_str()) == Some("video") {
                width = stream.get("width").and_then(|w| w.as_i64()).map(|w| w as i32);
                height = stream
                    .get("height")
                    .and_then(|h| h.as_i64())
                    .map(|h| h as i32);
                break;
            }
        }
    }

    // Extract creation time from format tags
    let creation_time = json
        .get("format")
        .and_then(|f| f.get("tags"))
        .and_then(|t| t.get("creation_time"))
        .and_then(|c| c.as_str())
        .map(|s| {
            // Convert ISO format to our format (YYYY-MM-DD HH:MM:SS)
            s.replace('T', " ")
                .split('.')
                .next()
                .unwrap_or(s)
                .replace('Z', "")
        });

    Some(VideoMetadata {
        duration_seconds: duration,
        resolution_width: width,
        resolution_height: height,
        creation_time,
    })
}
