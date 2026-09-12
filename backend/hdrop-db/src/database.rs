use std::borrow::Cow;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
    RunQueryDsl,
};
use hdrop_shared::{
    metrics::{names, UpdateMetrics},
    responses,
};
use uuid::Uuid;

use crate::{
    error::Result,
    models::{File, InsertFile},
    schema::files::dsl as files_table,
    utils::{TokenGenerator, UPDATE_TOKEN_LENGTH},
};

pub struct Database {
    pool: Pool<AsyncPgConnection>,
    generator: TokenGenerator,
}

impl Database {
    /// Initialize the database from environment variables.
    pub fn try_from_env() -> Result<Database> {
        let database_url = hdrop_shared::env::database_url()?;
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(manager).max_size(8).build()?;
        let generator = TokenGenerator::default();
        Ok(Database { pool, generator })
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
                    file.accessToken = TokenGenerator::generate_token(
                        file.accessToken
                            .len()
                            .max(self.generator.get_access_token_min_length()),
                    );
                }
                Err(error) => return Err(error.into()),
            }
        };
        drop(conn);
        // Action-based update of metrics
        self.update_metrics().await;
        Ok(file)
    }

    pub async fn update_file(&self, file: File) -> Result<()> {
        let mut conn = self.pool.get().await?;
        diesel::update(files_table::files.filter(files_table::uuid.eq(file.uuid)))
            .set(file)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn get_file_rows(&self) -> Result<i64> {
        let mut conn = self.pool.get().await?;
        Ok(files_table::files.count().get_result(&mut conn).await?)
    }

    pub async fn update_data_url<'a>(
        &self,
        uuid: Uuid,
        file_url: Option<impl Into<Cow<'a, str>>>,
    ) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let file_url: Option<String> = file_url.map(|inner| inner.into().into_owned());
        diesel::update(files_table::files.filter(files_table::uuid.eq(uuid)))
            .set(files_table::dataUrl.eq(file_url))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update_file_expiry(&self, file: File) -> Result<()> {
        let mut conn = self.pool.get().await?;
        diesel::update(files_table::files.filter(files_table::uuid.eq(file.uuid)))
            .set(files_table::expiresAt.eq(file.expiresAt))
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

    pub async fn get_file_by_access_token<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<File> {
        let mut conn = self.pool.get().await?;
        let access_token = access_token.into().into_owned();
        Ok(files_table::files
            .filter(files_table::accessToken.eq(access_token))
            .first(&mut conn)
            .await?)
    }

    pub async fn get_verification_data<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<responses::VerifyChallengeData> {
        let file = self.get_file_by_access_token(access_token).await?;
        Ok(responses::VerifyChallengeData {
            challenge_hash: Some(file.challengeHash),
            file_name_data: file.fileNameData,
        })
    }

    pub async fn get_file_metadata<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<responses::FileMetaData> {
        let file = self.get_file_by_access_token(access_token).await?;
        Ok(responses::FileMetaData {
            file_url: file.dataUrl,
        })
    }

    pub async fn get_files_to_flush(&self) -> Result<Vec<Uuid>> {
        let mut conn = self.pool.get().await?;
        Ok(files_table::files
            .filter(files_table::expiresAt.lt(Utc::now()))
            .select(files_table::uuid)
            .load::<Uuid>(&mut conn)
            .await?)
    }

    pub async fn get_challenge<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<responses::GetChallengeData> {
        let file = self.get_file_by_access_token(access_token).await?;

        Ok(responses::GetChallengeData {
            salt: file.salt,
            iv: file.iv,
            challenge: file.challengeData,
        })
    }

    pub async fn delete_file(&self, file: File) -> Result<File> {
        let mut conn = self.pool.get().await?;
        let r = Ok(
            diesel::delete(files_table::files.filter(files_table::uuid.eq(&file.uuid)))
                .get_result(&mut conn)
                .await?,
        );
        drop(conn);
        // Action-based update of metrics
        self.update_metrics().await;
        r
    }

    pub async fn delete_file_by_uuid(&self, uuid: Uuid) -> Result<()> {
        let mut conn = self.pool.get().await?;
        diesel::delete(files_table::files.filter(files_table::uuid.eq(uuid)))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn check_access_token_collission<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<bool> {
        let mut conn = self.pool.get().await?;
        let access_token = access_token.into().into_owned();
        Ok(diesel::select(diesel::dsl::exists(
            files_table::files.filter(files_table::accessToken.eq(access_token)),
        ))
        .get_result(&mut conn)
        .await?)
    }
    /// Generates access token with min length.
    /// Retriess generation when collissions happen (10 times), after that it increases the generated length by 1.
    pub async fn generate_access_token(&self) -> Result<String> {
        let mut target_length = self.generator.get_access_token_min_length();
        let mut access_token = TokenGenerator::generate_token(target_length);
        let mut collisions = 0;

        while self.check_access_token_collission(&access_token).await? {
            collisions += 1;

            if collisions > 10 {
                target_length += 1;
                // From now on, only two repeat attempts before increasing the length again
                collisions = 8;
            }
            access_token = TokenGenerator::generate_token(target_length);
        }

        Ok(access_token)
    }

    pub fn generate_update_token() -> String {
        TokenGenerator::generate_token(UPDATE_TOKEN_LENGTH)
    }
}

#[async_trait]
impl UpdateMetrics for Database {
    /// Monitor the database file count.
    async fn update_metrics(&self) {
        // Determine the number of files currently stored according to the database.
        let file_count = self.get_file_rows().await.unwrap_or(0);

        // Update file count gauge
        metrics::gauge!(names::storage::DATABASE_FILE_COUNT).set(file_count as f64);
    }
}

#[cfg(test)]
mod tests;
