use rusqlite::Connection;

use crate::database::migrations;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: impl AsRef<std::path::Path>) -> rusqlite::Result<Self> {
        let mut conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        migrations::run_migrations(&mut conn)?;

        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
