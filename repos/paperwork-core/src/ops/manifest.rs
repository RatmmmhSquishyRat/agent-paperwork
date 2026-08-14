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
use std::io::{Seek, Write};
use std::path::Path;

use chrono::Utc;
use regex::Regex;

use crate::error::{PaperworkError, Result};
use crate::format::check_single_line;
use crate::format::manifest::{
    extract_regex_groups, note_representation_issue, parse_manifest, serialize_manifest,
};
use crate::format::prose_representation_issue;
use crate::hash;
use crate::{Manifest, ManifestEntry, VerifyResult};

use super::create_new_file;
use super::lock::LockedFile;

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
        example: format!("paperwork brief add {} --entry <file>", path.display()),
    })
}

/// Add an entry to a brief.
///
/// The entry title is derived from the file name of `entry_path`.
/// Computes the SHA-256 hash of the file at `entry_path` (resolved relative
/// to the brief file's parent directory).
///
/// Note representability guard (review M1): a note whose first non-blank
/// line is attribute-shaped (`- key: value`) or opens a ```` ```regex ````
/// fence would be re-absorbed into the attribute zone on the next parse,
/// silently corrupting the entry — such notes are refused with a
/// Validation error before anything touches disk. The whole read → modify
/// → rewrite cycle runs under an fs2 exclusive lock (review M7).
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
            fix: "run `paperwork brief create <path>` first".to_string(),
            example: format!("paperwork brief create {} --title <title>", path.display()),
        });
    }

    // Write-side injection guards (NEW-1): the entry path becomes a `- path:`
    // attribute line (single-line field), and the entry title is derived
    // from its file name — a newline in either would corrupt the entry.
    check_single_line("entry path", entry_path)?;

    // Note representability guard (review M1, unified helper NEW-1) —
    // reject before locking/writing. The envelope wording is unchanged.
    // Notes sit after the entry attribute zone, so only the M1 first-line
    // shapes are refused; later attribute-shaped lines inside a note stay
    // legal (BDD:BRIEF-12, pinned by existing tests).
    if let Some(note_text) = note {
        if let Some(reason) = note_representation_issue(note_text) {
            return Err(PaperworkError::Validation {
                message: format!("note is not representable in brief format: {}", reason),
                fix: "start the note with a plain prose line; attribute-shaped '- key: value' first lines and ```regex fence openings are reserved for entry attributes".to_string(),
                example: format!("paperwork brief add {} --entry {} --note \"Reading notes for this file\"", path.display(), entry_path),
            });
        }
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

    let mut manifest = parse_manifest(&content)?;

    // Derive title from entry_path file name
    let title = Path::new(entry_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| entry_path.to_string());

    // Check for duplicate title
    if manifest.entries.iter().any(|e| e.title == title) {
        return Err(PaperworkError::AlreadyExists {
            resource: "Brief entry".to_string(),
            name: title,
            fix: "use a different entry path or remove the existing entry first".to_string(),
            example: format!(
                "paperwork brief remove {} --entry-title <title>",
                path.display()
            ),
        });
    }

    // Resolve entry file path: try as-is (CWD-relative) first, then relative to brief's parent
    let entry_as_given = Path::new(entry_path);
    let base_dir = path.parent().unwrap_or(Path::new("."));
    let abs_entry_path = if entry_as_given.exists() {
        entry_as_given.to_path_buf()
    } else {
        base_dir.join(entry_path)
    };

    let file_hash = hash::hash_file(&abs_entry_path)?;

    let groups = regex.map(extract_regex_groups).unwrap_or_default();

    let entry = ManifestEntry {
        title,
        path: entry_path.to_string(),
        hash: file_hash,
        regex: regex.map(|s| s.to_string()),
        groups,
        note: note.map(|s| s.to_string()),
    };

    manifest.entries.push(entry);

    let serialized = serialize_manifest(&manifest);

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

/// Remove an entry from a brief by title.
///
/// Runs under an fs2 exclusive lock for the whole read → modify → rewrite
/// cycle (review M7).
pub fn brief_remove_entry(path: &Path, title: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Brief".to_string(),
            name: path.display().to_string(),
            fix: "run `paperwork brief create <path>` first".to_string(),
            example: format!("paperwork brief create {} --title <title>", path.display()),
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

    let mut manifest = parse_manifest(&content)?;

    let original_len = manifest.entries.len();
    manifest.entries.retain(|e| e.title != title);

    if manifest.entries.len() == original_len {
        return Err(PaperworkError::NotFound {
            resource: "Brief entry".to_string(),
            name: title.to_string(),
            fix: "run `paperwork brief read <path>` to see available entries".to_string(),
            example: format!("paperwork brief read {}", path.display()),
        });
    }

    let serialized = serialize_manifest(&manifest);

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

/// Read a brief (reuses the Manifest type internally).
pub fn brief_read(path: &Path) -> Result<Manifest> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Brief".to_string(),
            name: path.display().to_string(),
            fix: "run `paperwork brief create <path>` first".to_string(),
            example: format!("paperwork brief create {} --title <title>", path.display()),
        });
    }

    let content = fs::read_to_string(path)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;

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
/// Sam-S5 (ruling A, final): a MISSING target stays Stale — that is the
/// frozen spec §6 three-state contract ("Stale: regex fails to match (or
/// file missing)") and intentional design, not error swallowing. Any OTHER
/// read failure (permission denied, read interruption, ...) is a genuine
/// IO fault and surfaces as an IoContext envelope instead of collapsing
/// into Stale.
fn verify_entry(base_dir: &Path, entry: &ManifestEntry) -> Result<VerifyResult> {
    let abs_path = base_dir.join(&entry.path);

    let bytes = match fs::read(&abs_path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(VerifyResult::Stale);
        }
        Err(e) => {
            return Err(PaperworkError::io_ctx(
                abs_path.clone(),
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
