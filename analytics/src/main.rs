mod utils;

use std::{
    fmt::Result,
    fs::{self, metadata},
    io,
};

use crate::utils::{
    FileType, Media, duplicates::export_duplicates_to_csv, duplicates::find_duplicates,
    duplicates::format_size, export_to_csv, export_to_json, scan_directory,
};

fn main() -> io::Result<()> {
    // let path = "./test_folder";
    let path = "/Users/saujanya/sandisk_media";

    let mut media_items = Vec::new();

    scan_directory(path.as_ref(), &mut media_items)?;

    println!("Exporting full data");

    export_to_csv(&media_items, "full.csv");

    export_to_json(&media_items, "full.json");

    println!("Exported full data");

    let duplicate_media = find_duplicates(media_items)?;

    if !duplicate_media.is_empty() {
        println!("Found {} groups of duplicates", duplicate_media.len());

        // Calculate total wasted space
        let total_wasted: u64 = duplicate_media.iter().map(|d| d.total_size).sum();
        println!("Total wasted space: {}", format_size(total_wasted));

        // Export to CSV
        export_duplicates_to_csv(duplicate_media, "duplicates.csv")?;
    } else {
        println!("No duplicates found!");
    }

    Ok(())
}
