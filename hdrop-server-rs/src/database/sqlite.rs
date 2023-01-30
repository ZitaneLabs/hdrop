use chrono::prelude::*;
use rusqlite::{Connection, Result};
use uuid::Uuid;

#[derive(Debug)]
struct Database {
    connection: Connection,
}

impl Database {
    pub fn new() -> Result<Database> {
        let db = Database {
            connection: Connection::open_in_memory()?,
        };

        db.connection.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id          INTEGER PRIMARY KEY,
                filename_enc     BLOB,
                filename_hash    TEXT,
                uuid             TEXT,
                short_code       TEXT,
                update_token     TEXT,
                salt             BLOB,
                iv               BLOB,
                created_at       TEXT,
                expires_at       TEXT,
            );

            CREATE TABLE IF NOT EXISTS statistics (
                id: INTEGER PRIMARY KEY,
                uploadCount: INTEGER AUTOINCREMENT,
                deleteCount: INTEGER AUTOINCREMENT,
                existCount:  INTEGER,
            );

            CREATE TABLE IF NOT EXISTS file_statistics (
                id INTEGER PRIMARY KEY,
                filetype: TEXT,
                filesize: INTEGER,
            );
            ",
            (),
        )?;

        // Save sqlite db

        Ok(db) // conn
    }

    pub fn insert_file(&self, file: File) -> Result<()> {
        todo!();
        self.connection.execute(
            "INSERT INTO files (filename_enc, filename_hash, uuid, short_code, update_token, salt, iv, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (&file.filename_enc, &file.filename_hash,
                &file.access_token.uuid.to_string(), 
                &file.access_token.short_code.to_string(), 
                &file.access_token.update_token.to_string(), 
                &file.salt, &file.iv, &file.created_at.to_string(), &file.expires_at.to_string()),
        )?;

        Ok(())
    }

    pub fn clean() {
        unimplemented!("")
    }

    pub fn is_expired() {}
}

struct File {
    filename_enc: Vec<u8>,
    filename_hash: String,
    access_token: AccessToken,
    salt: Vec<u8>,
    iv: Vec<u8>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

struct AccessToken {
    uuid: Uuid,
    short_code: String,
    update_token: String,
}

impl File {
    pub fn new(
        filename_enc: Vec<u8>,
        filename_hash: String,
        access_token: AccessToken,
        salt: Vec<u8>,
        iv: Vec<u8>,
        seconds: i64,
    ) -> File {
        let time = Utc::now();

        let file = File {
            filename_enc,
            filename_hash,
            access_token,
            salt,
            iv,
            created_at: time,
            expires_at: time + chrono::Duration::seconds(seconds),
        };

        file
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
