//! Profile operations: create, edit, show, list.

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::contacts::{parse_contacts, serialize_contacts};
use crate::format::profile::{parse_profile, serialize_profile};
use crate::layout;
use crate::{ContactEntry, Profile};

/// Create a new profile and update contacts.
///
/// Fails if the profile already exists (no overwrite).
pub fn create_profile(root: &Path, profile: &Profile) -> Result<()> {
    layout::ensure_initialized(root)?;

    let path = layout::profile_path(root, &profile.name);

    // Check if already exists
    if path.exists() {
        return Err(PaperworkError::AlreadyExists {
            resource: "Profile".to_string(),
            name: profile.name.clone(),
            hint: "Use `paperwork profile edit` to modify an existing profile.".to_string(),
        });
    }

    // Write profile file
    let content = serialize_profile(profile);
    fs::write(&path, content).map_err(|e| PaperworkError::IoContext {
        path: path.clone(),
        source: e,
    })?;

    // Update contacts
    add_to_contacts(root, &profile.name)?;

    Ok(())
}

/// Add an agent to the contacts list.
fn add_to_contacts(root: &Path, name: &str) -> Result<()> {
    let contacts_path = layout::contacts_path(root);

    // Read existing contacts
    let content = fs::read_to_string(&contacts_path).map_err(|e| PaperworkError::IoContext {
        path: contacts_path.clone(),
        source: e,
    })?;

    let mut contacts = parse_contacts(&content)?;

    // Check if already in contacts
    if contacts.iter().any(|c| c.agent == name) {
        return Ok(()); // Already present, idempotent
    }

    // Add new entry
    contacts.push(ContactEntry {
        agent: name.to_string(),
        profile_path: format!("profiles/{}.md", name),
    });

    // Sort by agent name for consistent output
    contacts.sort_by(|a, b| a.agent.cmp(&b.agent));

    // Write back
    let serialized = serialize_contacts(&contacts);
    fs::write(&contacts_path, serialized).map_err(|e| PaperworkError::IoContext {
        path: contacts_path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Edit an existing profile's fields.
///
/// Only updates the fields that are Some.
pub fn edit_profile(
    root: &Path,
    name: &str,
    model: Option<&str>,
    description: Option<&str>,
    scope_read: Option<Vec<String>>,
    scope_write: Option<Vec<String>>,
    scope_owns: Option<Vec<String>>,
) -> Result<()> {
    layout::ensure_initialized(root)?;

    let path = layout::profile_path(root, name);

    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Profile".to_string(),
            name: name.to_string(),
            hint: format!(
                "Run `paperwork profile create {}` or `paperwork invite {}` first.",
                name, name
            ),
        });
    }

    // Read existing profile
    let content = fs::read_to_string(&path).map_err(|e| PaperworkError::IoContext {
        path: path.clone(),
        source: e,
    })?;

    let mut profile = parse_profile(&content)?;

    // Update fields
    if let Some(m) = model {
        profile.model = m.to_string();
    }
    if let Some(d) = description {
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

    // Write back
    let serialized = serialize_profile(&profile);
    fs::write(&path, serialized).map_err(|e| PaperworkError::IoContext {
        path: path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Show (read) a profile by name.
pub fn show_profile(root: &Path, name: &str) -> Result<Profile> {
    layout::ensure_initialized(root)?;

    let path = layout::profile_path(root, name);

    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Profile".to_string(),
            name: name.to_string(),
            hint: format!(
                "Run `paperwork profile create {}` or `paperwork invite {}` first.",
                name, name
            ),
        });
    }

    let content = fs::read_to_string(&path).map_err(|e| PaperworkError::IoContext {
        path: path.clone(),
        source: e,
    })?;

    parse_profile(&content)
}

/// List all profiles in the workspace.
pub fn list_profiles(root: &Path) -> Result<Vec<Profile>> {
    layout::ensure_initialized(root)?;

    let profiles_dir = layout::profiles_dir(root);
    let mut profiles = Vec::new();

    let entries = fs::read_dir(&profiles_dir).map_err(|e| PaperworkError::IoContext {
        path: profiles_dir.clone(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| PaperworkError::IoContext {
            path: profiles_dir.clone(),
            source: e,
        })?;

        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            let content = fs::read_to_string(&path).map_err(|e| PaperworkError::IoContext {
                path: path.clone(),
                source: e,
            })?;

            if let Ok(profile) = parse_profile(&content) {
                profiles.push(profile);
            }
        }
    }

    // Sort by name for consistent output
    profiles.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(profiles)
}
