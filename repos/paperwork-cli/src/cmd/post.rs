//! `paperwork post` — group message create/send/read/summary/list.
//!
//! Command structure:
//! - paperwork post create <name> --participants <a,b,c> [--title <text>]
//! - paperwork post <name> send <body> [--mention <agent>] [--reply-to <seq>]
//! - paperwork post <name> read [--from <seq>] [--to <seq>]
//! - paperwork post <name> summary
//! - paperwork post list

use std::fs;

use anyhow::Result;
use chrono::Utc;
use clap::Args;
use serde::Serialize;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

/// Post (group message) operations
#[derive(Args)]
pub struct PostArgs {
    /// Post name or subcommand (create/list)
    pub name_or_cmd: String,

    /// Action (send/read/summary) or arguments for create
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub rest: Vec<String>,
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
    title: String,
    participants: Vec<String>,
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
    post: String,
    title: String,
    participants: Vec<String>,
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

#[derive(Serialize)]
struct PostListItem {
    name: String,
    title: String,
    participants: Vec<String>,
    path: String,
}

fn parse_post_meta(root: &std::path::Path, name: &str) -> Result<(String, Vec<String>)> {
    let meta_path = paperwork_core::layout::post_meta_path(root, name);
    if !meta_path.exists() {
        anyhow::bail!(
            "post \"{}\" not found\n  \u{2192} run: paperwork post create {}",
            name,
            name
        );
    }
    let content = fs::read_to_string(&meta_path)?;
    let mut title = name.to_string();
    let mut participants = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("**Title**: ") {
            title = val.trim().to_string();
        } else if let Some(val) = trimmed.strip_prefix("**Participants**: ") {
            participants = val
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    Ok((title, participants))
}

/// Extract a flag value from args: --flag value or --flag=value
fn get_flag(args: &[String], flag: &str) -> Option<String> {
    let flag_eq = format!("{}=", flag);
    for (i, arg) in args.iter().enumerate() {
        if arg == flag {
            return args.get(i + 1).cloned();
        }
        if let Some(val) = arg.strip_prefix(&flag_eq) {
            return Some(val.to_string());
        }
    }
    None
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

pub fn run(ctx: &Context, args: PostArgs) -> Result<()> {
    match args.name_or_cmd.as_str() {
        "create" => run_create(ctx, &args.rest),
        "list" => run_list(ctx),
        name => {
            // It's a post name, next arg is the action
            let action = args.rest.first().map(|s| s.as_str()).unwrap_or("");
            let action_args: Vec<String> = args.rest.iter().skip(1).cloned().collect();
            match action {
                "send" => run_send(ctx, name, &action_args),
                "read" => run_read(ctx, name, &action_args),
                "summary" => run_summary(ctx, name),
                "" => anyhow::bail!(
                    "Missing action for post \"{}\".\n  \u{2192} usage: paperwork post {} send|read|summary",
                    name,
                    name
                ),
                other => anyhow::bail!(
                    "Unknown action \"{}\" for post.\n  \u{2192} valid actions: send, read, summary",
                    other
                ),
            }
        }
    }
}

fn run_create(ctx: &Context, args: &[String]) -> Result<()> {
    // First positional is name
    let name = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing post name.\n  \u{2192} usage: paperwork post create <name> --participants <a,b,c>"
            )
        })?;

    let participants_str = get_flag(args, "--participants").ok_or_else(|| {
        anyhow::anyhow!(
            "Missing --participants.\n  \u{2192} usage: paperwork post create {} --participants alice,bob",
            name
        )
    })?;

    let title = get_flag(args, "--title");

    let post_dir = paperwork_core::layout::post_dir(&ctx.root, &name);
    if post_dir.exists() {
        anyhow::bail!(
            "post \"{}\" already exists\n  \u{2192} use: paperwork post {} send \"message\"",
            name,
            name
        );
    }

    let participants_list: Vec<&str> = participants_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if participants_list.is_empty() {
        anyhow::bail!(
            "Missing participants.\n  \u{2192} usage: paperwork post create {} --participants alice,bob",
            name
        );
    }

    let post_title = title.unwrap_or_else(|| name.clone());

    // Create post directory
    fs::create_dir_all(&post_dir)?;

    // Create meta.md
    let meta_path = paperwork_core::layout::post_meta_path(&ctx.root, &name);
    let meta_content = format!(
        "# Post: {}\n\n**Created**: {}  \n**Participants**: {}  \n**Title**: {}\n",
        name,
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        participants_list.join(", "),
        post_title
    );
    fs::write(&meta_path, meta_content)?;

    // Create empty log.md
    let log_path = paperwork_core::layout::post_log_path(&ctx.root, &name);
    fs::write(&log_path, "")?;

    match ctx.mode {
        OutputMode::Json => {
            let out = serde_json::json!({
                "created": format!("posts/{}/", name),
                "title": post_title,
                "participants": participants_list,
            });
            output::print_json(&out);
        }
        _ => output::success(ctx, &format!("post created: posts/{}/", name)),
    }

    Ok(())
}

fn run_send(ctx: &Context, name: &str, args: &[String]) -> Result<()> {
    let me = ctx.current_agent()?;
    let thread_rel = format!("posts/{}/log.md", name);

    // Check post exists
    let log_path = paperwork_core::layout::post_log_path(&ctx.root, name);
    if !log_path.exists() {
        anyhow::bail!(
            "post \"{}\" not found\n  \u{2192} run: paperwork post create {}",
            name,
            name
        );
    }

    // Get body: first non-flag arg or --stdin
    let body_text = if has_flag(args, "--stdin") {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf.trim_end().to_string()
    } else {
        args.iter()
            .find(|a| !a.starts_with('-'))
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Missing message body.\n  \u{2192} usage: paperwork post {} send \"message\"",
                    name
                )
            })?
    };

    let mention = get_flag(args, "--mention");
    let reply_to = get_flag(args, "--reply-to").and_then(|v| v.parse().ok());

    let msg = paperwork_core::Message {
        seq: 0,
        sender: me.clone(),
        timestamp: Utc::now(),
        to: vec![], // empty = "all" (broadcast)
        reply_to,
        body: body_text.clone(),
    };

    paperwork_core::ops::thread::append_msg(&ctx.root, &thread_rel, &msg)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

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

    Ok(())
}

fn run_read(ctx: &Context, name: &str, args: &[String]) -> Result<()> {
    let thread_rel = format!("posts/{}/log.md", name);
    let (title, participants) = parse_post_meta(&ctx.root, name)?;

    let summary = paperwork_core::ops::thread::summary(&ctx.root, &thread_rel)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let total = summary.message_count;

    if total == 0 {
        match ctx.mode {
            OutputMode::Json => {
                let out = ReadOutput {
                    thread: thread_rel,
                    title,
                    participants,
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

    let from_seq = get_flag(args, "--from")
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| total.saturating_sub(9).max(1));
    let to_seq = get_flag(args, "--to")
        .and_then(|v| v.parse().ok())
        .unwrap_or(total);

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
                title,
                participants,
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
            let log_path = paperwork_core::layout::resolve_thread_path(&ctx.root, &thread_rel);
            let content = fs::read_to_string(&log_path)?;
            output::print_plain(&content);
        }
        OutputMode::Default => {
            let mut out = format!(
                "\u{2500}\u{2500} {} \u{2500}\u{2500} {} messages \u{2500}\u{2500} showing #{}\u{2013}#{} \u{2500}\u{2500}\n",
                thread_rel, total, from_seq, to_seq
            );
            for m in &messages {
                let reply_marker = m
                    .reply_to
                    .map(|r| format!("  \u{21a9}#{}", r))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "\n#{}  {} \u{2192} all   {}{}\n",
                    m.seq,
                    m.sender,
                    m.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                    reply_marker
                ));
                out.push_str(&format!("     {}\n", m.body.replace('\n', "\n     ")));
            }
            output::print_default(out.trim_end());
        }
    }

    Ok(())
}

fn run_summary(ctx: &Context, name: &str) -> Result<()> {
    let thread_rel = format!("posts/{}/log.md", name);
    let (title, participants) = parse_post_meta(&ctx.root, name)?;

    let summary = paperwork_core::ops::thread::summary(&ctx.root, &thread_rel)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match ctx.mode {
        OutputMode::Json => {
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
                post: name.to_string(),
                title,
                participants,
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
            let mut out = format!("post: {} \u{2014} \"{}\"\n", name, title);
            out.push_str(&format!(
                "participants: {} ({})\n",
                participants.join(", "),
                participants.len()
            ));
            out.push_str(&format!("messages: {}\n", summary.message_count));
            if let (Some(sender), Some(ts)) = (&summary.last_sender, &summary.last_timestamp) {
                out.push_str(&format!(
                    "last: {} \u{00b7} {}\n",
                    sender,
                    ts.format("%Y-%m-%dT%H:%M:%SZ")
                ));
            }
            if summary.message_count > 0 {
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
                        out.push_str(&format!("  #{} {}: \"{}\"\n", m.seq, m.sender, preview));
                    }
                }
            }
            output::print_default(out.trim_end());
        }
    }

    Ok(())
}

fn run_list(ctx: &Context) -> Result<()> {
    let posts_dir = paperwork_core::layout::posts_dir(&ctx.root);
    let mut posts: Vec<PostListItem> = Vec::new();

    if posts_dir.exists() {
        let entries = fs::read_dir(&posts_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    if let Ok((title, participants)) = parse_post_meta(&ctx.root, &name) {
                        posts.push(PostListItem {
                            name: name.clone(),
                            title,
                            participants,
                            path: format!("posts/{}/", name),
                        });
                    }
                }
            }
        }
    }

    posts.sort_by(|a, b| a.name.cmp(&b.name));

    match ctx.mode {
        OutputMode::Json => {
            output::print_json(&posts);
        }
        _ => {
            if posts.is_empty() {
                output::print_default("No posts found.");
            } else {
                let mut out = String::from("POST      TITLE           PARTICIPANTS\n");
                for p in &posts {
                    out.push_str(&format!(
                        "{:<10}{:<16}{}\n",
                        p.name,
                        p.title,
                        p.participants.join(", ")
                    ));
                }
                output::print_default(out.trim_end());
            }
        }
    }

    Ok(())
}
