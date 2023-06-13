pub fn parse_and_upscale_to_mb(bytes: Option<String>) -> Option<usize> {
    match bytes {
        Some(bytes) => bytes.parse::<usize>().ok().map(|bytes| bytes * 1_000_000),
        None => None,
    }
}
