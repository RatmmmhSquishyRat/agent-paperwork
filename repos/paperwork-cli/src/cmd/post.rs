//! Post (group thread) commands: create, send, read, summary, edit.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cmd::{ensure_suffix, Context};
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct PostArgs {
    #[command(subcommand)]
    command: PostCommand,
}

#[derive(Subcommand)]
enum PostCommand {
    /// Create a new post thread
    Create {
        /// Path for the new post thread file
        path: PathBuf,

        /// Thread title
        #[arg(long)]
        title: String,

        /// Comma-separated participant names
        #[arg(long, value_delimiter = ',')]
        participants: Vec<String>,
    },

    /// Send a message to a post thread
    Send {
        /// Path to the post thread file
        path: PathBuf,

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

    /// Read messages from a post thread
    Read {
        /// Path to the post thread file
        path: PathBuf,

        /// Start from seq N (inclusive)
        #[arg(long)]
        from: Option<u64>,

        /// End at seq M (inclusive)
        #[arg(long)]
        to: Option<u64>,

        /// Filter: only show messages mentioning this name
        #[arg(long)]
        mention: Option<String>,

        /// Filter: only show messages replying to this seq
        #[arg(long = "reply-to")]
        reply_to: Option<u64>,
    },

    /// Get a summary of a post thread
    Summary {
        /// Path to the post thread file
        path: PathBuf,
    },

    /// Edit a message in a post thread
    Edit {
        /// Path to the post thread file
        path: PathBuf,

        /// Sequence number of the message to edit
        #[arg(long)]
        seq: u64,

        /// Sender name (must match original sender)
        #[arg(long)]
        from: String,

        /// New message body
        new_body: String,
    },
}

pub fn run(ctx: &Context, args: PostArgs) -> Result<()> {
    match args.command {
        PostCommand::Create {
            path,
            title,
            participants,
        } => {
            let path = ensure_suffix(path, ".post.md");
            // Create the thread by sending a system/creation message
            // The first message establishes the thread with title as body
            let body = if participants.is_empty() {
                format!("[Thread created: {}]", title)
            } else {
                format!(
                    "[Thread created: {} | participants: {}]",
                    title,
                    participants.join(", ")
                )
            };

            let seq = paperwork_core::ops::thread::thread_send(
                &path, "system", &[], &body, None, &[],
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "path": path.display().to_string(),
                        "title": title,
                        "participants": participants,
                        "seq": seq,
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Post created: {}", path.display())),
            }
            Ok(())
        }

        PostCommand::Send {
            path,
            from,
            body,
            reply_to,
            mention,
        } => {
            let path = ensure_suffix(path, ".post.md");
            // Reply carries implicit @: auto-add original sender to mentions
            let mut mentions = mention;
            if let Some(reply_seq) = reply_to {
                if let Ok(msgs) = paperwork_core::ops::thread::thread_read(&path, Some(reply_seq), Some(reply_seq)) {
                    if let Some(original) = msgs.first() {
                        if !mentions.contains(&original.sender) && original.sender != from {
                            mentions.push(original.sender.clone());
                        }
                    }
                }
            }

            let seq =
                paperwork_core::ops::thread::thread_send(&path, &from, &[], &body, reply_to, &mentions)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "seq": seq,
                        "path": path.display().to_string(),
                        "from": from,
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Post #{} sent → {}", seq, path.display())),
            }
            Ok(())
        }

        PostCommand::Read { path, from, to, mention, reply_to } => {
            let path = ensure_suffix(path, ".post.md");
            let mut messages = paperwork_core::ops::thread::thread_read(&path, from, to)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            // Apply filters
            if let Some(ref name) = mention {
                messages.retain(|m| m.mentions.iter().any(|mn| mn == name));
            }
            if let Some(seq) = reply_to {
                messages.retain(|m| m.reply_to == Some(seq));
            }

            match ctx.mode {
                OutputMode::Json => output::print_json(&messages),
                OutputMode::Plain => {
                    let content = std::fs::read_to_string(&path)
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

        PostCommand::Summary { path } => {
            let path = ensure_suffix(path, ".post.md");
            let summary = paperwork_core::ops::thread::thread_summary(&path)
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

        PostCommand::Edit {
            path,
            seq,
            from,
            new_body,
        } => {
            let path = ensure_suffix(path, ".post.md");
            paperwork_core::ops::thread::thread_edit(&path, seq, &from, &new_body)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let result = serde_json::json!({
                        "edited": seq,
                        "path": path.display().to_string(),
                    });
                    output::print_json(&result);
                }
                _ => output::success(ctx, &format!("Message #{} edited", seq)),
            }
            Ok(())
        }
    }
}
