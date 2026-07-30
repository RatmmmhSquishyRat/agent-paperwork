//! `.paperwork/` directory skeleton creation and path resolution helpers.
//!
//! All paths in the operations layer are resolved through this module.
//! No hardcoded paths in ops modules.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PaperworkError, Result};

/// The managed directory name.
pub const PAPERWORK_DIR: &str = ".paperwork";

/// Resolve the `.paperwork/` root directory from a workspace root.
pub fn paperwork_root(root: &Path) -> PathBuf {
    root.join(PAPERWORK_DIR)
}

/// Resolve the profiles directory.
pub fn profiles_dir(root: &Path) -> PathBuf {
    paperwork_root(root).join("profiles")
}

/// Resolve a specific profile file path.
pub fn profile_path(root: &Path, name: &str) -> PathBuf {
    profiles_dir(root).join(format!("{}.md", name))
}

/// Resolve the contacts file path.
pub fn contacts_path(root: &Path) -> PathBuf {
    paperwork_root(root).join("contacts.md")
}

/// Resolve the DM directory.
pub fn dm_dir(root: &Path) -> PathBuf {
    paperwork_root(root).join("dm")
}

/// Resolve a specific DM pair directory.
/// Names are sorted alphabetically and joined with `--` (invariant I5).
pub fn dm_pair_dir(root: &Path, agent_a: &str, agent_b: &str) -> PathBuf {
    let mut names = [agent_a, agent_b];
    names.sort();
    dm_dir(root).join(format!("{}--{}", names[0], names[1]))
}

/// Resolve the thread file for a DM pair.
pub fn dm_thread_path(root: &Path, agent_a: &str, agent_b: &str) -> PathBuf {
    dm_pair_dir(root, agent_a, agent_b).join("thread.md")
}

/// Resolve the meta file for a DM pair.
pub fn dm_meta_path(root: &Path, agent_a: &str, agent_b: &str) -> PathBuf {
    dm_pair_dir(root, agent_a, agent_b).join("meta.md")
}

/// Resolve the posts directory.
pub fn posts_dir(root: &Path) -> PathBuf {
    paperwork_root(root).join("posts")
}

/// Resolve a specific post directory.
pub fn post_dir(root: &Path, name: &str) -> PathBuf {
    posts_dir(root).join(name)
}

/// Resolve the log file for a post.
pub fn post_log_path(root: &Path, name: &str) -> PathBuf {
    post_dir(root, name).join("log.md")
}

/// Resolve the meta file for a post.
pub fn post_meta_path(root: &Path, name: &str) -> PathBuf {
    post_dir(root, name).join("meta.md")
}

/// Resolve the manifests directory.
pub fn manifests_dir(root: &Path) -> PathBuf {
    paperwork_root(root).join("manifests")
}

/// Resolve a specific manifest file path.
pub fn manifest_path(root: &Path, name: &str) -> PathBuf {
    manifests_dir(root).join(format!("{}.md", name))
}

/// Resolve the notifications directory.
pub fn notifications_dir(root: &Path) -> PathBuf {
    paperwork_root(root).join("notifications")
}

/// Resolve a specific agent's notification directory.
pub fn notification_agent_dir(root: &Path, agent: &str) -> PathBuf {
    notifications_dir(root).join(agent)
}

/// Resolve the unread notifications file for an agent.
pub fn unread_path(root: &Path, agent: &str) -> PathBuf {
    notification_agent_dir(root, agent).join("unread.md")
}

/// Resolve the history notifications file for an agent.
pub fn history_path(root: &Path, agent: &str) -> PathBuf {
    notification_agent_dir(root, agent).join("history.md")
}

/// Resolve a thread path (relative to .paperwork/) to an absolute path.
/// Thread paths are like "dm/alice--bob/thread.md" or "posts/general/log.md".
pub fn resolve_thread_path(root: &Path, thread_rel: &str) -> PathBuf {
    paperwork_root(root).join(thread_rel)
}

/// Create the full `.paperwork/` directory skeleton.
///
/// Creates:
/// - `.paperwork/`
/// - `.paperwork/profiles/`
/// - `.paperwork/dm/`
/// - `.paperwork/posts/`
/// - `.paperwork/manifests/`
/// - `.paperwork/notifications/`
/// - `.paperwork/.gitattributes` (enforces LF line endings, invariant I11)
///
/// Idempotent: calling on an existing workspace is a no-op.
pub fn create_skeleton(root: &Path) -> Result<()> {
    let pw_root = paperwork_root(root);

    // Create all directories
    let dirs = [
        pw_root.clone(),
        profiles_dir(root),
        dm_dir(root),
        posts_dir(root),
        manifests_dir(root),
        notifications_dir(root),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir).map_err(|e| PaperworkError::IoContext {
            path: dir.clone(),
            source: e,
        })?;
    }

    // Create .gitattributes for LF enforcement (invariant I11)
    let gitattributes = pw_root.join(".gitattributes");
    if !gitattributes.exists() {
        fs::write(&gitattributes, "* eol=lf\n").map_err(|e| PaperworkError::IoContext {
            path: gitattributes.clone(),
            source: e,
        })?;
    }

    // Create empty contacts.md if it doesn't exist
    let contacts = contacts_path(root);
    if !contacts.exists() {
        let content = "# Contacts\n\n| Agent | Profile |\n|-------|--------|\n";
        fs::write(&contacts, content).map_err(|e| PaperworkError::IoContext {
            path: contacts.clone(),
            source: e,
        })?;
    }

    Ok(())
}

/// Check if a workspace is initialized (has .paperwork/ directory).
pub fn is_initialized(root: &Path) -> bool {
    paperwork_root(root).is_dir()
}

/// Ensure workspace is initialized, returning an error if not.
pub fn ensure_initialized(root: &Path) -> Result<()> {
    if !is_initialized(root) {
        return Err(PaperworkError::NotInitialized {
            path: root.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_resolution() {
        let root = Path::new("/workspace");
        assert_eq!(paperwork_root(root), PathBuf::from("/workspace/.paperwork"));
        assert_eq!(profiles_dir(root), PathBuf::from("/workspace/.paperwork/profiles"));
        assert_eq!(profile_path(root, "alice"), PathBuf::from("/workspace/.paperwork/profiles/alice.md"));
        assert_eq!(contacts_path(root), PathBuf::from("/workspace/.paperwork/contacts.md"));
        assert_eq!(dm_pair_dir(root, "bob", "alice"), PathBuf::from("/workspace/.paperwork/dm/alice--bob"));
        assert_eq!(manifest_path(root, "onboarding"), PathBuf::from("/workspace/.paperwork/manifests/onboarding.md"));
    }

    #[test]
    fn test_dm_pair_sorted() {
        let root = Path::new("/ws");
        assert_eq!(dm_pair_dir(root, "zara", "alice"), PathBuf::from("/ws/.paperwork/dm/alice--zara"));
        assert_eq!(dm_pair_dir(root, "alice", "zara"), PathBuf::from("/ws/.paperwork/dm/alice--zara"));
    }
}
