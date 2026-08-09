//! Brief commands: create, add, remove, read, verify.
//!
//! v0.6 grammar: PATH is the only positional argument; payloads are
//! required flags -- `--title` (create), `--entry` (add),
//! `--entry-title` (remove).

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cmd::{ensure_suffix, Context};
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct BriefArgs {
    #[command(subcommand)]
    command: BriefCommand,
}

/// Command identifier for the output protocol (brief.<verb>).
pub fn command_id(args: &BriefArgs) -> &'static str {
    match &args.command {
        BriefCommand::Create { .. } => "brief.create",
        BriefCommand::Add { .. } => "brief.add",
        BriefCommand::Remove { .. } => "brief.remove",
        BriefCommand::Read { .. } => "brief.read",
        BriefCommand::Verify { .. } => "brief.verify",
    }
}

#[derive(Subcommand)]
enum BriefCommand {
    /// Create a new brief
    #[command(after_help = "Examples:\n  paperwork brief create onboarding --title \"Codebase Onboarding\" --owner alice")]
    Create {
        /// Path for the new brief file
        path: PathBuf,

        /// Brief title
        #[arg(long)]
        title: String,

        /// Owner name
        #[arg(long)]
        owner: Option<String>,

        /// Description
        #[arg(long, default_value = "")]
        description: String,
    },

    /// Add an entry to a brief
    #[command(after_help = "Examples:\n  paperwork brief add onboarding.brief.md --entry src/main.rs --regex \"fn main\" --note \"Entry point\"")]
    Add {
        /// Path to the brief file
        path: PathBuf,

        /// Path to the entry file (relative to brief's directory)
        #[arg(long)]
        entry: String,

        /// Regex pattern for content extraction
        #[arg(long)]
        regex: Option<String>,

        /// Note for the entry
        #[arg(long)]
        note: Option<String>,
    },

    /// Remove an entry from a brief by title
    #[command(after_help = "Examples:\n  paperwork brief remove onboarding.brief.md --entry-title main.rs")]
    Remove {
        /// Path to the brief file
        path: PathBuf,

        /// Title of the entry to remove (the entry's basename, as stored)
        #[arg(long = "entry-title")]
        entry_title: String,
    },

    /// Read a brief
    #[command(after_help = "Examples:\n  paperwork brief read onboarding.brief.md\n  paperwork brief read onboarding.brief.md --full\n  paperwork brief read onboarding.brief.md --entry-title main.rs")]
    Read {
        /// Path to the brief file
        path: PathBuf,

        /// Show full entry details (hashes, regex, etc.)
        #[arg(long)]
        full: bool,

        /// Show details of a single entry by its stored title
        #[arg(long = "entry-title")]
        entry_title: Option<String>,
    },

    /// Verify all entries in a brief
    #[command(after_help = "Examples:\n  paperwork brief verify onboarding.brief.md")]
    Verify {
        /// Path to the brief file
        path: PathBuf,

        /// Base directory for resolving entry paths (default: brief's parent dir)
        #[arg(long = "base-dir")]
        base_dir: Option<PathBuf>,
    },
}

pub fn run(ctx: &Context, args: BriefArgs) -> Result<()> {
    match args.command {
        BriefCommand::Create {
            path,
            title,
            owner,
            description,
        } => {
            let path = ensure_suffix(path, ".brief.md");
            paperwork_core::ops::manifest::brief_create(
                &path,
                &title,
                owner.as_deref(),
                &description,
            )?;

            let env = output::Envelope::new("brief.create", path.display().to_string())
                .field("path", &path.display().to_string())
                .field("title", &title);
            output::emit_ok(ctx, env);
            Ok(())
        }

        BriefCommand::Add {
            path,
            entry,
            regex,
            note,
        } => {
            let path = ensure_suffix(path, ".brief.md");
            paperwork_core::ops::manifest::brief_add_entry(
                &path,
                &entry,
                regex.as_deref(),
                note.as_deref(),
            )?;

            let conclusion = format!("{} -> {}", entry, path.display());
            let env = output::Envelope::new("brief.add", conclusion)
                .field("brief", &path.display().to_string())
                .field("entry", &entry);
            output::emit_ok(ctx, env);
            Ok(())
        }

        BriefCommand::Remove { path, entry_title } => {
            let path = ensure_suffix(path, ".brief.md");
            paperwork_core::ops::manifest::brief_remove_entry(&path, &entry_title)?;

            let env = output::Envelope::new("brief.remove", entry_title.clone())
                .field("brief", &path.display().to_string())
                .field("removed", &entry_title);
            output::emit_ok(ctx, env);
            Ok(())
        }

        BriefCommand::Read { path, full, entry_title } => {
            let path = ensure_suffix(path, ".brief.md");
            let manifest = paperwork_core::ops::manifest::brief_read(&path)?;

            // Optional single-entry filter (progressive reading, third
            // tier, spec cli-grammar-v0.6 §3.5): a hit is emitted with the
            // --full field set regardless of --full; a miss is not-found.
            // An empty/whitespace-only --entry-title is refused as
            // validation (mirrors the post send empty-value precedent):
            // an empty key is "no key", not a miss.
            if let Some(wanted) = entry_title.as_deref() {
                if wanted.trim().is_empty() {
                    return Err(paperwork_core::PaperworkError::Validation {
                        message: "entry title (--entry-title) is empty".to_string(),
                        fix: "provide a non-empty --entry-title value".to_string(),
                        example: format!("paperwork brief read {} --entry-title main.rs", path.display()),
                    }
                    .into());
                }
            }
            let entries: Vec<&paperwork_core::ManifestEntry> = match entry_title.as_deref() {
                Some(wanted) => {
                    let hits: Vec<&paperwork_core::ManifestEntry> = manifest
                        .entries
                        .iter()
                        .filter(|e| e.title == wanted)
                        .collect();
                    if hits.is_empty() {
                        return Err(paperwork_core::PaperworkError::NotFound {
                            resource: "Brief entry".to_string(),
                            name: wanted.to_string(),
                            fix: format!("run `paperwork brief read {}` to list entries", path.display()),
                            example: format!("paperwork brief read {}", path.display()),
                        }
                        .into());
                    }
                    hits
                }
                None => manifest.entries.iter().collect(),
            };
            let detailed = full || entry_title.is_some();
            let total = manifest.entries.len();

            match ctx.mode {
                OutputMode::Json => {
                    let mut obj = serde_json::Map::new();
                    obj.insert("status".to_string(), serde_json::json!("ok"));
                    obj.insert("command".to_string(), serde_json::json!("brief.read"));
                    obj.insert("conclusion".to_string(), serde_json::json!(format!("{} entries", total)));
                    obj.insert("title".to_string(), serde_json::json!(manifest.name));
                    obj.insert("owner".to_string(), serde_json::json!(manifest.author));
                    let entries_json: Vec<serde_json::Value> = entries.iter().map(|e| {
                        let mut entry_obj = serde_json::Map::new();
                        entry_obj.insert("title".to_string(), serde_json::json!(e.title));
                        entry_obj.insert("path".to_string(), serde_json::json!(e.path));
                        entry_obj.insert("hash".to_string(), serde_json::json!(e.hash));
                        if detailed {
                            if let Some(ref re) = e.regex {
                                entry_obj.insert("regex".to_string(), serde_json::json!(re));
                            }
                            if let Some(ref note) = e.note {
                                entry_obj.insert("note".to_string(), serde_json::json!(note));
                            }
                        }
                        serde_json::Value::Object(entry_obj)
                    }).collect();
                    obj.insert("entries".to_string(), serde_json::json!(entries_json));
                    println!("{}", serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default());
                }
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(&path)?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    let mut env = output::Envelope::new("brief.read", format!("{} entries", total))
                        .field("title", &manifest.name)
                        .field("owner", &manifest.author);
                    let body_lines: Vec<String> = entries.iter().map(|e| {
                        if detailed {
                            let mut line = format!("{}: {} (hash: {})", e.title, e.path, e.hash);
                            if let Some(ref re) = e.regex {
                                line.push_str(&format!(" regex: {}", re));
                            }
                            if let Some(ref note) = e.note {
                                line.push_str(&format!(" note: {}", note));
                            }
                            line
                        } else {
                            format!("{}: {}", e.title, e.path)
                        }
                    }).collect();
                    env = env.body_lines(body_lines);
                    output::emit_ok(ctx, env);
                }
            }
            Ok(())
        }

        BriefCommand::Verify { path, base_dir } => {
            let path = ensure_suffix(path, ".brief.md");
            let resolved_base = base_dir.unwrap_or_else(|| {
                path.parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."))
            });

            let results = paperwork_core::ops::manifest::brief_verify(&path, &resolved_base)?;

            let fresh_count = results.iter().filter(|(_, r)| *r == paperwork_core::VerifyResult::Fresh).count();
            let total = results.len();

            match ctx.mode {
                OutputMode::Json => {
                    let json_results: Vec<serde_json::Value> = results
                        .iter()
                        .map(|(entry, result)| {
                            serde_json::json!({
                                "title": entry.title,
                                "path": entry.path,
                                "status": format!("{:?}", result).to_lowercase(),
                            })
                        })
                        .collect();
                    let mut obj = serde_json::Map::new();
                    obj.insert("status".to_string(), serde_json::json!("ok"));
                    obj.insert("command".to_string(), serde_json::json!("brief.verify"));
                    obj.insert("conclusion".to_string(), serde_json::json!(format!("{}/{} fresh", fresh_count, total)));
                    obj.insert("results".to_string(), serde_json::json!(json_results));
                    println!("{}", serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default());
                }
                _ => {
                    let mut env = output::Envelope::new("brief.verify", format!("{}/{} fresh", fresh_count, total));
                    let body_lines: Vec<String> = results.iter().map(|(entry, result)| {
                        let status = match result {
                            paperwork_core::VerifyResult::Fresh => "fresh",
                            paperwork_core::VerifyResult::Shifted => "shifted",
                            paperwork_core::VerifyResult::Stale => "stale",
                        };
                        format!("{}: {}", entry.title, status)
                    }).collect();
                    env = env.body_lines(body_lines);
                    output::emit_ok(ctx, env);
                }
            }
            Ok(())
        }
    }
}
