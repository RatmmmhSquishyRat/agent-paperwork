//! Thread operations: atomic append, read-range, summary, self-edit.
//!
//! CRITICAL: append_msg uses fs2::FileExt::lock_exclusive() around read-seq + write
//! to prevent seq collision under concurrent writes.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use fs2::FileExt;
use regex::Regex;
use std::sync::LazyLock;

use crate::error::{PaperworkError, Result};
use crate::format::thread::{parse_messages, serialize_message, serialize_thread};
use crate::layout;
use crate::{Message, ThreadSummary};

/// Maximum message size (64KB hard cap).
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

/// Size of reverse-scan buffer for finding last seq (4KB).
const REVERSE_SCAN_SIZE: u64 = (64 * 1024 + 256) as u64; // Must exceed MAX_MESSAGE_SIZE to handle large last messages

/// Regex for extracting seq from message header.
static SEQ_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"### #(\d+) —").expect("valid regex"));

/// Append a message to a thread with atomic locking.
///
/// Uses fs2::FileExt::lock_exclusive() around read-seq + write to prevent
/// seq collision. The seq is assigned automatically based on the last message.
///
/// # Arguments
/// * `root` - Workspace root
/// * `thread_rel` - Relative thread path (e.g., "dm/alice--bob/thread.md")
/// * `msg` - Message to append (seq field is overwritten with assigned value)
pub fn append_msg(root: &Path, thread_rel: &str, msg: &Message) -> Result<()> {
    layout::ensure_initialized(root)?;

    let thread_path = layout::resolve_thread_path(root, thread_rel);

    // Ensure parent directory exists
    if let Some(parent) = thread_path.parent() {
        fs::create_dir_all(parent).map_err(|e| PaperworkError::IoContext {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    // Open file with append mode (creates if not exists)
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .read(true)
        .open(&thread_path)
        .map_err(|e| PaperworkError::IoContext {
            path: thread_path.clone(),
            source: e,
        })?;

    // Acquire exclusive lock (blocks concurrent writers)
    file.lock_exclusive().map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    // Read last seq within lock
    let last_seq = read_last_seq_locked(&file, &thread_path)?;
    let new_seq = last_seq + 1;

    // Create message with assigned seq
    let mut msg_with_seq = msg.clone();
    msg_with_seq.seq = new_seq;

    // Serialize message
    let serialized = serialize_message(&msg_with_seq);

    // Check size limit
    if serialized.len() > MAX_MESSAGE_SIZE {
        file.unlock().ok();
        return Err(PaperworkError::MessageTooLarge {
            size: serialized.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }

    // Single write() call for atomicity
    let mut writer = &file;
    writer
        .write_all(serialized.as_bytes())
        .map_err(|e| PaperworkError::IoContext {
            path: thread_path.clone(),
            source: e,
        })?;

    // Release lock
    file.unlock().map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Read the last seq number from a thread file (within lock).
///
/// Reverse-scans last 4KB for efficiency (O(1) regardless of file size).
fn read_last_seq_locked(file: &File, path: &Path) -> Result<u64> {
    let metadata = file.metadata().map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
    })?;

    let file_size = metadata.len();
    if file_size == 0 {
        return Ok(0); // Empty file, next seq is 1
    }

    // Calculate read position (last 4KB or whole file if smaller)
    let read_start = file_size.saturating_sub(REVERSE_SCAN_SIZE);
    let read_len = (file_size - read_start) as usize;

    // Read the tail portion
    let mut file_ref = file;
    file_ref
        .seek(SeekFrom::Start(read_start))
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
        })?;

    let mut buffer = vec![0u8; read_len];
    file_ref
        .read_exact(&mut buffer)
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
        })?;

    let content = String::from_utf8_lossy(&buffer);

    // Find all seq numbers and take the last one
    let mut last_seq = 0u64;
    for caps in SEQ_RE.captures_iter(&content) {
        if let Ok(seq) = caps[1].parse::<u64>() {
            last_seq = seq;
        }
    }

    Ok(last_seq)
}

/// Read messages from a thread within a seq range (inclusive).
pub fn read_range(root: &Path, thread_rel: &str, from: u64, to: u64) -> Result<Vec<Message>> {
    layout::ensure_initialized(root)?;

    let thread_path = layout::resolve_thread_path(root, thread_rel);

    if !thread_path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Thread".to_string(),
            name: thread_rel.to_string(),
            hint: "Send a message first to create the thread.".to_string(),
        });
    }

    let content = fs::read_to_string(&thread_path).map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    let messages = parse_messages(&content)?;

    // Filter by seq range (inclusive)
    let filtered: Vec<Message> = messages
        .into_iter()
        .filter(|m| m.seq >= from && m.seq <= to)
        .collect();

    Ok(filtered)
}

/// Get a summary of a thread.
pub fn summary(root: &Path, thread_rel: &str) -> Result<ThreadSummary> {
    layout::ensure_initialized(root)?;

    let thread_path = layout::resolve_thread_path(root, thread_rel);

    if !thread_path.exists() {
        return Ok(ThreadSummary {
            thread_path: thread_rel.to_string(),
            message_count: 0,
            last_sender: None,
            last_timestamp: None,
            snippets: Vec::new(),
        });
    }

    let content = fs::read_to_string(&thread_path).map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    let messages = parse_messages(&content)?;

    let message_count = messages.len() as u64;
    let last_sender = messages.last().map(|m| m.sender.clone());
    let last_timestamp = messages.last().map(|m| m.timestamp);

    // Get snippets from last 3 messages
    let snippets: Vec<String> = messages
        .iter()
        .rev()
        .take(3)
        .map(|m| {
            let preview: String = m.body.chars().take(50).collect();
            if m.body.len() > 50 {
                format!("{}...", preview)
            } else {
                preview
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    Ok(ThreadSummary {
        thread_path: thread_rel.to_string(),
        message_count,
        last_sender,
        last_timestamp,
        snippets,
    })
}

/// Self-edit: update the body of own message.
///
/// ONLY allowed if:
/// 1. The message is the sender's most recent message
/// 2. The message is the final message in the thread
///
/// Requires file lock for safe rewrite.
pub fn self_edit(
    root: &Path,
    thread_rel: &str,
    seq: u64,
    sender: &str,
    new_body: &str,
) -> Result<()> {
    layout::ensure_initialized(root)?;

    let thread_path = layout::resolve_thread_path(root, thread_rel);

    if !thread_path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Thread".to_string(),
            name: thread_rel.to_string(),
            hint: "Cannot edit a non-existent thread.".to_string(),
        });
    }

    // Open file for reading and writing
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&thread_path)
        .map_err(|e| PaperworkError::IoContext {
            path: thread_path.clone(),
            source: e,
        })?;

    // Acquire exclusive lock
    file.lock_exclusive().map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    // Read content through the locked file handle
    let mut content = String::new();
    file.seek(SeekFrom::Start(0)).map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;
    file.read_to_string(&mut content).map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    let mut messages = parse_messages(&content)?;

    if messages.is_empty() {
        file.unlock().ok();
        return Err(PaperworkError::NotFound {
            resource: "Message".to_string(),
            name: format!("#{}", seq),
            hint: "Thread is empty.".to_string(),
        });
    }

    // Find the message to edit
    let msg_index = messages.iter().position(|m| m.seq == seq);
    let msg_index = match msg_index {
        Some(idx) => idx,
        None => {
            file.unlock().ok();
            return Err(PaperworkError::NotFound {
                resource: "Message".to_string(),
                name: format!("#{}", seq),
                hint: "Check the seq number with `paperwork dm <agent> read`.".to_string(),
            });
        }
    };

    let msg = &messages[msg_index];

    // Check ownership
    if msg.sender != sender {
        file.unlock().ok();
        return Err(PaperworkError::NotAllowed {
            operation: "self_edit".to_string(),
            reason: format!(
                "Message #{} was sent by '{}', not '{}'",
                seq, msg.sender, sender
            ),
            hint: "You can only edit your own messages.".to_string(),
        });
    }

    // Check if it's the sender's most recent message
    let sender_last_seq = messages
        .iter()
        .filter(|m| m.sender == sender)
        .map(|m| m.seq)
        .max()
        .unwrap_or(0);

    if seq != sender_last_seq {
        file.unlock().ok();
        return Err(PaperworkError::NotAllowed {
            operation: "self_edit".to_string(),
            reason: format!(
                "Message #{} is not your most recent message (your last is #{})",
                seq, sender_last_seq
            ),
            hint: "You can only edit your most recent message.".to_string(),
        });
    }

    // Check if it's the final message in thread
    let last_seq = messages.last().map(|m| m.seq).unwrap_or(0);
    if seq != last_seq {
        file.unlock().ok();
        return Err(PaperworkError::NotAllowed {
            operation: "self_edit".to_string(),
            reason: format!(
                "Message #{} is not the final message in thread (last is #{})",
                seq, last_seq
            ),
            hint: "You can only edit the final message in a thread.".to_string(),
        });
    }

    // Update body (preserve all metadata)
    messages[msg_index].body = new_body.to_string();

    // Rewrite entire file
    let serialized = serialize_thread(&messages);

    // Truncate and write
    file.set_len(0).map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    file.seek(SeekFrom::Start(0)).map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;
    file.write_all(serialized.as_bytes()).map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    // Release lock
    file.unlock().map_err(|e| PaperworkError::IoContext {
        path: thread_path.clone(),
        source: e,
    })?;

    Ok(())
}
