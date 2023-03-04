use diesel::prelude::*;
// postgres://postgres:postgres@postgres:5432/hdrop
// docker-compose up postgres

use crate::core::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_types::Uuid;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

use crate::schema::files::dsl::*;
pub fn get_table() {
    // database

    unimplemented!("");
}
/*
pub fn get_file(conn: &mut PgConnection, id_uuid: Uuid) -> QueryResult<File> {
    files.filter(uuid.eq(id_uuid))
         .first(conn)
}
*/

use crate::schema::files;
pub fn insert_file(conn: &mut PgConnection, file: File) -> File {
    diesel::insert_into(files::table)
        .values(&file)
        .get_result(conn)
        .expect("Error saving new file")
    /*
    ToDo: thiserror, return Result types

    */
    //unimplemented!("");
}

pub fn update_file(conn: &mut PgConnection, file: File) {
    diesel::update(files.filter(uuid.eq(&file.uuid)))
        .set(&file)
        .execute(conn)
        .expect("update failed");
}

pub fn update_file_expiry(conn: &mut PgConnection, file: File) {
    diesel::update(files.filter(uuid.eq(&file.uuid)))
        .set(expiresAt.eq(&file.expiresAt))
        .execute(conn)
        .expect("update failed");
}

pub fn delete_file(conn: &mut PgConnection, file: File) -> File {
    diesel::delete(files.filter(uuid.eq(&file.uuid)))
        .get_result(conn)
        .expect("Error deleting files row")
}

pub fn get_file(conn: &mut PgConnection, s_uuid: Uuid) -> Option<File> {
    files.filter(uuid.eq(&s_uuid)).first(conn).optional().ok()?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::File;

    use chrono::{Duration, Utc};
    use diesel::{pg::Pg, sql_query, ConnectionError};
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
    fn initialize() {
        let mut conn = super::establish_connection();

        sql_query("DROP SCHEMA IF EXISTS public CASCADE")
            .execute(&mut conn)
            .expect("Error dropping db public");

        sql_query("CREATE SCHEMA public")
            .execute(&mut conn)
            .expect("Error creating db public");

        sql_query("GRANT ALL ON SCHEMA public TO postgres")
            .execute(&mut conn)
            .expect("Error creating db public");

        sql_query("GRANT ALL ON SCHEMA public TO public")
            .execute(&mut conn)
            .expect("Error creating db public");

        run_migrations(&mut conn).expect("error running migrations");
    }

    fn compare(file: File) {

    }

    #[test]
    fn test_connection() {
        //let conn = establish_connection();
        let conn = super::establish_connection();
    }

    #[test]
    fn test_insert() {
        initialize();

        let mut conn = super::establish_connection();

        let file = File {
            id: 1,
            uuid: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            accessToken: "AccessToken".to_string(),
            updateToken: "UpdateToken".to_string(),
            dataUrl: Some("url".to_string()),
            fileNameData: "TestNameData".to_string(),
            fileNameHash: "TestNameHash".to_string(),
            salt: "TestSalt".to_string(),
            iv: "TestIV".to_string(),
            createdAt: Utc::now().naive_utc(),
            expiresAt: Utc::now().naive_utc(),
        };

        insert_file(&mut conn, file);
    }

    #[test]
    fn test_insert_update() {
        initialize();
        let mut conn = super::establish_connection();

        let mut file = File {
            id: 1,
            uuid: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            accessToken: "AccessToken".to_string(),
            updateToken: "UpdateToken".to_string(),
            dataUrl: Some("url".to_string()),
            fileNameData: "TestNameData".to_string(),
            fileNameHash: "TestNameHash".to_string(),
            salt: "TestSalt".to_string(),
            iv: "TestIV".to_string(),
            createdAt: Utc::now().naive_utc(),
            expiresAt: Utc::now().naive_utc() + Duration::days(1),
        };

        insert_file(&mut conn, file.clone());

        let new_file = File {
            expiresAt: Utc::now().naive_utc() + Duration::days(2),
            ..file
        };

        update_file_expiry(&mut conn, new_file.clone());
        //delete_file(&mut conn, file);
    }

    #[test]
    fn test_insert_delete() {
        initialize();

        let mut conn = super::establish_connection();

        let file = File {
            id: 1,
            uuid: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            accessToken: "AccessToken".to_string(),
            updateToken: "UpdateToken".to_string(),
            dataUrl: Some("url".to_string()),
            fileNameData: "TestNameData".to_string(),
            fileNameHash: "TestNameHash".to_string(),
            salt: "TestSalt".to_string(),
            iv: "TestIV".to_string(),
            createdAt: Utc::now().naive_utc(),
            expiresAt: Utc::now().naive_utc(),
        };

        insert_file(&mut conn, file.clone());
        delete_file(&mut conn, file.clone());
    }
}
