//! Thread operations: send, read, summary, edit — all path-explicit.
//!
//! Used for Post/GDM threads (append-only group conversations).
//! `thread_send` auto-creates the file (and parent dirs) if it doesn't exist.
//! File locking (fs2) applies for send and edit.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use chrono::Utc;
use fs2::FileExt;
use regex::Regex;
use std::sync::LazyLock;

use crate::error::{PaperworkError, Result};
use crate::format::thread::{parse_messages, serialize_message, serialize_thread};
use crate::{Message, ThreadSummary};

/// Maximum message size (64 KB hard cap).
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

/// Size of reverse-scan buffer for finding last seq.
const REVERSE_SCAN_SIZE: u64 = (64 * 1024 + 256) as u64;

/// Regex for extracting seq from message header.
static SEQ_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"### #(\d+) ").expect("valid regex"));

/// Send a message to a thread. Auto-creates the file and parent dirs if absent.
///
/// Returns the assigned sequence number.
///
/// # Arguments
/// * `path` - Explicit path to the thread file
/// * `from` - Sender name
/// * `to` - Recipient names (empty = broadcast / "all")
/// * `body` - Message body (free-form Markdown)
/// * `reply_to` - Optional seq being replied to
/// * `mentions` - Names mentioned in the message (for notification hooks)
pub fn thread_send(
    path: &Path,
    from: &str,
    to: &[String],
    body: &str,
    reply_to: Option<u64>,
    mentions: &[String],
) -> Result<u64> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PaperworkError::IoContext {
            path: parent.to_path_buf(),
            source: e,
            fix: "check that the parent directory is writable".to_string(),
            example: String::new(),
        })?;
    }

    // Open file with append mode (creates if not exists)
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .read(true)
        .open(path)
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check that the file path is accessible".to_string(),
            example: String::new(),
        })?;

    // Acquire exclusive lock (blocks concurrent writers)
    file.lock_exclusive().map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "another process may hold the lock; retry shortly".to_string(),
        example: String::new(),
    })?;

    // Read last seq within lock
    let last_seq = read_last_seq_locked(&file, path)?;
    let new_seq = last_seq + 1;

    let msg = Message {
        seq: new_seq,
        sender: from.to_string(),
        timestamp: Utc::now(),
        to: to.to_vec(),
        reply_to,
        mentions: mentions.to_vec(),
        body: body.to_string(),
    };

    let serialized = serialize_message(&msg);

    // Check size limit
    if serialized.len() > MAX_MESSAGE_SIZE {
        file.unlock().ok();
        return Err(PaperworkError::MessageTooLarge {
            size: serialized.len(),
            max: MAX_MESSAGE_SIZE,
            fix: "split into smaller messages".to_string(),
            example: String::new(),
        });
    }

    // Single write() call for atomicity
    let mut writer = &file;
    writer
        .write_all(serialized.as_bytes())
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check disk space and file permissions".to_string(),
            example: String::new(),
        })?;

    file.unlock().map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file handle validity".to_string(),
        example: String::new(),
    })?;

    Ok(new_seq)
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
            example: format!("paperwork post send {} --author alice --message \"Hello\"", path.display()),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;

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
            message_count: 0,
            last_sender: None,
            last_timestamp: None,
            snippets: Vec::new(),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;

    let messages = parse_messages(&content)?;

    let message_count = messages.len() as u64;
    let last_sender = messages.last().map(|m| m.sender.clone());
    let last_timestamp = messages.last().map(|m| m.timestamp);

    // Snippets from last 3 messages (chronological order)
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
        thread_path: path.display().to_string(),
        message_count,
        last_sender,
        last_timestamp,
        snippets,
    })
}

/// Edit a message body in a thread (self-edit).
///
/// ONLY allowed if:
/// 1. The message was sent by `sender`
/// 2. It is the sender's most recent message
/// 3. It is the final message in the thread
///
/// Requires file lock for safe rewrite.
pub fn thread_edit(path: &Path, seq: u64, sender: &str, new_body: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Thread".to_string(),
            name: path.display().to_string(),
            fix: "cannot edit a non-existent thread".to_string(),
            example: format!("paperwork post send {} --author alice --message \"Hello\"", path.display()),
        });
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file permissions".to_string(),
            example: String::new(),
        })?;

    file.lock_exclusive().map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "another process may hold the lock; retry shortly".to_string(),
        example: String::new(),
    })?;

    // Read content through the locked file handle
    let mut content = String::new();
    file.seek(SeekFrom::Start(0))
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        })?;
    file.read_to_string(&mut content)
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file permissions".to_string(),
            example: String::new(),
        })?;

    let mut messages = parse_messages(&content)?;

    if messages.is_empty() {
        file.unlock().ok();
        return Err(PaperworkError::NotFound {
            resource: "Message".to_string(),
            name: format!("#{}", seq),
            fix: "thread is empty; send a message first".to_string(),
            example: format!("paperwork post send {} --author alice --message \"Hello\"", path.display()),
        });
    }

    // Find the message to edit
    let msg_index = match messages.iter().position(|m| m.seq == seq) {
        Some(idx) => idx,
        None => {
            file.unlock().ok();
            return Err(PaperworkError::NotFound {
                resource: "Message".to_string(),
                name: format!("#{}", seq),
                fix: "check the seq number with `paperwork post read`".to_string(),
                example: format!("paperwork post read {}", path.display()),
            });
        }
    };

    let msg = &messages[msg_index];

    // Check ownership
    if msg.sender != sender {
        file.unlock().ok();
        return Err(PaperworkError::NotAllowed {
            operation: "thread_edit".to_string(),
            reason: format!(
                "Message #{} was sent by '{}', not '{}'",
                seq, msg.sender, sender
            ),
            fix: "you can only edit your own messages".to_string(),
            example: format!("paperwork post edit {} --author {} --seq {} --message \"corrected body\"", path.display(), msg.sender, seq),
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
            operation: "thread_edit".to_string(),
            reason: format!(
                "Message #{} is not your most recent message (your last is #{})",
                seq, sender_last_seq
            ),
            fix: "you can only edit your most recent message".to_string(),
            example: format!("paperwork post edit {} --author {} --seq {} --message \"corrected body\"", path.display(), sender, sender_last_seq),
        });
    }

    // Check if it's the final message in thread
    let last_seq = messages.last().map(|m| m.seq).unwrap_or(0);
    if seq != last_seq {
        file.unlock().ok();
        return Err(PaperworkError::NotAllowed {
            operation: "thread_edit".to_string(),
            reason: format!(
                "Message #{} is not the final message in thread (last is #{})",
                seq, last_seq
            ),
            fix: "you can only edit the final message in a thread".to_string(),
            example: format!("paperwork post edit {} --author {} --seq {} --message \"corrected body\"", path.display(), sender, last_seq),
        });
    }

    // Update body (preserve all metadata)
    messages[msg_index].body = new_body.to_string();

    // Rewrite entire file
    let serialized = serialize_thread(&messages);

    file.set_len(0).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        })?;
    file.write_all(serialized.as_bytes())
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check disk space and file permissions".to_string(),
            example: String::new(),
        })?;

    file.unlock().map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file handle validity".to_string(),
        example: String::new(),
    })?;

    Ok(())
}

/// Read the last seq number from a thread file (within lock).
///
/// Reverse-scans the tail for efficiency (O(1) regardless of file size).
fn read_last_seq_locked(file: &File, path: &Path) -> Result<u64> {
    let metadata = file.metadata().map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file handle validity".to_string(),
        example: String::new(),
    })?;

    let file_size = metadata.len();
    if file_size == 0 {
        return Ok(0);
    }

    let read_start = file_size.saturating_sub(REVERSE_SCAN_SIZE);
    let read_len = (file_size - read_start) as usize;

    let mut file_ref = file;
    file_ref
        .seek(SeekFrom::Start(read_start))
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        })?;

    let mut buffer = vec![0u8; read_len];
    file_ref
        .read_exact(&mut buffer)
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file integrity".to_string(),
            example: String::new(),
        })?;

    let content = String::from_utf8_lossy(&buffer);

    let mut last_seq = 0u64;
    for caps in SEQ_RE.captures_iter(&content) {
        if let Ok(seq) = caps[1].parse::<u64>() {
            last_seq = seq;
        }
    }

    Ok(last_seq)
}
