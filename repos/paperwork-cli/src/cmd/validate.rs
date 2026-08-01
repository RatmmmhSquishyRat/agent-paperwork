//! Validate command: check Markdown structure of a file by type.

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::cmd::Context;
use crate::output;

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the file to validate
    pub path: PathBuf,
}

pub fn run(ctx: &Context, args: ValidateArgs) -> Result<()> {
    let path_str = args.path.to_string_lossy().to_string();

    // Detect file type from suffix
    let file_type = if path_str.ends_with(".post.md") {
        FileType::Post
    } else if path_str.ends_with(".profile.md") {
        FileType::Profile
    } else if path_str.ends_with(".brief.md") {
        FileType::Brief
    } else if path_str.ends_with(".contacts.md") {
        FileType::Contacts
    } else {
        // Unknown suffix
        return Err(paperwork_core::PaperworkError::Parse {
            message: format!("unknown file type: {}", path_str),
            fix: "file must end with .post.md, .profile.md, .brief.md, or .contacts.md".to_string(),
            example: "paperwork validate myfile.post.md".to_string(),
        }.into());
    };

    let content = std::fs::read_to_string(&args.path)
        .map_err(|e| paperwork_core::PaperworkError::IoContext {
            path: args.path.clone(),
            source: e,
            fix: "check that the file exists and is readable".to_string(),
            example: format!("paperwork validate {}", path_str),
        })?;

    // Call the corresponding parser
    let result = match file_type {
        FileType::Post => paperwork_core::format::thread::parse_messages(&content)
            .and_then(|msgs| {
                if msgs.is_empty() && !content.trim().is_empty() {
                    Err(paperwork_core::PaperworkError::Parse {
                        message: "no valid message boundaries found".to_string(),
                        fix: "expected --- separators with ### #N sender . timestamp headers and ````markdown fenced bodies".to_string(),
                        example: "paperwork post send myfile --from alice \"hello\"".to_string(),
                    })
                } else {
                    Ok(())
                }
            }),
        FileType::Profile => paperwork_core::format::profile::parse_profile(&content).map(|_| ()),
        FileType::Brief => paperwork_core::format::manifest::parse_manifest(&content).map(|_| ()),
        FileType::Contacts => paperwork_core::format::contacts::parse_contacts(&content).map(|_| ()),
    };

    match result {
        Ok(()) => {
            let env = output::Envelope::new("validate", path_str);
            output::emit_ok(ctx, env);
            Ok(())
        }
        Err(e) => {
            // Report as format error with specific issues
            let detail = match &e {
                paperwork_core::PaperworkError::Parse { message, .. } => message.clone(),
                other => other.to_string(),
            };
            Err(paperwork_core::PaperworkError::Parse {
                message: format!("{} is not a valid {} file: {}", args.path.display(), file_type.label(), detail),
                fix: e.fix(),
                example: e.example(),
            }.into())
        }
    }
}

enum FileType {
    Post,
    Profile,
    Brief,
    Contacts,
}

impl FileType {
    fn label(&self) -> &'static str {
        match self {
            Self::Post => ".post.md",
            Self::Profile => ".profile.md",
            Self::Brief => ".brief.md",
            Self::Contacts => ".contacts.md",
        }
    }
}
