//! Profile parsing and serialization.
//!
//! Format spec:
//! ```markdown
//! # <name>
//!
//! - Model: <model-id>
//! - Description: <free-text>
//!
//! ## Scope
//!
//! - Read: `<glob>`, `<glob>`, ...
//! - Write: `<glob>`, `<glob>`, ...
//! - Owns: `<glob>`, `<glob>`, ...
//! ```

use crate::{PaperworkError, Profile, Result};

use super::{extract_bullet_key, normalize_line_endings, parse_scope_globs, serialize_scope_globs};

/// Parse a profile from Markdown content.
pub fn parse_profile(content: &str) -> Result<Profile> {
    let content = normalize_line_endings(content);
    let lines: Vec<&str> = content.lines().collect();

    let mut name: Option<String> = None;
    let mut model: Option<String> = None;
    let mut description: Option<String> = None;
    let mut scope_read: Vec<String> = Vec::new();
    let mut scope_write: Vec<String> = Vec::new();
    let mut scope_owns: Vec<String> = Vec::new();

    let mut in_scope_section = false;

    for line in &lines {
        let trimmed = line.trim();

        // H1 heading → agent name
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            name = Some(trimmed[2..].trim().to_string());
            continue;
        }

        // H2 Scope section
        if trimmed == "## Scope" {
            in_scope_section = true;
            continue;
        }

        // Any other H2 ends scope section
        if trimmed.starts_with("## ") {
            in_scope_section = false;
            continue;
        }

        // Bullet key extraction
        if let Some((key, value)) = extract_bullet_key(trimmed) {
            match key.as_str() {
                "Model" => model = Some(value),
                "Description" => description = Some(value),
                "Read" if in_scope_section => scope_read = parse_scope_globs(&value),
                "Write" if in_scope_section => scope_write = parse_scope_globs(&value),
                "Owns" if in_scope_section => scope_owns = parse_scope_globs(&value),
                _ => {}
            }
        }
    }

    let name = name.ok_or_else(|| {
        PaperworkError::Parse {
            message: "missing agent name heading (# <name>)".to_string(),
            fix: "add a top-level heading with the agent name".to_string(),
            example: "# alice".to_string(),
        }
    })?;

    let model = model.ok_or_else(|| {
        PaperworkError::Parse {
            message: format!("missing - Model: line for profile '{}'", name),
            fix: "add a '- Model: <model-id>' bullet line".to_string(),
            example: "- Model: gpt-4o".to_string(),
        }
    })?;

    Ok(Profile {
        name,
        model,
        description: description.unwrap_or_default(),
        scope_read,
        scope_write,
        scope_owns,
    })
}

/// Serialize a profile to Markdown content.
pub fn serialize_profile(profile: &Profile) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", profile.name));
    out.push_str(&format!("- Model: {}\n", profile.model));
    out.push_str(&format!("- Description: {}\n\n", profile.description));
    out.push_str("## Scope\n\n");
    out.push_str(&format!(
        "- Read: {}\n",
        serialize_scope_globs(&profile.scope_read)
    ));
    out.push_str(&format!(
        "- Write: {}\n",
        serialize_scope_globs(&profile.scope_write)
    ));
    out.push_str(&format!(
        "- Owns: {}\n",
        serialize_scope_globs(&profile.scope_owns)
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_profile_basic() {
        let content = r#"# alice

- Model: gpt-4
- Description: Test agent for unit tests

## Scope

- Read: `src/**`, `docs/**`
- Write: `src/**`
- Owns: `src/core/**`
"#;
        let profile = parse_profile(content).expect("should parse");
        assert_eq!(profile.name, "alice");
        assert_eq!(profile.model, "gpt-4");
        assert_eq!(profile.description, "Test agent for unit tests");
        assert_eq!(profile.scope_read, vec!["src/**", "docs/**"]);
        assert_eq!(profile.scope_write, vec!["src/**"]);
        assert_eq!(profile.scope_owns, vec!["src/core/**"]);
    }

    #[test]
    fn test_parse_profile_empty_scope() {
        let content = r#"# bob

- Model: claude-3
- Description: Minimal agent

## Scope

- Read: —
- Write: —
- Owns: —
"#;
        let profile = parse_profile(content).expect("should parse");
        assert_eq!(profile.name, "bob");
        assert!(profile.scope_read.is_empty());
        assert!(profile.scope_write.is_empty());
        assert!(profile.scope_owns.is_empty());
    }

    #[test]
    fn test_parse_profile_multi_glob() {
        let content = r#"# multi

- Model: test
- Description: Multi-glob test

## Scope

- Read: `a/**`, `b/**`, `c/**`, `d/**`
- Write: —
- Owns: `x/*.rs`, `y/*.toml`
"#;
        let profile = parse_profile(content).expect("should parse");
        assert_eq!(profile.scope_read.len(), 4);
        assert_eq!(profile.scope_owns, vec!["x/*.rs", "y/*.toml"]);
    }

    #[test]
    fn test_serialize_profile_roundtrip() {
        let profile = Profile {
            name: "roundtrip".to_string(),
            model: "test-model".to_string(),
            description: "Roundtrip test".to_string(),
            scope_read: vec!["src/**".to_string()],
            scope_write: vec!["src/**".to_string(), "tests/**".to_string()],
            scope_owns: vec![],
        };

        let serialized = serialize_profile(&profile);
        let parsed = parse_profile(&serialized).expect("should parse serialized");
        assert_eq!(profile, parsed);
    }

    #[test]
    fn test_parse_profile_invalid_no_h1() {
        let content = r#"- Model: gpt-4
- Description: No name heading
"#;
        let result = parse_profile(content);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing agent name heading"));
    }

    #[test]
    fn test_parse_profile_missing_model() {
        let content = r#"# alice

- Description: No model line
"#;
        let result = parse_profile(content);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing - Model:"));
    }

    #[test]
    fn test_parse_profile_crlf() {
        let content = "# alice\r\n\r\n- Model: gpt-4\r\n- Description: CRLF test\r\n\r\n## Scope\r\n\r\n- Read: —\r\n- Write: —\r\n- Owns: —\r\n";
        let profile = parse_profile(content).expect("should parse CRLF");
        assert_eq!(profile.name, "alice");
        assert_eq!(profile.model, "gpt-4");
    }

    #[test]
    fn test_parse_profile_unicode() {
        let content = r#"# ünïcödé

- Model: mödel-π
- Description: Descriptión with émojis 🚀

## Scope

- Read: `src/**`
- Write: —
- Owns: —
"#;
        let profile = parse_profile(content).expect("should parse unicode");
        assert_eq!(profile.name, "ünïcödé");
        assert_eq!(profile.model, "mödel-π");
        assert!(profile.description.contains('🚀'));
    }
}
