use chrono::prelude::*;
use sqlx::{Connection, PgConnection, Postgres, Result};
use std::env;
use uuid::Uuid;
use warp::hyper::client::connect;

#[derive(Debug)]
struct Database {
    connection: PgConnection,
}

impl Database {
    pub async fn new() -> Result<Database> {
        let mut db = Database {
            connection: PgConnection::connect(
                env::var("DATABASE_URL")
                    .expect("DATABASE_URL not defined")
                    .as_str(),
            )
            .await?,
        };

        sqlx::query(
            r#"CREATE TABLE [IF NOT EXISTS] "File" (
            "id" SERIAL NOT NULL,
            "uuid" TEXT NOT NULL,
            "accessToken" TEXT NOT NULL,
            "updateToken" TEXT NOT NULL,
            "dataUrl" TEXT,
            "fileNameData" TEXT NOT NULL,
            "fileNameHash" TEXT NOT NULL,
            "salt" TEXT NOT NULL,
            "iv" TEXT NOT NULL,
            "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "expiresAt" TIMESTAMP(3) NOT NULL,
        
            --CONSTRAINT "File_pkey" PRIMARY KEY ("id")
            CONSTRAINT "File_pkey" PRIMARY KEY ("uuid")
        );

        CREATE UNIQUE INDEX "File_uuid_key" ON "File"("uuid"),
        CREATE UNIQUE INDEX "File_accessToken_key" ON "File"("accessToken");



        CREATE TABLE [IF NOT EXISTS] "Statistics" (
            "id" SERIAL NOT NULL,
            "uploadCount" INTEGER NOT NULL DEFAULT 0,
            "deleteCount" INTEGER NOT NULL DEFAULT 0,
            "existCount"  INTEGER NOT NULL DEFAULT 0,

            CONSTRAINT "Statistics_pkey" PRIMARY KEY ("id")
        );
        "#,
        )
        .execute(&mut db.connection)
        .await?;

        Ok(db) // conn
    }

    pub fn createFile(&self, file:File) {

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
