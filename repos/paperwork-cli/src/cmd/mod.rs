//! Command modules and shared context.

pub mod contacts;
pub mod dm;
pub mod init;
pub mod invite;
pub mod manifest;
pub mod notify;
pub mod post;
pub mod profile;
pub mod who;

use std::fs;
use std::path::PathBuf;

use anyhow::{Context as _, Result};

use crate::output::OutputMode;

/// Shared context for all commands.
pub struct Context {
    /// Workspace root directory.
    pub root: PathBuf,
    /// Output mode.
    pub mode: OutputMode,
    /// Suppress confirmation messages.
    pub quiet: bool,
}

impl Context {
    /// Get the current agent name from .paperwork/.agent file.
    pub fn current_agent(&self) -> Result<String> {
        let agent_file = paperwork_core::layout::paperwork_root(&self.root).join(".agent");
        if !agent_file.exists() {
            anyhow::bail!(
                "No agent set.\n  \u{2192} Run `paperwork init --name <agent>` first."
            );
        }
        let name = fs::read_to_string(&agent_file)
            .context("Failed to read .agent file")?
            .trim()
            .to_string();
        if name.is_empty() {
            anyhow::bail!(
                "No agent set.\n  \u{2192} Run `paperwork init --name <agent>` first."
            );
        }
        Ok(name)
    }

    /// Resolve DM thread relative path between current agent and another.
    pub fn dm_thread_rel(&self, other: &str) -> Result<String> {
        let me = self.current_agent()?;
        let mut names = [me.as_str(), other];
        names.sort();
        Ok(format!("dm/{}--{}/thread.md", names[0], names[1]))
    }
}
