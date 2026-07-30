//! Operations layer: filesystem operations composing the format layer.
//!
//! All operations are rooted at a workspace path and use layout.rs for path resolution.

pub mod contacts;
pub mod manifest;
pub mod notify;
pub mod profile;
pub mod thread;

use std::path::Path;

use crate::error::Result;
use crate::layout;
use crate::Profile;

/// Initialize a `.paperwork/` workspace.
///
/// Creates the full directory skeleton, initial profile, and contacts entry.
/// Idempotent: calling on an existing workspace adds the agent if not present.
///
/// # Arguments
/// * `root` - Workspace root directory
/// * `name` - Agent name
/// * `model` - Model identifier (e.g., "gpt-4", "claude-3")
pub fn init(root: &Path, name: &str, model: &str) -> Result<()> {
    // Create skeleton (idempotent)
    layout::create_skeleton(root)?;

    // Create initial profile if it doesn't exist
    let profile = Profile {
        name: name.to_string(),
        model: model.to_string(),
        description: String::new(),
        scope_read: Vec::new(),
        scope_write: Vec::new(),
        scope_owns: Vec::new(),
    };

    // Only create if profile doesn't exist (idempotent)
    let profile_path = layout::profile_path(root, name);
    if !profile_path.exists() {
        profile::create_profile(root, &profile)?;
    }

    Ok(())
}
