use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::ImageReader;
use image::DynamicImage;
use std::io::Cursor;
use std::path::Path;
use std::process::Command;

/// Small thumbnail size (200px)
pub const THUMB_SMALL: u32 = 200;

/// Medium thumbnail size (600px)
pub const THUMB_MEDIUM: u32 = 600;

/// Generate an image thumbnail
/// For RAW files, tries to extract embedded JPEG first
pub fn generate_image_thumbnail(path: &str, max_dim: u32) -> Result<Vec<u8>, String> {
    let path = Path::new(path);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // For RAW files, try to extract embedded JPEG from EXIF
    if is_raw_extension(&extension) {
        if let Ok(thumb) = extract_raw_embedded_jpeg(path, max_dim) {
            return Ok(thumb);
        }
        // If extraction fails, return error - we don't decode full RAW
        return Err(format!(
            "Could not extract embedded JPEG from RAW file: {}",
            path.display()
        ));
    }

    // For standard image formats, use the image crate
    let img = ImageReader::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    resize_and_encode_jpeg(&img, max_dim)
}

/// Generate a video thumbnail by extracting a frame with ffmpeg
pub fn generate_video_thumbnail(path: &str, max_dim: u32) -> Result<Vec<u8>, String> {
    // Extract a frame at 2 seconds into the video using ffmpeg
    let output = Command::new("ffmpeg")
        .args([
            "-ss",
            "2",
            "-i",
            path,
            "-frames:v",
            "1",
            "-q:v",
            "2",
            "-f",
            "image2pipe",
            "-vcodec",
            "mjpeg",
            "pipe:1",
        ])
        .output()
        .map_err(|e| format!("ffmpeg not found or failed to run: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffmpeg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    if output.stdout.is_empty() {
        return Err("ffmpeg produced no output".to_string());
    }

    // Resize the extracted frame
    let img = image::load_from_memory(&output.stdout)
        .map_err(|e| format!("Failed to load ffmpeg output: {}", e))?;

    resize_and_encode_jpeg(&img, max_dim)
}

/// Check if an extension is a RAW format
fn is_raw_extension(ext: &str) -> bool {
    matches!(ext, "arw" | "cr2" | "nef" | "dng" | "orf" | "rw2" | "pef")
}

/// Try to extract the embedded JPEG thumbnail from a RAW file's EXIF data
fn extract_raw_embedded_jpeg(path: &Path, max_dim: u32) -> Result<Vec<u8>, String> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut bufreader = BufReader::new(file);

    let exif = exif::Reader::new()
        .read_from_container(&mut bufreader)
        .map_err(|e| format!("Failed to read EXIF: {}", e))?;

    // Look for the thumbnail in EXIF
    // The thumbnail is usually stored in the EXIF IFD1 as a JPEG
    for field in exif.fields() {
        if field.tag == exif::Tag::JPEGInterchangeFormat {
            // Found thumbnail offset, but we need the actual bytes
            // This is tricky - for now, use dcraw or exiftool approach
            // Fallback: use exiftool to extract thumbnail
            return extract_thumbnail_with_exiftool(path, max_dim);
        }
    }

    Err("No embedded thumbnail found".to_string())
}

/// Use exiftool to extract embedded thumbnail (fallback method)
fn extract_thumbnail_with_exiftool(path: &Path, max_dim: u32) -> Result<Vec<u8>, String> {
    let output = Command::new("exiftool")
        .args(["-b", "-ThumbnailImage", path.to_str().unwrap_or("")])
        .output()
        .map_err(|e| format!("exiftool not found or failed: {}", e))?;

    if !output.status.success() || output.stdout.is_empty() {
        // Try PreviewImage instead
        let output = Command::new("exiftool")
            .args(["-b", "-PreviewImage", path.to_str().unwrap_or("")])
            .output()
            .map_err(|e| format!("exiftool failed: {}", e))?;

        if output.stdout.is_empty() {
            return Err("No thumbnail or preview found".to_string());
        }

        let img = image::load_from_memory(&output.stdout)
            .map_err(|e| format!("Failed to decode preview: {}", e))?;
        return resize_and_encode_jpeg(&img, max_dim);
    }

    let img = image::load_from_memory(&output.stdout)
        .map_err(|e| format!("Failed to decode thumbnail: {}", e))?;
    resize_and_encode_jpeg(&img, max_dim)
}

/// Resize image to fit within max_dim and encode as JPEG
fn resize_and_encode_jpeg(img: &DynamicImage, max_dim: u32) -> Result<Vec<u8>, String> {
    let (width, height) = (img.width(), img.height());

    // Calculate new dimensions maintaining aspect ratio
    let (new_width, new_height) = if width > height {
        let ratio = max_dim as f32 / width as f32;
        (max_dim, (height as f32 * ratio) as u32)
    } else {
        let ratio = max_dim as f32 / height as f32;
        ((width as f32 * ratio) as u32, max_dim)
    };

    // Only resize if the image is larger than max_dim
    let resized = if width > max_dim || height > max_dim {
        img.resize(new_width, new_height, FilterType::Lanczos3)
    } else {
        img.clone()
    };

    // Encode as JPEG with quality 80
    let mut buffer = Cursor::new(Vec::new());
    let encoder = JpegEncoder::new_with_quality(&mut buffer, 80);

    resized
        .to_rgb8()
        .write_with_encoder(encoder)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

    Ok(buffer.into_inner())
}
