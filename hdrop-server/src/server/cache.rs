use crate::parse_and_upscale_to_mb;
use crate::Result;
use bincache::compression::Zstd;
use bincache::strategies::Limits;
use bincache::{Cache, CacheBuilder, DiskStrategy, HybridStrategy, MemoryStrategy};
use std::borrow::Cow;
use std::env;
use std::path::Path;
use uuid::Uuid;

type ZstdCache<S> = Cache<Uuid, S, Zstd>;

pub enum CacheVariant {
    Memory(ZstdCache<MemoryStrategy>),
    Disk(ZstdCache<DiskStrategy>),
    Hybrid(ZstdCache<HybridStrategy>),
}

impl CacheVariant {
    pub async fn try_from_env() -> Result<Self> {
        let cache_variant: String =
            env::var("CACHE_STRATEGY").unwrap_or_else(|_| "memory".to_string());

        let cache_dir = env::var("CACHE_DIR").unwrap_or_else(|_| "file_cache".to_string());
        let cache_dir = Path::new(&cache_dir);
        let memory_byte_limit = env::var("CACHE_MEMORY_LIMIT_MB").ok();
        let disk_byte_limit = env::var("CACHE_DISK_LIMIT_MB").ok();
        match cache_variant.to_lowercase().as_ref() {
            "memory" => {
                let x = CacheBuilder::default()
                    .with_strategy(MemoryStrategy::new(
                        parse_and_upscale_to_mb(memory_byte_limit),
                        None,
                    ))
                    .with_compression(Zstd::default())
                    .build()
                    .await
                    .unwrap();
                Ok(CacheVariant::Memory(x))
            }
            "disk" => Ok(CacheVariant::Disk(
                CacheBuilder::default()
                    .with_strategy(DiskStrategy::new(
                        cache_dir,
                        parse_and_upscale_to_mb(disk_byte_limit),
                        None,
                    ))
                    .with_compression(Zstd::default())
                    .build()
                    .await
                    .unwrap(),
            )),
            "hybrid" => {
                let entry_limit = None;
                let memory_limit =
                    Limits::new(parse_and_upscale_to_mb(memory_byte_limit), entry_limit);
                let disk_entry_limit = None;
                let disk_limit =
                    Limits::new(parse_and_upscale_to_mb(disk_byte_limit), disk_entry_limit);
                Ok(CacheVariant::Hybrid(
                    CacheBuilder::default()
                        .with_strategy(HybridStrategy::new(cache_dir, memory_limit, disk_limit))
                        .with_compression(Zstd::default())
                        .build()
                        .await
                        .unwrap(),
                ))
            }
            _ => Err(crate::error::Error::Strategy),
        }
    }

    pub async fn put(&mut self, key: Uuid, value: Vec<u8>) -> Result<()> {
        match self {
            CacheVariant::Disk(cache) => Ok(cache.put(key, value).await?),
            CacheVariant::Hybrid(cache) => Ok(cache.put(key, value).await?),
            CacheVariant::Memory(cache) => Ok(cache.put(key, value).await?),
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
}