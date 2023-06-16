use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ExpiryData {
    pub expiry: i64,
}
