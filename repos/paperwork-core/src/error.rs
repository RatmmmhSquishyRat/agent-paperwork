//! Unified error type for paperwork-core.
//!
//! Every error includes the operation context, the path involved, a suggested fix,
//! and an example corrected command.

use std::path::PathBuf;
use thiserror::Error;

/// Unified error type for all paperwork operations.
#[derive(Debug, Error)]
pub enum PaperworkError {
    /// Parse error in a managed Markdown file.
    #[error("Parse error: {message}")]
    Parse {
        message: String,
        fix: String,
        example: String,
    },

    /// Validation error (e.g., seq gap, invalid state).
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        fix: String,
        example: String,
    },

    /// IO error with context.
    #[error("IO error at '{}': {source}", path.display())]
    IoContext {
        path: PathBuf,
        #[source]
        source: std::io::Error,
        fix: String,
        example: String,
    },

    /// Plain IO error (propagated from std).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Resource not found with actionable hint.
    #[error("{resource} '{name}' not found")]
    NotFound {
        resource: String,
        name: String,
        fix: String,
        example: String,
    },

    /// Resource already exists (no overwrite).
    #[error("{resource} '{name}' already exists")]
    AlreadyExists {
        resource: String,
        name: String,
        fix: String,
        example: String,
    },

    /// Operation not permitted in current state.
    #[error("{operation}: {reason}")]
    NotAllowed {
        operation: String,
        reason: String,
        fix: String,
        example: String,
    },

    /// Message size exceeds limit.
    #[error("Message too large ({size} bytes, max {max} bytes)")]
    MessageTooLarge {
        size: usize,
        max: usize,
        fix: String,
        example: String,
    },
}

impl PaperworkError {
    /// Return the error category string for the error envelope.
    pub fn category(&self) -> &'static str {
        match self {
            Self::Parse { .. } => "format",
            Self::Validation { .. } => "validation",
            Self::IoContext { .. } | Self::Io(_) => "io",
            Self::NotFound { .. } => "not-found",
            Self::AlreadyExists { .. } => "already-exists",
            Self::NotAllowed { .. } => "not-allowed",
            Self::MessageTooLarge { .. } => "validation",
        }
    }

    /// Return the fix suggestion.
    pub fn fix(&self) -> String {
        match self {
            Self::Parse { fix, .. } => fix.clone(),
            Self::Validation { fix, .. } => fix.clone(),
            Self::IoContext { fix, .. } => fix.clone(),
            Self::Io(e) => format!("check file permissions and disk space ({})", e),
            Self::NotFound { fix, .. } => fix.clone(),
            Self::AlreadyExists { fix, .. } => fix.clone(),
            Self::NotAllowed { fix, .. } => fix.clone(),
            Self::MessageTooLarge { fix, .. } => fix.clone(),
        }
    }

    /// Return the example corrected command.
    pub fn example(&self) -> String {
        match self {
            Self::Parse { example, .. } => example.clone(),
            Self::Validation { example, .. } => example.clone(),
            Self::IoContext { example, .. } => example.clone(),
            Self::Io(_) => String::new(),
            Self::NotFound { example, .. } => example.clone(),
            Self::AlreadyExists { example, .. } => example.clone(),
            Self::NotAllowed { example, .. } => example.clone(),
            Self::MessageTooLarge { example, .. } => example.clone(),
        }
    }
}

/// Result type alias for paperwork operations.
pub type Result<T> = std::result::Result<T, PaperworkError>;
