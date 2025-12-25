# Purpose

1. Give me an excel sheet or json that will have data about every photo and video in ssd.
2. Determine duplicates based on exif data and file size.

## Data needed

1. everythig exif
2. file path
3. file size
4. file type
5. date created
6. date modified
7. resolution (for photos)
8. duration (for videos)
9. camera model (if available)
10. tags or keywords (if available)
11. Final Filename

## Format for final filename

`[year]/[year]-[month]/[image or video]/[year]-[month]-[date]_[place_tag]_[camera_model].[file extension]`

### Columns

"Hash",
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

```rust

pub fn export_to_csv(media_items: &[Media], output_path: &str) -> io::Result<()> {
    let mut wtr =
        csv::Writer::from_path(output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Write headers
    wtr.write_record(&[
        "Hash",
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
            &media.hash,
            &media.file_name,
            media.file_path.to_str().unwrap_or(""),
            &format!("{:?}", media.file_type),
            &media.file_size.to_string(),
            // &media.file_size_human_readable(),
            format_size(media.file_size).as_str(),
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

```
