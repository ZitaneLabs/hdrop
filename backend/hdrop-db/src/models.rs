use ::uuid::Uuid;
use diesel::prelude::*;
use jiff::Timestamp;
use jiff_diesel::Timestamp as DbTimestamp;
use serde::{Deserialize, Serialize};

#[derive(
    Default, Serialize, Deserialize, Debug, Queryable, Selectable, Identifiable, AsChangeset,
)]
#[diesel(primary_key(uuid))]
#[diesel(table_name = crate::schema::files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(non_snake_case)]
pub struct File {
    pub uuid: Uuid,
    pub accessToken: String,
    pub updateToken: String,
    pub dataUrl: Option<String>,
    pub fileNameData: String,
    pub salt: String,
    pub iv: String,
    #[diesel(deserialize_as = DbTimestamp, serialize_as = DbTimestamp)]
    pub createdAt: Timestamp,
    #[diesel(deserialize_as = DbTimestamp, serialize_as = DbTimestamp)]
    pub expiresAt: Timestamp,
    pub challengeData: String,
    pub challengeHash: String,
}

#[derive(Debug, Insertable, Identifiable, AsChangeset)]
#[diesel(primary_key(uuid))]
#[diesel(table_name = crate::schema::files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(non_snake_case)]
pub struct InsertFile {
    #[diesel(deserialize_as = i32)]
    pub uuid: Uuid,
    pub accessToken: String,
    pub updateToken: String,
    pub dataUrl: Option<String>,
    pub fileNameData: String,
    pub salt: String,
    pub iv: String,
    pub createdAt: DbTimestamp,
    pub expiresAt: DbTimestamp,
    pub challengeData: String,
    pub challengeHash: String,
}
