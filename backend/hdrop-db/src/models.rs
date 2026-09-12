use ::uuid::Uuid;
use diesel::prelude::*;
use jiff_diesel::Timestamp;

#[derive(Debug, Queryable, Selectable, Insertable)]
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
    pub createdAt: Timestamp,
    pub expiresAt: Timestamp,
    pub challengeData: String,
    pub challengeHash: String,
}
