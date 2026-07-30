//! Format layer: parsing and serialization of managed Markdown files.
//!
//! Shared utilities for boundary detection, bold-key extraction, and CRLF normalization.

pub mod contacts;
pub mod manifest;
pub mod notification;
pub mod profile;
pub mod thread;

use regex::Regex;
use std::sync::LazyLock;

/// Normalize CRLF → LF (invariant I11).
/// All parsers must call this before processing.
pub fn normalize_line_endings(content: &str) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
}

/// Regex for extracting bold-key metadata lines: `**Key**: value`
static BOLD_KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\*\*([^*]+)\*\*:\s*(.*)$").expect("valid regex"));

/// Regex for message H3 header: `### #<seq> — <sender> · <timestamp>`
static MESSAGE_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^### #(\d+) — (.+) · (.+)$").expect("valid regex"));

/// Extract a bold-key value from a line.
/// Returns (key, value) if the line matches `**Key**: value`.
pub fn extract_bold_key(line: &str) -> Option<(String, String)> {
    BOLD_KEY_RE.captures(line.trim()).map(|caps| {
        (
            caps[1].to_string(),
            caps[2].trim().to_string(),
        )
    })
}

/// Check if a line is a valid message H3 header.
/// Returns (seq, sender, timestamp_str) if it matches.
pub fn parse_message_header(line: &str) -> Option<(u64, String, String)> {
    MESSAGE_HEADER_RE.captures(line.trim()).map(|caps| {
        (
            caps[1].parse().unwrap_or(0),
            caps[2].to_string(),
            caps[3].to_string(),
        )
    })
}

/// Check if a line is exactly `---` (horizontal rule / message boundary marker).
pub fn is_boundary_line(line: &str) -> bool {
    line.trim() == "---"
}

/// Detect message boundaries in content.
///
/// A message boundary is a `---` line immediately followed (within 2 lines)
/// by a valid H3 header matching `### #\d+ — .+ · .+`.
/// A lone `---` NOT followed by this pattern is body content (invariant I12).
///
/// Returns a list of (boundary_line_index, header_line_index) pairs.
pub fn find_message_boundaries(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut boundaries = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if is_boundary_line(lines[i]) {
            // Look ahead within 2 lines for a valid H3 header
            let mut found = false;
            for offset in 1..=2 {
                if i + offset < lines.len() {
                    let candidate = lines[i + offset];
                    if parse_message_header(candidate).is_some() {
                        boundaries.push((i, i + offset));
                        // Skip past this boundary to avoid double-counting
                        i += offset + 1;
                        found = true;
                        break;
                    }
                }
            }
            if found {
                continue;
            }
            // If no header found within 2 lines, this --- is body content
        }
        i += 1;
    }

    boundaries
}

/// Parse comma-separated backtick-quoted glob patterns.
/// Input: `` `src/**`, `docs/**` `` → `vec!["src/**", "docs/**"]`
/// Empty scope (`—`) returns empty vec.
pub fn parse_scope_globs(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed == "—" || trimmed.is_empty() {
        return Vec::new();
    }

    trimmed
        .split(',')
        .filter_map(|part| {
            let part = part.trim();
            // Extract content between backticks
            if part.starts_with('`') && part.ends_with('`') && part.len() >= 2 {
                Some(part[1..part.len() - 1].to_string())
            } else if !part.is_empty() {
                // Allow unquoted values too for flexibility
                Some(part.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Serialize a list of glob patterns to comma-separated backtick-quoted format.
/// Empty list → `—`
pub fn serialize_scope_globs(globs: &[String]) -> String {
    if globs.is_empty() {
        "—".to_string()
    } else {
        globs
            .iter()
            .map(|g| format!("`{}`", g))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_crlf() {
        assert_eq!(normalize_line_endings("a\r\nb\r\nc"), "a\nb\nc");
        assert_eq!(normalize_line_endings("a\nb\nc"), "a\nb\nc");
        assert_eq!(normalize_line_endings("a\rb\rc"), "a\nb\nc");
    }

    #[test]
    fn test_extract_bold_key() {
        assert_eq!(
            extract_bold_key("**Model**: gpt-4"),
            Some(("Model".to_string(), "gpt-4".to_string()))
        );
        assert_eq!(
            extract_bold_key("**To**: alice"),
            Some(("To".to_string(), "alice".to_string()))
        );
        assert_eq!(extract_bold_key("not a bold key"), None);
        assert_eq!(extract_bold_key("**Empty**:"), Some(("Empty".to_string(), String::new())));
    }

    #[test]
    fn test_parse_message_header() {
        assert_eq!(
            parse_message_header("### #1 — alice · 2026-01-15T10:30:00Z"),
            Some((1, "alice".to_string(), "2026-01-15T10:30:00Z".to_string()))
        );
        assert_eq!(
            parse_message_header("### #42 — bob-agent · 2026-07-29T23:59:59Z"),
            Some((42, "bob-agent".to_string(), "2026-07-29T23:59:59Z".to_string()))
        );
        assert_eq!(parse_message_header("### not a message"), None);
        assert_eq!(parse_message_header("# #1 — alice · time"), None);
    }

    #[test]
    fn test_is_boundary_line() {
        assert!(is_boundary_line("---"));
        assert!(is_boundary_line("  ---  "));
        assert!(!is_boundary_line("----"));
        assert!(!is_boundary_line("--"));
        assert!(!is_boundary_line("text ---"));
    }

    #[test]
    fn test_find_message_boundaries_basic() {
        let content = "---\n\n### #1 — alice · 2026-01-15T10:30:00Z\n\nbody\n\n---\n\n### #2 — bob · 2026-01-15T11:00:00Z\n\nbody2";
        let lines: Vec<&str> = content.split('\n').collect();
        let boundaries = find_message_boundaries(&lines);
        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0], (0, 2));
        assert_eq!(boundaries[1], (6, 8));
    }

    #[test]
    fn test_find_message_boundaries_lone_hr() {
        // A --- NOT followed by header is body content
        let content = "---\n\n### #1 — alice · 2026-01-15T10:30:00Z\n\nbody with\n---\ninside\n\n---\n\n### #2 — bob · 2026-01-15T11:00:00Z";
        let lines: Vec<&str> = content.split('\n').collect();
        let boundaries = find_message_boundaries(&lines);
        // Only 2 real boundaries, the --- in body is ignored
        assert_eq!(boundaries.len(), 2);
    }

    #[test]
    fn test_parse_scope_globs() {
        assert_eq!(
            parse_scope_globs("`src/**`, `docs/**`"),
            vec!["src/**", "docs/**"]
        );
        assert_eq!(parse_scope_globs("—"), Vec::<String>::new());
        assert_eq!(parse_scope_globs(""), Vec::<String>::new());
        assert_eq!(parse_scope_globs("`single`"), vec!["single"]);
    }

    #[test]
    fn test_serialize_scope_globs() {
        assert_eq!(
            serialize_scope_globs(&["src/**".to_string(), "docs/**".to_string()]),
            "`src/**`, `docs/**`"
        );
        assert_eq!(serialize_scope_globs(&[]), "—");
    }

    #[test]
    fn test_scope_roundtrip() {
        let globs = vec!["src/**".to_string(), "lib/*.rs".to_string()];
        let serialized = serialize_scope_globs(&globs);
        let parsed = parse_scope_globs(&serialized);
        assert_eq!(globs, parsed);
    }
}
