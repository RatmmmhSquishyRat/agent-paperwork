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
    /// One-line [`PaperworkError::IoContext`] constructor shared by every IO
    /// call site (T3 helper; T4 migrates the ~31 seven-line `map_err`
    /// blocks onto it).
    ///
    /// The fix/example wording is deliberately REQUIRED at every call and
    /// never defaulted: each site's wording is part of the output contract
    /// and pinned by tests, and the sites disagree with each other.
    pub(crate) fn io_ctx(
        path: impl Into<PathBuf>,
        source: std::io::Error,
        fix: impl Into<String>,
        example: impl Into<String>,
    ) -> Self {
        PaperworkError::IoContext {
            path: path.into(),
            source,
            fix: fix.into(),
            example: example.into(),
        }
    }

    /// Return the error category string for the error envelope.
    pub fn category(&self) -> &'static str {
        match self {
            Self::Parse { .. } => "format",
            Self::Validation { .. } => "validation",
            Self::IoContext { .. } => "io",
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
            Self::NotFound { example, .. } => example.clone(),
            Self::AlreadyExists { example, .. } => example.clone(),
            Self::NotAllowed { example, .. } => example.clone(),
            Self::MessageTooLarge { example, .. } => example.clone(),
        }
    }
}

/// Result type alias for paperwork operations.
pub type Result<T> = std::result::Result<T, PaperworkError>;

#[cfg(test)]
mod tests {
    use super::*;

    // T3: the io_ctx helper must build a byte-identical IoContext envelope
    // (path display, fix/example wording carried through untouched).
    #[test]
    fn test_io_ctx_envelope() {
        let err = PaperworkError::io_ctx(
            std::path::Path::new("agents/alice.profile.md"),
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied"),
            "check file permissions",
            "",
        );
        assert_eq!(err.category(), "io");
        assert_eq!(
            err.to_string(),
            "IO error at 'agents/alice.profile.md': access denied"
        );
        assert_eq!(err.fix(), "check file permissions");
        assert_eq!(err.example(), "");
        match &err {
            PaperworkError::IoContext { path, .. } => {
                assert_eq!(path, std::path::Path::new("agents/alice.profile.md"));
            }
            other => panic!("expected IoContext, got {:?}", other),
        }

        // PathBuf inputs and non-empty examples pass through as well
        let err = PaperworkError::io_ctx(
            PathBuf::from("standup.post.md"),
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
            "check that the file exists and is readable",
            "paperwork validate standup.post.md",
        );
        assert_eq!(err.fix(), "check that the file exists and is readable");
        assert_eq!(err.example(), "paperwork validate standup.post.md");
    }
}
