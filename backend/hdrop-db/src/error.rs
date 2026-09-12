use diesel::result::Error::NotFound;
use diesel_async::pooled_connection::deadpool;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Environment error: {0}")]
    Env(#[from] hdrop_shared::env::EnvError),
    #[error("{0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("{0}")]
    DeadpoolBuild(#[from] deadpool::BuildError),
    #[error("{0}")]
    DeadpoolPool(#[from] deadpool::PoolError),
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Diesel(NotFound))
    }
}
