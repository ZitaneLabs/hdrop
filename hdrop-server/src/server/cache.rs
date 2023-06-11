use crate::Result;
use bincache::compression::Noop;
use bincache::strategies::Limits;
use bincache::{Cache, CacheBuilder, DiskStrategy, HybridStrategy, MemoryStrategy};
use std::borrow::Cow;
use std::env;
use std::path::Path;
use uuid::Uuid;

type ZstdCache<S> = Cache<Uuid, S, Noop>;

pub enum CacheVariant {
    Memory(ZstdCache<MemoryStrategy>),
    Disk(ZstdCache<DiskStrategy>),
    Hybrid(ZstdCache<HybridStrategy>),
}

impl TryFrom<String> for CacheVariant {
    type Error = crate::error::Error;

    fn try_from(value: String) -> Result<Self> {
        let cache_dir = env::var("CACHE_DIR").unwrap_or_else(|_| "cache".to_string());
        let cache_dir = Path::new(&cache_dir);

        match value.as_ref() {
            "Memory" => {
                let byte_limit = env::var("MEMORY_BYTES_LIMIT")?.parse::<usize>().ok();
                let entry_limit = env::var("MEMORY_ENTRY_LIMIT")?.parse::<usize>().ok();
                let x = CacheBuilder::default()
                    .with_strategy(MemoryStrategy::new(byte_limit, entry_limit))
                    .build()
                    .unwrap();
                return Ok(CacheVariant::Memory(x));
            }
            "Disk" => {
                let disk_byte_limit = env::var("DISK_BYTE_LIMIT")?.parse::<usize>().ok();
                let disk_entry_limit = env::var("DISK_ENTRY_LIMIT")?.parse::<usize>().ok();
                return Ok(CacheVariant::Disk(
                    CacheBuilder::default()
                        .with_strategy(DiskStrategy::new(
                            cache_dir,
                            disk_byte_limit,
                            disk_entry_limit,
                        ))
                        .build()
                        .unwrap(),
                ));
            }
            "Hybrid" => {
                let byte_limit = env::var("MEMORY_BYTES_LIMIT")?.parse::<usize>().ok();
                let entry_limit = env::var("MEMORY_ENTRY_LIMIT")?.parse::<usize>().ok();
                let memory_limit = Limits::new(byte_limit, entry_limit);

                let disk_byte_limit = env::var("DISK_BYTE_LIMIT")?.parse::<usize>().ok();
                let disk_entry_limit = env::var("DISK_ENTRY_LIMIT")?.parse::<usize>().ok();
                let disk_limit = Limits::new(disk_byte_limit, disk_entry_limit);
                return Ok(CacheVariant::Hybrid(
                    CacheBuilder::default()
                        .with_strategy(HybridStrategy::new(cache_dir, memory_limit, disk_limit))
                        .build()
                        .unwrap(),
                ));
            }
            _ => return Err(crate::error::Error::Strategy),
        };
    }
}

impl CacheVariant {
    pub fn try_from_env() -> Result<Self> {
        // env vars parsen und die richtige Variante konstruieren
        let cache_variant: Result<CacheVariant> = env::var("CACHE_VARIANT")
            .unwrap_or_else(|_| "Memory".to_string())
            .try_into();
        cache_variant
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

    pub async fn flush(&mut self) -> Result<usize> {
        match self {
            CacheVariant::Hybrid(cache) => Ok(cache.flush().await?),
            _ => Err(crate::error::Error::Flush),
        }
    }

    pub fn key_from_str(key: &str) -> Option<Uuid> {
        Uuid::parse_str(key).ok()
    }

    pub async fn recover(&mut self) -> Result<usize> {
        match self {
            CacheVariant::Hybrid(cache) => Ok(cache.recover(Self::key_from_str).await?),
            _ => Err(crate::error::Error::Recover),
        }
    }
}
