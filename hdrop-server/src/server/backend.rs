use axum::{
    extract::DefaultBodyLimit,
    http::HeaderValue,
    routing::{delete, get, post},
    Router,
};
use hdrop_db::Database;
use std::{env, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use super::routes::{
    access_file, delete_file, get_challenge, get_raw_file_bytes, update_file_expiry, upload_file,
    verify_challenge,
};
use crate::{
    core::{S3Provider, StorageProvider},
    error::Result,
};

pub struct AppState {
    pub provider: RwLock<Box<dyn StorageProvider + Sync + Send>>,
    pub database: Database,
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

    let state = Arc::new(AppState {
        provider: RwLock::new(Box::new(S3Provider::try_from_env()?)),
        database: Database::try_from_env()?,
    });

    let app = Router::new()
        .route("/test", get(|| async { "Hello, World!" }))
        .route("/v1/files", post(upload_file))
        .route("/v1/files/:access_token", get(access_file))
        .route("/v1/files/:access_token", delete(delete_file))
        .route("/v1/files/:access_token/expiry", post(update_file_expiry))
        .route("/v1/files/:access_token/raw", get(get_raw_file_bytes))
        .route("/v1/files/:access_token/challenge", get(get_challenge))
        .route("/v1/files/:access_token/challenge", post(verify_challenge))
        .with_state(state)
        .layer(DefaultBodyLimit::max(256 * 1024 * 1024)) // 256MB
        .layer(
            CorsLayer::new()
                .allow_origin(
                    std::env::var("CORS_ORIGIN")
                        .unwrap()
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .allow_methods(Any),
        );

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
