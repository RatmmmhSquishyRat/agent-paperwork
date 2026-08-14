//! Contacts parsing and serialization — Managed File Format v2 (spec §7).
//!
//! Entries are Markdown link bullets: `- [<label>](<destination>)`.
//! Two destination forms are accepted on parse: bare paths and
//! angle-bracket paths (`(<path with spaces>)` with `\<`/`\>` escapes).
//! Serialization escapes per spec §7.3.

use crate::{ContactEntry, PaperworkError, Result};

use super::{first_outside_fence, for_each_outside_fence, normalize_line_endings};

/// Extract the title from contacts content (the H1 heading).
///
/// Fence-aware (NEW-5): an H1 inside a code fence (quoted example content)
/// is never mistaken for the title; the first fence-outside `# ` heading
/// wins. T4: converged onto the shared scanner family
/// ([`for_each_outside_fence`] first-hit).
pub fn parse_contacts_title(content: &str) -> Result<String> {
    let content = normalize_line_endings(content);
    let mut title = None;
    for_each_outside_fence(&content, |_i, line| {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            if !trimmed.starts_with("## ") {
                title = Some(stripped.trim().to_string());
                return false;
            }
        }
        true
    });
    title.ok_or_else(|| PaperworkError::Parse {
        message: "missing contacts title heading (# <title>)".to_string(),
        fix: "add a top-level heading with the contacts title".to_string(),
        example: "# my-team".to_string(),
    })
}

/// Parse contacts from Markdown content (spec §7.2).
///
/// Only link bullets are recognized; bare-path bullets (legacy) and any
/// other content are ignored (§3.6).
pub fn parse_contacts(content: &str) -> Result<Vec<ContactEntry>> {
    let content = normalize_line_endings(content);
    let mut entries = Vec::new();

    for line in content.lines() {
        let rest = match line.trim().strip_prefix("- ") {
            Some(rest) => rest,
            None => continue,
        };
        if let Some(entry) = parse_link_bullet(rest) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Whether the content carries at least one BARE bullet outside fences
/// (review B1): a `- ` bullet (flush left or indented, matching the
/// lenient trim policy of [`parse_contacts`]) that is NOT a Markdown link
/// bullet. v0.4 contacts files store bare paths this way; v0.5 parsing
/// silently ignores them, so a read-modify-rewrite would drop the legacy
/// entries — the write side uses this predicate as a refusal guard.
///
/// T4: converged onto the shared scanner family (`first_outside_fence`).
pub fn contains_bare_bullet(content: &str) -> bool {
    let content = normalize_line_endings(content);
    first_outside_fence(&content, |_i, line| {
        line.trim()
            .strip_prefix("- ")
            .is_some_and(|rest| parse_link_bullet(rest).is_none())
    })
    .is_some()
}

/// Parse `[label](destination)` (spec §7.2), returning `None` for
/// non-link bullets.
fn parse_link_bullet(text: &str) -> Option<ContactEntry> {
    let chars: Vec<char> = text.chars().collect();
    if chars.first() != Some(&'[') {
        return None;
    }

    // Label: up to the first unescaped `]`; unescape `\]` and `\\`
    // (escape reflexivity, review B2).
    let mut label = String::new();
    let mut i = 1usize;
    let mut closed = false;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' && i + 1 < chars.len() && matches!(chars[i + 1], ']' | '\\') {
            label.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if ch == ']' {
            closed = true;
            i += 1;
            break;
        }
        label.push(ch);
        i += 1;
    }
    if !closed || chars.get(i) != Some(&'(') {
        return None;
    }
    i += 1;

    // Destination.
    let mut dest = String::new();
    if chars.get(i) == Some(&'<') {
        // Angle-bracket form: up to the first unescaped `>`; unescape
        // `\<` / `\>` / `\\` (escape reflexivity, review B2).
        i += 1;
        let mut closed = false;
        while i < chars.len() {
            let ch = chars[i];
            if ch == '\\' && i + 1 < chars.len() && matches!(chars[i + 1], '<' | '>' | '\\') {
                dest.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if ch == '>' {
                closed = true;
                i += 1;
                break;
            }
            dest.push(ch);
            i += 1;
        }
        if !closed {
            return None;
        }
        // Skip optional whitespace + `"title"` (title syntax not accepted,
        // ignored leniently), then require `)`.
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if chars.get(i) == Some(&'"') {
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                i += 1;
            }
            i += 1; // past closing quote (or past end)
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
        }
        if chars.get(i) != Some(&')') {
            return None;
        }
    } else {
        // Bare form: token up to the first whitespace or `)`. An unbalanced
        // `(` would silently truncate at the first `)` with a wrong path,
        // so such bullets are rejected as non-links (review N1, §3.6).
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != ')' {
            if chars[i] == '(' {
                return None;
            }
            dest.push(chars[i]);
            i += 1;
        }
        if dest.is_empty() {
            return None;
        }
        // Skip optional whitespace + `"title"`, then require `)`.
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if chars.get(i) == Some(&'"') {
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                i += 1;
            }
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
        }
        if chars.get(i) != Some(&')') {
            return None;
        }
    }

    Some(ContactEntry {
        label,
        profile_path: dest,
    })
}

/// Serialize contacts to Markdown content (spec §7.3).
///
/// Paths containing space, tab, `(`, `)`, `<` or `>` use the angle-bracket
/// destination form (with `<`/`>` escaped as `\<`/`\>`); labels escape `]`
/// as `\]`. Escaping is reflexive: the backslash itself is escaped first
/// (`\` -> `\\`) so trailing/consecutive backslashes cannot fuse with the
/// structural characters that follow (review B2).
pub fn serialize_contacts(title: &str, contacts: &[ContactEntry]) -> String {
    let mut out = format!("# {}\n\n", title);

    for entry in contacts {
        let label = entry.label.replace('\\', "\\\\").replace(']', "\\]");
        let path = &entry.profile_path;
        let needs_angle = path
            .chars()
            .any(|c| c == ' ' || c == '\t' || c == '(' || c == ')' || c == '<' || c == '>');
        if needs_angle {
            let escaped = path
                .replace('\\', "\\\\")
                .replace('<', "\\<")
                .replace('>', "\\>");
            out.push_str(&format!("- [{}](<{}>)\n", label, escaped));
        } else {
            out.push_str(&format!("- [{}]({})\n", label, path));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(label: &str, path: &str) -> ContactEntry {
        ContactEntry {
            label: label.to_string(),
            profile_path: path.to_string(),
        }
    }

    // T-FC-01 (CONT-01)
    #[test]
    fn test_parse_links() {
        let content =
            "# Core Team\n\n- [alice](agents/alice.profile.md)\n- [bob](agents/bob.profile.md)\n";
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts.len(), 2);
        assert_eq!(contacts[0].label, "alice");
        assert_eq!(contacts[0].profile_path, "agents/alice.profile.md");
        assert_eq!(contacts[1].label, "bob");
        assert_eq!(contacts[1].profile_path, "agents/bob.profile.md");
        assert_eq!(parse_contacts_title(content).expect("title"), "Core Team");
    }

    // T-FC-02 (CONT-02)
    #[test]
    fn test_parse_angle_bracket() {
        let content = "# t\n\n- [alice](<agents/my profile.md>)\n";
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].profile_path, "agents/my profile.md");
    }

    // T-FC-03 (CONT-03/CONT-04/CONT-08)
    #[test]
    fn test_serialize_escaping() {
        // space → angle-bracket form
        let out = serialize_contacts("t", &[entry("alice", "team docs/alice.profile.md")]);
        assert!(out.contains("- [alice](<team docs/alice.profile.md>)"));

        // tab → angle-bracket form
        let out = serialize_contacts("t", &[entry("a", "a\tb.md")]);
        assert!(out.contains("(<a\tb.md>)"));

        // parentheses and angle chars get escaped inside <>
        let out = serialize_contacts("t", &[entry("a", "weird (x) <y>.md")]);
        assert!(out.contains("(<weird (x) \\<y\\>.md>)"));
        let parsed = parse_contacts(&out).expect("parse");
        assert_eq!(parsed[0].profile_path, "weird (x) <y>.md");

        // plain path → bare form
        let out = serialize_contacts("t", &[entry("bob", "agents/bob.profile.md")]);
        assert!(out.contains("- [bob](agents/bob.profile.md)"));
        assert!(!out.contains('<'));

        // label with `]` escaped
        let out = serialize_contacts("t", &[entry("we]ird", "a.md")]);
        assert!(out.contains("- [we\\]ird](a.md)"));
    }

    // T-FC-04 (CONT-03/CONT-04)
    #[test]
    fn test_roundtrip_windows_path() {
        let contacts = vec![entry("alice", "C:\\team docs\\alice.profile.md")];
        let serialized = serialize_contacts("team", &contacts);
        let parsed = parse_contacts(&serialized).expect("should roundtrip");
        assert_eq!(parsed, contacts);
    }

    // T-FC-05 (CONT-05)
    #[test]
    fn test_missing_title() {
        let err = parse_contacts_title("- [alice](a.md)").unwrap_err();
        assert!(err.to_string().contains("missing contacts title heading"));
        assert_eq!(err.category(), "format");
    }

    // T-FC-06 (CONT-06)
    #[test]
    fn test_bare_path_ignored() {
        let content = "# t\n\n- agents/alice.profile.md\n- [bob](agents/bob.profile.md)\n";
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].label, "bob");
    }

    // T-FC-07 (CONT-07)
    #[test]
    fn test_unicode() {
        let content = "# équipe 🚀\n\n- [alicé](agents/alicé.profile.md)\n";
        let contacts = parse_contacts(content).expect("should parse unicode");
        assert_eq!(contacts[0].label, "alicé");
        assert_eq!(contacts[0].profile_path, "agents/alicé.profile.md");
        assert_eq!(parse_contacts_title(content).expect("title"), "équipe 🚀");
    }

    // T-FC-08 (CONT-08)
    #[test]
    fn test_unescape_and_title() {
        // label `\]` unescape + destination `\<`/`\>` unescape
        let content = "# t\n\n- [we\\]ird](<a\\<b\\>c.md>)\n";
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts[0].label, "we]ird");
        assert_eq!(contacts[0].profile_path, "a<b>c.md");

        // `"title"` syntax ignored; destination still extracted (both forms)
        let content = "# t\n\n- [alice](agents/alice.md \"the title\")\n";
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts[0].profile_path, "agents/alice.md");

        let content = "# t\n\n- [alice](<agents/my file.md> \"the title\")\n";
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts[0].profile_path, "agents/my file.md");

        // roundtrip of escaped label
        let contacts_in = vec![entry("we]ird", "plain.md")];
        let parsed = parse_contacts(&serialize_contacts("t", &contacts_in)).expect("roundtrip");
        assert_eq!(parsed, contacts_in);
    }

    // CRLF normalization
    #[test]
    fn test_parse_contacts_crlf() {
        let content = "# t\r\n\r\n- [alice](agents/alice.profile.md)\r\n";
        let contacts = parse_contacts(content).expect("should parse CRLF");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].profile_path, "agents/alice.profile.md");
    }

    // B2: escape reflexivity — backslashes must survive the roundtrip
    #[test]
    fn test_roundtrip_backslash_escaping() {
        // label ending in a backslash
        let contacts = vec![entry("a\\", "p.md")];
        let serialized = serialize_contacts("t", &contacts);
        assert!(serialized.contains("- [a\\\\](p.md)"));
        assert_eq!(parse_contacts(&serialized).expect("parse"), contacts);

        // label with consecutive backslashes
        let contacts = vec![entry("a\\\\b", "p.md")];
        let parsed = parse_contacts(&serialize_contacts("t", &contacts)).expect("parse");
        assert_eq!(parsed, contacts);

        // bare-form path ending in a backslash
        let contacts = vec![entry("alice", "docs\\")];
        let parsed = parse_contacts(&serialize_contacts("t", &contacts)).expect("parse");
        assert_eq!(parsed, contacts);

        // angle-bracket path ending in a backslash (Windows dir-style)
        let contacts = vec![entry("alice", "team docs\\")];
        let serialized = serialize_contacts("t", &contacts);
        assert!(serialized.contains("(<team docs\\\\>)"));
        assert_eq!(parse_contacts(&serialized).expect("parse"), contacts);

        // angle-bracket path with consecutive backslashes
        let contacts = vec![entry("alice", "C:\\team\\\\share\\\\ docs\\\\")];
        let parsed = parse_contacts(&serialize_contacts("t", &contacts)).expect("parse");
        assert_eq!(parsed, contacts);
    }

    // N1: bare destination containing `(` is not a link bullet
    #[test]
    fn test_bare_dest_unbalanced_paren_ignored() {
        let content = "# t\n\n- [alice](path(x).md)\n- [bob](ok.md)\n";
        let contacts = parse_contacts(content).expect("should parse");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].label, "bob");
        assert_eq!(contacts[0].profile_path, "ok.md");
    }

    // B1: bare-bullet detection for the legacy write guard
    #[test]
    fn test_contains_bare_bullet() {
        // v0.4 legacy shape: bare path bullets
        assert!(contains_bare_bullet(
            "# t\n\n- agents/alice.profile.md\n- agents/bob.profile.md\n"
        ));
        // mixed file: one bare bullet among links still triggers
        assert!(contains_bare_bullet(
            "# t\n\n- [alice](a.md)\n- bare/path.md\n"
        ));
        // pure v0.5 link file: no bare bullets
        assert!(!contains_bare_bullet(
            "# t\n\n- [alice](a.md)\n- [bob](b.md)\n"
        ));
        // empty / title-only files
        assert!(!contains_bare_bullet("# t\n"));
        assert!(!contains_bare_bullet(""));
        // bare bullet inside a fence is quoted content, not a legacy entry
        assert!(!contains_bare_bullet("# t\n\n```\n- bare/path.md\n```\n"));
        // CRLF variant
        assert!(contains_bare_bullet("# t\r\n\r\n- bare/path.md\r\n"));
        // malformed link bullet counts as bare (not parseable -> would drop)
        assert!(contains_bare_bullet("# t\n\n- [unclosed(a.md\n"));
    }

    // ========================================================================
    // T4 differential corpus: pin the fence-aware scan semantics of
    // parse_contacts_title / contains_bare_bullet BEFORE their migration
    // onto the shared scanner family; the same corpus must pass unchanged
    // afterwards.
    // ========================================================================

    #[test]
    fn test_t4_parse_contacts_title_differential_corpus() {
        // plain + trailing newline
        assert_eq!(parse_contacts_title("# team\n").expect("title"), "team");
        assert_eq!(parse_contacts_title("# team").expect("title"), "team");
        // H1 inside a fence is quoted content, not the title
        assert_eq!(
            parse_contacts_title("```md\n# fake\n```\n# real").expect("title"),
            "real"
        );
        // <= 3 space indented fence is recognized
        assert_eq!(
            parse_contacts_title("   ```\n# fake\n   ```\n# real").expect("title"),
            "real"
        );
        // 4-space indent: no fence, the H1-shaped line stays visible
        assert_eq!(
            parse_contacts_title("    ```\n# real").expect("title"),
            "real"
        );
        // tilde fences are not recognized
        assert_eq!(
            parse_contacts_title("~~~\n# real\n~~~").expect("title"),
            "real"
        );
        // unclosed fence swallows the tail -> missing title
        assert!(parse_contacts_title("```md\n# fake\n").is_err());
        // nested backtick length: shorter run does not close the fence
        assert!(parse_contacts_title("````\n# fake\n```\n").is_err());
        assert_eq!(
            parse_contacts_title("````\n# fake\n```\n````\n# real").expect("title"),
            "real"
        );
        // CRLF input behaves like LF
        assert_eq!(
            parse_contacts_title("```md\r\n# fake\r\n```\r\n# real\r\n").expect("title"),
            "real"
        );
        // H2 is not a title
        assert!(parse_contacts_title("## not-a-title\n").is_err());
        // empty content
        assert!(parse_contacts_title("").is_err());
        // first fence-outside H1 wins
        assert_eq!(
            parse_contacts_title("# one\n\n# two").expect("title"),
            "one"
        );
    }

    #[test]
    fn test_t4_contains_bare_bullet_differential_corpus() {
        // <= 3 space indented fence hides the bullet
        assert!(!contains_bare_bullet("   ```\n- bare/path.md\n   ```"));
        // 4-space indent: no fence, the bullet is visible
        assert!(contains_bare_bullet("    ```\n- bare/path.md\n    ```"));
        // tilde fences are not recognized
        assert!(contains_bare_bullet("~~~\n- bare/path.md\n~~~"));
        // unclosed fence swallows the tail
        assert!(!contains_bare_bullet("```\n- bare/path.md"));
        // nested backtick length: shorter run does not close the fence
        assert!(!contains_bare_bullet("````\n- bare/path.md\n```\n"));
        assert!(contains_bare_bullet("````\n- x\n```\n````\n- bare/path.md"));
        // link bullets never trigger, even mixed with fences
        assert!(!contains_bare_bullet("```\n- [a](b.md)\n```\n- [c](d.md)"));
        // empty content
        assert!(!contains_bare_bullet(""));
    }
}
