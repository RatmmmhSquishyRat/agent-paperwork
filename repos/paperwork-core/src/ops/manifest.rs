//! Brief operations (formerly "manifest"): create, add/remove entry, read, verify.
//!
//! A brief is a standalone reading list / knowledge brief.
//! All operations take explicit file paths — no workspace root.

use std::fs;
use std::path::Path;

use chrono::Utc;
use regex::Regex;

use crate::error::{PaperworkError, Result};
use crate::format::manifest::{extract_regex_groups, parse_manifest, serialize_manifest};
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
            example: format!("paperwork brief add {} src/main.rs", path.display()),
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
            example: format!("paperwork brief create {} \"My Brief\"", path.display()),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
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
        return Err(PaperworkError::AlreadyExists {
            resource: "Brief entry".to_string(),
            name: title,
            fix: "use a different entry path or remove the existing entry first".to_string(),
            example: format!("paperwork brief remove {} main.rs", path.display()),
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
    fs::write(path, serialized).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check that the target path is writable".to_string(),
        example: String::new(),
    })?;

    Ok(())
}

/// Remove an entry from a brief by title.
pub fn brief_remove_entry(path: &Path, title: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Brief".to_string(),
            name: path.display().to_string(),
            fix: "run `paperwork brief create <path>` first".to_string(),
            example: format!("paperwork brief create {} \"My Brief\"", path.display()),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;

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
    fs::write(path, serialized).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check that the target path is writable".to_string(),
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
            example: format!("paperwork brief create {} \"My Brief\"", path.display()),
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
fn verify_entry(base_dir: &Path, entry: &ManifestEntry) -> Result<VerifyResult> {
    let abs_path = base_dir.join(&entry.path);

    let file_content = match fs::read_to_string(&abs_path) {
        Ok(content) => content,
        Err(_) => return Ok(VerifyResult::Stale),
    };

    // Check regex if present
    if let Some(ref pattern) = entry.regex {
        match Regex::new(pattern) {
            Ok(re) => {
                if !re.is_match(&file_content) {
                    return Ok(VerifyResult::Stale);
                }
            }
            Err(_) => return Ok(VerifyResult::Stale),
        }
    }

    // Compute current hash
    let current_hash = hash::hash_file(&abs_path)?;

    if current_hash == entry.hash {
        Ok(VerifyResult::Fresh)
    } else {
        Ok(VerifyResult::Shifted)
    }
}
