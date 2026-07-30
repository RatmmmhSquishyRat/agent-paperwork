//! `paperwork who` — query scope ownership.

use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// Query scope ownership
#[derive(Args)]
pub struct WhoArgs {
    /// Query by owns scope
    #[arg(long, group = "access")]
    pub owns: Option<String>,

    /// Query by read scope
    #[arg(long, group = "access")]
    pub reads: Option<String>,

    /// Query by write scope
    #[arg(long, group = "access")]
    pub writes: Option<String>,
}

#[derive(Serialize)]
struct WhoOutput {
    query: WhoQuery,
    results: Vec<WhoResult>,
}

#[derive(Serialize)]
struct WhoQuery {
    access: String,
    pattern: String,
}

#[derive(Serialize)]
struct WhoResult {
    name: String,
    profile: String,
    matched_scope: String,
}

pub fn run(ctx: &Context, args: WhoArgs) -> Result<()> {
    let (access, pattern) = if let Some(p) = &args.owns {
        (paperwork_core::Access::Owns, p.clone())
    } else if let Some(p) = &args.reads {
        (paperwork_core::Access::Read, p.clone())
    } else if let Some(p) = &args.writes {
        (paperwork_core::Access::Write, p.clone())
    } else {
        anyhow::bail!(
            "Missing required flag: --owns, --reads, or --writes.\n  \u{2192} usage: paperwork who --owns <glob>"
        );
    };

    let access_str = match access {
        paperwork_core::Access::Owns => "owns",
        paperwork_core::Access::Read => "reads",
        paperwork_core::Access::Write => "writes",
    };

    let profiles = paperwork_core::ops::contacts::who_query(&ctx.root, &pattern, access)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match ctx.mode {
        OutputMode::Json => {
            let results: Vec<WhoResult> = profiles
                .iter()
                .map(|p| {
                    let scope = match access {
                        paperwork_core::Access::Owns => &p.scope_owns,
                        paperwork_core::Access::Read => &p.scope_read,
                        paperwork_core::Access::Write => &p.scope_write,
                    };
                    let matched = scope
                        .iter()
                        .find(|s| *s == &pattern)
                        .cloned()
                        .unwrap_or_else(|| pattern.clone());
                    WhoResult {
                        name: p.name.clone(),
                        profile: format!("profiles/{}.md", p.name),
                        matched_scope: matched,
                    }
                })
                .collect();
            let out = WhoOutput {
                query: WhoQuery {
                    access: access_str.to_string(),
                    pattern: pattern.clone(),
                },
                results,
            };
            output::print_json(&out);
        }
        _ => {
            if profiles.is_empty() {
                output::print_default(&format!(
                    "{} \"{}\": no matches",
                    access_str, pattern
                ));
            } else {
                let mut out = format!("{} \"{}\":\n", access_str, pattern);
                for p in &profiles {
                    let scope = match access {
                        paperwork_core::Access::Owns => &p.scope_owns,
                        paperwork_core::Access::Read => &p.scope_read,
                        paperwork_core::Access::Write => &p.scope_write,
                    };
                    let matched = scope
                        .iter()
                        .find(|s| *s == &pattern)
                        .cloned()
                        .unwrap_or_else(|| pattern.clone());
                    out.push_str(&format!(
                        "  {:<8}profiles/{}.md   scope.{}: `{}`\n",
                        p.name, p.name, access_str, matched
                    ));
                }
                output::print_default(out.trim_end());
            }
        }
    }

    Ok(())
}
