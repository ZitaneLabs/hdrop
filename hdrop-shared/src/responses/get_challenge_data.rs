use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GetChallengeData {
    pub challenge: String,
    pub iv: String,
    pub salt: String,
}
