use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GetChallengeData {
    pub salt: String,
    pub iv: String,
    pub challenge: String,
}
