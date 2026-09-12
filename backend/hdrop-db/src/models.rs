use ::uuid::Uuid;
use chrono::{DateTime, Utc};
use diesel::prelude::*;

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
    pub createdAt: DateTime<Utc>,
    pub expiresAt: DateTime<Utc>,
    pub challengeData: String,
    pub challengeHash: String,
}
