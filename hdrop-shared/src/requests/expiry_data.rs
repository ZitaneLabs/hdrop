use serde::Deserialize;

/* API body equivalent structs */
#[derive(Debug, Deserialize, Clone)]
pub struct ExpiryData {
    pub expiry: i64,
}
