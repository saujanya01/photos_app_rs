/// Represents a media file in the database
pub struct MediaFileRow {
    pub id: Option<i64>,
    pub hash: String,
    pub file_size_bytes: i64,
    pub media_type: String,
    pub extension: String,

    // EXIF data
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub date_taken: Option<String>,
    pub iso: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<String>,
    pub software: Option<String>,

    // Video specific
    pub duration_seconds: Option<f64>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,

    // File system
    pub path: String,
    pub date_added: i64,
    pub date_modified: i64,
}

pub struct DuplicateGroupRow {
    pub id: Option<i64>,
    pub media_file_id: i64,
    pub total_copies: i32,
    pub total_size_bytes: i64,
    pub wasted_space_bytes: i64,
}

pub struct BackupSessionRow {
    pub id: Option<i64>,
    pub source_path: String,
    pub destination_path: String,
    pub files_scanned: i32,
    pub files_copied: i32,
    pub files_skipped: i32,
    pub bytes_copied: i64,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub duration_seconds: Option<i32>,
    pub status: String,
    pub error_message: Option<String>,
}
