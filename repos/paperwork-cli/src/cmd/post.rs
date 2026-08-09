//! Post (group thread) commands: send, read, summary, edit.
//!
//! Format v2 (spec §5): thread creation is folded into the first `post send`;
//! writes the preamble (H1 title only, owner ruling D1) together with the
//! first message inside the same lock. The CLI always passes
//! `Some(ThreadMeta)`; the ops layer guards on in-lock file size (spec §5.7,
//! OQ-1: ignored silently when the file is non-empty).
//!
//! Reference state is body-text only (owner ruling D2): `--reply-to N` /
//! `--mention a,b` are sugar flags whose values are injected into the body
//! as `@#N` / `@name` tokens before calling core (spec §11 OQ-4); the
//! `--to` / `--participants` flags are deleted (D1/D2).

use std::io::Read as _;
use std::path::{Path, PathBuf};

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
    /// Send a message to a post thread (first send creates the thread)
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

        /// Seq number being replied to (injected as an `@#N` body token,
        /// spec §11 OQ-4)
        #[arg(long = "reply-to")]
        reply_to: Option<u64>,

        /// Names mentioned (comma-separated; injected as `@name` body
        /// tokens, spec §11 OQ-4)
        #[arg(long = "mention", value_delimiter = ',')]
        mention: Vec<String>,

        /// Thread title for the preamble on first write
        /// (default: path with .post.md / .md suffix stripped)
        #[arg(long)]
        title: Option<String>,
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
        PostCommand::Send {
            path,
            from,
            body,
            stdin,
            reply_to,
            mention,
            title,
        } => {
            // Default title derives from the original path argument
            // (spec §5.7: strip .post.md, else strip .md, else keep as-is).
            let default_title = default_title(&path);
            let path = ensure_suffix(path, ".post.md");

            // Resolve body from --stdin or positional
            let body = resolve_body(&path, &from, body, stdin, BodyCommand::Send)?;

            // Reject empty/whitespace-only body
            if body.trim().is_empty() {
                return Err(paperwork_core::PaperworkError::Validation {
                    message: "message body is empty".to_string(),
                    fix: "provide a non-empty message body".to_string(),
                    example: format!("paperwork post send {} --from {} \"Hello\"", path.display(), from),
                }.into());
            }

            // Validate every --mention value at the flag layer (review MJ-2):
            // values are injected verbatim as `@name` body tokens, so shapes
            // that the spec §5.4 derivation would silently mangle or drop are
            // rejected up front instead of writing corrupted references.
            for value in &mention {
                validate_mention_value(value, &from)?;
            }

            // Reply carries an implicit @: auto-add the original sender to
            // the mention list (boundaries unchanged: self-reply, already
            // listed, and missing seq never trigger).
            let mut mentions = clean_list(mention);
            if let Some(reply_seq) = reply_to {
                if let Ok(msgs) = paperwork_core::ops::thread::thread_read(&path, Some(reply_seq), Some(reply_seq)) {
                    if let Some(original) = msgs.first() {
                        if !mentions.contains(&original.sender) && original.sender != from {
                            mentions.push(original.sender.clone());
                        }
                    }
                }
            }
            // Deduplicate mention tokens (first occurrence wins).
            let mut seen: Vec<String> = Vec::new();
            for name in mentions {
                if !seen.contains(&name) {
                    seen.push(name);
                }
            }
            let mentions = seen;

            // Reference state lives in the body text only (D2): inject
            // `@#N` / `@name` tokens before calling core (OQ-4). The 64KB
            // cap is then enforced by core on the final body.
            let body = inject_reference_tokens(&body, reply_to, &mentions);

            // Preamble metadata: the CLI always passes Some(meta); the ops
            // layer writes it only when the file is empty inside the lock
            // (spec §5.7, OQ-1). Preamble is the H1 title only (D1).
            let meta = paperwork_core::ThreadMeta {
                title: title.unwrap_or(default_title),
            };

            let seq = paperwork_core::ops::thread::thread_send(
                &path, &from, &body, Some(&meta),
            )?;

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
                // Filter on parse-time derived mentions (D2; spec §5.4).
                messages.retain(|m| m.mentions.iter().any(|mn| mn == name));
            }
            if let Some(seq) = reply_to {
                // Filter on the parse-time derived reply reference (D2).
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
                    // (subset output carries no preamble, BDD:POST-31).
                    let content = paperwork_core::format::thread::serialize_messages(&messages);
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    let mut env = output::Envelope::new("post.read", format!("{} messages", total));
                    if total > limit {
                        env = env.field("showing", &format!("{}/{}", messages.len(), total));
                    }
                    let mut body_lines = Vec::new();
                    for msg in &messages {
                        // reply/mentions shown from parse-time derivations
                        // (D2); no `to` output remains (field deleted).
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

            // Title comes straight from the preamble (ops thread_meta);
            // participants are derived from the message sender set (D1,
            // spec §5.4) and carried by the summary.
            let meta = paperwork_core::ops::thread::thread_meta(&path)?;
            let title = meta.title;
            let participants = summary.participants.join(", ");

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

            let new_body = resolve_body(&path, &from, new_body, stdin, BodyCommand::Edit { seq })?;

            paperwork_core::ops::thread::thread_edit(&path, seq, &from, &new_body)?;

            let env = output::Envelope::new("post.edit", format!("#{}", seq))
                .field("seq", &seq.to_string())
                .field("path", &path.display().to_string());
            output::emit_ok(ctx, env);
            Ok(())
        }
    }
}

/// Default preamble title (spec §5.7): strip the `.post.md` suffix, else
/// strip the `.md` suffix, else keep the file name as-is.
fn default_title(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());
    if let Some(stem) = name.strip_suffix(".post.md") {
        stem.to_string()
    } else if let Some(stem) = name.strip_suffix(".md") {
        stem.to_string()
    } else {
        name
    }
}

/// Trim each segment of a comma-separated list and drop empty segments.
fn clean_list(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Validate a single `--mention` flag value (review MJ-2).
///
/// Each value is injected verbatim as an `@name` body token (spec §11 OQ-4)
/// and must survive the spec §5.4 derivation unchanged:
/// - non-empty;
/// - no whitespace / `@` / `(` / `)` (the token scan would truncate it);
/// - not `#<pure digits>` (that shape derives as a reply reference, and the
///   structured channel for replies is `--reply-to`);
/// - not the sender itself (self-mentions are silently dropped).
fn validate_mention_value(value: &str, from: &str) -> anyhow::Result<()> {
    let bad_chars = value
        .chars()
        .any(|c| c.is_whitespace() || c == '@' || c == '(' || c == ')');
    let reply_shaped = value
        .strip_prefix('#')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()));

    let reason = if value.is_empty() {
        Some("it is empty")
    } else if bad_chars {
        Some("it contains whitespace, '@' or parentheses")
    } else if reply_shaped {
        Some("it is a reply reference shape (#<seq>), not a mention")
    } else if value == from {
        Some("it mentions the sender itself")
    } else {
        None
    };

    match reason {
        None => Ok(()),
        Some(reason) => Err(paperwork_core::PaperworkError::Validation {
            message: format!("invalid --mention value '{}': {}", value, reason),
            fix: "mention values must be non-empty single tokens without whitespace, '@' or parentheses; use --reply-to N for reply references; do not mention the sender itself".to_string(),
            example: format!("paperwork post send myfile --from {} --mention bob \"Hello\"", from),
        }
        .into()),
    }
}

/// Inject reference tokens into the body head (spec §11 OQ-4, D2).
///
/// `--reply-to N` yields `@#N`; each mention yields `@name`. All tokens sit
/// on a single first line separated by single spaces, followed by a blank
/// line, then the original body — so the spec §5.4 derivation rules can
/// recover reply-to and mentions from the persisted body text.
fn inject_reference_tokens(body: &str, reply_to: Option<u64>, mentions: &[String]) -> String {
    let mut tokens: Vec<String> = Vec::new();
    if let Some(seq) = reply_to {
        tokens.push(format!("@#{}", seq));
    }
    for name in mentions {
        tokens.push(format!("@{}", name));
    }
    if tokens.is_empty() {
        body.to_string()
    } else {
        format!("{}\n\n{}", tokens.join(" "), body)
    }
}

/// Which command is resolving the body — drives the error-envelope example
/// shape so the one-retry contract points at the correct invocation
/// (review F3: edit previously showed a send-shaped example).
enum BodyCommand {
    Send,
    Edit { seq: u64 },
}

/// Resolve body from positional arg or --stdin flag.
fn resolve_body(
    path: &Path,
    from: &str,
    positional: Option<String>,
    stdin: bool,
    command: BodyCommand,
) -> Result<String> {
    let (stdin_example, body_example) = match command {
        BodyCommand::Send => (
            format!("paperwork post send {} --from {} --stdin", path.display(), from),
            format!("paperwork post send {} --from {} \"Hello\"", path.display(), from),
        ),
        BodyCommand::Edit { seq } => (
            format!("paperwork post edit {} --seq {} --from {} --stdin", path.display(), seq, from),
            format!("paperwork post edit {} --seq {} --from {} \"New body\"", path.display(), seq, from),
        ),
    };
    match (positional, stdin) {
        (Some(_), true) => Err(paperwork_core::PaperworkError::Validation {
            message: "both positional body and --stdin provided".to_string(),
            fix: "use either a positional body argument or --stdin, not both".to_string(),
            example: stdin_example,
        }.into()),
        (None, false) => Err(paperwork_core::PaperworkError::Validation {
            message: "no message body provided".to_string(),
            fix: "provide a body as a positional argument or use --stdin".to_string(),
            example: body_example,
        }.into()),
        (Some(body), false) => Ok(body),
        (None, true) => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
