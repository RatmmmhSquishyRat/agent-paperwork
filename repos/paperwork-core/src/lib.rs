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

/// A contact entry: a Markdown link to a profile file.
///
/// `label` is the link text (profile name); `profile_path` is the link destination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactEntry {
    pub label: String,
    pub profile_path: String,
}

/// Thread preamble metadata (title only).
///
/// Owner ruling D1: the preamble is reduced to the H1 title — the
/// `participants` attribute line is abolished; participant lists are derived
/// from message senders when needed (spec §5.2/§5.4). Parse-only view of the
/// preamble: `thread_edit` carries the preamble bytes verbatim and never
/// re-serializes through this type.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ThreadMeta {
    pub title: String,
}

/// A message in a post thread.
///
/// Owner ruling D2: message attribute lines (`- reply-to:` / `- mentions:` /
/// `- to:`) are abolished; the `to` field is deleted entirely. `reply_to` and
/// `mentions` are parse-time derivations from the body text (`@#N` / `@name`
/// tokens, spec §5.4) and are never serialized back to disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub seq: u64,
    pub sender: String,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<u64>,
    pub mentions: Vec<String>,
    pub body: String,
}

/// Summary information about a thread.
///
/// `title` is the preamble H1 captured in the same parse pass (review M8:
/// callers no longer need a second `thread_meta` walk over the file).
/// `participants` is derived from the set of message senders, deduplicated
/// in first-appearance order (spec §5.4, D1); it is never stored on disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadSummary {
    pub thread_path: String,
    pub title: String,
    pub message_count: u64,
    pub participants: Vec<String>,
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
