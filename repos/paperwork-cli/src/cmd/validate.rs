//! Validate command: check Markdown structure of a file.

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::cmd::Context;
use crate::output::{self, OutputMode};

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the file to validate
    pub path: PathBuf,
}

pub fn run(ctx: &Context, args: ValidateArgs) -> Result<()> {
    let content = std::fs::read_to_string(&args.path)
        .map_err(|e| anyhow::anyhow!("Cannot read '{}': {}", args.path.display(), e))?;

    let issues = paperwork_core::format::validate_markdown(&content);

    match ctx.mode {
        OutputMode::Json => {
            let result = serde_json::json!({
                "path": args.path.display().to_string(),
                "valid": issues.is_empty(),
                "issues": issues,
            });
            output::print_json(&result);
        }
        _ => {
            if issues.is_empty() {
                output::success(ctx, &format!("Valid: {}", args.path.display()));
            } else {
                for issue in &issues {
                    output::print_default(&format!("\u{2717} {}", issue));
                }
            }
        }
    }

    if !issues.is_empty() {
        anyhow::bail!("{} issue(s) found", issues.len());
    }

    Ok(())
}
