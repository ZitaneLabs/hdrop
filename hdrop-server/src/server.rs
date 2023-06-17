use axum::{
    extract::DefaultBodyLimit,
    http::HeaderValue,
    routing::{get, post},
    Router,
};
use hdrop_shared::env;
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, Any, CorsLayer},
    limit::RequestBodyLimitLayer,
};

mod app_state;
mod cache;
mod multipart;
mod routes;

use app_state::AppState;
pub use cache::CacheVariant;
use routes::{
    delete_file, get_challenge, get_file, get_raw_file_bytes, update_file_expiry, upload_file,
    verify_challenge,
};

use crate::{
    background_workers::{
        expiration_worker::ExpirationWorker,
        storage_synchronizer::{ProviderSyncEntry, StorageSynchronizer},
    },
    error::{Error, Result},
    utils::mb_to_bytes,
};

pub struct Server {
    state: Arc<AppState>,
    provider_sync_rx: UnboundedReceiver<ProviderSyncEntry>,
}

impl Server {
    /// Recover cache from last session, if possible.
    async fn recover_cache(&self) {
        let mut cache = self.state.cache.write().await;
        match cache.recover().await {
            Ok(val) => tracing::info!("Recovery finished: recovered {val} files"),
            Err(err) => match err {
                Error::NoRecover => tracing::info!(
                    "No recovery executed, as it's not supported for the given cache strategy"
                ),
                err => tracing::error!("Recovery failed: {err}"),
            },
        }
    }

    fn cors_origin() -> Result<AllowOrigin> {
        match env::cors_origin() {
            // Handle wildcard origin
            Ok(ref origin) if origin == "*" => Ok(AllowOrigin::any()),
            // Handle list of origins
            Ok(origins) => {
                let origin_list = origins
                    .split(',')
                    .map(|origin| {
                        origin
                            .trim()
                            .parse::<HeaderValue>()
                            .map_err(|_| Error::InvalidCorsOrigin(origin.to_string()))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(AllowOrigin::list(origin_list))
            }
            // Handle default origin
            Err(_) => Ok(AllowOrigin::any()),
        }
    }
}

impl Server {
    /// Create a new [Server] instance.
    pub async fn new() -> Result<Self> {
        // Initialize app state
        let (provider_sync_tx, provider_sync_rx) = mpsc::unbounded_channel();
        let state = Arc::new(AppState::new(provider_sync_tx).await?);

        Ok(Self {
            state,
            provider_sync_rx,
        })
    }

    /// Run the server.
    pub async fn run(self) -> Result<()> {
        // Recover cache from last session
        self.recover_cache().await;

        // Start storage synchronization worker.
        // This worker is responsible for synchronizing cached files with the storage provider.
        tokio::spawn(StorageSynchronizer::new(self.provider_sync_rx).run());

        // Start expiration worker.
        // This worker is responsible for deleting expired files from the storage provider and cache.
        tokio::spawn(
            ExpirationWorker::new(
                self.state.provider.clone(),
                self.state.database.clone(),
                self.state.cache.clone(),
            )
            .run(),
        );

        // Calculate request body limit
        let request_body_limit_bytes = mb_to_bytes(env::single_file_limit_mb().unwrap_or(100));

        // Define API routes
        let app = Router::new()
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
            .with_state(self.state)
            // Limit request body size
            .layer(RequestBodyLimitLayer::new(request_body_limit_bytes))
            // Use brotli compression if applicable
            .layer(CompressionLayer::new())
            // Order matters! The CORS layer must be the last layer in the middleware stack.
            .layer(
                CorsLayer::new()
                    .allow_origin(Self::cors_origin()?)
                    .allow_methods(Any)
                    .allow_headers(Any),
            );

        // Server configuration
        let server_host = "0.0.0.0";
        let server_port = env::port().unwrap_or(8080);
        let server_addr = format!("{server_host}:{server_port}");
        let socket_addr = SocketAddr::from_str(&server_addr)?;

        tracing::info!("Starting server on {server_addr}");

        // Start the server
        axum::Server::bind(&socket_addr)
            .serve(app.into_make_service())
            .await
            .unwrap_or_else(|err| panic!("Server failed to start on {server_addr}: {err:?}"));

        Ok(())
    }
}
