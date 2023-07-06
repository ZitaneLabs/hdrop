use crate::Result;
use async_trait::async_trait;
use sysinfo::{CpuExt, NetworkExt, System, SystemExt};

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

    /// Get the capacity utilization as a value between 0 and 1.
    pub fn utilization(&self) -> f64 {
        self.used_amount as f64 / self.total_amount as f64
    }

    /// Get the capacity utilization as a value between 0% and 100%.
    pub fn utilization_percentage(&self) -> f64 {
        self.utilization() * 100.00
    }
}

/// Trait defining functions to monitor storage related metrics.
#[async_trait]
pub trait StorageMonitoring {
    /// Get the occupied storage space in bytes. Returns None if not implemented/available.
    async fn used_storage(&self) -> Option<Result<u64>> {
        None
    }
}

/// Struct for network interfaces with outgoing and incoming data (octets).
pub struct NetworkStatus {
    interface: String,
    data_received: usize,
    data_transmitted: usize,
}

impl NetworkStatus {
    pub fn interface(&self) -> String {
        self.interface.clone()
    }

    pub fn received(&self) -> usize {
        self.data_received
    }

    pub fn transmitted(&self) -> usize {
        self.data_transmitted
    }
}

/// Struct to persist system monitoring.
pub struct SystemMonitoring {
    sys: System,
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
    pub fn cpu_status(&mut self) -> Vec<Status> {
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
    pub fn network_status(&mut self) -> Vec<NetworkStatus> {
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
