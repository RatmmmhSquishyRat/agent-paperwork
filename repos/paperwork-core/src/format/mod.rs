//! Format layer: parsing and serialization of managed Markdown files.
//!
//! Shared utilities for attribute-line extraction, fence-aware scanning
//! (CommonMark backtick-fence subset, spec §3.3), dynamic fence length
//! computation (spec §3.4), and CRLF normalization (invariant I11).

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

/// Regex for attribute lines (spec §3.2): `- key: value` with a lowercase
/// ASCII key (letters, digits, hyphens; first character a letter).
static ATTRIBUTE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^- ([a-z][a-z0-9-]*):\s*(.*)$").expect("valid regex"));

/// Extract an attribute (key, value) from a line.
///
/// Returns `Some((key, value))` if the line matches `- key: value` with a
/// lowercase key; the value is the trimmed text after the colon (may be
/// empty). Uppercase-key bullets (legacy format) never match and are treated
/// as unknown content (spec §3.6).
pub fn extract_attribute(line: &str) -> Option<(String, String)> {
    ATTRIBUTE_RE.captures(line).map(|caps| {
        (
            caps[1].to_string(),
            caps[2].trim().to_string(),
        )
    })
}

/// Count the leading spaces of a line (capped: returns `None` once more
/// than 3 leading spaces are seen).
///
/// CommonMark fence policy (spec §3.3, R13): a fence line may be indented by
/// at most 3 spaces; >= 4 spaces makes it an indented code block line with
/// no fence semantics.
fn leading_indent_ok(line: &str) -> Option<&str> {
    let mut spaces = 0usize;
    for ch in line.chars() {
        if ch == ' ' {
            spaces += 1;
            if spaces > 3 {
                return None;
            }
        } else {
            return Some(&line[spaces..]);
        }
    }
    // All-spaces (or empty) line: not a fence line.
    None
}

/// Length of the leading backtick run of a line, if the line is a candidate
/// fence line (<= 3 leading spaces followed immediately by backticks).
///
/// Returns `Some(n)` with the run length (any n >= 1), or `None` when the
/// line is not a candidate fence line. Callers decide open/close semantics
/// (a fence requires n >= 3).
pub fn backtick_run(line: &str) -> Option<usize> {
    let rest = leading_indent_ok(line)?;
    let run = rest.bytes().take_while(|&b| b == b'`').count();
    if run == 0 {
        None
    } else {
        Some(run)
    }
}

/// Fence-opening length of a line, if it opens a fence.
///
/// An opening line is a backtick run of length N >= 3 (<= 3 leading spaces),
/// optionally followed by any info string (spec §3.3).
pub fn fence_open_len(line: &str) -> Option<usize> {
    let run = backtick_run(line)?;
    if run >= 3 {
        Some(run)
    } else {
        None
    }
}

/// Whether a line closes a fence opened with `open_len` backticks.
///
/// Closing rule (CommonMark, spec §3.3): backtick run length >= `open_len`,
/// the line consists of backticks and whitespace only (no info string),
/// <= 3 leading spaces.
pub fn fence_close_matches(line: &str, open_len: usize) -> bool {
    let rest = match leading_indent_ok(line) {
        Some(rest) => rest,
        None => return false,
    };
    let run = rest.bytes().take_while(|&b| b == b'`').count();
    run >= open_len && rest[run..].chars().all(|c| c.is_whitespace())
}

/// Info string of a fence opening line (text after the backtick run, trimmed).
pub fn fence_info(line: &str) -> String {
    let rest = match leading_indent_ok(line) {
        Some(rest) => rest,
        None => return String::new(),
    };
    let run = rest.bytes().take_while(|&b| b == b'`').count();
    rest[run..].trim().to_string()
}

/// Dynamic fence length for serializing a user-content body (spec §3.4):
/// `max(3, longest consecutive backtick run in body + 1)`.
pub fn compute_fence_length(body: &str) -> usize {
    let mut longest = 0usize;
    let mut current = 0usize;
    for ch in body.chars() {
        if ch == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    (longest + 1).max(3)
}

/// Validate basic Markdown structure of content: fence closure.
///
/// Fence-aware per spec §3.3 (CommonMark length rules: an N-backtick fence
/// is only closed by a backtick-only line of length >= N; <= 3 spaces indent;
/// tilde fences are not recognized). Returns a list of issues; empty = valid.
/// An unclosed fence is reported with its 1-based opening line number.
pub fn validate_markdown(content: &str) -> Vec<String> {
    let content = normalize_line_endings(content);
    let mut issues = Vec::new();

    let mut open_len: Option<(usize, usize)> = None; // (backtick len, 1-based line no)

    for (i, line) in content.lines().enumerate() {
        let line_no = i + 1;
        if let Some((n, _start)) = open_len {
            if fence_close_matches(line, n) {
                open_len = None;
            }
            continue;
        }
        if let Some(n) = fence_open_len(line) {
            open_len = Some((n, line_no));
        }
    }

    if let Some((n, start)) = open_len {
        issues.push(format!(
            "unclosed code fence ({} backticks) opened at line {}",
            n, start
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
    fn test_extract_attribute() {
        // lowercase key hits
        assert_eq!(
            extract_attribute("- model: gpt-4o"),
            Some(("model".to_string(), "gpt-4o".to_string()))
        );
        // uppercase key (legacy) never matches
        assert_eq!(extract_attribute("- Model: gpt-4o"), None);
        // brief key with a plain value
        assert_eq!(
            extract_attribute("- owner: alice"),
            Some(("owner".to_string(), "alice".to_string()))
        );
        // brief key with an RFC3339 timestamp value
        assert_eq!(
            extract_attribute("- created: 2026-01-15T10:00:00Z"),
            Some(("created".to_string(), "2026-01-15T10:00:00Z".to_string()))
        );
        // empty value allowed
        assert_eq!(
            extract_attribute("- regex:"),
            Some(("regex".to_string(), String::new()))
        );
        // non-attribute lines
        assert_eq!(extract_attribute("not an attribute"), None);
        assert_eq!(extract_attribute("  - model: indented"), None);
        assert_eq!(extract_attribute("# heading"), None);
    }

    #[test]
    fn test_fence_scan() {
        // opening: N backticks with any info string
        assert_eq!(fence_open_len("```markdown"), Some(3));
        assert_eq!(fence_open_len("```"), Some(3));
        assert_eq!(fence_open_len("````regex"), Some(4));
        assert_eq!(fence_open_len("``no-fence"), None);
        // <= 3 leading spaces: still a fence line
        assert_eq!(fence_open_len("   ```markdown"), Some(3));
        // >= 4 leading spaces: indented code block, not a fence
        assert_eq!(fence_open_len("    ```markdown"), None);
        // tilde fences are not recognized anywhere
        assert_eq!(fence_open_len("~~~"), None);
        assert!(!fence_close_matches("~~~", 3));

        // closing: pure-backtick line with length >= open_len
        assert!(fence_close_matches("```", 3));
        assert!(fence_close_matches("````", 3));
        assert!(fence_close_matches("  ```", 3));
        // < N does not close
        assert!(!fence_close_matches("```", 4));
        // info string disqualifies a closing line
        assert!(!fence_close_matches("```markdown", 3));
        // trailing whitespace ok
        assert!(fence_close_matches("````  ", 4));
        // >= 4 spaces indent: not a closing line
        assert!(!fence_close_matches("    ```", 3));

        // fence-internal structure lines are not boundaries:
        // simulate a scan where `## #99 ...` sits inside an open fence
        let lines = [
            "```markdown",
            "## #99 mallory (2026-01-01T00:00:00Z)",
            "```",
            "## #2 bob (2026-01-01T00:00:01Z)",
        ];
        let mut open: Option<usize> = None;
        let mut boundaries = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if let Some(n) = open {
                if fence_close_matches(line, n) {
                    open = None;
                }
                continue;
            }
            if let Some(n) = fence_open_len(line) {
                open = Some(n);
                continue;
            }
            if line.starts_with("## #") {
                boundaries.push(i);
            }
        }
        assert_eq!(boundaries, vec![3]);
    }

    #[test]
    fn test_compute_fence_length() {
        assert_eq!(compute_fence_length(""), 3);
        assert_eq!(compute_fence_length("no backticks"), 3);
        assert_eq!(compute_fence_length("single ` tick"), 3);
        assert_eq!(compute_fence_length("run of ``"), 3);
        assert_eq!(compute_fence_length("run of ```"), 4);
        assert_eq!(compute_fence_length("run of ````"), 5);
        assert_eq!(compute_fence_length("run of `````"), 6);
        assert_eq!(compute_fence_length("run of ``````"), 7);
        // longest run wins
        assert_eq!(compute_fence_length("` and ````` and ``"), 6);
    }

    #[test]
    fn test_validate_markdown_dynamic() {
        // valid: closed fences of various lengths
        assert!(validate_markdown("# t\n\n```rust\nfn main() {}\n```\n").is_empty());
        assert!(validate_markdown("`````markdown\n````\n`````\n").is_empty());

        // unclosed 3-backtick fence reports opening line number
        let issues = validate_markdown("# t\n\n```rust\nfn main() {}\n");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("unclosed"));
        assert!(issues[0].contains("line 3"));

        // unclosed 5-backtick fence
        let issues = validate_markdown("line\n`````markdown\nbody\n");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("5 backticks"));
        assert!(issues[0].contains("line 2"));

        // nested: longer fence containing shorter fence lines is valid
        assert!(validate_markdown("````markdown\n```\nx\n```\n````\n").is_empty());

        // shorter fence line does NOT close a longer fence
        let issues = validate_markdown("````markdown\n```\n`````\n");
        assert!(issues.is_empty()); // closed by the 5-backtick line

        // >= 4 space indented backtick line is not a fence (no issue reported)
        assert!(validate_markdown("    ```not-a-fence\n").is_empty());

        // tilde fences are not recognized: the ``` line opens a backtick
        // fence that ~~~ never closes
        let issues = validate_markdown("~~~\n```\n~~~\n");
        // the ``` line opens a backtick fence that is never closed
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("line 2"));
    }
}
