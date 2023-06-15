use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct VerifyChallengeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name_hash: Option<String>,
    pub iv: String,
    pub salt: String,
}