//! Format layer: parsing and serialization of managed Markdown files.
//!
//! Shared utilities for boundary detection, bullet-key extraction, and CRLF normalization.

pub mod contacts;
pub mod manifest;
pub mod profile;
pub mod thread;

use regex::Regex;
use std::sync::LazyLock;

/// Normalize CRLF → LF (invariant I11).
/// All parsers must call this before processing.
pub fn normalize_line_endings(content: &str) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
}

/// Regex for extracting bullet-key metadata lines: `- Key: value`
static BULLET_KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^- ([^:]+):\s*(.*)$").expect("valid regex"));

/// Regex for message H3 header: `### #<seq> <sender> · <timestamp>`
static MESSAGE_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^### #(\d+) (.+) · (.+)$").expect("valid regex"));

/// Extract a bullet-key value from a line.
/// Returns (key, value) if the line matches `- Key: value`.
pub fn extract_bullet_key(line: &str) -> Option<(String, String)> {
    BULLET_KEY_RE.captures(line.trim()).map(|caps| {
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

/// Check if a line opens or closes a 4-backtick fence.
fn is_four_backtick_fence(line: &str) -> bool {
    line.trim().starts_with("````")
}

/// Detect message boundaries in content (fence-aware).
///
/// A message boundary is a `---` line immediately followed (within 2 lines)
/// by a valid H3 header matching `### #\d+ .+ · .+`.
/// A `---` inside a 4-backtick fenced code block is NEVER a boundary.
///
/// Returns a list of (boundary_line_index, header_line_index) pairs.
pub fn find_message_boundaries(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut boundaries = Vec::new();
    let mut i = 0;
    let mut in_fence = false;

    while i < lines.len() {
        // Track fence state
        if is_four_backtick_fence(lines[i]) {
            in_fence = !in_fence;
            i += 1;
            continue;
        }

        if !in_fence && is_boundary_line(lines[i]) {
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

/// Validate basic Markdown structure of content.
///
/// Returns a list of warning/error messages. Empty vec = valid.
/// Checks:
/// - Unclosed fenced code blocks (``` or ````)
/// - Unclosed 4-backtick fences
pub fn validate_markdown(content: &str) -> Vec<String> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();
    let mut issues = Vec::new();

    // Track fence state
    let mut in_four_fence = false;
    let mut four_fence_start = 0;
    let mut in_three_fence = false;
    let mut three_fence_start = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if in_four_fence {
            // Inside a 4-backtick fence, only ```` closes it
            if trimmed == "````" {
                in_four_fence = false;
            }
            continue;
        }

        if in_three_fence {
            if trimmed == "```" || (trimmed.starts_with("```") && !trimmed.starts_with("````")) {
                in_three_fence = false;
            }
            continue;
        }

        // Not inside any fence
        if trimmed.starts_with("````") {
            in_four_fence = true;
            four_fence_start = i + 1; // 1-based line number
        } else if trimmed.starts_with("```") {
            in_three_fence = true;
            three_fence_start = i + 1;
        }
    }

    if in_four_fence {
        issues.push(format!(
            "unclosed 4-backtick fence opened at line {}",
            four_fence_start
        ));
    }
    if in_three_fence {
        issues.push(format!(
            "unclosed 3-backtick fence opened at line {}",
            three_fence_start
        ));
    }

    issues
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
    fn test_extract_bullet_key() {
        assert_eq!(
            extract_bullet_key("- Model: gpt-4"),
            Some(("Model".to_string(), "gpt-4".to_string()))
        );
        assert_eq!(
            extract_bullet_key("- To: alice"),
            Some(("To".to_string(), "alice".to_string()))
        );
        assert_eq!(extract_bullet_key("not a bullet key"), None);
        assert_eq!(
            extract_bullet_key("- Empty:"),
            Some(("Empty".to_string(), String::new()))
        );
        assert_eq!(
            extract_bullet_key("- Reply-To: #1"),
            Some(("Reply-To".to_string(), "#1".to_string()))
        );
    }

    #[test]
    fn test_parse_message_header() {
        assert_eq!(
            parse_message_header("### #1 alice · 2026-01-15T10:30:00Z"),
            Some((1, "alice".to_string(), "2026-01-15T10:30:00Z".to_string()))
        );
        assert_eq!(
            parse_message_header("### #42 bob-agent · 2026-07-29T23:59:59Z"),
            Some((42, "bob-agent".to_string(), "2026-07-29T23:59:59Z".to_string()))
        );
        assert_eq!(parse_message_header("### not a message"), None);
        assert_eq!(parse_message_header("# #1 alice · time"), None);
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
        let content = "---\n\n### #1 alice · 2026-01-15T10:30:00Z\n\nbody\n\n---\n\n### #2 bob · 2026-01-15T11:00:00Z\n\nbody2";
        let lines: Vec<&str> = content.split('\n').collect();
        let boundaries = find_message_boundaries(&lines);
        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0], (0, 2));
        assert_eq!(boundaries[1], (6, 8));
    }

    #[test]
    fn test_find_message_boundaries_lone_hr() {
        // A --- NOT followed by header is body content
        let content = "---\n\n### #1 alice · 2026-01-15T10:30:00Z\n\nbody with\n---\ninside\n\n---\n\n### #2 bob · 2026-01-15T11:00:00Z";
        let lines: Vec<&str> = content.split('\n').collect();
        let boundaries = find_message_boundaries(&lines);
        // Only 2 real boundaries, the --- in body is ignored
        assert_eq!(boundaries.len(), 2);
    }

    #[test]
    fn test_find_message_boundaries_fence_aware() {
        // --- inside a 4-backtick fence should NOT be a boundary
        let content = "---\n\n### #1 alice · 2026-01-15T10:30:00Z\n\n````markdown\n---\n### #99 fake · 2026-01-01T00:00:00Z\n````\n\n---\n\n### #2 bob · 2026-01-15T11:00:00Z";
        let lines: Vec<&str> = content.split('\n').collect();
        let boundaries = find_message_boundaries(&lines);
        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0], (0, 2));
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

    #[test]
    fn test_validate_markdown_valid() {
        let content = "# Hello\n\nSome text\n\n```rust\nfn main() {}\n```\n";
        assert!(validate_markdown(content).is_empty());
    }

    #[test]
    fn test_validate_markdown_unclosed_three_fence() {
        let content = "# Hello\n\n```rust\nfn main() {}\n";
        let issues = validate_markdown(content);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("unclosed 3-backtick fence"));
    }

    #[test]
    fn test_validate_markdown_unclosed_four_fence() {
        let content = "# Hello\n\n````markdown\nSome content\n";
        let issues = validate_markdown(content);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("unclosed 4-backtick fence"));
    }

    #[test]
    fn test_validate_markdown_nested_fences() {
        // 4-backtick fence containing 3-backtick fence is valid
        let content = "````markdown\n```rust\nfn main() {}\n```\n````\n";
        let issues = validate_markdown(content);
        assert!(issues.is_empty());
    }
}
