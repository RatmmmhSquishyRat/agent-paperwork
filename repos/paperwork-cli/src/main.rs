//! paperwork-cli: Stateless, path-explicit CLI for Agent Paperwork.
//!
//! Thin CLI layer: parses args -> calls paperwork_core::ops -> formats output.
//! No workspace, no init, no login. Every command takes explicit file paths.
//!
//! v0.6 grammar: PATH is the only positional argument; every required
//! payload is a named flag (--author/--message/--seq for post send/edit,
//! --name, --title, --entry, --entry-title, --profile). clap usage errors
//! render as the standard `usage` error envelope (seventh category) and
//! exit 2; runtime errors keep exit 1.

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
    after_help = "Grammar: paperwork [global flags] <group> <verb> <PATH> --required-flag ... [--optional-flag ...]"
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
            // original semantics -- print and exit 0, never a usage envelope.
            if matches!(err.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                let _ = err.print();
                process::exit(0);
            }

            // --json awareness: no parse result yet, fall back to an argv
            // scan. Values after `--` are positional payloads, never flags.
            let json_mode = std::env::args()
                .skip(1)
                .take_while(|a| a != "--")
                .any(|a| a == "--json");
            let (command, example) = canonical_example();
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

/// Extract the message for a usage envelope.
///
/// Collects every rendered line up to the first blank line or the
/// "For more information" footer, so multi-line messages (e.g. the
/// MissingRequiredArgument list of missing arguments) are not truncated.
/// MissingSubcommand gets an explicit "missing subcommand" message (clap's
/// default rendering shows about text instead); the default and --json
/// envelopes share this exact message. clap reports a bare group invocation
/// as DisplayHelpOnMissingArgumentOrSubcommand, which maps to the same shape.
fn usage_message(err: &clap::Error) -> String {
    if matches!(
        err.kind(),
        ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    ) {
        return missing_subcommand_message();
    }
    let rendered = err.render().to_string();
    let mut parts: Vec<&str> = Vec::new();
    for line in rendered.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("For more information") {
            break;
        }
        parts.push(trimmed);
    }
    let joined = parts.join(" ");
    joined
        .strip_prefix("error:")
        .unwrap_or(&joined)
        .trim()
        .to_string()
}

/// Explicit "missing subcommand" message for top-level and group-level
/// MissingSubcommand failures (S-OUT-06).
fn missing_subcommand_message() -> String {
    let group = std::env::args()
        .skip(1)
        .take_while(|a| a != "--")
        .find(|a| !a.starts_with('-'));
    let canonical = match group.as_deref() {
        Some("profile") | Some("p") => Some("profile"),
        Some("post") | Some("po") => Some("post"),
        Some("brief") | Some("b") => Some("brief"),
        Some("contacts") | Some("c") => Some("contacts"),
        Some("validate") | Some("v") => Some("validate"),
        _ => None,
    };
    match canonical {
        Some(g) => format!(
            "missing subcommand for group '{}'; run 'paperwork {} --help' to list its verbs",
            g, g
        ),
        None => "missing subcommand: expected one of profile, post, brief, contacts, validate".to_string(),
    }
}

/// Build the fix line for a usage envelope.
///
/// v0.6: required values are named flags. Unknown-argument errors carry
/// migration teaching; a suspected body value starting with '-' is guided
/// to the `--message` flag (allow_hyphen_values), the `--` boundary form
/// is retired.
fn usage_fix(err: &clap::Error, command: &str) -> String {
    let base = "required values are named flags (--author/--message for post send/edit); see the canonical example below";
    let unknown = extract_unknown_argument(err);
    match unknown.as_deref() {
        // Long unknown flag (--from as identity, pre-v0.6 leftovers, ...):
        // old-grammar migration case. v0.6 re-legalized --seq/--title/
        // --entry/--entry-title/--name/--profile, so they no longer appear
        // in the teaching list.
        Some(tok) if tok.starts_with("--") => format!(
            "{}; this flag is not recognized; if it came from older grammar, give the value via the matching named flag (e.g. --author for the sender)",
            base
        ),
        // Short/dash token: suspected body value starting with '-'
        Some(_) if matches!(command, "post.send" | "post.edit") => format!(
            "{}; if a body value starts with '-', pass it via --message (e.g. paperwork post send standup.post.md --author alice --message \"-fix flag text\")",
            base
        ),
        Some(_) => format!(
            "{}; values are given via their named flags, not as bare tokens",
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
/// Every example is the single v0.6 named-flag normative form (F5).
fn canonical_example() -> (&'static str, &'static str) {
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
        _ => return ("usage", "paperwork post send standup.post.md --author alice --message \"Hello\""),
    };
    let verb = tokens.get(1).map(String::as_str);

    match (group, verb) {
        ("post", Some("send")) => (
            "post.send",
            "paperwork post send standup.post.md --author alice --message \"Parser module is 80% done.\"",
        ),
        ("post", Some("edit")) => (
            "post.edit",
            "paperwork post edit standup.post.md --author alice --seq 3 --message \"corrected body\"",
        ),
        ("post", Some("read")) => (
            "post.read",
            "paperwork post read standup.post.md --from 5 --to 20",
        ),
        ("post", Some("summary")) => ("post.summary", "paperwork post summary standup.post.md"),
        ("post", _) => ("usage", "paperwork post send standup.post.md --author alice --message \"Hello\""),
        ("profile", Some("create")) => (
            "profile.create",
            "paperwork profile create agents/alice --name alice --model gpt-4o",
        ),
        ("profile", Some("show")) => ("profile.show", "paperwork profile show agents/alice.profile.md"),
        ("profile", Some("edit")) => (
            "profile.edit",
            "paperwork profile edit agents/alice.profile.md --model gpt-4o",
        ),
        ("profile", Some("list")) => ("profile.list", "paperwork profile list agents"),
        ("profile", _) => ("usage", "paperwork profile create agents/alice --name alice --model gpt-4o"),
        ("brief", Some("create")) => (
            "brief.create",
            "paperwork brief create onboarding --title \"Codebase Onboarding\" --owner alice",
        ),
        ("brief", Some("add")) => (
            "brief.add",
            "paperwork brief add onboarding.brief.md --entry src/main.rs --regex \"fn main\"",
        ),
        ("brief", Some("remove")) => (
            "brief.remove",
            "paperwork brief remove onboarding.brief.md --entry-title main.rs",
        ),
        ("brief", Some("read")) => ("brief.read", "paperwork brief read onboarding.brief.md"),
        ("brief", Some("verify")) => ("brief.verify", "paperwork brief verify onboarding.brief.md"),
        ("brief", _) => ("usage", "paperwork brief create onboarding --title \"Codebase Onboarding\" --owner alice"),
        ("contacts", Some("create")) => (
            "contacts.create",
            "paperwork contacts create team --title \"Core Team\"",
        ),
        ("contacts", Some("add")) => (
            "contacts.add",
            "paperwork contacts add team.contacts.md --profile agents/alice.profile.md",
        ),
        ("contacts", Some("read")) => ("contacts.read", "paperwork contacts read team.contacts.md"),
        ("contacts", _) => ("usage", "paperwork contacts create team --title \"Core Team\""),
        ("validate", _) => ("validate", "paperwork validate mystery.md --type post"),
        _ => ("usage", "paperwork post send standup.post.md --author alice --message \"Hello\""),
    }
}
