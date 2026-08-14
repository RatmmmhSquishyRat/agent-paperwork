//! Operations layer: stateless, path-explicit filesystem operations.
//!
//! Every operation takes an explicit file path. No workspace root, no init, no state.
//! Files are independent — no cross-references managed by the CLI.

pub mod contacts;
pub(crate) mod lock;
pub mod manifest;
pub mod profile;
pub mod thread;

use std::fs::{self, OpenOptions};
use std::path::Path;

use crate::error::PaperworkError;

/// Create a brand-new file atomically (NEW-2 TOCTOU fix).
///
/// Creates parent directories first, then opens the target with
/// `create_new(true)`: the existence check and the creation happen inside a
/// single kernel operation, so two racing creators can never both succeed
/// (the old `path.exists()` + `fs::write` window let the second writer
/// overwrite the first file). Windows `ERROR_FILE_EXISTS` maps to the same
/// `ErrorKind::AlreadyExists`.
///
/// `already_exists` builds the caller-specific AlreadyExists envelope; the
/// resource/name/fix/example wording stays byte-identical to the pre-fix
/// envelopes (output contract).
pub(crate) fn create_new_file(
    path: &Path,
    content: &str,
    already_exists: impl FnOnce() -> PaperworkError,
) -> crate::error::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            PaperworkError::io_ctx(parent, e, "check that the parent directory is writable", "")
        })?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                already_exists()
            } else {
                PaperworkError::io_ctx(path, e, "check that the target path is writable", "")
            }
        })?;

    use std::io::Write;
    file.write_all(content.as_bytes()).map_err(|e| {
        PaperworkError::io_ctx(path, e, "check that the target path is writable", "")
    })?;

    Ok(())
}
