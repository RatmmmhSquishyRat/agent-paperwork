//! Profile commands: create, show, edit, list.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cmd::Context;
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    command: ProfileCommand,
}

#[derive(Subcommand)]
enum ProfileCommand {
    /// Create a new agent profile
    Create {
        /// Path for the new profile file
        path: PathBuf,

        /// Agent name
        #[arg(long)]
        name: String,

        /// Model identifier
        #[arg(long, default_value = "")]
        model: String,

        /// Description of the agent
        #[arg(long, default_value = "")]
        description: String,

        /// Read scope glob patterns
        #[arg(long = "scope-read", num_args = 0..)]
        scope_read: Vec<String>,

        /// Write scope glob patterns
        #[arg(long = "scope-write", num_args = 0..)]
        scope_write: Vec<String>,

        /// Owned scope glob patterns
        #[arg(long = "scope-owns", num_args = 0..)]
        scope_owns: Vec<String>,
    },

    /// Show a profile
    Show {
        /// Path to the profile file
        path: PathBuf,
    },

    /// Edit an existing profile
    Edit {
        /// Path to the profile file
        path: PathBuf,

        /// New model identifier
        #[arg(long)]
        model: Option<String>,

        /// New description
        #[arg(long)]
        description: Option<String>,

        /// New read scope glob patterns
        #[arg(long = "scope-read", num_args = 0..)]
        scope_read: Option<Vec<String>>,

        /// New write scope glob patterns
        #[arg(long = "scope-write", num_args = 0..)]
        scope_write: Option<Vec<String>>,

        /// New owned scope glob patterns
        #[arg(long = "scope-owns", num_args = 0..)]
        scope_owns: Option<Vec<String>>,
    },

    /// List all .md profiles in a directory
    List {
        /// Directory to scan
        dir: PathBuf,
    },
}

pub fn run(ctx: &Context, args: ProfileArgs) -> Result<()> {
    match args.command {
        ProfileCommand::Create {
            path,
            name,
            model,
            description,
            scope_read,
            scope_write,
            scope_owns,
        } => {
            paperwork_core::ops::profile::create_profile(&path, &name, &model, &description)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            // Apply scopes if provided
            if !scope_read.is_empty() || !scope_write.is_empty() || !scope_owns.is_empty() {
                paperwork_core::ops::profile::edit_profile(
                    &path,
                    None,
                    None,
                    if scope_read.is_empty() { None } else { Some(scope_read) },
                    if scope_write.is_empty() { None } else { Some(scope_write) },
                    if scope_owns.is_empty() { None } else { Some(scope_owns) },
                )
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            }

            match ctx.mode {
                OutputMode::Json => {
                    let profile = paperwork_core::ops::profile::show_profile(&path)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    output::print_json(&profile);
                }
                _ => output::success(ctx, &format!("Profile created: {}", path.display())),
            }
            Ok(())
        }

        ProfileCommand::Show { path } => {
            let profile = paperwork_core::ops::profile::show_profile(&path)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => output::print_json(&profile),
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| anyhow::anyhow!("IO error: {}", e))?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    output::print_default(&format!("# {}", profile.name));
                    if !profile.model.is_empty() {
                        output::print_default(&format!("**Model**: {}", profile.model));
                    }
                    if !profile.description.is_empty() {
                        output::print_default(&format!("**Description**: {}", profile.description));
                    }
                    if !profile.scope_read.is_empty() {
                        output::print_default(&format!("**Scope (read)**: {}", profile.scope_read.join(", ")));
                    }
                    if !profile.scope_write.is_empty() {
                        output::print_default(&format!("**Scope (write)**: {}", profile.scope_write.join(", ")));
                    }
                    if !profile.scope_owns.is_empty() {
                        output::print_default(&format!("**Scope (owns)**: {}", profile.scope_owns.join(", ")));
                    }
                }
            }
            Ok(())
        }

        ProfileCommand::Edit {
            path,
            model,
            description,
            scope_read,
            scope_write,
            scope_owns,
        } => {
            paperwork_core::ops::profile::edit_profile(
                &path,
                model.as_deref(),
                description.as_deref(),
                scope_read,
                scope_write,
                scope_owns,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let profile = paperwork_core::ops::profile::show_profile(&path)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    output::print_json(&profile);
                }
                _ => output::success(ctx, &format!("Profile updated: {}", path.display())),
            }
            Ok(())
        }

        ProfileCommand::List { dir } => {
            if !dir.is_dir() {
                anyhow::bail!(
                    "Directory '{}' not found.\n  \u{2192} Provide a valid directory path.",
                    dir.display()
                );
            }

            let mut profiles: Vec<String> = Vec::new();
            let entries = std::fs::read_dir(&dir)
                .map_err(|e| anyhow::anyhow!("IO error at '{}': {}", dir.display(), e))?;

            for entry in entries {
                let entry = entry.map_err(|e| anyhow::anyhow!("IO error: {}", e))?;
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) && path.is_file() {
                    profiles.push(path.display().to_string());
                }
            }
            profiles.sort();

            match ctx.mode {
                OutputMode::Json => output::print_json(&profiles),
                _ => {
                    if profiles.is_empty() {
                        output::print_default("No profiles found.");
                    } else {
                        for p in &profiles {
                            output::print_default(p);
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
