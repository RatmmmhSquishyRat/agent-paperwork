//! Operations layer: stateless, path-explicit filesystem operations.
//!
//! Every operation takes an explicit file path. No workspace root, no init, no state.
//! Files are independent — no cross-references managed by the CLI.

pub mod contacts;
pub mod manifest;
pub mod notify;
pub mod profile;
pub mod thread;

use std::path::{Path, PathBuf};

/// Compute the DM thread path for a profile and another party.
///
/// Convention: profile at `any/path/alice.md` has DM threads at
/// `any/path/alice.dm/<other_party>.md`.
///
/// # Example
/// ```
/// use std::path::{Path, PathBuf};
/// let profile = Path::new("/foo/alice.md");
/// let dm = paperwork_core::ops::dm_thread_path(profile, "bob");
/// assert_eq!(dm, PathBuf::from("/foo/alice.dm/bob.md"));
/// ```
pub fn dm_thread_path(profile_path: &Path, other_party: &str) -> PathBuf {
    let parent = profile_path.parent().unwrap_or(Path::new("."));
    let stem = profile_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    parent
        .join(format!("{}.dm", stem))
        .join(format!("{}.md", other_party))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dm_thread_path_basic() {
        let profile = Path::new("/foo/alice.md");
        assert_eq!(
            dm_thread_path(profile, "bob"),
            PathBuf::from("/foo/alice.dm/bob.md")
        );
    }

    #[test]
    fn test_dm_thread_path_nested() {
        let profile = Path::new("/a/b/c/agent.md");
        assert_eq!(
            dm_thread_path(profile, "other"),
            PathBuf::from("/a/b/c/agent.dm/other.md")
        );
    }

    #[test]
    fn test_dm_thread_path_relative() {
        let profile = Path::new("alice.md");
        assert_eq!(
            dm_thread_path(profile, "bob"),
            PathBuf::from("alice.dm/bob.md")
        );
    }
}
