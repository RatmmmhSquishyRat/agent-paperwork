//! Post (group thread) commands: create, send, read, summary, edit.
//!
//! v0.5.0 grammar: PATH is always the first required positional argument;
//! NAME (the signing actor) is the second required positional for send/edit;
//! content (BODY / NEW_BODY) is always the last positional argument.

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

/// Command identifier for the output protocol (post.<verb>).
pub fn command_id(args: &PostArgs) -> &'static str {
    match &args.command {
        PostCommand::Create { .. } => "post.create",
        PostCommand::Send { .. } => "post.send",
        PostCommand::Read { .. } => "post.read",
        PostCommand::Summary { .. } => "post.summary",
        PostCommand::Edit { .. } => "post.edit",
    }
}

#[derive(Subcommand)]
enum PostCommand {
    /// Create a new post thread
    #[command(after_help = "Examples:\n  paperwork post create standup \"Daily Standup\" --participants alice,bob")]
    Create {
        /// Path for the new post thread file
        path: PathBuf,

        /// Thread title
        title: String,

        /// Comma-separated participant names
        #[arg(long, value_delimiter = ',')]
        participants: Vec<String>,
    },

    /// Send a message to a post thread
    #[command(after_help = "Examples:\n  paperwork post send standup.post.md alice \"Parser module is 80% done.\"\n  paperwork post send standup alice --reply-to 2 --mention bob \"Tests merged.\"\n  echo \"multi-line body\" | paperwork post send standup.post.md alice --stdin\n  paperwork post send standup.post.md alice -- \"-fix flag text\"")]
    Send {
        /// Path to the post thread file
        path: PathBuf,

        /// Sender name (signature)
        name: String,

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
    #[command(after_help = "Examples:\n  paperwork post read standup.post.md --from 5 --to 20\n  paperwork post read standup.post.md --mention alice --limit 20")]
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
    #[command(after_help = "Examples:\n  paperwork post edit standup.post.md alice 3 \"corrected body\"\n  paperwork post edit standup.post.md alice 3 -- \"-starts with dash\"")]
    Edit {
        /// Path to the post thread file
        path: PathBuf,

        /// Editor name (must match original sender)
        name: String,

        /// Sequence number of the message to edit
        seq: u64,

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

            // create never appends to an existing thread
            if path.is_file() {
                return Err(paperwork_core::PaperworkError::AlreadyExists {
                    resource: "Thread".to_string(),
                    name: path.display().to_string(),
                    fix: "the thread already exists; send to it, or choose a different path".to_string(),
                    example: format!("paperwork post send {} alice \"Hello\"", path.display()),
                }.into());
            }

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
            name,
            body,
            stdin,
            reply_to,
            mention,
        } => {
            let path = ensure_suffix(path, ".post.md");

            // Stage-1 hit on a foreign file -> format error, no re-routing
            // (spec S-PATH-07, see reject_foreign_thread).
            reject_foreign_thread(&path)?;

            // Resolve body from --stdin or positional
            let body = resolve_body(body, stdin, BodyOwner::Send, &path.display().to_string())?;

            // Reject empty/whitespace-only body
            if body.trim().is_empty() {
                return Err(paperwork_core::PaperworkError::Validation {
                    message: "message body is empty".to_string(),
                    fix: "provide a non-empty message body; a body starting with '-' must be placed after -- (e.g. paperwork post send standup.post.md alice -- \"-fix flag text\")".to_string(),
                    example: format!("paperwork post send {} alice \"Hello\"", path.display()),
                }.into());
            }

            // Reject empty/whitespace-only NAME
            if name.trim().is_empty() {
                return Err(paperwork_core::PaperworkError::Validation {
                    message: "sender name (NAME) is empty".to_string(),
                    fix: "provide a non-empty NAME as the second positional argument (right after PATH)".to_string(),
                    example: format!("paperwork post send {} alice \"Hello\"", path.display()),
                }.into());
            }

            // Reply carries implicit @: auto-add original sender to mentions
            let mut mentions = mention;
            let mut implicit_mention: Option<String> = None;
            if let Some(reply_seq) = reply_to {
                if let Ok(msgs) = paperwork_core::ops::thread::thread_read(&path, Some(reply_seq), Some(reply_seq)) {
                    if let Some(original) = msgs.first() {
                        if !mentions.contains(&original.sender) && original.sender != name {
                            mentions.push(original.sender.clone());
                            implicit_mention = Some(original.sender.clone());
                        }
                    }
                }
            }

            let seq =
                paperwork_core::ops::thread::thread_send(&path, &name, &[], &body, reply_to, &mentions)?;

            let conclusion = format!("#{} -> {}", seq, path.display());
            let mut env = output::Envelope::new("post.send", conclusion)
                .field("seq", &seq.to_string())
                .field("path", &path.display().to_string())
                .field("sender", &name);
            if let Some(ref im) = implicit_mention {
                env = env.field("implicit-mention", im);
            }
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

            // Window bounds by first/last displayed seq (thread-based)
            let window = match (messages.first(), messages.last()) {
                (Some(first), Some(last)) => Some(format!("#{}-#{}", first.seq, last.seq)),
                _ => None,
            };

            match ctx.mode {
                OutputMode::Json => {
                    let mut obj = serde_json::Map::new();
                    obj.insert("status".to_string(), serde_json::json!("ok"));
                    obj.insert("command".to_string(), serde_json::json!("post.read"));
                    obj.insert("conclusion".to_string(), serde_json::json!(format!("{} messages", total)));
                    obj.insert("showing".to_string(), serde_json::json!(format!("{}/{}", messages.len(), total)));
                    if let Some(ref w) = window {
                        obj.insert("window".to_string(), serde_json::json!(w));
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
                    let mut env = output::Envelope::new("post.read", format!("{} messages", total))
                        .field("showing", &format!("{}/{}", messages.len(), total));
                    if let Some(ref w) = window {
                        env = env.field("window", w);
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

            // Read-only command: when all three resolution stages miss,
            // report not-found in the same shape as post read
            // (core thread_summary returns an empty summary instead).
            if !path.is_file() {
                return Err(paperwork_core::PaperworkError::NotFound {
                    resource: "Thread".to_string(),
                    name: path.display().to_string(),
                    fix: "send a message first to create the thread".to_string(),
                    example: format!("paperwork post send {} alice \"Hello\"", path.display()),
                }.into());
            }

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
            name,
            seq,
            new_body,
            stdin,
        } => {
            let path = ensure_suffix(path, ".post.md");

            // Stage-1 hit on a foreign file -> format error, not not-found
            // (mirrors send; spec S-PATH-07, review M2).
            reject_foreign_thread(&path)?;

            let new_body = resolve_body(new_body, stdin, BodyOwner::Edit, &path.display().to_string())?;

            paperwork_core::ops::thread::thread_edit(&path, seq, &name, &new_body)?;

            let env = output::Envelope::new("post.edit", format!("#{}", seq))
                .field("seq", &seq.to_string())
                .field("path", &path.display().to_string());
            output::emit_ok(ctx, env);
            Ok(())
        }
    }
}

/// Which command owns a resolve_body call (error examples differ per verb).
enum BodyOwner {
    Send,
    Edit,
}

/// Reject a stage-1 path hit on a file that is not a valid paperwork thread.
///
/// Three-stage resolution stage 1 may hit an existing file that is not a
/// thread: validate with the thread parser up front so the format error
/// surfaces instead of corrupting the file (no re-routing, spec S-PATH-07).
/// Content without any valid message boundary is rejected the same way
/// `validate` rejects it. Missing paths pass through (write commands create
/// them; read-side commands report their own not-found).
fn reject_foreign_thread(path: &std::path::Path) -> Result<()> {
    if path.is_file() {
        let pre_existing = paperwork_core::ops::thread::thread_read(path, None, None)?;
        let raw = std::fs::read_to_string(path)?;
        if pre_existing.is_empty() && !raw.trim().is_empty() {
            return Err(paperwork_core::PaperworkError::Parse {
                message: format!("{} is not a valid post thread: no valid message boundaries found", path.display()),
                fix: "expected --- separators with ### #N sender . timestamp headers; or validate it explicitly".to_string(),
                example: format!("paperwork validate {} --type post", path.display()),
            }.into());
        }
    }
    Ok(())
}

/// Resolve body from positional arg or --stdin flag.
///
/// Error examples are verb-specific: edit failures must not show send-form
/// commands (feasibility review m-2).
fn resolve_body(positional: Option<String>, stdin: bool, owner: BodyOwner, path: &str) -> Result<String> {
    match (positional, stdin) {
        (Some(_), true) => {
            let example = match owner {
                BodyOwner::Send => format!("paperwork post send {} alice --stdin", path),
                BodyOwner::Edit => format!("paperwork post edit {} alice 2 --stdin", path),
            };
            Err(paperwork_core::PaperworkError::Validation {
                message: "both positional body and --stdin provided".to_string(),
                fix: "use either a positional body argument or --stdin, not both".to_string(),
                example,
            }.into())
        }
        (None, false) => {
            let (message, example) = match owner {
                BodyOwner::Send => (
                    "no message body provided; if you already gave a body, check that you did not miss the NAME slot (NAME comes right after PATH)".to_string(),
                    format!("paperwork post send {} alice \"Hello\"", path),
                ),
                BodyOwner::Edit => (
                    "no message body provided".to_string(),
                    format!("paperwork post edit {} alice 2 \"corrected body\"", path),
                ),
            };
            Err(paperwork_core::PaperworkError::Validation {
                message,
                fix: "provide a body as a positional argument or use --stdin; a body starting with '-' must be placed after -- (e.g. paperwork post send standup.post.md alice -- \"-fix flag text\")".to_string(),
                example,
            }.into())
        }
        (Some(body), false) => Ok(body),
        (None, true) => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
