//! Thread (post) parsing and serialization — Managed File Format v2 (spec §5).
//!
//! Owner rulings D1–D3 (2026-08-09):
//! - preamble: H1 title only (D1); any other preamble content (prose,
//!   attribute-shaped lines such as the historical `- participants:`,
//!   non-matching H2s) is leniently ignored;
//! - message header (fence-aware, flush left): `## #<seq> <sender> (<RFC3339>)`;
//! - no message attribute zone (D2): `- reply-to:` / `- mentions:` / `- to:`
//!   lines no longer exist; reference state is derived from the body text at
//!   read time (`@somebody` → mention, `@#N` → reply) and never persisted;
//! - body: dynamic-length backtick fence (spec §3.4) with info string `md`
//!   on the write side (D3); the parse side leniently accepts any info
//!   string, `md`/`markdown` included; first fence wins.

use regex::Regex;
use std::sync::LazyLock;

use crate::{Message, PaperworkError, Result, ThreadMeta};

use super::{
    collect_outside_fence, compute_fence_length, dedup_preserve_order, fence_close_matches,
    fence_open_len, first_outside_fence, normalize_line_endings, parse_timestamp, RFC3339_FMT,
};

/// Message header regex (spec §5.3, exact).
///
/// Whitespace-lenient (R9): `\s+` between fields, trailing whitespace
/// tolerated; the header MUST be flush left (any leading whitespace degrades
/// it to preamble/body). The sender token excludes whitespace and parentheses.
pub static MESSAGE_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^##\s+#(\d+)\s+([^\s()]+)\s+\((.+)\)\s*$").expect("valid regex"));

// ============================================================================
// Header-regex family — centralized reference point (T3; T4 unified the
// per-layer twins onto this module). The family has THREE variants today;
// their whitespace stances differ ON PURPOSE and must not be unified blindly:
//
// 1. MESSAGE_HEADER_RE (above, authoritative parse grammar, spec §5.3):
//    `\s+` between fields — whitespace-lenient per R9. The ops-side tail
//    scan (spec §5.5) re-checks candidates with `header_seq` directly since
//    T4 deleted its historical `SEQ_RE` prefilter (the prefilter was
//    redundant: `header_seq` is the single authoritative gate, and the
//    tail-scan candidate lines are few by construction).
// 2. LEGACY_HEADER_RE_FMT (below, sole definition; ops/thread.rs imports
//    it): `^###\s+#\d+` — v0.4 legacy-header heuristic; `\s+` kept lenient
//    because it only ever gates a refusal (false positives refuse a write,
//    they never corrupt).
// 3. SUSPECTED_HEADER_RE (cli/cmd/validate.rs): `^##\s+#\d` — warning-only
//    heuristic aligned with MESSAGE_HEADER_RE's `\s+` lenient stance (R9,
//    N2). It lives in the CLI crate and cannot reference `pub(crate)`
//    statics here, so it stays where it is.
// ============================================================================

/// Legacy v0.4 message-header heuristic (`### #N`, flush left) — the single
/// definition of the pattern (T4: `ops/thread.rs` imports it; the ops-side
/// twin was deleted).
///
/// Used by the unmigrated-thread write guard: a non-empty file with no v0.5
/// headers (tail scan seq == 0) that still carries `### #N` lines is legacy
/// data and refuses writes.
pub(crate) static LEGACY_HEADER_RE_FMT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^###\s+#\d+").expect("valid regex"));

/// Mention token scan (spec §5.4 derivation): `@` followed by a run of
/// characters that are neither whitespace, `@`, nor parentheses.
static MENTION_TOKEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@([^\s@()]+)").expect("valid regex"));

/// Reply-reference scan (spec §5.4 derivation): `@#<digits>`.
static REPLY_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@#(\d+)").expect("valid regex"));

/// A mention token shaped `#<pure digits>` is a reply reference, not a
/// mention (spec §5.4).
fn is_reply_shaped_token(token: &str) -> bool {
    let Some(digits) = token.strip_prefix('#') else {
        return false;
    };
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

/// Derive `reply_to` from body text (spec §5.4): the first parseable
/// `@#(\d+)` reference wins; later references are ignored. The referenced
/// target's existence is NOT validated (lenient). Unparseable (overflowing)
/// digit runs are skipped leniently.
pub fn derive_reply_to(body: &str) -> Option<u64> {
    REPLY_REF_RE
        .captures_iter(body)
        .filter_map(|caps| caps[1].parse::<u64>().ok())
        .next()
}

/// Derive `mentions` from body text (spec §5.4): scan `@([^\s@()]+)` in
/// order of appearance, deduplicate keeping first occurrence, drop
/// reply-shaped `#<digits>` tokens, and exclude the sender's self-mentions.
/// Bare `@` without a valid token derives nothing and never errors.
///
/// T4/NEW-10: dedup runs through the shared [`dedup_preserve_order`]
/// (HashSet+Vec, O(n) instead of the historical O(n²) `Vec::contains`).
pub fn derive_mentions(body: &str, sender: &str) -> Vec<String> {
    dedup_preserve_order(
        MENTION_TOKEN_RE
            .captures_iter(body)
            .map(|caps| caps[1].to_string())
            .filter(|token| !is_reply_shaped_token(token) && token != sender),
    )
}

/// Derive both body-text references (spec §5.4, invariant I10).
pub fn derive_message_refs(body: &str, sender: &str) -> (Option<u64>, Vec<String>) {
    (derive_reply_to(body), derive_mentions(body, sender))
}

/// Seq of a message header line, if it is a *valid* header.
///
/// A regex-matched line whose seq does not fit `u64` (overflow) — or whose
/// seq is 0 — is treated as NOT a message header per the lenient semantics
/// (§3.6): the H2 falls into the preamble. This matches the tail-scan skip
/// behavior (§5.5) and never produces seq 0 (review M1).
/// Shared predicate (review MJ-1): every header-recognizing scan — the
/// parse side AND the `thread_edit` preamble carry-over — must agree on
/// what counts as a message header.
pub fn header_seq(line: &str) -> Option<u64> {
    let seq: u64 = MESSAGE_HEADER_RE
        .captures(line)?
        .get(1)?
        .as_str()
        .parse()
        .ok()?;
    (seq != 0).then_some(seq)
}

/// Locate fence-aware message header line indices (spec §3.3/§5.3).
///
/// T4: delegates to the shared scanner family ([`collect_outside_fence`]);
/// the `&[&str]` signature is retained so the differential corpus and the
/// `lines()`-based call sites stay unchanged (the join is fence-neutral:
/// callers pass already-normalized lines).
fn header_indices(lines: &[&str]) -> Vec<usize> {
    let joined = lines.join("\n");
    collect_outside_fence(&joined, |_i, line| header_seq(line).is_some())
}

/// Short-circuit variant of [`header_indices`]: the index of the FIRST
/// fence-aware message header only (M-review M8). `parse_preamble` needs
/// just the first boundary and must not walk the whole file.
fn first_header_index(lines: &[&str]) -> Option<usize> {
    let joined = lines.join("\n");
    first_outside_fence(&joined, |_i, line| header_seq(line).is_some())
}

/// Parse the thread preamble (spec §5.2, owner ruling D1).
///
/// The preamble is everything before the first fence-aware message header.
/// Only the H1 title is mapped (`ThreadMeta { title }`); a missing H1 yields
/// an empty title (lenient). All other preamble content — prose, attribute-
/// shaped lines including the historical `- participants:`, non-matching H2s
/// — is ignored (§3.6).
pub fn parse_preamble(content: &str) -> ThreadMeta {
    let content = normalize_line_endings(content);
    if content.trim().is_empty() {
        return ThreadMeta::default();
    }

    let lines: Vec<&str> = content.lines().collect();
    let preamble_end = first_header_index(&lines).unwrap_or(lines.len());

    let mut meta = ThreadMeta::default();
    for line in &lines[..preamble_end] {
        if let Some(title) = line.strip_prefix("# ") {
            meta.title = title.trim().to_string();
            break;
        }
    }

    meta
}

/// Parse all messages from thread content (fence-aware, spec §5.3/§5.4).
///
/// `reply_to` / `mentions` are derived from each message's body text at
/// parse time (spec §5.4); the `to` field no longer exists (D2).
pub fn parse_messages(content: &str) -> Result<Vec<Message>> {
    let content = normalize_line_endings(content);

    // Empty or whitespace-only content → no messages
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let lines: Vec<&str> = content.lines().collect();
    let headers = header_indices(&lines);

    if headers.is_empty() {
        return Ok(Vec::new());
    }

    let mut messages = Vec::new();

    for (idx, &header_line) in headers.iter().enumerate() {
        let seq = header_seq(lines[header_line]).expect("filtered by header_indices");
        let caps = MESSAGE_HEADER_RE
            .captures(lines[header_line])
            .expect("matched");
        let sender = caps[2].to_string();
        let timestamp_str = caps[3].to_string();

        let timestamp = parse_timestamp(&timestamp_str).map_err(|e| PaperworkError::Parse {
            message: format!(
                "invalid timestamp '{}' in message #{}: {}",
                timestamp_str, seq, e
            ),
            fix: "use RFC 3339 format: YYYY-MM-DDTHH:MM:SSZ".to_string(),
            example: "2026-01-15T10:30:00Z".to_string(),
        })?;

        let content_start = header_line + 1;
        let content_end = if idx + 1 < headers.len() {
            headers[idx + 1]
        } else {
            lines.len()
        };

        let body = parse_message_body(&lines[content_start..content_end]);
        let (reply_to, mentions) = derive_message_refs(&body, &sender);

        messages.push(Message {
            seq,
            sender,
            timestamp,
            reply_to,
            mentions,
            body,
        });
    }

    Ok(messages)
}

/// Extract the body of a message: the first fenced block after the header
/// (spec §5.4, owner ruling D2).
///
/// There is NO attribute zone (D2): anything between the header and the
/// first fence (historical attribute-shaped lines, prose) is ignored
/// leniently. Body normalization (R12): the lines between the opening and
/// closing fence lines, leading/trailing blank lines removed, joined with
/// `\n`. A missing fence yields an empty body (lenient). Only the first
/// fence of a message is the body; later fences are ignored. The fence
/// info string is irrelevant to the parser (D3 lenience: `md`, `markdown`
/// or anything else is accepted).
fn parse_message_body(lines: &[&str]) -> String {
    let mut body_lines: Vec<&str> = Vec::new();

    let mut in_body_fence = false;
    let mut body_found = false;
    let mut body_open_len = 0usize;

    for line in lines {
        if in_body_fence {
            if fence_close_matches(line, body_open_len) {
                in_body_fence = false;
            } else {
                body_lines.push(line);
            }
            continue;
        }

        if let Some(n) = fence_open_len(line) {
            if !body_found {
                // First fence of the message: the body.
                body_found = true;
                in_body_fence = true;
                body_open_len = n;
            }
            // Later fences: ignored (content after the body is discarded).
            continue;
        }
    }

    // Normalize: strip leading/trailing blank lines (R12).
    while body_lines.last().is_some_and(|l| l.trim().is_empty()) {
        body_lines.pop();
    }
    let lead = body_lines
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(body_lines.len());
    let body_lines: Vec<&str> = body_lines.drain(lead..).collect();

    body_lines.join("\n")
}

/// Validate a sender token on the write side (spec §5.6).
///
/// Non-empty, no whitespace (space/tab/newline), no `(` or `)`.
pub fn validate_sender(sender: &str) -> Result<()> {
    let valid = !sender.is_empty()
        && !sender
            .chars()
            .any(|c| c.is_whitespace() || c == '(' || c == ')');
    if valid {
        Ok(())
    } else {
        Err(PaperworkError::Validation {
            message: format!(
                "invalid sender '{}': must be a single token without spaces or parentheses",
                sender
            ),
            fix: "sender must be a single token without spaces or parentheses".to_string(),
            example: "paperwork post send standup --from alice \"Hello\"".to_string(),
        })
    }
}

/// Serialize the thread preamble (spec §5.9, owner ruling D1).
///
/// Title only — no `- participants:` line and no other attributes.
pub fn serialize_preamble(meta: &ThreadMeta) -> String {
    format!("# {}\n\n", meta.title)
}

/// Serialize a single message (spec §5.9, canonical single-space header).
///
/// No attribute lines between the header and the fence (D2); the body fence
/// info string is strictly `md` on the write side (D3). `reply_to` /
/// `mentions` are read-time derivations and are never serialized.
pub fn serialize_message(msg: &Message) -> String {
    let mut out = format!(
        "## #{} {} ({})\n\n",
        msg.seq,
        msg.sender,
        msg.timestamp.format(RFC3339_FMT),
    );

    let fence_len = compute_fence_length(&msg.body);
    let fence = "`".repeat(fence_len);
    out.push_str(&format!("{}md\n", fence));
    if !msg.body.is_empty() {
        out.push_str(&msg.body);
        out.push('\n');
    }
    out.push_str(&format!("{}\n\n", fence));

    out
}

/// Serialize messages only (no preamble; subset output, spec §8 POST-31).
pub fn serialize_messages(messages: &[Message]) -> String {
    messages.iter().map(serialize_message).collect()
}

/// Serialize a complete thread: preamble + messages (spec §5.9).
pub fn serialize_thread(meta: &ThreadMeta, messages: &[Message]) -> String {
    let mut out = serialize_preamble(meta);
    out.push_str(&serialize_messages(messages));
    out
}

/// Validate that message sequence numbers start at 1 and are consecutive.
pub fn validate_seq_monotonicity(messages: &[Message]) -> Result<()> {
    if messages.is_empty() {
        return Ok(());
    }

    // First message should be seq 1
    if messages[0].seq != 1 {
        return Err(PaperworkError::Validation {
            message: format!("first message has seq {}, expected 1", messages[0].seq),
            fix: "thread messages must start at seq 1".to_string(),
            example: String::new(),
        });
    }

    for window in messages.windows(2) {
        let prev = &window[0];
        let curr = &window[1];

        // checked_add guards the theoretical overflow at u64::MAX (N3).
        let expected = prev.seq.checked_add(1);
        if expected != Some(curr.seq) {
            let expected_note = match expected {
                Some(e) => format!(" (expected #{})", e),
                None => String::new(),
            };
            return Err(PaperworkError::Validation {
                message: format!(
                    "sequence gap: message #{} followed by #{}{}",
                    prev.seq, curr.seq, expected_note
                ),
                fix: "message sequence numbers must be consecutive with no gaps".to_string(),
                example: String::new(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};

    // ========================================================================
    // T4 differential corpus: pin the header-boundary scan semantics
    // (fence/indent/tilde/unclosed/nested-length/CRLF/empty) BEFORE the
    // header_indices / first_header_index migration onto the shared
    // scanner family; the same corpus must pass unchanged afterwards.
    // ========================================================================

    const T4_HEADER_A: &str = "## #1 alice (2026-01-15T10:30:00Z)";
    const T4_HEADER_B: &str = "## #2 bob (2026-01-15T10:31:00Z)";

    fn t4_indices_lf(content: &str) -> Vec<usize> {
        let content = normalize_line_endings(content);
        let lines: Vec<&str> = content.lines().collect();
        header_indices(&lines)
    }

    fn t4_first_lf(content: &str) -> Option<usize> {
        let content = normalize_line_endings(content);
        let lines: Vec<&str> = content.lines().collect();
        first_header_index(&lines)
    }

    #[test]
    fn test_t4_header_indices_differential_corpus() {
        // plain: two headers, no trailing newline
        assert_eq!(
            t4_indices_lf(&format!(
                "# t\n\n{}\n\n```md\nx\n```\n\n{}",
                T4_HEADER_A, T4_HEADER_B
            )),
            vec![2, 8]
        );
        // trailing newline yields the same indices
        assert_eq!(t4_indices_lf(&format!("# t\n\n{}\n", T4_HEADER_A)), vec![2]);
        // fence-inside headers are hidden; fence open/close lines too
        assert_eq!(
            t4_indices_lf(&format!("```md\n{}\n```\n{}", T4_HEADER_A, T4_HEADER_B)),
            vec![3]
        );
        // <= 3 space indented fence is recognized
        assert_eq!(
            t4_indices_lf(&format!("   ```md\n{}\n   ```", T4_HEADER_A)),
            vec![]
        );
        // 4-space indent: no fence, the header-shaped line stays visible
        assert_eq!(
            t4_indices_lf(&format!("    ```md\n{}\n    ```", T4_HEADER_A)),
            vec![1]
        );
        // tilde fences are not recognized
        assert_eq!(
            t4_indices_lf(&format!("~~~\n{}\n~~~", T4_HEADER_A)),
            vec![1]
        );
        // unclosed fence swallows the tail
        assert_eq!(
            t4_indices_lf(&format!("{}\n```md\n{}", T4_HEADER_A, T4_HEADER_B)),
            vec![0]
        );
        // nested backtick length: shorter run does not close the fence
        assert_eq!(
            t4_indices_lf(&format!(
                "````md\n{}\n```\nstill\n````\n{}",
                T4_HEADER_A, T4_HEADER_B
            )),
            vec![5]
        );
        // CRLF input behaves like LF
        let crlf = format!(
            "# t\r\n\r\n{}\r\n\r\n```md\r\nx\r\n```\r\n\r\n{}",
            T4_HEADER_A, T4_HEADER_B
        );
        assert_eq!(t4_indices_lf(&crlf), vec![2, 8]);
        // degenerate inputs
        assert_eq!(t4_indices_lf(""), vec![]);
        assert_eq!(t4_indices_lf("# only a title\n"), vec![]);
        // seq-0 / overflowing H2s are NOT headers (parse-side lenience)
        assert_eq!(
            t4_indices_lf("## #0 alice (2026-01-15T10:30:00Z)\n"),
            vec![]
        );
        assert_eq!(
            t4_indices_lf("## #99999999999999999999999999 alice (2026-01-15T10:30:00Z)\n"),
            vec![]
        );
    }

    #[test]
    fn test_t4_first_header_index_differential_corpus() {
        assert_eq!(t4_first_lf(&format!("# t\n\n{}", T4_HEADER_A)), Some(2));
        assert_eq!(
            t4_first_lf(&format!("```md\n{}\n```\n{}", T4_HEADER_A, T4_HEADER_B)),
            Some(3)
        );
        assert_eq!(t4_first_lf(&format!("```md\n{}", T4_HEADER_A)), None); // unclosed
        assert_eq!(t4_first_lf(""), None);
        let crlf = format!("# t\r\n{}", T4_HEADER_A);
        assert_eq!(t4_first_lf(&crlf), Some(1));
    }

    // T3: the centralized legacy-header twin must behave exactly like the
    // ops-side original (identical pattern string, identical matches).
    #[test]
    fn test_legacy_header_re_fmt() {
        assert!(LEGACY_HEADER_RE_FMT.is_match("### #1 alice (2026-01-01T00:00:00Z)"));
        assert!(LEGACY_HEADER_RE_FMT.is_match("###   #42"));
        assert!(!LEGACY_HEADER_RE_FMT.is_match("## #1 alice"));
        assert!(!LEGACY_HEADER_RE_FMT.is_match(" ### #1")); // flush left only
        assert!(!LEGACY_HEADER_RE_FMT.is_match("### no seq"));
    }

    fn make_timestamp(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, s).unwrap()
    }

    /// Build a message whose derived refs are NOT computed — callers must
    /// fill `reply_to` / `mentions` exactly as the body text derives them
    /// (serialization never writes them; parsing re-derives them).
    fn msg(seq: u64, sender: &str, body: &str) -> Message {
        let (reply_to, mentions) = derive_message_refs(body, sender);
        Message {
            seq,
            sender: sender.to_string(),
            timestamp: make_timestamp(2026, 1, 15, 10, 30, 0),
            reply_to,
            mentions,
            body: body.to_string(),
        }
    }

    // T-FT-01 (POST-01, D1/D2/D3)
    #[test]
    fn test_parse_full_thread() {
        let content = "# Daily Standup\n\n## #1 alice (2026-08-01T19:38:22Z)\n\n```md\nParser module is 80% done.\n```\n\n## #2 bob (2026-08-01T19:38:22Z)\n\n```md\n@alice @#1 tests merged, all green.\n```\n";

        let meta = parse_preamble(content);
        assert_eq!(meta.title, "Daily Standup");

        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].seq, 1);
        assert_eq!(messages[0].sender, "alice");
        assert_eq!(messages[0].body, "Parser module is 80% done.");
        // body-text derivation (D2): reply-to + mention from the body
        assert_eq!(messages[1].reply_to, Some(1));
        assert_eq!(messages[1].mentions, vec!["alice"]);
        assert_eq!(messages[1].body, "@alice @#1 tests merged, all green.");
    }

    // D1: a historical `- participants:` preamble line is ignored leniently.
    #[test]
    fn test_preamble_participants_line_ignored() {
        let content = "# Standup\n\n- participants: alice, bob\n\nSome prose.\n";
        let meta = parse_preamble(content);
        assert_eq!(
            meta,
            ThreadMeta {
                title: "Standup".to_string()
            }
        );
    }

    // D2: attribute-shaped lines in the message zone carry no semantics.
    #[test]
    fn test_message_attribute_lines_ignored() {
        let content = "# t\n\n## #2 bob (2026-01-15T10:30:00Z)\n\n- reply-to: #1\n- mentions: alice\n- to: charlie\n\n```md\nplain body\n```\n";
        let messages = parse_messages(content).expect("parse");
        assert_eq!(messages[0].reply_to, None);
        assert!(messages[0].mentions.is_empty());
        assert_eq!(messages[0].body, "plain body");
    }

    // T-FT-02 (POST-02, D2): serialization carries no attribute lines.
    #[test]
    fn test_serialize_no_attribute_lines() {
        let message = msg(1, "alice", "@bob hello");
        let serialized = serialize_message(&message);
        assert!(!serialized.contains("- to:"));
        assert!(!serialized.contains("- reply-to:"));
        assert!(!serialized.contains("- mentions:"));
        // derived refs live only in the parsed model, never on disk
        assert_eq!(message.mentions, vec!["bob"]);
    }

    // §5.4 mentions derivation: order of appearance + dedup.
    #[test]
    fn test_derive_mentions_order_and_dedup() {
        assert_eq!(
            derive_mentions("@bob ping\n@carol @bob again @dave", "alice"),
            vec!["bob", "carol", "dave"]
        );
        // reply-shaped tokens never count as mentions
        assert_eq!(derive_mentions("@#1 @bob", "alice"), vec!["bob"]);
        assert!(derive_mentions("@#7", "alice").is_empty());
    }

    // §5.4 mentions derivation: sender self-mentions are excluded.
    #[test]
    fn test_derive_mentions_self_exclusion() {
        assert!(derive_mentions("@alice doing this myself", "alice").is_empty());
        assert_eq!(derive_mentions("@alice @bob @alice", "alice"), vec!["bob"]);
        // self-reply references are NOT excluded (only mentions are)
        assert_eq!(derive_reply_to("@#3 following up"), Some(3));
    }

    // §5.4 reply-to derivation: first `@#N` wins, the rest are ignored and
    // the target's existence is never validated.
    #[test]
    fn test_derive_reply_to_first_wins() {
        assert_eq!(derive_reply_to("@#2 then @#3 and @#4"), Some(2));
        assert_eq!(derive_reply_to("no refs here"), None);
        // lenient: overflowing digit runs are skipped, the next one wins
        assert_eq!(derive_reply_to("@#99999999999999999999999999 @#5"), Some(5));
    }

    // §5.4 (BDD:POST-33/34): bare / malformed `@` derives nothing, no error.
    #[test]
    fn test_derive_bare_at_tokens() {
        assert!(derive_mentions("trailing @", "alice").is_empty());
        assert!(derive_mentions("@ spaced out", "alice").is_empty());
        assert!(derive_mentions("paren @) close", "alice").is_empty());
        assert!(derive_mentions("", "alice").is_empty());
        assert_eq!(derive_reply_to("just @ and #1"), None);
    }

    // T-FT-04 (POST-04)
    #[test]
    fn test_parse_bad_timestamp() {
        let content = "# t\n\n## #1 alice (not-a-timestamp)\n\n```md\nx\n```\n";
        let result = parse_messages(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let text = err.to_string();
        assert!(text.contains("invalid timestamp"));
        assert!(err.fix().contains("RFC 3339"));
    }

    // T-FT-05 (POST-05)
    #[test]
    fn test_fence_fake_header() {
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nreal body\n## #99 mallory (2026-01-01T00:00:00Z)\nstill body\n```\n\n## #2 bob (2026-01-15T10:31:00Z)\n\n```md\nsecond\n```\n";
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].seq, 1);
        assert!(messages[0]
            .body
            .contains("## #99 mallory (2026-01-01T00:00:00Z)"));
        assert_eq!(messages[1].seq, 2);
        assert!(!messages.iter().any(|m| m.seq == 99));
    }

    // T-FT-06 (POST-06, D3): dynamic fence length; write side emits `md`.
    #[test]
    fn test_dynamic_fence_roundtrip() {
        for k in 3usize..=6 {
            let run = "`".repeat(k);
            let body = format!("code with {} run", run);
            let message = msg(1, "alice", &body);
            let serialized = serialize_message(&message);
            // the fence grows and the info string stays `md`
            let expected_open = format!("{}md\n", "`".repeat(k + 1));
            assert!(
                serialized.contains(&expected_open),
                "k={} should open with {} backticks + md",
                k,
                k + 1
            );
            assert!(!serialized.contains("markdown"));
            let parsed = parse_messages(&serialized).expect("roundtrip parse");
            assert_eq!(parsed[0].body, body);
        }

        // no backticks → exactly 3, info `md`
        let message = msg(1, "alice", "plain");
        let serialized = serialize_message(&message);
        assert!(serialized.contains("```md\n"));
        assert!(!serialized.contains("````"));

        // closing line longer than opening line is accepted (CommonMark)
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nbody text\n`````\n";
        let parsed = parse_messages(content).expect("parse");
        assert_eq!(parsed[0].body, "body text");
    }

    // T-FT-07 (POST-07)
    #[test]
    fn test_sender_not_boundary() {
        let content = "# t\n\n## #1 two words (2026-01-15T10:30:00Z)\n\n## #2 bob(x) (2026-01-15T10:31:00Z)\n";
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages.len(), 0);
        // both H2s fall into the preamble
        let meta = parse_preamble(content);
        assert_eq!(meta.title, "t");
    }

    // T-FT-08 (POST-08)
    #[test]
    fn test_parse_empty() {
        assert_eq!(parse_preamble(""), ThreadMeta::default());
        assert!(parse_messages("").expect("parse").is_empty());
        assert_eq!(parse_preamble("   \n\n  "), ThreadMeta::default());
        assert!(parse_messages("   \n\n  ").expect("parse").is_empty());
    }

    // T-FT-09 (POST-09)
    #[test]
    fn test_parse_preamble_only() {
        let content = "# Standup\n\nSome trailing prose.\n";
        let meta = parse_preamble(content);
        assert_eq!(meta.title, "Standup");
        assert!(parse_messages(content).expect("parse").is_empty());
    }

    // T-FT-10 (POST-12)
    #[test]
    fn test_parse_crlf() {
        let content = "# t\r\n\r\n## #1 alice (2026-01-15T10:30:00Z)\r\n\r\n```md\r\nCRLF body @bob\r\n```\r\n";
        let messages = parse_messages(content).expect("should parse CRLF");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].body, "CRLF body @bob");
        assert!(!messages[0].body.contains('\r'));
        assert_eq!(messages[0].mentions, vec!["bob"]);
        let meta = parse_preamble(content);
        assert_eq!(meta.title, "t");
    }

    // T-FT-11 (POST-13)
    #[test]
    fn test_parse_unicode() {
        let content =
            "# t\n\n## #1 alicé (2026-01-15T10:30:00Z)\n\n```md\nHéllo 🚀 你好世界\n```\n";
        let messages = parse_messages(content).expect("should parse unicode");
        assert_eq!(messages[0].sender, "alicé");
        assert!(messages[0].body.contains("🚀"));
        assert!(messages[0].body.contains("你好世界"));
    }

    // T-FT-12 (POST-14, D1/D2/D3)
    #[test]
    fn test_serialize_thread_roundtrip() {
        let meta = ThreadMeta {
            title: "Daily Standup".to_string(),
        };
        let m1 = msg(1, "alice", "First");
        let m2 = msg(2, "bob", "@#1 @alice Second");
        let m3 = msg(3, "alice", ""); // empty body
        let m4 = msg(4, "bob", "body with ``` triple backticks");

        assert_eq!(m2.reply_to, Some(1));
        assert_eq!(m2.mentions, vec!["alice"]);

        let serialized = serialize_thread(&meta, &[m1.clone(), m2.clone(), m3.clone(), m4.clone()]);
        assert!(serialized.starts_with("# Daily Standup\n"));
        assert!(!serialized.contains("- participants:"));
        assert!(!serialized.contains("---"));
        assert!(!serialized.contains('·'));
        assert!(!serialized.contains('—'));
        assert!(!serialized.contains("- to:"));
        assert!(!serialized.contains("markdown"));

        let parsed_meta = parse_preamble(&serialized);
        assert_eq!(parsed_meta, meta);
        let parsed = parse_messages(&serialized).expect("roundtrip");
        assert_eq!(parsed, vec![m1, m2, m3, m4]);
    }

    // empty body keeps the `md` fence wrapper (spec §5.9)
    #[test]
    fn test_serialize_empty_body() {
        let serialized = serialize_message(&msg(1, "alice", ""));
        assert!(serialized.contains("```md\n```\n"));
        let parsed = parse_messages(&serialized).expect("parse");
        assert_eq!(parsed[0].body, "");
    }

    // T-FT-13 (POST-15)
    #[test]
    fn test_preamble_variants() {
        // variant 1: bare title
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nx\n```\n";
        assert_eq!(parse_preamble(content).title, "t");

        // variant 2: description prose + extra H2 belong to the preamble
        let content = "# t\n\nSome description prose.\n\n## Notes\n\nnote text\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nx\n```\n";
        let meta = parse_preamble(content);
        assert_eq!(meta.title, "t");
        let messages = parse_messages(content).expect("parse");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].seq, 1);

        // variant 3: no H1 (file starts directly with a message header)
        let content = "## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nx\n```\n";
        assert_eq!(parse_preamble(content).title, "");
        assert_eq!(parse_messages(content).expect("parse").len(), 1);
    }

    // T-FT-14 (POST-10, VAL-02)
    #[test]
    fn test_seq_monotonicity() {
        let ok = vec![msg(1, "a", ""), msg(2, "b", ""), msg(3, "a", "")];
        assert!(validate_seq_monotonicity(&ok).is_ok());
        assert!(validate_seq_monotonicity(&[]).is_ok());

        let gap = vec![msg(1, "a", ""), msg(3, "b", "")];
        let err = validate_seq_monotonicity(&gap).unwrap_err();
        assert!(err.to_string().contains("gap"));

        let wrong_start = vec![msg(5, "a", "")];
        let err = validate_seq_monotonicity(&wrong_start).unwrap_err();
        assert!(err.to_string().contains("expected 1"));
    }

    // T-FT-15 (POST-21)
    #[test]
    fn test_preamble_closed_fence_then_header() {
        let content = "# t\n\nExample code:\n\n```\nlet x = 1;\n```\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nx\n```\n";
        let messages = parse_messages(content).expect("parse");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].seq, 1);
    }

    // T-FT-16 (POST-22)
    #[test]
    fn test_preamble_unclosed_fence() {
        let content = "# t\n\n```md\nswallowed\n## #1 alice (2026-01-15T10:30:00Z)\n## #2 bob (2026-01-15T10:31:00Z)\n";
        let messages = parse_messages(content).expect("parse");
        assert_eq!(messages.len(), 0);
    }

    // T-FT-17 (POST-23)
    #[test]
    fn test_body_normalization() {
        let content =
            "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\n\n\nfirst\nsecond\n\n\n```\n";
        let messages = parse_messages(content).expect("parse");
        assert_eq!(messages[0].body, "first\nsecond");
    }

    // T-FT-18 (POST-24)
    #[test]
    fn test_fence_indent_policy() {
        // 3 leading spaces: recognized fence
        let content =
            "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n   ```md\nindented fence body\n   ```\n";
        let messages = parse_messages(content).expect("parse");
        assert_eq!(messages[0].body, "indented fence body");

        // 4 leading spaces: NOT a fence (indented code block); body stays
        // empty (lenient), following headers still split
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n    ```md\nnot a fence\n\n## #2 bob (2026-01-15T10:31:00Z)\n\n```md\nsecond\n```\n";
        let messages = parse_messages(content).expect("parse");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].body, "");
        assert_eq!(messages[1].body, "second");
    }

    // T-FT-19 (POST-25, D3): parse side is lenient about the fence info.
    #[test]
    fn test_body_fence_info_lenient() {
        // no info string
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```\nplain fence\n```\n";
        assert_eq!(
            parse_messages(content).expect("parse")[0].body,
            "plain fence"
        );

        // `md` (canonical write form)
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nmd fence\n```\n";
        assert_eq!(parse_messages(content).expect("parse")[0].body, "md fence");

        // `markdown` (legacy/handwritten form, D3 lenience)
        let content =
            "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```markdown\nmarkdown fence\n```\n";
        assert_eq!(
            parse_messages(content).expect("parse")[0].body,
            "markdown fence"
        );

        // arbitrary info string
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```rust\nfn main() {}\n```\n";
        assert_eq!(
            parse_messages(content).expect("parse")[0].body,
            "fn main() {}"
        );

        // writer side always emits `md`
        let serialized = serialize_message(&msg(1, "alice", "b"));
        assert!(serialized.contains("```md\n"));
        assert!(!serialized.contains("markdown"));
    }

    // T-FT-20 (POST-26): first fence wins; missing refs derive None.
    #[test]
    fn test_multi_fence_first_wins() {
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nfirst fence\n```\n\n```md\nsecond fence\n```\n";
        let messages = parse_messages(content).expect("parse");
        assert_eq!(messages[0].body, "first fence");
        assert_eq!(messages[0].reply_to, None);
        assert!(messages[0].mentions.is_empty());
    }

    // T-FT-21 (POST-28)
    #[test]
    fn test_header_trailing_garbage() {
        let content = "# t\n\n## #1 alice (2026-01-15T10:30:00Z) (备注)\n\n```md\nx\n```\n";
        let result = parse_messages(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid timestamp"));
        assert_eq!(err.category(), "format");
    }

    // T-FT-22 (POST-01 supplement, R9)
    #[test]
    fn test_header_whitespace_lenient() {
        // extra spaces between fields + trailing whitespace
        let content = "# t\n\n##  #1   alice   (2026-01-15T10:30:00Z)  \n\n```md\nx\n```\n";
        let messages = parse_messages(content).expect("lenient parse");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].seq, 1);
        assert_eq!(messages[0].sender, "alice");

        // serialization stays canonical single-space
        let serialized = serialize_message(&messages[0]);
        assert!(serialized.starts_with("## #1 alice ("));
    }

    // sender validation (spec §5.6, POST-17 write side)
    #[test]
    fn test_validate_sender() {
        assert!(validate_sender("alice").is_ok());
        assert!(validate_sender("alicé").is_ok());
        for bad in ["two words", "bob(x)", "line\nbreak", "", "tab\there"] {
            let result = validate_sender(bad);
            assert!(result.is_err(), "sender {:?} must be rejected", bad);
            assert_eq!(result.unwrap_err().category(), "validation");
        }
    }

    // M1: seq overflow → the H2 is leniently treated as preamble content;
    // seq 0 is never produced.
    #[test]
    fn test_seq_overflow_header_is_preamble() {
        let content = "# t\n\n## #99999999999999999999999999 alice (2026-01-15T10:30:00Z)\n\n```md\nx\n```\n\n## #1 bob (2026-01-15T10:31:00Z)\n\n```md\ny\n```\n";
        let messages = parse_messages(content).expect("lenient parse");
        assert!(!messages.iter().any(|m| m.seq == 0), "seq 0 forbidden");
        // the overflow header fell into the preamble; #1 bob stays a message
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].sender, "bob");
        assert_eq!(messages[0].body, "y");

        // overflow-only file: zero messages, everything is preamble
        let content = "# t\n\n## #99999999999999999999999999 alice (2026-01-15T10:30:00Z)\n";
        let messages = parse_messages(content).expect("lenient parse");
        assert!(messages.is_empty());
        assert!(!messages.iter().any(|m| m.seq == 0));

        // seq 0 headers are equally non-headers
        let content = "# t\n\n## #0 alice (2026-01-15T10:30:00Z)\n";
        assert!(parse_messages(content).expect("parse").is_empty());
    }

    // N3: validate_seq_monotonicity uses checked_add (no overflow panic)
    #[test]
    fn test_seq_monotonicity_overflow_safe() {
        // huge seq values never panic in the window comparison
        let huge_gap = vec![msg(1, "a", ""), msg(u64::MAX, "b", "")];
        let err = validate_seq_monotonicity(&huge_gap).unwrap_err();
        assert!(err.to_string().contains("gap"));

        let at_max = vec![msg(u64::MAX, "a", ""), msg(1, "b", "")];
        let err = validate_seq_monotonicity(&at_max).unwrap_err();
        assert!(err.to_string().contains("expected 1"));
    }
}
