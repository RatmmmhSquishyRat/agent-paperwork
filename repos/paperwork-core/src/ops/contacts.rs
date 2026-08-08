//! Contacts operations: create, add, read — all path-explicit.
//!
//! A contacts file is a special brief: a table of profile paths + summaries.

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::contacts::{parse_contacts, parse_contacts_title, serialize_contacts};
use crate::ContactEntry;

/// Create a new empty contacts file at the given path.
///
/// Creates parent directories if needed.
/// Fails if the file already exists.
pub fn contacts_create(path: &Path, title: &str) -> Result<()> {
    if path.exists() {
        return Err(PaperworkError::AlreadyExists {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: "use `paperwork contacts add` to add entries".to_string(),
            example: format!("paperwork contacts add {} agents/alice.profile.md", path.display()),
        });
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PaperworkError::IoContext {
            path: parent.to_path_buf(),
            source: e,
            fix: "check that the parent directory is writable".to_string(),
            example: String::new(),
        })?;
    }

    let content = serialize_contacts(title, &[]);
    fs::write(path, content).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check that the target path is writable".to_string(),
        example: String::new(),
    })?;

    Ok(())
}

/// Add a profile path to a contacts file.
///
/// The summary is left empty; the agent name is derived from the file name.
/// Idempotent: adding an already-present path is a no-op.
pub fn contacts_add(path: &Path, profile_path: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: format!("run `paperwork contacts create {}` first", path.display()),
            example: format!("paperwork contacts create {}", path.display()),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;

    let title = parse_contacts_title(&content)?;
    let mut contacts = parse_contacts(&content)?;

    // Idempotent: skip if already present
    if contacts.iter().any(|c| c.profile_path == profile_path) {
        return Ok(());
    }

    contacts.push(ContactEntry {
        profile_path: profile_path.to_string(),
        summary: String::new(),
    });

    let serialized = serialize_contacts(&title, &contacts);
    fs::write(path, serialized).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check that the target path is writable".to_string(),
        example: String::new(),
    })?;

    Ok(())
}

/// Read all contacts from a contacts file.
pub fn contacts_read(path: &Path) -> Result<Vec<ContactEntry>> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: format!("run `paperwork contacts create {}` first", path.display()),
            example: format!("paperwork contacts create {}", path.display()),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;

    parse_contacts(&content)
}
