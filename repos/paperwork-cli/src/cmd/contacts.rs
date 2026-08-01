//! Contacts commands: create, add, read.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cmd::{ensure_suffix, Context};
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct ContactsArgs {
    #[command(subcommand)]
    command: ContactsCommand,
}

#[derive(Subcommand)]
enum ContactsCommand {
    /// Create a new contacts file
    Create {
        /// Path for the new contacts file
        path: PathBuf,

        /// Title for the contacts list
        #[arg(long, default_value = "Contacts")]
        title: String,
    },

    /// Add a profile to the contacts file
    Add {
        /// Path to the contacts file
        path: PathBuf,

        /// Path to the profile to add
        #[arg(long)]
        profile: String,
    },

    /// Read all contacts
    Read {
        /// Path to the contacts file
        path: PathBuf,
    },
}

pub fn run(ctx: &Context, args: ContactsArgs) -> Result<()> {
    match args.command {
        ContactsCommand::Create { path, title } => {
            let path = ensure_suffix(path, ".contacts.md");
            paperwork_core::ops::contacts::contacts_create(&path, &title)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "path": path.display().to_string(),
                        "title": title,
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Contacts created: {}", path.display())),
            }
            Ok(())
        }

        ContactsCommand::Add { path, profile } => {
            let path = ensure_suffix(path, ".contacts.md");
            paperwork_core::ops::contacts::contacts_add(&path, &profile)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "contacts": path.display().to_string(),
                        "added": profile,
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Contact added: {}", profile)),
            }
            Ok(())
        }

        ContactsCommand::Read { path } => {
            let path = ensure_suffix(path, ".contacts.md");
            let contacts = paperwork_core::ops::contacts::contacts_read(&path)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => output::print_json(&contacts),
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| anyhow::anyhow!("IO error: {}", e))?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    if contacts.is_empty() {
                        output::print_default("(no contacts)");
                    } else {
                        for contact in &contacts {
                            if contact.summary.is_empty() {
                                output::print_default(&format!("  - {}", contact.profile_path));
                            } else {
                                output::print_default(&format!(
                                    "  - {} ({})",
                                    contact.profile_path, contact.summary
                                ));
                            }
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
