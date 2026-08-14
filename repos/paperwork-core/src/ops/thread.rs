//! Thread operations: send, read, summary, meta, edit — all path-explicit.
//!
//! Used for Post threads (append-only group conversations).
//! `thread_send` auto-creates the file (and parent dirs) if it doesn't exist
//! and writes the preamble on first write (spec §5.7, invariant I9).
//! File locking (fs2) applies for send and edit.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use chrono::Utc;
use fs2::FileExt;
use regex::Regex;
use std::sync::LazyLock;

use crate::error::{PaperworkError, Result};
use crate::format::thread::{
    derive_message_refs, header_seq, parse_messages, parse_preamble, serialize_message,
    serialize_messages, serialize_preamble, validate_sender,
};
use crate::format::{fence_close_matches, fence_open_len, normalize_line_endings};
use crate::{Message, ThreadMeta, ThreadSummary};

/// Maximum message size (64 KB hard cap, invariant I3).
/// Applies to a single serialized message only — the preamble is exempt
/// (spec §5.7).
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

/// Size of reverse-scan buffer for finding last seq (64KB + 256B, spec §5.5).
const REVERSE_SCAN_SIZE: u64 = (64 * 1024 + 256) as u64;

/// Number of trailing messages quoted in `thread_summary` snippets (review n10).
const SNIPPET_COUNT: usize = 3;

/// Character budget of a single summary snippet before ellipsis (review n10).
const SNIPPET_CHAR_LIMIT: usize = 50;

/// Tail-scan seq regex (spec §5.5). Applied per line while scanning;
/// `[ \t]+` is the intra-line equivalent of the header's `\s+`.
static SEQ_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^##[ \t]+#(\d+)").expect("valid regex"));

/// Legacy v0.4 message-header heuristic (`### #N`, flush left). Used by the
/// unmigrated-thread write guard: a non-empty file with no v0.5 headers
/// (tail scan seq == 0) that still carries `### #N` lines is legacy data.
static LEGACY_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^###\s+#\d+").expect("valid regex"));

/// Send a message to a thread. Auto-creates the file and parent dirs if absent.
///
/// Returns the assigned sequence number.
///
/// Reference state is carried inside the body text only (owner ruling D2):
/// `@somebody` / `@#N` tokens are injected by the caller (CLI layer) into
/// `body` before the call; core derives `reply_to` / `mentions` from the
/// final body at read time and never persists them (spec §5.4, OQ-4).
///
/// # Arguments
/// * `path` - Explicit path to the thread file
/// * `from` - Sender name (validated, spec §5.6)
/// * `body` - Final message body (free-form Markdown; reference tokens
///   already merged in by the caller)
/// * `preamble` - Thread metadata for first write; used only when the file
///   is empty inside the lock (size == 0, invariant I9). Ignored when the
///   file is non-empty (spec §5.7, OQ-1).
pub fn thread_send(
    path: &Path,
    from: &str,
    body: &str,
    preamble: Option<&ThreadMeta>,
) -> Result<u64> {
    // Write-side sender validation (spec §5.6) — before touching the file.
    validate_sender(from)?;

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
    file.lock_exclusive()
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "another process may hold the lock; retry shortly".to_string(),
            example: String::new(),
        })?;

    // First-write gate: the in-lock file size is the single source of truth
    // (spec §5.7; an exists() pre-check would be TOCTOU).
    let file_empty = file
        .metadata()
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        })?
        .len()
        == 0;

    // Read last seq within lock
    let last_seq = read_last_seq_locked(&file, path)?;

    // Legacy-format write guard: v0.4 threads carry no v0.5 headers, so the
    // tail scan yields seq 0. Appending would silently produce a mixed-format
    // corrupt file (old `### #N` content + new `## #1` message) — refuse.
    // Legitimate preamble-only new files (no legacy traces) and empty files
    // (first-write branch) are unaffected.
    if !file_empty && last_seq == 0 && contains_legacy_headers(&file, path)? {
        file.unlock().ok();
        return Err(PaperworkError::Parse {
            message: "thread file contains legacy v0.4 message headers but no v0.5 message headers".to_string(),
            fix: "this file is in the v0.4 legacy format; v0.5 is not forward compatible - migrate it by hand per the CHANGELOG migration guide before writing".to_string(),
            example: "see CHANGELOG.md, [0.5.0] 'Migration guide (manual)', step 1 (post)".to_string(),
        });
    }

    let new_seq = last_seq
        .checked_add(1)
        .ok_or_else(|| PaperworkError::Validation {
            message: "thread seq exhausted".to_string(),
            fix: "start a new thread file".to_string(),
            example: String::new(),
        })?;

    // `reply_to` / `mentions` are derived from the final body text (D2);
    // serialization ignores them, keeping disk and model consistent.
    let (reply_to, mentions) = derive_message_refs(body, from);

    let msg = Message {
        seq: new_seq,
        sender: from.to_string(),
        timestamp: Utc::now(),
        reply_to,
        mentions,
        body: body.to_string(),
    };

    let serialized = serialize_message(&msg);

    // Check size limit (single message only; preamble exempt, spec §5.7)
    if serialized.len() > MAX_MESSAGE_SIZE {
        file.unlock().ok();
        return Err(PaperworkError::MessageTooLarge {
            size: serialized.len(),
            max: MAX_MESSAGE_SIZE,
            fix: "split into smaller messages".to_string(),
            example: String::new(),
        });
    }

    // First write: preamble + first message in one write_all (invariant I9).
    // Non-empty file: preamble parameters ignored (OQ-1).
    //
    // Append guard (review F1): serialize_message starts at a line boundary
    // (`## #N ...`), so the existing content must end with a newline. Files
    // written by this tool always do, but external edits may strip the final
    // newline; without this check the new header glues onto the previous
    // line and the message is silently swallowed into the prior body.
    // The last-byte probe returns a bool; no closure mutates outer state
    // (review n10).
    let needs_leading_newline = if file_empty {
        false
    } else {
        // Non-empty branch guarantees len > 0, so End(-1) is valid.
        let mut file_ref = &file;
        let last = file_ref
            .seek(SeekFrom::End(-1))
            .and_then(|_| {
                let mut last = [0u8; 1];
                file_ref.read_exact(&mut last).map(|_| last)
            })
            .map_err(|e| PaperworkError::IoContext {
                path: path.to_path_buf(),
                source: e,
                fix: "check that the file is readable".to_string(),
                example: String::new(),
            })?;
        last[0] != b'\n'
    };
    let payload = if file_empty {
        match preamble {
            Some(meta) => format!("{}{}", serialize_preamble(meta), serialized),
            None => serialized,
        }
    } else if needs_leading_newline {
        format!("\n{}", serialized)
    } else {
        serialized
    };

    // Single write() call for atomicity (invariant I4)
    let mut writer = &file;
    writer
        .write_all(payload.as_bytes())
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

/// Read the thread preamble metadata (spec §5.2).
///
/// A missing file yields the default meta (no error).
pub fn thread_meta(path: &Path) -> Result<ThreadMeta> {
    if !path.exists() {
        return Ok(ThreadMeta::default());
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
        fix: "check file permissions".to_string(),
        example: String::new(),
    })?;

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
            title: String::new(),
            message_count: 0,
            participants: Vec::new(),
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

    // Title from the preamble in the SAME pass (review M8): callers no
    // longer need a second full-file `thread_meta` walk.
    let title = parse_preamble(&content).title;

    let messages = parse_messages(&content)?;

    let message_count = messages.len() as u64;
    let last_sender = messages.last().map(|m| m.sender.clone());
    let last_timestamp = messages.last().map(|m| m.timestamp);

    // Participants derived from the sender set, deduplicated in
    // first-appearance order (spec §5.4, owner ruling D1).
    let mut participants: Vec<String> = Vec::new();
    for m in &messages {
        if !participants.contains(&m.sender) {
            participants.push(m.sender.clone());
        }
    }

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

/// Edit a message body in a thread (self-edit).
///
/// ONLY allowed if:
/// 1. The message was sent by `sender`
/// 2. It is the sender's most recent message
/// 3. It is the final message in the thread
///
/// Requires file lock for safe rewrite. The preamble (everything before the
/// first message header) is carried over byte-for-byte (spec §5.7, R5);
/// the new body is subject to the 64KB limit (R8) — on overflow the file
/// stays unchanged.
///
/// Crash-window note (spec §5.7): the in-lock truncate + rewrite can lose
/// the whole file on power loss / process kill; accepted (fs2 lock excludes
/// concurrent writers).
pub fn thread_edit(path: &Path, seq: u64, sender: &str, new_body: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Thread".to_string(),
            name: path.display().to_string(),
            fix: "cannot edit a non-existent thread".to_string(),
            example: format!(
                "paperwork post send {} --from <name> <body>",
                path.display()
            ),
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

    file.lock_exclusive()
        .map_err(|e| PaperworkError::IoContext {
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
            example: format!(
                "paperwork post send {} --from <name> <body>",
                path.display()
            ),
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
            example: format!(
                "paperwork post edit {} --seq {} --from {} <body>",
                path.display(),
                seq,
                msg.sender
            ),
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
            example: format!(
                "paperwork post edit {} --seq {} --from {} <body>",
                path.display(),
                sender_last_seq,
                sender
            ),
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
            example: format!(
                "paperwork post edit {} --seq {} --from {} <body>",
                path.display(),
                last_seq,
                sender
            ),
        });
    }

    // Update body (preserve all metadata)
    messages[msg_index].body = new_body.to_string();

    // 64KB guard on the edited message BEFORE truncating (R8): the file
    // stays unchanged on overflow.
    let edited_serialized = serialize_message(&messages[msg_index]);
    if edited_serialized.len() > MAX_MESSAGE_SIZE {
        file.unlock().ok();
        return Err(PaperworkError::MessageTooLarge {
            size: edited_serialized.len(),
            max: MAX_MESSAGE_SIZE,
            fix: "split into smaller messages".to_string(),
            example: String::new(),
        });
    }

    // Preamble bytes carried over verbatim from the ORIGINAL content
    // (R5 / invariant I9): everything before the first fence-aware message
    // header line, no re-serialization.
    let preamble_end = first_message_header_offset(&content).unwrap_or(0);
    let mut new_content: Vec<u8> = content.as_bytes()[..preamble_end].to_vec();
    new_content.extend_from_slice(serialize_messages(&messages).as_bytes());

    // Rewrite entire file (truncate + write within the lock)
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
    file.write_all(&new_content)
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

/// Byte offset of the first fence-aware message header line in raw content.
///
/// Iterates line boundaries exactly like `normalize_line_endings` does
/// (`\n`, `\r\n` and lone `\r` — spec §3.1 / invariant I11), accumulating
/// RAW byte offsets while applying the fence/header predicates to the
/// terminator-stripped line. This keeps the byte-offset view consistent
/// with the normalized view used by `parse_messages`; a `split('\n')` scan
/// would glue lone-`\r`-terminated lines to their successor and silently
/// lose the whole preamble (review B1).
fn first_message_header_offset(content: &str) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut open: Option<usize> = None;
    let mut start = 0usize;
    loop {
        // Find the next line boundary: `\n`, `\r\n` or lone `\r`.
        let mut end = start;
        while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
            end += 1;
        }
        // `\n`/`\r` are ASCII: the byte range is a valid char boundary.
        let line = &content[start..end];
        if let Some(n) = open {
            if fence_close_matches(line, n) {
                open = None;
            }
        } else if let Some(n) = fence_open_len(line) {
            open = Some(n);
        } else if header_seq(line).is_some() {
            // Same predicate as the parse side (`header_indices`, review
            // MJ-1): seq-0 and overflowing-seq H2s are preamble content,
            // so they never terminate the preamble carry-over range.
            return Some(start);
        }
        // Advance past the line terminator (`\r\n` counts 2 bytes,
        // lone `\r` / `\n` count 1).
        if end >= bytes.len() {
            break;
        }
        start = if bytes[end] == b'\r' && end + 1 < bytes.len() && bytes[end + 1] == b'\n' {
            end + 2
        } else {
            end + 1
        };
    }
    None
}

/// Whether the file content (read through the locked handle) contains legacy
/// v0.4 message headers (`### #N` lines).
///
/// Fence-aware (review mn-4): a `### #N` line inside a fenced code block of
/// preamble prose is quoted content, not a legacy header trace, so it must
/// not trigger the unmigrated-thread write refusal.
fn contains_legacy_headers(file: &File, path: &Path) -> Result<bool> {
    let mut file_ref = file;
    file_ref
        .seek(SeekFrom::Start(0))
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file handle validity".to_string(),
            example: String::new(),
        })?;
    let mut content = String::new();
    file_ref
        .read_to_string(&mut content)
        .map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
            fix: "check file integrity".to_string(),
            example: String::new(),
        })?;
    let content = normalize_line_endings(&content);
    let mut open: Option<usize> = None;
    for line in content.lines() {
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
        if LEGACY_HEADER_RE.is_match(line) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Read the last seq number from a thread file (within lock).
///
/// Reverse-scans the tail for efficiency (O(1) regardless of file size),
/// spec §5.5:
/// - buffer = last 64KB + 256B;
/// - incomplete first line dropped ONLY when `read_start > 0` and the byte
///   preceding the buffer is not `\n` (R7);
/// - fence open/close tracking within the buffer: candidate headers inside
///   an open fence are skipped (R6; the residual limitation of an unknown
///   fence parity before the buffer start is documented in spec §5.5).
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

    // Incomplete-first-line rule (R7): only when the buffer does not cover
    // the whole file.
    let mut scan: &[u8] = &buffer;
    if read_start > 0 {
        let mut prev = [0u8; 1];
        file_ref
            .seek(SeekFrom::Start(read_start - 1))
            .map_err(|e| PaperworkError::IoContext {
                path: path.to_path_buf(),
                source: e,
                fix: "check file handle validity".to_string(),
                example: String::new(),
            })?;
        file_ref
            .read_exact(&mut prev)
            .map_err(|e| PaperworkError::IoContext {
                path: path.to_path_buf(),
                source: e,
                fix: "check file integrity".to_string(),
                example: String::new(),
            })?;
        if prev[0] != b'\n' {
            scan = match buffer.iter().position(|&b| b == b'\n') {
                Some(pos) => &buffer[pos + 1..],
                None => &buffer[buffer.len()..],
            };
        }
    }

    let content = String::from_utf8_lossy(scan);

    // Fence-aware scan within the buffer (R6): candidate headers inside an
    // open fence are skipped.
    let mut last_seq = 0u64;
    let mut open: Option<usize> = None;
    for line in content.lines() {
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
        if SEQ_RE.is_match(line) {
            // header_seq re-check shares the parse-side predicate (review
            // n2): seq-0 pseudo-headers and overflowing seqs never reset
            // last_seq, exactly like `header_indices` on the read path.
            if let Some(seq) = header_seq(line) {
                last_seq = seq;
            }
        }
    }

    Ok(last_seq)
}
