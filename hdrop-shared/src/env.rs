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

// Server
env_get!(port => u16);
env_get!(cors_origin);
env_get!(single_file_limit_mb => usize);

// Database
env_get!(database_url);

// Cache
env_get!(cache_strategy);
env_get!(cache_memory_limit_mb => usize);
env_get!(cache_disk_limit_mb => usize);
env_get!(cache_dir => PathBuf);

// S3 Provider
env_get!(s3_region);
env_get!(s3_endpoint);
env_get!(s3_access_key_id);
env_get!(s3_secret_access_key);
env_get!(s3_bucket_name);
env_get!(s3_public_url);

// On-Premise Provider
env_get!(onpremise_storage_dir => PathBuf);
env_get!(onpremise_storage_limit_mb => usize);
