use bincache::{
    strategies::Limits, Cache, CacheBuilder, CacheCapacity, DiskStrategy, HybridStrategy,
    MemoryStrategy, Noop,
};
use hdrop_shared::env;
use std::{borrow::Cow, path::PathBuf};
use uuid::Uuid;

use crate::{utils::mb_to_bytes, Result};

type FileCache<S> = Cache<Uuid, S, Noop>;

pub enum CacheVariant {
    Memory(FileCache<MemoryStrategy>),
    Disk(FileCache<DiskStrategy>),
    Hybrid(FileCache<HybridStrategy>),
}
// ToDo: Add more Error Tracing
impl CacheVariant {
    pub async fn try_from_env() -> Result<Self> {
        let cache_variant: String = env::cache_strategy().unwrap_or_else(|_| "memory".to_string());
        let cache_dir = env::cache_dir().unwrap_or_else(|_| PathBuf::from("file_cache"));
        let memory_byte_limit = env::cache_memory_limit_mb().map(mb_to_bytes).ok();
        let disk_byte_limit = env::cache_disk_limit_mb().map(mb_to_bytes).ok();

        match cache_variant.to_lowercase().as_ref() {
            "memory" => Ok(CacheVariant::Memory(
                CacheBuilder::default()
                    .with_strategy(MemoryStrategy::new(memory_byte_limit, None))
                    .build()
                    .await?,
            )),
            "disk" => Ok(CacheVariant::Disk(
                CacheBuilder::default()
                    .with_strategy(DiskStrategy::new(cache_dir, disk_byte_limit, None))
                    .build()
                    .await?,
            )),
            "hybrid" => {
                let memory_limits = Limits::new(memory_byte_limit, None);
                let disk_limits = Limits::new(disk_byte_limit, None);
                Ok(CacheVariant::Hybrid(
                    CacheBuilder::default()
                        .with_strategy(HybridStrategy::new(cache_dir, memory_limits, disk_limits))
                        .build()
                        .await?,
                ))
            }
            _ => Err(crate::error::Error::Strategy),
        }
    }

    pub async fn put(&mut self, key: Uuid, value: Vec<u8>) -> Result<()> {
        let result = match self {
            CacheVariant::Disk(cache) => cache.put(key, value).await,
            CacheVariant::Hybrid(cache) => cache.put(key, value).await,
            CacheVariant::Memory(cache) => cache.put(key, value).await,
        };
        {
            match &result {
                Err(bincache::Error::LimitExceeded { limit_kind }) => {
                    tracing::error!("Cache limit exceeded: {limit_kind}");

                    Ok(result?)
                }
                _ => Ok(result?),
            }
        }
    }

    pub async fn get(&self, key: Uuid) -> Result<Cow<'_, [u8]>> {
        match self {
            CacheVariant::Disk(cache) => Ok(cache.get(key).await?),
            CacheVariant::Hybrid(cache) => Ok(cache.get(key).await?),
            CacheVariant::Memory(cache) => Ok(cache.get(key).await?),
        }
    }

    pub async fn delete(&mut self, key: Uuid) -> Result<()> {
        match self {
            CacheVariant::Disk(cache) => Ok(cache.delete(key).await?),
            CacheVariant::Hybrid(cache) => Ok(cache.delete(key).await?),
            CacheVariant::Memory(cache) => Ok(cache.delete(key).await?),
        }
    }

    pub fn key_from_str(key: &str) -> Option<Uuid> {
        Uuid::parse_str(key).ok()
    }

    pub async fn recover(&mut self) -> Result<usize> {
        match self {
            CacheVariant::Hybrid(cache) => Ok(cache.recover(Self::key_from_str).await?),
            CacheVariant::Disk(_) => Err(crate::error::Error::NoRecover),
            CacheVariant::Memory(_) => Err(crate::error::Error::NoRecover),
        }
    }

    pub fn exists(&self, key: Uuid) -> bool {
        match self {
            CacheVariant::Disk(cache) => cache.exists(key),
            CacheVariant::Hybrid(cache) => cache.exists(key),
            CacheVariant::Memory(cache) => cache.exists(key),
        }
    }

    pub fn capacity(&self) -> Option<CacheCapacity> {
        match self {
            CacheVariant::Disk(cache) => cache.capacity(),
            CacheVariant::Hybrid(cache) => cache.capacity(),
            CacheVariant::Memory(cache) => cache.capacity(),
        }
    }
}
