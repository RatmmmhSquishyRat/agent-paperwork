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

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use chrono::Utc;
use fs2::FileExt;
use regex::Regex;

use crate::error::{PaperworkError, Result};
use crate::format::manifest::{
    extract_regex_groups, note_representation_issue, parse_manifest, serialize_manifest,
};
use crate::hash;
use crate::{Manifest, ManifestEntry, VerifyResult};

/// Create a new empty brief at the given path.
///
/// Creates parent directories if needed.
/// Fails if the file already exists.
pub fn brief_create(
    path: &Path,
    title: &str,
    owner: Option<&str>,
    description: &str,
) -> Result<()> {
    if path.exists() {
        return Err(PaperworkError::AlreadyExists {
            resource: "Brief".to_string(),
            name: path.display().to_string(),
            fix: "use `paperwork brief add` to add entries".to_string(),
            example: format!("paperwork brief add {} --entry <file>", path.display()),
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

    let manifest = Manifest {
        name: title.to_string(),
        author: owner.unwrap_or("").to_string(),
        created: Utc::now(),
        description: description.to_string(),
        entries: Vec::new(),
    };

    let content = serialize_manifest(&manifest);
    fs::write(path, content).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check that the target path is writable".to_string(),
        example: String::new(),
    })?;

    Ok(())
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

    // Note representability guard (review M1) — reject before locking/writing.
    if let Some(note_text) = note {
        if let Some(reason) = note_representation_issue(note_text) {
            return Err(PaperworkError::Validation {
                message: format!("note is not representable in brief format: {}", reason),
                fix: "start the note with a plain prose line; attribute-shaped '- key: value' first lines and ```regex fence openings are reserved for entry attributes".to_string(),
                example: format!("paperwork brief add {} --entry {} --note \"Reading notes for this file\"", path.display(), entry_path),
            });
        }
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

    let mut manifest = parse_manifest(&content)?;

    // Derive title from entry_path file name
    let title = Path::new(entry_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| entry_path.to_string());

    // Check for duplicate title
    if manifest.entries.iter().any(|e| e.title == title) {
        file.unlock().ok();
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

    let mut manifest = parse_manifest(&content)?;

    let original_len = manifest.entries.len();
    manifest.entries.retain(|e| e.title != title);

    if manifest.entries.len() == original_len {
        file.unlock().ok();
        return Err(PaperworkError::NotFound {
            resource: "Brief entry".to_string(),
            name: title.to_string(),
            fix: "run `paperwork brief read <path>` to see available entries".to_string(),
            example: format!("paperwork brief read {}", path.display()),
        });
    }

    let serialized = serialize_manifest(&manifest);

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

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
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
fn verify_entry(base_dir: &Path, entry: &ManifestEntry) -> Result<VerifyResult> {
    let abs_path = base_dir.join(&entry.path);

    let bytes = match fs::read(&abs_path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(VerifyResult::Stale),
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
