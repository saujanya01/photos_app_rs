use rusqlite::Connection;

const MIGRATIONS: &[(&str, &str)] = &[(
    "001_initial",
    include_str!("migrations/001_initial_schema.sql"),
)];

pub fn run_migrations(conn: &mut Connection) -> rusqlite::Result<()> {
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

            println!("Applied migration: {}", version);
        }
    }

    Ok(())
}
