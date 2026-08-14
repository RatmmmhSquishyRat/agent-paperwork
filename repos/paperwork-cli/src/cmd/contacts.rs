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
                            let (name, desc) = enrich_profile(&c.profile_path);
                            serde_json::json!({
                                "label": c.label,
                                "path": c.profile_path,
                                "name": name,
                                "description": desc,
                            })
                        })
                        .collect();
                    let mut obj = serde_json::Map::new();
                    obj.insert("status".to_string(), serde_json::json!("ok"));
                    obj.insert("command".to_string(), serde_json::json!("contacts.read"));
                    obj.insert(
                        "conclusion".to_string(),
                        serde_json::json!(format!("{} contacts", contacts.len())),
                    );
                    obj.insert("contacts".to_string(), serde_json::json!(json_contacts));
                    println!(
                        "{}",
                        serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default()
                    );
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
                            let (name, desc) = enrich_profile(&c.profile_path);
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
fn enrich_profile(profile_path: &str) -> (String, String) {
    let path = std::path::Path::new(profile_path);
    match paperwork_core::ops::profile::show_profile(path) {
        Ok(p) => (p.name, p.description),
        Err(_) => ("(unreadable)".to_string(), String::new()),
    }
}
