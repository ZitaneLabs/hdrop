use axum::body::Bytes;
use futures::TryFutureExt;
use hdrop_db::Database;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc::UnboundedReceiver, RwLock};
use tracing::instrument;
use uuid::Uuid;

use crate::{core::StorageProvider, error::Result, server::CacheVariant};
use bincache::{Cache, HybridStrategy};

pub struct ProviderSyncEntry {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub file_data: Bytes,
    pub uuid: Uuid,
    pub cache: Arc<RwLock<CacheVariant>>,
}

impl ProviderSyncEntry {
    pub async fn synchronization_retry_worker(retry_sync_entry: ProviderSyncEntry) -> Result<()> {
        let mut x = 1;
        while x <= 3 {
            let data_url = match retry_sync_entry
                .provider
                .read()
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
            retry_sync_entry
                .database
                .update_file_url(retry_sync_entry.uuid, data_url)
                .await?; // ToDo: what happens with the file if smth fails here, there is no recovery
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

    #[instrument]
    pub async fn storage_synchronizer(mut rx: UnboundedReceiver<ProviderSyncEntry>) -> Result<()> {
        tracing::info!("storage synchronizer started");
        while let Some(provider_sync_entry) = rx.recv().await {
            'sync_files: {
                let data_url = match provider_sync_entry
                    .provider
                    .read()
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
                        break 'sync_files;
                        //panic!("arsch"); // propagate an error
                    }
                };
                // DB Update FileUrl here
                provider_sync_entry
                    .database
                    .update_file_url(provider_sync_entry.uuid, data_url)
                    .await?; /* ToDo: what happens with the file if smth fails here, there is no recovery & file never gets deleted from s3 */
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
}

pub struct ExpirationWorkerEntry {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub cache: Arc<RwLock<CacheVariant>>,
}
impl ExpirationWorkerEntry {
    #[instrument(skip_all)]
    pub async fn expiration_worker(expiration_worker_entry: ExpirationWorkerEntry) -> () {
        tracing::info!("expiration worker started");
        async fn delete_file_by_uuid(expiration_worker_entry: &ExpirationWorkerEntry, uuid: Uuid) {
            match expiration_worker_entry
                .database
                .delete_file_by_uuid(uuid)
                .await
            {
                Ok(_) => tracing::debug!("File {uuid} deleted from db"),
                Err(err) => tracing::error!("{err}"),
            }
        }

        loop {
            'delete_files: {
                let Ok(files_to_delete) = expiration_worker_entry.database.flush().await else {
                    break 'delete_files
                };

                tracing::debug!("{:?}", files_to_delete);
                for i in files_to_delete {
                    match expiration_worker_entry
                        .provider
                        .read()
                        .await
                        .file_exists(i.to_string())
                        .await
                    {
                        // We don't care if the file exists or not, there is a db entry which must be deleted in both cases
                        Ok(case) => {
                            if !case {
                                // Log the uncommon case, that the file is actually not there for whatever reason to inspect the infra
                                tracing::warn!("File deleted/non-existent although it's in the db and got scheduled");
                                delete_file_by_uuid(&expiration_worker_entry, i).await;
                                // Skip S3 deletion
                                continue;
                            };
                            if let Err(err) = expiration_worker_entry
                                .provider
                                .read()
                                .await
                                .delete_file(i.to_string())
                                .await
                            {
                                // S3 Could not delete, so it will be scheduled again for the next flush as the db is not cleared
                                tracing::error!(
                                    "File could not get deleted from StorageProvider: {err}"
                                );
                            } else {
                                delete_file_by_uuid(&expiration_worker_entry, i).await;
                            }
                        }
                        // If there is an error, we can't delete the db entry and it will be scheduled for the next flush to retry again
                        Err(err) => tracing::error!("Check if file exists failed: {err}"),
                    }

                    /*
                    ToDo: What should happen if the db entry can't get deleted?
                    */
                    /*
                    ToDo: What happens with the file if file deletion fails: it's still in the db and tries to delete in next schedule again
                    */
                }
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
