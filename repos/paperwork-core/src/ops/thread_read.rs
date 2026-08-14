//! Read-side thread operations: read, summary, meta (T5 split of the
//! historical monolithic `ops/thread.rs`).
//!
//! All three are path-explicit, lock-free reads (`fs::read_to_string`):
//! concurrent writers are excluded by the send/edit locks, and a torn read
//! simply fails parsing like any malformed file. The write-side crash
//! windows live in the send/edit orchestration (`super::thread`).

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::dedup_preserve_order;
use crate::format::thread::{parse_messages, parse_preamble};
use crate::{Message, ThreadMeta, ThreadSummary};

/// Number of trailing messages quoted in `thread_summary` snippets (review n10).
const SNIPPET_COUNT: usize = 3;

/// Character budget of a single summary snippet before ellipsis (review n10).
const SNIPPET_CHAR_LIMIT: usize = 50;

/// Read the thread preamble metadata (spec §5.2).
///
/// A missing file yields the default meta (no error).
pub fn thread_meta(path: &Path) -> Result<ThreadMeta> {
    if !path.exists() {
        return Ok(ThreadMeta::default());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;

    Ok(parse_preamble(&content))
}

/// Read messages from a thread within an optional seq range (inclusive).
///
/// - `from = None` → start from beginning
/// - `to = None` → read to end
pub fn thread_read(path: &Path, from: Option<u64>, to: Option<u64>) -> Result<Vec<Message>> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Thread".to_string(),
            name: path.display().to_string(),
            fix: "send a message first to create the thread".to_string(),
            example: format!(
                "paperwork post send {} --from <name> <body>",
                path.display()
            ),
        });
    }

    let content = fs::read_to_string(path)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;

    let messages = parse_messages(&content)?;

    let from_seq = from.unwrap_or(1);
    let to_seq = to.unwrap_or(u64::MAX);

    let filtered: Vec<Message> = messages
        .into_iter()
        .filter(|m| m.seq >= from_seq && m.seq <= to_seq)
        .collect();

    Ok(filtered)
}

/// Get a summary of a thread.
pub fn thread_summary(path: &Path) -> Result<ThreadSummary> {
    if !path.exists() {
        return Ok(ThreadSummary {
            thread_path: path.display().to_string(),
            title: String::new(),
            message_count: 0,
            participants: Vec::new(),
            last_sender: None,
            last_timestamp: None,
            snippets: Vec::new(),
        });
    }

    let content = fs::read_to_string(path)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file permissions", ""))?;

    // Title from the preamble in the SAME pass (review M8): callers no
    // longer need a second full-file `thread_meta` walk.
    let title = parse_preamble(&content).title;

    let messages = parse_messages(&content)?;

    let message_count = messages.len() as u64;
    let last_sender = messages.last().map(|m| m.sender.clone());
    let last_timestamp = messages.last().map(|m| m.timestamp);

    // Participants derived from the sender set, deduplicated in
    // first-appearance order (spec §5.4, owner ruling D1). T4/NEW-10: the
    // shared [`dedup_preserve_order`] (HashSet+Vec) runs in O(n) instead of
    // the historical O(n²) `Vec::contains` loop.
    let participants = dedup_preserve_order(messages.iter().map(|m| m.sender.clone()));

    // Snippets from the last SNIPPET_COUNT messages (chronological order)
    let snippets: Vec<String> = messages
        .iter()
        .rev()
        .take(SNIPPET_COUNT)
        .map(|m| snippet_of(&m.body))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    Ok(ThreadSummary {
        thread_path: path.display().to_string(),
        title,
        message_count,
        participants,
        last_sender,
        last_timestamp,
        snippets,
    })
}

/// One summary snippet: first `SNIPPET_CHAR_LIMIT` chars of the body,
/// `...` appended when truncated. Single `char_indices` pass decides both
/// the cut point and the truncation flag (review n10).
fn snippet_of(body: &str) -> String {
    let mut end = body.len();
    let mut truncated = false;
    for (i, (byte_idx, _ch)) in body.char_indices().enumerate() {
        if i == SNIPPET_CHAR_LIMIT {
            end = byte_idx;
            truncated = true;
            break;
        }
    }
    let mut snippet = body[..end].to_string();
    if truncated {
        snippet.push_str("...");
    }
    snippet
}
