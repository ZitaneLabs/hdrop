use sha3::{Digest, Sha3_256};
use uuid::Uuid;

pub const ACCESS_TOKEN_LENGTH: usize = 5;
pub const UPDATE_TOKEN_LENGTH: usize = 8;

/// Generates a SHA3(uuidv4) as String truncated to the given length.
pub fn generate_token(length: usize) -> String {
    let uuid = Uuid::new_v4();
    let mut hasher = Sha3_256::new();
    hasher.update(uuid);
    let result = hasher.finalize();
    let result = hex::encode(result);
    result.chars().take(length).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    #[test]
    fn token_formats_and_lengths() {
        assert_eq!(Database::generate_access_token().len(), 5);
        assert_eq!(Database::generate_update_token().len(), 8);
        for length in [0, 5, 8, 12, 32, 64, 65] {
            let token = generate_token(length);
            assert_eq!(token.len(), length.min(64));
            assert!(token
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
        }
    }
}
