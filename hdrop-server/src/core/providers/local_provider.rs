use async_trait::async_trait;
use bincache::{Cache, CacheBuilder, DiskStrategy, Noop};
use hdrop_shared::env;
use std::path::PathBuf;

use super::provider::{Fetchtype, StorageProvider};
use crate::{
    core::{Status, StorageMonitoring},
    utils::mb_to_bytes,
    Result,
};

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
        Ok(None)
    }

    async fn delete_file(&mut self, ident: String) -> Result<()> {
        self.storage.delete(ident).await?;
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
impl StorageMonitoring for LocalProvider {
    async fn used_storage(&self) -> Result<u64> {
        use tokio::fs::read_dir;

        let mut upload_dir =
            read_dir(env::local_storage_dir().unwrap_or_else(|_| PathBuf::from("files"))).await?;

        let mut acc = 0;

        while let Some(dir_entry) = upload_dir.next_entry().await? {
            let file = dir_entry.metadata().await?;
            let file_size = if file.is_file() {
                file.len()
            } else {
                tracing::warn!("Subfolder found in upload directory. You should check this manually. Possible recovery folder from bincache or someone changed the upload directory");
                0
            };

            acc += file_size;
        }

        Ok(acc)
    }
}
