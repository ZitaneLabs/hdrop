use serde::Serialize;
mod file_metadata;
mod get_challenge_data;
mod upload_file_data;
mod verify_challenge_data;

pub use file_metadata::FileMetaData;
pub use get_challenge_data::GetChallengeData;
pub use upload_file_data::UploadFileData;
pub use verify_challenge_data::VerifyChallengeData;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    reason: &'static str,
}

impl ErrorResponse {
    pub fn new(reason: &'static str) -> Self {
        Self { reason }
    }
}
