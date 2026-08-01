//! Command modules and shared context.

pub mod brief;
pub mod contacts;
pub mod post;
pub mod profile;
pub mod validate;

use std::path::PathBuf;

use crate::output::OutputMode;

/// Shared context for all commands (stateless — no workspace root).
pub struct Context {
    /// Output mode.
    pub mode: OutputMode,
    /// Suppress confirmation messages.
    pub quiet: bool,
}

/// Ensure a path ends with the given suffix (e.g. `.profile.md`).
/// If the path already ends with the suffix, return as-is.
/// Otherwise append the suffix (replacing a bare `.md` if present).
pub fn ensure_suffix(path: PathBuf, suffix: &str) -> PathBuf {
    let s = path.to_string_lossy();
    if s.ends_with(suffix) {
        return path;
    }
    // If it ends with .md, replace that with the suffix
    if let Some(base) = s.strip_suffix(".md") {
        return PathBuf::from(format!("{}{}", base, suffix));
    }
    PathBuf::from(format!("{}{}", s, suffix))
}
