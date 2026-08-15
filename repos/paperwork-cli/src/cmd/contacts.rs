//! Contacts commands: create, add, remove, update, read.
//!
//! v0.6 grammar: PATH is the only positional argument; the profile to
//! add/remove is the required `--profile` flag; update additionally takes
//! the required `--new-profile` flag. All new flags are long-form only.

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

/// Command identifier for the output protocol (`contacts.<verb>`).
pub fn command_id(args: &ContactsArgs) -> &'static str {
    match &args.command {
        ContactsCommand::Create { .. } => "contacts.create",
        ContactsCommand::Add { .. } => "contacts.add",
        ContactsCommand::Remove { .. } => "contacts.remove",
        ContactsCommand::Update { .. } => "contacts.update",
        ContactsCommand::Read { .. } => "contacts.read",
    }
}

#[derive(Subcommand)]
enum ContactsCommand {
    /// Create a new contacts file
    #[command(
        after_help = "Examples:\n  paperwork contacts create team --title \"Core Team\"\n\nNote: --title is an OPTIONAL flag here (default \"Contacts\"); it is required on brief create."
    )]
    Create {
        /// Path for the new contacts file
        path: PathBuf,

        /// Title for the contacts list
        #[arg(long, default_value = "Contacts")]
        title: String,
    },

    /// Add a profile to the contacts file
    #[command(
        after_help = "Examples:\n  paperwork contacts add team.contacts.md --profile agents/alice.profile.md"
    )]
    Add {
        /// Path to the contacts file
        path: PathBuf,

        /// Path to the profile to add
        #[arg(long)]
        profile: String,
    },

    /// Remove a profile from the contacts file (key = stored profile path)
    #[command(
        after_help = "Examples:\n  paperwork contacts remove team.contacts.md --profile alice.profile.md\n\nNote: the key is the profile path as stored in the contacts file, not the label."
    )]
    Remove {
        /// Path to the contacts file
        path: PathBuf,

        /// Profile path of the entry to remove (exactly as stored)
        #[arg(long)]
        profile: String,
    },

    /// Re-bind an entry to a new profile path (contacts has no edit verb)
    #[command(
        after_help = "Examples:\n  paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md\n\nNote: update re-binds an entry destination; edit (in-place content change) does not exist in this group."
    )]
    Update {
        /// Path to the contacts file
        path: PathBuf,

        /// Profile path of the entry to re-bind (exactly as stored)
        #[arg(long)]
        profile: String,

        /// New profile path for the entry
        #[arg(long = "new-profile")]
        new_profile: String,
    },

    /// Read all contacts
    #[command(after_help = "Examples:\n  paperwork contacts read team.contacts.md")]
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
            let mut env = output::Envelope::new("contacts.add", conclusion)
                .field("contacts", &path.display().to_string())
                .field("profile", &profile);
            // Non-blocking destination advisory (2026-08-15 owner ruling,
            // spec §3.6): probe AFTER the write succeeded; never changes the
            // exit code or the write outcome (add-only, output protocol).
            if let Some(note) = destination_advisory(&path, &profile) {
                env = env.field("advisory", &note);
            }
            output::emit_ok(ctx, env);
            Ok(())
        }

        ContactsCommand::Remove { path, profile } => {
            let path = ensure_suffix(path, ".contacts.md");
            paperwork_core::ops::contacts::contacts_remove(&path, &profile)?;

            let conclusion = format!("{} -> {}", profile, path.display());
            let env = output::Envelope::new("contacts.remove", conclusion)
                .field("contacts", &path.display().to_string())
                .field("removed", &profile);
            output::emit_ok(ctx, env);
            Ok(())
        }

        ContactsCommand::Update {
            path,
            profile,
            new_profile,
        } => {
            let path = ensure_suffix(path, ".contacts.md");
            paperwork_core::ops::contacts::contacts_update(&path, &profile, &new_profile)?;

            let conclusion = format!("{} -> {}", profile, new_profile);
            let mut env = output::Envelope::new("contacts.update", conclusion)
                .field("contacts", &path.display().to_string())
                .field("updated", &format!("{} -> {}", profile, new_profile));
            // Non-blocking destination advisory (2026-08-15 owner ruling,
            // spec §3.6): the destination is the NEW profile path; probe
            // AFTER the write succeeded, never changes the exit code.
            if let Some(note) = destination_advisory(&path, &new_profile) {
                env = env.field("advisory", &note);
            }
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
                            output::JsonBuilder::new()
                                .insert("label", serde_json::json!(c.label))
                                .insert("path", serde_json::json!(c.profile_path))
                                .insert("name", serde_json::json!(name))
                                .insert("description", serde_json::json!(desc))
                                .build()
                        })
                        .collect();
                    let obj = output::JsonBuilder::new()
                        .insert("status", serde_json::json!("ok"))
                        .insert("command", serde_json::json!("contacts.read"))
                        .insert(
                            "conclusion",
                            serde_json::json!(format!("{} contacts", contacts.len())),
                        )
                        .insert("contacts", serde_json::json!(json_contacts))
                        .build();
                    output::print_json(obj);
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

/// Non-blocking destination advisory (2026-08-15 owner ruling, spec §3.6).
///
/// Runs AFTER the write succeeded: a cheap, read-only probe of the
/// destination profile path. Never changes the exit code and introduces no
/// new write-failure path — any probe failure simply yields the matching
/// advisory text. Resolution matches the read side (`resolve_contact_path`,
/// spec §7.3 R11): as given (CWD-relative) first, then relative to the
/// contacts file's own directory.
///
/// Wording frozen 2026-08-15 (task #36; spec §3.6 回冻): three forms —
/// `does not exist` / `is not readable` / `is not a valid profile file`.
/// The wording TEMPLATE is always pure ASCII; the destination is echoed
/// back verbatim as given (same weight as the conclusion/profile fields),
/// so the whole line is ASCII only when the destination path is ASCII
/// (Ray S-1, wording-scope narrowing 2026-08-15).
///
/// Non-UTF-8 destination content fails `read_to_string` (InvalidData) and
/// therefore falls into the SECOND probe level "is not readable" — by
/// probe order, readable is defined as decodable to a string (Ray S-2).
fn destination_advisory(contacts_path: &std::path::Path, destination: &str) -> Option<String> {
    let resolved = paperwork_core::ops::contacts::resolve_contact_path(contacts_path, destination);
    if !resolved.exists() {
        return Some(format!("destination '{}' does not exist", destination));
    }
    let content = match std::fs::read_to_string(&resolved) {
        Ok(c) => c,
        Err(_) => return Some(format!("destination '{}' is not readable", destination)),
    };
    if paperwork_core::format::profile::parse_profile(&content).is_err() {
        return Some(format!(
            "destination '{}' is not a valid profile file",
            destination
        ));
    }
    None
}

/// Try to read a profile file and return (name, description).
/// Returns ("(unreadable)", "") if the file cannot be parsed.
///
/// NEW-4 (P-6): resolution matches the write side (`derive_label`, spec §7.3
/// R11): the entry path is tried as given (CWD-relative) first, then relative
/// to the contacts file's own directory.
fn enrich_profile(contacts_path: &std::path::Path, profile_path: &str) -> (String, String) {
    let path = paperwork_core::ops::contacts::resolve_contact_path(contacts_path, profile_path);
    match paperwork_core::ops::profile::show_profile(&path) {
        Ok(p) => (p.name, p.description),
        Err(_) => ("(unreadable)".to_string(), String::new()),
    }
}
