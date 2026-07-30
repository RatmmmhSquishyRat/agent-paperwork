//! Contacts operations: invite, list, who-query.

use std::fs;
use std::path::Path;

use chrono::Utc;
use glob::Pattern;

use crate::error::{PaperworkError, Result};
use crate::format::contacts::{parse_contacts, serialize_contacts};
use crate::layout;
use crate::{Access, ContactEntry, Profile};

/// Invite a new agent: create stub profile + DM folder with inviter.
///
/// Creates:
/// - Profile for the invited agent
/// - DM folder between inviter and invitee (alphabetically sorted)
/// - meta.md and empty thread.md in the DM folder
/// - Updates contacts.md
pub fn invite(root: &Path, inviter: &str, invitee: &str, model: &str) -> Result<()> {
    layout::ensure_initialized(root)?;

    // Validate inviter exists
    let inviter_path = layout::profile_path(root, inviter);
    if !inviter_path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Profile".to_string(),
            name: inviter.to_string(),
            hint: format!("Run `paperwork init --name {}` first.", inviter),
        });
    }

    // Check if invitee already exists
    let invitee_path = layout::profile_path(root, invitee);
    if invitee_path.exists() {
        return Err(PaperworkError::AlreadyExists {
            resource: "Profile".to_string(),
            name: invitee.to_string(),
            hint: "Agent is already registered. Use `paperwork contacts` to see all agents."
                .to_string(),
        });
    }

    // Create stub profile for invitee
    let profile = Profile {
        name: invitee.to_string(),
        model: model.to_string(),
        description: String::new(),
        scope_read: Vec::new(),
        scope_write: Vec::new(),
        scope_owns: Vec::new(),
    };

    let content = crate::format::profile::serialize_profile(&profile);
    fs::write(&invitee_path, content).map_err(|e| PaperworkError::IoContext {
        path: invitee_path.clone(),
        source: e,
    })?;

    // Update contacts
    add_to_contacts(root, invitee)?;

    // Create DM folder
    create_dm_folder(root, inviter, invitee)?;

    Ok(())
}

/// Add an agent to the contacts list.
fn add_to_contacts(root: &Path, name: &str) -> Result<()> {
    let contacts_path = layout::contacts_path(root);

    let content = fs::read_to_string(&contacts_path).map_err(|e| PaperworkError::IoContext {
        path: contacts_path.clone(),
        source: e,
    })?;

    let mut contacts = parse_contacts(&content)?;

    // Check if already in contacts
    if contacts.iter().any(|c| c.agent == name) {
        return Ok(());
    }

    contacts.push(ContactEntry {
        agent: name.to_string(),
        profile_path: format!("profiles/{}.md", name),
    });

    contacts.sort_by(|a, b| a.agent.cmp(&b.agent));

    let serialized = serialize_contacts(&contacts);
    fs::write(&contacts_path, serialized).map_err(|e| PaperworkError::IoContext {
        path: contacts_path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Create DM folder with meta.md and empty thread.md.
fn create_dm_folder(root: &Path, agent_a: &str, agent_b: &str) -> Result<()> {
    let dm_dir = layout::dm_pair_dir(root, agent_a, agent_b);

    fs::create_dir_all(&dm_dir).map_err(|e| PaperworkError::IoContext {
        path: dm_dir.clone(),
        source: e,
    })?;

    // Create meta.md
    let meta_path = layout::dm_meta_path(root, agent_a, agent_b);
    let mut names = [agent_a, agent_b];
    names.sort();
    let meta_content = format!(
        "# DM: {} ↔ {}\n\n**Created**: {}  \n**Participants**: {}, {}\n",
        names[0],
        names[1],
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        names[0],
        names[1]
    );
    fs::write(&meta_path, meta_content).map_err(|e| PaperworkError::IoContext {
        path: meta_path.clone(),
        source: e,
    })?;

    // Create empty thread.md
    let thread_path = layout::dm_thread_path(root, agent_a, agent_b);
    fs::write(&thread_path, "").map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    Ok(())
}

/// List all contacts.
pub fn contacts_list(root: &Path) -> Result<Vec<ContactEntry>> {
    layout::ensure_initialized(root)?;

    let contacts_path = layout::contacts_path(root);
    let content = fs::read_to_string(&contacts_path).map_err(|e| PaperworkError::IoContext {
        path: contacts_path.clone(),
        source: e,
    })?;

    parse_contacts(&content)
}

/// Query who has a specific access level to paths matching a pattern.
///
/// Scans all profiles and matches glob patterns against the query path.
pub fn who_query(root: &Path, query_path: &str, access: Access) -> Result<Vec<Profile>> {
    layout::ensure_initialized(root)?;

    let profiles = super::profile::list_profiles(root)?;
    let mut matches = Vec::new();

    for profile in profiles {
        let scope = match access {
            Access::Owns => &profile.scope_owns,
            Access::Read => &profile.scope_read,
            Access::Write => &profile.scope_write,
        };

        // Check if any pattern in scope matches the query path
        for pattern in scope {
            if glob_matches(pattern, query_path) {
                matches.push(profile.clone());
                break;
            }
        }
    }

    Ok(matches)
}

/// Check if a glob pattern matches a path.
///
/// Uses the glob crate's Pattern::matches() for pattern-vs-path comparison.
fn glob_matches(pattern: &str, path: &str) -> bool {
    // Try to compile the pattern
    if let Ok(pat) = Pattern::new(pattern) {
        // Use matches_with with require_literal_separator = false
        // This allows ** to match across path separators
        pat.matches_with(path, glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        })
    } else {
        // If pattern is invalid, try exact match
        pattern == path
    }
}
