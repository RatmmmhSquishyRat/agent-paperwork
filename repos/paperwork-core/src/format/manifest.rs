//! Manifest (brief) parsing and serialization — Managed File Format v2 (spec §6).
//!
//! - preamble: H1 (title) + optional description prose + `- owner:` / `- created:`;
//! - entries: fence-aware H2 sections directly after the preamble (no
//!   `## Entries` wrapper); entry attribute zone extends to the first
//!   non-attribute non-blank line (blank lines do NOT terminate it);
//! - `- regex:` inline for simple patterns, ```regex fence for complex ones;
//! - note = bare prose (no blockquote);
//! - groups derived from named captures, never persisted.

use chrono::{DateTime, Utc};
use regex::Regex;
use std::sync::LazyLock;

use crate::{Manifest, ManifestEntry, PaperworkError, Result};

use super::{
    collect_outside_fence, compute_fence_length, extract_attribute, fence_close_matches,
    fence_info, fence_open_len, first_outside_fence, normalize_line_endings, parse_timestamp,
    RFC3339_FMT,
};

/// Named-capture-group scanner (`(?<name>...)`), compiled once (M-review M5).
static CAPTURE_GROUP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(\?<([^>]+)>").expect("valid regex"));

/// Extract named capture group names from a regex pattern.
pub fn extract_regex_groups(pattern: &str) -> Vec<String> {
    CAPTURE_GROUP_RE
        .captures_iter(pattern)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Representability check for a brief entry note (M-review M1).
///
/// A note whose FIRST non-blank line is attribute-shaped (`- key: value`,
/// matching the attribute regex) or opens a ```` ```regex ```` fence cannot
/// survive a parse → serialize roundtrip: the attribute-zone rule (blank
/// lines do not terminate it) would re-absorb that line as an attribute or
/// as a regex carrier. Returns a human-readable reason if unrepresentable.
pub fn note_representation_issue(note: &str) -> Option<&'static str> {
    let first = note.lines().find(|l| !l.trim().is_empty())?;
    if extract_attribute(first).is_some() {
        return Some("note starts with an attribute-shaped line '- key: value'");
    }
    if fence_info(first) == "regex" && fence_open_len(first).is_some() {
        return Some("note starts with a ```regex fence opening line");
    }
    None
}

/// Locate fence-aware H2 line indices (entry boundaries, spec §3.3/§6.2).
///
/// Delegates to the shared scanner family ([`collect_outside_fence`]); the
/// `&[&str]` signature is retained so the `lines()`-based call sites stay
/// unchanged (the join is fence-neutral: callers pass already-normalized
/// lines).
fn h2_indices(lines: &[&str]) -> Vec<usize> {
    let joined = lines.join("\n");
    collect_outside_fence(&joined, |_i, line| line.trim().starts_with("## "))
}

/// Fence-aware scan for legacy (v0.4) brief residue (SAM-1 guard).
///
/// A brief that already uses lowercase attribute keys but still carries the
/// v0.4 `## Entries` wrapper heading or `### ` H3 entry headers would parse
/// under v0.5 rules and be destroyed by the next read-modify-rewrite (the
/// wrapper/H3 lines become entry titles or preamble prose). Only structural
/// positions trigger: fence-outside lines only, so a `### ` example inside
/// a note fence is quoted content and stays legal.
fn contains_legacy_brief_residue(lines: &[&str]) -> bool {
    let joined = lines.join("\n");
    first_outside_fence(&joined, |_i, line| {
        let trimmed = line.trim();
        trimmed == "## Entries" || trimmed.starts_with("### ")
    })
    .is_some()
}

/// Parse a manifest (brief) from Markdown content (spec §6.2).
pub fn parse_manifest(content: &str) -> Result<Manifest> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();

    // Legacy residue guard (SAM-1): a half-migrated v0.4 brief (lowercase
    // keys but residual `## Entries` wrapper / `### ` entry headers) would
    // parse silently and be corrupted by the next RMW — refuse at parse.
    if contains_legacy_brief_residue(&lines) {
        return Err(PaperworkError::Parse {
            message: "brief contains legacy v0.4 residue ('## Entries' wrapper heading or '### ' entry headers)".to_string(),
            fix: "migrate this brief to the v0.5 entry layout per the CHANGELOG migration guide: entries are '## <title>' sections directly after the preamble, without an '## Entries' wrapper".to_string(),
            example: "# title\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## entry title\n\n- path: file.rs\n- hash: <sha256>".to_string(),
        });
    }

    let headers = h2_indices(&lines);
    let preamble_end = headers.first().copied().unwrap_or(lines.len());

    // ---- preamble ----
    let mut name: Option<String> = None;
    let mut author: Option<String> = None;
    let mut owner_present = false;
    let mut created: Option<DateTime<Utc>> = None;
    let mut desc_lines: Vec<String> = Vec::new();

    for line in &lines[..preamble_end] {
        let trimmed = line.trim();
        if name.is_none() {
            if let Some(stripped) = line.strip_prefix("# ") {
                name = Some(stripped.trim().to_string());
                continue;
            }
        }
        if let Some((key, value)) = extract_attribute(line) {
            match key.as_str() {
                "owner" => {
                    owner_present = true;
                    if author.is_none() {
                        author = Some(value);
                    }
                }
                "created" => created = parse_timestamp(&value).ok(),
                _ => {}
            }
            continue;
        }
        if name.is_some() && !trimmed.is_empty() {
            desc_lines.push(trimmed.to_string());
        }
    }

    let name = name.ok_or_else(|| PaperworkError::Parse {
        message: "missing title heading (# <title>)".to_string(),
        fix: "add a top-level heading with the brief title".to_string(),
        example: "# onboarding".to_string(),
    })?;

    let author = if owner_present {
        author.unwrap_or_default()
    } else {
        return Err(PaperworkError::Parse {
            message: format!("missing - owner: line for brief '{}'", name),
            fix: "add a '- owner: <agent>' bullet line".to_string(),
            example: "- owner: alice".to_string(),
        });
    };

    let created = created.ok_or_else(|| PaperworkError::Parse {
        message: format!("missing or invalid - created: line for brief '{}'", name),
        fix: "add a '- created: <RFC3339>' bullet line".to_string(),
        example: "- created: 2026-01-15T10:00:00Z".to_string(),
    })?;

    // ---- entries ----
    let mut entries: Vec<ManifestEntry> = Vec::new();
    for (idx, &header_line) in headers.iter().enumerate() {
        let title = lines[header_line].trim()[3..].trim().to_string();
        let body_end = if idx + 1 < headers.len() {
            headers[idx + 1]
        } else {
            lines.len()
        };
        entries.push(parse_entry_body(title, &lines[header_line + 1..body_end]));
    }

    Ok(Manifest {
        name,
        author,
        created,
        description: desc_lines.join("\n"),
        entries,
    })
}

/// Parse one entry body: attribute zone (up to the first non-attribute
/// non-blank line; blank lines do NOT terminate it — BDD:BRIEF-12), an
/// optional ```regex fence before the note starts, and bare-prose note.
fn parse_entry_body(title: String, lines: &[&str]) -> ManifestEntry {
    let mut path: Option<String> = None;
    let mut hash: Option<String> = None;
    let mut regex: Option<String> = None;
    let mut groups: Vec<String> = Vec::new();
    let mut note_lines: Vec<&str> = Vec::new();

    let mut attr_zone = true;
    let mut open_len: Option<usize> = None;
    let mut collecting_regex = false;
    let mut regex_lines: Vec<&str> = Vec::new();

    for line in lines {
        // Inside a fence.
        if let Some(n) = open_len {
            if fence_close_matches(line, n) {
                open_len = None;
                if collecting_regex {
                    collecting_regex = false;
                    let pattern = regex_lines.join("\n");
                    groups = extract_regex_groups(&pattern);
                    regex = Some(pattern);
                    regex_lines.clear();
                }
            } else if collecting_regex {
                regex_lines.push(line);
            } else {
                note_lines.push(line);
            }
            continue;
        }

        if attr_zone {
            if line.trim().is_empty() {
                // Blank lines do not terminate the attribute zone.
                continue;
            }
            if let Some((key, value)) = extract_attribute(line) {
                match key.as_str() {
                    "path" => {
                        if path.is_none() {
                            path = Some(value);
                        }
                    }
                    "hash" => {
                        if hash.is_none() {
                            hash = Some(value);
                        }
                    }
                    "regex" if regex.is_none() => {
                        let pattern = value;
                        groups = extract_regex_groups(&pattern);
                        regex = Some(pattern);
                    }
                    _ => {}
                }
                continue;
            }
            if let Some(n) = fence_open_len(line) {
                if fence_info(line) == "regex" && regex.is_none() {
                    // ```regex fence as an attribute-zone regex carrier.
                    open_len = Some(n);
                    collecting_regex = true;
                    continue;
                }
                // Any other fence line is the first non-attribute content:
                // the note starts (and the fence content belongs to it).
                attr_zone = false;
                open_len = Some(n);
                note_lines.push(line);
                continue;
            }
            // First non-blank non-attribute line: note starts (BDD:BRIEF-12).
            attr_zone = false;
            note_lines.push(line);
            continue;
        }

        // Note zone: verbatim lines (attribute-shaped lines belong to the
        // note and never override attributes); fences still tracked so a
        // fenced `## ` line cannot split the entry.
        note_lines.push(line);
        if let Some(n) = fence_open_len(line) {
            open_len = Some(n);
        }
    }

    // Trim trailing blank lines from the note.
    while note_lines.last().is_some_and(|l| l.trim().is_empty()) {
        note_lines.pop();
    }

    let note = if note_lines.is_empty() {
        None
    } else {
        Some(note_lines.join("\n"))
    };

    ManifestEntry {
        title,
        path: path.unwrap_or_default(),
        hash: hash.unwrap_or_default(),
        regex,
        groups,
        note,
    }
}

/// Serialize a manifest to Markdown content (spec §6.3).
pub fn serialize_manifest(manifest: &Manifest) -> String {
    let mut out = format!("# {}\n\n", manifest.name);

    if !manifest.description.is_empty() {
        out.push_str(&manifest.description);
        out.push_str("\n\n");
    }

    out.push_str(&format!("- owner: {}\n", manifest.author));
    out.push_str(&format!(
        "- created: {}\n",
        manifest.created.format(RFC3339_FMT)
    ));

    for entry in &manifest.entries {
        out.push_str(&serialize_entry(entry));
    }

    out
}

/// Serialize a single entry (spec §6.3).
fn serialize_entry(entry: &ManifestEntry) -> String {
    let mut out = format!("\n## {}\n\n", entry.title);
    out.push_str(&format!("- path: {}\n", entry.path));
    out.push_str(&format!("- hash: {}\n", entry.hash));

    if let Some(pattern) = &entry.regex {
        if pattern.contains('\n') || pattern.contains('`') {
            // Complex pattern: ```regex fence (dynamic length, spec §3.4).
            let fence = "`".repeat(compute_fence_length(pattern));
            out.push_str(&format!("{}regex\n{}\n{}\n", fence, pattern, fence));
        } else {
            out.push_str(&format!("- regex: {}\n", pattern));
        }
    }

    if let Some(note) = &entry.note {
        out.push('\n');
        out.push_str(note);
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_timestamp(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, s).unwrap()
    }

    const HASH64: &str = "42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540";

    // T-FB-01 (BRIEF-01)
    #[test]
    fn test_parse_entry_full() {
        let content = format!(
            "# Codebase Onboarding\n\nHow to understand this project\n\n- owner: alice\n- created: 2026-08-01T19:40:36Z\n\n## main.rs\n\n- path: src/main.rs\n- hash: {}\n- regex: fn main\n\nEntry point\n",
            HASH64
        );
        let manifest = parse_manifest(&content).expect("should parse");
        assert_eq!(manifest.name, "Codebase Onboarding");
        assert_eq!(manifest.author, "alice");
        assert_eq!(manifest.description, "How to understand this project");
        assert_eq!(manifest.entries.len(), 1);
        let entry = &manifest.entries[0];
        assert_eq!(entry.title, "main.rs");
        assert_eq!(entry.path, "src/main.rs"); // bare text, no backtick stripping
        assert_eq!(entry.hash, HASH64);
        assert_eq!(entry.regex, Some("fn main".to_string()));
        assert_eq!(entry.note, Some("Entry point".to_string()));
    }

    // T-FB-02 (BRIEF-02)
    #[test]
    fn test_no_regex_omitted() {
        let content = format!(
            "# b\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## cfg\n\n- path: config.toml\n- hash: {}\n",
            HASH64
        );
        let manifest = parse_manifest(&content).expect("should parse");
        assert_eq!(manifest.entries[0].regex, None);
        assert!(manifest.entries[0].groups.is_empty());

        let serialized = serialize_manifest(&manifest);
        assert!(!serialized.contains("- regex:"));
        assert!(!serialized.contains('—'));
    }

    // T-FB-03 (BRIEF-03)
    #[test]
    fn test_fenced_regex() {
        let content = format!(
            "# b\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## log\n\n- path: data.log\n- hash: {}\n```regex\n(?<year>\\d{{4}})-(?<month>\\d{{2}})\nwith `backtick` and\nmultiple lines\n```\n",
            HASH64
        );
        let manifest = parse_manifest(&content).expect("should parse");
        let entry = &manifest.entries[0];
        assert_eq!(
            entry.regex,
            Some(
                "(?<year>\\d{4})-(?<month>\\d{2})\nwith `backtick` and\nmultiple lines".to_string()
            )
        );
        assert_eq!(entry.groups, vec!["year", "month"]);

        // serialization of a complex regex uses a fence, not inline
        let serialized = serialize_manifest(&manifest);
        assert!(serialized.contains("```regex\n"));
        assert!(!serialized.contains("- regex:"));
        let reparsed = parse_manifest(&serialized).expect("reparse");
        assert_eq!(reparsed.entries[0].regex, entry.regex);
    }

    // T-FB-04 (BRIEF-04)
    #[test]
    fn test_hash_full_hex() {
        let content = format!(
            "# b\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## f\n\n- path: f.rs\n- hash: {}\n",
            HASH64
        );
        let manifest = parse_manifest(&content).expect("should parse");
        assert_eq!(manifest.entries[0].hash, HASH64);
        assert_eq!(manifest.entries[0].hash.len(), 64);
        let serialized = serialize_manifest(&manifest);
        assert!(serialized.contains(&format!("- hash: {}", HASH64)));
    }

    // T-FB-05 (BRIEF-05)
    #[test]
    fn test_groups_derived() {
        let content = format!(
            "# b\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## f\n\n- path: f.rs\n- hash: {}\n- regex: (?<year>\\d{{4}})-(?<month>\\d{{2}})\n",
            HASH64
        );
        let manifest = parse_manifest(&content).expect("should parse");
        assert_eq!(manifest.entries[0].groups, vec!["year", "month"]);

        let serialized = serialize_manifest(&manifest);
        assert!(!serialized.contains("groups"));
        assert!(!serialized.contains("year\", \"month"));
        let reparsed = parse_manifest(&serialized).expect("reparse");
        assert_eq!(reparsed.entries[0].groups, vec!["year", "month"]);
    }

    // T-FB-06 (BRIEF-06)
    #[test]
    fn test_missing_required() {
        // missing owner
        let content = "# b\n\n- created: 2026-01-15T10:00:00Z\n";
        let err = parse_manifest(content).unwrap_err();
        assert!(err.to_string().contains("missing - owner:"));
        assert_eq!(err.fix(), "add a '- owner: <agent>' bullet line");
        assert_eq!(err.example(), "- owner: alice");

        // missing created
        let content = "# b\n\n- owner: alice\n";
        let err = parse_manifest(content).unwrap_err();
        assert!(err.to_string().contains("missing or invalid - created:"));
        assert_eq!(err.fix(), "add a '- created: <RFC3339>' bullet line");

        // invalid created value
        let content = "# b\n\n- owner: alice\n- created: not-a-date\n";
        let err = parse_manifest(content).unwrap_err();
        assert!(err.to_string().contains("missing or invalid - created:"));

        // missing H1
        let content = "- owner: alice\n- created: 2026-01-15T10:00:00Z\n";
        let err = parse_manifest(content).unwrap_err();
        assert!(err.to_string().contains("missing title heading"));
    }

    // T-FB-07 (BRIEF-07)
    #[test]
    fn test_prose_note() {
        let content = format!(
            "# b\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## f\n\n- path: file.rs\n- hash: {}\n\nFirst line of note.\nSecond line of note.\n",
            HASH64
        );
        let manifest = parse_manifest(&content).expect("should parse");
        assert_eq!(
            manifest.entries[0].note,
            Some("First line of note.\nSecond line of note.".to_string())
        );
        let serialized = serialize_manifest(&manifest);
        assert!(!serialized.contains("> ")); // no blockquote prefix
        assert!(serialized.contains("First line of note.\nSecond line of note."));
    }

    // T-FB-08 (BRIEF-10)
    #[test]
    fn test_parse_crlf_unicode() {
        let lf = format!(
            "# 入门指南 🚀\n\n- owner: alicé\n- created: 2026-01-15T10:00:00Z\n\n## 入口\n\n- path: src/main.rs\n- hash: {}\n\n中文 note\n",
            HASH64
        );
        let crlf = lf.replace('\n', "\r\n");
        let a = parse_manifest(&lf).expect("lf");
        let b = parse_manifest(&crlf).expect("crlf");
        assert_eq!(a, b);
        assert_eq!(a.name, "入门指南 🚀");
        assert_eq!(a.entries[0].title, "入口");
        assert_eq!(a.entries[0].note, Some("中文 note".to_string()));
    }

    // T-FB-09 (BRIEF-11)
    #[test]
    fn test_roundtrip() {
        let manifest = Manifest {
            name: "roundtrip".to_string(),
            author: "tester".to_string(),
            created: make_timestamp(2026, 1, 15, 10, 0, 0),
            description: "Roundtrip test".to_string(),
            entries: vec![
                ManifestEntry {
                    title: "Entry A".to_string(),
                    path: "src/a.rs".to_string(),
                    hash: HASH64.to_string(),
                    regex: Some("fn test".to_string()),
                    groups: vec![],
                    note: Some("Note A".to_string()),
                },
                ManifestEntry {
                    title: "Entry B".to_string(),
                    path: "src/b.rs".to_string(),
                    hash: HASH64.to_string(),
                    regex: None,
                    groups: vec![],
                    note: None,
                },
                ManifestEntry {
                    title: "Entry C".to_string(),
                    path: "src/c.rs".to_string(),
                    hash: HASH64.to_string(),
                    regex: Some("(?<name>\\w+)\nmulti `line`".to_string()),
                    groups: vec!["name".to_string()],
                    note: Some("Note C\n- path: not-an-attribute".to_string()),
                },
            ],
        };

        let serialized = serialize_manifest(&manifest);
        assert!(!serialized.contains("## Entries"));
        assert!(!serialized.contains('—'));
        assert!(!serialized.contains("- Owner:"));
        assert!(!serialized.contains("- Path:"));
        let parsed = parse_manifest(&serialized).expect("roundtrip");
        assert_eq!(manifest, parsed);

        // no-description variant
        let no_desc = Manifest {
            description: String::new(),
            ..manifest.clone()
        };
        assert_eq!(
            parse_manifest(&serialize_manifest(&no_desc)).expect("roundtrip"),
            no_desc
        );
    }

    // T-FB-10 (BRIEF-05, retained)
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

    // T-FB-11 (BRIEF-12)
    #[test]
    fn test_entry_attribute_zone_boundary() {
        let content = "# b\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## f\n\n- path: a\n\n- hash: b\nNote starts\n- path: c\n";
        let manifest = parse_manifest(content).expect("should parse");
        let entry = &manifest.entries[0];
        // blank line did not terminate the attribute zone
        assert_eq!(entry.path, "a");
        assert_eq!(entry.hash, "b");
        // note contains the attribute-shaped line verbatim; it did not
        // override the path attribute
        assert_eq!(entry.note, Some("Note starts\n- path: c".to_string()));
        assert_eq!(entry.path, "a");
    }

    // fence-aware entry boundaries: an H2 inside a note fence is not a
    // new entry (spec §3.3, BDD:BRIEF-03 fence awareness)
    #[test]
    fn test_h2_inside_fence_not_entry() {
        let content = "# b\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## f\n\n- path: a\n- hash: b\n\nexample:\n\n```\n## fake entry\n```\n";
        let manifest = parse_manifest(content).expect("should parse");
        assert_eq!(manifest.entries.len(), 1);
        assert!(manifest.entries[0]
            .note
            .as_ref()
            .unwrap()
            .contains("## fake entry"));
    }
}
