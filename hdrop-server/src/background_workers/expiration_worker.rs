use std::{sync::Arc, time::Duration};

use hdrop_db::Database;
use tokio::sync::RwLock;
use tracing::instrument;
use uuid::Uuid;

use crate::{core::StorageProvider, error::Error, server::CacheVariant, Result};

pub struct ExpirationWorker {
    provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    database: Arc<Database>,
    cache: Arc<RwLock<CacheVariant>>,
}

impl ExpirationWorker {
    pub fn new(
        provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
        database: Arc<Database>,
        cache: Arc<RwLock<CacheVariant>>,
    ) -> Self {
        Self {
            provider,
            database,
            cache,
        }
    }

    pub async fn get_expired_files(&self) -> Vec<Uuid> {
        self.database
            .get_files_to_flush()
            .await
            .unwrap_or_else(|_| Vec::with_capacity(0))
    }

    async fn delete_file_from_database(&self, uuid: Uuid) -> Result<()> {
        if let Err(err) = self.database.delete_file_by_uuid(uuid).await {
            tracing::error!("Could not delete file from database: {err}");
            Err(Error::Database(err))
        } else {
            tracing::trace!("File deleted from database: {uuid}");
            Ok(())
        }
    }

    async fn delete_file_from_provider(&self, uuid: Uuid) -> Result<()> {
        // Check if file exists in storage provider
        let result = self
            .provider
            .read()
            .await
            .file_exists(uuid.to_string())
            .await;

        // Match result
        match result {
            Ok(true) => (),
            Ok(false) => {
                tracing::warn!("File not found in StorageProvider: {uuid}");
                return Ok(());
            }
            Err(err) => {
                tracing::error!("Unable to check if file exists in StorageProvider: {err}");
                return Err(err);
            }
        }

        // Delete file from storage provider
        let result = self
            .provider
            .write()
            .await
            .delete_file(uuid.to_string())
            .await;

        // Match result
        match result {
            Ok(_) => {
                tracing::trace!("File deleted from StorageProvider: {uuid}");
                Ok(())
            }
            Err(err) => {
                tracing::error!("Could not delete file from StorageProvider: {err}");
                Err(err)
            }
        }
    }

    pub async fn delete_file(&self, file: Uuid) -> Result<()> {
        // Check if file exists in cache
        let mut cache_error: bool = false;
        if self.cache.read().await.exists(file) {
            // Actually delete file from cache
            if let Err(err) = self.cache.write().await.delete(file).await {
                tracing::error!("Could not delete file from cache: {err}");
                cache_error = true;
            } else {
                tracing::trace!("File deleted from Cache");
            }
        } else {
            tracing::debug!("File not found in cache");
        }

        // Delete file from provider
        if self.delete_file_from_provider(file).await.is_ok() {
            // Delete file from database
            if self.delete_file_from_database(file).await.is_err() {
                return Err(Error::DatabaseDeletion);
            }
        } else {
            return Err(Error::ProviderDeletion);
        }

        if cache_error {
            Err(Error::CacheDeletion)
        } else {
            Ok(())
        }
    }

    pub async fn sweep(&self) {
        // Get expired files from database
        let files = self.get_expired_files().await;

        // Log count of expired files
        if !files.is_empty() {
            tracing::trace!("Found {} files to be deleted", files.len());
        }

        // Iterate over expired files
        for file in files {
            // Error cases get ignored in background workers
            let _ = self.delete_file(file).await;
        }
    }

    #[instrument(skip(self))]
    pub async fn run(self) -> () {
        tracing::info!("Expiration worker started");

        loop {
            tracing::trace!("Deleting expired files");
            self.sweep().await;

            // Repeat every 60 seconds
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
