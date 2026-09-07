use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeData {
    pub challenge: String,
}
