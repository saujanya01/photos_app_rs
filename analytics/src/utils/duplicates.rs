use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::utils::Media;

#[derive(Debug, Clone, Serialize)]
pub struct Duplicates {
    pub hash: String,
    pub files: Vec<PathBuf>,
    pub count: usize,
    pub total_size: u64,
}

pub fn find_duplicates(data: Vec<Media>) -> io::Result<Vec<Duplicates>> {
    println!("Finding Duplicates");

    let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

    println!("Calculating hashes");

    for media in data {
        let hash = calculate_hash(&media.file_path)?;

        println!("Hash : {:?}", hash);

        hash_map
            .entry(hash)
            .or_insert_with(Vec::new)
            .push(media.file_path);
    }

    println!("Preparing duplicate files data");

    let duplicate_files = hash_map
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(hash, files)| {
            let metadata = fs::metadata(&files[0])?;
            let file_size = metadata.len();

            println!("Duplcate File : {:?}", files[0]);

            Ok(Duplicates {
                hash,
                count: files.len(),
                total_size: file_size * files.len() as u64,
                files,
            })
        })
        .collect::<io::Result<Vec<Duplicates>>>();

    println!("Done Duplicate data");

    return duplicate_files;
}

fn calculate_hash(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn export_duplicates_to_csv(
    duplicate_media: Vec<Duplicates>,
    output_path: &str,
) -> io::Result<()> {
    use std::io::Write;

    let mut wtr =
        csv::Writer::from_path(output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Write headers
    wtr.write_record(&[
        "Hash",
        "Duplicate Count",
        "Total Wasted Space (bytes)",
        "Total Wasted Space (Human)",
        "File Paths",
    ])
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Write data
    for dup in duplicate_media {
        let paths = dup
            .files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" | ");

        wtr.write_record(&[
            &dup.hash,
            &dup.count.to_string(),
            &dup.total_size.to_string(),
            &format_size(dup.total_size),
            &paths,
        ])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    wtr.flush()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("✅ Exported duplicates to {}", output_path);
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
