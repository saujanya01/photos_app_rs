use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::utils::{ExifData, FileType, Media};

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
}

pub fn find_duplicates(data: Vec<Media>) -> io::Result<Vec<Duplicates>> {
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

            Ok(Duplicates {
                hash,
                count: media_files.len(),
                total_size: file_size * media_files.len() as u64,
                files: media_files.iter().map(|f| f.file_path.clone()).collect(),
                file_type: media_files[0].clone().file_type,
                file_size: media_files[0].file_size,
                exif_data: media_files[0].clone().exif_data,
                media: media_files.get(0).unwrap().clone(),
            })
        })
        .collect::<io::Result<Vec<Duplicates>>>();

    println!("Done Duplicate data");

    return duplicate_files;
}

pub fn calculate_hash(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;

    let mut buffer = vec![0u8; PARTIAL_HASH_SIZE];
    let bytes_read = file.read(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer[..bytes_read]);

    Ok(format!("{:x}", hasher.finalize()))
}
