// Convert megabytes to bytes
pub fn mb_to_bytes(bytes: usize) -> usize {
    bytes * 1_000_000
}

/// Convert bytes to megabytes. Uses natural division
pub fn bytes_to_mb(bytes: usize) -> usize {
    bytes / 1_000_000
}
