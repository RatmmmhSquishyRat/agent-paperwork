//! Command modules and shared context.

pub mod brief;
pub mod contacts;
pub mod dm;
pub mod notify;
pub mod post;
pub mod profile;

use crate::output::OutputMode;

/// Shared context for all commands (stateless — no workspace root).
pub struct Context {
    /// Output mode.
    pub mode: OutputMode,
    /// Suppress confirmation messages.
    pub quiet: bool,
}
