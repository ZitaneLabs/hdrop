use std::path::PathBuf;

use once_cell::sync::OnceCell;
use paste::paste;

#[derive(Debug, Clone, thiserror::Error)]
pub enum EnvError {
    #[error("Environment variable '{key}' not found.")]
    KeyNotFound { key: String },
    #[error("Unable to parse environment variable '{key}'.")]
    ParseError { key: String },
}

macro_rules! env_get {
    ($name:ident) => {
        paste! {
            static [<$name:upper _CELL>]: OnceCell<Result<String, EnvError>> = OnceCell::new();

            #[doc ="Get the value of the '" $name:upper "' environment variable."]
            pub fn [<$name>]() -> Result<String, EnvError> {
                [<$name:upper _CELL>].get_or_init(|| {
                    std::env::var(stringify!([<$name:upper>]))
                        .map_err(|_| EnvError::KeyNotFound { key: stringify!([<$name:upper>]).to_string() })
                }).clone()
            }
        }
    };

    ($name:ident => $target_type:ty) => {
        paste! {
            static [<$name:upper _CELL>]: OnceCell<Result<$target_type, EnvError>> = OnceCell::new();

            #[doc ="Get the value of the '" $name:upper "' environment variable."]
            pub fn [<$name>]() -> Result<$target_type, EnvError> {
                [<$name:upper _CELL>].get_or_init(|| {
                    std::env::var(stringify!([<$name:upper>]))
                        .map_err(|_| EnvError::KeyNotFound { key: stringify!([<$name:upper>]).to_string() })?
                        .parse()
                        .map_err(|_| EnvError::ParseError { key: stringify!([<$name:upper>]).to_string() })
                }).clone()
            }
        }
    };
}

macro_rules! env_and_const {
    ($($name:ident => $type:ty),* $(,)?) => {
        const ENV_VARS_BACKEND: &[&str] = &[$(stringify!($name),)*];
        $(
            env_get!($name => $type);
        )*
    };
}

// Generate the environment variables metadata and fetch functions
env_and_const!(
// Server
hdrop_port => u16,
prometheus_port => u16,
cors_origin => String,
single_file_limit_mb => usize,
storage_provider => String,
// Database
database_url => String,
// Cache
cache_strategy => String,
cache_memory_limit_mb => usize,
cache_disk_limit_mb => usize,
cache_dir => PathBuf,
// S3 Provider
s3_region => String,
s3_endpoint => String,
s3_access_key_id => String,
s3_secret_access_key => String,
s3_bucket_name => String,
s3_public_url => String,
// Local Provider
local_storage_dir => PathBuf,
local_storage_limit_mb => usize,
);

/// Get a list of all environment variables used in the hdrop backend.
pub fn get_env_vars() -> Vec<String> {
    ENV_VARS_BACKEND
        .iter()
        .map(|&var| var.to_uppercase())
        .collect()
}
