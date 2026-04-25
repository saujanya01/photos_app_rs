use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rusqlite::{Connection, Result};

use crate::models::{IndexStats, MediaFile, MediaItem, SearchFilter, TimelineGroup};
use crate::search::build_where_clause;

const MIGRATION_001: &str = include_str!("../../migrations/001_initial_schema.sql");
const MIGRATION_002: &str = include_str!("../../migrations/002_thumbnails_and_search.sql");
const MIGRATION_003: &str = include_str!("../../migrations/003_gps.sql");
const MIGRATION_004: &str = include_str!("../../migrations/004_v06_extensions.sql");

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial", MIGRATION_001),
    ("002_thumbnails_and_search", MIGRATION_002),
    ("003_gps", MIGRATION_003),
    ("004_v06_extensions", MIGRATION_004),
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
                path, date_added, date_modified,
                gps_latitude, gps_longitude, dominant_color, rating
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
                gps_latitude: row.get(20)?,
                gps_longitude: row.get(21)?,
                dominant_color: row.get(22)?,
                rating: row.get(23)?,
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
            path, date_added, date_modified,
            gps_latitude, gps_longitude, dominant_color, rating
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19,
            ?20, ?21, ?22, ?23
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
            media.date_modified,
            media.gps_latitude,
            media.gps_longitude,
            media.dominant_color,
            media.rating,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn columns_of(conn: &Connection, table: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .unwrap();
        stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    fn applied(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT version FROM __schema_migrations ORDER BY version")
            .unwrap();
        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    #[test]
    fn migrations_apply_to_fresh_db() {
        let tmp = std::env::temp_dir().join(format!("phorsmig-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        let conn = open_db(tmp.to_str().unwrap()).expect("open_db");

        let versions = applied(&conn);
        assert_eq!(
            versions,
            vec![
                "001_initial",
                "002_thumbnails_and_search",
                "003_gps",
                "004_v06_extensions",
            ]
        );

        let cols = columns_of(&conn, "media_files");
        for c in [
            "gps_latitude",
            "gps_longitude",
            "dominant_color",
            "rating",
        ] {
            assert!(cols.iter().any(|x| x == c), "missing column {c}");
        }

        for table in ["albums", "album_items"] {
            let exists: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1, "missing table {table}");
        }

        drop(conn);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn upgrades_from_002_preserve_existing_rows() {
        // Simulate the real-world case: a DB sitting at migrations 001+002
        // with existing media_files rows, then upgrading to 003+004.
        let tmp = std::env::temp_dir().join(format!("phorsupg-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&tmp);

        // Stand up a DB with only 001+002 applied, then insert a row.
        {
            let mut c = Connection::open(&tmp).unwrap();
            c.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            c.execute_batch(
                "CREATE TABLE __schema_migrations (version TEXT PRIMARY KEY, \
                 applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);",
            )
            .unwrap();
            for (v, sql) in &MIGRATIONS[..2] {
                let tx = c.transaction().unwrap();
                tx.execute_batch(sql).unwrap();
                tx.execute(
                    "INSERT INTO __schema_migrations (version) VALUES (?1)",
                    [v],
                )
                .unwrap();
                tx.commit().unwrap();
            }
            c.execute(
                "INSERT INTO media_files (
                    hash, file_size_bytes, media_type, extension, path,
                    date_added, date_modified
                 ) VALUES ('preexisting', 100, 'image', 'jpg', '/old.jpg', 0, 0)",
                [],
            )
            .unwrap();
        }

        // Now open via the real path — should apply 003+004 cleanly.
        let conn = open_db(tmp.to_str().unwrap()).expect("upgrade open");

        assert_eq!(applied(&conn).len(), 4);

        // The pre-existing row survived and has default values for new columns.
        let (rating, lat, color): (i32, Option<f64>, Option<String>) = conn
            .query_row(
                "SELECT rating, gps_latitude, dominant_color FROM media_files WHERE hash='preexisting'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(rating, 0);
        assert_eq!(lat, None);
        assert_eq!(color, None);

        drop(conn);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn migrations_are_idempotent() {
        let tmp = std::env::temp_dir().join(format!("phorsmig2-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&tmp);

        let _ = open_db(tmp.to_str().unwrap()).expect("first open");
        let conn = open_db(tmp.to_str().unwrap()).expect("second open");

        let versions = applied(&conn);
        assert_eq!(versions.len(), 4, "duplicate migration entries: {versions:?}");

        drop(conn);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn insert_round_trip_with_v06_columns() {
        let tmp = std::env::temp_dir().join(format!("phorsmig3-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        let conn = open_db(tmp.to_str().unwrap()).expect("open_db");

        let m = MediaFile {
            id: None,
            hash: "deadbeef".into(),
            file_size_bytes: 42,
            media_type: "image".into(),
            extension: "jpg".into(),
            camera_make: None,
            camera_model: None,
            lens_model: None,
            date_taken: None,
            iso: None,
            aperture: None,
            shutter_speed: None,
            focal_length: None,
            software: None,
            duration_seconds: None,
            resolution_width: None,
            resolution_height: None,
            path: "/tmp/x.jpg".into(),
            date_added: 0,
            date_modified: 0,
            gps_latitude: Some(35.6762),
            gps_longitude: Some(139.6503),
            dominant_color: Some("oklch(0.42 0.08 60)".into()),
            rating: 4,
        };

        let id = insert_media_file(&conn, &m).expect("insert");
        let back = get_media_detail(&conn, id).expect("fetch");

        assert_eq!(back.gps_latitude, Some(35.6762));
        assert_eq!(back.gps_longitude, Some(139.6503));
        assert_eq!(back.dominant_color.as_deref(), Some("oklch(0.42 0.08 60)"));
        assert_eq!(back.rating, 4);

        drop(conn);
        let _ = std::fs::remove_file(&tmp);
    }
}
