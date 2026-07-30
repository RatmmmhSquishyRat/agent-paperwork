//! Unified error type for paperwork-core.
//!
//! Every error includes the operation context, the path involved, and a suggested fix.

use std::path::PathBuf;
use thiserror::Error;

/// Unified error type for all paperwork operations.
#[derive(Debug, Error)]
pub enum PaperworkError {
    /// Parse error in a managed Markdown file.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Validation error (e.g., seq gap, invalid state).
    #[error("Validation error: {0}")]
    Validation(String),

    /// IO error with context.
    #[error("IO error at '{}': {source}", path.display())]
    IoContext {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Plain IO error (propagated from std).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Resource not found with actionable hint.
    #[error("{resource} '{name}' not found.\n  → {hint}")]
    NotFound {
        resource: String,
        name: String,
        hint: String,
    },

    /// Resource already exists (no overwrite).
    #[error("{resource} '{name}' already exists.\n  → {hint}")]
    AlreadyExists {
        resource: String,
        name: String,
        hint: String,
    },

    /// Operation not permitted in current state.
    #[error("{operation}: {reason}\n  → {hint}")]
    NotAllowed {
        operation: String,
        reason: String,
        hint: String,
    },

    /// Workspace not initialized.
    #[error("Workspace not initialized at '{}'.\n  → Run `paperwork init` first.", path.display())]
    NotInitialized { path: PathBuf },

    /// Message size exceeds limit.
    #[error("Message too large ({size} bytes, max {max} bytes).\n  → Split into smaller messages.")]
    MessageTooLarge { size: usize, max: usize },
}

/// Result type alias for paperwork operations.
pub type Result<T> = std::result::Result<T, PaperworkError>;
