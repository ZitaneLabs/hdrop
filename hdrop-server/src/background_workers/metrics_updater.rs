use crate::{
    core::{
        monitoring::{CacheMonitoring, DatabaseMonitoring, SystemMonitoring},
        names::{network, storage, system},
        LocalProvider, StorageProvider,
    },
    server::{AppState, CacheVariant},
};
use hdrop_db::Database;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

pub struct MetricsUpdater {
    storage: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    cache: CacheMonitoring,
    database: DatabaseMonitoring,
    system: SystemMonitoring,
}

impl MetricsUpdater {
    pub fn new(
        provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
        database: Arc<Database>,
        cache: Arc<RwLock<CacheVariant>>,
    ) -> Self {
        Self {
            storage: provider,
            cache: CacheMonitoring::new(cache),
            database: DatabaseMonitoring::new(database),
            system: SystemMonitoring::new(),
        }
    }

    /// Update all metrics except for the self updating ones (requests)
    pub async fn update_metrics(mut self) {
        loop {
            // Update Storage
            let provider = self.storage.read().await;
            let used_storage = provider
                .used_storage()
                .await
                .map(|s| s.ok())
                .flatten()
                .unwrap_or_else(|| 0);
            // Update number of files stored
            let stored_files = self.database.stored_files().await.ok().unwrap_or_else(|| 0);
            // Update system
            let ram_status = self.system.ram_status();
            let cpu_status = self.system.cpu_status();
            let network_status = self.system.network_status();
            // Update Cache
            let cache_status = self.cache.cache_status().await;

            // Update Gauges
            // Update storage gauge
            metrics::gauge!(storage::USED_STORAGE_B, used_storage as f64);
            metrics::gauge!(storage::DATABASE_FILE_COUNT, stored_files as f64);
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}

pub mod metrics_middleware {
    use axum::{extract::MatchedPath, http::Request, middleware::Next, response::IntoResponse};
    use std::time::Instant;

    /// Middleware which is plugged in to track everything related to requests.
    pub async fn track_requests<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
        let start = Instant::now();
        let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
            matched_path.as_str().to_owned()
        } else {
            req.uri().path().to_owned()
        };
        let method = req.method().clone();

        let response = next.run(req).await;

        let latency = start.elapsed().as_secs_f64();
        let status = response.status().as_u16().to_string();

        let labels = [
            ("method", method.to_string()),
            ("path", path),
            ("status", status),
        ];

        metrics::increment_counter!("http_requests_total", &labels);
        metrics::histogram!("http_requests_duration_seconds", latency, &labels);

        response
    }
}
