//! Validate command: check Markdown structure of a file by type (spec section 8).
//!
//! For `.post.md` the checks run in order: parse (>= 1 message) -> seq
//! monotonicity -> fence closure -> suspected-header heuristic (warning only).
//! Error envelopes surface the underlying variant directly (R10): parse/fence
//! failures are `Parse` (category `format`), seq failures are `Validation`
//! (category `validation`).

use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::Result;
use clap::{Args, ValueEnum};
use regex::Regex;

use crate::cmd::Context;
use crate::output;

/// Explicit parser selection for `validate --type` (U-15, additive).
#[derive(ValueEnum, Clone, Copy)]
pub enum FileKind {
    Post,
    Profile,
    Brief,
    Contacts,
}

/// Suspected message header heuristic (spec section 8 step 4): `##` + whitespace +
/// `#<digit>`. Regex-based so multi-space variants (`##  #1 ...`) are caught,
/// aligned with `MESSAGE_HEADER_RE`'s `\s+` lenient stance (R9, review N2).
static SUSPECTED_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^##\s+#\d").expect("valid regex"));

#[derive(Args)]
#[command(
    after_help = "Examples:\n  paperwork validate standup.post.md\n  paperwork validate mystery.md --type post"
)]
pub struct ValidateArgs {
    /// Path to the file to validate
    pub path: PathBuf,

    /// Parse as this type instead of inferring from the suffix
    #[arg(long = "type", value_enum)]
    pub kind: Option<FileKind>,
}

/// Command identifier for the output protocol.
pub fn command_id(_args: &ValidateArgs) -> &'static str {
    "validate"
}

pub fn run(ctx: &Context, args: ValidateArgs) -> Result<()> {
    let path_str = args.path.to_string_lossy().to_string();

    // Detect file type: --type overrides suffix inference (spec 3.5)
    let file_type = if let Some(kind) = args.kind {
        match kind {
            FileKind::Post => FileType::Post,
            FileKind::Profile => FileType::Profile,
            FileKind::Brief => FileType::Brief,
            FileKind::Contacts => FileType::Contacts,
        }
    } else if path_str.ends_with(".post.md") {
        FileType::Post
    } else if path_str.ends_with(".profile.md") {
        FileType::Profile
    } else if path_str.ends_with(".brief.md") {
        FileType::Brief
    } else if path_str.ends_with(".contacts.md") {
        FileType::Contacts
    } else {
        // Unknown suffix and no --type given
        return Err(paperwork_core::PaperworkError::Parse {
            message: format!("unknown file type: {}", path_str),
            fix: "file must end with .post.md/.profile.md/.brief.md/.contacts.md, or pass --type"
                .to_string(),
            example: "paperwork validate myfile.md --type post".to_string(),
        }
        .into());
    };

    let content = std::fs::read_to_string(&args.path).map_err(|e| {
        paperwork_core::PaperworkError::IoContext {
            path: args.path.clone(),
            source: e,
            fix: "check that the file exists and is readable".to_string(),
            example: format!("paperwork validate {}", path_str),
        }
    })?;

    let mut env = output::Envelope::new("validate", path_str);

    match file_type {
        FileType::Post => {
            // Step 1: parse; empty content or zero messages -> Parse (spec section 8;
            // the v0.4 empty-file exemption is removed, BDD:VAL-07).
            let messages = paperwork_core::format::thread::parse_messages(&content)?;
            if messages.is_empty() {
                return Err(paperwork_core::PaperworkError::Parse {
                    message: "no valid messages found".to_string(),
                    fix:
                        "expected '## #<seq> <sender> (<timestamp>)' headers with dynamic md fences"
                            .to_string(),
                    example: "paperwork post send myfile --author alice --message \"hello\""
                        .to_string(),
                }
                .into());
            }

            // Step 2: seq monotonicity (Validation surfaces directly, R10).
            paperwork_core::format::thread::validate_seq_monotonicity(&messages)?;

            // Step 3: fence closure (dynamic-length fence aware).
            fence_check(&content)?;

            // Step 4: suspected-header heuristic (warning only; does not
            // change the ok/error conclusion, BDD:VAL-08).
            let warnings = suspected_header_warnings(&content);
            if !warnings.is_empty() {
                env = env.body_lines(warnings);
            }
        }
        FileType::Profile => {
            paperwork_core::format::profile::parse_profile(&content)?;
            fence_check(&content)?;
        }
        FileType::Brief => {
            paperwork_core::format::manifest::parse_manifest(&content)?;
            fence_check(&content)?;
        }
        FileType::Contacts => {
            let entries = paperwork_core::format::contacts::parse_contacts(&content)?;
            // Lower bound symmetric with post (review M2): zero link bullets
            // PLUS fence-outside bare bullets = unmigrated v0.4 legacy form.
            // Empty files keep the existing pass-through semantics (no
            // bullets at all -> nothing to migrate).
            if entries.is_empty()
                && paperwork_core::format::contacts::contains_bare_bullet(&content)
            {
                return Err(paperwork_core::PaperworkError::Parse {
                    message: "contacts file contains legacy bare-path bullets but no link bullets".to_string(),
                    fix: "this file is in the v0.4 legacy format; migrate it by hand per the CHANGELOG migration guide: wrap each path in a Markdown link bullet '- [label](path)'".to_string(),
                    example: "- [alice](agents/alice.profile.md)".to_string(),
                }.into());
            }
            fence_check(&content)?;
        }
    }

    output::emit_ok(ctx, env);
    Ok(())
}

/// Fence closure check (spec section 8 step 3 / validate_markdown). Unclosed fences
/// are reported as `Parse` (category `format`) with the opening line number.
fn fence_check(content: &str) -> Result<()> {
    let issues = paperwork_core::format::validate_markdown(content);
    if issues.is_empty() {
        return Ok(());
    }
    Err(paperwork_core::PaperworkError::Parse {
        message: issues.join("; "),
        fix:
            "close every code fence with a backtick-only line at least as long as the opening fence"
                .to_string(),
        example: "paperwork validate standup.post.md --type post".to_string(),
    }
    .into())
}

/// Suspected message header heuristic (spec section 8 step 4, R9).
///
/// A flush-left line that looks like `## #<digits>` but does not strictly
/// match the message header grammar (and is not inside a fence) is reported
/// as a warning with the expected-format fix. Warnings never change the
/// ok/error conclusion.
fn suspected_header_warnings(content: &str) -> Vec<String> {
    use paperwork_core::format::thread::MESSAGE_HEADER_RE;
    use paperwork_core::format::{fence_close_matches, fence_open_len, normalize_line_endings};

    let content = normalize_line_endings(content);
    let mut warnings = Vec::new();
    let mut open: Option<usize> = None;

    for (i, line) in content.lines().enumerate() {
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
        let looks_like_header = SUSPECTED_HEADER_RE.is_match(line);
        if looks_like_header && !MESSAGE_HEADER_RE.is_match(line) {
            warnings.push(format!(
                "warning: line {}: suspected message header: {}",
                i + 1,
                line.trim_end()
            ));
            warnings.push("fix: expected format: ## #<seq> <sender> (<timestamp>)".to_string());
            warnings.push("example: ## #1 alice (2026-01-15T10:30:00Z)".to_string());
        }
    }

    warnings
}

enum FileType {
    Post,
    Profile,
    Brief,
    Contacts,
}
