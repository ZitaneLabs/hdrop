use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UploadFileData {
    pub access_token: String,
    pub update_token: String,
}
