use std::sync::Arc;

use hdrop_db::Database;
use hdrop_shared::env;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

use super::cache::CacheVariant;
use crate::{
    background_workers::storage_synchronizer::ProviderSyncEntry,
    core::{LocalProvider, S3Provider, StorageProvider},
    error::Error,
    Result,
};

pub struct AppState {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub cache: Arc<RwLock<CacheVariant>>,
    provider_sync_tx: UnboundedSender<ProviderSyncEntry>,
}

impl AppState {
    pub async fn new(provider_sync_tx: UnboundedSender<ProviderSyncEntry>) -> Result<Self> {
        let provider: Box<dyn StorageProvider + Sync + Send> = match env::storage_provider()
            .map(|s| s.to_lowercase())
        {
            Ok(ref provider) if provider == "s3" => {
                Ok(Box::new(S3Provider::try_from_env()?) as Box<dyn StorageProvider + Sync + Send>)
            }
            Ok(ref provider) if provider == "local" => {
                Ok(Box::new(LocalProvider::try_from_env().await?)
                    as Box<dyn StorageProvider + Sync + Send>)
            }
            Ok(provider) => Err(Error::InvalidProvider(provider)),
            Err(_) => Err(Error::NoProvider),
        }?;

        let database = Arc::new(Database::try_from_env()?);
        let cache = Arc::new(RwLock::new(CacheVariant::try_from_env().await?));

        Ok(AppState {
            provider: Arc::new(RwLock::new(provider)),
            database,
            provider_sync_tx,
            cache,
        })
    }

    pub fn get_provider_sync_tx(&self) -> UnboundedSender<ProviderSyncEntry> {
        self.provider_sync_tx.clone()
    }
}
