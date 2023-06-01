use diesel::prelude::*;
// postgres://postgres:postgres@postgres:5432/hdrop
// docker-compose up postgres

use crate::models::*;
use anyhow::Result;
use crate::schema::files;
use crate::schema::files::dsl::*;
use ::uuid::Uuid;
use diesel::pg::PgConnection;
use dotenvy::dotenv;
use std::env;
pub struct Database {
    connection: PgConnection,
}

impl Database {
    pub fn new() -> Result<Database> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pgconn = PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        Ok(Database { connection: pgconn })
    }

    pub fn insert_file(&self, file: InsertFile) -> InsertFile {
        //let data = serde_json::to_value(file).unwrap();
        diesel::insert_into(files::table)
            .values(&file)
            .get_result(&mut self.connection)
            .expect("Error saving new file")
        /*
        ToDo: thiserror, return Result types

        */
        //unimplemented!("");
    }

    pub fn update_file(&self, file: InsertFile) {
        diesel::update(files.filter(uuid.eq(&file.uuid)))
            .set(&file)
            .execute(&mut self.connection)
            .expect("update failed");
    }

    pub fn get_table() {
        // database
        unimplemented!("");
    }

    pub fn update_file_expiry(&self, file: InsertFile) {
        diesel::update(files.filter(uuid.eq(&file.uuid)))
            .set(expiresAt.eq(&file.expiresAt))
            .execute(&mut self.connection)
            .expect("update failed");
    }

    pub fn get_file(&self, s_uuid: &Uuid) -> Option<File> {
        use crate::schema::files::dsl::*;
        files
            .filter(uuid.eq(s_uuid))
            .first(&mut self.connection)
            .ok()
    }

    pub fn delete_file(&self, file: InsertFile) -> File {
        diesel::delete(files.filter(uuid.eq(&file.uuid)))
            .get_result(&mut self.connection)
            .expect("Error deleting files row")
    }
}

/*
pub fn get_file(conn: &mut PgConnection, id_uuid: Uuid) -> QueryResult<File> {
    files.filter(uuid.eq(id_uuid))
         .first(conn)
}
*/

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
        let mut db = Database::new().unwrap();

        sql_query("DROP SCHEMA IF EXISTS public CASCADE")
            .execute(&mut db.connection)
            .expect("Error dropping db public");

        sql_query("CREATE SCHEMA public")
            .execute(&mut db.connection)
            .expect("Error creating db public");

        sql_query("GRANT ALL ON SCHEMA public TO postgres")
            .execute(&mut db.connection)
            .expect("Error creating db public");

        sql_query("GRANT ALL ON SCHEMA public TO public")
            .execute(&mut db.connection)
            .expect("Error creating db public");

        run_migrations(&mut db.connection).expect("error running migrations");

        sql_query("ALTER SEQUENCE files_id_seq RESTART WITH 1")
            .execute(&mut db.connection)
            .expect("Error resetting id sequence");

        db
    }

    #[test]
    fn test_connection() {
        //let conn = establish_connection();
        let _ = Database::new().unwrap();
    }

    #[test]
    fn test_insert() {
        let db = initialize();

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
            createdAt: Utc::now().naive_utc(),
            expiresAt: Utc::now().naive_utc(),
        };

        db.insert_file(file);
    }

    #[test]
    fn test_insert_update() {
        let db = initialize();

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
            createdAt: Utc::now().naive_utc(),
            expiresAt: Utc::now().naive_utc() + Duration::days(1),
        };

        let file = db.insert_file(file);

        let new_file = InsertFile {
            expiresAt: Utc::now().naive_utc() + Duration::days(2),
            ..file
        };

        db.update_file_expiry(new_file);
        //delete_file(&mut conn, file);
    }

    #[test]
    fn test_insert_delete() {
        let db = initialize();

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
            createdAt: Utc::now().naive_utc(),
            expiresAt: Utc::now().naive_utc(),
        };

        let file = db.insert_file(file);
        db.delete_file(file);
    }
}
