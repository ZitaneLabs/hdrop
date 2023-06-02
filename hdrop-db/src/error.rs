use diesel::ConnectionError;
use thiserror::Error;
use std::env::VarError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Env(#[from] VarError),
    #[error("{0}")]
    Connection(#[from] ConnectionError),
    #[error("Conversion to UploadedFileData failed due to PartialUploadedFileData being incomplete (Missing field: {0})")]
    FileDataConversionError(&'static str),
}
