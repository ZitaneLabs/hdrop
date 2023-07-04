use chrono::Utc;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel::prelude::*;
use hdrop_shared::responses;
use std::borrow::Cow;
use uuid::Uuid;

use crate::{
    error::Result,
    models::*,
    schema::files::dsl as files_table,
    utils::{TokenGenerator, UPDATE_TOKEN_LENGTH},
};

pub struct Database {
    pool: Pool,
    generator: TokenGenerator,
}

impl Database {
    /// Initialize the database from environment variables.
    pub fn try_from_env() -> Result<Database> {
        let database_url = hdrop_shared::env::database_url()?;
        let manager = Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
        let pool = Pool::builder(manager).max_size(8).build()?;
        let generator = TokenGenerator::default();
        Ok(Database { pool, generator })
    }

    pub async fn insert_file(&self, file: InsertFile) -> Result<File> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(|conn| {
                diesel::insert_into(files_table::files)
                    .values(file)
                    .get_result::<File>(conn)
            })
            .await??)
    }

    pub async fn update_file(&self, file: File) -> Result<()> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(|conn| {
                diesel::update(files_table::files.filter(files_table::uuid.eq(file.uuid)))
                    .set(file)
                    .execute(conn)
                    .map(|_| ())
            })
            .await??)
    }

    pub async fn get_file_amount(&self) -> Result<i64> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(|conn| files_table::files.count().get_result(conn))
            .await??)
    }

    pub async fn update_data_url<'a>(
        &self,
        uuid: Uuid,
        file_url: Option<impl Into<Cow<'a, str>>>,
    ) -> Result<()> {
        let file_url: Option<String> = file_url.map(|inner| inner.into().into_owned());
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                diesel::update(files_table::files.filter(files_table::uuid.eq(uuid)))
                    .set(files_table::dataUrl.eq(file_url))
                    .execute(conn)
                    .map(|_| ())
            })
            .await??)
    }

    pub async fn update_file_expiry(&self, file: File) -> Result<()> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                diesel::update(files_table::files.filter(files_table::uuid.eq(file.uuid)))
                    .set(files_table::expiresAt.eq(file.expiresAt))
                    .execute(conn)
                    .map(|_| ())
            })
            .await??)
    }

    pub async fn get_file_by_uuid(&self, uuid: Uuid) -> Result<File> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                files_table::files
                    .filter(files_table::uuid.eq(uuid))
                    .first(conn)
            })
            .await??)
    }

    pub async fn get_file_by_access_token<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<File> {
        let access_token = access_token.into().into_owned();
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                files_table::files
                    .filter(files_table::accessToken.eq(access_token))
                    .first(conn)
            })
            .await??)
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

    pub async fn flush(&self) -> Result<Vec<Uuid>> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                files_table::files
                    .filter(files_table::expiresAt.lt(Utc::now()))
                    .select(files_table::uuid)
                    .load::<Uuid>(conn)
            })
            .await??)
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
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                diesel::delete(files_table::files.filter(files_table::uuid.eq(&file.uuid)))
                    .get_result(conn)
            })
            .await??)
    }

    pub async fn delete_file_by_uuid(&self, uuid: Uuid) -> Result<()> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                diesel::delete(files_table::files.filter(files_table::uuid.eq(uuid)))
                    .execute(conn)
                    .map(|_| ())
            })
            .await??)
    }

    pub async fn check_access_token_collission<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<bool> {
        let access_token = access_token.into().into_owned();
        Ok(self
            .pool
            .get()
            .await?
            .interact(|conn| {
                files_table::files
                    .filter(files_table::accessToken.eq(access_token))
                    .first::<File>(conn)
                    .is_ok()
            })
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
