//! Command modules and shared context.

pub mod brief;
pub mod contacts;
pub mod post;
pub mod profile;
pub mod validate;

use std::path::{Path, PathBuf};

use crate::output::OutputMode;

/// Shared context for all commands (stateless -- no workspace root).
pub struct Context {
    /// Output mode.
    pub mode: OutputMode,
    /// Suppress confirmation messages.
    pub quiet: bool,
}

/// Resolve the operation target path via three-stage parsing (v0.5 semantics):
///
/// 1. The given path exists as-is and is a **file** (`is_file()`; directories
///    never match) -> use the original path unchanged.
/// 2. Otherwise, the type-suffixed variant exists as a file -> use it.
/// 3. Neither exists -> return the suffixed variant as the operation landing
///    path.
///
/// This only decides the path. Physical file creation happens exclusively in
/// write commands (send/create/add); read-only commands (read/summary/validate)
/// report not-found when all three stages miss.
pub fn ensure_suffix(path: PathBuf, suffix: &str) -> PathBuf {
    // Stage 1: original path exists as a file -> use as-is.
    if path.is_file() {
        return path;
    }
    // Stage 2/3: suffixed variant is the fallback (existing file wins).
    let suffixed = suffixed_variant(&path, suffix);
    if suffixed.is_file() {
        return suffixed;
    }
    suffixed
}

/// Compute the type-suffixed variant of a path:
/// already ends with the suffix -> unchanged; bare `.md` -> replaced;
/// otherwise -> appended.
fn suffixed_variant(path: &Path, suffix: &str) -> PathBuf {
    let s = path.to_string_lossy();
    if s.ends_with(suffix) {
        return path.to_path_buf();
    }
    if let Some(base) = s.strip_suffix(".md") {
        return PathBuf::from(format!("{}{}", base, suffix));
    }
    PathBuf::from(format!("{}{}", s, suffix))
}
