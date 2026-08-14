//! Locked read-modify-write helper for non-parallel write paths.
//!
//! Extracted from the `thread_edit` six-step template (spec cli-grammar-v0.6
//! §3.9): open read+write handle -> `lock_exclusive` -> read through the
//! locked handle -> mutate -> truncate + rewrite within the lock -> unlock.
//!
//! Windows hard constraint: file content MUST be read through the locked
//! handle itself; a cross-handle read fails immediately on the locked byte
//! range with os error 33 (ERROR_LOCK_VIOLATION, QA BUG-2).
//!
//! `lock_exclusive` blocks until the lock is available (no built-in
//! timeout, fs2 semantics); a lock acquisition failure fast-fails as an io
//! error envelope — there is no lock-less fallback write path.
//!
//! Crash-window note: the in-lock truncate + rewrite can lose the whole file
//! on power loss / process kill; accepted precedent (format-v2 spec §5.7).
//!
//! Lock-layer ruling (P-1 / NEW-13 closure, 2026-08-15): this helper is the
//! single SSOT for locked read-modify-write. The wip-era `LockedFile` RAII
//! guard (branch wip/v0.5-perfection-snapshot-2026-08-15) was evaluated as
//! design input only and NOT merged: the early-return unlock paths it was
//! built to eliminate are already collapsed inside this helper (every error
//! path — closure errors included — unlocks before returning; no-op results
//! skip the rewrite), and per-step fix wording is part of the byte-frozen
//! io-envelope contract, which a single-context RAII mapper cannot carry.
//! `thread_send` / `thread_edit` keep their inline lock sequences because
//! they are append / rewrite shapes, not RMW, and their per-step wording
//! differs from this helper's (byte-freeze, P-5 golden snapshots).

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use fs2::FileExt;

use crate::error::{PaperworkError, Result};

/// Run a read-modify-write cycle on `path` under an exclusive fs2 lock.
///
/// The closure receives the current file content (read through the locked
/// handle) and returns either the new content to rewrite, or an error.
/// Error paths unlock before returning; the file is only rewritten when the
/// closure returns `Ok` and the new content differs from the original
/// byte-for-byte — an unchanged result skips the truncate + rewrite entirely
/// (keeps mtime stable and removes the no-op crash window, restoring the
/// pre-lock zero-write idempotency semantics of the callers).
pub fn locked_read_modify_write<F>(path: &Path, modify: F) -> Result<()>
where
    F: FnOnce(String) -> Result<String>,
{
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check that the target path is writable".to_string(),
            example: String::new(),
        })?;

    file.lock_exclusive()
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "another process may hold the lock; retry shortly".to_string(),
            example: String::new(),
        })?;

    // Read content through the locked file handle (os error 33 guard).
    let mut content = String::new();
    if let Err(e) = file.seek(SeekFrom::Start(0)) {
        file.unlock().ok();
        return Err(PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        });
    }
    if let Err(e) = file.read_to_string(&mut content) {
        file.unlock().ok();
        return Err(PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file permissions".to_string(),
            example: String::new(),
        });
    }

    // Keep a snapshot for the no-change comparison (content is moved into
    // the closure below).
    let original = content.clone();
    let new_content = match modify(content) {
        Ok(c) => c,
        Err(e) => {
            file.unlock().ok();
            return Err(e);
        }
    };

    // No-op: content unchanged -> skip truncate + rewrite (zero write).
    if new_content == original {
        file.unlock().map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        })?;
        return Ok(());
    }

    // Rewrite entire file (truncate + write within the lock).
    if let Err(e) = file.set_len(0) {
        file.unlock().ok();
        return Err(PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check that the target path is writable".to_string(),
            example: String::new(),
        });
    }
    if let Err(e) = file.seek(SeekFrom::Start(0)) {
        file.unlock().ok();
        return Err(PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        });
    }
    if let Err(e) = file.write_all(new_content.as_bytes()) {
        file.unlock().ok();
        return Err(PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check that the target path is writable".to_string(),
            example: String::new(),
        });
    }

    file.unlock().map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file handle validity".to_string(),
        example: String::new(),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs2::FileExt;
    use std::fs;
    use tempfile::tempdir;

    /// Stronger form of the error-path unlock guarantee (P-1, absorbed from
    /// the wip LockedFile RAII test suite): after the closure returns Err,
    /// the lock must actually be released — proven by a fresh handle being
    /// able to take it immediately (no Drop-based cleanup involved: the
    /// helper's File stays open until the function returns, so a leaked
    /// lock would make try_lock_exclusive fail on Windows).
    #[test]
    fn closure_error_path_releases_lock() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        fs::write(&path, "original\n").expect("write");

        let err = locked_read_modify_write(&path, |_content| {
            Err(PaperworkError::Validation {
                message: "synthetic closure failure".to_string(),
                fix: String::new(),
                example: String::new(),
            })
        })
        .expect_err("closure error must propagate");
        assert_eq!(err.category(), "validation");

        // The file must be untouched AND the lock released.
        assert_eq!(fs::read_to_string(&path).expect("read"), "original\n");
        let probe = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open probe");
        assert!(
            probe.try_lock_exclusive().is_ok(),
            "lock must be released after a closure error"
        );
        probe.unlock().expect("unlock probe");
    }

    /// No-op skip keeps bytes AND mtime stable (zero-write idempotency).
    #[test]
    fn unchanged_result_skips_rewrite() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        fs::write(&path, "same\n").expect("write");
        let before = fs::metadata(&path).expect("metadata").modified().expect("mtime");

        locked_read_modify_write(&path, Ok).expect("no-op rmw");

        assert_eq!(fs::read_to_string(&path).expect("read"), "same\n");
        let after = fs::metadata(&path).expect("metadata").modified().expect("mtime");
        assert_eq!(before, after, "no-op must not rewrite (mtime stable)");
    }

    /// Changed content is rewritten under the lock and visible on disk.
    #[test]
    fn changed_result_rewrites_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        fs::write(&path, "old\n").expect("write");

        locked_read_modify_write(&path, |content| Ok(format!("{}new\n", content)))
            .expect("rmw");

        assert_eq!(fs::read_to_string(&path).expect("read"), "old\nnew\n");
    }

    /// Missing target file fast-fails as an io envelope (no fallback create).
    #[test]
    fn missing_file_is_io_error() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("absent.post.md");

        let err = locked_read_modify_write(&path, Ok).expect_err("must fail");
        assert_eq!(err.category(), "io");
        assert!(err.fix().contains("writable"));
    }
}
