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
pub struct Response<T: Serialize> {
    status: &'static str,
    data: ResponseData<T>,
}

impl<T: Serialize> Response<T> {
    pub fn new(data: ResponseData<T>) -> Self {
        let status = match data {
            ResponseData::Success(_) => "success",
            ResponseData::Error(_) => "error",
        };
        Self { status, data }
    }

    pub fn error(reason: String) -> Self {
        Response::new(ResponseData::Error(ErrorData { reason }))
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ResponseData<T: Serialize> {
    Success(T),
    Error(ErrorData),
}

#[derive(Debug, Serialize)]
pub struct ErrorData {
    pub reason: String,
}
