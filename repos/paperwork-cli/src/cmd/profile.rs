//! `paperwork profile` — create/edit/show/list profiles.

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// Manage agent profiles
#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommand,
}

#[derive(Subcommand)]
pub enum ProfileCommand {
    /// Create a new profile
    Create {
        /// Agent name
        name: String,
        /// Model identifier
        #[arg(long)]
        model: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Read scope (comma-separated globs)
        #[arg(long)]
        scope_read: Option<String>,
        /// Write scope (comma-separated globs)
        #[arg(long)]
        scope_write: Option<String>,
        /// Owns scope (comma-separated globs)
        #[arg(long)]
        scope_owns: Option<String>,
    },
    /// Edit an existing profile
    Edit {
        /// Agent name
        name: String,
        /// Model identifier
        #[arg(long)]
        model: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Read scope (comma-separated globs)
        #[arg(long)]
        scope_read: Option<String>,
        /// Write scope (comma-separated globs)
        #[arg(long)]
        scope_write: Option<String>,
        /// Owns scope (comma-separated globs)
        #[arg(long)]
        scope_owns: Option<String>,
    },
    /// Show a profile
    Show {
        /// Agent name
        name: String,
    },
    /// List all profiles
    List,
}

#[derive(Serialize)]
struct ProfileJson {
    name: String,
    model: String,
    description: String,
    scope: ScopeJson,
    path: String,
}

#[derive(Serialize)]
struct ScopeJson {
    read: Vec<String>,
    write: Vec<String>,
    owns: Vec<String>,
}

#[derive(Serialize)]
struct ProfileListItem {
    name: String,
    model: String,
    description: String,
    path: String,
}

fn parse_scope(s: Option<String>) -> Option<Vec<String>> {
    s.map(|v| {
        v.split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    })
}

pub fn run(ctx: &Context, args: ProfileArgs) -> Result<()> {
    match args.command {
        ProfileCommand::Create {
            name,
            model,
            description,
            scope_read,
            scope_write,
            scope_owns,
        } => {
            let profile = paperwork_core::Profile {
                name: name.clone(),
                model: model.unwrap_or_else(|| "\u{2014}".to_string()),
                description: description.unwrap_or_default(),
                scope_read: parse_scope(scope_read).unwrap_or_default(),
                scope_write: parse_scope(scope_write).unwrap_or_default(),
                scope_owns: parse_scope(scope_owns).unwrap_or_default(),
            };

            paperwork_core::ops::profile::create_profile(&ctx.root, &profile)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let out = serde_json::json!({
                        "created": format!("profiles/{}.md", name),
                    });
                    output::print_json(&out);
                }
                _ => output::success(ctx, &format!("profile created: profiles/{}.md", name)),
            }
        }
        ProfileCommand::Edit {
            name,
            model,
            description,
            scope_read,
            scope_write,
            scope_owns,
        } => {
            paperwork_core::ops::profile::edit_profile(
                &ctx.root,
                &name,
                model.as_deref(),
                description.as_deref(),
                parse_scope(scope_read),
                parse_scope(scope_write),
                parse_scope(scope_owns),
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let out = serde_json::json!({
                        "edited": format!("profiles/{}.md", name),
                    });
                    output::print_json(&out);
                }
                _ => output::success(ctx, &format!("profile updated: profiles/{}.md", name)),
            }
        }
        ProfileCommand::Show { name } => {
            let profile = paperwork_core::ops::profile::show_profile(&ctx.root, &name)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let path = format!("profiles/{}.md", name);

            match ctx.mode {
                OutputMode::Json => {
                    let out = ProfileJson {
                        name: profile.name,
                        model: profile.model,
                        description: profile.description,
                        scope: ScopeJson {
                            read: profile.scope_read,
                            write: profile.scope_write,
                            owns: profile.scope_owns,
                        },
                        path,
                    };
                    output::print_json(&out);
                }
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(
                        paperwork_core::layout::profile_path(&ctx.root, &name),
                    )?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    let mut out = format!("# {}\n\n", profile.name);
                    out.push_str(&format!("**Model**: {}  \n", profile.model));
                    out.push_str(&format!("**Description**: {}\n\n", profile.description));
                    out.push_str("## Scope\n\n");
                    out.push_str(&format!(
                        "**Read**: {}  \n",
                        format_scope(&profile.scope_read)
                    ));
                    out.push_str(&format!(
                        "**Write**: {}  \n",
                        format_scope(&profile.scope_write)
                    ));
                    out.push_str(&format!(
                        "**Owns**: {}",
                        format_scope(&profile.scope_owns)
                    ));
                    output::print_default(&out);
                }
            }
        }
        ProfileCommand::List => {
            let profiles = paperwork_core::ops::profile::list_profiles(&ctx.root)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let items: Vec<ProfileListItem> = profiles
                        .iter()
                        .map(|p| ProfileListItem {
                            name: p.name.clone(),
                            model: p.model.clone(),
                            description: p.description.clone(),
                            path: format!("profiles/{}.md", p.name),
                        })
                        .collect();
                    output::print_json(&items);
                }
                OutputMode::Plain => {
                    let mut out = String::from("| Agent | Profile |\n|-------|--------|\n");
                    for p in &profiles {
                        out.push_str(&format!("| {} | profiles/{}.md |\n", p.name, p.name));
                    }
                    output::print_plain(&out);
                }
                OutputMode::Default => {
                    if profiles.is_empty() {
                        output::print_default("No profiles found.");
                    } else {
                        let mut out = String::from("AGENT   MODEL           DESCRIPTION\n");
                        for p in &profiles {
                            out.push_str(&format!(
                                "{:<8}{:<16}{}\n",
                                p.name, p.model, p.description
                            ));
                        }
                        output::print_default(out.trim_end());
                    }
                }
            }
        }
    }
    Ok(())
}

fn format_scope(globs: &[String]) -> String {
    if globs.is_empty() {
        "\u{2014}".to_string()
    } else {
        globs
            .iter()
            .map(|g| format!("`{}`", g))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
