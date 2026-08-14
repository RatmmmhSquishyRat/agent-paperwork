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
/// When `\r` is present the rewrite runs in a SINGLE pass with a single
/// allocation (T3 NEW-9): `\r\n` → `\n`, lone `\r` → `\n`, byte-for-byte
/// identical to the historical two-step `replace("\r\n","\n")` +
/// `replace('\r','\n')` semantics.
pub fn normalize_line_endings(content: &str) -> Cow<'_, str> {
    let bytes = content.as_bytes();
    let Some(first_cr) = bytes.iter().position(|&b| b == b'\r') else {
        return Cow::Borrowed(content);
    };

    let mut out = Vec::with_capacity(bytes.len());
    // Copy verbatim up to the first `\r`, then walk the rest once.
    out.extend_from_slice(&bytes[..first_cr]);
    let mut i = first_cr;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            out.push(b'\n');
            // `\r\n` collapses as one terminator; a lone `\r` alone.
            i += if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                2
            } else {
                1
            };
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'\r' {
                i += 1;
            }
            out.extend_from_slice(&bytes[start..i]);
        }
    }
    // Only ASCII line-ending bytes are rewritten, so UTF-8 validity holds.
    Cow::Owned(String::from_utf8(out).expect("line-ending rewrite preserves UTF-8"))
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

// ============================================================================
// Shared fence-aware line scanners (T3: collapses the per-site
// `open: Option<usize>` state-machine loops; T4 migrates the call sites)
// ============================================================================

/// Fence-aware line walk: invoke `f(line_no, line)` for every line OUTSIDE
/// any code fence, in order of appearance.
///
/// Calling convention: `content` MUST already be normalized through
/// [`normalize_line_endings`] — the scanner splits on `\n` only (no heap
/// allocation; line slices borrow `content`).
///
/// Semantics are byte-for-byte those of the historical per-site loops
/// (authoritative reference: `format/thread.rs` `header_indices`):
/// - opening fence = backtick run >= 3 with <= 3 leading spaces
///   ([`fence_open_len`] / `leading_indent_ok` stance);
/// - closing fence = same character, length >= opening, backticks and
///   whitespace only ([`fence_close_matches`]);
/// - an unclosed fence swallows everything to end of content;
/// - tilde fences are NOT recognized (plain lines);
/// - the fence opening/closing lines themselves are never reported;
/// - `\n`-split note: a trailing newline yields one final empty line
///   (harmless for every current predicate, which never matches `""`).
///
/// `f` receives the 0-based line number and the line without its line
/// terminator; returning `false` stops the walk early.
///
/// This is the single fence-state-machine of the whole crate (T4: every
/// fence-aware scan funnels through it; CLI cross-crate users go through
/// the `pub` variants below — the only sanctioned cross-layer surface).
pub fn for_each_outside_fence(content: &str, mut f: impl FnMut(usize, &str) -> bool) {
    walk_outside_fence(content, &mut f);
}

/// Shared fence walk engine: run `f` over every fence-outside line; when
/// the walk ends (early stop or end of content) with a fence still open,
/// return its 0-based opening line number and backtick length.
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

/// Fence still open at end of content, if any: `(0-based opening line
/// number, backtick length)` of the swallowing fence.
///
/// `content` MUST already be normalized through [`normalize_line_endings`]
/// (same calling convention as [`for_each_outside_fence`]).
pub fn unclosed_fence(content: &str) -> Option<(usize, usize)> {
    walk_outside_fence(content, &mut |_i, _line| true)
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

    if let Some((start, n)) = unclosed_fence(&content) {
        issues.push(format!(
            "unclosed code fence ({} backticks) opened at line {}",
            n,
            start + 1
        ));
    }

    issues
}

// ============================================================================
// Shared small helpers (T4 DRY convergence)
// ============================================================================

/// Order-preserving deduplication (HashSet + Vec): keep the FIRST occurrence
/// of each item, O(n) overall. T4/NEW-10: replaces the historical O(n²)
/// `Vec::contains` loops (`thread_summary` participants, `derive_mentions`).
pub fn dedup_preserve_order<I, S>(items: I) -> Vec<S>
where
    I: IntoIterator<Item = S>,
    S: Eq + std::hash::Hash + Clone,
{
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for item in items {
        if seen.insert(item.clone()) {
            out.push(item);
        }
    }
    out
}

/// Strip the known managed-file suffixes from a file name: `.profile.md`
/// first, then `.post.md`, then `.md`; anything else is kept as-is
/// (spec §7.3 R11 label fallback). Single shared definition (T4; T5 wired
/// the CLI `default_title` onto it — Sam-m-γ): consumed by
/// `ops/contacts.rs::derive_label` and `cmd/post.rs::default_title`.
pub fn strip_known_suffix(file_name: &str) -> &str {
    file_name
        .strip_suffix(".profile.md")
        .or_else(|| file_name.strip_suffix(".post.md"))
        .or_else(|| file_name.strip_suffix(".md"))
        .unwrap_or(file_name)
}

// ============================================================================
// Write-side representability guards (NEW-1 injection guardrails)
// ============================================================================

/// Write-side guard for single-line fields (titles, names, labels, paths).
///
/// A value carrying `\n` or `\r` cannot survive serialization: single-line
/// fields are emitted inline (H1 / attribute line / link bullet), so an
/// embedded newline would inject extra structural lines into the managed
/// file. The parse side stays lenient; only the write side refuses.
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
/// Ultra Review F1 (write/read symmetry): on top of that it rejects a
/// fence-outside RESERVED HEADING SHAPE line (`## Entries` or `### ...`)
/// — the same shape the brief residue guard
/// (`contains_legacy_brief_residue` in `format/manifest.rs`) refuses at
/// parse time, so a write smuggling it into prose would produce a file the
/// tool itself can never read back. Fence-internal occurrences are quoted
/// content and stay legal.
///
/// Brief entry NOTES use the first-line check plus the reserved-heading
/// check (`note_representation_issue` in `format/manifest.rs`): a note
/// sits AFTER the entry attribute zone, so later attribute-shaped lines
/// inside a note are verbatim content and stay legal — but a fence-outside
/// reserved heading shape trips the FILE-level residue guard on the next
/// parse, so the write side refuses it too.
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
    if contains_reserved_heading_shape(prose) {
        return Some(
            "prose embeds a reserved heading shape line ('## Entries' or '### ...') \
             outside any code fence",
        );
    }
    None
}

/// Attribute keys whose bullet-shaped lines inside bare prose would shadow
/// real structural attributes on re-parse (first match wins per key).
const DANGEROUS_ATTRIBUTE_KEYS: &[&str] = &["model", "owner", "created", "path", "hash", "regex"];

/// Whether any line of the prose is attribute-shaped with one of the
/// `DANGEROUS_ATTRIBUTE_KEYS`.
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

/// Fence-aware reserved-heading-shape detection (Ultra Review F1).
///
/// Returns true when any fence-OUTSIDE line has a trimmed form equal to
/// `## Entries` or starting with `### ` — exactly the shapes the brief
/// residue guard (`contains_legacy_brief_residue` in `format/manifest.rs`)
/// refuses at parse time. The write-side guards
/// ([`prose_representation_issue`] / `note_representation_issue` in
/// `format/manifest.rs`) call this so the tool can never write a brief it
/// cannot read back. Fence-internal occurrences are quoted content and
/// stay legal; the fence semantics are byte-for-byte those of the shared
/// scanner family ([`first_outside_fence`]), so write side and read side
/// agree on every edge case (indent stance, tilde fences, unclosed fences,
/// CRLF).
pub fn contains_reserved_heading_shape(content: &str) -> bool {
    let content = normalize_line_endings(content);
    first_outside_fence(&content, |_i, line| {
        let trimmed = line.trim();
        trimmed == "## Entries" || trimmed.starts_with("### ")
    })
    .is_some()
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

    // NEW-9: single-pass rewrite must stay byte-for-byte equivalent to the
    // historical two-step replace chain, in all four line-ending states.
    #[test]
    fn test_normalize_single_pass_equivalence() {
        // mixed CRLF + LF + lone CR in one buffer
        assert_eq!(normalize_line_endings("a\r\nb\nc\rd\r\n"), "a\nb\nc\nd\n");
        // `\r\r\n` = lone CR + CRLF -> two terminators (parity with old impl)
        assert_eq!(normalize_line_endings("x\r\r\ny"), "x\n\ny");
        // `\r\n\r` = CRLF + lone CR -> two terminators
        assert_eq!(normalize_line_endings("x\r\n\ry"), "x\n\ny");
        // leading / trailing lone CR
        assert_eq!(normalize_line_endings("\ra\r"), "\na\n");
        // pure terminators
        assert_eq!(normalize_line_endings("\r\n"), "\n");
        assert_eq!(normalize_line_endings("\r"), "\n");
    }

    // NEW-9: no-`\r` content stays borrowed (zero allocation).
    #[test]
    fn test_normalize_borrowed_state() {
        let plain = "a\nb\nc";
        assert!(matches!(normalize_line_endings(plain), Cow::Borrowed(_)));
        assert!(matches!(normalize_line_endings(""), Cow::Borrowed(_)));
        assert!(matches!(normalize_line_endings("a\r\nb"), Cow::Owned(_)));
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

        // T4 differential: CRLF input behaves like LF (1-based line numbers)
        assert!(validate_markdown("# t\r\n```rust\r\nx\r\n```\r\n").is_empty());
        let issues = validate_markdown("# t\r\n\r\n```rust\r\nfn main() {}\r\n");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("line 3"));
        // empty content
        assert!(validate_markdown("").is_empty());
    }

    #[test]
    fn test_check_single_line() {
        assert!(check_single_line("title", "plain title").is_ok());
        assert!(check_single_line("title", "").is_ok());
        assert!(check_single_line("title", "unicode 标题 🚀").is_ok());
        for bad in [
            "two\nlines",
            "carriage\rreturn",
            "crlf\r\nbreak",
            "\n",
            "\r",
        ] {
            let err = check_single_line("title", bad).expect_err("must reject");
            assert_eq!(err.category(), "validation");
            assert!(err.to_string().contains("title"));
        }
    }

    #[test]
    fn test_prose_representation_issue() {
        // legal multi-line prose
        assert!(prose_representation_issue("First line.\nSecond line.").is_none());
        assert!(prose_representation_issue("").is_none());
        // harmless attribute-shaped line with an unknown key is fine
        assert!(prose_representation_issue("Prose.\n- anything: value").is_none());
        // attribute-shaped first line (M1, legacy wording preserved)
        assert_eq!(
            prose_representation_issue("- path: x\nrest"),
            Some("note starts with an attribute-shaped line '- key: value'")
        );
        // ```regex fence opening first line (M1, legacy wording preserved)
        assert_eq!(
            prose_representation_issue("\n```regex\nx\n```\n"),
            Some("note starts with a ```regex fence opening line")
        );
        // dangerous structural key embedded on a later line
        for key in DANGEROUS_ATTRIBUTE_KEYS {
            let prose = format!("Prose first.\n- {}: fake", key);
            assert!(
                prose_representation_issue(&prose).is_some(),
                "embedded '- {}:' must be rejected",
                key
            );
        }
        // a dangerous line inside a fence is refused too: the preamble
        // parsers are not fence-aware, so the fence does not shield it
        assert!(prose_representation_issue("Prose.\n```\n- model: quoted\n```\n").is_some());
    }

    // Ultra Review F1: fence-aware reserved heading shape detection —
    // the write side must refuse exactly the shapes the read-side brief
    // residue guard refuses, and nothing else.
    #[test]
    fn test_reserved_heading_shape_fence_aware() {
        // fence-outside reserved shapes are detected (trim stance matches
        // the read-side residue guard)
        assert!(contains_reserved_heading_shape("Prose.\n## Entries"));
        assert!(contains_reserved_heading_shape("Prose.\n### sub"));
        assert!(contains_reserved_heading_shape("Prose.\n  ### indented"));
        assert!(contains_reserved_heading_shape("## Entries\n"));
        // fence-internal occurrences are quoted content
        assert!(!contains_reserved_heading_shape(
            "Prose.\n```\n### sub\n```\n"
        ));
        assert!(!contains_reserved_heading_shape("```\n## Entries\n```"));
        // an unclosed fence swallows the tail
        assert!(!contains_reserved_heading_shape("Prose.\n```\n### sub"));
        // 4-space indent: no fence, the shape stays visible
        assert!(contains_reserved_heading_shape("    ```\n### sub\n    ```"));
        // tilde fences are not fences
        assert!(contains_reserved_heading_shape("~~~\n### sub\n~~~"));
        // CRLF input behaves like LF
        assert!(contains_reserved_heading_shape("Prose.\r\n### sub\r\n"));
        // degenerate / negative shapes
        assert!(!contains_reserved_heading_shape(""));
        assert!(!contains_reserved_heading_shape("###no-space"));
        assert!(!contains_reserved_heading_shape("## EntriesX"));
        assert!(!contains_reserved_heading_shape("#### deep"));

        // prose-level integration: refuse outside, allow inside a fence
        assert!(prose_representation_issue("Prose.\n### Background section").is_some());
        assert!(prose_representation_issue("Prose.\n## Entries").is_some());
        assert!(prose_representation_issue("Prose.\n```\n### inside\n```\n").is_none());
    }

    // ========================================================================
    // Shared fence scanner family (T3)
    // ========================================================================

    /// Every outside-fence line, in order, with 0-based line numbers.
    #[test]
    fn test_for_each_outside_fence_basic() {
        let content = "a\n```md\nfenced\n## fake\n```\nb\n";
        let mut seen = Vec::new();
        for_each_outside_fence(content, |i, line| {
            seen.push((i, line.to_string()));
            true
        });
        // fence open/close lines and fence content are skipped;
        // the trailing newline yields one final empty line (split convention)
        assert_eq!(
            seen,
            vec![
                (0, "a".to_string()),
                (5, "b".to_string()),
                (6, String::new())
            ]
        );
    }

    /// Returning false stops the walk early.
    #[test]
    fn test_for_each_outside_fence_early_stop() {
        let content = "a\nb\nc\n";
        let mut seen = Vec::new();
        for_each_outside_fence(content, |i, _line| {
            seen.push(i);
            i != 1 // stop after line 1
        });
        assert_eq!(seen, vec![0, 1]);
    }

    /// CRLF input: callers normalize first (calling convention), the
    /// scanner then sees clean LF content.
    #[test]
    fn test_scanners_crlf_normalized_input() {
        let raw = "a\r\n```md\r\nfenced\r\n```\r\nb\r\n";
        let content = normalize_line_endings(raw);
        let hits = collect_outside_fence(&content, |_i, line| line == "a" || line == "b");
        assert_eq!(hits, vec![0, 4]);
    }

    /// <= 3 leading spaces: the fence is recognized; 4+ spaces: indented
    /// code block, the line stays outside and fence-shaped.
    #[test]
    fn test_scanners_indent_stance() {
        // 3-space indent: real fence, inner line hidden
        let content = "x\n   ```md\nhidden\n   ```\ny";
        assert_eq!(
            collect_outside_fence(content, |_i, line| line == "x"
                || line == "hidden"
                || line == "y"),
            vec![0, 4]
        );
        // 4-space indent: NOT a fence; every line is outside
        let content = "x\n    ```md\nvisible\n    ```\ny";
        assert_eq!(
            collect_outside_fence(content, |_i, _line| true),
            vec![0, 1, 2, 3, 4]
        );
    }

    /// Tilde fences are not recognized: `~~~` lines are ordinary content
    /// and never open or close a backtick fence.
    #[test]
    fn test_scanners_tilde_not_a_fence() {
        let content = "~~~\nvisible\n~~~";
        assert_eq!(
            collect_outside_fence(content, |_i, _line| true),
            vec![0, 1, 2]
        );
        // a backtick fence opened after ~~~ is still tracked normally
        let content = "~~~\n```\nhidden\n```\nvisible";
        assert_eq!(
            collect_outside_fence(content, |_i, line| line == "visible" || line == "hidden"),
            vec![4]
        );
    }

    /// An unclosed fence swallows everything to end of content.
    #[test]
    fn test_scanners_unclosed_fence_swallows_tail() {
        let content = "a\n```md\nb\nc";
        assert_eq!(collect_outside_fence(content, |_i, _line| true), vec![0]);
        assert_eq!(first_outside_fence(content, |_i, line| line == "b"), None);
    }

    /// Nested-length semantics: a shorter backtick run does not close a
    /// longer fence; an equal-or-longer backtick-only line does.
    #[test]
    fn test_scanners_fence_length_comparison() {
        let content = "````md\n```\nstill-fenced\n`````\noutside";
        assert_eq!(
            collect_outside_fence(content, |_i, line| line == "outside"
                || line == "still-fenced"),
            vec![4]
        );
        // equal length closes
        let content = "````md\nhidden\n````\noutside";
        assert_eq!(
            collect_outside_fence(content, |_i, line| line == "outside"),
            vec![3]
        );
        // closing line with an info string never closes
        let content = "```\nhidden\n```md\nstill\n```";
        assert!(first_outside_fence(content, |_i, line| line == "still").is_none());
    }

    /// Empty content and fence-only content.
    #[test]
    fn test_scanners_degenerate_inputs() {
        // empty: no lines at all
        let mut count = 0usize;
        for_each_outside_fence("", |_i, _line| {
            count += 1;
            true
        });
        assert_eq!(count, 1); // split('\n') yields one empty line
        assert_eq!(first_outside_fence("", |_i, line| !line.is_empty()), None);
        assert!(collect_outside_fence("", |_i, line| !line.is_empty()).is_empty());

        // pure fence content: nothing outside
        let content = "```md\nbody\n```";
        assert!(collect_outside_fence(content, |_i, _line| true).is_empty());

        // unclosed fence only
        let content = "```";
        assert!(collect_outside_fence(content, |_i, _line| true).is_empty());
    }

    /// `first_outside_fence` short-circuits and returns the first match.
    #[test]
    fn test_first_outside_fence() {
        let content = "## Notes\n```\n## fake\n```\n## #1 alice (2026-01-15T10:30:00Z)";
        assert_eq!(
            first_outside_fence(content, |_i, line| line.starts_with("## ")),
            Some(0)
        );
        assert_eq!(
            first_outside_fence(content, |_i, line| line.starts_with("## #")),
            Some(4)
        );
        assert_eq!(
            first_outside_fence(content, |_i, line| line == "## fake"),
            None
        );
    }

    /// Scanner semantics mirror the authoritative `header_indices` loop:
    /// the same content yields the same boundary indices.
    #[test]
    fn test_scanners_match_header_indices_reference() {
        let content = "# t\n\n```markdown\n## #99 mallory (2026-01-01T00:00:00Z)\n```\n## #2 bob (2026-01-01T00:00:01Z)\n";
        let content = normalize_line_endings(content);
        let hits = collect_outside_fence(&content, |_i, line| line.starts_with("## #"));
        assert_eq!(hits, vec![5]);
    }
}
