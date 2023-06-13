use hdrop_db::Database;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tracing::instrument;
use uuid::Uuid;

use crate::{core::StorageProvider, server::CacheVariant};

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

    #[instrument(skip(self))]
    pub async fn run(self) -> () {
        tracing::info!("expiration worker started");

        loop {
            'delete_files: {
                let Ok(files_to_delete) = self.database.flush().await else {
                    break 'delete_files
                };

                tracing::debug!("Found files to delete: {:?}", files_to_delete);
                for i in files_to_delete {
                    match self.provider.read().await.file_exists(i.to_string()).await {
                        Ok(case) => {
                            // Check if file does not exist on StorageProvider
                            if !case {
                                // File does not exist on StorageProvider, checks for Cache
                                tracing::info!("File does not exist on StorageProvider");
                                if self.cache.read().await.exists(i) {
                                    match self.cache.write().await.delete(i).await {
                                        Ok(_) => (),
                                        Err(err) => tracing::error!(
                                            "Could not delete file from cache {err}"
                                        ),
                                    }
                                } else {
                                    // Log the uncommon case, that the file is actually not there for whatever reason to inspect the infra
                                    tracing::error!(
                                        "File does neither exist on StorageProvider nor in Cache"
                                    );
                                }
                                // We don't care if the file exists or not, there is a db entry which must be deleted in both cases
                                self.delete_file_by_uuid(i).await;
                                // Skip S3 deletion
                                continue;
                            } else {
                                // File exists on StorageProvider, delete it
                                if let Err(err) =
                                    self.provider.write().await.delete_file(i.to_string()).await
                                {
                                    // StorageProvider Could not delete, so it will be scheduled again for the next flush as the db is not cleared
                                    tracing::error!(
                                        "File could not get deleted from StorageProvider: {err}"
                                    );
                                } else {
                                    // Delete db entry
                                    self.delete_file_by_uuid(i).await;
                                }
                            }
                        }
                        // If there is a StorageProvider error, we must not delete the db entry and it will be scheduled for the next flush to retry again
                        Err(err) => tracing::error!("Check if file exists failed: {err}"),
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    async fn delete_file_by_uuid(&self, uuid: Uuid) {
        match self.database.delete_file_by_uuid(uuid).await {
            Ok(_) => tracing::debug!("File {uuid} deleted from db"),
            Err(err) => tracing::error!("{err}"),
        }
    }
}
