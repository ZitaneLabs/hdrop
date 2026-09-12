use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
    RunQueryDsl,
};
use hdrop_shared::metrics::{names, UpdateMetrics};
use uuid::Uuid;

use crate::{
    error::Result,
    models::{File, InsertFile},
    schema::files::dsl as files_table,
    utils::{generate_token, ACCESS_TOKEN_LENGTH, UPDATE_TOKEN_LENGTH},
};

pub struct Database {
    pool: Pool<AsyncPgConnection>,
}

impl Database {
    /// Initialize the database from environment variables.
    pub fn try_from_env() -> Result<Database> {
        let database_url = hdrop_shared::env::database_url()?;
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(manager)
            .max_size(hdrop_shared::env::database_pool_size()?)
            .build()?;
        Ok(Database { pool })
    }

    /// Insert a file, regenerating its access token if another insert claimed it.
    /// Callers must use the access token in the returned file.
    pub async fn insert_file(&self, mut file: InsertFile) -> Result<File> {
        let mut conn = self.pool.get().await?;
        let mut retries = 0;
        let file = loop {
            match diesel::insert_into(files_table::files)
                .values(&file)
                .get_result::<File>(&mut conn)
                .await
            {
                Ok(file) => break file,
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    ref info,
                )) if info.constraint_name() == Some("files_accessToken_key") && retries < 10 => {
                    retries += 1;
                    file.accessToken =
                        generate_token(file.accessToken.len().max(ACCESS_TOKEN_LENGTH));
                }
                Err(error) => return Err(error.into()),
            }
        };
        metrics::gauge!(names::storage::DATABASE_FILE_COUNT).increment(1.0);
        Ok(file)
    }

    async fn get_file_rows(&self) -> Result<i64> {
        let mut conn = self.pool.get().await?;
        Ok(files_table::files.count().get_result(&mut conn).await?)
    }

    pub async fn update_data_url(&self, uuid: Uuid, file_url: Option<&str>) -> Result<()> {
        let mut conn = self.pool.get().await?;
        diesel::update(files_table::files.filter(files_table::uuid.eq(uuid)))
            .set(files_table::dataUrl.eq(file_url))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update_file_expiry(&self, uuid: Uuid, expires_at: DateTime<Utc>) -> Result<()> {
        let mut conn = self.pool.get().await?;
        diesel::update(files_table::files.filter(files_table::uuid.eq(uuid)))
            .set(files_table::expiresAt.eq(expires_at))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn get_file_by_uuid(&self, uuid: Uuid) -> Result<File> {
        let mut conn = self.pool.get().await?;
        Ok(files_table::files
            .filter(files_table::uuid.eq(uuid))
            .first(&mut conn)
            .await?)
    }

    pub async fn get_file_by_access_token(&self, access_token: &str) -> Result<File> {
        let mut conn = self.pool.get().await?;
        Ok(files_table::files
            .filter(files_table::accessToken.eq(access_token))
            .first(&mut conn)
            .await?)
    }

    pub async fn get_files_to_flush(&self) -> Result<Vec<Uuid>> {
        let mut conn = self.pool.get().await?;
        Ok(files_table::files
            .filter(files_table::expiresAt.lt(Utc::now()))
            .select(files_table::uuid)
            .load::<Uuid>(&mut conn)
            .await?)
    }

    pub async fn delete_file(&self, uuid: Uuid) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let deleted = diesel::delete(files_table::files.filter(files_table::uuid.eq(uuid)))
            .execute(&mut conn)
            .await?;
        metrics::gauge!(names::storage::DATABASE_FILE_COUNT).decrement(deleted as f64);
        Ok(())
    }

    /// Generate an access token; insertion handles collisions through the unique index.
    pub fn generate_access_token() -> String {
        generate_token(ACCESS_TOKEN_LENGTH)
    }

    pub fn generate_update_token() -> String {
        generate_token(UPDATE_TOKEN_LENGTH)
    }
}

#[async_trait]
impl UpdateMetrics for Database {
    /// Reconcile at startup and periodically, including changes by other processes.
    /// Concurrent mutations can briefly skew this snapshot; the next pass repairs it.
    async fn update_metrics(&self) {
        if let Ok(file_count) = self.get_file_rows().await {
            metrics::gauge!(names::storage::DATABASE_FILE_COUNT).set(file_count as f64);
        }
    }
}

#[cfg(test)]
mod tests;
