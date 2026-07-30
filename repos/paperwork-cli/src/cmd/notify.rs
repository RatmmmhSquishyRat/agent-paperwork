//! `paperwork notify` — view/acknowledge notifications.

use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// Notification operations
#[derive(Args)]
pub struct NotifyArgs {
    /// Target agent (defaults to current agent)
    #[arg(long)]
    pub agent: Option<String>,

    /// Acknowledge all unread notifications
    #[arg(long)]
    pub ack: bool,
}

#[derive(Serialize)]
struct NotifyOutput {
    agent: String,
    unread_count: usize,
    notifications: Vec<NotifyItemJson>,
}

#[derive(Serialize)]
struct NotifyItemJson {
    timestamp: String,
    from: String,
    #[serde(rename = "type")]
    notify_type: String,
    thread: String,
    seq: u64,
    snippet: String,
}

pub fn run(ctx: &Context, args: NotifyArgs) -> Result<()> {
    let agent = match &args.agent {
        Some(a) => a.clone(),
        None => ctx.current_agent()?,
    };

    if args.ack {
        let acked = paperwork_core::ops::notify::ack_notify(&ctx.root, &agent)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        match ctx.mode {
            OutputMode::Json => {
                let out = serde_json::json!({
                    "acknowledged": acked.len(),
                    "agent": agent,
                });
                output::print_json(&out);
            }
            _ => {
                if acked.is_empty() {
                    output::success(ctx, "no unread notifications");
                } else {
                    output::success(
                        ctx,
                        &format!(
                            "{} notifications acknowledged \u{2192} notifications/{}/history.md",
                            acked.len(),
                            agent
                        ),
                    );
                }
            }
        }
    } else {
        let notifications = paperwork_core::ops::notify::list_unread(&ctx.root, &agent)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        match ctx.mode {
            OutputMode::Json => {
                let items: Vec<NotifyItemJson> = notifications
                    .iter()
                    .map(|n| NotifyItemJson {
                        timestamp: n.timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        from: n.from.clone(),
                        notify_type: match n.notify_type {
                            paperwork_core::NotifyType::Mention => "mention".to_string(),
                            paperwork_core::NotifyType::Reply => "reply".to_string(),
                        },
                        thread: n.thread_path.clone(),
                        seq: n.seq,
                        snippet: n.snippet.clone(),
                    })
                    .collect();
                let out = NotifyOutput {
                    agent: agent.clone(),
                    unread_count: notifications.len(),
                    notifications: items,
                };
                output::print_json(&out);
            }
            _ => {
                if notifications.is_empty() {
                    output::print_default(&format!(
                        "notifications for {} \u{2014} 0 unread",
                        agent
                    ));
                } else {
                    let mut out = format!(
                        "notifications for {} \u{2014} {} unread\n",
                        agent,
                        notifications.len()
                    );
                    for n in &notifications {
                        let type_str = match n.notify_type {
                            paperwork_core::NotifyType::Mention => "mention",
                            paperwork_core::NotifyType::Reply => "reply",
                        };
                        out.push_str(&format!(
                            "\n  {}  from {:<8} {}  in {} #{}\n",
                            n.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                            n.from,
                            type_str,
                            n.thread_path,
                            n.seq
                        ));
                        out.push_str(&format!("    \"{}\"\n", n.snippet));
                    }
                    output::print_default(out.trim_end());
                }
            }
        }
    }

    Ok(())
}
