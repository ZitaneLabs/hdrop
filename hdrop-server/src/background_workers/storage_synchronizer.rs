use axum::body::Bytes;
use chrono::Utc;
use hdrop_db::Database;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc::UnboundedReceiver, RwLock};
use tracing::instrument;
use uuid::Uuid;

use crate::{core::StorageProvider, error::Result, server::CacheVariant};

pub struct ProviderSyncEntry {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub file_data: Bytes,
    pub uuid: Uuid,
    pub cache: Arc<RwLock<CacheVariant>>,
}

pub struct StorageSynchronizer {
    pub rx: UnboundedReceiver<ProviderSyncEntry>,
}

impl StorageSynchronizer {
    pub fn new(rx: UnboundedReceiver<ProviderSyncEntry>) -> Self {
        Self { rx }
    }

    pub async fn synchronize(provider_sync_entry: ProviderSyncEntry) {
        // Check if file got successfully stored on StorageProvider
        match Self::save_to_provider(&provider_sync_entry).await {
            Ok(url) => {
                tracing::trace!("File stored to {url:?}");
                Self::update_db_and_cache(
                    url,
                    provider_sync_entry.database,
                    provider_sync_entry.uuid,
                    provider_sync_entry.cache,
                )
                .await;
                return;
            }
            Err(_err) => {
                tracing::error!("File could not be stored on StorageProvider");
            }
        };

        // Start background task to retry synchronization
        tokio::spawn(Self::synchronization_retry_worker(provider_sync_entry));
    }

    pub async fn save_to_provider(
        provider_sync_entry: &ProviderSyncEntry,
    ) -> Result<Option<String>> {
        // Store file on StorageProvider
        provider_sync_entry
            .provider
            .write()
            .await
            .store_file(
                provider_sync_entry.uuid.to_string(),
                &provider_sync_entry.file_data,
            )
            .await
    }

    pub async fn update_db_and_cache(
        data_url: Option<String>,
        database: Arc<Database>,
        uuid: Uuid,
        cache: Arc<RwLock<CacheVariant>>,
    ) {
        // Database update DataUrl here
        if let Err(err) = database.update_data_url(uuid, data_url.as_ref()).await {
            tracing::error!("Database data url update failed: {err}");
            tokio::spawn(Self::database_retry_worker(data_url, database, uuid));
        } else {
            tracing::trace!("File dataUrl successfully updated in database");
            //clear_cache()
            Self::clear_cache(cache, uuid).await;
        }
    }

    pub async fn clear_cache(cache: Arc<RwLock<CacheVariant>>, file_uuid: Uuid) {
        if let Err(err) = Self::check_and_delete_cache_entry(cache.clone(), file_uuid).await {
            tracing::error!("Could not delete file from cache: {err}");
            tokio::spawn(Self::cache_retry_worker(cache.clone(), file_uuid));
        }
    }

    pub async fn check_and_delete_cache_entry(
        cache: Arc<RwLock<CacheVariant>>,
        file_uuid: Uuid,
    ) -> Result<()> {
        if cache.read().await.exists(file_uuid) {
            if let Err(err) = cache.write().await.delete(file_uuid).await {
                return Err(err);
            } else {
                tracing::trace!("File deleted from Cache");
            }
        } else {
            tracing::debug!("File not found in cache");
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn run(mut self) -> Result<()> {
        tracing::info!("Storage synchronizer started");

        // Synchronize every received file to storage
        while let Some(provider_sync_entry) = self.rx.recv().await {
            Self::synchronize(provider_sync_entry).await;
        }

        tracing::warn!("Storage synchronizer stopped");
        Ok(())
    }

    /*
        Retry workers that spawn in error scenarios
    */

    pub async fn synchronization_retry_worker(retry_sync_entry: ProviderSyncEntry) {
        tracing::warn!("Storage retry worker started for one file");
        for i in 0..=6u32 {
            tracing::info!("[Attempt {i}] Retrying to sync {} (attempt {i})", retry_sync_entry.uuid);
            // Check if file is already expired
            match retry_sync_entry
                .database
                .get_file_by_uuid(retry_sync_entry.uuid)
                .await
            {
                Ok(file) => {
                    if Utc::now() > file.expiresAt {
                        tracing::trace!(
                            "File already expired, stopping storage synchronization retry worker"
                        );
                        return;
                    }
                }

                Err(err) => tracing::warn!("{err}"),
            }

            match Self::save_to_provider(&retry_sync_entry).await {
                Ok(url) => {
                    tracing::trace!("File stored to {url:?}");
                    Self::update_db_and_cache(
                        url,
                        retry_sync_entry.database,
                        retry_sync_entry.uuid,
                        retry_sync_entry.cache,
                    )
                    .await;
                    return;
                }

                Err(_err) => {
                    tracing::error!(
                        "Retry worker failed, file could not be stored on StorageProvider"
                    );
                    Self::exponential_backoff(i).await;
                }
            };
        }
    }

    pub async fn database_retry_worker(
        data_url: Option<String>,
        database: Arc<Database>,
        uuid: Uuid,
    ) {
        for i in 0..=6u32 {
            if let Err(err) = database.update_data_url(uuid, data_url.as_ref()).await {
                tracing::error!("Retry worker failed, database dataUrl update failed: {err}");
                Self::exponential_backoff(i).await;
            } else {
                tracing::trace!("File dataUrl successfully updated in database");
                return;
            }
        }
    }

    pub async fn cache_retry_worker(cache: Arc<RwLock<CacheVariant>>, file_uuid: Uuid) {
        for i in 0..=6u32 {
            if let Err(err) = Self::check_and_delete_cache_entry(cache.clone(), file_uuid).await {
                tracing::error!("Retry worker failed, could not delete file from cache: {err}");
                Self::exponential_backoff(i).await;
            } else {
                tracing::trace!("Cache retry worker successful. Closing Thread.");
                return;
            }
        }
    }

    // Simple exponential backoff
    pub async fn exponential_backoff(exponent: u32) {
        let exponential_base: u64 = 2;
        let timer = exponential_base.pow(exponent).min(60) * 60;
        tokio::time::sleep(Duration::from_secs(timer)).await;
    }
}
