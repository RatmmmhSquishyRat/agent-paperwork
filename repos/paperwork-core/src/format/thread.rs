//! Thread message parsing and serialization.
//!
//! Format spec (§2.3):
//! ```markdown
//! ---
//!
//! ### #<seq> — <sender> · <ISO-8601>
//!
//! **To**: <recipient>
//! **Reply-To**: #<seq> | —
//!
//! <body: free-form Markdown, multi-line>
//!
//! ---
//! ```
//!
//! CRITICAL (invariant I12): Message boundary = `---` line immediately followed
//! (within 2 lines) by a valid H3 header. A lone `---` NOT followed by this
//! pattern is BODY CONTENT, not a boundary.

use chrono::{DateTime, Utc};

use crate::{Message, PaperworkError, Result};

use super::{extract_bold_key, find_message_boundaries, normalize_line_endings, parse_message_header};

/// Parse all messages from thread content.
///
/// Uses boundary-anchored parsing: only `---` + valid H3 header pairs
/// trigger message splits. Lone `---` in body is content.
pub fn parse_messages(content: &str) -> Result<Vec<Message>> {
    let content = normalize_line_endings(content);

    // Empty or whitespace-only content → no messages
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let lines: Vec<&str> = content.lines().collect();
    let boundaries = find_message_boundaries(&lines);

    if boundaries.is_empty() {
        return Ok(Vec::new());
    }

    let mut messages = Vec::new();

    for (idx, &(_boundary_line, header_line)) in boundaries.iter().enumerate() {
        // Parse the header
        let (seq, sender, timestamp_str) = parse_message_header(lines[header_line]).ok_or_else(
            || {
                PaperworkError::Parse(format!(
                    "invalid message header at line {}: '{}'",
                    header_line + 1,
                    lines[header_line]
                ))
            },
        )?;

        // Parse timestamp
        let timestamp = parse_timestamp(&timestamp_str).map_err(|e| {
            PaperworkError::Parse(format!(
                "invalid timestamp '{}' in message #{}: {}",
                timestamp_str, seq, e
            ))
        })?;

        // Determine the content range for this message
        let content_start = header_line + 1;
        let content_end = if idx + 1 < boundaries.len() {
            boundaries[idx + 1].0 // Next boundary line
        } else {
            lines.len()
        };

        // Extract metadata and body from content range
        let (to, reply_to, mentions, body) =
            parse_message_content(&lines[content_start..content_end], seq)?;

        messages.push(Message {
            seq,
            sender,
            timestamp,
            to,
            reply_to,
            mentions,
            body,
        });
    }

    Ok(messages)
}

/// Parse timestamp from ISO-8601 string.
fn parse_timestamp(s: &str) -> std::result::Result<DateTime<Utc>, String> {
    // Try parsing with Z suffix
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    // Try parsing without timezone (assume UTC)
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
    }
    Err(format!("cannot parse '{}' as ISO-8601 timestamp", s))
}

/// Parse message content (metadata lines + body) from lines after header.
#[allow(clippy::type_complexity)]
fn parse_message_content(
    lines: &[&str],
    seq: u64,
) -> Result<(Vec<String>, Option<u64>, Vec<String>, String)> {
    let mut to: Vec<String> = Vec::new();
    let mut reply_to: Option<u64> = None;
    let mut mentions: Vec<String> = Vec::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut in_body = false;
    let mut metadata_done = false;

    for line in lines {
        if !in_body {
            let trimmed = line.trim();

            // Skip empty lines before metadata
            if trimmed.is_empty() && !metadata_done {
                continue;
            }

            // Try to extract bold key metadata
            if let Some((key, value)) = extract_bold_key(trimmed) {
                metadata_done = true;
                match key.as_str() {
                    "To" => {
                        to = parse_to_field(&value);
                    }
                    "Reply-To" => {
                        reply_to = parse_reply_to(&value);
                    }
                    "Mentions" => {
                        mentions = parse_to_field(&value);
                    }
                    _ => {
                        // Unknown metadata key - treat as body start
                        in_body = true;
                        body_lines.push(line);
                    }
                }
                continue;
            }

            // First non-metadata, non-empty line starts body
            if metadata_done || !trimmed.is_empty() {
                in_body = true;
                body_lines.push(line);
            }
        } else {
            body_lines.push(line);
        }
    }

    // Trim trailing empty lines from body
    while body_lines.last().is_some_and(|l| l.trim().is_empty()) {
        body_lines.pop();
    }

    // Trim leading empty lines from body
    while body_lines.first().is_some_and(|l| l.trim().is_empty()) {
        body_lines.remove(0);
    }

    let body = body_lines.join("\n");

    // Validate: To field should be present (warning level, not error for flexibility)
    let _ = seq; // Used for error context if needed

    Ok((to, reply_to, mentions, body))
}

/// Parse the To field value.
/// "all" → empty Vec (broadcast)
/// "alice" → vec!["alice"]
/// "alice, bob" → vec!["alice", "bob"]
fn parse_to_field(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed == "all" || trimmed.is_empty() {
        return Vec::new();
    }
    trimmed
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse the Reply-To field value.
/// "#5" → Some(5)
/// "—" → None
fn parse_reply_to(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed == "—" || trimmed.is_empty() {
        return None;
    }
    if let Some(seq_str) = trimmed.strip_prefix('#') {
        seq_str.parse().ok()
    } else {
        trimmed.parse().ok()
    }
}

/// Serialize a single message to Markdown.
pub fn serialize_message(msg: &Message) -> String {
    let to_str = if msg.to.is_empty() {
        "all".to_string()
    } else {
        msg.to.join(", ")
    };

    let reply_to_str = msg
        .reply_to
        .map(|r| format!("#{}", r))
        .unwrap_or_else(|| "—".to_string());

    let mentions_line = if msg.mentions.is_empty() {
        String::new()
    } else {
        format!("**Mentions**: {}  \n", msg.mentions.join(", "))
    };

    format!(
        "---\n\n### #{} — {} · {}\n\n**To**: {}  \n**Reply-To**: {}\n{}\n{}\n\n",
        msg.seq,
        msg.sender,
        msg.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
        to_str,
        reply_to_str,
        mentions_line,
        msg.body
    )
}

/// Serialize multiple messages to a complete thread.
pub fn serialize_thread(messages: &[Message]) -> String {
    messages.iter().map(serialize_message).collect()
}

/// Validate that message sequence numbers are monotonically increasing with no gaps.
/// Returns Ok if valid, Err with description of the problem.
pub fn validate_seq_monotonicity(messages: &[Message]) -> Result<()> {
    if messages.is_empty() {
        return Ok(());
    }

    // First message should be seq 1
    if messages[0].seq != 1 {
        return Err(PaperworkError::Validation(format!(
            "first message has seq {}, expected 1",
            messages[0].seq
        )));
    }

    for window in messages.windows(2) {
        let prev = &window[0];
        let curr = &window[1];

        if curr.seq != prev.seq + 1 {
            return Err(PaperworkError::Validation(format!(
                "sequence gap: message #{} followed by #{} (expected #{})",
                prev.seq,
                curr.seq,
                prev.seq + 1
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_timestamp(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, s).unwrap()
    }

    #[test]
    fn test_parse_single_message() {
        let content = r#"---

### #1 — alice · 2026-01-15T10:30:00Z

**To**: bob  
**Reply-To**: —

Hello, Bob!
"#;
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].seq, 1);
        assert_eq!(messages[0].sender, "alice");
        assert_eq!(messages[0].to, vec!["bob"]);
        assert_eq!(messages[0].reply_to, None);
        assert_eq!(messages[0].body, "Hello, Bob!");
    }

    #[test]
    fn test_parse_multi_message() {
        let content = r#"---

### #1 — alice · 2026-01-15T10:30:00Z

**To**: bob  
**Reply-To**: —

First message

---

### #2 — bob · 2026-01-15T10:35:00Z

**To**: alice  
**Reply-To**: #1

Second message

---

### #3 — alice · 2026-01-15T10:40:00Z

**To**: bob  
**Reply-To**: #2

Third message
"#;
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].seq, 1);
        assert_eq!(messages[1].seq, 2);
        assert_eq!(messages[2].seq, 3);
        assert_eq!(messages[0].body, "First message");
        assert_eq!(messages[1].body, "Second message");
        assert_eq!(messages[2].body, "Third message");
    }

    #[test]
    fn test_parse_message_with_reply() {
        let content = r#"---

### #2 — bob · 2026-01-15T10:35:00Z

**To**: alice  
**Reply-To**: #1

Replying to your message
"#;
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages[0].reply_to, Some(1));
    }

    #[test]
    fn test_parse_message_no_reply() {
        let content = r#"---

### #1 — alice · 2026-01-15T10:30:00Z

**To**: bob  
**Reply-To**: —

No reply
"#;
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages[0].reply_to, None);
    }

    #[test]
    fn test_parse_message_multiline_body() {
        let content = r#"---

### #1 — alice · 2026-01-15T10:30:00Z

**To**: bob  
**Reply-To**: —

Line 1
Line 2
Line 3
Line 4
Line 5
"#;
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages[0].body, "Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
    }

    #[test]
    fn test_parse_message_body_with_hr() {
        // CRITICAL TEST: Body containing --- should NOT split the message
        let content = r#"---

### #1 — alice · 2026-01-15T10:30:00Z

**To**: bob  
**Reply-To**: —

Here is some text

---

This is still part of the body!

---

### #2 — bob · 2026-01-15T10:35:00Z

**To**: alice  
**Reply-To**: #1

Got it
"#;
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages.len(), 2);
        assert!(messages[0].body.contains("---"));
        assert!(messages[0].body.contains("This is still part of the body!"));
        assert_eq!(messages[1].body, "Got it");
    }

    #[test]
    fn test_parse_message_body_with_h3() {
        // Body containing H3 that doesn't match message pattern
        let content = r#"---

### #1 — alice · 2026-01-15T10:30:00Z

**To**: bob  
**Reply-To**: —

### This is a body heading

Some text under it

---

### #2 — bob · 2026-01-15T10:35:00Z

**To**: alice  
**Reply-To**: —

Ok
"#;
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages.len(), 2);
        assert!(messages[0].body.contains("### This is a body heading"));
    }

    #[test]
    fn test_serialize_message_roundtrip() {
        let msg = Message {
            seq: 1,
            sender: "alice".to_string(),
            timestamp: make_timestamp(2026, 1, 15, 10, 30, 0),
            to: vec!["bob".to_string()],
            reply_to: None,
            mentions: vec![],
            body: "Hello, World!".to_string(),
        };

        let serialized = serialize_message(&msg);
        let parsed = parse_messages(&serialized).expect("should parse serialized");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], msg);
    }

    #[test]
    fn test_serialize_message_with_reply_roundtrip() {
        let msg = Message {
            seq: 5,
            sender: "bob".to_string(),
            timestamp: make_timestamp(2026, 7, 29, 23, 59, 59),
            to: vec!["alice".to_string()],
            reply_to: Some(3),
            mentions: vec![],
            body: "Reply body".to_string(),
        };

        let serialized = serialize_message(&msg);
        let parsed = parse_messages(&serialized).expect("should parse serialized");
        assert_eq!(parsed[0], msg);
    }

    #[test]
    fn test_serialize_message_broadcast() {
        let msg = Message {
            seq: 1,
            sender: "alice".to_string(),
            timestamp: make_timestamp(2026, 1, 15, 10, 30, 0),
            to: vec![], // empty = "all"
            reply_to: None,
            mentions: vec![],
            body: "Broadcast message".to_string(),
        };

        let serialized = serialize_message(&msg);
        assert!(serialized.contains("**To**: all"));

        let parsed = parse_messages(&serialized).expect("should parse");
        assert_eq!(parsed[0].to, Vec::<String>::new());
    }

    #[test]
    fn test_parse_empty_thread() {
        let messages = parse_messages("").expect("should parse empty");
        assert!(messages.is_empty());

        let messages = parse_messages("   \n\n  ").expect("should parse whitespace");
        assert!(messages.is_empty());
    }

    #[test]
    fn test_seq_monotonicity_valid() {
        let messages = vec![
            Message {
                seq: 1,
                sender: "a".to_string(),
                timestamp: make_timestamp(2026, 1, 1, 0, 0, 0),
                to: vec![],
                reply_to: None,
                mentions: vec![],
                body: String::new(),
            },
            Message {
                seq: 2,
                sender: "b".to_string(),
                timestamp: make_timestamp(2026, 1, 1, 0, 1, 0),
                to: vec![],
                reply_to: None,
                mentions: vec![],
                body: String::new(),
            },
            Message {
                seq: 3,
                sender: "a".to_string(),
                timestamp: make_timestamp(2026, 1, 1, 0, 2, 0),
                to: vec![],
                reply_to: None,
                mentions: vec![],
                body: String::new(),
            },
        ];

        assert!(validate_seq_monotonicity(&messages).is_ok());
    }

    #[test]
    fn test_seq_gap_detection() {
        let messages = vec![
            Message {
                seq: 1,
                sender: "a".to_string(),
                timestamp: make_timestamp(2026, 1, 1, 0, 0, 0),
                to: vec![],
                reply_to: None,
                mentions: vec![],
                body: String::new(),
            },
            Message {
                seq: 3, // Gap! Missing 2
                sender: "b".to_string(),
                timestamp: make_timestamp(2026, 1, 1, 0, 1, 0),
                to: vec![],
                reply_to: None,
                mentions: vec![],
                body: String::new(),
            },
        ];

        let result = validate_seq_monotonicity(&messages);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("gap"));
    }

    #[test]
    fn test_seq_wrong_start() {
        let messages = vec![Message {
            seq: 5, // Should start at 1
            sender: "a".to_string(),
            timestamp: make_timestamp(2026, 1, 1, 0, 0, 0),
            to: vec![],
            reply_to: None,
            mentions: vec![],
            body: String::new(),
        }];

        let result = validate_seq_monotonicity(&messages);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("expected 1"));
    }

    #[test]
    fn test_parse_crlf_thread() {
        let content = "---\r\n\r\n### #1 — alice · 2026-01-15T10:30:00Z\r\n\r\n**To**: bob  \r\n**Reply-To**: —\r\n\r\nCRLF body\r\n";
        let messages = parse_messages(content).expect("should parse CRLF");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].body, "CRLF body");
    }

    #[test]
    fn test_parse_unicode_message() {
        let content = r#"---

### #1 — alicé · 2026-01-15T10:30:00Z

**To**: böb  
**Reply-To**: —

Héllo Wörld! 🚀
Unicode: 你好世界
"#;
        let messages = parse_messages(content).expect("should parse unicode");
        assert_eq!(messages[0].sender, "alicé");
        assert_eq!(messages[0].to, vec!["böb"]);
        assert!(messages[0].body.contains("🚀"));
        assert!(messages[0].body.contains("你好世界"));
    }

    #[test]
    fn test_parse_multi_recipient() {
        let content = r#"---

### #1 — alice · 2026-01-15T10:30:00Z

**To**: bob, charlie, dave  
**Reply-To**: —

Multi-recipient message
"#;
        let messages = parse_messages(content).expect("should parse");
        assert_eq!(messages[0].to, vec!["bob", "charlie", "dave"]);
    }

    #[test]
    fn test_serialize_thread_roundtrip() {
        let messages = vec![
            Message {
                seq: 1,
                sender: "alice".to_string(),
                timestamp: make_timestamp(2026, 1, 15, 10, 30, 0),
                to: vec!["bob".to_string()],
                reply_to: None,
                mentions: vec![],
                body: "First".to_string(),
            },
            Message {
                seq: 2,
                sender: "bob".to_string(),
                timestamp: make_timestamp(2026, 1, 15, 10, 35, 0),
                to: vec!["alice".to_string()],
                reply_to: Some(1),
                mentions: vec![],
                body: "Second".to_string(),
            },
        ];

        let serialized = serialize_thread(&messages);
        let parsed = parse_messages(&serialized).expect("should parse serialized thread");
        assert_eq!(messages, parsed);
    }

    #[test]
    fn test_body_with_bold_text() {
        let content = r#"---

### #1 — alice · 2026-01-15T10:30:00Z

**To**: bob  
**Reply-To**: —

This has **bold text** in the body.
And **another bold** line.
"#;
        let messages = parse_messages(content).expect("should parse");
        assert!(messages[0].body.contains("**bold text**"));
        assert!(messages[0].body.contains("**another bold**"));
    }

    #[test]
    fn test_empty_body() {
        let msg = Message {
            seq: 1,
            sender: "alice".to_string(),
            timestamp: make_timestamp(2026, 1, 15, 10, 30, 0),
            to: vec!["bob".to_string()],
            reply_to: None,
            mentions: vec![],
            body: String::new(),
        };

        let serialized = serialize_message(&msg);
        let parsed = parse_messages(&serialized).expect("should parse");
        assert_eq!(parsed[0].body, "");
    }
}
