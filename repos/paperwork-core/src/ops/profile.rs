//! Profile operations: create, show, edit — all path-explicit.
//!
//! Concurrency (review M7): the read-modify-write op (`edit_profile`)
//! holds an fs2 exclusive lock for the whole read → modify → rewrite
//! cycle, so concurrent writers serialize and no update is lost. The
//! rewrite is an in-lock `truncate + write_all`; a crash inside that
//! window can leave the file truncated (accepted, identical to
//! `thread_edit`, spec §5.7 note).

use std::fs;
use std::io::{Seek, Write};
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::profile::{parse_profile, serialize_profile};
use crate::format::{check_single_line, prose_representation_issue};
use crate::Profile;

use super::create_new_file;
use super::lock::LockedFile;

/// Create a new profile file at the given path.
///
/// Fails if the file already exists (no overwrite; atomic `create_new`,
/// NEW-2: no exists()-then-write race window).
/// Creates parent directories if needed.
///
/// Write-side injection guards (NEW-1): `name` and `model` must be single
/// line; `description` must survive a bare-prose roundtrip.
pub fn create_profile(path: &Path, name: &str, model: &str, description: &str) -> Result<()> {
    check_single_line("name", name)?;
    check_single_line("model", model)?;
    if let Some(reason) = prose_representation_issue(description) {
        return Err(PaperworkError::Validation {
            message: format!("profile description is not representable: {}", reason),
            fix: "start the description with a plain prose line and keep attribute-shaped '- key: value' lines out of the description".to_string(),
            example: format!(
                "paperwork profile create {} --name <agent> --model <model-id>",
                path.display()
            ),
        });
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
    create_new_file(path, &content, || PaperworkError::AlreadyExists {
        resource: "Profile".to_string(),
        name: path.display().to_string(),
        fix: "use `paperwork profile edit` to modify an existing profile".to_string(),
        example: format!(
            "paperwork profile edit {} --model <new-model>",
            path.display()
        ),
    })
}

/// One-shot creation of a COMPLETE profile file (Sam-S3).
///
/// Writes the full profile (name, model, description and all scope lists)
/// in a single atomic `create_new` write — no create-then-edit second pass,
/// so there is no intermediate scope-less file that a concurrent reader can
/// observe and no double-open / double-lock window.
///
/// Wired into the CLI since T6: `cmd/profile.rs` Create-with-scope routes
/// here (Ultra Review F7: removed the stale "CLI wiring pending" note —
/// the wiring landed in the T6 CLI JSON convergence batch).
pub fn create_profile_full(path: &Path, profile: &Profile) -> Result<()> {
    check_single_line("name", &profile.name)?;
    check_single_line("model", &profile.model)?;
    if let Some(reason) = prose_representation_issue(&profile.description) {
        return Err(PaperworkError::Validation {
            message: format!("profile description is not representable: {}", reason),
            fix: "start the description with a plain prose line and keep attribute-shaped '- key: value' lines out of the description".to_string(),
            example: format!(
                "paperwork profile create {} --name <agent> --model <model-id>",
                path.display()
            ),
        });
    }

    let content = serialize_profile(profile);
    create_new_file(path, &content, || PaperworkError::AlreadyExists {
        resource: "Profile".to_string(),
        name: path.display().to_string(),
        fix: "use `paperwork profile edit` to modify an existing profile".to_string(),
        example: format!(
            "paperwork profile edit {} --model <new-model>",
            path.display()
        ),
    })
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

    let content = fs::read_to_string(path)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;

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

    let mut profile = parse_profile(&content)?;

    if let Some(m) = model {
        check_single_line("model", m)?;
        profile.model = m.to_string();
    }
    if let Some(d) = description {
        if let Some(reason) = prose_representation_issue(d) {
            return Err(PaperworkError::Validation {
                message: format!("profile description is not representable: {}", reason),
                fix: "start the description with a plain prose line and keep attribute-shaped '- key: value' lines out of the description".to_string(),
                example: format!(
                    "paperwork profile edit {} --description <prose>",
                    path.display()
                ),
            });
        }
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
