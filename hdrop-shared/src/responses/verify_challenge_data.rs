use serde::Serialize;

// ToDo Idea: Squash GetChallengeData & ChallengeFinishData together with Option<String> on challenge and serde skip serialize if None
#[derive(Debug, Serialize)]
pub struct VerifyChallengeData {
    pub iv: String,
    pub salt: String,
}
