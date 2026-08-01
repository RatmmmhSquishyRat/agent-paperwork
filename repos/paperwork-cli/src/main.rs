//! paperwork-cli: Stateless, path-explicit CLI for Agent Paperwork.
//!
//! Thin CLI layer: parses args -> calls paperwork_core::ops -> formats output.
//! No workspace, no init, no login. Every command takes explicit file paths.

mod cmd;
mod output;

use std::process;

use clap::{Parser, Subcommand};

use crate::output::OutputMode;

/// Agent Paperwork - stateless file-based collaboration toolkit for AI agents.
#[derive(Parser)]
#[command(name = "paperwork", version, about, long_about = None)]
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
    let cli = Cli::parse();

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

    let result = match cli.command {
        Commands::Profile(args) => cmd::profile::run(&ctx, args),
        Commands::Post(args) => cmd::post::run(&ctx, args),
        Commands::Brief(args) => cmd::brief::run(&ctx, args),
        Commands::Contacts(args) => cmd::contacts::run(&ctx, args),
        Commands::Validate(args) => cmd::validate::run(&ctx, args),
    };

    if let Err(e) = result {
        // Try to downcast to PaperworkError for structured output
        if let Some(pw_err) = e.downcast_ref::<paperwork_core::PaperworkError>() {
            output::emit_err(
                &ctx,
                pw_err.category(),
                &pw_err.to_string(),
                &pw_err.fix(),
                &pw_err.example(),
            );
        } else {
            // Generic anyhow error
            output::emit_err(&ctx, "io", &e.to_string(), "", "");
        }
        process::exit(1);
    }
}
