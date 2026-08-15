//! Brief operations (formerly "manifest"): create, add/remove entry, read, verify.
//!
//! A brief is a standalone reading list / knowledge brief.
//! All operations take explicit file paths — no workspace root.
//!
//! Concurrency (review M7): every read-modify-write op (`brief_add_entry`,
//! `brief_remove_entry`) holds an fs2 exclusive lock for the whole
//! read → modify → rewrite cycle, so concurrent writers serialize and no
//! update is lost. The rewrite is an in-lock `truncate + write_all`; a
//! crash inside that window can leave the file truncated (accepted,
//! identical to `thread_edit`, spec §5.7 note).

use std::fs;
use std::path::Path;

use chrono::Utc;
use regex::Regex;

use crate::error::{PaperworkError, Result};
use crate::format::manifest::{
    extract_regex_groups, note_representation_issue, parse_manifest, serialize_manifest,
};
use crate::format::{check_single_line, prose_representation_issue};
use crate::hash;
use crate::ops::lock::locked_read_modify_write;
use crate::{Manifest, ManifestEntry, VerifyResult};

use super::create_new_file;

/// Create a new empty brief at the given path.
///
/// Creates parent directories if needed.
/// Fails if the file already exists (atomic `create_new`, NEW-2).
///
/// Write-side injection guards (NEW-1): `title` and `owner` must be single
/// line; `description` must survive a bare-prose roundtrip.
pub fn brief_create(
    path: &Path,
    title: &str,
    owner: Option<&str>,
    description: &str,
) -> Result<()> {
    check_single_line("title", title)?;
    if let Some(owner) = owner {
        check_single_line("owner", owner)?;
    }
    if let Some(reason) = prose_representation_issue(description) {
        return Err(PaperworkError::Validation {
            message: format!("brief description is not representable: {}", reason),
            fix: "start the description with a plain prose line and keep attribute-shaped '- key: value' lines out of the description".to_string(),
            example: format!("paperwork brief create {} --title <title>", path.display()),
        });
    }

    let manifest = Manifest {
        name: title.to_string(),
        author: owner.unwrap_or("").to_string(),
        created: Utc::now(),
        description: description.to_string(),
        entries: Vec::new(),
    };

    let content = serialize_manifest(&manifest);
    create_new_file(path, &content, || PaperworkError::AlreadyExists {
        resource: "Brief".to_string(),
        name: path.display().to_string(),
        fix: "use `paperwork brief add` to add entries".to_string(),
        example: format!("paperwork brief add {} --entry src/main.rs", path.display()),
    })
}

/// Add an entry to a brief.
///
/// The entry title is derived from the file name of `entry_path`.
/// Computes the SHA-256 hash of the file at `entry_path` (resolved relative
/// to the brief file's parent directory).
///
/// Note representability guard (review M1, extended C-1): a note whose
/// first non-blank line is attribute-shaped (`- key: value`) or opens a
/// ```` ```regex ```` fence would be re-absorbed into the attribute zone on
/// the next parse, silently corrupting the entry; a heading-shaped line
/// outside a fence trips the residue parse guard (`### `, locking the whole
/// brief out) or splits the entry (`## `); an unclosed fence swallows every
/// later entry. All such notes are refused with a Validation error before
/// anything touches disk.
///
/// Entry-title residue guard (C-1): a title serializing to the legacy
/// `## Entries` wrapper heading would trip the read-side SAM-1 parse guard
/// and permanently lock the brief out of read/add/remove/verify — refused
/// before locking/writing.
///
/// Runs under the locked read-modify-write template (spec cli-grammar-v0.6
/// §3.9, review M7).
pub fn brief_add_entry(
    path: &Path,
    entry_path: &str,
    regex: Option<&str>,
    note: Option<&str>,
) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Brief".to_string(),
            name: path.display().to_string(),
            fix: format!(
                "run `paperwork brief create {} --title \"My Brief\"` first",
                path.display()
            ),
            example: format!(
                "paperwork brief create {} --title \"My Brief\"",
                path.display()
            ),
        });
    }

    // Write-side injection guard (NEW-1): the entry path becomes a `- path:`
    // attribute line (single-line field); a newline would corrupt the entry.
    check_single_line("entry path", entry_path)?;

    // Derive the entry title from the entry_path file name BEFORE the lock:
    // the residue guard below needs it, and the derivation is a pure
    // function of entry_path (C-1).
    let title = Path::new(entry_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| entry_path.to_string());

    // Entry-title residue guard (C-1): `## Entries` trips the read-side
    // SAM-1 legacy-residue parse guard and locks the whole brief out.
    if title.trim() == "Entries" {
        return Err(PaperworkError::Validation {
            message: format!(
                "entry title '{}' serializes to the legacy '## Entries' wrapper heading, which the parser refuses",
                title
            ),
            fix: "use an entry file whose name is not 'Entries'; the v0.5 layout has no entries wrapper — every entry is its own '## <title>' section".to_string(),
            example: format!(
                "paperwork brief add {} --entry notes/chapter-one.rs",
                path.display()
            ),
        });
    }

    // Note representability guard (review M1, extended C-1) — reject before
    // locking/writing.
    if let Some(note_text) = note {
        if let Some(reason) = note_representation_issue(note_text) {
            return Err(PaperworkError::Validation {
                message: format!("note is not representable in brief format: {}", reason),
                fix: "start the note with a plain prose line; attribute-shaped '- key: value' first lines and ```regex fence openings are reserved for entry attributes, and heading-shaped lines ('#', '##', '###') belong inside a code fence — outside a fence they would trip the residue parse guard or split the entry".to_string(),
                example: format!("paperwork brief add {} --entry {} --note \"Reading notes for this file\"", path.display(), entry_path),
            });
        }
    }

    // Pre-compute the entry file hash OUTSIDE the lock (impact review
    // Oscar m-1): the exclusive critical section must only read-modify-write
    // the locked brief file itself, never external files.
    // Resolve entry file path: try as-is (CWD-relative) first, then relative to brief's parent
    let entry_as_given = Path::new(entry_path);
    let base_dir = path.parent().unwrap_or(Path::new("."));
    let abs_entry_path = if entry_as_given.exists() {
        entry_as_given.to_path_buf()
    } else {
        base_dir.join(entry_path)
    };

    let file_hash = hash::hash_file(&abs_entry_path)?;

    locked_read_modify_write(path, |content| {
        let mut manifest = parse_manifest(&content)?;

        // Check for duplicate title
        if manifest.entries.iter().any(|e| e.title == title) {
            return Err(PaperworkError::AlreadyExists {
                resource: "Brief entry".to_string(),
                name: title,
                fix: "use a different entry path or remove the existing entry first".to_string(),
                example: format!(
                    "paperwork brief remove {} --entry-title main.rs",
                    path.display()
                ),
            });
        }

        let groups = regex.map(extract_regex_groups).unwrap_or_default();

        let entry = ManifestEntry {
            title: title.clone(),
            path: entry_path.to_string(),
            hash: file_hash,
            regex: regex.map(|s| s.to_string()),
            groups,
            note: note.map(|s| s.to_string()),
        };

        manifest.entries.push(entry);

        Ok(serialize_manifest(&manifest))
    })
}

/// Remove an entry from a brief by title.
///
/// Runs under the locked read-modify-write template (spec cli-grammar-v0.6
/// §3.9, review M7).
pub fn brief_remove_entry(path: &Path, title: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Brief".to_string(),
            name: path.display().to_string(),
            fix: format!(
                "run `paperwork brief create {} --title \"My Brief\"` first",
                path.display()
            ),
            example: format!(
                "paperwork brief create {} --title \"My Brief\"",
                path.display()
            ),
        });
    }

    locked_read_modify_write(path, |content| {
        let mut manifest = parse_manifest(&content)?;

        let original_len = manifest.entries.len();
        manifest.entries.retain(|e| e.title != title);

        if manifest.entries.len() == original_len {
            return Err(PaperworkError::NotFound {
                resource: "Brief entry".to_string(),
                name: title.to_string(),
                fix: format!(
                    "run `paperwork brief read {}` to see available entries",
                    path.display()
                ),
                example: format!("paperwork brief read {}", path.display()),
            });
        }

        Ok(serialize_manifest(&manifest))
    })
}

/// Read a brief (reuses the Manifest type internally).
pub fn brief_read(path: &Path) -> Result<Manifest> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Brief".to_string(),
            name: path.display().to_string(),
            fix: format!(
                "run `paperwork brief create {} --title \"My Brief\"` first",
                path.display()
            ),
            example: format!(
                "paperwork brief create {} --title \"My Brief\"",
                path.display()
            ),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| {
        PaperworkError::io_ctx_file_read(
            path.to_path_buf(),
            e,
            "check file permissions",
            String::new(),
        )
    })?;

    parse_manifest(&content)
}

/// Verify all entries in a brief.
///
/// `base_dir` is used to resolve relative paths in entries.
///
/// Three-state verification:
/// - Fresh: regex matches (or no regex) + hash matches
/// - Shifted: regex matches (or no regex) + hash differs
/// - Stale: regex fails to match (or file missing)
pub fn brief_verify(path: &Path, base_dir: &Path) -> Result<Vec<(ManifestEntry, VerifyResult)>> {
    let manifest = brief_read(path)?;
    let mut results = Vec::new();

    for entry in &manifest.entries {
        let result = verify_entry(base_dir, entry)?;
        results.push((entry.clone(), result));
    }

    Ok(results)
}

/// Verify a single brief entry against the current file state.
///
/// Reads the target file ONCE as bytes (review n15): the regex check runs
/// on a `from_utf8_lossy` view and the hash on the raw bytes — non-UTF-8
/// files no longer collapse to Stale, and the hash matches `hash_file`
/// (raw-byte SHA-256) exactly.
///
/// SAM-4 (ruling A): a MISSING target stays Stale — that is the frozen
/// spec §6 three-state contract ("Stale: regex fails to match (or file
/// missing)") and intentional design, not error swallowing. Any OTHER read
/// failure (permission denied, is-a-directory, read interruption, ...) is a
/// genuine IO fault and surfaces as an IoContext envelope instead of
/// collapsing into Stale.
fn verify_entry(base_dir: &Path, entry: &ManifestEntry) -> Result<VerifyResult> {
    let abs_path = base_dir.join(&entry.path);

    let bytes = match fs::read(&abs_path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(VerifyResult::Stale);
        }
        Err(e) => {
            return Err(PaperworkError::io_ctx(
                abs_path,
                e,
                "the brief entry target could not be read; check file permissions and disk integrity, or fix the entry path",
                format!("paperwork brief read <brief-path>  # then check the '- path:' value of entry '{}'", entry.title),
            ));
        }
    };

    // Check regex if present (lossy view: non-UTF-8 bytes become U+FFFD).
    if let Some(ref pattern) = entry.regex {
        match Regex::new(pattern) {
            Ok(re) => {
                let text = String::from_utf8_lossy(&bytes);
                if !re.is_match(&text) {
                    return Ok(VerifyResult::Stale);
                }
            }
            Err(_) => return Ok(VerifyResult::Stale),
        }
    }

    // Hash the same bytes in memory (no second file read).
    let current_hash = hash::hash_bytes(&bytes);

    if current_hash == entry.hash {
        Ok(VerifyResult::Fresh)
    } else {
        Ok(VerifyResult::Shifted)
    }
}
