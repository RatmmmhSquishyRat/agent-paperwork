//! Post (group thread) commands: send, read, summary, edit.
//!
//! v0.6 grammar: PATH is the only positional argument; every required
//! payload is a named flag -- `--author` (signature, required),
//! `--message` or `--stdin` (body channel, exactly one required),
//! `--seq` (edit target, required for edit).
//!
//! Format v2 (spec section 5): thread creation is folded into the first `post send`;
//! writes the preamble (H1 title only, owner ruling D1) together with the
//! first message inside the same lock. The CLI always passes
//! `Some(ThreadMeta)`; the ops layer guards on in-lock file size (spec section 5.7,
//! OQ-1: ignored silently when the file is non-empty).
//!
//! Reference state is body-text only (owner ruling D2): `--reply-to N` /
//! `--mention a,b` are sugar flags whose values are injected into the body
//! as `@#N` / `@name` tokens before calling core (spec section 11 OQ-4); the
//! `--to` / `--participants` flags are deleted (D1/D2).

use std::io::Read as _;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Subcommand};
use fs2::FileExt;

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
        PostCommand::Send { .. } => "post.send",
        PostCommand::Read { .. } => "post.read",
        PostCommand::Summary { .. } => "post.summary",
        PostCommand::Edit { .. } => "post.edit",
    }
}

#[derive(Subcommand)]
enum PostCommand {
    /// Send a message to a post thread (first send creates the thread)
    #[command(
        after_help = "Examples:\n  paperwork post send standup.post.md --author alice --message \"Parser module is 80% done.\"\n  paperwork post send standup.post.md -a alice -m \"Tests merged.\" --reply-to 2 --mention bob\n  echo \"multi-line body\" | paperwork post send standup.post.md --author alice --stdin\n  paperwork post send standup.post.md --author alice --message \"-starts with dash is fine\"\n  paperwork post send new-topic.post.md --author alice --message \"kickoff\" --title \"New Topic\"\n  # --title (thread title, honoured on first write only, silently ignored on existing threads);\n  # --reply-to / --mention are sugar flags: their values are injected into the body as @#N / @name tokens;\n  # a body that looks like a flag (e.g. literal \"--stdin\") is taken verbatim after -m/--message; use the equals form -m=\"--stdin\" / --message=\"--stdin\" to make the intent explicit."
    )]
    Send {
        /// Path to the post thread file
        path: PathBuf,

        /// Sender name (signature)
        #[arg(short = 'a', long)]
        author: String,

        /// Message body (conflicts with --stdin; one of them is required).
        /// allow_hyphen_values lets bodies starting with '-' pass through.
        #[arg(
            short = 'm',
            long,
            allow_hyphen_values = true,
            required_unless_present = "stdin",
            conflicts_with = "stdin"
        )]
        message: Option<String>,

        /// Read body from stdin
        #[arg(long)]
        stdin: bool,

        /// Seq number being replied to (injected as an `@#N` body token,
        /// spec section 11 OQ-4)
        #[arg(long = "reply-to")]
        reply_to: Option<u64>,

        /// Names mentioned (comma-separated; injected as `@name` body
        /// tokens, spec section 11 OQ-4)
        #[arg(long = "mention", value_delimiter = ',')]
        mention: Vec<String>,

        /// Thread title for the preamble on first write
        /// (default: path with .post.md / .md suffix stripped)
        #[arg(long)]
        title: Option<String>,
    },

    /// Read messages from a post thread
    #[command(
        after_help = "Examples:\n  paperwork post read standup.post.md --from 5 --to 20\n  paperwork post read standup.post.md --mention alice --limit 20"
    )]
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
    #[command(
        after_help = "Examples:\n  paperwork post edit standup.post.md --author alice --seq 3 --message \"corrected body\"\n  paperwork post edit standup.post.md --author alice --seq 3 --message \"-starts with dash is fine\"\n  # a body that looks like a flag (e.g. literal \"--stdin\") is taken verbatim after -m/--message; use the equals form -m=\"--stdin\" / --message=\"--stdin\" to make the intent explicit."
    )]
    Edit {
        /// Path to the post thread file
        path: PathBuf,

        /// Editor name (must match original sender)
        #[arg(short = 'a', long)]
        author: String,

        /// Sequence number of the message to edit
        #[arg(long)]
        seq: u64,

        /// New message body (conflicts with --stdin; one of them is
        /// required). allow_hyphen_values lets bodies starting with '-'
        /// pass through.
        #[arg(
            short = 'm',
            long,
            allow_hyphen_values = true,
            required_unless_present = "stdin",
            conflicts_with = "stdin"
        )]
        message: Option<String>,

        /// Read new body from stdin
        #[arg(long)]
        stdin: bool,
    },
}

pub fn run(ctx: &Context, args: PostArgs) -> Result<()> {
    match args.command {
        PostCommand::Send {
            path,
            author,
            message,
            stdin,
            reply_to,
            mention,
            title,
        } => {
            // Default title derives from the original path argument
            // (spec section 5.7: strip .post.md, else strip .md, else keep as-is).
            let default_title = default_title(&path);
            let path = ensure_suffix(path, ".post.md");

            // Stage-1 hit on a foreign file -> format error, no re-routing
            // (spec S-PATH-07, see reject_foreign_thread).
            reject_foreign_thread(&path)?;

            // Resolve body from --message or --stdin (the "exactly one"
            // invariant is enforced by clap: required_unless_present +
            // conflicts_with, usage exit 2).
            let body = resolve_body(message, stdin, BodyOwner::Send, &path.display().to_string())?;

            // Reject empty/whitespace-only body
            if body.trim().is_empty() {
                return Err(paperwork_core::PaperworkError::Validation {
                    message: "message body is empty".to_string(),
                    fix: "provide a non-empty --message value (bodies starting with '-' are accepted) or pipe content via --stdin".to_string(),
                    example: format!("paperwork post send {} --author alice --message \"Hello\"", path.display()),
                }.into());
            }

            // Reject empty/whitespace-only author
            if author.trim().is_empty() {
                return Err(paperwork_core::PaperworkError::Validation {
                    message: "sender name (--author) is empty".to_string(),
                    fix: "provide a non-empty --author value".to_string(),
                    example: format!(
                        "paperwork post send {} --author alice --message \"Hello\"",
                        path.display()
                    ),
                }
                .into());
            }

            // Reject --reply-to 0 up front (review n4): seq numbers start at
            // 1 (spec §5.3), so 0 can never reference an existing message.
            if reply_to == Some(0) {
                return Err(paperwork_core::PaperworkError::Validation {
                    message: "reply-to must be >= 1".to_string(),
                    fix: "pass the seq number of an existing message (seq numbers start at 1)"
                        .to_string(),
                    example: format!(
                        "paperwork post send {} --author alice --reply-to 1 --message \"Hello\"",
                        path.display()
                    ),
                }
                .into());
            }

            // Clean the --mention list first (trim each segment, drop empty
            // segments so "alice, bob" and trailing commas are legal —
            // review n3), then validate every surviving value at the flag
            // layer (review MJ-2): values are injected verbatim as `@name`
            // body tokens, so shapes that the spec section 5.4 derivation
            // would silently mangle or drop are rejected up front instead
            // of writing corrupted references.
            let mut mentions = clean_list(mention);
            for value in &mentions {
                validate_mention_value(value, &author)?;
            }

            // Reply carries an implicit @: auto-add the original sender to
            // the mention list (boundaries unchanged: self-reply, already
            // listed, and missing seq never trigger).
            let mut implicit_mention: Option<String> = None;
            if let Some(reply_seq) = reply_to {
                if let Ok(msgs) = paperwork_core::ops::thread::thread_read(
                    &path,
                    Some(reply_seq),
                    Some(reply_seq),
                ) {
                    if let Some(original) = msgs.first() {
                        if !mentions.contains(&original.sender) && original.sender != author {
                            mentions.push(original.sender.clone());
                            implicit_mention = Some(original.sender.clone());
                        }
                    }
                }
            }
            // Deduplicate mention tokens (first occurrence wins).
            let mut seen: Vec<String> = Vec::new();
            for m in mentions {
                if !seen.contains(&m) {
                    seen.push(m);
                }
            }
            let mentions = seen;

            // Reference state lives in the body text only (D2): inject
            // `@#N` / `@name` tokens before calling core (OQ-4). The 64KB
            // cap is then enforced by core on the final body.
            let body = inject_reference_tokens(&body, reply_to, &mentions);

            // Preamble metadata: the CLI always passes Some(meta); the ops
            // layer writes it only when the file is empty inside the lock
            // (spec section 5.7, OQ-1). Preamble is the H1 title only (D1).
            let meta = paperwork_core::ThreadMeta {
                title: title.unwrap_or(default_title),
            };

            let seq = paperwork_core::ops::thread::thread_send(&path, &author, &body, Some(&meta))?;

            let conclusion = format!("#{} -> {}", seq, path.display());
            let mut env = output::Envelope::new("post.send", conclusion)
                .field("seq", &seq.to_string())
                .field("path", &path.display().to_string())
                .field("sender", &author);
            if let Some(ref im) = implicit_mention {
                env = env.field("implicit-mention", im);
            }
            output::emit_ok(ctx, env);
            Ok(())
        }

        PostCommand::Read {
            path,
            from,
            to,
            mention,
            reply_to,
            limit,
        } => {
            let path = ensure_suffix(path, ".post.md");

            // Symmetric foreign-thread guard (review Kim M1): a stage-1 hit
            // on a file that is not a post thread reports `format`
            // (exit 1) instead of silently returning 0 messages (exit 0).
            // Missing paths pass through to thread_read's not-found.
            reject_foreign_thread(&path)?;

            let all_messages = paperwork_core::ops::thread::thread_read(&path, from, to)?;

            // Apply filters
            let mut messages = all_messages;
            if let Some(ref name) = mention {
                // Filter on parse-time derived mentions (D2; spec section 5.4).
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

            // Window bounds by first/last displayed seq (thread-based)
            let window = match (messages.first(), messages.last()) {
                (Some(first), Some(last)) => Some(format!("#{}-#{}", first.seq, last.seq)),
                _ => None,
            };

            match ctx.mode {
                OutputMode::Json => {
                    let obj = output::JsonBuilder::new()
                        .insert("status", serde_json::json!("ok"))
                        .insert("command", serde_json::json!("post.read"))
                        .insert(
                            "conclusion",
                            serde_json::json!(format!("{} messages", total)),
                        )
                        .insert(
                            "showing",
                            serde_json::json!(format!("{}/{}", messages.len(), total)),
                        )
                        .insert_opt("window", window.as_ref().map(|w| serde_json::json!(w)))
                        .insert("messages", serde_json::json!(messages))
                        .build();
                    output::print_json(obj);
                }
                OutputMode::Plain => {
                    // Serialize only selected messages back to file format
                    // (subset output carries no preamble, BDD:POST-31).
                    let content = paperwork_core::format::thread::serialize_messages(&messages);
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
                        // reply/mentions shown from parse-time derivations
                        // (D2); no `to` output remains (field deleted).
                        let mut header = format!(
                            "#{} {} {}",
                            msg.seq,
                            msg.sender,
                            msg.timestamp.format(paperwork_core::format::RFC3339_FMT)
                        );
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
                    example: format!(
                        "paperwork post send {} --author alice --message \"Hello\"",
                        path.display()
                    ),
                }
                .into());
            }

            // Symmetric foreign-thread guard (review Kim M1): mirror the
            // write side so a stage-1 hit on a non-thread file reports
            // `format` (exit 1) instead of an empty garbage summary.
            reject_foreign_thread(&path)?;

            let summary = paperwork_core::ops::thread::thread_summary(&path)?;

            // Title comes straight from the summary (parsed in the same
            // pass — review M8: no second full-file thread_meta walk);
            // participants are derived from the message sender set (D1,
            // spec §5.4) and carried by the summary.
            let title = summary.title.clone();
            let participants = summary.participants.join(", ");

            let last_snippet = summary.snippets.last().cloned().unwrap_or_default();

            match ctx.mode {
                OutputMode::Json => {
                    let obj = output::JsonBuilder::new()
                        .insert("status", serde_json::json!("ok"))
                        .insert("command", serde_json::json!("post.summary"))
                        .insert("conclusion", serde_json::json!(path.display().to_string()))
                        .insert("title", serde_json::json!(title))
                        .insert("participants", serde_json::json!(participants))
                        .insert("messages", serde_json::json!(summary.message_count))
                        .insert_opt(
                            "last.sender",
                            summary.last_sender.as_ref().map(|s| serde_json::json!(s)),
                        )
                        .insert_opt(
                            "last.time",
                            summary.last_timestamp.map(|t| {
                                serde_json::json!(t
                                    .format(paperwork_core::format::RFC3339_FMT)
                                    .to_string())
                            }),
                        )
                        .insert("last.snippet", serde_json::json!(last_snippet))
                        .build();
                    output::print_json(obj);
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
                        env = env.field(
                            "last.time",
                            &t.format(paperwork_core::format::RFC3339_FMT).to_string(),
                        );
                    }
                    env = env.field("last.snippet", &last_snippet);
                    output::emit_ok(ctx, env);
                }
            }
            Ok(())
        }

        PostCommand::Edit {
            path,
            author,
            seq,
            message,
            stdin,
        } => {
            let path = ensure_suffix(path, ".post.md");

            // Stage-1 hit on a foreign file -> format error, not not-found
            // (mirrors send; spec S-PATH-07, review M2).
            reject_foreign_thread(&path)?;

            let new_body =
                resolve_body(message, stdin, BodyOwner::Edit, &path.display().to_string())?;

            // Reject empty/whitespace-only body (symmetric with send,
            // review Kim m2): editing a message to a blank body is a
            // silent-corruption surface, refuse it the same way send does.
            if new_body.trim().is_empty() {
                return Err(paperwork_core::PaperworkError::Validation {
                    message: "message body is empty".to_string(),
                    fix: "provide a non-empty --message value (bodies starting with '-' are accepted) or pipe content via --stdin".to_string(),
                    example: format!("paperwork post edit {} --author {} --seq {} --message \"corrected body\"", path.display(), author, seq),
                }.into());
            }

            paperwork_core::ops::thread::thread_edit(&path, seq, &author, &new_body)?;

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
///
/// Concurrency (QA BUG-2): the pre-read runs under an fs2 exclusive lock
/// acquired on the SAME handle that performs the read. Under Windows
/// mandatory byte-range locking, reading a byte range another process has
/// locked fails instantly with ERROR_LOCK_VIOLATION (os error 33); the old
/// lock-less pre-read raced concurrent `thread_send` writers and
/// intermittently failed sends, losing messages. Taking the lock first
/// blocks until the writer finishes (exactly like the write path), and
/// reading through the locking handle avoids the violation. Lock/seq
/// semantics are unchanged: core `thread_send`/`thread_edit` still acquire
/// their own lock and own seq allocation, so this check remains advisory
/// (the same TOCTOU window as before).
fn reject_foreign_thread(path: &std::path::Path) -> Result<()> {
    if path.is_file() {
        let file =
            std::fs::File::open(path).map_err(|e| paperwork_core::PaperworkError::IoContext {
                path: path.to_path_buf(),
                source: e,
                fix: "check that the file is readable".to_string(),
                example: String::new(),
            })?;
        file.lock_exclusive()
            .map_err(|e| paperwork_core::PaperworkError::IoContext {
                path: path.to_path_buf(),
                source: e,
                fix: "another process may hold the lock; retry shortly".to_string(),
                example: String::new(),
            })?;

        // Read through the locking handle: on Windows a cross-handle read
        // into a locked byte range fails with os error 33 even inside the
        // same process.
        let mut raw = String::new();
        let mut reader = &file;
        reader
            .read_to_string(&mut raw)
            .map_err(|e| paperwork_core::PaperworkError::IoContext {
                path: path.to_path_buf(),
                source: e,
                fix: "check that the file is readable".to_string(),
                example: String::new(),
            })?;
        file.unlock().ok();

        let pre_existing = paperwork_core::format::thread::parse_messages(&raw)?;
        if pre_existing.is_empty() && !raw.trim().is_empty() {
            return Err(paperwork_core::PaperworkError::Parse {
                message: format!("{} is not a valid post thread: no valid message boundaries found", path.display()),
                fix: "expected an H1 title preamble with `## #N sender timestamp` message headers; or validate it explicitly".to_string(),
                example: format!("paperwork validate {} --type post", path.display()),
            }.into());
        }
    }
    Ok(())
}

/// Default preamble title (spec section 5.7): strip the managed-file
/// suffixes (`.profile.md`, `.post.md`, `.md`) from the file name, else
/// keep the file name as-is.
///
/// NEW-3 (P-6): the suffix stripping runs on the native `OsStr` so a
/// non-Unicode file name is never rewritten by a `to_string_lossy()`
/// roundtrip BEFORE stripping; the final `String` conversion is lossless
/// for every valid-Unicode name (the only shape that can produce legal
/// file content anyway), and falls back to the lossy representation with
/// U+FFFD only for the non-representable remainder.
fn default_title(path: &Path) -> String {
    use std::ffi::OsStr;

    let name: std::ffi::OsString = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_else(|| path.as_os_str().to_os_string());

    let mut stem = name;
    for suffix in [".profile.md", ".post.md", ".md"] {
        if let Some(base) = crate::cmd::os_strip_suffix(&stem, OsStr::new(suffix)) {
            stem = base;
            break;
        }
    }

    stem.into_string()
        .unwrap_or_else(|os| os.to_string_lossy().into_owned())
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
/// Each value is injected verbatim as an `@name` body token (spec section 11 OQ-4)
/// and must survive the spec section 5.4 derivation unchanged:
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
            example: format!("paperwork post send myfile --author {} --mention bob --message \"Hello\"", from),
        }
        .into()),
    }
}

/// Inject reference tokens into the body head (spec section 11 OQ-4, D2).
///
/// `--reply-to N` yields `@#N`; each mention yields `@name`. All tokens sit
/// on a single first line separated by single spaces, followed by a blank
/// line, then the original body - so the spec section 5.4 derivation rules can
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

/// Resolve body from the --message flag or --stdin flag.
///
/// The "exactly one channel" invariant is enforced by clap
/// (required_unless_present + conflicts_with -> usage exit 2); this helper
/// only reads the chosen channel. Error examples are verb-specific: edit
/// failures must not show send-form commands (feasibility review m-2).
fn resolve_body(
    message: Option<String>,
    stdin: bool,
    owner: BodyOwner,
    path: &str,
) -> Result<String> {
    match (message, stdin) {
        (Some(body), false) => Ok(body),
        (None, true) => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
        // Unreachable via clap (conflicts_with / required_unless_present),
        // kept as a defensive arm with verb-specific examples.
        _ => {
            let example = match owner {
                BodyOwner::Send => format!(
                    "paperwork post send {} --author alice --message \"Hello\"",
                    path
                ),
                BodyOwner::Edit => format!(
                    "paperwork post edit {} --author alice --seq 2 --message \"corrected body\"",
                    path
                ),
            };
            Err(paperwork_core::PaperworkError::Validation {
                message: "body channel is ambiguous".to_string(),
                fix: "use either --message or --stdin, not both and not neither".to_string(),
                example,
            }
            .into())
        }
    }
}
