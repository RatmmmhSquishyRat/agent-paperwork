//! `paperwork init` — workspace initialization.

use std::fs;

use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// Initialize a .paperwork/ workspace
#[derive(Args)]
pub struct InitArgs {
    /// Agent name (required)
    #[arg(long)]
    pub name: String,

    /// Model identifier
    #[arg(long, default_value = "\u{2014}")]
    pub model: String,
}

#[derive(Serialize)]
struct InitOutput {
    created: String,
    profile: String,
}

pub fn run(ctx: &Context, args: InitArgs) -> Result<()> {
    let already_exists = paperwork_core::layout::is_initialized(&ctx.root);

    // Call core init
    paperwork_core::ops::init(&ctx.root, &args.name, &args.model)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Store current agent name
    let agent_file = paperwork_core::layout::paperwork_root(&ctx.root).join(".agent");
    fs::write(&agent_file, &args.name)?;

    // Create notification directory for this agent
    let notif_dir = paperwork_core::layout::notification_agent_dir(&ctx.root, &args.name);
    fs::create_dir_all(&notif_dir)?;
    let unread = paperwork_core::layout::unread_path(&ctx.root, &args.name);
    if !unread.exists() {
        fs::write(&unread, format!("# Notifications: {}\n", args.name))?;
    }
    let history = paperwork_core::layout::history_path(&ctx.root, &args.name);
    if !history.exists() {
        fs::write(&history, format!("# Notifications: {}\n", args.name))?;
    }

    let profile_rel = format!("profiles/{}.md", args.name);

    match ctx.mode {
        OutputMode::Json => {
            let out = InitOutput {
                created: ".paperwork/".to_string(),
                profile: profile_rel,
            };
            output::print_json(&out);
        }
        _ => {
            if already_exists {
                output::success(ctx, "already initialized");
            } else {
                output::success(ctx, &format!("initialized .paperwork/ \u{2192} {}", profile_rel));
            }
        }
    }

    Ok(())
}
