//! Manifest operations: create, add entry, remove entry, verify (3-state).

use std::fs;
use std::path::Path;

use chrono::Utc;
use regex::Regex;

use crate::error::{PaperworkError, Result};
use crate::format::manifest::{extract_regex_groups, parse_manifest, serialize_manifest};
use crate::hash;
use crate::layout;
use crate::{Manifest, ManifestEntry, VerifyResult};

/// Create a new empty manifest.
pub fn create_manifest(root: &Path, name: &str, author: &str, description: &str) -> Result<()> {
    layout::ensure_initialized(root)?;

    let path = layout::manifest_path(root, name);

    if path.exists() {
        return Err(PaperworkError::AlreadyExists {
            resource: "Manifest".to_string(),
            name: name.to_string(),
            hint: "Use `paperwork manifest <name> add` to add entries.".to_string(),
        });
    }

    let manifest = Manifest {
        name: name.to_string(),
        author: author.to_string(),
        created: Utc::now(),
        description: description.to_string(),
        entries: Vec::new(),
    };

    let content = serialize_manifest(&manifest);
    fs::write(&path, content).map_err(|e| PaperworkError::IoContext {
        path: path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Add an entry to a manifest.
///
/// Computes the SHA-256 hash of the file at the given path.
pub fn add_entry(
    root: &Path,
    manifest_name: &str,
    title: &str,
    file_path: &str,
    regex_pattern: Option<&str>,
    note: Option<&str>,
) -> Result<()> {
    layout::ensure_initialized(root)?;

    let manifest_path = layout::manifest_path(root, manifest_name);

    if !manifest_path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Manifest".to_string(),
            name: manifest_name.to_string(),
            hint: format!("Run `paperwork manifest create {}` first.", manifest_name),
        });
    }

    // Read and parse manifest
    let content = fs::read_to_string(&manifest_path).map_err(|e| PaperworkError::IoContext {
        path: manifest_path.clone(),
        source: e,
    })?;

    let mut manifest = parse_manifest(&content)?;

    // Check if entry title already exists
    if manifest.entries.iter().any(|e| e.title == title) {
        return Err(PaperworkError::AlreadyExists {
            resource: "Manifest entry".to_string(),
            name: title.to_string(),
            hint: "Use a different title or remove the existing entry first.".to_string(),
        });
    }

    // Resolve file path relative to workspace root
    let abs_file_path = root.join(file_path);

    // Compute hash
    let file_hash = hash::hash_file(&abs_file_path)?;

    // Extract groups from regex if present
    let groups = regex_pattern
        .map(extract_regex_groups)
        .unwrap_or_default();

    let entry = ManifestEntry {
        title: title.to_string(),
        path: file_path.to_string(),
        hash: file_hash,
        regex: regex_pattern.map(|s| s.to_string()),
        groups,
        note: note.map(|s| s.to_string()),
    };

    manifest.entries.push(entry);

    // Write back
    let serialized = serialize_manifest(&manifest);
    fs::write(&manifest_path, serialized).map_err(|e| PaperworkError::IoContext {
        path: manifest_path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Remove an entry from a manifest by title.
pub fn remove_entry(root: &Path, manifest_name: &str, entry_title: &str) -> Result<()> {
    layout::ensure_initialized(root)?;

    let manifest_path = layout::manifest_path(root, manifest_name);

    if !manifest_path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Manifest".to_string(),
            name: manifest_name.to_string(),
            hint: format!("Run `paperwork manifest create {}` first.", manifest_name),
        });
    }

    // Read and parse manifest
    let content = fs::read_to_string(&manifest_path).map_err(|e| PaperworkError::IoContext {
        path: manifest_path.clone(),
        source: e,
    })?;

    let mut manifest = parse_manifest(&content)?;

    // Find and remove entry
    let original_len = manifest.entries.len();
    manifest.entries.retain(|e| e.title != entry_title);

    if manifest.entries.len() == original_len {
        return Err(PaperworkError::NotFound {
            resource: "Manifest entry".to_string(),
            name: entry_title.to_string(),
            hint: format!(
                "Run `paperwork manifest {} read` to see available entries.",
                manifest_name
            ),
        });
    }

    // Write back
    let serialized = serialize_manifest(&manifest);
    fs::write(&manifest_path, serialized).map_err(|e| PaperworkError::IoContext {
        path: manifest_path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Read a manifest by name.
pub fn read_manifest(root: &Path, manifest_name: &str) -> Result<Manifest> {
    layout::ensure_initialized(root)?;

    let manifest_path = layout::manifest_path(root, manifest_name);

    if !manifest_path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Manifest".to_string(),
            name: manifest_name.to_string(),
            hint: format!("Run `paperwork manifest create {}` first.", manifest_name),
        });
    }

    let content = fs::read_to_string(&manifest_path).map_err(|e| PaperworkError::IoContext {
        path: manifest_path.clone(),
        source: e,
    })?;

    parse_manifest(&content)
}

/// List all manifests.
pub fn list_manifests(root: &Path) -> Result<Vec<String>> {
    layout::ensure_initialized(root)?;

    let manifests_dir = layout::manifests_dir(root);
    let mut names = Vec::new();

    if !manifests_dir.exists() {
        return Ok(names);
    }

    let entries = fs::read_dir(&manifests_dir).map_err(|e| PaperworkError::IoContext {
        path: manifests_dir.clone(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| PaperworkError::IoContext {
            path: manifests_dir.clone(),
            source: e,
        })?;

        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            if let Some(stem) = path.file_stem() {
                names.push(stem.to_string_lossy().to_string());
            }
        }
    }

    names.sort();
    Ok(names)
}

/// Verify all entries in a manifest.
///
/// Three-state verification:
/// - Fresh: regex matches (or no regex) + hash matches
/// - Shifted: regex matches (or no regex) + hash differs
/// - Stale: regex fails to match
pub fn verify_manifest(root: &Path, manifest_name: &str) -> Result<Vec<(ManifestEntry, VerifyResult)>> {
    layout::ensure_initialized(root)?;

    let manifest = read_manifest(root, manifest_name)?;
    let mut results = Vec::new();

    for entry in &manifest.entries {
        let result = verify_entry(root, entry)?;
        results.push((entry.clone(), result));
    }

    Ok(results)
}

/// Verify a single manifest entry.
fn verify_entry(root: &Path, entry: &ManifestEntry) -> Result<VerifyResult> {
    let abs_path = root.join(&entry.path);

    // Read file
    let file_content = match fs::read_to_string(&abs_path) {
        Ok(content) => content,
        Err(_) => {
            // File doesn't exist or can't be read → Stale
            return Ok(VerifyResult::Stale);
        }
    };

    // Check regex if present
    if let Some(ref pattern) = entry.regex {
        match Regex::new(pattern) {
            Ok(re) => {
                if !re.is_match(&file_content) {
                    // Regex fails → Stale
                    return Ok(VerifyResult::Stale);
                }
            }
            Err(_) => {
                // Invalid regex → treat as Stale
                return Ok(VerifyResult::Stale);
            }
        }
    }

    // Compute current hash
    let current_hash = hash::hash_file(&abs_path)?;

    // Compare hashes
    if current_hash == entry.hash {
        Ok(VerifyResult::Fresh)
    } else {
        Ok(VerifyResult::Shifted)
    }
}
