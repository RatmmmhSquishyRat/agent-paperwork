//! DM commands: send, read, summary.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cmd::Context;
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct DmArgs {
    #[command(subcommand)]
    command: DmCommand,
}

#[derive(Subcommand)]
enum DmCommand {
    /// Send a direct message
    Send {
        /// Path to the sender's profile file (DM thread path is derived from this)
        profile_path: PathBuf,

        /// Recipient name
        #[arg(long)]
        to: String,

        /// Sender name
        #[arg(long)]
        from: String,

        /// Message body
        body: String,

        /// Seq number being replied to
        #[arg(long = "reply-to")]
        reply_to: Option<u64>,

        /// Names mentioned in the message
        #[arg(long = "mention", num_args = 0..)]
        mention: Vec<String>,
    },

    /// Read messages from a DM thread
    Read {
        /// Path to the DM thread file
        thread_path: PathBuf,

        /// Start from seq N (inclusive)
        #[arg(long)]
        from: Option<u64>,

        /// End at seq M (inclusive)
        #[arg(long)]
        to: Option<u64>,
    },

    /// Get a summary of a DM thread
    Summary {
        /// Path to the DM thread file
        thread_path: PathBuf,
    },
}

pub fn run(ctx: &Context, args: DmArgs) -> Result<()> {
    match args.command {
        DmCommand::Send {
            profile_path,
            to,
            from,
            body,
            reply_to,
            mention,
        } => {
            let thread_path = paperwork_core::ops::dm_thread_path(&profile_path, &to);

            let seq = paperwork_core::ops::thread::thread_send(
                &thread_path,
                &from,
                std::slice::from_ref(&to),
                &body,
                reply_to,
                &mention,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "seq": seq,
                        "thread_path": thread_path.display().to_string(),
                        "from": from,
                        "to": to,
                    });
                    output::print_json(&result);
                }
                _ => output::success(
                    ctx,
                    &format!("DM #{} sent → {}", seq, thread_path.display()),
                ),
            }
            Ok(())
        }

        DmCommand::Read {
            thread_path,
            from,
            to,
        } => {
            let messages = paperwork_core::ops::thread::thread_read(&thread_path, from, to)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => output::print_json(&messages),
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(&thread_path)
                        .map_err(|e| anyhow::anyhow!("IO error: {}", e))?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    if messages.is_empty() {
                        output::print_default("(no messages)");
                    } else {
                        for msg in &messages {
                            let reply_info = msg
                                .reply_to
                                .map(|r| format!(" (reply to #{})", r))
                                .unwrap_or_default();
                            output::print_default(&format!(
                                "**#{}** {}{}:",
                                msg.seq, msg.sender, reply_info
                            ));
                            output::print_default(&format!("  {}", msg.body));
                        }
                    }
                }
            }
            Ok(())
        }

        DmCommand::Summary { thread_path } => {
            let summary = paperwork_core::ops::thread::thread_summary(&thread_path)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => output::print_json(&summary),
                _ => {
                    output::print_default(&format!(
                        "Thread: {} ({} messages)",
                        summary.thread_path, summary.message_count
                    ));
                    if let Some(ref sender) = summary.last_sender {
                        output::print_default(&format!("Last sender: {}", sender));
                    }
                    for snippet in &summary.snippets {
                        output::print_default(&format!("  > {}", snippet));
                    }
                }
            }
            Ok(())
        }
    }
}
