//! `paperwork dm` — direct message send/read/edit/summary.

use anyhow::Result;
use chrono::Utc;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// Direct message operations
#[derive(Args)]
pub struct DmArgs {
    /// Target agent
    pub agent: String,

    #[command(subcommand)]
    pub command: DmCommand,
}

#[derive(Subcommand)]
pub enum DmCommand {
    /// Send a message
    Send {
        /// Message body
        body: Option<String>,
        /// Read body from stdin
        #[arg(long)]
        stdin: bool,
        /// Mention an agent (triggers notification)
        #[arg(long)]
        mention: Option<String>,
        /// Reply to a specific message seq
        #[arg(long)]
        reply_to: Option<u64>,
    },
    /// Read messages (default: last 10)
    Read {
        /// Start seq (inclusive)
        #[arg(long)]
        from: Option<u64>,
        /// End seq (inclusive)
        #[arg(long)]
        to: Option<u64>,
    },
    /// Edit your own last message
    Edit {
        /// Message seq to edit
        seq: u64,
        /// New body
        body: Option<String>,
        /// Read body from stdin
        #[arg(long)]
        stdin: bool,
    },
    /// Show thread summary
    Summary,
}

#[derive(Serialize)]
struct MessageJson {
    seq: u64,
    sender: String,
    to: Vec<String>,
    timestamp: String,
    reply_to: Option<u64>,
    body: String,
}

#[derive(Serialize)]
struct ReadOutput {
    thread: String,
    total: u64,
    showing: ShowingRange,
    messages: Vec<MessageJson>,
}

#[derive(Serialize)]
struct ShowingRange {
    from: u64,
    to: u64,
}

#[derive(Serialize)]
struct SummaryOutput {
    thread: String,
    message_count: u64,
    last_sender: Option<String>,
    last_timestamp: Option<String>,
    snippets: Vec<SnippetJson>,
}

#[derive(Serialize)]
struct SnippetJson {
    seq: u64,
    sender: String,
    preview: String,
}

fn get_body(body: Option<String>, stdin: bool) -> Result<String> {
    if stdin {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf.trim_end().to_string())
    } else if let Some(b) = body {
        Ok(b)
    } else {
        anyhow::bail!(
            "Missing message body.\n  \u{2192} usage: paperwork dm <agent> send \"message\" or --stdin"
        )
    }
}

pub fn run(ctx: &Context, args: DmArgs) -> Result<()> {
    let me = ctx.current_agent()?;
    let thread_rel = ctx.dm_thread_rel(&args.agent)?;

    match args.command {
        DmCommand::Send {
            body,
            stdin,
            mention,
            reply_to,
        } => {
            let body_text = get_body(body, stdin)?;

            // Check DM folder exists
            let thread_path = paperwork_core::layout::resolve_thread_path(&ctx.root, &thread_rel);
            if !thread_path.exists() {
                anyhow::bail!(
                    "cannot send: dm folder {} does not exist\n  \u{2192} run: paperwork invite {}",
                    thread_rel,
                    args.agent
                );
            }

            let msg = paperwork_core::Message {
                seq: 0, // Will be assigned by core
                sender: me.clone(),
                timestamp: Utc::now(),
                to: vec![args.agent.clone()],
                reply_to,
                body: body_text.clone(),
            };

            paperwork_core::ops::thread::append_msg(&ctx.root, &thread_rel, &msg)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            // Get assigned seq by reading summary
            let summary = paperwork_core::ops::thread::summary(&ctx.root, &thread_rel)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let assigned_seq = summary.message_count;

            // Handle mention notification
            if let Some(mentioned) = &mention {
                let notification = paperwork_core::Notification {
                    timestamp: Utc::now(),
                    from: me.clone(),
                    thread_path: thread_rel.clone(),
                    seq: assigned_seq,
                    notify_type: paperwork_core::NotifyType::Mention,
                    snippet: body_text.chars().take(80).collect(),
                };
                paperwork_core::ops::notify::push_notify(&ctx.root, mentioned, &notification)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
            }

            // Handle reply-to notification (implicit mention of original sender)
            if reply_to.is_some() {
                // Notify the target agent about the reply
                let notification = paperwork_core::Notification {
                    timestamp: Utc::now(),
                    from: me.clone(),
                    thread_path: thread_rel.clone(),
                    seq: assigned_seq,
                    notify_type: paperwork_core::NotifyType::Reply,
                    snippet: body_text.chars().take(80).collect(),
                };
                paperwork_core::ops::notify::push_notify(&ctx.root, &args.agent, &notification)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
            }

            match ctx.mode {
                OutputMode::Json => {
                    let out = serde_json::json!({
                        "seq": assigned_seq,
                        "thread": thread_rel,
                    });
                    output::print_json(&out);
                }
                _ => output::success(
                    ctx,
                    &format!("sent #{} \u{2192} {}", assigned_seq, thread_rel),
                ),
            }
        }
        DmCommand::Read { from, to } => {
            // Get total count first
            let summary = paperwork_core::ops::thread::summary(&ctx.root, &thread_rel)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let total = summary.message_count;

            if total == 0 {
                match ctx.mode {
                    OutputMode::Json => {
                        let out = ReadOutput {
                            thread: thread_rel,
                            total: 0,
                            showing: ShowingRange { from: 0, to: 0 },
                            messages: vec![],
                        };
                        output::print_json(&out);
                    }
                    _ => output::print_default(&format!(
                        "\u{2500}\u{2500} {} \u{2500}\u{2500} 0 messages \u{2500}\u{2500}",
                        thread_rel
                    )),
                }
                return Ok(());
            }

            // Default: last 10
            let from_seq = from.unwrap_or_else(|| total.saturating_sub(9).max(1));
            let to_seq = to.unwrap_or(total);

            let messages =
                paperwork_core::ops::thread::read_range(&ctx.root, &thread_rel, from_seq, to_seq)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let msgs: Vec<MessageJson> = messages
                        .iter()
                        .map(|m| MessageJson {
                            seq: m.seq,
                            sender: m.sender.clone(),
                            to: m.to.clone(),
                            timestamp: m.timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            reply_to: m.reply_to,
                            body: m.body.clone(),
                        })
                        .collect();
                    let out = ReadOutput {
                        thread: thread_rel,
                        total,
                        showing: ShowingRange {
                            from: from_seq,
                            to: to_seq,
                        },
                        messages: msgs,
                    };
                    output::print_json(&out);
                }
                OutputMode::Plain => {
                    let thread_path =
                        paperwork_core::layout::resolve_thread_path(&ctx.root, &thread_rel);
                    let content = std::fs::read_to_string(&thread_path)?;
                    output::print_plain(&content);
                }
                OutputMode::Default => {
                    let mut out = format!(
                        "\u{2500}\u{2500} {} \u{2500}\u{2500} {} messages \u{2500}\u{2500}\n",
                        thread_rel, total
                    );
                    for m in &messages {
                        let reply_marker = m
                            .reply_to
                            .map(|r| format!("  \u{21a9}#{}", r))
                            .unwrap_or_default();
                        out.push_str(&format!(
                            "\n#{}  {} \u{2192} {}   {}{}\n",
                            m.seq,
                            m.sender,
                            if m.to.is_empty() {
                                "all".to_string()
                            } else {
                                m.to.join(", ")
                            },
                            m.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                            reply_marker
                        ));
                        out.push_str(&format!("    {}\n", m.body.replace('\n', "\n    ")));
                    }
                    output::print_default(out.trim_end());
                }
            }
        }
        DmCommand::Edit { seq, body, stdin } => {
            let new_body = get_body(body, stdin)?;

            paperwork_core::ops::thread::self_edit(&ctx.root, &thread_rel, seq, &me, &new_body)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    let out = serde_json::json!({
                        "edited": seq,
                        "thread": thread_rel,
                    });
                    output::print_json(&out);
                }
                _ => output::success(ctx, &format!("edited #{} \u{2192} {}", seq, thread_rel)),
            }
        }
        DmCommand::Summary => {
            let summary = paperwork_core::ops::thread::summary(&ctx.root, &thread_rel)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match ctx.mode {
                OutputMode::Json => {
                    // Get last 3 messages for snippets with seq info
                    let snippets = if summary.message_count > 0 {
                        let from = summary.message_count.saturating_sub(2).max(1);
                        let msgs = paperwork_core::ops::thread::read_range(
                            &ctx.root,
                            &thread_rel,
                            from,
                            summary.message_count,
                        )
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                        msgs.iter()
                            .map(|m| SnippetJson {
                                seq: m.seq,
                                sender: m.sender.clone(),
                                preview: m.body.chars().take(50).collect(),
                            })
                            .collect()
                    } else {
                        vec![]
                    };
                    let out = SummaryOutput {
                        thread: thread_rel,
                        message_count: summary.message_count,
                        last_sender: summary.last_sender,
                        last_timestamp: summary
                            .last_timestamp
                            .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                        snippets,
                    };
                    output::print_json(&out);
                }
                _ => {
                    let mut out = format!(
                        "{} \u{2014} {} messages\n",
                        thread_rel, summary.message_count
                    );
                    if let (Some(sender), Some(ts)) =
                        (&summary.last_sender, &summary.last_timestamp)
                    {
                        out.push_str(&format!(
                            "last: {} \u{00b7} {}\n",
                            sender,
                            ts.format("%Y-%m-%dT%H:%M:%SZ")
                        ));
                    }
                    if !summary.snippets.is_empty() {
                        out.push_str("recent:\n");
                        let from = summary.message_count.saturating_sub(2).max(1);
                        if let Ok(msgs) = paperwork_core::ops::thread::read_range(
                            &ctx.root,
                            &thread_rel,
                            from,
                            summary.message_count,
                        ) {
                            for m in &msgs {
                                let preview: String = m.body.chars().take(50).collect();
                                out.push_str(&format!(
                                    "  #{} {}: \"{}\"\n",
                                    m.seq, m.sender, preview
                                ));
                            }
                        }
                    }
                    output::print_default(out.trim_end());
                }
            }
        }
    }

    Ok(())
}
