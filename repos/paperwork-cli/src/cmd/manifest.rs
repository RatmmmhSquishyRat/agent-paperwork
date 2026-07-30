//! `paperwork manifest` — create/add/remove/read/verify/list.
//!
//! Command structure:
//! - paperwork manifest create <name> [--description <text>]
//! - paperwork manifest <name> add --path <p> [--regex <r>] [--note <n>]
//! - paperwork manifest <name> remove <entry-title>
//! - paperwork manifest <name> read [--full]
//! - paperwork manifest <name> verify
//! - paperwork manifest list

use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// Manifest operations
#[derive(Args)]
pub struct ManifestArgs {
    /// Manifest name or subcommand (create/list)
    pub name_or_cmd: String,

    /// Action (add/remove/read/verify) or arguments for create
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub rest: Vec<String>,
}

#[derive(Serialize)]
struct VerifyEntryJson {
    title: String,
    path: String,
    verdict: String,
    hash_match: bool,
    regex_match: bool,
}

#[derive(Serialize)]
struct VerifyOutput {
    manifest: String,
    entries: Vec<VerifyEntryJson>,
    summary: VerifySummary,
}

#[derive(Serialize)]
struct VerifySummary {
    fresh: usize,
    shifted: usize,
    stale: usize,
}

#[derive(Serialize)]
struct ManifestReadJson {
    name: String,
    author: String,
    created: String,
    description: String,
    entries: Vec<ManifestEntryJson>,
}

#[derive(Serialize)]
struct ManifestEntryJson {
    title: String,
    path: String,
    hash: String,
    regex: Option<String>,
    note: Option<String>,
}

/// Extract a flag value from args: --flag value or --flag=value
fn get_flag(args: &[String], flag: &str) -> Option<String> {
    let flag_eq = format!("{}=", flag);
    for (i, arg) in args.iter().enumerate() {
        if arg == flag {
            return args.get(i + 1).cloned();
        }
        if let Some(val) = arg.strip_prefix(&flag_eq) {
            return Some(val.to_string());
        }
    }
    None
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

pub fn run(ctx: &Context, args: ManifestArgs) -> Result<()> {
    match args.name_or_cmd.as_str() {
        "create" => run_create(ctx, &args.rest),
        "list" => run_list(ctx),
        name => {
            // It's a manifest name, next arg is the action
            let action = args.rest.first().map(|s| s.as_str()).unwrap_or("");
            let action_args: Vec<String> = args.rest.iter().skip(1).cloned().collect();
            match action {
                "add" => run_add(ctx, name, &action_args),
                "remove" => run_remove(ctx, name, &action_args),
                "read" => run_read(ctx, name, &action_args),
                "verify" => run_verify(ctx, name),
                "" => anyhow::bail!(
                    "Missing action for manifest \"{}\".\n  \u{2192} usage: paperwork manifest {} add|remove|read|verify",
                    name,
                    name
                ),
                other => anyhow::bail!(
                    "Unknown action \"{}\" for manifest.\n  \u{2192} valid actions: add, remove, read, verify",
                    other
                ),
            }
        }
    }
}

fn run_create(ctx: &Context, args: &[String]) -> Result<()> {
    let name = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing manifest name.\n  \u{2192} usage: paperwork manifest create <name>"
            )
        })?;

    let author = ctx.current_agent()?;
    let description = get_flag(args, "--description").unwrap_or_else(|| "\u{2014}".to_string());

    paperwork_core::ops::manifest::create_manifest(&ctx.root, &name, &author, &description)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match ctx.mode {
        OutputMode::Json => {
            let out = serde_json::json!({
                "created": format!("manifests/{}.md", name),
            });
            output::print_json(&out);
        }
        _ => output::success(ctx, &format!("manifest created: manifests/{}.md", name)),
    }

    Ok(())
}

fn run_add(ctx: &Context, name: &str, args: &[String]) -> Result<()> {
    let path = get_flag(args, "--path").ok_or_else(|| {
        anyhow::anyhow!(
            "Missing --path.\n  \u{2192} usage: paperwork manifest {} add --path <file>",
            name
        )
    })?;

    let title = get_flag(args, "--title").unwrap_or_else(|| path.clone());
    let regex = get_flag(args, "--regex");
    let note = get_flag(args, "--note");

    paperwork_core::ops::manifest::add_entry(
        &ctx.root,
        name,
        &title,
        &path,
        regex.as_deref(),
        note.as_deref(),
    )
    .map_err(|e| anyhow::anyhow!("{}", e))?;

    match ctx.mode {
        OutputMode::Json => {
            let out = serde_json::json!({
                "added": title,
                "manifest": format!("manifests/{}.md", name),
            });
            output::print_json(&out);
        }
        _ => output::success(
            ctx,
            &format!("entry added: \"{}\" \u{2192} manifests/{}.md", title, name),
        ),
    }

    Ok(())
}

fn run_remove(ctx: &Context, name: &str, args: &[String]) -> Result<()> {
    let title = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing entry title.\n  \u{2192} usage: paperwork manifest {} remove <title>",
                name
            )
        })?;

    paperwork_core::ops::manifest::remove_entry(&ctx.root, name, &title)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match ctx.mode {
        OutputMode::Json => {
            let out = serde_json::json!({
                "removed": title,
                "manifest": format!("manifests/{}.md", name),
            });
            output::print_json(&out);
        }
        _ => output::success(
            ctx,
            &format!("entry removed: \"{}\" \u{2192} manifests/{}.md", title, name),
        ),
    }

    Ok(())
}

fn run_read(ctx: &Context, name: &str, args: &[String]) -> Result<()> {
    let full = has_flag(args, "--full");

    let manifest = paperwork_core::ops::manifest::read_manifest(&ctx.root, name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match ctx.mode {
        OutputMode::Json => {
            let entries: Vec<ManifestEntryJson> = manifest
                .entries
                .iter()
                .map(|e| ManifestEntryJson {
                    title: e.title.clone(),
                    path: e.path.clone(),
                    hash: e.hash.clone(),
                    regex: e.regex.clone(),
                    note: e.note.clone(),
                })
                .collect();
            let out = ManifestReadJson {
                name: manifest.name,
                author: manifest.author,
                created: manifest.created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                description: manifest.description,
                entries,
            };
            output::print_json(&out);
        }
        OutputMode::Plain => {
            let manifest_path = paperwork_core::layout::manifest_path(&ctx.root, name);
            let content = std::fs::read_to_string(&manifest_path)?;
            output::print_plain(&content);
        }
        OutputMode::Default => {
            let mut out = format!(
                "manifest: {} ({} entries)\n",
                manifest.name,
                manifest.entries.len()
            );
            out.push_str(&format!("author: {}  \n", manifest.author));
            out.push_str(&format!("description: {}\n", manifest.description));
            if !manifest.entries.is_empty() {
                out.push_str("\nentries:\n");
                for e in &manifest.entries {
                    if full {
                        out.push_str(&format!("\n  ### {}\n", e.title));
                        out.push_str(&format!("  path: `{}`\n", e.path));
                        out.push_str(&format!("  hash: `{}`\n", e.hash));
                        if let Some(ref r) = e.regex {
                            out.push_str(&format!("  regex: `{}`\n", r));
                        }
                        if let Some(ref n) = e.note {
                            out.push_str(&format!("  note: {}\n", n));
                        }
                    } else {
                        out.push_str(&format!("  {}  `{}`\n", e.title, e.path));
                    }
                }
            }
            output::print_default(out.trim_end());
        }
    }

    Ok(())
}

fn run_verify(ctx: &Context, name: &str) -> Result<()> {
    let results = paperwork_core::ops::manifest::verify_manifest(&ctx.root, name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let fresh = results
        .iter()
        .filter(|(_, v)| *v == paperwork_core::VerifyResult::Fresh)
        .count();
    let shifted = results
        .iter()
        .filter(|(_, v)| *v == paperwork_core::VerifyResult::Shifted)
        .count();
    let stale = results
        .iter()
        .filter(|(_, v)| *v == paperwork_core::VerifyResult::Stale)
        .count();

    match ctx.mode {
        OutputMode::Json => {
            let entries: Vec<VerifyEntryJson> = results
                .iter()
                .map(|(e, v)| {
                    let (verdict, hash_match, regex_match) = match v {
                        paperwork_core::VerifyResult::Fresh => ("fresh", true, true),
                        paperwork_core::VerifyResult::Shifted => ("shifted", false, true),
                        paperwork_core::VerifyResult::Stale => ("stale", false, false),
                    };
                    VerifyEntryJson {
                        title: e.title.clone(),
                        path: e.path.clone(),
                        verdict: verdict.to_string(),
                        hash_match,
                        regex_match,
                    }
                })
                .collect();
            let out = VerifyOutput {
                manifest: name.to_string(),
                entries,
                summary: VerifySummary {
                    fresh,
                    shifted,
                    stale,
                },
            };
            output::print_json(&out);
        }
        _ => {
            let mut out = format!("manifest: {} ({} entries)\n\n", name, results.len());
            for (entry, verdict) in &results {
                let (label, hash_sym, regex_sym) = match verdict {
                    paperwork_core::VerifyResult::Fresh => ("FRESH  ", "\u{2713}", "\u{2713}"),
                    paperwork_core::VerifyResult::Shifted => ("SHIFTED", "\u{2717}", "\u{2713}"),
                    paperwork_core::VerifyResult::Stale => ("STALE  ", "\u{2717}", "\u{2717}"),
                };
                let regex_part = if entry.regex.is_some() {
                    format!("  regex {}", regex_sym)
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "  {}  {:<24} hash {}{}\n",
                    label, entry.path, hash_sym, regex_part
                ));
            }
            out.push_str(&format!(
                "\nsummary: {} fresh \u{00b7} {} shifted \u{00b7} {} stale",
                fresh, shifted, stale
            ));
            output::print_default(&out);
        }
    }

    Ok(())
}

fn run_list(ctx: &Context) -> Result<()> {
    let manifests = paperwork_core::ops::manifest::list_manifests(&ctx.root)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match ctx.mode {
        OutputMode::Json => {
            output::print_json(&manifests);
        }
        _ => {
            if manifests.is_empty() {
                output::print_default("No manifests found.");
            } else {
                let mut out = String::from("MANIFEST\n");
                for m in &manifests {
                    out.push_str(&format!("{}\n", m));
                }
                output::print_default(out.trim_end());
            }
        }
    }

    Ok(())
}
