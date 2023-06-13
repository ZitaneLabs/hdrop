use super::provider::Fetchtype;
use super::provider::StorageProvider;
use crate::{parse_and_upscale_to_mb, Result};
use async_trait::async_trait;
use bincache::compression::Zstd;
use bincache::Cache;
use bincache::CacheBuilder;
use bincache::DiskStrategy;
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct OnPremiseProvider {
    storage: Cache<String, DiskStrategy, Zstd>,
}

impl OnPremiseProvider {
    pub async fn try_from_env() -> Result<Self> {
        let storage_path = env::var("STORAGE_PATH").unwrap_or_else(|_| "files".to_owned());
        let storage_path = Path::new(&storage_path);
        let storage_limit = env::var("STORAGE_LIMIT").ok();

        Ok(Self {
            storage: CacheBuilder::default()
                .with_strategy(DiskStrategy::new(
                    storage_path,
                    parse_and_upscale_to_mb(storage_limit),
                    None,
                ))
                .with_compression(Zstd::default())
                .build()
                .await
                .unwrap(),
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
