//! Contacts operations: create, add, read — all path-explicit.
//!
//! A contacts file is a bullet list of Markdown links to profile files.
//!
//! Concurrency (review M7): every read-modify-write op (`contacts_add`)
//! holds an fs2 exclusive lock for the whole read → modify → rewrite cycle,
//! so concurrent writers serialize and no update is lost. The rewrite is an
//! in-lock `truncate + write_all`; a crash inside that window can leave the
//! file truncated (accepted, identical to `thread_edit`, spec §5.7 note).

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use fs2::FileExt;

use crate::error::{PaperworkError, Result};
use crate::format::contacts::{
    contains_bare_bullet, parse_contacts, parse_contacts_title, serialize_contacts,
};
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
            example: format!("paperwork contacts add {} --profile <path>", path.display()),
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
///
/// Legacy write guard (review B1): a file carrying fence-outside bare
/// bullets (v0.4 `- path/to/profile.md` entries) is refused with a Parse
/// error — v0.5 parsing ignores those bullets, so the read-modify-rewrite
/// would silently drop every legacy entry. The whole read → modify →
/// rewrite cycle runs under an fs2 exclusive lock (review M7).
pub fn contacts_add(path: &Path, profile_path: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: "run `paperwork contacts create <path>` first".to_string(),
            example: format!("paperwork contacts create {}", path.display()),
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

    // Legacy guard (review B1): refuse to rewrite over uninterpreted
    // bare-path bullets; migration is manual (CHANGELOG guide).
    if contains_bare_bullet(&content) {
        file.unlock().ok();
        return Err(PaperworkError::Parse {
            message: "contacts file contains legacy bare-path bullets that v0.5 parsing ignores".to_string(),
            fix: "this file is in the v0.4 legacy format; v0.5 is not forward compatible - migrate it by hand per the CHANGELOG migration guide before adding entries".to_string(),
            example: "see CHANGELOG.md, [0.5.0] 'Migration guide (manual)', contacts".to_string(),
        });
    }

    let title = parse_contacts_title(&content)?;
    let mut contacts = parse_contacts(&content)?;

    // Idempotent: skip if already present
    if contacts.iter().any(|c| c.profile_path == profile_path) {
        file.unlock().ok();
        return Ok(());
    }

    contacts.push(ContactEntry {
        label: derive_label(path, profile_path),
        profile_path: profile_path.to_string(),
    });

    let serialized = serialize_contacts(&title, &contacts);

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

/// Read all contacts from a contacts file.
pub fn contacts_read(path: &Path) -> Result<Vec<ContactEntry>> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: "run `paperwork contacts create <path>` first".to_string(),
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
