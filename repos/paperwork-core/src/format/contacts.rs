//! Contacts table parsing and serialization.
//!
//! Format spec (§2.2):
//! ```markdown
//! # Contacts
//!
//! | Agent | Profile |
//! |-------|--------|
//! | alice | profiles/alice.md |
//! | bob | profiles/bob.md |
//! ```

use crate::{ContactEntry, PaperworkError, Result};

use super::normalize_line_endings;

/// Parse contacts from Markdown table content.
pub fn parse_contacts(content: &str) -> Result<Vec<ContactEntry>> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();

    let mut entries = Vec::new();
    let mut in_table = false;
    let mut header_seen = false;

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
            if cells.len() >= 2 && cells[0] == "Agent" && cells[1] == "Profile" {
                in_table = true;
                header_seen = true;
                continue;
            }

            // Separator row (|---|---|)
            if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':')) {
                continue;
            }

            // Data row
            if in_table && cells.len() >= 2 {
                entries.push(ContactEntry {
                    agent: cells[0].to_string(),
                    profile_path: cells[1].to_string(),
                });
            }
        }
    }

    if !header_seen && !entries.is_empty() {
        return Err(PaperworkError::Parse(
            "contacts table missing header row (| Agent | Profile |)".to_string(),
        ));
    }

    Ok(entries)
}

/// Serialize contacts to Markdown table content.
pub fn serialize_contacts(contacts: &[ContactEntry]) -> String {
    let mut out = String::new();

    out.push_str("# Contacts\n\n");
    out.push_str("| Agent | Profile |\n");
    out.push_str("|-------|--------|\n");

    for entry in contacts {
        out.push_str(&format!("| {} | {} |\n", entry.agent, entry.profile_path));
    }

    out
}

/// Derive the DM folder path for two agents.
/// Names are sorted alphabetically and joined with `--`.
/// Invariant I5: DM pair folder name = alphabetically sorted names joined by `--`
pub fn dm_folder_path(agent_a: &str, agent_b: &str) -> String {
    let mut names = [agent_a, agent_b];
    names.sort();
    format!("dm/{}--{}", names[0], names[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contacts_table() {
        let content = r#"# Contacts

| Agent | Profile |
|-------|--------|
| alice | profiles/alice.md |
| bob | profiles/bob.md |
| charlie | profiles/charlie.md |
"#;
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts.len(), 3);
        assert_eq!(contacts[0].agent, "alice");
        assert_eq!(contacts[0].profile_path, "profiles/alice.md");
        assert_eq!(contacts[1].agent, "bob");
        assert_eq!(contacts[2].agent, "charlie");
    }

    #[test]
    fn test_parse_contacts_empty() {
        let content = r#"# Contacts

| Agent | Profile |
|-------|--------|
"#;
        let contacts = parse_contacts(content).expect("should parse empty");
        assert!(contacts.is_empty());
    }

    #[test]
    fn test_serialize_contacts_roundtrip() {
        let contacts = vec![
            ContactEntry {
                agent: "alice".to_string(),
                profile_path: "profiles/alice.md".to_string(),
            },
            ContactEntry {
                agent: "bob".to_string(),
                profile_path: "profiles/bob.md".to_string(),
            },
        ];

        let serialized = serialize_contacts(&contacts);
        let parsed = parse_contacts(&serialized).expect("should parse serialized");
        assert_eq!(contacts, parsed);
    }

    #[test]
    fn test_dm_folder_path_sorted() {
        assert_eq!(dm_folder_path("alice", "bob"), "dm/alice--bob");
        assert_eq!(dm_folder_path("bob", "alice"), "dm/alice--bob");
        assert_eq!(dm_folder_path("zara", "alice"), "dm/alice--zara");
    }

    #[test]
    fn test_parse_contacts_crlf() {
        let content = "# Contacts\r\n\r\n| Agent | Profile |\r\n|-------|--------|\r\n| alice | profiles/alice.md |\r\n";
        let contacts = parse_contacts(content).expect("should parse CRLF");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].agent, "alice");
    }

    #[test]
    fn test_parse_contacts_unicode_agents() {
        let content = r#"# Contacts

| Agent | Profile |
|-------|--------|
| alicé | profiles/alicé.md |
| böb | profiles/böb.md |
"#;
        let contacts = parse_contacts(content).expect("should parse unicode");
        assert_eq!(contacts.len(), 2);
        assert_eq!(contacts[0].agent, "alicé");
        assert_eq!(contacts[1].agent, "böb");
    }
}
