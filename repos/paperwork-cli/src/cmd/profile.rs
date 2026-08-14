//! Profile commands: create, show, edit, list.
//!
//! v0.6 grammar: PATH is the only positional argument; the agent name is
//! the required `--name` flag on create.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cmd::{ensure_suffix, Context};
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    command: ProfileCommand,
}

/// Command identifier for the output protocol (profile.<verb>).
pub fn command_id(args: &ProfileArgs) -> &'static str {
    match &args.command {
        ProfileCommand::Create { .. } => "profile.create",
        ProfileCommand::Show { .. } => "profile.show",
        ProfileCommand::Edit { .. } => "profile.edit",
        ProfileCommand::List { .. } => "profile.list",
    }
}

#[derive(Subcommand)]
enum ProfileCommand {
    /// Create a new agent profile
    #[command(
        after_help = "Examples:\n  paperwork profile create agents/alice --name alice --model gpt-4o --description \"Parser implementer\"\n  paperwork profile create agents/alice --name alice --scope-read \"src/**\" --scope-write \"src/parser/**\""
    )]
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

    /// List all .profile.md files in a directory
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
            let path = ensure_suffix(path, ".profile.md");
            paperwork_core::ops::profile::create_profile(&path, &name, &model, &description)?;

            // Apply scopes if provided
            if !scope_read.is_empty() || !scope_write.is_empty() || !scope_owns.is_empty() {
                paperwork_core::ops::profile::edit_profile(
                    &path,
                    None,
                    None,
                    if scope_read.is_empty() {
                        None
                    } else {
                        Some(scope_read)
                    },
                    if scope_write.is_empty() {
                        None
                    } else {
                        Some(scope_write)
                    },
                    if scope_owns.is_empty() {
                        None
                    } else {
                        Some(scope_owns)
                    },
                )?;
            }

            let env = output::Envelope::new("profile.create", path.display().to_string())
                .field("path", &path.display().to_string())
                .field("name", &name);
            output::emit_ok(ctx, env);
            Ok(())
        }

        ProfileCommand::Show { path } => {
            let path = ensure_suffix(path, ".profile.md");
            let profile = paperwork_core::ops::profile::show_profile(&path)?;

            match ctx.mode {
                OutputMode::Json => {
                    let obj = output::JsonBuilder::new()
                        .insert("status", serde_json::json!("ok"))
                        .insert("command", serde_json::json!("profile.show"))
                        .insert("conclusion", serde_json::json!(profile.name))
                        .insert("name", serde_json::json!(profile.name))
                        .insert("model", serde_json::json!(profile.model))
                        .insert_opt(
                            "description",
                            (!profile.description.is_empty())
                                .then(|| serde_json::json!(profile.description)),
                        )
                        .insert_opt(
                            "scope.read",
                            (!profile.scope_read.is_empty())
                                .then(|| serde_json::json!(profile.scope_read.join(", "))),
                        )
                        .insert_opt(
                            "scope.write",
                            (!profile.scope_write.is_empty())
                                .then(|| serde_json::json!(profile.scope_write.join(", "))),
                        )
                        .insert_opt(
                            "scope.owns",
                            (!profile.scope_owns.is_empty())
                                .then(|| serde_json::json!(profile.scope_owns.join(", "))),
                        )
                        .build();
                    output::print_json(obj);
                }
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(&path)?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    let mut env = output::Envelope::new("profile.show", profile.name.clone())
                        .field("name", &profile.name)
                        .field("model", &profile.model);
                    if !profile.description.is_empty() {
                        env = env.field("description", &profile.description);
                    }
                    if !profile.scope_read.is_empty() {
                        env = env.field("scope.read", &profile.scope_read.join(", "));
                    }
                    if !profile.scope_write.is_empty() {
                        env = env.field("scope.write", &profile.scope_write.join(", "));
                    }
                    if !profile.scope_owns.is_empty() {
                        env = env.field("scope.owns", &profile.scope_owns.join(", "));
                    }
                    output::emit_ok(ctx, env);
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
            let path = ensure_suffix(path, ".profile.md");

            // Track which fields changed
            let mut changed: Vec<&str> = Vec::new();
            if model.is_some() {
                changed.push("model");
            }
            if description.is_some() {
                changed.push("description");
            }
            if scope_read.is_some() {
                changed.push("scope.read");
            }
            if scope_write.is_some() {
                changed.push("scope.write");
            }
            if scope_owns.is_some() {
                changed.push("scope.owns");
            }

            paperwork_core::ops::profile::edit_profile(
                &path,
                model.as_deref(),
                description.as_deref(),
                scope_read,
                scope_write,
                scope_owns,
            )?;

            let env = output::Envelope::new("profile.edit", path.display().to_string())
                .field("changed", &changed.join(", "));
            output::emit_ok(ctx, env);
            Ok(())
        }

        ProfileCommand::List { dir } => {
            if !dir.is_dir() {
                return Err(paperwork_core::PaperworkError::NotFound {
                    resource: "Directory".to_string(),
                    name: dir.display().to_string(),
                    fix: "provide a valid directory path".to_string(),
                    example: format!("paperwork profile list {}", dir.display()),
                }
                .into());
            }

            let mut profiles: Vec<(String, String, String)> = Vec::new();
            let entries = std::fs::read_dir(&dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                let fname = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                // Only list .profile.md files
                if fname.ends_with(".profile.md") && path.is_file() {
                    // Parse to get name + model
                    match paperwork_core::ops::profile::show_profile(&path) {
                        Ok(p) => {
                            profiles.push((fname, p.name, p.model));
                        }
                        Err(_) => {
                            profiles.push((fname, "(unreadable)".to_string(), String::new()));
                        }
                    }
                }
            }
            profiles.sort_by(|a, b| a.0.cmp(&b.0));

            match ctx.mode {
                OutputMode::Json => {
                    let json_profiles: Vec<serde_json::Value> = profiles
                        .iter()
                        .map(|(fname, name, model)| {
                            output::JsonBuilder::new()
                                .insert("path", serde_json::json!(fname))
                                .insert("name", serde_json::json!(name))
                                .insert("model", serde_json::json!(model))
                                .build()
                        })
                        .collect();
                    let obj = output::JsonBuilder::new()
                        .insert("status", serde_json::json!("ok"))
                        .insert("command", serde_json::json!("profile.list"))
                        .insert(
                            "conclusion",
                            serde_json::json!(format!("{} profiles", profiles.len())),
                        )
                        .insert("profiles", serde_json::json!(json_profiles))
                        .build();
                    output::print_json(obj);
                }
                _ => {
                    let mut env = output::Envelope::new(
                        "profile.list",
                        format!("{} profiles", profiles.len()),
                    );
                    let body_lines: Vec<String> = profiles
                        .iter()
                        .map(|(fname, name, model)| {
                            if model.is_empty() {
                                format!("{}: {}", fname, name)
                            } else {
                                format!("{}: {} ({})", fname, name, model)
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
