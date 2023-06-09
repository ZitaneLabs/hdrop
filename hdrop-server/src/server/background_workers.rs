use axum::body::Bytes;
use futures::TryFutureExt;
use hdrop_db::Database;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc::UnboundedReceiver, RwLock};
use uuid::Uuid;

use crate::{core::StorageProvider, error::Result};
use bincache::{Cache, HybridStrategy};

pub struct ProviderSyncEntry {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub file_data: Bytes,
    pub uuid: Uuid,
    pub cache: Arc<RwLock<Cache<Uuid, HybridStrategy>>>,
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
                .delete(retry_sync_entry.uuid).await?;
            return Ok(());
        }
        panic!("GIGA FAIL");
    }

    pub async fn storage_synchronizer(mut rx: UnboundedReceiver<ProviderSyncEntry>) -> Result<()> {
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
                    .delete(provider_sync_entry.uuid).await?;
            }
            let _ = tokio::spawn(Self::synchronization_retry_worker(provider_sync_entry));
        }

        Ok(())
    }
}

pub struct ExpirationWorkerEntry {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub cache: Arc<RwLock<Cache<Uuid, HybridStrategy>>>,
}
impl ExpirationWorkerEntry {
    pub async fn expiration_worker(expiration_worker_entry: ExpirationWorkerEntry) -> ! {
        loop {
            'delete_files: {
                println!("expiration worker started"); // ToDo
                                                       // Logic
                let Ok(files_to_delete) = expiration_worker_entry.database.flush().await else {
                break 'delete_files
            };

                println!("{:?}", files_to_delete);
                for i in files_to_delete {
                    expiration_worker_entry
                        .provider
                        .read()
                        .await
                        .delete_file(i.to_string())
                        .await
                        .unwrap(); /*
                                   ToDo: what happens with the file if smth fails here, it's still on s3? => but gets deleted in next run
                                   prob doesnt matter as db gets cleared after s3 is successfully deleted
                                   */
                    expiration_worker_entry
                        .database
                        .delete_file_by_uuid(i)
                        .await
                        .unwrap(); /*
                                   ToDo: what happens with the file if smth fails here, it's still in the db and maybe tries to delete on s3 again (which will fail) => infinite cycle?
                                   */
                }
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
