//! Brief commands: create, add, remove, read, verify.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cmd::Context;
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct BriefArgs {
    #[command(subcommand)]
    command: BriefCommand,
}

#[derive(Subcommand)]
enum BriefCommand {
    /// Create a new brief
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
    Remove {
        /// Path to the brief file
        path: PathBuf,

        /// Title of the entry to remove
        #[arg(long = "entry-title")]
        entry_title: String,
    },

    /// Read a brief
    Read {
        /// Path to the brief file
        path: PathBuf,

        /// Show full entry details (hashes, regex, etc.)
        #[arg(long)]
        full: bool,
    },

    /// Verify all entries in a brief
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
            paperwork_core::ops::manifest::brief_create(
                &path,
                &title,
                owner.as_deref(),
                &description,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "path": path.display().to_string(),
                        "title": title,
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Brief created: {}", path.display())),
            }
            Ok(())
        }

        BriefCommand::Add {
            path,
            entry,
            regex,
            note,
        } => {
            paperwork_core::ops::manifest::brief_add_entry(
                &path,
                &entry,
                regex.as_deref(),
                note.as_deref(),
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "brief": path.display().to_string(),
                        "entry": entry,
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Entry added: {}", entry)),
            }
            Ok(())
        }

        BriefCommand::Remove { path, entry_title } => {
            paperwork_core::ops::manifest::brief_remove_entry(&path, &entry_title)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "brief": path.display().to_string(),
                        "removed": entry_title,
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Entry removed: {}", entry_title)),
            }
            Ok(())
        }

        BriefCommand::Read { path, full } => {
            let manifest = paperwork_core::ops::manifest::brief_read(&path)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => output::print_json(&manifest),
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| anyhow::anyhow!("IO error: {}", e))?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    output::print_default(&format!("# {}", manifest.name));
                    if !manifest.author.is_empty() {
                        output::print_default(&format!("Owner: {}", manifest.author));
                    }
                    if !manifest.description.is_empty() {
                        output::print_default(&manifest.description);
                    }
                    output::print_default(&format!("Entries: {}", manifest.entries.len()));
                    for entry in &manifest.entries {
                        if full {
                            output::print_default(&format!(
                                "  - {} (path: {}, hash: {})",
                                entry.title, entry.path, entry.hash
                            ));
                            if let Some(ref re) = entry.regex {
                                output::print_default(&format!("    regex: {}", re));
                            }
                            if let Some(ref note) = entry.note {
                                output::print_default(&format!("    note: {}", note));
                            }
                        } else {
                            output::print_default(&format!("  - {}", entry.title));
                        }
                    }
                }
            }
            Ok(())
        }

        BriefCommand::Verify { path, base_dir } => {
            let resolved_base = base_dir.unwrap_or_else(|| {
                path.parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."))
            });

            let results = paperwork_core::ops::manifest::brief_verify(&path, &resolved_base)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let json_results: Vec<serde_json::Value> = results
                        .iter()
                        .map(|(entry, result)| {
                            serde_json::json!({
                                "title": entry.title,
                                "path": entry.path,
                                "status": format!("{:?}", result),
                            })
                        })
                        .collect();
                    output::print_json(&json_results);
                }
                _ => {
                    for (entry, result) in &results {
                        let icon = match result {
                            paperwork_core::VerifyResult::Fresh => "\u{2713}",
                            paperwork_core::VerifyResult::Shifted => "~",
                            paperwork_core::VerifyResult::Stale => "\u{2717}",
                        };
                        output::print_default(&format!(
                            "{} {} ({:?})",
                            icon, entry.title, result
                        ));
                    }
                }
            }
            Ok(())
        }
    }
}
