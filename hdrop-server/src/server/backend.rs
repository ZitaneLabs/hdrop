use crate::core::{S3Provider, StorageProvider};
//use crate::schema::files::dataUrl;
use axum::{
    extract::Extension,
    http::HeaderValue,
    routing::{delete, get, post},
    Router,
};
use hdrop_db::Database;

use std::{env, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use crate::error::{Error, Result};
use super::routes::{
    access_file, delete_file, get_challenge, get_raw_file_bytes, update_file_expiry, upload_file,
    verify_challenge,
};

//#[derive(Clone)]
pub struct StateInfos<T: StorageProvider> {
    pub provider: T,
    pub db: Database // ToDo: this is flawed, PgConnection shouldn't be restricted behind an Rw or mutex imo
}

impl<T: StorageProvider> StateInfos<T> {
    pub fn try_from_env(provider: T) -> Result<Self> {
        Ok(Self { 
            provider, 
            db: Database::try_from_env()?
        })
    }
}

pub async fn start_server() -> Result<()> {
    // ToDo: Start Recovery Process (Cache Issue Github)
    // ToDo: Background Worker for deleting expired files, every 5minutes and yielding in between

    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    _ = dotenvy::dotenv();

    for (key, value) in env::vars() {
        println!("{key}: {value}");
    }

    let storage_provider = S3Provider::try_from_env().unwrap(); // ToDo: Add env var to decide which storageprovider
    let state_info = Arc::new(StateInfos::try_from_env(storage_provider)?);

    let app = Router::new()
        .route("/test", get(|| async { "Hello, World!" }))
        .route("/v1/files", post(upload_file))
        .route("/v1/files/:access_token", get(access_file))
        .route("/v1/files/:access_token", delete(delete_file))
        .route("/v1/files/:access_token/expiry", post(update_file_expiry))
        .route("/v1/files/:access_token/raw", get(get_raw_file_bytes))
        .route("/v1/files/:access_token/challenge", get(get_challenge))
        .route("/v1/files/:access_token/challenge", post(verify_challenge))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    std::env::var("CORS_ORIGIN")
                        .unwrap()
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .allow_methods(Any),
        )
        .layer(Extension(state_info));

    let server_port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = format!("0.0.0.0:{server_port}");
    let addr = SocketAddr::from_str(&host).unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

/*
// Responses
#[derive(Debug, Serialize)]
enum ResponesType<T> {
    Success(T),
    Error(ErrorData)
}

#[derive(Debug, Serialize)]
pub struct Response<T> {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct ErrorData {
    reason: String,
}

impl<T> Response<T>
where
    T: Serialize,
{
    fn success() -> Response<()> {
        Response {
            status: "success".to_string(),
            data: None,
        }
    }

    fn octet_steam(octet_stream: Vec<u8>) -> Response<Vec<u8>> {
        Response {
            status: "success".to_string(),
            data: Some(octet_stream),
        }
    }

    fn missing_update_token() -> Response<ErrorData> {
        Response {
            status: "error".to_string(),
            data: Some(ErrorData {
                reason: "Missing update token".to_string(),
            }),
        }
    }
}
*/
