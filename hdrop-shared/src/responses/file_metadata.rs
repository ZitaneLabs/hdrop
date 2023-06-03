use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FileMetaData {
    pub file_url: Option<String>,
    pub file_name_data: String,
    pub iv: String,
    pub salt: String,
}
