// postgres://postgres:postgres@postgres:5432/hdrop
// docker-compose up postgres

use crate::schema;
use crate::schema::files::dsl::*;
use crate::{error::Result, models::*, utils};
use ::uuid::Uuid;
use chrono::Utc;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel::prelude::*;
use std::borrow::Cow;
use std::env;
use utils::{TokenGenerator, UPDATE_TOKEN_LENGTH};

pub struct Database {
    pool: Pool,
    generator: TokenGenerator,
}

impl Database {
    pub fn try_from_env() -> Result<Database> {
        let database_url = env::var("DATABASE_URL")?; // expect DATABASE_URL must be set
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
                diesel::insert_into(files)
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
                diesel::update(files.filter(uuid.eq(file.uuid)))
                    .set(file)
                    .execute(conn)
                    .map(|_| ())
            })
            .await??)
    }

    pub async fn update_file_url<'a>(
        &self,
        s_uuid: Uuid,
        file_url: impl Into<Cow<'a, str>>,
    ) -> Result<()> {
        let file_url = file_url.into().into_owned();
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                diesel::update(files.filter(uuid.eq(s_uuid)))
                    .set(dataUrl.eq(file_url))
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
                diesel::update(files.filter(uuid.eq(file.uuid)))
                    .set(expiresAt.eq(file.expiresAt))
                    .execute(conn)
                    .map(|_| ())
            })
            .await??)
    }

    pub async fn get_file_by_uuid(&self, s_uuid: Uuid) -> Result<File> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| files.filter(uuid.eq(s_uuid)).first(conn))
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
            .interact(move |conn| files.filter(accessToken.eq(access_token)).first(conn))
            .await??)
    }

    pub async fn get_hash<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<(String, String, String)> {
        let file = self.get_file_by_access_token(access_token).await?;

        Ok((file.fileNameHash, file.iv, file.salt))
    }

    pub async fn get_file_metadata<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<(Option<String>, String, String, String)> {
        let file = self.get_file_by_access_token(access_token).await?;

        Ok((file.dataUrl, file.fileNameData, file.iv, file.salt))
    }

    pub async fn flush(&self) -> Result<Vec<Uuid>> {
        /*let x = Ok(self
        .pool
        .get()
        .await?
        .interact(move |conn| {
            files.select(uuid).load::<Uuid>(conn)
            //files.filter(expiresAt.gt(Utc::now())).select(uuid).get_result::<schema::files::columns::uuid>(conn)
            //diesel::QueryDsl::filter(files, expiresAt.gt(Utc::now())).load::<schema::files::columns::uuid>(conn)
                /*.group_by(expiresAt)
                .having(expiresAt.gt(Utc::now()))
                .select(uuid).load(conn)*/
        })
        .await??);*/
        let x: Result<Vec<Uuid>> = Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                files
                    .filter(expiresAt.gt(Utc::now()))
                    .select(uuid)
                    .load::<Uuid>(conn)
            })
            .await??);

        x
    }

    pub async fn get_challenge<'a>(
        &self,
        access_token: impl Into<Cow<'a, str>>,
    ) -> Result<(String, String, String)> {
        //let access_token = access_token.into().into_owned();
        let file = self.get_file_by_access_token(access_token).await?;

        Ok((file.fileNameData, file.iv, file.salt))
    }

    pub async fn delete_file(&self, file: File) -> Result<File> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                diesel::delete(files.filter(uuid.eq(&file.uuid))).get_result(conn)
            })
            .await??)
    }

    pub async fn delete_file_by_uuid(&self, s_uuid: Uuid) -> Result<File> {
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| diesel::delete(files.filter(uuid.eq(s_uuid))).get_result(conn))
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
                files
                    .filter(accessToken.eq(access_token))
                    .first::<File>(conn)
                    .is_ok()
            })
            .await?)
    }

    pub async fn generate_access_token(&self) -> Result<String> {
        let mut target_length = self.generator.get_access_token_min_length();
        let mut access_token = TokenGenerator::generate_token(target_length);
        let mut collisions = 0;

        while self.check_access_token_collission(&access_token).await? {
            collisions += 1;

            if collisions > 10 {
                target_length += 1;
            }
            access_token = TokenGenerator::generate_token(target_length);
        }

        Ok(access_token)
    }

    pub fn generate_update_token() -> String {
        TokenGenerator::generate_token(UPDATE_TOKEN_LENGTH)
    }

    /*
    pub fn get_file(conn: &mut PgConnection, id_uuid: Uuid) -> QueryResult<File> {
        files.filter(uuid.eq(id_uuid))
             .first(conn)
    }
    */
}

/*
#[cfg(test)]
mod tests {
    use super::Database;
    use crate::models::File;
    use crate::InsertFile;
    use chrono::{Duration, Utc};
    use diesel::expression::is_aggregate::No;
    use diesel::prelude::*;
    use diesel::{pg::Pg, sql_query};
    //use diesel::sqlite::SqliteConnection;

    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use std::error::Error;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    fn run_migrations(
        connection: &mut impl MigrationHarness<Pg>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        // This will run the necessary migrations.
        //
        // See the documentation for `MigrationHarness` for
        // all available methods.
        connection.run_pending_migrations(MIGRATIONS)?;

        Ok(())
    }

    // Run before each tests
    fn initialize() -> Database {
        let mut db = Database::try_from_env().unwrap();

        sql_query("DROP SCHEMA IF EXISTS public CASCADE")
            .execute(&mut *db.connection.write())
            .expect("Error dropping db public");

        sql_query("CREATE SCHEMA public")
            .execute(&mut *db.connection.write())
            .expect("Error creating db public");

        sql_query("GRANT ALL ON SCHEMA public TO postgres")
            .execute(&mut *db.connection.write())
            .expect("Error creating db public");

        sql_query("GRANT ALL ON SCHEMA public TO public")
            .execute(&mut *db.connection.write())
            .expect("Error creating db public");

        run_migrations(&mut *db.connection.write()).expect("error running migrations");

        sql_query("ALTER SEQUENCE files_id_seq RESTART WITH 1")
            .execute(&mut *db.connection.write())
            .expect("Error resetting id sequence");

        db
    }

    #[test]
    fn test_connection() {
        //let conn = establish_connection();
        let _ = Database::try_from_env().unwrap();
    }

    #[test]
    fn test_insert() {
        let mut db = initialize();

        let file = InsertFile {
            id: None,
            uuid: uuid::Uuid::new_v4(),
            accessToken: "AccessToken".to_string(),
            updateToken: "UpdateToken".to_string(),
            dataUrl: Some("url".to_string()),
            fileNameData: "TestNameData".to_string(),
            fileNameHash: "TestNameHash".to_string(),
            salt: "TestSalt".to_string(),
            iv: "TestIV".to_string(),
            createdAt: Utc::now(),
            expiresAt: Utc::now(),
        };

        db.insert_file(&file);
    }

    #[test]
    fn test_insert_update() {
        let mut db = initialize();

        let file = InsertFile {
            id: None,
            uuid: uuid::Uuid::new_v4(),
            accessToken: "AccessToken".to_string(),
            updateToken: "UpdateToken".to_string(),
            dataUrl: Some("url".to_string()),
            fileNameData: "TestNameData".to_string(),
            fileNameHash: "TestNameHash".to_string(),
            salt: "TestSalt".to_string(),
            iv: "TestIV".to_string(),
            createdAt: Utc::now(),
            expiresAt: Utc::now() + Duration::days(1),
        };

        let file = db.insert_file(&file);

        let new_file = InsertFile {
            expiresAt: Utc::now() + Duration::days(2),
            ..file
        };

        db.update_file_expiry(&new_file);
        //delete_file(&mut conn, file);
    }

    #[test]
    fn test_insert_delete() {
        let mut db = initialize();

        let file = InsertFile {
            id: None,
            uuid: uuid::Uuid::new_v4(),
            accessToken: "AccessToken".to_string(),
            updateToken: "UpdateToken".to_string(),
            dataUrl: Some("url".to_string()),
            fileNameData: "TestNameData".to_string(),
            fileNameHash: "TestNameHash".to_string(),
            salt: "TestSalt".to_string(),
            iv: "TestIV".to_string(),
            createdAt: Utc::now(),
            expiresAt: Utc::now(),
        };

        let file = db.insert_file(&file);
        db.delete_file(&file);
    }
}
*/
