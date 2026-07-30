//! Output formatting: Default (Markdown), JSON, Plain modes.

use serde::Serialize;

/// Output mode selected by global flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Rich Markdown rendering (default)
    Default,
    /// Structured JSON output
    Json,
    /// Raw file content
    Plain,
}

/// Print a success message (suppressed in quiet mode).
pub fn success(ctx: &crate::cmd::Context, msg: &str) {
    if ctx.quiet {
        return;
    }
    match ctx.mode {
        OutputMode::Json => {} // JSON success handled by command output
        _ => println!("\u{2713} {}", msg),
    }
}

/// Print JSON output for a serializable value.
pub fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("\u{2717} failed to serialize output: {}", e),
    }
}

/// Print raw/plain text output.
pub fn print_plain(text: &str) {
    println!("{}", text);
}

/// Print default (Markdown) output.
pub fn print_default(text: &str) {
    println!("{}", text);
}

/// Print an error in the appropriate format.
pub fn print_error(ctx: &crate::cmd::Context, err: &anyhow::Error) {
    match ctx.mode {
        OutputMode::Json => {
            let msg = err.to_string();
            let (error_line, hint) = parse_error_hint(&msg);
            let json = serde_json::json!({
                "error": error_line,
                "hint": hint,
                "exit_code": 1
            });
            println!("{}", serde_json::to_string(&json).unwrap_or_default());
        }
        _ => {
            let msg = err.to_string();
            let lines: Vec<&str> = msg.lines().collect();
            if let Some(first) = lines.first() {
                eprintln!("\u{2717} {}", first.trim());
                for line in &lines[1..] {
                    let trimmed = line.trim();
                    if trimmed.starts_with('\u{2192}') || trimmed.starts_with("→") {
                        eprintln!("  {}", trimmed);
                    } else if !trimmed.is_empty() {
                        eprintln!("  \u{2192} {}", trimmed);
                    }
                }
            }
        }
    }
}

/// Parse error message into (error, hint) parts.
fn parse_error_hint(msg: &str) -> (String, String) {
    let lines: Vec<&str> = msg.lines().collect();
    let error_line = lines.first().unwrap_or(&"unknown error").to_string();
    let hint = lines
        .iter()
        .skip(1)
        .find_map(|l| {
            let trimmed = l.trim();
            trimmed
                .strip_prefix('\u{2192}')
                .or_else(|| trimmed.strip_prefix("→"))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_default();
    (error_line, hint)
}
