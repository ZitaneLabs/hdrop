use std::net::AddrParseError;

use axum::{extract::multipart::MultipartError, http::StatusCode, response::IntoResponse, Json};
use bincache::Error as BincacheError;
use hdrop_shared::ErrorResponse;
use regex::Error as RegexError;
use s3::{creds::error::CredentialsError, error::S3Error};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // Env
    #[error("Environment error: {0}")]
    Env(#[from] hdrop_shared::env::EnvError),
    // StorageProvider
    #[error("Invalid Storage provider: {0}")]
    InvalidProvider(String),
    #[error("No Storage provider given")]
    NoProvider,
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
    #[error("Invalid CORS origin: {0}")]
    InvalidCorsOrigin(String),
    // Webserver
    #[error("{0}")]
    Multipart(#[from] MultipartError),
    #[error("Conversion to UploadedFileData failed due to PartialUploadedFileData being incomplete (Missing field: {0})")]
    FileDataConversionError(&'static str),
    #[error("Challenge failed")]
    InvalidChallenge,
    #[error("Wrong update token")]
    UpdateToken,
    #[error("Invalid Expiry")]
    InvalidExpiry,
    #[error("Unable to locate file data")]
    InvalidFile,
    #[error("File upload failed: {reason}")]
    FileUpload { reason: String },
    #[error("Socket could not get parsed: {0}")]
    SocketParse(#[from] AddrParseError),
    #[error("Storage provider deletion failed")]
    ProviderDeletion,
    #[error("Database deletion failed")]
    DatabaseDeletion,
    #[error("Cache deletion failed")]
    CacheDeletion,
    // Cache
    #[error("Recover does not exist for Disk or Memory Strategy")]
    NoRecover,
    #[error("Cache variant does not exist. Existing Cache variants: Memory, Disk, Hybrid")]
    Strategy,
    // DB
    #[error("{0}")]
    Database(#[from] hdrop_db::error::Error),
}

impl Error {
    pub fn to_statuscode(&self) -> StatusCode {
        match self {
            Self::FileUpload { .. } => StatusCode::BAD_REQUEST,
            Self::FileDataConversionError(_) => StatusCode::BAD_REQUEST,
            Self::InvalidChallenge => StatusCode::UNAUTHORIZED,
            Self::UpdateToken => StatusCode::UNAUTHORIZED,
            Self::InvalidExpiry => StatusCode::BAD_REQUEST,
            Self::Database(e) if e.is_not_found() => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let reason = match &self {
            Self::FileUpload { .. } => "File upload failed",
            Self::InvalidChallenge => "Challenge failed",
            Self::UpdateToken => "Wrong update token",
            Self::InvalidExpiry => "Invalid Expiry",
            Self::InvalidFile => "Unable to locate file data",
            Self::Database(e) if e.is_not_found() => "No file found for given access token",
            Self::ProviderDeletion | Self::CacheDeletion => "File deletion failed",
            Self::DatabaseDeletion => "File contents safely deleted, database could not delete additional metadata. This is safe, database will be purged automatically later",
            _ => "Unexpected error",
        };

        let error = ErrorResponse::new(reason);

        tracing::error!("{:?}", self);
        (self.to_statuscode(), Json(error)).into_response()
    }
}
