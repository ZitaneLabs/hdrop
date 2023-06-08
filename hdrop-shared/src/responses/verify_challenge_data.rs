use serde::Serialize;

// ToDo Idea: Squash GetChallengeData & ChallengeFinishData together with Option<String> on challenge and serde skip serialize if None
#[derive(Debug, Serialize)]
pub struct VerifyChallengeData {
    // Belg Verifychallenge data mit option<file_name_hash>
    // serde skip serialize wenn none
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name_hash: Option<String>,
    pub iv: String,
    pub salt: String,
}
