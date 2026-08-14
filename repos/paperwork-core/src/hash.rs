//! SHA-256 blob hashing utility.
//!
//! Used for manifest entry verification (invariant I7).

use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};

/// Compute SHA-256 hash of raw bytes.
/// Returns lowercase hex string.
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex_encode(&result)
}

/// Compute SHA-256 hash of a file's contents.
/// Returns lowercase hex string.
pub fn hash_file(path: &Path) -> Result<String> {
    let data = fs::read(path).map_err(|e| {
        PaperworkError::io_ctx(path, e, "check that the file exists and is readable", "")
    })?;
    Ok(hash_bytes(&data))
}

/// Encode bytes as lowercase hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes_empty() {
        // SHA-256 of empty string
        let hash = hash_bytes(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_bytes_hello() {
        // SHA-256 of "hello"
        let hash = hash_bytes(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hash_deterministic() {
        let data = b"test data for hashing";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_data() {
        let hash1 = hash_bytes(b"data1");
        let hash2 = hash_bytes(b"data2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(&[0x00, 0xff, 0xab, 0xcd]), "00ffabcd");
    }
}
