use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rusqlite::{Connection, Result};

use crate::models::{IndexStats, MediaFile, MediaItem, SearchFilter, TimelineGroup};
use crate::search::build_where_clause;

const MIGRATION_001: &str = include_str!("../../migrations/001_initial_schema.sql");
const MIGRATION_002: &str = include_str!("../../migrations/002_thumbnails_and_search.sql");

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial", MIGRATION_001),
    ("002_thumbnails_and_search", MIGRATION_002),
];

/// Opens a SQLite database, enables foreign keys, WAL mode, and runs migrations
pub fn open_db(path: &str) -> Result<Connection> {
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let mut conn = Connection::open(path)?;

    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;",
    )?;

    run_migrations(&mut conn)?;

    Ok(conn)
}

fn run_migrations(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS __schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    for (version, sql) in MIGRATIONS {
        let already_applied = {
            let mut stmt = conn.prepare("SELECT 1 FROM __schema_migrations WHERE version = ?1")?;
            stmt.exists([version])?
        };

        if !already_applied {
            let tx = conn.transaction()?;

            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO __schema_migrations (version) VALUES (?1)",
                (version,),
            )?;

            tx.commit()?;

            log::info!("Applied migration: {}", version);
        }
    }

    Ok(())
}

/// Query media_files grouped by date, joining thumbnails for thumb_small
pub fn get_timeline(
    conn: &Connection,
    offset: i64,
    limit: i64,
    filter: &SearchFilter,
) -> Result<Vec<TimelineGroup>> {
    let (where_clause, params) = build_where_clause(filter);

    let query = format!(
        "SELECT
            m.id,
            m.media_type,
            m.extension,
            m.date_taken,
            m.camera_model,
            m.path,
            t.thumb_small
         FROM media_files m
         LEFT JOIN thumbnails t ON m.id = t.media_file_id
         WHERE 1=1 {}
         ORDER BY m.date_taken DESC
         LIMIT ?1 OFFSET ?2",
        where_clause
    );

    let mut stmt = conn.prepare(&query)?;

    // Build parameters: limit, offset, then filter params
    let mut all_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    all_params.push(Box::new(limit));
    all_params.push(Box::new(offset));
    for p in params {
        all_params.push(p);
    }

    let param_refs: Vec<&dyn rusqlite::ToSql> = all_params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        let thumb_blob: Option<Vec<u8>> = row.get(6)?;
        let thumb_b64 = thumb_blob.map(|b| BASE64.encode(&b));

        Ok(MediaItem {
            id: row.get(0)?,
            media_type: row.get(1)?,
            extension: row.get(2)?,
            date_taken: row.get(3)?,
            camera_model: row.get(4)?,
            file_path: row.get(5)?,
            thumb_small_b64: thumb_b64,
        })
    })?;

    let mut items: Vec<MediaItem> = Vec::new();
    for row in rows {
        items.push(row?);
    }

    // Group by date (just the date part of date_taken)
    let mut groups: Vec<TimelineGroup> = Vec::new();

    for item in items {
        let date_str = item
            .date_taken
            .as_ref()
            .and_then(|dt| dt.split(' ').next().map(|s| s.to_string()))
            .unwrap_or_else(|| "Unknown".to_string());

        if let Some(group) = groups.iter_mut().find(|g| g.date == date_str) {
            group.media.push(item);
        } else {
            groups.push(TimelineGroup {
                date: date_str,
                media: vec![item],
            });
        }
    }

    Ok(groups)
}

/// Get full details of a single media file
pub fn get_media_detail(conn: &Connection, id: i64) -> Result<MediaFile> {
    conn.query_row(
        "SELECT id, hash, file_size_bytes, media_type, extension,
                camera_make, camera_model, lens_model, date_taken,
                iso, aperture, shutter_speed, focal_length, software,
                duration_seconds, resolution_width, resolution_height,
                path, date_added, date_modified
         FROM media_files WHERE id = ?1",
        [id],
        |row| {
            Ok(MediaFile {
                id: row.get(0)?,
                hash: row.get(1)?,
                file_size_bytes: row.get(2)?,
                media_type: row.get(3)?,
                extension: row.get(4)?,
                camera_make: row.get(5)?,
                camera_model: row.get(6)?,
                lens_model: row.get(7)?,
                date_taken: row.get(8)?,
                iso: row.get(9)?,
                aperture: row.get(10)?,
                shutter_speed: row.get(11)?,
                focal_length: row.get(12)?,
                software: row.get(13)?,
                duration_seconds: row.get(14)?,
                resolution_width: row.get(15)?,
                resolution_height: row.get(16)?,
                path: row.get(17)?,
                date_added: row.get(18)?,
                date_modified: row.get(19)?,
            })
        },
    )
}

/// Get thumbnail BLOB from thumbnails table
pub fn get_thumbnail(conn: &Connection, media_file_id: i64, size: &str) -> Result<Option<Vec<u8>>> {
    let column = match size {
        "medium" => "thumb_medium",
        _ => "thumb_small",
    };

    let query = format!(
        "SELECT {} FROM thumbnails WHERE media_file_id = ?1",
        column
    );

    conn.query_row(&query, [media_file_id], |row| row.get(0))
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            _ => Err(e),
        })
}

/// Get aggregate statistics about the library
pub fn get_stats(conn: &Connection) -> Result<IndexStats> {
    let total_media: i64 =
        conn.query_row("SELECT COUNT(*) FROM media_files", [], |row| row.get(0))?;

    let total_images: i64 = conn.query_row(
        "SELECT COUNT(*) FROM media_files WHERE media_type = 'image'",
        [],
        |row| row.get(0),
    )?;

    let total_videos: i64 = conn.query_row(
        "SELECT COUNT(*) FROM media_files WHERE media_type = 'video'",
        [],
        |row| row.get(0),
    )?;

    let total_size_bytes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(file_size_bytes), 0) FROM media_files",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut cameras: Vec<String> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT camera_model FROM media_files
             WHERE camera_model IS NOT NULL ORDER BY camera_model",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        for row in rows {
            cameras.push(row?);
        }
    }

    let min_date: Option<String> = conn
        .query_row(
            "SELECT MIN(date_taken) FROM media_files WHERE date_taken IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .ok();

    let max_date: Option<String> = conn
        .query_row(
            "SELECT MAX(date_taken) FROM media_files WHERE date_taken IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(IndexStats {
        total_media,
        total_images,
        total_videos,
        total_size_bytes,
        cameras,
        date_range: (min_date, max_date),
    })
}

/// Get the primary file path for a media file
pub fn get_file_path(conn: &Connection, media_file_id: i64) -> Result<String> {
    conn.query_row(
        "SELECT path FROM media_files WHERE id = ?1",
        [media_file_id],
        |row| row.get(0),
    )
}

/// Check if a hash already exists in the database
pub fn hash_exists(conn: &Connection, hash: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM media_files WHERE hash = ?1",
        [hash],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Insert a new media file into the database
pub fn insert_media_file(conn: &Connection, media: &MediaFile) -> Result<i64> {
    conn.execute(
        "INSERT INTO media_files (
            hash, file_size_bytes, media_type, extension,
            camera_make, camera_model, lens_model, date_taken,
            iso, aperture, shutter_speed, focal_length, software,
            duration_seconds, resolution_width, resolution_height,
            path, date_added, date_modified
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19
        )",
        rusqlite::params![
            media.hash,
            media.file_size_bytes,
            media.media_type,
            media.extension,
            media.camera_make,
            media.camera_model,
            media.lens_model,
            media.date_taken,
            media.iso,
            media.aperture,
            media.shutter_speed,
            media.focal_length,
            media.software,
            media.duration_seconds,
            media.resolution_width,
            media.resolution_height,
            media.path,
            media.date_added,
            media.date_modified
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Insert a thumbnail for a media file
pub fn insert_thumbnail(
    conn: &Connection,
    media_file_id: i64,
    thumb_small: &[u8],
    thumb_medium: Option<&[u8]>,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO thumbnails (media_file_id, thumb_small, thumb_medium)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![media_file_id, thumb_small, thumb_medium],
    )?;
    Ok(())
}
