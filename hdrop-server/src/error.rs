use axum::extract::multipart::MultipartError;
use s3::{creds::error::CredentialsError, error::S3Error};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // S3
    #[error("{0}")]
    S3(#[from] S3Error),
    #[error("{0}")]
    S3Credential(#[from] CredentialsError),
    #[error("{0}")]
    OtherProvider(String),
    // Webserver
    #[error("{0}")]
    Multipart(#[from] MultipartError),
    #[error("Missing field name in Multipart Formdata")]
    MissingFieldName,
    #[error("Conversion to UploadedFileData failed due to PartialUploadedFileData being incomplete (Missing field: {0})")]
    FileDataConversionError(&'static str),
    #[error("{0}")]
    State(#[from] hdrop_db::error::Error),
}
