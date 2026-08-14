//! Contacts operations: create, add, read — all path-explicit.
//!
//! A contacts file is a bullet list of Markdown links to profile files.
//!
//! Concurrency (review M7): every read-modify-write op (`contacts_add`)
//! holds an fs2 exclusive lock for the whole read → modify → rewrite cycle,
//! so concurrent writers serialize and no update is lost. The rewrite is an
//! in-lock `truncate + write_all`; a crash inside that window can leave the
//! file truncated (accepted, identical to `thread_edit`, spec §5.7 note).

use std::fs;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

use crate::error::{PaperworkError, Result};
use crate::format::check_single_line;
use crate::format::contacts::{
    contains_bare_bullet, parse_contacts, parse_contacts_title, serialize_contacts,
};
use crate::format::profile::parse_profile;
use crate::format::strip_known_suffix;
use crate::ContactEntry;

use super::create_new_file;
use super::lock::LockedFile;

/// Create a new empty contacts file at the given path.
///
/// Creates parent directories if needed.
/// Fails if the file already exists (atomic `create_new`, NEW-2).
pub fn contacts_create(path: &Path, title: &str) -> Result<()> {
    check_single_line("title", title)?;

    let content = serialize_contacts(title, &[]);
    create_new_file(path, &content, || PaperworkError::AlreadyExists {
        resource: "Contacts".to_string(),
        name: path.display().to_string(),
        fix: "use `paperwork contacts add` to add entries".to_string(),
        example: format!("paperwork contacts add {} --profile <path>", path.display()),
    })
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
    // Write-side injection guard (NEW-1): a newline inside the destination
    // would break the single-line link bullet structure.
    check_single_line("profile path", profile_path)?;

    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Contacts".to_string(),
            name: path.display().to_string(),
            fix: "run `paperwork contacts create <path>` first".to_string(),
            example: format!("paperwork contacts create {}", path.display()),
        });
    }

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;

    // Exclusive lock around the full read-modify-write cycle (review M7);
    // the guard's Drop releases the lock on every exit path (T4).
    let guard = LockedFile::acquire(file, |e| {
        PaperworkError::io_ctx(
            path,
            e,
            "another process may hold the lock; retry shortly",
            "",
        )
    })?;

    let content =
        guard.read_to_string(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;

    // Legacy guard (review B1): refuse to rewrite over uninterpreted
    // bare-path bullets; migration is manual (CHANGELOG guide).
    if contains_bare_bullet(&content) {
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
        return Ok(());
    }

    contacts.push(ContactEntry {
        label: derive_label(path, profile_path),
        profile_path: profile_path.to_string(),
    });

    let serialized = serialize_contacts(&title, &contacts);

    // Rewrite through the locked handle (truncate + write within the lock);
    // per-step wording preserved via the `file()` escape hatch (T4).
    let file = guard.file();
    file.set_len(0)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;
    let mut handle = file;
    handle
        .seek(std::io::SeekFrom::Start(0))
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file handle validity", ""))?;
    handle.write_all(serialized.as_bytes()).map_err(|e| {
        PaperworkError::io_ctx(path, e, "check disk space and file permissions", "")
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

    let content = fs::read_to_string(path)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;

    parse_contacts(&content)
}

/// Resolve a contact entry's profile path (NEW-4, spec §7.3 R11).
///
/// Two-level resolution, shared by every consumer that follows a contacts
/// link: the entry path is tried as given (CWD-relative) first, then
/// relative to the contacts file's own directory. `derive_label` uses this
/// internally; the CLI enrichment path uses the same helper (T6 wired in
/// `cmd/contacts.rs::enrich_profile`).
pub fn resolve_contact_path(contacts_path: &Path, entry_path: &str) -> PathBuf {
    let as_given = Path::new(entry_path);
    if as_given.exists() {
        return as_given.to_path_buf();
    }
    match contacts_path.parent() {
        Some(dir) => dir.join(entry_path),
        None => as_given.to_path_buf(),
    }
}

/// Derive the link label for a profile path (spec §7.3, R11).
///
/// Reads the target profile's H1 as the label; on any failure falls back to
/// the file-name stem: strip `.profile.md` first, then `.md`, else keep the
/// original name. Resolution is delegated to [`resolve_contact_path`]
/// (as-given first, then contacts-directory-relative).
fn derive_label(contacts_path: &Path, profile_path: &str) -> String {
    let as_given = Path::new(profile_path);
    let resolved = resolve_contact_path(contacts_path, profile_path);

    if let Ok(content) = fs::read_to_string(&resolved) {
        if let Ok(profile) = parse_profile(&content) {
            return profile.name;
        }
    }

    // Fallback: file-name stem (T4: shared [`strip_known_suffix`], pub for
    // the T6 CLI `default_title` wiring).
    let file_name = as_given
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| profile_path.to_string());
    strip_known_suffix(&file_name).to_string()
}
