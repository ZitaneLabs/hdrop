use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::{
    extract::DefaultBodyLimit,
    http::HeaderValue,
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use hdrop_shared::{env, metrics::UpdateMetrics};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use super::{
    app_state::AppState,
    routes::{
        delete_file,
        get_challenge,
        get_file,
        update_file_expiry,
        upload_file,
        verify_challenge,
    },
};
use crate::{
    background_workers::{
        expiration_worker::ExpirationWorker,
        metrics_middleware,
        storage_synchronizer::{ProviderSyncEntry, StorageSynchronizer},
        MetricsUpdater,
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

    /// Run the main server.
    /// The order of execution must be maintained.
    ///
    /// 1. Upon running the server the cache will get recovered.
    ///
    /// 2. Background workers will start for storage synchronization and expiration workers.
    ///
    /// 3. Metrics will be initialized.
    ///
    /// 4. Lastly, the server will be initialized and started.
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

        // Init metrics after start
        // Update cache metrics
        self.state.cache.read().await.update_metrics().await;

        // Update db metrics
        self.state.database.update_metrics().await;

        // Update storage metrics
        self.state.provider.read().await.update_metrics().await;

        // Start metrics update worker for time-based updates of system gauges
        tokio::spawn(MetricsUpdater::new().update_metrics());

        // Calculate request body limit
        let request_body_limit_bytes = mb_to_bytes(env::single_file_limit_mb().unwrap_or(100));

        // Define API routes
        let app = Router::new()
            .route("/status", get(|| async { "OK" }))
            .route(
                "/v1/files",
                post(upload_file).layer(DefaultBodyLimit::max(request_body_limit_bytes)), // 256MB
            )
            .route("/v1/files/:access_token", get(get_file).delete(delete_file))
            .route("/v1/files/:access_token/expiry", post(update_file_expiry))
            .route(
                "/v1/files/:access_token/challenge",
                get(get_challenge).post(verify_challenge),
            )
            .with_state(self.state)
            // Limit request body size
            .layer(RequestBodyLimitLayer::new(request_body_limit_bytes))
            // Use brotli compression if applicable
            .layer(CompressionLayer::new())
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            .route_layer(from_fn(metrics_middleware::track_requests))
            // Order matters! The CORS layer must be the last layer in the middleware stack.
            .layer(
                CorsLayer::new()
                    .allow_origin(Self::cors_origin()?)
                    .allow_methods(Any)
                    .allow_headers(Any),
            );

        // Server configuration
        let addr = SocketAddr::from(([0; 4], env::port().unwrap_or(8080)));

        tracing::info!("Starting server on {addr}");

        // Start the server
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap_or_else(|err| panic!("Server failed to start on {addr}: {err:?}"));

        Ok(())
    }
}
