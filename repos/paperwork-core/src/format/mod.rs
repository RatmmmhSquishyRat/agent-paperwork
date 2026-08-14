//! Format layer: parsing and serialization of managed Markdown files.
//!
//! Shared utilities for attribute-line extraction, fence-aware scanning
//! (CommonMark backtick-fence subset, spec §3.3), dynamic fence length
//! computation (spec §3.4), and CRLF normalization (invariant I11).

pub mod contacts;
pub mod manifest;
pub mod profile;
pub mod thread;

use std::borrow::Cow;

use chrono::{DateTime, Utc};
use regex::Regex;
use std::sync::LazyLock;

use crate::error::{PaperworkError, Result};

/// Normalize CRLF → LF (invariant I11).
/// All parsers must call this before processing.
///
/// Zero-copy when the content carries no `\r` at all (M-review n14).
pub fn normalize_line_endings(content: &str) -> Cow<'_, str> {
    if !content.contains('\r') {
        return Cow::Borrowed(content);
    }
    Cow::Owned(content.replace("\r\n", "\n").replace('\r', "\n"))
}

/// Canonical write-side RFC 3339 timestamp format (spec §3.5) — the single
/// format string shared by every timestamp serialization site (M-review M6).
pub const RFC3339_FMT: &str = "%Y-%m-%dT%H:%M:%SZ";

/// Parse timestamp from RFC 3339 string (spec §3.5).
///
/// Accepts any RFC 3339 offset (normalized to UTC); a timezone-less
/// `%Y-%m-%dT%H:%M:%S` is treated as UTC. Single shared implementation
/// for the whole format layer (M-review M6).
pub(crate) fn parse_timestamp(s: &str) -> std::result::Result<DateTime<Utc>, String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
    }
    Err(format!("cannot parse '{}' as RFC 3339 timestamp", s))
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
    ATTRIBUTE_RE
        .captures(line)
        .map(|caps| (caps[1].to_string(), caps[2].trim().to_string()))
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

/// Shared fence walk engine: run `f` over every fence-outside line; when
/// the walk ends (early stop or end of content) with a fence still open,
/// return its 0-based opening line number and backtick length.
///
/// `content` MUST already be normalized through [`normalize_line_endings`]
/// (callers pass `&Cow` from the parsers).
fn walk_outside_fence(
    content: &str,
    f: &mut impl FnMut(usize, &str) -> bool,
) -> Option<(usize, usize)> {
    let mut open: Option<(usize, usize)> = None; // (backtick len, 0-based open line)
    for (i, line) in content.split('\n').enumerate() {
        if let Some((n, _)) = open {
            if fence_close_matches(line, n) {
                open = None;
            }
            continue;
        }
        if let Some(n) = fence_open_len(line) {
            open = Some((n, i));
            continue;
        }
        if !f(i, line) {
            return open.map(|(n, line_no)| (line_no, n));
        }
    }
    open.map(|(n, line_no)| (line_no, n))
}

/// Run `f` over every fence-outside line (CommonMark backtick-fence subset,
/// spec §3.3). `f` receives the 0-based line number and the line; returning
/// `false` stops the walk early.
///
/// `content` MUST already be normalized through [`normalize_line_endings`].
pub fn for_each_outside_fence(content: &str, mut f: impl FnMut(usize, &str) -> bool) {
    walk_outside_fence(content, &mut f);
}

/// Short-circuit variant of [`for_each_outside_fence`]: the line number of
/// the FIRST fence-outside line satisfying `pred`, or `None`.
pub(crate) fn first_outside_fence(
    content: &str,
    pred: impl Fn(usize, &str) -> bool,
) -> Option<usize> {
    let mut found = None;
    for_each_outside_fence(content, |i, line| {
        if pred(i, line) {
            found = Some(i);
            return false;
        }
        true
    });
    found
}

/// Collecting variant of [`for_each_outside_fence`]: line numbers of ALL
/// fence-outside lines satisfying `pred`, in order of appearance.
// P-3 pull-forward: the fence-state-machine call-site migration consumes
// this helper; pinned by the inline scanner corpus until then.
#[allow(dead_code)]
pub(crate) fn collect_outside_fence(
    content: &str,
    pred: impl Fn(usize, &str) -> bool,
) -> Vec<usize> {
    let mut hits = Vec::new();
    for_each_outside_fence(content, |i, line| {
        if pred(i, line) {
            hits.push(i);
        }
        true
    });
    hits
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

// ============================================================================
// Write-side injection guardrails (NEW-1)
// ============================================================================

/// Single-line field guard (NEW-1): refuse `\n` / `\r` inside a value that
/// serializes as a single structural line (titles, names, models, paths).
/// An embedded newline would inject structure into the managed file.
pub fn check_single_line(field_name: &str, value: &str) -> Result<()> {
    if value.contains('\n') || value.contains('\r') {
        return Err(PaperworkError::Validation {
            message: format!(
                "{} contains a line break; single-line fields cannot span multiple lines",
                field_name
            ),
            fix: format!(
                "keep {} on a single line; remove newline and carriage-return characters",
                field_name
            ),
            example: format!("paperwork <command> with {} as one single line", field_name),
        });
    }
    Ok(())
}

/// M1 first-line representability check shared by every prose carrier.
///
/// Returns a reason when the FIRST non-blank line is attribute-shaped
/// (`- key: value`) or opens a ```` ```regex ```` fence: the
/// attribute-zone rule (blank lines do not terminate it) would re-absorb
/// that line as an attribute or as a regex carrier on the next parse.
/// The reason wording is the historical M1 wording (existing tests pin it).
pub fn first_line_representation_issue(prose: &str) -> Option<&'static str> {
    let first = prose.lines().find(|l| !l.trim().is_empty())?;
    if extract_attribute(first).is_some() {
        return Some("note starts with an attribute-shaped line '- key: value'");
    }
    if fence_info(first) == "regex" && fence_open_len(first).is_some() {
        return Some("note starts with a ```regex fence opening line");
    }
    None
}

/// Representability check for PREAMBLE prose (profile description, brief
/// description) — the unified helper (NEW-1).
///
/// On top of [`first_line_representation_issue`] it rejects ANY line that
/// is attribute-shaped with a known structural key
/// (`model` / `owner` / `created` / `path` / `hash` / `regex`): preamble
/// prose is serialized bare (unfenced) BEFORE the real attribute lines, and
/// first-match-wins parsing would let the embedded line shadow the real
/// structure (e.g. a profile description carrying `- model: fake` preempts
/// the real `- model:` line on the next parse).
///
/// Brief entry NOTES use only the first-line check: a note sits AFTER the
/// entry attribute zone, so later attribute-shaped lines inside a note are
/// verbatim content and stay legal.
pub fn prose_representation_issue(prose: &str) -> Option<&'static str> {
    if let Some(reason) = first_line_representation_issue(prose) {
        return Some(reason);
    }
    if contains_dangerous_attribute_line(prose) {
        return Some(
            "prose embeds an attribute-shaped line with a known structural key \
             ('- model:', '- owner:', '- created:', '- path:', '- hash:' or '- regex:')",
        );
    }
    None
}

/// Attribute keys whose bullet-shaped lines inside bare prose would shadow
/// real structural attributes on re-parse (first match wins per key).
const DANGEROUS_ATTRIBUTE_KEYS: &[&str] = &["model", "owner", "created", "path", "hash", "regex"];

/// Whether any line of the prose is attribute-shaped with one of the
/// [`DANGEROUS_ATTRIBUTE_KEYS`].
///
/// Deliberately NOT fence-aware: the preamble parsers (profile /
/// brief) run [`extract_attribute`] on every preamble line without fence
/// tracking, so a fence inside the prose does NOT shield an embedded
/// `- model:`-style line from being re-absorbed as an attribute on the
/// next parse. The write-side guard mirrors exactly what the parser sees.
pub fn contains_dangerous_attribute_line(prose: &str) -> bool {
    let prose = normalize_line_endings(prose);
    for line in prose.lines() {
        if let Some((key, _)) = extract_attribute(line) {
            if DANGEROUS_ATTRIBUTE_KEYS.contains(&key.as_str()) {
                return true;
            }
        }
    }
    false
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

    // ========================================================================
    // Shared fence scanner family (P-2 pull-forward of the P-3 helpers)
    // ========================================================================

    #[test]
    fn test_outside_fence_scanner_corpus() {
        // first / collect see fence-outside lines only
        let content = "```md\n## fake\n```\n## real\n## real2";
        let norm = normalize_line_endings(content);
        assert_eq!(
            first_outside_fence(&norm, |_i, l| l.starts_with("## ")),
            Some(3)
        );
        assert_eq!(
            collect_outside_fence(&norm, |_i, l| l.starts_with("## ")),
            vec![3, 4]
        );

        // <= 3 space indented fence is recognized; 4-space indent is not
        let norm = normalize_line_endings("   ```\nhidden\n   ```\nvisible");
        assert_eq!(
            collect_outside_fence(&norm, |_i, l| l == "hidden" || l == "visible"),
            vec![3]
        );
        let norm = normalize_line_endings("    ```\nnot-hidden");
        assert_eq!(
            collect_outside_fence(&norm, |_i, l| l == "not-hidden"),
            vec![1]
        );

        // tilde fences are not recognized
        let norm = normalize_line_endings("~~~\nvisible\n~~~");
        assert_eq!(
            collect_outside_fence(&norm, |_i, l| l == "visible"),
            vec![1]
        );

        // unclosed fence swallows the tail
        let norm = normalize_line_endings("```\nhidden");
        assert!(collect_outside_fence(&norm, |_i, l| l == "hidden").is_empty());

        // nested backtick length: shorter run does not close the fence
        let norm = normalize_line_endings("````\nhidden\n```\nstill-hidden\n````\nvisible");
        assert_eq!(
            collect_outside_fence(&norm, |_i, l| l == "visible"),
            vec![5]
        );

        // early stop
        let mut seen = Vec::new();
        let norm = normalize_line_endings("a\nb\nc");
        for_each_outside_fence(&norm, |_i, l| {
            seen.push(l.to_string());
            l != "b"
        });
        assert_eq!(seen, vec!["a", "b"]);

        // empty content: `"".split('\n')` yields one empty line, so an
        // always-true predicate sees it; concrete predicates never match.
        assert_eq!(collect_outside_fence("", |_i, _l| true), vec![0]);
        assert!(collect_outside_fence("", |_i, l| !l.is_empty()).is_empty());
    }

    #[test]
    fn test_check_single_line_guard() {
        assert!(check_single_line("title", "fine single line").is_ok());
        assert!(check_single_line("title", "").is_ok());
        let err = check_single_line("title", "two\nlines").unwrap_err();
        assert_eq!(err.category(), "validation");
        assert!(err.to_string().contains("title"));
        let err = check_single_line("name", "carriage\rreturn").unwrap_err();
        assert_eq!(err.category(), "validation");
        // CRLF is refused as well
        assert!(check_single_line("model", "a\r\nb").is_err());
    }

    #[test]
    fn test_representation_issue_helpers() {
        // first-line shapes
        assert_eq!(
            first_line_representation_issue("- model: fake"),
            Some("note starts with an attribute-shaped line '- key: value'")
        );
        assert_eq!(
            first_line_representation_issue("\n\n```regex\nx"),
            Some("note starts with a ```regex fence opening line")
        );
        assert_eq!(
            first_line_representation_issue("Prose.\n- model: later"),
            None
        );

        // preamble prose: dangerous key ANYWHERE is refused
        assert!(prose_representation_issue("Prose.\n- model: fake").is_some());
        assert!(prose_representation_issue("Prose.\n- hash: deadbeef").is_some());
        // unknown keys stay legal
        assert!(prose_representation_issue("Line one.\n- unknown-key: fine").is_none());
        // fences do NOT shield (mirrors the non-fence-aware parsers)
        assert!(prose_representation_issue("Prose.\n```\n- owner: x\n```").is_some());

        // dangerous key scan standalone
        assert!(contains_dangerous_attribute_line("- path: /etc/passwd"));
        assert!(contains_dangerous_attribute_line("- regex: x"));
        assert!(!contains_dangerous_attribute_line("- created-at: prose"));
        assert!(!contains_dangerous_attribute_line(""));
    }
}
