//! Contacts table parsing and serialization.
//!
//! A contacts file is a special brief: a table of profile paths + summaries.
//!
//! Format:
//! ```markdown
//! # Contacts: <title>
//!
//! | Path | Summary |
//! |------|---------|
//! | /path/to/alice.md | Alice agent profile |
//! | /path/to/bob.md | Bob agent profile |
//! ```

use crate::{ContactEntry, PaperworkError, Result};

use super::normalize_line_endings;

/// Extract the title from contacts content (the H1 heading).
pub fn parse_contacts_title(content: &str) -> Result<String> {
    let content = normalize_line_endings(content);
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# Contacts: ") {
            return Ok(stripped.to_string());
        }
    }
    Err(PaperworkError::Parse(
        "missing contacts title heading (# Contacts: <title>)".to_string(),
    ))
}

/// Parse contacts from Markdown table content.
pub fn parse_contacts(content: &str) -> Result<Vec<ContactEntry>> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();

    let mut entries = Vec::new();
    let mut in_table = false;

    for line in &lines {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip H1 header
        if trimmed.starts_with("# ") {
            continue;
        }

        // Table row detection
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let cells: Vec<&str> = trimmed
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(|s| s.trim())
                .collect();

            // Header row
            if cells.len() >= 2 && cells[0] == "Path" && cells[1] == "Summary" {
                in_table = true;
                continue;
            }

            // Separator row (|---|---|)
            if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':')) {
                continue;
            }

            // Data row
            if in_table && cells.len() >= 2 {
                entries.push(ContactEntry {
                    profile_path: cells[0].to_string(),
                    summary: cells[1].to_string(),
                });
            }
        }
    }

    Ok(entries)
}

/// Serialize contacts to Markdown table content.
pub fn serialize_contacts(title: &str, contacts: &[ContactEntry]) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Contacts: {}\n\n", title));
    out.push_str("| Path | Summary |\n");
    out.push_str("|------|--------|\n");

    for entry in contacts {
        out.push_str(&format!("| {} | {} |\n", entry.profile_path, entry.summary));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contacts_table() {
        let content = r#"# Contacts: team

| Path | Summary |
|------|--------|
| /agents/alice.md | Alice agent profile |
| /agents/bob.md | Bob agent profile |
"#;
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts.len(), 2);
        assert_eq!(contacts[0].profile_path, "/agents/alice.md");
        assert_eq!(contacts[0].summary, "Alice agent profile");
        assert_eq!(contacts[1].profile_path, "/agents/bob.md");
    }

    #[test]
    fn test_parse_contacts_empty() {
        let content = r#"# Contacts: empty

| Path | Summary |
|------|--------|
"#;
        let contacts = parse_contacts(content).expect("should parse empty");
        assert!(contacts.is_empty());
    }

    #[test]
    fn test_serialize_contacts_roundtrip() {
        let contacts = vec![
            ContactEntry {
                profile_path: "/agents/alice.md".to_string(),
                summary: "Alice profile".to_string(),
            },
            ContactEntry {
                profile_path: "/agents/bob.md".to_string(),
                summary: "Bob profile".to_string(),
            },
        ];

        let serialized = serialize_contacts("team", &contacts);
        let parsed = parse_contacts(&serialized).expect("should parse serialized");
        assert_eq!(contacts, parsed);
    }

    #[test]
    fn test_parse_contacts_title() {
        let content = "# Contacts: my-team\n\n| Path | Summary |\n|------|--------|\n";
        let title = parse_contacts_title(content).expect("should parse title");
        assert_eq!(title, "my-team");
    }

    #[test]
    fn test_parse_contacts_crlf() {
        let content = "# Contacts: test\r\n\r\n| Path | Summary |\r\n|------|--------|\r\n| /a.md | Agent A |\r\n";
        let contacts = parse_contacts(content).expect("should parse CRLF");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].profile_path, "/a.md");
    }

    #[test]
    fn test_parse_contacts_unicode() {
        let content = r#"# Contacts: équipe

| Path | Summary |
|------|--------|
| /agents/alicé.md | Profil d'Alicé 🚀 |
"#;
        let contacts = parse_contacts(content).expect("should parse unicode");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].profile_path, "/agents/alicé.md");
        assert!(contacts[0].summary.contains('🚀'));
    }
}
