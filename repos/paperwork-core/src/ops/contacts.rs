//! Contacts operations: create, add, remove, update, read — all path-explicit.
//!
//! A contacts file is a bullet list of Markdown links to profile files.
//! The three write paths (add/remove/update) run their read-modify-write
//! cycle under an exclusive fs2 lock (spec cli-grammar-v0.6 §3.9).

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::contacts::{parse_contacts, parse_contacts_title, serialize_contacts};
use crate::format::profile::parse_profile;
use crate::ops::lock::locked_read_modify_write;
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
    // Reject empty/whitespace-only profile paths (mirrors the post send
    // --author/--message empty-value precedent): an empty key serializes to
    // an unparseable bullet `- []()` and silently corrupts the file.
    if profile_path.trim().is_empty() {
        return Err(PaperworkError::Validation {
            message: "profile path (--profile) is empty".to_string(),
            fix: "provide a non-empty --profile value".to_string(),
            example: format!("paperwork contacts add {} --profile agents/alice.profile.md", path.display()),
        });
    }

    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: format!("run `paperwork contacts create {}` first", path.display()),
            example: format!("paperwork contacts create {}", path.display()),
        });
    }

    // Pre-derive the label OUTSIDE the lock (impact review Oscar m-1): the
    // exclusive critical section must only touch the locked file itself.
    let label = derive_label(path, profile_path);

    locked_read_modify_write(path, |content| {
        let title = parse_contacts_title(&content)?;
        let mut contacts = parse_contacts(&content)?;

        // Idempotent: skip if already present (the lock helper skips the
        // rewrite when the content is unchanged -> zero write).
        if contacts.iter().any(|c| c.profile_path == profile_path) {
            return Ok(content);
        }

        contacts.push(ContactEntry {
            label,
            profile_path: profile_path.to_string(),
        });

        Ok(serialize_contacts(&title, &contacts))
    })
}

/// Remove a profile path from a contacts file.
///
/// The key is the profile path string exactly as stored (same matching
/// rule as the add idempotency check); labels are derived data and are
/// never usable as keys.
pub fn contacts_remove(path: &Path, profile_path: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: format!("run `paperwork contacts create {}` first", path.display()),
            example: format!("paperwork contacts create {}", path.display()),
        });
    }

    locked_read_modify_write(path, |content| {
        let title = parse_contacts_title(&content)?;
        let mut contacts = parse_contacts(&content)?;

        let original_len = contacts.len();
        contacts.retain(|c| c.profile_path != profile_path);

        if contacts.len() == original_len {
            return Err(PaperworkError::NotFound {
                resource: "Contacts entry".to_string(),
                name: profile_path.to_string(),
                fix: format!(
                    "run `paperwork contacts read {}` to list entries; the key is the profile path as stored in the contacts file, not the label",
                    path.display()
                ),
                example: format!("paperwork contacts read {}", path.display()),
            });
        }

        Ok(serialize_contacts(&title, &contacts))
    })
}

/// Re-bind an entry's destination profile path (in-place replacement).
///
/// Judgment order: the OLD-hit check precedes the NEW-exists check (when
/// OLD == NEW and OLD is present, the result is AlreadyExists). The label
/// is re-derived for NEW per spec §7.3 (R11); entry order is preserved.
/// A non-existent/unreadable NEW still succeeds silently (label falls back
/// to the file-name stem), matching add's frozen behavior (spec §3.6).
pub fn contacts_update(path: &Path, old_profile: &str, new_profile: &str) -> Result<()> {
    // Reject empty/whitespace-only keys (same silent-corruption guard as
    // contacts_add, applied to both --profile and --new-profile).
    if old_profile.trim().is_empty() {
        return Err(PaperworkError::Validation {
            message: "profile path (--profile) is empty".to_string(),
            fix: "provide a non-empty --profile value".to_string(),
            example: format!(
                "paperwork contacts update {} --profile alice.profile.md --new-profile carol.profile.md",
                path.display()
            ),
        });
    }
    if new_profile.trim().is_empty() {
        return Err(PaperworkError::Validation {
            message: "new profile path (--new-profile) is empty".to_string(),
            fix: "provide a non-empty --new-profile value".to_string(),
            example: format!(
                "paperwork contacts update {} --profile alice.profile.md --new-profile carol.profile.md",
                path.display()
            ),
        });
    }

    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: format!("run `paperwork contacts create {}` first", path.display()),
            example: format!("paperwork contacts create {}", path.display()),
        });
    }

    // Pre-derive the NEW label OUTSIDE the lock (impact review Oscar m-1).
    let label = derive_label(path, new_profile);

    locked_read_modify_write(path, |content| {
        let title = parse_contacts_title(&content)?;
        let mut contacts = parse_contacts(&content)?;

        // OLD hit check first.
        let index = match contacts.iter().position(|c| c.profile_path == old_profile) {
            Some(idx) => idx,
            None => {
                return Err(PaperworkError::NotFound {
                    resource: "Contacts entry".to_string(),
                    name: old_profile.to_string(),
                    fix: format!(
                        "run `paperwork contacts read {}` to list entries; the key is the profile path as stored in the contacts file, not the label",
                        path.display()
                    ),
                    example: format!("paperwork contacts read {}", path.display()),
                });
            }
        };

        // NEW already present (covers OLD == NEW with OLD hit).
        if contacts.iter().any(|c| c.profile_path == new_profile) {
            return Err(PaperworkError::AlreadyExists {
                resource: "Contacts entry".to_string(),
                name: new_profile.to_string(),
                fix: "remove the existing entry first or use a different profile path".to_string(),
                example: format!("paperwork contacts remove {} --profile {}", path.display(), new_profile),
            });
        }

        contacts[index] = ContactEntry {
            label,
            profile_path: new_profile.to_string(),
        };

        Ok(serialize_contacts(&title, &contacts))
    })
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
