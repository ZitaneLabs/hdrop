use axum::extract::multipart::MultipartError;
use bincache::Error as BincacheError;
use regex::Error as RegexError;
use s3::{creds::error::CredentialsError, error::S3Error};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // Env
    #[error("Environment error: {0}")]
    Env(#[from] hdrop_shared::env::EnvError),
    // S3
    #[error("S3 error: {0}")]
    S3(#[from] S3Error),
    #[error("S3 error: {0}")]
    S3Credential(#[from] CredentialsError),
    #[error("Regex error: {0}")]
    Regex(#[from] RegexError),
    #[error("Bincache error: {0}")]
    Cache(#[from] BincacheError),
    #[error("{0}")]
    InvalidHeaderValue(#[from] axum::http::header::InvalidHeaderValue),
    // Webserver
    #[error("{0}")]
    Multipart(#[from] MultipartError),
    #[error("Conversion to UploadedFileData failed due to PartialUploadedFileData being incomplete (Missing field: {0})")]
    FileDataConversionError(&'static str),
    #[error("Conversion to UploadedFileData failed due to file being too large (Size: {0}, Allowed size: {1})")]
    FileLimitExceeded(&'static str, &'static str),
    #[error("{0}")]
    State(#[from] hdrop_db::error::Error),
    #[error("File upload failed: {reason}")]
    FileUpload { reason: String },
    #[error("Recover does not exist for Disk or Memory Strategy")]
    NoRecover,
    #[error("Cache variant does not exist. Existing Cache variants: Memory, Disk, Hybrid")]
    Strategy,
}
