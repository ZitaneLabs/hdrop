use async_trait::async_trait;

/// Trait to define an update metrics function
#[async_trait]
pub trait UpdateMetrics {
    async fn update_metrics(&self) {}
}

pub mod names {
    pub const GAUGE_NAMES: [&str; 4] = [
        storage::USED_STORAGE_B,
        storage::CACHE_TOTAL_CAPACITY_B,
        storage::CACHE_USED_CAPACITY_B,
        storage::DATABASE_FILE_COUNT,
    ];

    pub const HISTOGRAM_NAMES: [&str; 1] = [
        network::HTTP_REQUESTS_DURATION_SECONDS,
    ];

    pub const COUNTER_NAMES: [&str; 1] = [
        network::HTTP_REQUESTS_TOTAL,
    ];

    /// Server requests, latency
    /// Network interface
    pub mod network {
        pub const HTTP_REQUESTS_TOTAL: &str = "http_requests_total";
        pub const HTTP_REQUESTS_DURATION_SECONDS: &str = "http_requests_duration_seconds";
        pub const NETWORK_OCTETS_RECEIVED: &str = "network_octets_received";
        pub const NETWORK_OCTETS_TRANSMITTED: &str = "network_octets_transmitted";
    }

    /// Cache, Storage, Database file amount
    pub mod storage {
        pub const USED_STORAGE_B: &str = "used_storage_bytes";
        pub const CACHE_TOTAL_CAPACITY_B: &str = "cache_total_capacity_bytes";
        pub const CACHE_USED_CAPACITY_B: &str = "cache_used_capacity_bytes";
        pub const DATABASE_FILE_COUNT: &str = "database_file_count";
    }

    pub mod system {
        pub const AVG_CPU_USAGE: &str = "avg_cpu_usage";
        pub const RAM_USAGE_B: &str = "ram_usage_bytes";
    }
}
