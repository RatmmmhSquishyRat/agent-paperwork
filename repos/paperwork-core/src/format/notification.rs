//! Notification parsing and serialization.
//!
//! Format spec (§2.8):
//! ```markdown
//! # Notifications: <name>
//!
//! ---
//!
//! ### <ISO-8601> — from <sender>
//!
//! **In**: <thread-path>
//! **Seq**: #<seq>
//! **Type**: mention | reply
//!
//! > <snippet of the triggering message>
//!
//! ---
//! ```

use chrono::{DateTime, Utc};
use regex::Regex;
use std::sync::LazyLock;

use crate::{Notification, NotifyType, PaperworkError, Result};

use super::{extract_bold_key, normalize_line_endings};

/// Regex for notification H3 header: `### <ISO-8601> — from <sender>`
static NOTIFICATION_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^### (.+) — from (.+)$").expect("valid regex"));

/// Parse notifications from Markdown content.
pub fn parse_notifications(content: &str) -> Result<Vec<Notification>> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();

    let mut notifications = Vec::new();
    let mut current: Option<NotificationBuilder> = None;

    for line in &lines {
        let trimmed = line.trim();

        // Skip H1 header
        if trimmed.starts_with("# Notifications:") {
            continue;
        }

        // Boundary line
        if trimmed == "---" {
            if let Some(builder) = current.take() {
                notifications.push(builder.build()?);
            }
            continue;
        }

        // Notification header: ### <timestamp> — from <sender>
        if let Some(caps) = NOTIFICATION_HEADER_RE.captures(trimmed) {
            // Save previous notification if exists
            if let Some(builder) = current.take() {
                notifications.push(builder.build()?);
            }

            let timestamp_str = &caps[1];
            let from = caps[2].to_string();

            let timestamp = parse_timestamp(timestamp_str).map_err(|e| {
                PaperworkError::Parse(format!(
                    "invalid timestamp '{}' in notification from '{}': {}",
                    timestamp_str, from, e
                ))
            })?;

            current = Some(NotificationBuilder::new(timestamp, from));
            continue;
        }

        // Bold key extraction
        if let Some((key, value)) = extract_bold_key(trimmed) {
            if let Some(ref mut builder) = current {
                match key.as_str() {
                    "In" => builder.thread_path = Some(value),
                    "Seq" => {
                        builder.seq = parse_seq(&value);
                    }
                    "Type" => {
                        builder.notify_type = parse_notify_type(&value);
                    }
                    _ => {}
                }
            }
            continue;
        }

        // Blockquote (snippet)
        if trimmed.starts_with("> ") || trimmed == ">" {
            if let Some(ref mut builder) = current {
                let snippet_text = trimmed.strip_prefix("> ").unwrap_or("").to_string();
                if builder.snippet.is_empty() {
                    builder.snippet = snippet_text;
                } else {
                    builder.snippet.push('\n');
                    builder.snippet.push_str(&snippet_text);
                }
            }
        }
    }

    // Don't forget the last notification
    if let Some(builder) = current.take() {
        notifications.push(builder.build()?);
    }

    Ok(notifications)
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

/// Parse sequence number from "#5" or "5".
fn parse_seq(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if let Some(seq_str) = trimmed.strip_prefix('#') {
        seq_str.parse().ok()
    } else {
        trimmed.parse().ok()
    }
}

/// Parse notification type from string.
fn parse_notify_type(value: &str) -> NotifyType {
    match value.trim().to_lowercase().as_str() {
        "reply" => NotifyType::Reply,
        _ => NotifyType::Mention,
    }
}

/// Builder for constructing Notification during parsing.
struct NotificationBuilder {
    timestamp: DateTime<Utc>,
    from: String,
    thread_path: Option<String>,
    seq: Option<u64>,
    notify_type: NotifyType,
    snippet: String,
}

impl NotificationBuilder {
    fn new(timestamp: DateTime<Utc>, from: String) -> Self {
        Self {
            timestamp,
            from,
            thread_path: None,
            seq: None,
            notify_type: NotifyType::Mention,
            snippet: String::new(),
        }
    }

    fn build(self) -> Result<Notification> {
        let thread_path = self.thread_path.ok_or_else(|| {
            PaperworkError::Parse(format!(
                "missing **In**: line for notification from '{}'",
                self.from
            ))
        })?;

        let seq = self.seq.ok_or_else(|| {
            PaperworkError::Parse(format!(
                "missing **Seq**: line for notification from '{}'",
                self.from
            ))
        })?;

        Ok(Notification {
            timestamp: self.timestamp,
            from: self.from,
            thread_path,
            seq,
            notify_type: self.notify_type,
            snippet: self.snippet,
        })
    }
}

/// Serialize notifications to Markdown content.
pub fn serialize_notifications(name: &str, notifications: &[Notification]) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Notifications: {}\n", name));

    for notif in notifications {
        out.push_str(&serialize_notification(notif));
    }

    out
}

/// Serialize a single notification block.
fn serialize_notification(notif: &Notification) -> String {
    let type_str = match notif.notify_type {
        NotifyType::Mention => "mention",
        NotifyType::Reply => "reply",
    };

    let mut out = String::new();

    out.push_str("\n---\n\n");
    out.push_str(&format!(
        "### {} — from {}\n\n",
        notif.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
        notif.from
    ));
    out.push_str(&format!("**In**: {}  \n", notif.thread_path));
    out.push_str(&format!("**Seq**: #{}  \n", notif.seq));
    out.push_str(&format!("**Type**: {}\n", type_str));

    if !notif.snippet.is_empty() {
        out.push('\n');
        for line in notif.snippet.lines() {
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
    fn test_parse_notification_entry() {
        let content = r#"# Notifications: bob

---

### 2026-01-15T10:30:00Z — from alice

**In**: dm/alice--bob/thread.md  
**Seq**: #5  
**Type**: mention

> Hey @bob, check this out!

---
"#;
        let notifications = parse_notifications(content).expect("should parse");
        assert_eq!(notifications.len(), 1);

        let notif = &notifications[0];
        assert_eq!(notif.from, "alice");
        assert_eq!(notif.thread_path, "dm/alice--bob/thread.md");
        assert_eq!(notif.seq, 5);
        assert_eq!(notif.notify_type, NotifyType::Mention);
        assert_eq!(notif.snippet, "Hey @bob, check this out!");
    }

    #[test]
    fn test_parse_notification_reply_type() {
        let content = r#"# Notifications: alice

---

### 2026-01-15T11:00:00Z — from bob

**In**: dm/alice--bob/thread.md  
**Seq**: #6  
**Type**: reply

> Thanks for the info!

---
"#;
        let notifications = parse_notifications(content).expect("should parse");
        assert_eq!(notifications[0].notify_type, NotifyType::Reply);
    }

    #[test]
    fn test_parse_empty_notifications() {
        let content = r#"# Notifications: bob
"#;
        let notifications = parse_notifications(content).expect("should parse empty");
        assert!(notifications.is_empty());
    }

    #[test]
    fn test_parse_multiple_notifications() {
        let content = r#"# Notifications: charlie

---

### 2026-01-15T10:00:00Z — from alice

**In**: posts/general/log.md  
**Seq**: #1  
**Type**: mention

> First notification

---

### 2026-01-15T11:00:00Z — from bob

**In**: dm/bob--charlie/thread.md  
**Seq**: #3  
**Type**: reply

> Second notification

---
"#;
        let notifications = parse_notifications(content).expect("should parse");
        assert_eq!(notifications.len(), 2);
        assert_eq!(notifications[0].from, "alice");
        assert_eq!(notifications[1].from, "bob");
    }

    #[test]
    fn test_serialize_notification_roundtrip() {
        let notifications = vec![
            Notification {
                timestamp: make_timestamp(2026, 1, 15, 10, 30, 0),
                from: "alice".to_string(),
                thread_path: "dm/alice--bob/thread.md".to_string(),
                seq: 5,
                notify_type: NotifyType::Mention,
                snippet: "Test snippet".to_string(),
            },
            Notification {
                timestamp: make_timestamp(2026, 1, 15, 11, 0, 0),
                from: "charlie".to_string(),
                thread_path: "posts/general/log.md".to_string(),
                seq: 10,
                notify_type: NotifyType::Reply,
                snippet: "Another snippet".to_string(),
            },
        ];

        let serialized = serialize_notifications("bob", &notifications);
        let parsed = parse_notifications(&serialized).expect("should parse serialized");

        assert_eq!(notifications.len(), parsed.len());
        for (orig, parsed) in notifications.iter().zip(parsed.iter()) {
            assert_eq!(orig.from, parsed.from);
            assert_eq!(orig.thread_path, parsed.thread_path);
            assert_eq!(orig.seq, parsed.seq);
            assert_eq!(orig.notify_type, parsed.notify_type);
            assert_eq!(orig.snippet, parsed.snippet);
        }
    }

    #[test]
    fn test_parse_notification_crlf() {
        let content = "# Notifications: bob\r\n\r\n---\r\n\r\n### 2026-01-15T10:30:00Z — from alice\r\n\r\n**In**: dm/alice--bob/thread.md  \r\n**Seq**: #1  \r\n**Type**: mention\r\n\r\n> CRLF snippet\r\n\r\n---\r\n";
        let notifications = parse_notifications(content).expect("should parse CRLF");
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].snippet, "CRLF snippet");
    }

    #[test]
    fn test_parse_notification_unicode() {
        let content = r#"# Notifications: böb

---

### 2026-01-15T10:30:00Z — from alicé

**In**: dm/alicé--böb/thread.md  
**Seq**: #1  
**Type**: mention

> Héllo Wörld! 🚀

---
"#;
        let notifications = parse_notifications(content).expect("should parse unicode");
        assert_eq!(notifications[0].from, "alicé");
        assert!(notifications[0].snippet.contains('🚀'));
    }

    #[test]
    fn test_parse_notification_multiline_snippet() {
        let content = r#"# Notifications: bob

---

### 2026-01-15T10:30:00Z — from alice

**In**: dm/alice--bob/thread.md  
**Seq**: #1  
**Type**: mention

> Line one of snippet.
> Line two of snippet.

---
"#;
        let notifications = parse_notifications(content).expect("should parse");
        let snippet = &notifications[0].snippet;
        assert!(snippet.contains("Line one"));
        assert!(snippet.contains("Line two"));
    }

    #[test]
    fn test_parse_notification_missing_in() {
        let content = r#"# Notifications: bob

---

### 2026-01-15T10:30:00Z — from alice

**Seq**: #1  
**Type**: mention

> Snippet

---
"#;
        let result = parse_notifications(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing **In**:"));
    }
}
