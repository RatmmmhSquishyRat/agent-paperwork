//! Notify commands: read, push.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cmd::Context;
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct NotifyArgs {
    #[command(subcommand)]
    command: NotifyCommand,
}

#[derive(Subcommand)]
enum NotifyCommand {
    /// Read notifications from a file
    Read {
        /// Path to the notification file
        path: PathBuf,
    },

    /// Push a notification to a file
    Push {
        /// Path to the notification file
        path: PathBuf,

        /// Sender name
        #[arg(long)]
        from: String,

        /// Thread path that triggered the notification
        #[arg(long)]
        thread: String,

        /// Sequence number of the triggering message
        #[arg(long)]
        seq: u64,

        /// Notification type: mention or reply
        #[arg(long = "type")]
        notify_type: String,

        /// Snippet of the triggering message
        #[arg(long, default_value = "")]
        snippet: String,
    },
}

pub fn run(ctx: &Context, args: NotifyArgs) -> Result<()> {
    match args.command {
        NotifyCommand::Read { path } => {
            let notifications = paperwork_core::ops::notify::notify_read(&path)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => output::print_json(&notifications),
                OutputMode::Plain => {
                    if path.exists() {
                        let content = std::fs::read_to_string(&path)
                            .map_err(|e| anyhow::anyhow!("IO error: {}", e))?;
                        output::print_plain(&content);
                    } else {
                        output::print_plain("(no notifications)");
                    }
                }
                OutputMode::Default => {
                    if notifications.is_empty() {
                        output::print_default("(no notifications)");
                    } else {
                        for n in &notifications {
                            output::print_default(&format!(
                                "[{:?}] from {} in {} #{}: {}",
                                n.notify_type, n.from, n.thread_path, n.seq, n.snippet
                            ));
                        }
                    }
                }
            }
            Ok(())
        }

        NotifyCommand::Push {
            path,
            from,
            thread,
            seq,
            notify_type,
            snippet,
        } => {
            let nt = match notify_type.as_str() {
                "mention" => paperwork_core::NotifyType::Mention,
                "reply" => paperwork_core::NotifyType::Reply,
                other => {
                    anyhow::bail!(
                        "Invalid notify type '{}'.\n  \u{2192} Use 'mention' or 'reply'.",
                        other
                    );
                }
            };

            let notification = paperwork_core::Notification {
                timestamp: chrono::Utc::now(),
                from: from.clone(),
                thread_path: thread,
                seq,
                notify_type: nt,
                snippet,
            };

            // Derive name from file stem for the H1 heading
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Notifications".to_string());

            paperwork_core::ops::notify::notify_push(&path, &name, &notification)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "path": path.display().to_string(),
                        "from": from,
                        "type": format!("{:?}", nt),
                        "seq": seq,
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Notification pushed → {}", path.display())),
            }
            Ok(())
        }
    }
}
