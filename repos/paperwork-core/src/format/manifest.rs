//! Manifest parsing and serialization.
//!
//! Format spec (§2.7):
//! ```markdown
//! # Manifest: <name>
//!
//! **Author**: <agent>
//! **Created**: <ISO-8601>
//! **Description**: <what this manifest helps you understand>
//!
//! ## Entries
//!
//! ### <entry-title>
//!
//! **Path**: `<relative-path-or-glob>`
//! **Hash**: `<sha256-hex>`
//! **Regex**: `<pattern>` | —
//! **Groups**: <group1>, <group2> | —
//!
//! > Optional note about why this entry matters.
//!
//! ---
//! ```
//!
//! Regex is stored in fenced code blocks (```regex ... ```) to handle special characters.
//! Groups are derived automatically from regex named captures at parse time.

use chrono::{DateTime, Utc};
use regex::Regex;

use crate::{Manifest, ManifestEntry, PaperworkError, Result};

use super::{extract_bold_key, normalize_line_endings};

/// Extract named capture group names from a regex pattern.
pub fn extract_regex_groups(pattern: &str) -> Vec<String> {
    let re = Regex::new(r"\(\?<([^>]+)>").expect("valid regex");
    re.captures_iter(pattern)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Parse a manifest from Markdown content.
pub fn parse_manifest(content: &str) -> Result<Manifest> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();

    let mut name: Option<String> = None;
    let mut author: Option<String> = None;
    let mut created: Option<DateTime<Utc>> = None;
    let mut description: Option<String> = None;
    let mut entries: Vec<ManifestEntry> = Vec::new();

    let mut in_entries = false;
    let mut current_entry: Option<EntryBuilder> = None;
    let mut in_regex_block = false;
    let mut regex_lines: Vec<String> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();

        // Handle regex fenced code block
        if in_regex_block {
            if trimmed == "```" {
                // End of regex block
                in_regex_block = false;
                if let Some(ref mut entry) = current_entry {
                    entry.regex = Some(regex_lines.join("\n"));
                    entry.groups = extract_regex_groups(&regex_lines.join("\n"));
                }
                regex_lines.clear();
            } else {
                regex_lines.push(line.to_string());
            }
            continue;
        }

        // Start of regex fenced code block
        if trimmed == "```regex" {
            in_regex_block = true;
            regex_lines.clear();
            continue;
        }

        // H1: Manifest name
        if let Some(stripped) = trimmed.strip_prefix("# Manifest: ") {
            name = Some(stripped.to_string());
            continue;
        }

        // H2: Entries section
        if trimmed == "## Entries" {
            in_entries = true;
            continue;
        }

        // H3: Entry title (only in Entries section)
        if in_entries && trimmed.starts_with("### ") && !trimmed.starts_with("#### ") {
            // Save previous entry if exists
            if let Some(builder) = current_entry.take() {
                entries.push(builder.build());
            }
            current_entry = Some(EntryBuilder::new(trimmed[4..].to_string()));
            continue;
        }

        // Horizontal rule (entry separator)
        if trimmed == "---" {
            if let Some(builder) = current_entry.take() {
                entries.push(builder.build());
            }
            continue;
        }

        // Bold key extraction
        if let Some((key, value)) = extract_bold_key(trimmed) {
            match key.as_str() {
                "Author" => author = Some(value),
                "Created" => {
                    created = parse_timestamp(&value).ok();
                }
                "Description" => description = Some(value),
                "Path" => {
                    if let Some(ref mut entry) = current_entry {
                        entry.path = Some(strip_backticks(&value));
                    }
                }
                "Hash" => {
                    if let Some(ref mut entry) = current_entry {
                        entry.hash = Some(strip_backticks(&value));
                    }
                }
                "Regex" => {
                    // Inline regex (— means none, or backtick-quoted for simple patterns)
                    if let Some(ref mut entry) = current_entry {
                        if value == "—" || value.is_empty() {
                            entry.regex = None;
                            entry.groups = Vec::new();
                        } else if !value.starts_with("```") {
                            // Simple inline regex (backtick-quoted)
                            let pattern = strip_backticks(&value);
                            entry.groups = extract_regex_groups(&pattern);
                            entry.regex = Some(pattern);
                        }
                        // If value indicates fenced block follows, it's handled above
                    }
                }
                "Groups" => {
                    // Groups are derived from regex, but we parse for validation
                    // This field is informational; actual groups come from regex
                }
                _ => {}
            }
            continue;
        }

        // Blockquote (note)
        if trimmed.starts_with("> ") || trimmed == ">" {
            if let Some(ref mut entry) = current_entry {
                let note_text = trimmed.strip_prefix("> ").unwrap_or("").to_string();
                match &mut entry.note {
                    Some(existing) => {
                        existing.push('\n');
                        existing.push_str(&note_text);
                    }
                    None => entry.note = Some(note_text),
                }
            }
        }
    }

    // Don't forget the last entry
    if let Some(builder) = current_entry.take() {
        entries.push(builder.build());
    }

    let name = name.ok_or_else(|| {
        PaperworkError::Parse("missing manifest name heading (# Manifest: <name>)".to_string())
    })?;

    let author = author.ok_or_else(|| {
        PaperworkError::Parse(format!("missing **Author**: line for manifest '{}'", name))
    })?;

    let created = created.ok_or_else(|| {
        PaperworkError::Parse(format!("missing or invalid **Created**: line for manifest '{}'", name))
    })?;

    Ok(Manifest {
        name,
        author,
        created,
        description: description.unwrap_or_default(),
        entries,
    })
}

/// Parse timestamp from ISO-8601 string.
fn parse_timestamp(s: &str) -> std::result::Result<DateTime<Utc>, String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
    }
    Err(format!("cannot parse '{}' as ISO-8601 timestamp", s))
}

/// Strip surrounding backticks from a value.
fn strip_backticks(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Builder for constructing ManifestEntry during parsing.
struct EntryBuilder {
    title: String,
    path: Option<String>,
    hash: Option<String>,
    regex: Option<String>,
    groups: Vec<String>,
    note: Option<String>,
}

impl EntryBuilder {
    fn new(title: String) -> Self {
        Self {
            title,
            path: None,
            hash: None,
            regex: None,
            groups: Vec::new(),
            note: None,
        }
    }

    fn build(self) -> ManifestEntry {
        ManifestEntry {
            title: self.title,
            path: self.path.unwrap_or_default(),
            hash: self.hash.unwrap_or_default(),
            regex: self.regex,
            groups: self.groups,
            note: self.note,
        }
    }
}

/// Serialize a manifest to Markdown content.
pub fn serialize_manifest(manifest: &Manifest) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Manifest: {}\n\n", manifest.name));
    out.push_str(&format!("**Author**: {}  \n", manifest.author));
    out.push_str(&format!(
        "**Created**: {}  \n",
        manifest.created.format("%Y-%m-%dT%H:%M:%SZ")
    ));
    out.push_str(&format!("**Description**: {}\n\n", manifest.description));
    out.push_str("## Entries\n");

    for entry in &manifest.entries {
        out.push_str(&serialize_entry(entry));
    }

    out
}

/// Serialize a single manifest entry.
fn serialize_entry(entry: &ManifestEntry) -> String {
    let mut out = String::new();

    out.push_str(&format!("\n### {}\n\n", entry.title));
    out.push_str(&format!("**Path**: `{}`  \n", entry.path));
    out.push_str(&format!("**Hash**: `{}`  \n", entry.hash));

    // Regex: use fenced code block for complex patterns, — for none
    match &entry.regex {
        Some(pattern) => {
            if pattern.contains('\n') || pattern.contains('`') {
                // Complex pattern: use fenced code block
                out.push_str("**Regex**:  \n");
                out.push_str("```regex\n");
                out.push_str(pattern);
                out.push_str("\n```\n");
            } else {
                // Simple pattern: inline backticks
                out.push_str(&format!("**Regex**: `{}`  \n", pattern));
            }
        }
        None => {
            out.push_str("**Regex**: —  \n");
        }
    }

    // Groups
    if entry.groups.is_empty() {
        out.push_str("**Groups**: —\n");
    } else {
        out.push_str(&format!("**Groups**: {}\n", entry.groups.join(", ")));
    }

    // Note
    if let Some(ref note) = entry.note {
        out.push('\n');
        for line in note.lines() {
            out.push_str(&format!("> {}\n", line));
        }
    }

    out.push_str("\n---\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_timestamp(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, s).unwrap()
    }

    #[test]
    fn test_parse_manifest_entry_full() {
        let content = r#"# Manifest: onboarding

**Author**: alice  
**Created**: 2026-01-15T10:00:00Z  
**Description**: Understanding the codebase structure

## Entries

### Main Entry Point

**Path**: `src/main.rs`  
**Hash**: `abc123def456`  
**Regex**: `fn main\(\)`  
**Groups**: —

> This is where the application starts.

---
"#;
        let manifest = parse_manifest(content).expect("should parse");
        assert_eq!(manifest.name, "onboarding");
        assert_eq!(manifest.author, "alice");
        assert_eq!(manifest.description, "Understanding the codebase structure");
        assert_eq!(manifest.entries.len(), 1);

        let entry = &manifest.entries[0];
        assert_eq!(entry.title, "Main Entry Point");
        assert_eq!(entry.path, "src/main.rs");
        assert_eq!(entry.hash, "abc123def456");
        assert_eq!(entry.regex, Some("fn main\\(\\)".to_string()));
        assert_eq!(entry.note, Some("This is where the application starts.".to_string()));
    }

    #[test]
    fn test_parse_manifest_no_regex() {
        let content = r#"# Manifest: simple

**Author**: bob  
**Created**: 2026-01-15T10:00:00Z  
**Description**: Simple manifest

## Entries

### Config File

**Path**: `config.toml`  
**Hash**: `deadbeef`  
**Regex**: —  
**Groups**: —

---
"#;
        let manifest = parse_manifest(content).expect("should parse");
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.entries[0].regex, None);
        assert!(manifest.entries[0].groups.is_empty());
    }

    #[test]
    fn test_parse_manifest_multi_entry() {
        let content = r#"# Manifest: multi

**Author**: alice  
**Created**: 2026-01-15T10:00:00Z  
**Description**: Multiple entries

## Entries

### Entry One

**Path**: `src/one.rs`  
**Hash**: `hash1`  
**Regex**: —  
**Groups**: —

---

### Entry Two

**Path**: `src/two.rs`  
**Hash**: `hash2`  
**Regex**: —  
**Groups**: —

---

### Entry Three

**Path**: `src/three.rs`  
**Hash**: `hash3`  
**Regex**: —  
**Groups**: —

---
"#;
        let manifest = parse_manifest(content).expect("should parse");
        assert_eq!(manifest.entries.len(), 3);
        assert_eq!(manifest.entries[0].title, "Entry One");
        assert_eq!(manifest.entries[1].title, "Entry Two");
        assert_eq!(manifest.entries[2].title, "Entry Three");
    }

    #[test]
    fn test_parse_manifest_regex_with_groups() {
        let content = r#"# Manifest: grouped

**Author**: alice  
**Created**: 2026-01-15T10:00:00Z  
**Description**: Regex with named groups

## Entries

### Function Parser

**Path**: `src/**/*.rs`  
**Hash**: `abc123`  
**Regex**: `fn (?<name>\w+)\((?<args>[^)]*)\)`  
**Groups**: name, args

---
"#;
        let manifest = parse_manifest(content).expect("should parse");
        let entry = &manifest.entries[0];
        assert_eq!(
            entry.regex,
            Some("fn (?<name>\\w+)\\((?<args>[^)]*)\\)".to_string())
        );
        assert_eq!(entry.groups, vec!["name", "args"]);
    }

    #[test]
    fn test_parse_manifest_fenced_regex() {
        let content = r#"# Manifest: fenced

**Author**: alice  
**Created**: 2026-01-15T10:00:00Z  
**Description**: Fenced regex block

## Entries

### Complex Pattern

**Path**: `data.txt`  
**Hash**: `xyz789`  
**Regex**:  
```regex
(?<year>\d{4})-(?<month>\d{2})
```
**Groups**: year, month

---
"#;
        let manifest = parse_manifest(content).expect("should parse");
        let entry = &manifest.entries[0];
        assert_eq!(entry.regex, Some("(?<year>\\d{4})-(?<month>\\d{2})".to_string()));
        assert_eq!(entry.groups, vec!["year", "month"]);
    }

    #[test]
    fn test_serialize_manifest_roundtrip() {
        let manifest = Manifest {
            name: "roundtrip".to_string(),
            author: "tester".to_string(),
            created: make_timestamp(2026, 1, 15, 10, 0, 0),
            description: "Roundtrip test".to_string(),
            entries: vec![
                ManifestEntry {
                    title: "Entry A".to_string(),
                    path: "src/a.rs".to_string(),
                    hash: "hash_a".to_string(),
                    regex: Some("fn test".to_string()),
                    groups: vec![],
                    note: Some("Note A".to_string()),
                },
                ManifestEntry {
                    title: "Entry B".to_string(),
                    path: "src/b.rs".to_string(),
                    hash: "hash_b".to_string(),
                    regex: None,
                    groups: vec![],
                    note: None,
                },
            ],
        };

        let serialized = serialize_manifest(&manifest);
        let parsed = parse_manifest(&serialized).expect("should parse serialized");

        assert_eq!(manifest.name, parsed.name);
        assert_eq!(manifest.author, parsed.author);
        assert_eq!(manifest.description, parsed.description);
        assert_eq!(manifest.entries.len(), parsed.entries.len());

        for (orig, parsed) in manifest.entries.iter().zip(parsed.entries.iter()) {
            assert_eq!(orig.title, parsed.title);
            assert_eq!(orig.path, parsed.path);
            assert_eq!(orig.hash, parsed.hash);
            assert_eq!(orig.regex, parsed.regex);
            assert_eq!(orig.note, parsed.note);
        }
    }

    #[test]
    fn test_extract_regex_groups() {
        assert_eq!(
            extract_regex_groups(r"(?<name>\w+) (?<value>\d+)"),
            vec!["name", "value"]
        );
        assert_eq!(extract_regex_groups(r"\d+"), Vec::<String>::new());
        assert_eq!(
            extract_regex_groups(r"(?<year>\d{4})-(?<month>\d{2})-(?<day>\d{2})"),
            vec!["year", "month", "day"]
        );
    }

    #[test]
    fn test_parse_manifest_missing_name() {
        let content = r#"**Author**: alice
**Created**: 2026-01-15T10:00:00Z
"#;
        let result = parse_manifest(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing manifest name"));
    }

    #[test]
    fn test_parse_manifest_crlf() {
        let content = "# Manifest: test\r\n\r\n**Author**: alice  \r\n**Created**: 2026-01-15T10:00:00Z  \r\n**Description**: CRLF test\r\n\r\n## Entries\r\n";
        let manifest = parse_manifest(content).expect("should parse CRLF");
        assert_eq!(manifest.name, "test");
        assert_eq!(manifest.author, "alice");
    }

    #[test]
    fn test_parse_manifest_multiline_note() {
        let content = r#"# Manifest: noted

**Author**: alice  
**Created**: 2026-01-15T10:00:00Z  
**Description**: Multi-line note

## Entries

### Entry

**Path**: `file.rs`  
**Hash**: `abc`  
**Regex**: —  
**Groups**: —

> First line of note.
> Second line of note.

---
"#;
        let manifest = parse_manifest(content).expect("should parse");
        let note = manifest.entries[0].note.as_ref().expect("should have note");
        assert!(note.contains("First line"));
        assert!(note.contains("Second line"));
    }
}
