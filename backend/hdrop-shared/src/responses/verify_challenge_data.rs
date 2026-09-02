use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct VerifyChallengeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_hash: Option<String>,
    pub file_name_data: String,
}
