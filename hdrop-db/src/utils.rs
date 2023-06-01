use sha3::{Digest, Sha3_256};
use uuid::Uuid;

/* TokenGenerator */
pub const UPDATE_TOKEN_LENGTH: usize = 8;
pub struct TokenGenerator {
    access_token_min_length: usize,
}

impl Default for TokenGenerator {
    fn default() -> Self {
        TokenGenerator {
            access_token_min_length: 5,
        }
    }
}

impl TokenGenerator {
    pub fn new(access_token_min_length: usize) -> Self {
        TokenGenerator {
            access_token_min_length,
        }
    }

    pub fn get_access_token_min_length(&self) -> usize {
        self.access_token_min_length
    }

    /// Generate Token
    /// Generates Sha3(uuidv4) as String
    pub fn generate_token(length: usize) -> String {
        let uuid = Uuid::new_v4();
        let mut hasher = Sha3_256::new();
        hasher.update(uuid);
        let result = hasher.finalize();
        let result = format!("{:x}", result);
        result.chars().take(length).collect::<String>();
        result
    }
}
