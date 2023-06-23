use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FileMetaData {
    pub file_url: Option<String>,
}
