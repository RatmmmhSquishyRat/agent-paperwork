//! paperwork-cli: Command-line interface for Agent Paperwork.
//!
//! Thin CLI layer: parses args → calls paperwork_core::ops → formats output.

mod cmd;
mod output;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use crate::output::OutputMode;

/// Agent Paperwork — file-based collaboration toolkit for AI agent teams.
#[derive(Parser)]
#[command(name = "paperwork", version, about, long_about = None)]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Output raw file content
    #[arg(long, global = true)]
    plain: bool,

    /// Disable ANSI colors
    #[arg(long, global = true)]
    no_color: bool,

    /// Suppress confirmation messages
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Workspace root directory (default: current directory)
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a .paperwork/ workspace
    Init(cmd::init::InitArgs),

    /// Manage agent profiles
    #[command(alias = "p")]
    Profile(cmd::profile::ProfileArgs),

    /// Invite an agent to the workspace
    Invite(cmd::invite::InviteArgs),

    /// List all registered agents
    #[command(alias = "c")]
    Contacts(cmd::contacts::ContactsArgs),

    /// Query scope ownership
    #[command(alias = "w")]
    Who(cmd::who::WhoArgs),

    /// Direct message operations
    #[command(alias = "d")]
    Dm(cmd::dm::DmArgs),

    /// Post (group message) operations
    #[command(alias = "g")]
    Post(cmd::post::PostArgs),

    /// Manifest operations
    #[command(alias = "m")]
    Manifest(cmd::manifest::ManifestArgs),

    /// Notification operations
    #[command(alias = "n")]
    Notify(cmd::notify::NotifyArgs),
}

fn main() {
    let cli = Cli::parse();

    let mode = if cli.json {
        OutputMode::Json
    } else if cli.plain {
        OutputMode::Plain
    } else {
        OutputMode::Default
    };

    let root = cli.root.unwrap_or_else(|| PathBuf::from("."));

    let ctx = cmd::Context {
        root,
        mode,
        quiet: cli.quiet,
    };

    let result = match cli.command {
        Commands::Init(args) => cmd::init::run(&ctx, args),
        Commands::Profile(args) => cmd::profile::run(&ctx, args),
        Commands::Invite(args) => cmd::invite::run(&ctx, args),
        Commands::Contacts(args) => cmd::contacts::run(&ctx, args),
        Commands::Who(args) => cmd::who::run(&ctx, args),
        Commands::Dm(args) => cmd::dm::run(&ctx, args),
        Commands::Post(args) => cmd::post::run(&ctx, args),
        Commands::Manifest(args) => cmd::manifest::run(&ctx, args),
        Commands::Notify(args) => cmd::notify::run(&ctx, args),
    };

    if let Err(e) = result {
        output::print_error(&ctx, &e);
        let code = if e.is::<clap::Error>() { 2 } else { 1 };
        process::exit(code);
    }
}
