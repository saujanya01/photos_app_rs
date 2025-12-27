mod utils;

use std::io;
use std::time::Instant;

use crate::utils::duplicates::{Duplicates, find_duplicates};
use crate::utils::{export_images_to_new_destination, scan_directory};

fn main() -> io::Result<()> {
    // let path = "./test_folder";

    let path = "/Users/saujanya/sandisk_media";

    // Measure time for directory scanning
    let scan_start = Instant::now();

    let media_items = scan_directory(path.as_ref())?;

    let scan_duration = scan_start.elapsed();

    let duplicates = find_duplicates(media_items)?;

    println!("Scan Duration : {:?}", scan_duration);

    export_to_csv(duplicates.clone(), "./final_export.csv")?;

    let waste_space = calculate_waste_space(duplicates.clone());
    println!("Waste Space : {:?}", format_size(waste_space));

    export_images_to_new_destination(duplicates)?;

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
