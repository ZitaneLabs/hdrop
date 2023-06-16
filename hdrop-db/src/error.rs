use diesel::ConnectionError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Environment error: {0}")]
    Env(#[from] hdrop_shared::env::EnvError),
    #[error("{0}")]
    Connection(#[from] ConnectionError),
    #[error("{0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("{0}")]
    DeadpoolBuild(#[from] deadpool::managed::BuildError<deadpool_diesel::Error>),
    #[error("{0}")]
    DeadpoolPool(
        #[from]
        deadpool::managed::PoolError<
            <deadpool_diesel::postgres::Manager as deadpool::managed::Manager>::Error,
        >,
    ),
    #[error("{0}")]
    DeadpoolInteract(#[from] deadpool_diesel::postgres::InteractError),
    #[error("Conversion to UploadedFileData failed due to PartialUploadedFileData being incomplete (Missing field: {0})")]
    FileDataConversionError(&'static str),
}
