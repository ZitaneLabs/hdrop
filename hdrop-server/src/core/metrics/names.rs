pub const GAUGE_NAMES: [&str; 4] = [
    network::HTTP_REQUESTS_TOTAL,
    network::HTTP_REQUESTS_DURATION_SECONDS,
    storage::USED_STORAGE_B,
    storage::DATABASE_FILE_COUNT,
];

/// Server requests, latency
/// Network interface
pub mod network {
    pub const HTTP_REQUESTS_TOTAL: &str = "http_requests_total";
    pub const HTTP_REQUESTS_DURATION_SECONDS: &str = "http_requests_duration_seconds";
    pub const NETWORK_OCTETS_IN_OUT: &str = "network_octets_in_out";
}

/// Cache, Storage, Database file amount
pub mod storage {
    pub const USED_STORAGE_B: &str = "used_storage_bytes";
    pub const CACHE_TOTAL_CAPACITY_B: &str = "cache_total_capacity_bytes";
    pub const CACHE_USED_CAPACITY_B: &str = "cache_used_capacity_bytes";
    pub const DATABASE_FILE_COUNT: &str = "database_file_count";
}

pub mod system {
    pub const CPU_USAGE: &str = "cpu_usage";
    pub const RAM_USAGE: &str = "ram_usage";
}
