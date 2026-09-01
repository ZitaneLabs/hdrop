use std::{path::PathBuf, sync::OnceLock, time::Duration};

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
            static [<$name:upper _CELL>]: OnceLock<Result<String, EnvError>> = OnceLock::new();

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
            static [<$name:upper _CELL>]: OnceLock<Result<$target_type, EnvError>> = OnceLock::new();

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

macro_rules! map_env {
    ($($name:ident => $type:ty),* $(,)?) => {
        const ENV_VARS_BACKEND: &[&str] = &[$(stringify!($name),)*];
        $(
            env_get!($name => $type);
        )*
    };
}

// Generate the environment variables metadata and fetch functions
map_env!(
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

static S3_ADDRESSING_STYLE_CELL: OnceLock<Result<bool, EnvError>> = OnceLock::new();

/// Whether S3 requests should use virtual-hosted-style addressing.
///
/// Path-style addressing remains the default when `S3_ADDRESSING_STYLE` is unset.
pub fn s3_virtual_hosted_style_request() -> Result<bool, EnvError> {
    S3_ADDRESSING_STYLE_CELL
        .get_or_init(|| {
            let value = match std::env::var("S3_ADDRESSING_STYLE") {
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(EnvError::ParseError {
                        key: "S3_ADDRESSING_STYLE".to_string(),
                    });
                }
            };

            parse_s3_addressing_style(value.as_deref())
        })
        .clone()
}

fn parse_s3_addressing_style(value: Option<&str>) -> Result<bool, EnvError> {
    match value.unwrap_or("path") {
        "path" => Ok(false),
        "virtual" => Ok(true),
        _ => Err(EnvError::ParseError {
            key: "S3_ADDRESSING_STYLE".to_string(),
        }),
    }
}

static S3_REQUEST_TIMEOUT_CELL: OnceLock<Result<Option<Duration>, EnvError>> = OnceLock::new();

/// Get the optional overall S3 request timeout. An unset variable disables it.
pub fn s3_request_timeout() -> Result<Option<Duration>, EnvError> {
    S3_REQUEST_TIMEOUT_CELL
        .get_or_init(|| {
            let value = match std::env::var("S3_REQUEST_TIMEOUT_SECS") {
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(EnvError::ParseError {
                        key: "S3_REQUEST_TIMEOUT_SECS".to_string(),
                    });
                }
            };

            parse_s3_request_timeout(value.as_deref())
        })
        .clone()
}

fn parse_s3_request_timeout(value: Option<&str>) -> Result<Option<Duration>, EnvError> {
    match value {
        None => Ok(None),
        Some(value) => value
            .parse::<u64>()
            .ok()
            .filter(|seconds| *seconds > 0)
            .map(|seconds| Some(Duration::from_secs(seconds)))
            .ok_or_else(|| EnvError::ParseError {
                key: "S3_REQUEST_TIMEOUT_SECS".to_string(),
            }),
    }
}

/// Get a list of all environment variables used in the hdrop backend.
pub fn get_env_vars() -> Vec<String> {
    ENV_VARS_BACKEND
        .iter()
        .map(|&var| var.to_uppercase())
        .chain([
            "S3_ADDRESSING_STYLE".to_string(),
            "S3_REQUEST_TIMEOUT_SECS".to_string(),
        ])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s3_addressing_style_defaults_to_path() {
        assert!(!parse_s3_addressing_style(None).unwrap());
    }

    #[test]
    fn parses_s3_addressing_styles() {
        assert!(!parse_s3_addressing_style(Some("path")).unwrap());
        assert!(parse_s3_addressing_style(Some("virtual")).unwrap());
    }

    #[test]
    fn rejects_invalid_s3_addressing_style() {
        assert!(matches!(
            parse_s3_addressing_style(Some("auto")),
            Err(EnvError::ParseError { key }) if key == "S3_ADDRESSING_STYLE"
        ));
    }

    #[test]
    fn s3_request_timeout_is_optional() {
        assert_eq!(parse_s3_request_timeout(None).unwrap(), None);
    }

    #[test]
    fn parses_s3_request_timeout() {
        assert_eq!(
            parse_s3_request_timeout(Some("45")).unwrap(),
            Some(Duration::from_secs(45))
        );
    }

    #[test]
    fn rejects_invalid_s3_request_timeouts() {
        for value in ["", "0", "-1", "invalid"] {
            assert!(matches!(
                parse_s3_request_timeout(Some(value)),
                Err(EnvError::ParseError { key }) if key == "S3_REQUEST_TIMEOUT_SECS"
            ));
        }
    }
}
