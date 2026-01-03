use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use chrono::{Datelike, NaiveDateTime};
use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::database::operations;
use crate::utils::core::{ExifData, FileType, Media};

const PARTIAL_HASH_SIZE: usize = 128 * 1024; // 128 KB

#[derive(Debug, Clone, Serialize)]
pub struct Duplicates {
    pub hash: String,
    pub count: usize,
    pub total_size: u64,
    pub files: Vec<PathBuf>,
    pub file_type: FileType,
    pub file_size: u64,
    pub exif_data: Option<ExifData>,
    pub media: Media,
    pub final_path: PathBuf,
}

pub fn find_duplicates(
    conn: &Connection,
    data: Vec<Media>,
    destination_path: &Path,
) -> io::Result<Vec<Duplicates>> {
    let mut hash_map: HashMap<String, Vec<Media>> = HashMap::new();

    for media in data {
        let hash = media.hash.clone();

        println!("Hash : {:?}", hash);

        hash_map.entry(hash).or_insert_with(Vec::new).push(media);
    }

    println!("Preparing duplicate files data");

    let duplicate_files = hash_map
        .into_iter()
        // .filter(|(_, files)| files.len() > 1)
        .map(|(hash, media_files)| {
            let metadata = fs::metadata(&media_files[0].file_path)?;
            let file_size = metadata.len();

            println!("Duplcate File : {:?}", media_files[0].file_path);

            // Insert duplicate group data into database
            operations::insert_duplicate_group(conn, &media_files[0]).unwrap_or_else(|e| {
                println!(
                    "Error inserting duplicate group to database; Hash : {:?}; Error : {:?}",
                    media_files[0].hash, e
                );
            });

            Ok(Duplicates {
                hash,
                count: media_files.len(),
                total_size: file_size * media_files.len() as u64,
                files: media_files.iter().map(|f| f.file_path.clone()).collect(),
                file_type: media_files[0].clone().file_type,
                file_size: media_files[0].file_size,
                exif_data: media_files[0].clone().exif_data,
                media: media_files.get(0).unwrap().clone(),
                final_path: PathBuf::from(final_path_for_media(
                    media_files[0].clone(),
                    destination_path.to_str().unwrap(),
                )),
            })
        })
        .collect::<io::Result<Vec<Duplicates>>>();

    println!("Done Duplicate data");

    return duplicate_files;
}

fn final_path_for_media(media: Media, destination_path: &str) -> String {
    let date_taken = media
        .exif_data
        .as_ref()
        .and_then(|e| e.date_taken.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    let formatted_date = NaiveDateTime::parse_from_str(date_taken, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| panic!("Invalid date time format, {:?}", media));

    let year = formatted_date.year();
    let month = formatted_date.month();
    let day_of_month = formatted_date.day();

    format!(
        "{}/{}/{}/{}/{}/{}",
        destination_path,
        year,
        month,
        day_of_month,
        media.file_type.to_string(),
        media.file_name
    )
}

pub fn calculate_hash(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;

    let mut buffer = vec![0u8; PARTIAL_HASH_SIZE];
    let bytes_read = file.read(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer[..bytes_read]);

    Ok(format!("{:x}", hasher.finalize()))
}
