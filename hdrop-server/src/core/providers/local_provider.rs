use async_trait::async_trait;
use bincache::{Cache, CacheBuilder, DiskStrategy, Noop};
use hdrop_shared::{
    env,
    metrics::{names, UpdateMetrics},
};
use std::path::PathBuf;

use super::provider::{Fetchtype, StorageProvider};
use crate::{error::Error, utils::mb_to_bytes, Result};

#[derive(Debug)]
pub struct LocalProvider {
    storage: Cache<String, DiskStrategy, Noop>,
}

impl LocalProvider {
    pub async fn try_from_env() -> Result<Self> {
        let storage_path = env::local_storage_dir().unwrap_or_else(|_| PathBuf::from("files"));
        let storage_limit_mb = env::local_storage_limit_mb().ok().map(mb_to_bytes);

        let mut cache = CacheBuilder::default()
            .with_strategy(DiskStrategy::new(storage_path, storage_limit_mb, None))
            .build()
            .await?;

        cache.recover(|s| Some(s.to_string())).await?;

        Ok(Self { storage: cache })
    }
}

#[async_trait]
impl StorageProvider for LocalProvider {
    async fn store_file(&mut self, ident: String, content: &[u8]) -> Result<Option<String>> {
        self.storage.put(ident, content).await?;
        // Action-based update of metrics due to write operation
        self.update_metrics().await;
        Ok(None)
    }

    async fn delete_file(&mut self, ident: String) -> Result<()> {
        self.storage.delete(ident).await?;
        // Action-based update of metrics due to write operation
        self.update_metrics().await;
        Ok(())
    }

    async fn get_file(&self, ident: String) -> Result<Fetchtype> {
        let data = self.storage.get(ident).await?;
        Ok(Fetchtype::FileData(data.into_owned()))
    }

    async fn file_exists(&self, ident: String) -> Result<bool> {
        Ok(self.storage.exists(ident))
    }
}

#[async_trait]
impl UpdateMetrics for LocalProvider {
    async fn update_metrics(&self) {
        async fn used_storage() -> Option<Result<u64>> {
            use tokio::fs::read_dir;

            let upload_dir =
                read_dir(env::local_storage_dir().unwrap_or_else(|_| PathBuf::from("files"))).await;

            let mut upload_dir = match upload_dir {
                Ok(read_dir) => read_dir,
                Err(e) => {
                    tracing::error!("Io Error: {}", e);
                    return Some(Err(Error::Io(e)));
                }
            };

            {
                let mut used_storage = 0;

                while let Some(dir_entry) = match upload_dir.next_entry().await {
                    Ok(dir_entry) => dir_entry,
                    Err(e) => {
                        tracing::error!("Io Error: {}", e);
                        return Some(Err(Error::Io(e)));
                    }
                } {
                    let file = match dir_entry.metadata().await {
                        Ok(metadata) => metadata,
                        Err(e) => {
                            tracing::error!("Io Error: {}", e);
                            return Some(Err(Error::Io(e)));
                        }
                    };
                    let file_size = if file.is_file() {
                        file.len()
                    } else {
                        tracing::warn!("Subfolder found in upload directory. You should check this manually. Possible recovery folder from bincache or someone changed the upload directory");
                        0
                    };

                    used_storage += file_size;
                }

                Some(Ok(used_storage))
            }
        }

        let used_storage = used_storage().await.and_then(|s| s.ok()).unwrap_or(0);

        // Update storage gauge
        metrics::gauge!(names::storage::USED_STORAGE_B, used_storage as f64);
    }
}
