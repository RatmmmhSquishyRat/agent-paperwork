//! Profile operations: create, show, edit — all path-explicit.

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::profile::{parse_profile, serialize_profile};
use crate::ops::lock::locked_read_modify_write;
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
            example: format!("paperwork profile edit {} --model gpt-4o", path.display()),
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
            fix: format!("run `paperwork profile create {} --name alice` first", path.display()),
            example: format!("paperwork profile create {} --name alice", path.display()),
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
/// Runs under the locked read-modify-write template (spec cli-grammar-v0.6
/// §3.9).
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
            fix: format!("run `paperwork profile create {} --name alice` first", path.display()),
            example: format!("paperwork profile create {} --name alice", path.display()),
        });
    }

    locked_read_modify_write(path, |content| {
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

        Ok(serialize_profile(&profile))
    })
}
