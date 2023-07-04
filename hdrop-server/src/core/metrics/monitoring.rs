use crate::{server::CacheVariant, Result};
use async_trait::async_trait;
use hdrop_db::Database;
use std::sync::Arc;
use sysinfo::{CpuExt, NetworkExt, RefreshKind, System, SystemExt};
use tokio::sync::RwLock;

/// Agnostic status struct for any limited resource.
pub struct Status {
    total_amount: usize,
    used_amount: usize,
}

impl Status {
    pub fn new(total_amount: usize, used_amount: usize) -> Self {
        Self {
            total_amount,
            used_amount,
        }
    }

    /// Get the total capacity.
    pub fn total(&self) -> usize {
        self.total_amount
    }

    /// Get the used capacity.
    pub fn used(&self) -> usize {
        self.used_amount
    }
}

#[async_trait]
pub trait StorageMonitoring {
    /// Get the occupied storage space in bytes.
    async fn used_storage(&self) -> Result<u64>;
}

pub struct CacheMonitoring {
    pub cache: Arc<RwLock<CacheVariant>>,
}

impl CacheMonitoring {
    /// Get the status of the Cache.
    pub async fn cache_status(&self) -> Option<Status> {
        let status = self.cache.read().await.capacity();
        status.map(|capacity| Status::new(capacity.total(), capacity.used()))
    }
}

pub struct DatabaseMonitoring {
    pub database: Arc<Database>,
}

impl DatabaseMonitoring {
    /// Determine the number of files currently stored according to the database.
    pub async fn stored_files(&self) -> Result<usize> {
        Ok(self.database.get_file_amount().await? as usize)
    }
}

/// Struct for network interfaces with outgoing and incoming data (octets).
pub struct NetworkStatus {
    pub interface: String,
    pub data_received: usize,
    pub data_transmitted: usize,
}

/// Struct to persist system monitoring.
pub struct SystemMonitoring {
    pub sys: System,
}

impl SystemMonitoring {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    /// Get the status of the server RAM
    pub fn ram_status(&mut self) -> Status {
        self.sys.refresh_memory();
        Status {
            total_amount: self.sys.total_memory() as usize,
            used_amount: self.sys.used_memory() as usize,
        }
    }

    /// Get the usage of the server CPU
    pub async fn cpu_status(&mut self) -> Vec<Status> {
        self.sys.refresh_cpu(); // refresh_cpu_specifics(refresh_kind)
        let mut cpus = vec![];
        for cpu in self.sys.cpus() {
            cpus.push(Status {
                total_amount: 100,
                used_amount: cpu.cpu_usage() as usize,
            });
        }

        cpus
    }

    /// Get current network status
    /// Network octets in/out
    pub async fn network_status(&mut self) -> Vec<NetworkStatus> {
        self.sys.refresh_all();
        let mut networks = vec![];
        for (interface_name, data) in self.sys.networks() {
            networks.push(NetworkStatus {
                interface: interface_name.to_owned(),
                data_received: data.received() as usize,
                data_transmitted: data.transmitted() as usize,
            });
        }

        networks
    }

    /// Simple healthcheck on StorageProvider
    pub async fn storage_healthcheck() -> Result<bool> {
        todo!()
    }

    /// Simple healthcheck on Cache
    pub async fn cache_healthcheck() -> Result<bool> {
        todo!()
    }

    /// Simple healtcheck on Database
    pub async fn database_healthcheck() -> Result<bool> {
        todo!()
    }

    /// Simple healthcheck on server integrity
    pub async fn check_server_integrity() -> Result<bool> {
        todo!()
    }
}
