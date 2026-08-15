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

/// Chain-style JSON object builder over `serde_json::Map`.
///
/// Every command-side `--json` payload is assembled through this builder so
/// the frozen output contract (char_tests byte snapshots) has exactly one
/// construction path. It deliberately wraps `serde_json::Map` instead of a
/// derive struct: the map's key-order semantics stay exactly as they are
/// today, while a derive struct's field declaration order could diverge
/// from the frozen key order (P-6).
///
/// Key-order fact (Ultra Review F4, backported from wip 8539a08):
/// `serde_json` is built WITHOUT the `preserve_order` feature, so
/// `serde_json::Map` is backed by a `BTreeMap` — serialized keys come out
/// in ALPHABETICAL order regardless of insertion order. That is
/// byte-identical to the pre-builder output and pinned by the char_tests
/// snapshots (and by the unit test below).
pub struct JsonBuilder {
    map: serde_json::Map<String, serde_json::Value>,
}

impl JsonBuilder {
    pub fn new() -> Self {
        Self {
            map: serde_json::Map::new(),
        }
    }

    /// Insert a key unconditionally.
    pub fn insert(mut self, key: &str, value: serde_json::Value) -> Self {
        self.map.insert(key.to_string(), value);
        self
    }

    /// Insert only when present — an absent value keeps the key out of the
    /// object entirely (the frozen "missing key, not null" skip semantics).
    pub fn insert_opt(self, key: &str, value: Option<serde_json::Value>) -> Self {
        match value {
            Some(value) => self.insert(key, value),
            None => self,
        }
    }

    pub fn build(self) -> serde_json::Value {
        serde_json::Value::Object(self.map)
    }
}

impl Default for JsonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Emit a JSON value as one compact line on stdout — the single print path
/// for all command-side `--json` output (P-6).
pub fn print_json(value: serde_json::Value) {
    println!("{}", serde_json::to_string(&value).unwrap_or_default());
}

/// Print raw/plain text output (for --plain mode).
pub fn print_plain(text: &str) {
    print!("{}", text);
    if !text.ends_with('\n') {
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::JsonBuilder;

    /// Ultra Review F4 (backported from wip 8539a08): pins the key-order
    /// fact documented on [`JsonBuilder`] — insertion order must NOT leak
    /// into the serialized bytes; alphabetical (BTreeMap) order must.
    #[test]
    fn json_builder_keys_serialize_alphabetically_not_by_insertion() {
        let value = JsonBuilder::new()
            .insert("zebra", serde_json::json!(1))
            .insert("status", serde_json::json!("ok"))
            .insert("alpha", serde_json::json!(3))
            .build();
        assert_eq!(
            serde_json::to_string(&value).expect("serialize"),
            r#"{"alpha":3,"status":"ok","zebra":1}"#
        );
    }
}
