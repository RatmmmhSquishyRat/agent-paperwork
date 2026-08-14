//! Contacts commands: create, add, read.

use std::path::{Path, PathBuf};

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
            paperwork_core::ops::contacts::contacts_create(&path, &title)?;

            let env = output::Envelope::new("contacts.create", path.display().to_string())
                .field("path", &path.display().to_string())
                .field("title", &title);
            output::emit_ok(ctx, env);
            Ok(())
        }

        ContactsCommand::Add { path, profile } => {
            let path = ensure_suffix(path, ".contacts.md");
            paperwork_core::ops::contacts::contacts_add(&path, &profile)?;

            let conclusion = format!("{} -> {}", profile, path.display());
            let env = output::Envelope::new("contacts.add", conclusion)
                .field("contacts", &path.display().to_string())
                .field("profile", &profile);
            output::emit_ok(ctx, env);
            Ok(())
        }

        ContactsCommand::Read { path } => {
            let path = ensure_suffix(path, ".contacts.md");
            let contacts = paperwork_core::ops::contacts::contacts_read(&path)?;

            match ctx.mode {
                OutputMode::Json => {
                    let json_contacts: Vec<serde_json::Value> = contacts
                        .iter()
                        .map(|c| {
                            // Try to read profile for enrichment
                            let (name, desc) = enrich_profile(&path, &c.profile_path);
                            serde_json::json!({
                                "label": c.label,
                                "path": c.profile_path,
                                "name": name,
                                "description": desc,
                            })
                        })
                        .collect();
                    let obj = output::JsonBuilder::new()
                        .insert("status", serde_json::json!("ok"))
                        .insert("command", serde_json::json!("contacts.read"))
                        .insert(
                            "conclusion",
                            serde_json::json!(format!("{} contacts", contacts.len())),
                        )
                        .insert("contacts", serde_json::json!(json_contacts));
                    output::print_json(obj.build());
                }
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(&path)?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    let mut env = output::Envelope::new(
                        "contacts.read",
                        format!("{} contacts", contacts.len()),
                    );
                    let body_lines: Vec<String> = contacts
                        .iter()
                        .map(|c| {
                            let (name, desc) = enrich_profile(&path, &c.profile_path);
                            if name == "(unreadable)" {
                                format!("{}: (unreadable)", c.profile_path)
                            } else if desc.is_empty() {
                                format!("{}: {}", c.profile_path, name)
                            } else {
                                format!("{}: {} ({})", c.profile_path, name, desc)
                            }
                        })
                        .collect();
                    env = env.body_lines(body_lines);
                    output::emit_ok(ctx, env);
                }
            }
            Ok(())
        }
    }
}

/// Try to read a profile file and return (name, description).
/// Returns ("(unreadable)", "") if the file cannot be parsed.
///
/// Path resolution goes through core's two-level helper (NEW-4 wiring,
/// spec §7.3 R11): the entry path is tried as given (CWD-relative) first,
/// then relative to the contacts file's own directory — the exact same
/// resolution `derive_label` uses on the write side.
fn enrich_profile(contacts_path: &Path, profile_path: &str) -> (String, String) {
    let path = paperwork_core::ops::contacts::resolve_contact_path(contacts_path, profile_path);
    match paperwork_core::ops::profile::show_profile(&path) {
        Ok(p) => (p.name, p.description),
        Err(_) => ("(unreadable)".to_string(), String::new()),
    }
}
