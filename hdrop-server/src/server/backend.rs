use axum::{
    body::Bytes,
    error_handling::{HandleError, HandleErrorLayer},
    extract::DefaultBodyLimit,
    http::HeaderValue,
    routing::{delete, get, post},
    BoxError, Json, Router,
};
use hdrop_db::{error, Database, File};
use hdrop_shared::{ErrorData, Response, ResponseData};
use std::{env, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tokio::{
    runtime::Handle,
    sync::{
        mpsc::{self, UnboundedSender},
        RwLock,
    },
};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use super::{
    background_workers::{ExpirationWorkerEntry, ProviderSyncEntry},
    routes::{
        access_file, delete_file, get_challenge, get_raw_file_bytes, update_file_expiry,
        upload_file, verify_challenge,
    },
};
use crate::{
    core::{S3Provider, StorageProvider},
    error::{Error, Result},
};
use bincache::{Cache, HybridCacheBuilder, HybridStrategy};

pub struct AppState {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub tx: UnboundedSender<ProviderSyncEntry>,
    pub cache: Arc<RwLock<Cache<Uuid, HybridStrategy>>>,
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

    let (tx, rx) = mpsc::unbounded_channel::<ProviderSyncEntry>();

    let _ = tokio::spawn(ProviderSyncEntry::storage_synchronizer(rx));

    let state = Arc::new(AppState {
        provider: Arc::new(RwLock::new(Box::new(S3Provider::try_from_env()?))),
        database: Arc::new(Database::try_from_env()?),
        tx,
        cache: Arc::new(RwLock::new(HybridCacheBuilder::new().build()?)),
    });

    let expiration_worker_entry = ExpirationWorkerEntry {
        provider: state.provider.clone(),
        database: state.database.clone(),
        cache: state.cache.clone(),
    };
    let _ = tokio::spawn(ExpirationWorkerEntry::expiration_worker(
        expiration_worker_entry,
    ));

    let app = Router::new()
        //.route("/test", get(|| async { "Hello, World!" }))
        .route(
            "/v1/files",
            post(upload_file).layer(DefaultBodyLimit::max(256 * 1024 * 1024)), // 256MB
        )
        .route(
            "/v1/files/:access_token",
            get(access_file).delete(delete_file), // ToDo: Rename access_file to get_file?
        )
        .route("/v1/files/:access_token/expiry", post(update_file_expiry))
        .route("/v1/files/:access_token/raw", get(get_raw_file_bytes))
        .route("/v1/files/:access_token/challenge", get(get_challenge))
        .route("/v1/files/:access_token/challenge", post(verify_challenge))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(std::env::var("CORS_ORIGIN")?.parse::<HeaderValue>()?)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Route & api refactor:: BELG
    //let files_route ;

    let server_port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = format!("0.0.0.0:{server_port}");
    let addr = SocketAddr::from_str(&host).unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

fn handle_error(err: Error) -> Json<Response<String>> {
    Json(Response::new(ResponseData::Error(ErrorData {
        reason: format!("{}", err),
    })))
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
