//! paperwork-core: Core library for Agent Paperwork.
//!
//! Provides format parsing/serialization and operations for managed Markdown files.

pub mod error;
pub mod format;
pub mod hash;
pub mod ops;

// Re-export error types for convenience
pub use error::{PaperworkError, Result};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Glob pattern for scope declarations.
pub type GlobPattern = String;

/// Agent profile with scope declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub model: String,
    pub description: String,
    pub scope_read: Vec<GlobPattern>,
    pub scope_write: Vec<GlobPattern>,
    pub scope_owns: Vec<GlobPattern>,
}

/// A contact entry: a profile path with an optional summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactEntry {
    pub profile_path: String,
    pub summary: String,
}

/// A message in a post thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub seq: u64,
    pub sender: String,
    pub timestamp: DateTime<Utc>,
    pub to: Vec<String>,
    pub reply_to: Option<u64>,
    pub mentions: Vec<String>,
    pub body: String,
}

/// Summary information about a thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadSummary {
    pub thread_path: String,
    pub message_count: u64,
    pub last_sender: Option<String>,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub snippets: Vec<String>,
}

/// A manifest entry describing a curated file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub title: String,
    pub path: String,
    pub hash: String,
    pub regex: Option<String>,
    pub groups: Vec<String>,
    pub note: Option<String>,
}

/// A complete manifest with metadata and entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub author: String,
    pub created: DateTime<Utc>,
    pub description: String,
    pub entries: Vec<ManifestEntry>,
}

/// Result of verifying a manifest entry against current file state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifyResult {
    /// Regex matches + hash matches (or no regex + hash matches).
    Fresh,
    /// Regex matches + hash differs (or no regex + hash differs).
    Shifted,
    /// Regex fails to match.
    Stale,
}
