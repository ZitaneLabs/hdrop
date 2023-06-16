use hdrop_db::Database;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

use crate::{
    background_workers::storage_synchronizer::ProviderSyncEntry,
    core::{S3Provider, StorageProvider},
    Result,
};

use super::CacheVariant;

pub struct AppState {
    pub provider: Arc<RwLock<Box<dyn StorageProvider + Sync + Send>>>,
    pub database: Arc<Database>,
    pub cache: Arc<RwLock<CacheVariant>>,
    pub provider_sync_tx: UnboundedSender<ProviderSyncEntry>,
}

impl AppState {
    pub async fn new(provider_sync_tx: UnboundedSender<ProviderSyncEntry>) -> Result<Self> {
        let provider: Arc<RwLock<Box<dyn StorageProvider + Send + Sync>>> =
            Arc::new(RwLock::new(Box::new(S3Provider::try_from_env()?)));
        let database = Arc::new(Database::try_from_env()?);
        let cache = Arc::new(RwLock::new(CacheVariant::try_from_env().await?));

        Ok(AppState {
            provider,
            database,
            provider_sync_tx,
            cache,
        })
    }
}
