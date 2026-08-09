//! Contacts operations: create, add, read — all path-explicit.
//!
//! A contacts file is a bullet list of Markdown links to profile files.

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::contacts::{parse_contacts, parse_contacts_title, serialize_contacts};
use crate::format::profile::parse_profile;
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
            example: format!("paperwork contacts add {} --profile agents/alice.profile.md", path.display()),
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
/// The link label is derived per spec §7.3 (R11): the target profile's H1
/// name, falling back to the file-name stem (`.profile.md` stripped first,
/// then `.md`, else the original name).
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
        label: derive_label(path, profile_path),
        profile_path: profile_path.to_string(),
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

/// Derive the link label for a profile path (spec §7.3, R11).
///
/// Reads the target profile's H1 as the label; on any failure falls back to
/// the file-name stem: strip `.profile.md` first, then `.md`, else keep the
/// original name. The profile path is resolved as given first, then relative
/// to the contacts file's directory.
fn derive_label(contacts_path: &Path, profile_path: &str) -> String {
    let as_given = Path::new(profile_path);
    let resolved = if as_given.exists() {
        as_given.to_path_buf()
    } else if let Some(dir) = contacts_path.parent() {
        dir.join(profile_path)
    } else {
        as_given.to_path_buf()
    };

    if let Ok(content) = fs::read_to_string(&resolved) {
        if let Ok(profile) = parse_profile(&content) {
            return profile.name;
        }
    }

    // Fallback: file-name stem.
    let file_name = as_given
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| profile_path.to_string());
    if let Some(stem) = file_name.strip_suffix(".profile.md") {
        stem.to_string()
    } else if let Some(stem) = file_name.strip_suffix(".md") {
        stem.to_string()
    } else {
        file_name
    }
}
