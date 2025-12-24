use exif::{In, Reader, Tag};
use serde::Serialize;
use std::io::Write;
use std::thread::panicking;
use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

pub mod duplicates;

#[derive(Debug, Clone, Serialize)]
pub enum ImageFormat {
    Jpg,
    Jpeg,
    Arw,
    Png,
    Heic,
    Tiff,
    Gif,
    Bmp,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "jpg" => Some(Self::Jpg),
            "jpeg" => Some(Self::Jpeg),
            "arw" => Some(Self::Arw),
            "png" => Some(Self::Png),
            "heic" => Some(Self::Heic),
            "tiff" | "tif" => Some(Self::Tiff),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum VideoFormat {
    Mp4,
    Mov,
    Avi,
    Mkv,
}

impl VideoFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "mp4" => Some(Self::Mp4),
            "mov" => Some(Self::Mov),
            "avi" => Some(Self::Avi),
            "mkv" => Some(Self::Mkv),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum FileType {
    Image(ImageFormat),
    Video(VideoFormat),
    Folder,
}

impl FileType {
    pub fn from_path(path: &Path) -> Option<Self> {
        if path.is_dir() {
            return Some(FileType::Folder);
        }

        if path.is_file() {
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if let Some(format) = ImageFormat::from_extension(&extension) {
                return Some(FileType::Image(format));
            }

            if let Some(format) = VideoFormat::from_extension(&extension) {
                return Some(FileType::Video(format));
            }
        }

        None
    }

    pub fn is_image(&self) -> bool {
        matches!(self, FileType::Image(_))
    }

    pub fn is_video(&self) -> bool {
        matches!(self, FileType::Video(_))
    }
}

#[derive(Debug, Clone, Serialize)]
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

impl ExifData {
    pub fn from_file(path: &Path) -> Option<Self> {
        let file = File::open(path).ok()?;
        let mut bufreader = BufReader::new(&file);
        let exifreader = Reader::new().read_from_container(&mut bufreader).ok()?;

        Some(ExifData {
            camera_make: exifreader
                .get_field(Tag::Make, In::PRIMARY)
                .map(|f| f.display_value().to_string()),
            camera_model: exifreader
                .get_field(Tag::Model, In::PRIMARY)
                .map(|f| f.display_value().to_string()),
            lens_model: exifreader
                .get_field(Tag::LensModel, In::PRIMARY)
                .map(|f| f.display_value().to_string()),
            date_taken: exifreader
                .get_field(Tag::DateTime, In::PRIMARY)
                .or_else(|| exifreader.get_field(Tag::DateTimeOriginal, In::PRIMARY))
                .map(|f| f.display_value().to_string()),
            iso: exifreader
                .get_field(Tag::PhotographicSensitivity, In::PRIMARY)
                .map(|f| f.display_value().to_string()),
            aperture: exifreader
                .get_field(Tag::FNumber, In::PRIMARY)
                .map(|f| f.display_value().to_string()),
            shutter_speed: exifreader
                .get_field(Tag::ExposureTime, In::PRIMARY)
                .map(|f| f.display_value().to_string()),
            focal_length: exifreader
                .get_field(Tag::FocalLength, In::PRIMARY)
                .map(|f| f.display_value().to_string()),
            software: exifreader
                .get_field(Tag::Software, In::PRIMARY)
                .map(|f| f.display_value().to_string()),
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Media {
    pub file_path: PathBuf,
    pub file_name: String,
    pub file_type: FileType,
    pub file_size: u64,
    pub exif_data: Option<ExifData>,
}

impl Media {
    pub fn new(path: &Path) -> io::Result<Self> {
        let file_type = FileType::from_path(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Not a media file"))?;

        if matches!(file_type, FileType::Folder) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot create Media from a folder",
            ));
        }

        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Extract EXIF data for images
        let exif_data = if file_type.is_image() {
            ExifData::from_file(path)
        } else {
            None
        };

        Ok(Media {
            file_path: path.to_path_buf(),
            file_name,
            file_type,
            file_size,
            exif_data,
        })
    }

    pub fn file_size_human_readable(&self) -> String {
        let size = self.file_size as f64;
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
}
pub fn scan_directory(path: &Path, media_items: &mut Vec<Media>) -> io::Result<()> {
    if !path.is_dir() {
        return Ok(());
    }

    println!("Scanning dir");

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_directory(&path, media_items);
        } else {
            match Media::new(&path) {
                Ok(media) => {
                    println!("Scanned Media {:?}", media.file_name);
                    media_items.push(media);
                }
                Err(e) => {
                    println!(
                        "Error while creating; Path : {:?}; Error : {:?}",
                        path.to_str(),
                        e
                    )
                }
            }
        }
    }

    println!("Done media scan");

    Ok(())
}

// Export to JSON
pub fn export_to_json(media_items: &[Media], output_path: &str) -> io::Result<()> {
    let json = serde_json::to_string_pretty(media_items)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;

    println!("✅ Exported to {}", output_path);
    Ok(())
}

pub fn export_to_csv(media_items: &[Media], output_path: &str) -> io::Result<()> {
    let mut wtr =
        csv::Writer::from_path(output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Write headers
    wtr.write_record(&[
        "File Name",
        "File Path",
        "File Type",
        "File Size (bytes)",
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

    // Write data
    for media in media_items {
        wtr.write_record(&[
            &media.file_name,
            media.file_path.to_str().unwrap_or(""),
            &format!("{:?}", media.file_type),
            &media.file_size.to_string(),
            &media.file_size_human_readable(),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.camera_make.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.camera_model.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.lens_model.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.date_taken.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.iso.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.aperture.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.shutter_speed.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.focal_length.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            media
                .exif_data
                .as_ref()
                .and_then(|e| e.software.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
        ])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    wtr.flush()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("✅ Exported to {}", output_path);
    Ok(())
}
