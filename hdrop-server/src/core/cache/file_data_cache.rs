use crate::{Result, error::Error};
use axum::body::Bytes;
use std::{
    collections::HashMap,
    env,
    fs::File,
    hash::Hash,
    io::{BufWriter, Write},
    mem::size_of_val,
    path::{Path, PathBuf},
};
use uuid::Uuid;

struct Cache {
    ram_cache: HashMap<Uuid, Vec<u8>>, // <usize, (Uuid, Vec<u8>)> / => Indexmap
    disk_cache: HashMap<Uuid, Box<Path>>, // <usize, (Uuid, Box<Path>)> / => Indexmap
    cache_disk_path: Box<Path>,
    disk_limit: DiskLimitManager,
    memory_limit: MemoryLimitManager,
    cache_strategy: CacheStrategy,
    cache_hybrid_flush_ratio: usize,
}

struct DiskLimitManager {
    cache_disk_limit_mb: usize,
    used_cache_disk_limit_mb: usize,
}

impl DiskLimitManager {
    fn try_from_env() -> Result<Self> {
        Ok(DiskLimitManager {
            cache_disk_limit_mb: env::var("CACHE_MEMORY_LIMIT_MB").unwrap().parse().unwrap(),
            used_cache_disk_limit_mb: 0,
        })
    }

    fn decrease_limit(&mut self, size: usize) -> Result<()> {
        if self.cache_disk_limit_mb > size {
        self.cache_disk_limit_mb -= size;
        self.used_cache_disk_limit_mb += size;
        } else {
            panic!("skibedi bob disk out of blobs")
        }
        Ok(())
    }

    fn increase_limit(&mut self, size: usize) -> () {
        self.cache_disk_limit_mb += size;
        self.used_cache_disk_limit_mb -= size;
        ()
    }
}

struct MemoryLimitManager {
    cache_memory_limit_mb: usize,
    used_cache_memory_limit_mb: usize,
}

impl MemoryLimitManager {
    fn try_from_env() -> Result<Self> {
        Ok(MemoryLimitManager {
            cache_memory_limit_mb: env::var("CACHE_DISK_LIMIT_MB").unwrap().parse().unwrap(),
            used_cache_memory_limit_mb: 0,
        })
    }

    fn decrease_limit(&mut self, size: usize) -> Result<()> {
        if self.cache_memory_limit_mb > size { // reduce to 90% to never overestimate?
        self.cache_memory_limit_mb -= size;
        self.used_cache_memory_limit_mb += size;
        } else {
            panic!("Skibedi bob memory out of blobs")
        }
        Ok(())
    }

    fn increase_limit(&mut self, size: usize) -> () {
        self.cache_memory_limit_mb += size;
        self.used_cache_memory_limit_mb -= size;
        ()
    }
}

enum CacheStrategy {
    Memory,
    Disk,
    Hybrid,
}

impl Cache {
    fn try_from_env() -> Result<Self> {
        Ok(Cache {
            ram_cache: HashMap::new(),
            disk_cache: HashMap::new(),
            cache_disk_path: Path::new(&env::var("CACHE_DISK_PATH").unwrap()).into(),
            disk_limit: DiskLimitManager::try_from_env().unwrap(),
            memory_limit: MemoryLimitManager::try_from_env().unwrap(),
            cache_strategy: match env::var("CACHE_STRATEGY").unwrap().as_str() {
                "memory" => CacheStrategy::Memory,
                "disk" => CacheStrategy::Disk,
                "hybrid" => CacheStrategy::Hybrid,
                _ => CacheStrategy::Memory,
            },
            cache_hybrid_flush_ratio: {
                let ratio = env::var("CACHE_HYBRID_FLUSH_RATIO")
                    .unwrap()
                    .parse()
                    .unwrap();
                if ratio <= 100 && ratio > 0 {
                    ratio
                } else {
                    panic!("wrong ratio");
                }
            },
        })
    }

    pub fn cache_data(&mut self, uuid: Uuid, file_data: Vec<u8>) -> Result<()> {
        match self.cache_strategy {
            CacheStrategy::Memory => self.set_ram_cache(uuid, file_data),
            CacheStrategy::Disk => self.set_disk_cache(uuid, file_data),
            CacheStrategy::Hybrid => self.set_hybrid_cache(uuid, file_data),
        }
    }

    pub fn set_ram_cache(&mut self, uuid: Uuid, file_data: Vec<u8>) -> Result<()> {
        let size = size_of_val(&file_data);
        self.memory_limit.decrease_limit(size);
        self.ram_cache.insert(uuid, file_data);

        Ok(())
    }

    pub fn set_disk_cache(&self, uuid: Uuid, file_data: Vec<u8>) -> Result<()> {
        let mut file_path = PathBuf::from(self.cache_disk_path.clone());
        file_path.push(uuid.to_string());
        let f = File::create(&file_path).expect("Unable to create file"); // Ok panic => propagate
        let mut f = BufWriter::new(f);
        f.write_all(self.ram_cache.get(&uuid).unwrap())
            .expect("Unable to write data"); // Ok panic => propagate
        Ok(())
    }

    pub fn set_hybrid_cache(&mut self, uuid: Uuid, file_data: Vec<u8>) -> Result<()> {
        let size = size_of_val(&file_data);
        if self.cache_memory_limit_mb > size {
            // reduce to 90% to never overestimate?
            self.cache_memory_limit_mb -= size;
            self.used_cache_memory_limit_mb += size;
            self.ram_cache.insert(uuid, file_data);
        } else {
            self.set_disk_cache(uuid, file_data)?;
            self.flush()?;
        }
        Ok(())
    }

    // Gets called on specific file after s3 is finished to delete from cache
    pub fn remove_file_from_cache(uuid: Uuid) -> Result<()> {
        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        Ok(())
        /*
        Pseudo Code Logic
        initiate temp size var
        While Loop through cache based on temp size var < 0.xx =>
        Get Elements
        Get size of elements
        Add to temp size var
        check if temp size var > 0.25, 0.5 whatever
        finish flush


         */
    }
}
