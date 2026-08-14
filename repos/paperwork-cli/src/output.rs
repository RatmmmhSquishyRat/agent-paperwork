//! Output formatting: unified envelope grammar for agent-friendly CLI output.
//!
//! Success envelope (stdout):
//! ```text
//! ok <command> <conclusion>
//! <key>: <value>
//! ---
//! <body lines>
//! ```
//!
//! Error envelope (stderr):
//! ```text
//! error <category>: <message>
//! fix: <corrective action>
//! example: <corrected command>
//! ```

use crate::cmd::Context;

/// Output mode selected by global flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Structured envelope (default)
    Default,
    /// Structured JSON output
    Json,
    /// Raw file content
    Plain,
}

/// A success envelope to be emitted.
pub struct Envelope {
    pub command: &'static str,
    pub conclusion: String,
    pub fields: Vec<(String, String)>,
    pub body: Vec<String>,
}

impl Envelope {
    pub fn new(command: &'static str, conclusion: String) -> Self {
        Self {
            command,
            conclusion,
            fields: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn field(mut self, key: &str, value: &str) -> Self {
        self.fields.push((key.to_string(), value.to_string()));
        self
    }

    pub fn body_lines(mut self, lines: Vec<String>) -> Self {
        self.body.extend(lines);
        self
    }
}

/// Emit a success envelope according to the output mode.
pub fn emit_ok(ctx: &Context, env: Envelope) {
    match ctx.mode {
        OutputMode::Json => {
            let mut obj = serde_json::Map::new();
            obj.insert("status".to_string(), serde_json::json!("ok"));
            obj.insert("command".to_string(), serde_json::json!(env.command));
            obj.insert("conclusion".to_string(), serde_json::json!(env.conclusion));
            for (k, v) in &env.fields {
                obj.insert(k.clone(), serde_json::json!(v));
            }
            if !env.body.is_empty() {
                obj.insert("body".to_string(), serde_json::json!(env.body));
            }
            println!(
                "{}",
                serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default()
            );
        }
        OutputMode::Plain => {
            // Plain mode: raw content only (handled by caller)
        }
        OutputMode::Default => {
            if !ctx.quiet {
                println!("ok {} {}", env.command, env.conclusion);
            }
            for (k, v) in &env.fields {
                println!("{}: {}", k, v);
            }
            if !env.body.is_empty() {
                println!("---");
                for line in &env.body {
                    println!("{}", line);
                }
            }
        }
    }
}

/// Emit an error envelope.
/// In default mode: stderr. In JSON mode: stdout with exit_code field.
/// The `command` field identifies the failing command (e.g. `post.send`).
pub fn emit_err(
    ctx: &Context,
    command: &str,
    category: &str,
    message: &str,
    fix: &str,
    example: &str,
) {
    match ctx.mode {
        OutputMode::Json => {
            let mut obj = serde_json::Map::new();
            obj.insert("status".to_string(), serde_json::json!("error"));
            obj.insert("category".to_string(), serde_json::json!(category));
            obj.insert("message".to_string(), serde_json::json!(message));
            if !fix.is_empty() {
                obj.insert("fix".to_string(), serde_json::json!(fix));
            }
            if !example.is_empty() {
                obj.insert("example".to_string(), serde_json::json!(example));
            }
            obj.insert("command".to_string(), serde_json::json!(command));
            obj.insert("exit_code".to_string(), serde_json::json!(1));
            println!(
                "{}",
                serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default()
            );
        }
        _ => {
            eprintln!("error {}: {}", category, message);
            if !fix.is_empty() {
                eprintln!("fix: {}", fix);
            }
            if !example.is_empty() {
                eprintln!("example: {}", example);
            }
        }
    }
}

/// Emit a usage error envelope (seventh category, exit 2).
///
/// Used before a parse result exists, so `--json` detection is done by the
/// caller (argv scan). In JSON mode the error object goes to stdout and
/// `exit_code` truthfully reflects the process exit code (2).
pub fn emit_usage_error(json_mode: bool, command: &str, message: &str, fix: &str, example: &str) {
    if json_mode {
        let mut obj = serde_json::Map::new();
        obj.insert("status".to_string(), serde_json::json!("error"));
        obj.insert("category".to_string(), serde_json::json!("usage"));
        obj.insert("message".to_string(), serde_json::json!(message));
        if !fix.is_empty() {
            obj.insert("fix".to_string(), serde_json::json!(fix));
        }
        if !example.is_empty() {
            obj.insert("example".to_string(), serde_json::json!(example));
        }
        obj.insert("command".to_string(), serde_json::json!(command));
        obj.insert("exit_code".to_string(), serde_json::json!(2));
        println!(
            "{}",
            serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default()
        );
    } else {
        eprintln!("error usage: {}", message);
        if !fix.is_empty() {
            eprintln!("fix: {}", fix);
        }
        if !example.is_empty() {
            eprintln!("example: {}", example);
        }
    }
}

/// Print raw/plain text output (for --plain mode).
pub fn print_plain(text: &str) {
    print!("{}", text);
    if !text.ends_with('\n') {
        println!();
    }
}
