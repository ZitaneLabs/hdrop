use async_trait::async_trait;
use bincache::{Cache, CacheBuilder, DiskStrategy, Noop};
use hdrop_shared::env;
use std::path::PathBuf;

use super::provider::{Fetchtype, StorageProvider};
use crate::{utils::mb_to_bytes, Result};

#[derive(Debug)]
pub struct OnPremiseProvider {
    storage: Cache<String, DiskStrategy, Noop>,
}

impl OnPremiseProvider {
    pub async fn try_from_env() -> Result<Self> {
        let storage_path = env::local_storage_dir().unwrap_or_else(|_| PathBuf::from("files"));
        let storage_limit_mb = env::local_storage_limit_mb().ok().map(mb_to_bytes);

        Ok(Self {
            storage: CacheBuilder::default()
                .with_strategy(DiskStrategy::new(storage_path, storage_limit_mb, None))
                .build()
                .await?,
        })
    }
}

#[async_trait]
impl StorageProvider for OnPremiseProvider {
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
