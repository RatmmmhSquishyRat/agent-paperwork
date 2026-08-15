//! SHA-256 blob hashing utility.
//!
//! Used for manifest entry verification (invariant I7).

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::{PaperworkError, Result};

/// Streaming read chunk for [`hash_file`] (NEW-7): large files are hashed
/// in fixed-size increments instead of being loaded whole.
const HASH_CHUNK_SIZE: usize = 64 * 1024;

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
///
/// NEW-7 streaming digest: the file is read through a `BufReader` in
/// [`HASH_CHUNK_SIZE`] chunks and fed to the hasher incrementally, so the
/// peak memory stays constant regardless of file size (the historical
/// `fs::read` loaded the whole file). The digest — and therefore the hex
/// output — is bit-identical to the one-shot form.
pub fn hash_file(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "check that the file exists and is readable",
            "paperwork brief add onboarding.brief.md --entry src/main.rs",
        )
    })?;

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut chunk = vec![0u8; HASH_CHUNK_SIZE];
    loop {
        let n = reader.read(&mut chunk).map_err(|e| {
            PaperworkError::io_ctx(
                path.to_path_buf(),
                e,
                "check that the file exists and is readable",
                "paperwork brief add onboarding.brief.md --entry src/main.rs",
            )
        })?;
        if n == 0 {
            break;
        }
        hasher.update(&chunk[..n]);
    }

    Ok(hex_encode(&hasher.finalize()))
}

/// Encode bytes as lowercase hex string.
///
/// NEW-11 single-pass encoding: one preallocated `String` plus a nibble
/// lookup table, instead of the historical per-byte `format!` (one heap
/// allocation per byte — 32 for a SHA-256 digest). Output is byte-identical.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX_DIGITS[(byte >> 4) as usize] as char);
        out.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
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

    // NEW-11: every byte value round-trips through the single-pass encoder
    // exactly like the historical `format!("{:02x}", b)` loop.
    #[test]
    fn test_hex_encode_full_byte_range() {
        let all_bytes: Vec<u8> = (0u16..=255).map(|b| b as u8).collect();
        let expected: String = all_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex_encode(&all_bytes), expected);
        assert_eq!(hex_encode(&[]), "");
    }

    // NEW-7: the streaming digest matches the one-shot `hash_bytes` on
    // empty files, small files, and files spanning many chunks (>1MB).
    #[test]
    fn test_hash_file_matches_hash_bytes() {
        let dir = tempfile::tempdir().expect("tempdir");

        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("empty", Vec::new()),
            ("small", b"hello streaming world".to_vec()),
            // exactly one chunk boundary
            ("one-chunk", vec![0x5a; HASH_CHUNK_SIZE]),
            // chunk boundary + 1 (off-by-one probe)
            ("chunk-plus-one", vec![0x5a; HASH_CHUNK_SIZE + 1]),
            // >1MB repetitive content: many full chunks
            (
                "large",
                "agent-paperwork streaming sha256 line\n"
                    .repeat(30_000)
                    .into_bytes(),
            ),
        ];

        for (name, data) in cases {
            let path = dir.path().join(format!("{}.bin", name));
            std::fs::write(&path, &data).expect("write");
            assert_eq!(
                hash_file(&path).expect("hash_file"),
                hash_bytes(&data),
                "streaming digest diverged on case '{}'",
                name
            );
        }

        // the >1MB probe is genuinely multi-chunk
        let large_len = "agent-paperwork streaming sha256 line\n"
            .repeat(30_000)
            .len();
        assert!(large_len > 1024 * 1024);
        assert!(large_len > HASH_CHUNK_SIZE);
    }

    // NEW-7: an empty file keeps the canonical empty-input digest.
    #[test]
    fn test_hash_file_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("empty.bin");
        std::fs::write(&path, b"").expect("write");
        assert_eq!(
            hash_file(&path).expect("hash_file"),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    // NEW-7: missing files keep the historical IoContext envelope.
    #[test]
    fn test_hash_file_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = hash_file(&dir.path().join("missing.bin")).expect_err("must fail");
        assert!(err
            .fix()
            .contains("check that the file exists and is readable"));
    }
}
