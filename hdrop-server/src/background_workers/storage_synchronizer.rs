use axum::body::Bytes;
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

    #[instrument(skip(self))]
    pub async fn run(mut self) -> Result<()> {
        tracing::info!("storage synchronizer started");
        while let Some(provider_sync_entry) = self.rx.recv().await {
            'sync_files: {
                let data_url = match provider_sync_entry
                    .provider
                    .write()
                    .await
                    .store_file(
                        provider_sync_entry.uuid.to_string().clone(),
                        &provider_sync_entry.file_data,
                    )
                    .await
                {
                    Ok(url) => url,
                    Err(_err) => {
                        /*return Json(Response::new(ResponseData::Error(ErrorData {
                            reason: format!("{:?}", err),
                        })))*/
                        tracing::error!("File could not be stored on StorageProvider");
                        break 'sync_files;
                        //panic!("arsch"); // propagate an error
                    }
                };
                // DB Update FileUrl here
                if let Some(data_url) = data_url {
                    provider_sync_entry
                        .database
                        .update_data_url(provider_sync_entry.uuid, data_url)
                        .await? // Add retry worker for db insertion
                } /* ToDo: what happens with the file if smth fails here, there is no recovery & file never gets deleted from s3 */

                // Clear Cache
                provider_sync_entry
                    .cache
                    .write()
                    .await
                    .delete(provider_sync_entry.uuid)
                    .await?;
            }
            let _ = tokio::spawn(Self::synchronization_retry_worker(provider_sync_entry));
        }

        Ok(())
    }

    pub async fn synchronization_retry_worker(retry_sync_entry: ProviderSyncEntry) -> Result<()> {
        let mut x = 1;
        while x <= 3 {
            let data_url = match retry_sync_entry
                .provider
                .write()
                .await
                .store_file(
                    retry_sync_entry.uuid.to_string().clone(),
                    &retry_sync_entry.file_data,
                )
                .await
            {
                Ok(url) => url,
                Err(_err) => {
                    tokio::time::sleep(Duration::from_secs(60 * x)).await;
                    x += 1;
                    continue;
                }
            };
            // DB Update FileUrl here
            if let Some(data_url) = data_url {
                retry_sync_entry
                    .database
                    .update_data_url(retry_sync_entry.uuid, data_url)
                    .await?
            } // ToDo: what happens with the file if smth fails here, there is no recovery
              // Clear Cache

            retry_sync_entry
                .cache
                .write()
                .await
                .delete(retry_sync_entry.uuid)
                .await?;
            return Ok(());
        }
        panic!("GIGA FAIL");
    }
}
