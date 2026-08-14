//! Profile parsing and serialization — Managed File Format v2 (spec §4).
//!
//! - H1 = agent name; prose between H1 and first H2 = description;
//! - `- model:` attribute line (required);
//! - `## Scope` section body = attribute-line list (`- read: <glob>` etc.,
//!   keys repeatable, order preserved, spec §4.2 R3).

use crate::{PaperworkError, Profile, Result};

use super::{extract_attribute, normalize_line_endings};

/// Parse a profile from Markdown content (spec §4.2).
pub fn parse_profile(content: &str) -> Result<Profile> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();

    let mut name: Option<String> = None;
    let mut model: Option<String> = None;
    let mut desc_lines: Vec<String> = Vec::new();
    let mut scope_read: Vec<String> = Vec::new();
    let mut scope_write: Vec<String> = Vec::new();
    let mut scope_owns: Vec<String> = Vec::new();

    let mut seen_h2 = false;
    let mut in_scope = false;

    for line in &lines {
        let trimmed = line.trim();

        // H1 heading → agent name (first H1 wins)
        if let Some(stripped) = line.strip_prefix("# ") {
            if name.is_none() {
                name = Some(stripped.trim().to_string());
            }
            continue;
        }

        // H2 handling: `## Scope` opens the scope section; any H2 ends the
        // description zone.
        if trimmed.starts_with("## ") {
            seen_h2 = true;
            in_scope = trimmed == "## Scope";
            continue;
        }

        if in_scope {
            // Scope section body = attribute-line list (spec §4.2, R3):
            // one (permission, glob) per line; keys repeatable; unknown
            // permissions ignored (§3.6); glob is the bare trimmed value.
            if let Some((key, value)) = extract_attribute(line) {
                match key.as_str() {
                    "read" => scope_read.push(value),
                    "write" => scope_write.push(value),
                    "owns" => scope_owns.push(value),
                    _ => {}
                }
            }
            continue;
        }

        // Preamble attribute lines (`- model:` etc.). Attribute-shaped lines
        // never become description (BDD:PROF-11).
        if let Some((key, value)) = extract_attribute(line) {
            if key == "model" && model.is_none() {
                model = Some(value);
            }
            continue;
        }

        // Prose between H1 and the first H2 = description (§2 rule 1).
        if name.is_some() && !seen_h2 && !trimmed.is_empty() {
            desc_lines.push(trimmed.to_string());
        }
    }

    let name = name.ok_or_else(|| PaperworkError::Parse {
        message: "missing agent name heading (# <name>)".to_string(),
        fix: "add a top-level heading with the agent name".to_string(),
        example: "# alice".to_string(),
    })?;

    let model = model.ok_or_else(|| PaperworkError::Parse {
        message: format!("missing - model: line for profile '{}'", name),
        fix: "add a '- model: <model-id>' bullet line".to_string(),
        example: "- model: gpt-4o".to_string(),
    })?;

    Ok(Profile {
        name,
        model,
        description: desc_lines.join("\n"),
        scope_read,
        scope_write,
        scope_owns,
    })
}

/// Serialize a profile to Markdown content (spec §4.3).
///
/// Exactly one blank line between blocks; exactly one `\n` at end of file.
/// Empty description / empty scope are omitted entirely (no `—` placeholders).
pub fn serialize_profile(profile: &Profile) -> String {
    let mut out = format!("# {}\n\n", profile.name);

    if !profile.description.is_empty() {
        out.push_str(&profile.description);
        out.push_str("\n\n");
    }

    out.push_str(&format!("- model: {}\n", profile.model));

    let mut scope_lines: Vec<String> = Vec::new();
    for glob in &profile.scope_read {
        scope_lines.push(format!("- read: {}", glob));
    }
    for glob in &profile.scope_write {
        scope_lines.push(format!("- write: {}", glob));
    }
    for glob in &profile.scope_owns {
        scope_lines.push(format!("- owns: {}", glob));
    }

    if !scope_lines.is_empty() {
        out.push_str("\n## Scope\n\n");
        for line in scope_lines {
            out.push_str(&line);
            out.push('\n');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-FP-01 (PROF-01)
    #[test]
    fn test_parse_minimal() {
        let content = "# alice\n\n- model: gpt-4o\n";
        let profile = parse_profile(content).expect("should parse");
        assert_eq!(profile.name, "alice");
        assert_eq!(profile.model, "gpt-4o");
        assert_eq!(profile.description, "");
        assert!(profile.scope_read.is_empty());
        assert!(profile.scope_write.is_empty());
        assert!(profile.scope_owns.is_empty());
    }

    // T-FP-02 (PROF-02)
    #[test]
    fn test_parse_description_scope_lines() {
        let content = "# alice\n\nParser module implementer\n\n- model: gpt-4o\n\n## Scope\n\n- read: src/**\n- write: src/parser/**\n- owns: src/parser/**\n";
        let profile = parse_profile(content).expect("should parse");
        assert_eq!(profile.name, "alice");
        assert_eq!(profile.model, "gpt-4o");
        assert_eq!(profile.description, "Parser module implementer");
        assert_eq!(profile.scope_read, vec!["src/**"]);
        assert_eq!(profile.scope_write, vec!["src/parser/**"]);
        assert_eq!(profile.scope_owns, vec!["src/parser/**"]);
    }

    // T-FP-03 (PROF-03)
    #[test]
    fn test_parse_multi_row_permission() {
        let content = "# alice\n\n- model: gpt-4o\n\n## Scope\n\n- read: src/**\n- read: docs/**\n- write: src/**\n";
        let profile = parse_profile(content).expect("should parse");
        assert_eq!(profile.scope_read, vec!["src/**", "docs/**"]);
        assert_eq!(profile.scope_write, vec!["src/**"]);
    }

    // T-FP-04 (PROF-04)
    #[test]
    fn test_serialize_empty_scope_omitted() {
        let profile = Profile {
            name: "alice".to_string(),
            model: "gpt-4o".to_string(),
            description: String::new(),
            scope_read: vec![],
            scope_write: vec![],
            scope_owns: vec![],
        };
        let serialized = serialize_profile(&profile);
        assert!(!serialized.contains("## Scope"));
        assert!(!serialized.contains("- read:"));
        assert!(!serialized.contains('—'));
        assert_eq!(serialized, "# alice\n\n- model: gpt-4o\n");
        let parsed = parse_profile(&serialized).expect("roundtrip");
        assert_eq!(parsed, profile);
    }

    // T-FP-05 (PROF-05)
    #[test]
    fn test_parse_missing_h1() {
        let content = "- model: gpt-4o\n";
        let result = parse_profile(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing agent name heading"));
        assert_eq!(err.category(), "format");
    }

    // T-FP-06 (PROF-06)
    #[test]
    fn test_parse_missing_model() {
        let content = "# alice\n\nSome description.\n";
        let result = parse_profile(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let text = err.to_string();
        assert!(text.contains("missing - model:"));
        assert!(err.fix().contains("- model:"));
        assert_eq!(err.example(), "- model: gpt-4o");

        // legacy uppercase key is not recognized (spec §3.2)
        let legacy = "# alice\n\n- Model: gpt-4o\n";
        assert!(parse_profile(legacy).is_err());
    }

    // T-FP-07 (PROF-07)
    #[test]
    fn test_parse_lenient() {
        let content = "# alice\n\n- model: gpt-4o\n- favorite: rust\n\n## Notes\n\nfree text\n\n## Scope\n\n- read: src/**\n- admin: everything\n";
        let profile = parse_profile(content).expect("should parse");
        assert_eq!(profile.scope_read, vec!["src/**"]);
        assert!(profile.scope_write.is_empty());
        assert!(profile.scope_owns.is_empty());
        // unknown section content and unknown attributes are ignored
        assert_eq!(profile.description, "");
    }

    // T-FP-08 (PROF-08)
    #[test]
    fn test_parse_crlf() {
        let lf = "# alice\n\n- model: gpt-4o\n\n## Scope\n\n- read: src/**\n";
        let crlf = lf.replace('\n', "\r\n");
        assert_eq!(
            parse_profile(lf).expect("lf"),
            parse_profile(&crlf).expect("crlf")
        );
    }

    // T-FP-09 (PROF-09)
    #[test]
    fn test_parse_unicode() {
        let content = "# ünïcödé\n\n描述 with émojis 🚀\n\n- model: mödel-π\n";
        let profile = parse_profile(content).expect("should parse unicode");
        assert_eq!(profile.name, "ünïcödé");
        assert_eq!(profile.model, "mödel-π");
        assert!(profile.description.contains('🚀'));
    }

    // T-FP-10 (PROF-10)
    #[test]
    fn test_roundtrip() {
        let full = Profile {
            name: "roundtrip".to_string(),
            model: "test-model".to_string(),
            description: "Roundtrip test agent".to_string(),
            scope_read: vec!["src/**".to_string(), "docs/**".to_string()],
            scope_write: vec!["src/**".to_string()],
            scope_owns: vec!["src/core/**".to_string()],
        };
        let serialized = serialize_profile(&full);
        assert!(!serialized.contains('·'));
        assert!(!serialized.contains('—'));
        assert!(!serialized.contains('`')); // globs are bare text
        assert!(!serialized.contains('|')); // no GFM tables
        assert_eq!(parse_profile(&serialized).expect("roundtrip"), full);

        let minimal = Profile {
            name: "min".to_string(),
            model: "m".to_string(),
            description: String::new(),
            scope_read: vec![],
            scope_write: vec![],
            scope_owns: vec![],
        };
        assert_eq!(
            parse_profile(&serialize_profile(&minimal)).expect("roundtrip"),
            minimal
        );
    }

    // T-FP-11 (PROF-11)
    #[test]
    fn test_description_bullet_attribution() {
        // attribute-shaped lines inside the description zone are recognized
        // as attribute lines (unknown key → ignored) and never enter the
        // description text
        let content =
            "# alice\n\nProse line one.\n- anything: value\nProse line two.\n\n- model: gpt-4o\n";
        let profile = parse_profile(content).expect("should parse");
        assert_eq!(profile.description, "Prose line one.\nProse line two.");
        assert!(!profile.description.contains("anything"));
    }
}
