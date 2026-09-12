use uuid::Uuid;

pub const ACCESS_TOKEN_LENGTH: usize = 5;
pub const UPDATE_TOKEN_LENGTH: usize = 8;

/// Generate lowercase hex with four random bits per character.
pub fn generate_token(length: usize) -> String {
    let length = length.min(64); // Preserve the previous SHA3-256 output limit.
    let mut token = String::with_capacity(length);
    while token.len() < length {
        // Each fresh UUID's first 32 bits are random, without version/variant bits.
        token.push_str(&format!("{:08x}", Uuid::new_v4().as_fields().0));
    }
    token.truncate(length);
    token
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
