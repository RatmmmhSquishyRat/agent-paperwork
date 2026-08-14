//! Profile operations: create, show, edit — all path-explicit.
//!
//! Concurrency (review M7): the read-modify-write op (`edit_profile`)
//! holds an fs2 exclusive lock for the whole read → modify → rewrite
//! cycle, so concurrent writers serialize and no update is lost. The
//! rewrite is an in-lock `truncate + write_all`; a crash inside that
//! window can leave the file truncated (accepted, identical to
//! `thread_edit`, spec §5.7 note).

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use fs2::FileExt;

use crate::error::{PaperworkError, Result};
use crate::format::profile::{parse_profile, serialize_profile};
use crate::Profile;

/// Create a new profile file at the given path.
///
/// Fails if the file already exists (no overwrite).
/// Creates parent directories if needed.
pub fn create_profile(path: &Path, name: &str, model: &str, description: &str) -> Result<()> {
    if path.exists() {
        return Err(PaperworkError::AlreadyExists {
            resource: "Profile".to_string(),
            name: path.display().to_string(),
            fix: "use `paperwork profile edit` to modify an existing profile".to_string(),
            example: format!(
                "paperwork profile edit {} --model <new-model>",
                path.display()
            ),
        });
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PaperworkError::IoContext {
            path: parent.to_path_buf(),
            source: e,
            fix: "check that the parent directory is writable".to_string(),
            example: String::new(),
        })?;
    }

    let profile = Profile {
        name: name.to_string(),
        model: model.to_string(),
        description: description.to_string(),
        scope_read: Vec::new(),
        scope_write: Vec::new(),
        scope_owns: Vec::new(),
    };

    let content = serialize_profile(&profile);
    fs::write(path, content).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check that the target path is writable".to_string(),
        example: String::new(),
    })?;

    Ok(())
}

/// Read and parse a profile from the given path.
pub fn show_profile(path: &Path) -> Result<Profile> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Profile".to_string(),
            name: path.display().to_string(),
            fix: "run `paperwork profile create <path>` first".to_string(),
            example: format!("paperwork profile create {} --name <agent>", path.display()),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;

    parse_profile(&content)
}

/// Edit an existing profile's fields.
///
/// Only updates the fields that are `Some`.
///
/// Runs under an fs2 exclusive lock for the whole read → modify → rewrite
/// cycle (review M7).
pub fn edit_profile(
    path: &Path,
    model: Option<&str>,
    description: Option<&str>,
    scope_read: Option<Vec<String>>,
    scope_write: Option<Vec<String>>,
    scope_owns: Option<Vec<String>>,
) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Profile".to_string(),
            name: path.display().to_string(),
            fix: "run `paperwork profile create <path>` first".to_string(),
            example: format!("paperwork profile create {} --name <agent>", path.display()),
        });
    }

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

    // Exclusive lock around the full read-modify-write cycle (review M7).
    file.lock_exclusive()
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "another process may hold the lock; retry shortly".to_string(),
            example: String::new(),
        })?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file permissions".to_string(),
            example: String::new(),
        })?;

    let mut profile = parse_profile(&content)?;

    if let Some(m) = model {
        profile.model = m.to_string();
    }
    if let Some(d) = description {
        profile.description = d.to_string();
    }
    if let Some(sr) = scope_read {
        profile.scope_read = sr;
    }
    if let Some(sw) = scope_write {
        profile.scope_write = sw;
    }
    if let Some(so) = scope_owns {
        profile.scope_owns = so;
    }

    let serialized = serialize_profile(&profile);

    // Rewrite through the locked handle (truncate + write within the lock).
    file.set_len(0).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        })?;
    file.write_all(serialized.as_bytes())
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check disk space and file permissions".to_string(),
            example: String::new(),
        })?;

    file.unlock().map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file handle validity".to_string(),
        example: String::new(),
    })?;

    Ok(())
}
