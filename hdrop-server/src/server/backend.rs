use super::{
    routes::{
        delete_file, get_challenge, get_file, get_raw_file_bytes, update_file_expiry, upload_file,
        verify_challenge,
    },
    CacheVariant,
};
use crate::{
    background_workers::expiration_worker::ExpirationWorker,
    background_workers::storage_synchronizer::{ProviderSyncEntry, StorageSynchronizer},
    core::{S3Provider, StorageProvider},
    error::{Error, Result},
};
use axum::{
    extract::DefaultBodyLimit,
    http::HeaderValue,
    routing::{get, post},
    Router,
};
use hdrop_db::Database;
use std::{env, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    RwLock,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub tx: UnboundedSender<ProviderSyncEntry>,
    pub cache: Arc<RwLock<CacheVariant>>,
}

impl AppState {
    async fn new(tx: UnboundedSender<ProviderSyncEntry>) -> Result<Self> {
        Ok(AppState {
            provider: Arc::new(RwLock::new(Box::new(S3Provider::try_from_env()?))),
            database: Arc::new(Database::try_from_env()?),
            tx,
            cache: Arc::new(RwLock::new(CacheVariant::try_from_env().await?)),
        })
    }
}

pub async fn start_server() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hdrop_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // Load environment variables from .env file. (for development)
    // Fails if .env file not found, not readable or invalid.
    _ = dotenvy::dotenv();

    // Start storage synchronizer worker
    let (tx, rx) = mpsc::unbounded_channel::<ProviderSyncEntry>();
    _ = tokio::spawn(StorageSynchronizer::new(rx).run());

    // Get db, storage provider and cache
    let state = Arc::new(AppState::new(tx).await?);
    tracing::info!("AppState (Database, StorageProvider, Cache) created");
    // Start recovery process and try to recover cache if needed
    let mut cache = state.cache.write().await;
    match cache.recover().await {
        Ok(val) => tracing::info!("Recovery finished: recovered {val} files"),
        Err(err) => match err {
            Error::NoRecover => tracing::info!(
                "No recovery executed, as it does not exist for the given cache strategy"
            ),
            err => tracing::error!("Recovery failed: {err}"),
        },
    };
    drop(cache);

    let expiration_worker_entry = ExpirationWorker::new(
        state.provider.clone(),
        state.database.clone(),
        state.cache.clone(),
    );
    // Background Worker for deleting expired files, every 60 seconds
    _ = tokio::spawn(expiration_worker_entry.run());

    // API Routes
    let app = Router::new()
        //.route("/test", get(|| async { "Hello, World!" }))
        .route(
            "/v1/files",
            post(upload_file).layer(DefaultBodyLimit::max(256 * 1024 * 1024)), // 256MB
        )
        .route("/v1/files/:access_token", get(get_file).delete(delete_file))
        .route("/v1/files/:access_token/expiry", post(update_file_expiry))
        .route("/v1/files/:access_token/raw", get(get_raw_file_bytes))
        .route(
            "/v1/files/:access_token/challenge",
            get(get_challenge).post(verify_challenge),
        )
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(std::env::var("CORS_ORIGIN")?.parse::<HeaderValue>()?)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // ToDo: Route & api refactor:: BELG
    //let files_route ;

    let server_port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = format!("0.0.0.0:{server_port}");

    let addr = SocketAddr::from_str(&host).unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    tracing::info!("Server started");
    Ok(())
}
