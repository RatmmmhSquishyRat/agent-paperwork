//! paperwork-cli: Stateless, path-explicit CLI for Agent Paperwork.
//!
//! Thin CLI layer: parses args -> calls paperwork_core::ops -> formats output.
//! No workspace, no init, no login. Every command takes explicit file paths.
//!
//! v0.5.0: clap usage errors render as the standard `usage` error envelope
//! (seventh category) and exit 2; runtime errors keep exit 1.

mod cmd;
mod output;

use std::process;

use clap::{error::ErrorKind, Parser, Subcommand};

use crate::output::OutputMode;

/// Agent Paperwork - stateless file-based collaboration toolkit for AI agents.
#[derive(Parser)]
#[command(
    name = "paperwork",
    version,
    about,
    long_about = None,
    after_help = "Grammar: paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]"
)]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Output raw file content
    #[arg(long, global = true)]
    plain: bool,

    /// Suppress status line (still outputs fields and body)
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage agent profiles
    #[command(alias = "p")]
    Profile(cmd::profile::ProfileArgs),

    /// Post (group thread) operations - also covers 1:1 conversations
    #[command(alias = "po")]
    Post(cmd::post::PostArgs),

    /// Brief (reading list / knowledge brief) operations
    #[command(alias = "b")]
    Brief(cmd::brief::BriefArgs),

    /// Contacts operations
    #[command(alias = "c")]
    Contacts(cmd::contacts::ContactsArgs),

    /// Validate Markdown structure of a file
    #[command(alias = "v")]
    Validate(cmd::validate::ValidateArgs),
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Pass-through (F5): --help/-h (all levels) and -V keep clap's
            // original semantics — print and exit 0, never a usage envelope.
            if matches!(err.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                let _ = err.print();
                process::exit(0);
            }

            // --json awareness: no parse result yet, fall back to an argv scan.
            let json_mode = std::env::args().any(|a| a == "--json");
            let unknown = extract_unknown_argument(&err);
            // Suspected body value starting with '-' (short unknown token,
            // not an old long flag): the example must demonstrate `--` (NF-2).
            let dash_body = unknown
                .as_deref()
                .is_some_and(|t| t.starts_with('-') && !t.starts_with("--"));
            let (command, example) = canonical_example(dash_body);
            let message = usage_message(&err);
            let fix = usage_fix(&err, command);
            output::emit_usage_error(json_mode, command, &message, &fix, example);
            process::exit(2);
        }
    };

    let mode = if cli.json {
        OutputMode::Json
    } else if cli.plain {
        OutputMode::Plain
    } else {
        OutputMode::Default
    };

    let ctx = cmd::Context {
        mode,
        quiet: cli.quiet,
    };

    let (result, command_id) = match cli.command {
        Commands::Profile(args) => {
            let id = cmd::profile::command_id(&args);
            (cmd::profile::run(&ctx, args), id)
        }
        Commands::Post(args) => {
            let id = cmd::post::command_id(&args);
            (cmd::post::run(&ctx, args), id)
        }
        Commands::Brief(args) => {
            let id = cmd::brief::command_id(&args);
            (cmd::brief::run(&ctx, args), id)
        }
        Commands::Contacts(args) => {
            let id = cmd::contacts::command_id(&args);
            (cmd::contacts::run(&ctx, args), id)
        }
        Commands::Validate(args) => {
            let id = cmd::validate::command_id(&args);
            (cmd::validate::run(&ctx, args), id)
        }
    };

    if let Err(e) = result {
        // Try to downcast to PaperworkError for structured output
        if let Some(pw_err) = e.downcast_ref::<paperwork_core::PaperworkError>() {
            output::emit_err(
                &ctx,
                command_id,
                pw_err.category(),
                &pw_err.to_string(),
                &pw_err.fix(),
                &pw_err.example(),
            );
        } else {
            // Generic anyhow error
            output::emit_err(&ctx, command_id, "io", &e.to_string(), "", "");
        }
        process::exit(1);
    }
}

/// Extract a single-line message from a clap usage error.
fn usage_message(err: &clap::Error) -> String {
    let rendered = err.render().to_string();
    let first = rendered.lines().next().unwrap_or("usage error").trim();
    first
        .strip_prefix("error:")
        .unwrap_or(first)
        .trim()
        .to_string()
}

/// Build the fix line for a usage envelope.
///
/// Unknown-argument errors involving a suspected flag carry `--` boundary
/// teaching (NF-2): bodies starting with '-' must be placed after `--`.
fn usage_fix(err: &clap::Error, command: &str) -> String {
    let base = "required values are positional (PATH first; NAME second for post send/edit); see the canonical example below";
    let unknown = extract_unknown_argument(err);
    match unknown.as_deref() {
        // Long unknown flag (--from, --name, ...): old-grammar migration case
        Some(tok) if tok.starts_with("--") => format!(
            "{}; this flag no longer exists in v0.5 grammar — give the value as a positional argument",
            base
        ),
        // Short/dash token: suspected body value starting with '-'
        Some(_) if matches!(command, "post.send" | "post.edit") => format!(
            "{}; if a body value starts with '-', place it after -- (e.g. paperwork post send standup.post.md alice -- \"-fix flag text\")",
            base
        ),
        Some(_) => format!(
            "{}; if a value starts with '-', place it after --",
            base
        ),
        None => base.to_string(),
    }
}

/// Return the unexpected argument token from a clap UnknownArgument error.
fn extract_unknown_argument(err: &clap::Error) -> Option<String> {
    let rendered = err.render().to_string();
    let marker = "unexpected argument '";
    let start = rendered.find(marker)? + marker.len();
    let rest = &rendered[start..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

/// Map the argv (group, verb) pair to the command identifier and a static
/// canonical, copy-paste-executable example (F2: never carries user argv
/// values; migration teaching is delegated to example + SKILL.md + after_help).
///
/// Top-level parse failures (group/verb undetermined) use command = "usage".
///
/// When `dash_body` is set (suspected body value starting with '-'), the
/// post send/edit example switches to the `--` boundary form (NF-2).
fn canonical_example(dash_body: bool) -> (&'static str, &'static str) {
    let mut tokens: Vec<String> = Vec::new();
    for arg in std::env::args().skip(1) {
        if arg == "--" {
            break;
        }
        if !arg.starts_with('-') {
            tokens.push(arg);
            if tokens.len() == 2 {
                break;
            }
        }
    }

    let group = match tokens.first().map(String::as_str) {
        Some("profile") | Some("p") => "profile",
        Some("post") | Some("po") => "post",
        Some("brief") | Some("b") => "brief",
        Some("contacts") | Some("c") => "contacts",
        Some("validate") | Some("v") => "validate",
        _ => return ("usage", "paperwork post send standup.post.md alice \"Hello\""),
    };
    let verb = tokens.get(1).map(String::as_str);

    match (group, verb) {
        ("post", Some("send")) => {
            if dash_body {
                (
                    "post.send",
                    "paperwork post send standup.post.md alice -- \"-fix flag text\"",
                )
            } else {
                (
                    "post.send",
                    "paperwork post send standup.post.md alice \"Parser module is 80% done.\"",
                )
            }
        }
        ("post", Some("edit")) => {
            if dash_body {
                (
                    "post.edit",
                    "paperwork post edit standup.post.md alice 3 -- \"-starts with dash\"",
                )
            } else {
                (
                    "post.edit",
                    "paperwork post edit standup.post.md alice 3 \"corrected body\"",
                )
            }
        }
        ("post", Some("read")) => (
            "post.read",
            "paperwork post read standup.post.md --from 5 --to 20",
        ),
        ("post", Some("create")) => (
            "post.create",
            "paperwork post create standup \"Daily Standup\" --participants alice,bob",
        ),
        ("post", Some("summary")) => ("post.summary", "paperwork post summary standup.post.md"),
        ("post", _) => ("usage", "paperwork post send standup.post.md alice \"Hello\""),
        ("profile", Some("create")) => (
            "profile.create",
            "paperwork profile create agents/alice alice --model gpt-4o",
        ),
        ("profile", Some("show")) => ("profile.show", "paperwork profile show agents/alice.profile.md"),
        ("profile", Some("edit")) => (
            "profile.edit",
            "paperwork profile edit agents/alice.profile.md --model gpt-4o",
        ),
        ("profile", Some("list")) => ("profile.list", "paperwork profile list agents"),
        ("profile", _) => ("usage", "paperwork profile create agents/alice alice --model gpt-4o"),
        ("brief", Some("create")) => (
            "brief.create",
            "paperwork brief create onboarding \"Codebase Onboarding\" --owner alice",
        ),
        ("brief", Some("add")) => (
            "brief.add",
            "paperwork brief add onboarding.brief.md src/main.rs --regex \"fn main\"",
        ),
        ("brief", Some("remove")) => (
            "brief.remove",
            "paperwork brief remove onboarding.brief.md main.rs",
        ),
        ("brief", Some("read")) => ("brief.read", "paperwork brief read onboarding.brief.md"),
        ("brief", Some("verify")) => ("brief.verify", "paperwork brief verify onboarding.brief.md"),
        ("brief", _) => ("usage", "paperwork brief create onboarding \"Codebase Onboarding\" --owner alice"),
        ("contacts", Some("create")) => (
            "contacts.create",
            "paperwork contacts create team --title \"Core Team\"",
        ),
        ("contacts", Some("add")) => (
            "contacts.add",
            "paperwork contacts add team.contacts.md agents/alice.profile.md",
        ),
        ("contacts", Some("read")) => ("contacts.read", "paperwork contacts read team.contacts.md"),
        ("contacts", _) => ("usage", "paperwork contacts create team --title \"Core Team\""),
        ("validate", _) => ("validate", "paperwork validate standup.post.md"),
        _ => ("usage", "paperwork post send standup.post.md alice \"Hello\""),
    }
}
