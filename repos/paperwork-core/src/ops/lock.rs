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
/// closure returns `Ok`.
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
            fix: "check file permissions".to_string(),
            example: String::new(),
        })?;

    file.lock_exclusive().map_err(|e| PaperworkError::IoContext {
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

    let new_content = match modify(content) {
        Ok(c) => c,
        Err(e) => {
            file.unlock().ok();
            return Err(e);
        }
    };

    // Rewrite entire file (truncate + write within the lock).
    if let Err(e) = file.set_len(0) {
        file.unlock().ok();
        return Err(PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file permissions".to_string(),
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
            fix: "check disk space and file permissions".to_string(),
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
