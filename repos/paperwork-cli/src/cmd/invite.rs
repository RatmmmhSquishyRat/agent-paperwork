//! `paperwork invite` — invite an agent to the workspace.

use std::fs;

use anyhow::Result;
use clap::Args;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// Invite an agent to the workspace
#[derive(Args)]
pub struct InviteArgs {
    /// Agent name to invite
    pub name: String,

    /// Model identifier for the invited agent
    #[arg(long, default_value = "\u{2014}")]
    pub model: String,
}

pub fn run(ctx: &Context, args: InviteArgs) -> Result<()> {
    let inviter = ctx.current_agent()?;

    // Check if already invited (idempotent)
    let invitee_path = paperwork_core::layout::profile_path(&ctx.root, &args.name);
    if invitee_path.exists() {
        match ctx.mode {
            OutputMode::Json => {
                let mut names = [inviter.as_str(), args.name.as_str()];
                names.sort();
                let dm_folder = format!("dm/{}--{}/", names[0], names[1]);
                let out = serde_json::json!({
                    "invited": args.name,
                    "dm_folder": dm_folder,
                    "status": "already_invited"
                });
                output::print_json(&out);
            }
            _ => output::success(ctx, &format!("{} already invited", args.name)),
        }
        return Ok(());
    }

    paperwork_core::ops::contacts::invite(&ctx.root, &inviter, &args.name, &args.model)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Create notification directory for invited agent
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

    let mut names = [inviter.as_str(), args.name.as_str()];
    names.sort();
    let dm_folder = format!("dm/{}--{}/", names[0], names[1]);

    match ctx.mode {
        OutputMode::Json => {
            let out = serde_json::json!({
                "invited": args.name,
                "dm_folder": dm_folder,
            });
            output::print_json(&out);
        }
        _ => output::success(ctx, &format!("invited {} \u{2192} {}", args.name, dm_folder)),
    }

    Ok(())
}
