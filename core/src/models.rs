use serde::{Deserialize, Serialize};

/// Represents a media file in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: Option<i64>,
    pub hash: String,
    pub file_size_bytes: i64,
    pub media_type: String, // "image" or "video"
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
    pub path: String,
    pub date_added: i64,
    pub date_modified: i64,

    // v0.6 additions — populated lazily by later phases
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub dominant_color: Option<String>,
    pub rating: i32,
}

/// User-curated album. ID is a TEXT slug (e.g. "ed26") to match the
/// portable-sidecar design — autoincrement IDs would collide across drives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cover_media_id: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A group of media items for a specific date in the timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineGroup {
    pub date: String,
    pub media: Vec<MediaItem>,
}

/// A single media item for display in the timeline grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: i64,
    pub media_type: String,
    pub extension: String,
    pub date_taken: Option<String>,
    pub camera_model: Option<String>,
    pub file_path: String,
    pub thumb_small_b64: Option<String>,
}

/// Filter criteria for searching media
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub media_type: Option<String>,
    pub camera_model: Option<String>,
    pub extension: Option<String>,
    pub query: Option<String>,
}

/// Progress information during a directory scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total_files: u64,
    pub scanned: u64,
    pub new_files: u64,
    pub current_file: String,
}

/// Statistics about the indexed media library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_media: i64,
    pub total_images: i64,
    pub total_videos: i64,
    pub total_size_bytes: i64,
    pub cameras: Vec<String>,
    pub date_range: (Option<String>, Option<String>),
}

/// EXIF metadata extracted from a media file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifData {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub date_taken: Option<String>,
    pub iso: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<String>,
    pub software: Option<String>,
}

/// Video metadata extracted from ffprobe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub duration_seconds: Option<f64>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub creation_time: Option<String>,
}
