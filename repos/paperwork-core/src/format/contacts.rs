//! Contacts table parsing and serialization.
//!
//! A contacts file is a simple bullet list of profile paths.
//!
//! Format:
//! ```markdown
//! # <title>
//!
//! - ./agents/alice.profile.md
//! - ./agents/bob.profile.md
//! ```

use crate::{ContactEntry, PaperworkError, Result};

use super::normalize_line_endings;

/// Extract the title from contacts content (the H1 heading).
pub fn parse_contacts_title(content: &str) -> Result<String> {
    let content = normalize_line_endings(content);
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            if !trimmed.starts_with("## ") {
                return Ok(stripped.to_string());
            }
        }
    }
    Err(PaperworkError::Parse {
        message: "missing contacts title heading (# <title>)".to_string(),
        fix: "add a top-level heading with the contacts title".to_string(),
        example: "# my-team".to_string(),
    })
}

/// Parse contacts from Markdown bullet list content.
pub fn parse_contacts(content: &str) -> Result<Vec<ContactEntry>> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();

    let mut entries = Vec::new();

    for line in &lines {
        let trimmed = line.trim();

        // Skip empty lines and headings
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Bullet items: `- <path>`
        if let Some(path) = trimmed.strip_prefix("- ") {
            let path = path.trim();
            if !path.is_empty() {
                entries.push(ContactEntry {
                    profile_path: path.to_string(),
                    summary: String::new(),
                });
            }
        }
    }

    Ok(entries)
}

/// Serialize contacts to Markdown bullet list content.
pub fn serialize_contacts(title: &str, contacts: &[ContactEntry]) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", title));

    for entry in contacts {
        out.push_str(&format!("- {}\n", entry.profile_path));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contacts_bullets() {
        let content = r#"# team

- /agents/alice.md
- /agents/bob.md
"#;
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts.len(), 2);
        assert_eq!(contacts[0].profile_path, "/agents/alice.md");
        assert_eq!(contacts[1].profile_path, "/agents/bob.md");
    }

    #[test]
    fn test_parse_contacts_empty() {
        let content = r#"# empty
"#;
        let contacts = parse_contacts(content).expect("should parse empty");
        assert!(contacts.is_empty());
    }

    #[test]
    fn test_serialize_contacts_roundtrip() {
        let contacts = vec![
            ContactEntry {
                profile_path: "/agents/alice.md".to_string(),
                summary: String::new(),
            },
            ContactEntry {
                profile_path: "/agents/bob.md".to_string(),
                summary: String::new(),
            },
        ];

        let serialized = serialize_contacts("team", &contacts);
        let parsed = parse_contacts(&serialized).expect("should parse serialized");
        assert_eq!(contacts, parsed);
    }

    #[test]
    fn test_parse_contacts_title() {
        let content = "# my-team\n\n- /a.md\n";
        let title = parse_contacts_title(content).expect("should parse title");
        assert_eq!(title, "my-team");
    }

    #[test]
    fn test_parse_contacts_crlf() {
        let content = "# test\r\n\r\n- /a.md\r\n";
        let contacts = parse_contacts(content).expect("should parse CRLF");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].profile_path, "/a.md");
    }

    #[test]
    fn test_parse_contacts_unicode() {
        let content = r#"# équipe

- /agents/alicé.md
"#;
        let contacts = parse_contacts(content).expect("should parse unicode");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].profile_path, "/agents/alicé.md");
    }
}
