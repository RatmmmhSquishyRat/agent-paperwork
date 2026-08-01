//! Post (group thread) commands: create, send, read, summary, edit.

use std::io::Read as _;
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

        /// Message body (positional, optional if --stdin)
        body: Option<String>,

        /// Read body from stdin
        #[arg(long)]
        stdin: bool,

        /// Seq number being replied to
        #[arg(long = "reply-to")]
        reply_to: Option<u64>,

        /// Names mentioned (comma-separated)
        #[arg(long = "mention", value_delimiter = ',')]
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

        /// Maximum number of messages to show (default 20)
        #[arg(long, default_value = "20")]
        limit: usize,
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

        /// New message body (positional, optional if --stdin)
        new_body: Option<String>,

        /// Read new body from stdin
        #[arg(long)]
        stdin: bool,
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
            )?;

            let env = output::Envelope::new("post.create", path.display().to_string())
                .field("path", &path.display().to_string())
                .field("title", &title)
                .field("participants", &participants.join(", "))
                .field("seq", &seq.to_string());
            output::emit_ok(ctx, env);
            Ok(())
        }

        PostCommand::Send {
            path,
            from,
            body,
            stdin,
            reply_to,
            mention,
        } => {
            let path = ensure_suffix(path, ".post.md");

            // Resolve body from --stdin or positional
            let body = resolve_body(body, stdin)?;

            // Reject empty/whitespace-only body
            if body.trim().is_empty() {
                return Err(paperwork_core::PaperworkError::Validation {
                    message: "message body is empty".to_string(),
                    fix: "provide a non-empty message body".to_string(),
                    example: format!("paperwork post send {} --from {} \"Hello\"", path.display(), from),
                }.into());
            }

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
                paperwork_core::ops::thread::thread_send(&path, &from, &[], &body, reply_to, &mentions)?;

            let conclusion = format!("#{} -> {}", seq, path.display());
            let env = output::Envelope::new("post.send", conclusion)
                .field("seq", &seq.to_string())
                .field("path", &path.display().to_string())
                .field("sender", &from);
            output::emit_ok(ctx, env);
            Ok(())
        }

        PostCommand::Read { path, from, to, mention, reply_to, limit } => {
            let path = ensure_suffix(path, ".post.md");
            let all_messages = paperwork_core::ops::thread::thread_read(&path, from, to)?;

            // Apply filters
            let mut messages = all_messages;
            if let Some(ref name) = mention {
                messages.retain(|m| m.mentions.iter().any(|mn| mn == name));
            }
            if let Some(seq) = reply_to {
                messages.retain(|m| m.reply_to == Some(seq));
            }

            let total = messages.len();

            // Apply limit (take last N)
            if messages.len() > limit {
                let skip = messages.len() - limit;
                messages = messages.into_iter().skip(skip).collect();
            }

            match ctx.mode {
                OutputMode::Json => {
                    let mut obj = serde_json::Map::new();
                    obj.insert("status".to_string(), serde_json::json!("ok"));
                    obj.insert("command".to_string(), serde_json::json!("post.read"));
                    obj.insert("conclusion".to_string(), serde_json::json!(format!("{} messages", total)));
                    if total > limit {
                        obj.insert("showing".to_string(), serde_json::json!(format!("{}/{}", messages.len(), total)));
                    }
                    obj.insert("messages".to_string(), serde_json::json!(messages));
                    println!("{}", serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default());
                }
                OutputMode::Plain => {
                    // Serialize only selected messages back to file format
                    let content = paperwork_core::format::thread::serialize_thread(&messages);
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    let mut env = output::Envelope::new("post.read", format!("{} messages", total));
                    if total > limit {
                        env = env.field("showing", &format!("{}/{}", messages.len(), total));
                    }
                    let mut body_lines = Vec::new();
                    for msg in &messages {
                        let mut header = format!("#{} {} {}", msg.seq, msg.sender, msg.timestamp.format("%Y-%m-%dT%H:%M:%SZ"));
                        if let Some(r) = msg.reply_to {
                            header.push_str(&format!(" reply:#{}", r));
                        }
                        if !msg.mentions.is_empty() {
                            header.push_str(&format!(" mentions:{}", msg.mentions.join(",")));
                        }
                        body_lines.push(header);
                        for line in msg.body.lines() {
                            body_lines.push(format!("  {}", line));
                        }
                    }
                    env = env.body_lines(body_lines);
                    output::emit_ok(ctx, env);
                }
            }
            Ok(())
        }

        PostCommand::Summary { path } => {
            let path = ensure_suffix(path, ".post.md");
            let summary = paperwork_core::ops::thread::thread_summary(&path)?;

            // Extract title from first system message
            let messages = paperwork_core::ops::thread::thread_read(&path, Some(1), Some(1)).unwrap_or_default();
            let title = messages.first()
                .map(|m| {
                    m.body.strip_prefix("[Thread created: ")
                        .map(|s| s.split(" |").next().unwrap_or(s))
                        .map(|s| s.trim_end_matches(']'))
                        .unwrap_or(&m.body)
                        .to_string()
                })
                .unwrap_or_default();

            // Get participants from first message
            let participants = messages.first()
                .map(|m| {
                    m.body.split("participants: ")
                        .nth(1)
                        .and_then(|s| s.strip_suffix(']'))
                        .unwrap_or("")
                        .to_string()
                })
                .unwrap_or_default();

            let last_snippet = summary.snippets.last().cloned().unwrap_or_default();

            match ctx.mode {
                OutputMode::Json => {
                    let mut obj = serde_json::Map::new();
                    obj.insert("status".to_string(), serde_json::json!("ok"));
                    obj.insert("command".to_string(), serde_json::json!("post.summary"));
                    obj.insert("conclusion".to_string(), serde_json::json!(path.display().to_string()));
                    obj.insert("title".to_string(), serde_json::json!(title));
                    obj.insert("participants".to_string(), serde_json::json!(participants));
                    obj.insert("messages".to_string(), serde_json::json!(summary.message_count));
                    if let Some(ref s) = summary.last_sender {
                        obj.insert("last.sender".to_string(), serde_json::json!(s));
                    }
                    if let Some(t) = summary.last_timestamp {
                        obj.insert("last.time".to_string(), serde_json::json!(t.format("%Y-%m-%dT%H:%M:%SZ").to_string()));
                    }
                    obj.insert("last.snippet".to_string(), serde_json::json!(last_snippet));
                    println!("{}", serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default());
                }
                _ => {
                    let mut env = output::Envelope::new("post.summary", path.display().to_string())
                        .field("title", &title)
                        .field("participants", &participants)
                        .field("messages", &summary.message_count.to_string());
                    if let Some(ref s) = summary.last_sender {
                        env = env.field("last.sender", s);
                    }
                    if let Some(t) = summary.last_timestamp {
                        env = env.field("last.time", &t.format("%Y-%m-%dT%H:%M:%SZ").to_string());
                    }
                    env = env.field("last.snippet", &last_snippet);
                    output::emit_ok(ctx, env);
                }
            }
            Ok(())
        }

        PostCommand::Edit {
            path,
            seq,
            from,
            new_body,
            stdin,
        } => {
            let path = ensure_suffix(path, ".post.md");

            let new_body = resolve_body(new_body, stdin)?;

            paperwork_core::ops::thread::thread_edit(&path, seq, &from, &new_body)?;

            let env = output::Envelope::new("post.edit", format!("#{}", seq))
                .field("seq", &seq.to_string())
                .field("path", &path.display().to_string());
            output::emit_ok(ctx, env);
            Ok(())
        }
    }
}

/// Resolve body from positional arg or --stdin flag.
fn resolve_body(positional: Option<String>, stdin: bool) -> Result<String> {
    match (positional, stdin) {
        (Some(_), true) => Err(paperwork_core::PaperworkError::Validation {
            message: "both positional body and --stdin provided".to_string(),
            fix: "use either a positional body argument or --stdin, not both".to_string(),
            example: "paperwork post send thread.post.md --from alice --stdin".to_string(),
        }.into()),
        (None, false) => Err(paperwork_core::PaperworkError::Validation {
            message: "no message body provided".to_string(),
            fix: "provide a body as a positional argument or use --stdin".to_string(),
            example: "paperwork post send thread.post.md --from alice \"Hello\"".to_string(),
        }.into()),
        (Some(body), false) => Ok(body),
        (None, true) => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
