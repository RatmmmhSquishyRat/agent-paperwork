//! Profile operations: create, show, edit — all path-explicit.
//!
//! Concurrency (review M7): the read-modify-write op (`edit_profile`)
//! holds an fs2 exclusive lock for the whole read → modify → rewrite
//! cycle, so concurrent writers serialize and no update is lost. The
//! rewrite is an in-lock `truncate + write_all`; a crash inside that
//! window can leave the file truncated (accepted, identical to
//! `thread_edit`, spec §5.7 note).

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::profile::{parse_profile, serialize_profile};
use crate::format::{check_single_line, prose_representation_issue};
use crate::ops::lock::locked_read_modify_write;
use crate::Profile;

use super::create_new_file;

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
        example: format!("paperwork profile edit {} --model gpt-4o", path.display()),
    })
}

/// One-shot creation of a COMPLETE profile file (SAM-2).
///
/// Writes the full profile (name, model, description and all scope lists)
/// in a single atomic `create_new` write — no create-then-edit second pass,
/// so there is no intermediate scope-less file that a concurrent reader can
/// observe and no double-open / double-lock window. Same injection guards
/// as [`create_profile`].
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
        example: format!("paperwork profile edit {} --model gpt-4o", path.display()),
    })
}

/// Read and parse a profile from the given path.
pub fn show_profile(path: &Path) -> Result<Profile> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Profile".to_string(),
            name: path.display().to_string(),
            fix: format!(
                "run `paperwork profile create {} --name alice` first",
                path.display()
            ),
            example: format!("paperwork profile create {} --name alice", path.display()),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "check file permissions",
            String::new(),
        )
    })?;

    parse_profile(&content)
}

/// Edit an existing profile's fields.
///
/// Only updates the fields that are `Some`.
/// Runs under the locked read-modify-write template (spec cli-grammar-v0.6
/// §3.9, review M7).
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
            fix: format!(
                "run `paperwork profile create {} --name alice` first",
                path.display()
            ),
            example: format!("paperwork profile create {} --name alice", path.display()),
        });
    }

    locked_read_modify_write(path, |content| {
        let mut profile = parse_profile(&content)?;

        if let Some(m) = model {
            check_single_line("model", m)?;
            profile.model = m.to_string();
        }
        if let Some(d) = description {
            // NEW-1 preamble prose guard; envelope wording mirrors
            // create_profile. A Validation error here leaves the file
            // untouched (the lock helper skips the rewrite on Err).
            if let Some(reason) = prose_representation_issue(d) {
                return Err(PaperworkError::Validation {
                    message: format!("profile description is not representable: {}", reason),
                    fix: "start the description with a plain prose line and keep attribute-shaped '- key: value' lines out of the description".to_string(),
                    example: format!(
                        "paperwork profile edit {} --model gpt-4o",
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

        Ok(serialize_profile(&profile))
    })
}
