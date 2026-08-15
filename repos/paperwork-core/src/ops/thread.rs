//! Thread write operations: send and edit — all path-explicit (T5 split:
//! the read side lives in [`super::thread_read`], the byte-level scans in
//! [`super::thread_scan`]; the historical re-export surface below keeps
//! every `ops::thread::*` path unchanged).
//!
//! Used for Post threads (append-only group conversations).
//! `thread_send` auto-creates the file (and parent dirs) if it doesn't exist
//! and writes the preamble on first write (spec §5.7, invariant I9).
//!
//! Concurrency semantics: file locking (fs2) applies for send and edit; the
//! exclusive lock excludes concurrent writers for the whole read-modify-write
//! window, and every exit path releases it explicitly (P-1: the master
//! `locked_read_modify_write` stance, manual lock/unlock, stays the SSOT).
//! The lock-free readers in [`super::thread_read`] tolerate the
//! writer-exclusion stance because a torn read merely fails parsing like any
//! malformed file.
//!
//! Crash window (spec §5.7): `thread_edit`'s in-lock truncate + rewrite can
//! lose the whole file on power loss / process kill; accepted (the fs2 lock
//! excludes concurrent writers). `thread_send` only ever appends, so a crash
//! mid-write loses at most the new message.

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use chrono::Utc;
use fs2::FileExt;

use crate::error::{PaperworkError, Result};
use crate::format::check_single_line;
use crate::format::thread::{
    derive_message_refs, parse_messages, serialize_message, serialize_messages, serialize_preamble,
    validate_sender,
};
use crate::{Message, ThreadMeta};

use super::thread_scan::{
    contains_legacy_headers, find_message_sender_locked, first_message_header_offset,
    last_message_header_offset, read_last_seq_locked,
};

// Historical re-export surface (T5 split): every `ops::thread::*` path the
// CLI and the test suites use keeps resolving through this module, so the
// lib.rs public API is byte-for-byte unchanged.
pub use super::thread_read::{thread_meta, thread_read, thread_summary};

/// Maximum message size (64 KB hard cap, invariant I3).
/// Applies to a single serialized message only — the preamble is exempt
/// (spec §5.7).
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

// P-3/T5: the historical `SEQ_RE` tail-scan prefilter and the ops-side
// `LEGACY_HEADER_RE` twin were deleted here. `header_seq` (the parse-side
// predicate, review MJ-1) is the single authoritative tail-scan gate — the
// prefilter was redundant — and the legacy-header pattern now lives as the
// single definition `format::thread::LEGACY_HEADER_RE_FMT`. The
// `REVERSE_SCAN_SIZE` / `SNIPPET_*` constants moved to `thread_scan` /
// `thread_read` together with their consumers (T5 split).

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

    // Write-side injection guard (NEW-1): the preamble title is serialized
    // as a single H1 line; an embedded newline would inject structure.
    if let Some(meta) = preamble {
        check_single_line("thread title", &meta.title)?;
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            PaperworkError::io_ctx(
                parent.to_path_buf(),
                e,
                "check that the parent directory is writable",
                String::new(),
            )
        })?;
    }

    // Open file with append mode (creates if not exists)
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .read(true)
        .open(path)
        .map_err(|e| {
            PaperworkError::io_ctx(
                path.to_path_buf(),
                e,
                "check that the file path is accessible",
                String::new(),
            )
        })?;

    // Acquire exclusive lock (blocks concurrent writers)
    file.lock_exclusive().map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "another process may hold the lock; retry shortly",
            String::new(),
        )
    })?;

    // First-write gate: the in-lock file size is the single source of truth
    // (spec §5.7; an exists() pre-check would be TOCTOU).
    let file_empty = file
        .metadata()
        .map_err(|e| {
            PaperworkError::io_ctx(
                path.to_path_buf(),
                e,
                "check file handle validity",
                String::new(),
            )
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
            .map_err(|e| {
                PaperworkError::io_ctx(
                    path.to_path_buf(),
                    e,
                    "check that the file is readable",
                    String::new(),
                )
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
    writer.write_all(payload.as_bytes()).map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "check disk space and file permissions",
            String::new(),
        )
    })?;

    file.unlock().map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "check file handle validity",
            String::new(),
        )
    })?;

    Ok(new_seq)
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
/// Write shape (NEW-8 incremental rewrite): the edit target is ALWAYS the
/// final message, so when the on-disk prefix (verbatim preamble + earlier
/// messages) is byte-identical to its canonical re-serialization, the file
/// is truncated at the last message header and only the re-serialized final
/// message is appended — the earlier messages are never re-serialized nor
/// re-written. Non-canonical prefixes (hand-edited CRLF files,
/// whitespace-lenient headers, ...) take the historical full-rewrite
/// fallback; both paths are byte-identical (differential corpus pinned in
/// `ops_tests.rs`).
///
/// Crash-window note (spec §5.7): the in-lock truncate (+ append / rewrite)
/// can lose content on power loss / process kill; accepted (fs2 lock
/// excludes concurrent writers).
pub fn thread_edit(path: &Path, seq: u64, sender: &str, new_body: &str) -> Result<()> {
    if !path.exists() {
        return Err(PaperworkError::NotFound {
            resource: "Thread".to_string(),
            name: path.display().to_string(),
            fix: "cannot edit a non-existent thread".to_string(),
            example: format!(
                "paperwork post send {} --author alice --message \"Hello\"",
                path.display()
            ),
        });
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| {
            PaperworkError::io_ctx(
                path.to_path_buf(),
                e,
                "check file permissions",
                String::new(),
            )
        })?;

    file.lock_exclusive().map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "another process may hold the lock; retry shortly",
            String::new(),
        )
    })?;

    // Read content through the locked file handle
    let mut content = String::new();
    file.seek(SeekFrom::Start(0)).map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "check file handle validity",
            String::new(),
        )
    })?;
    file.read_to_string(&mut content).map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "check file permissions",
            String::new(),
        )
    })?;

    let mut messages = parse_messages(&content)?;

    if messages.is_empty() {
        file.unlock().ok();
        return Err(PaperworkError::NotFound {
            resource: "Message".to_string(),
            name: format!("#{}", seq),
            fix: "thread is empty; send a message first".to_string(),
            example: format!(
                "paperwork post send {} --author alice --message \"Hello\"",
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
                "paperwork post edit {} --author {} --seq {} --message \"corrected body\"",
                path.display(),
                msg.sender,
                seq
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
                "paperwork post edit {} --author {} --seq {} --message \"corrected body\"",
                path.display(),
                sender,
                sender_last_seq
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
                "paperwork post edit {} --author {} --seq {} --message \"corrected body\"",
                path.display(),
                sender,
                last_seq
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

    // NEW-8 incremental rewrite: the canonical serialization of the earlier
    // messages doubles as the equivalence probe — when the on-disk region
    // between the preamble and the last header matches it byte-for-byte,
    // that region can stay on disk untouched.
    let prefix_serialized = serialize_messages(&messages[..messages.len() - 1]);
    let incremental_offset = last_message_header_offset(&content).filter(|&last_header_start| {
        preamble_end <= last_header_start
            && content.as_bytes().get(preamble_end..last_header_start)
                == Some(prefix_serialized.as_bytes())
    });

    match incremental_offset {
        Some(offset) => {
            // Truncate at the last message header and append the
            // re-serialized final message.
            file.set_len(offset as u64).map_err(|e| {
                PaperworkError::io_ctx(
                    path.to_path_buf(),
                    e,
                    "check file permissions",
                    String::new(),
                )
            })?;
            file.seek(SeekFrom::Start(offset as u64)).map_err(|e| {
                PaperworkError::io_ctx(
                    path.to_path_buf(),
                    e,
                    "check file handle validity",
                    String::new(),
                )
            })?;
            file.write_all(edited_serialized.as_bytes()).map_err(|e| {
                PaperworkError::io_ctx(
                    path.to_path_buf(),
                    e,
                    "check disk space and file permissions",
                    String::new(),
                )
            })?;
        }
        None => {
            // Fallback for non-canonical on-disk prefixes: the historical
            // full rewrite (verbatim preamble + re-serialized message list).
            let mut new_content: Vec<u8> = content.as_bytes()[..preamble_end].to_vec();
            new_content.extend_from_slice(prefix_serialized.as_bytes());
            new_content.extend_from_slice(edited_serialized.as_bytes());

            // Rewrite entire file (truncate + write within the lock).
            file.set_len(0).map_err(|e| {
                PaperworkError::io_ctx(
                    path.to_path_buf(),
                    e,
                    "check file permissions",
                    String::new(),
                )
            })?;
            file.seek(SeekFrom::Start(0)).map_err(|e| {
                PaperworkError::io_ctx(
                    path.to_path_buf(),
                    e,
                    "check file handle validity",
                    String::new(),
                )
            })?;
            file.write_all(&new_content).map_err(|e| {
                PaperworkError::io_ctx(
                    path.to_path_buf(),
                    e,
                    "check disk space and file permissions",
                    String::new(),
                )
            })?;
        }
    }

    file.unlock().map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "check file handle validity",
            String::new(),
        )
    })?;

    Ok(())
}

/// Look up the sender of message `seq` in a thread file (NEW-12).
///
/// Bounded reverse tail scan (spec §5.5) instead of a whole-file parse:
/// the caller only needs one header's sender field, so the same 64KB +
/// 256B window the send path already scans is reused. Runs under an
/// exclusive lock; every exit path releases it explicitly (P-1 master
/// lock stance).
///
/// Returns `Ok(None)` when the file carries no fence-aware header with
/// that seq inside the tail window (missing seq, or a target beyond the
/// window — the residual limitation documented in spec §5.5); callers
/// treat it like a missing target. `Err` only for I/O / lock failures.
pub fn find_message_sender(path: &Path, seq: u64) -> Result<Option<String>> {
    let file = OpenOptions::new().read(true).open(path).map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "check that the file exists and is readable",
            String::new(),
        )
    })?;

    file.lock_exclusive().map_err(|e| {
        PaperworkError::io_ctx(
            path.to_path_buf(),
            e,
            "another process may hold the lock; retry shortly",
            String::new(),
        )
    })?;

    let result = find_message_sender_locked(&file, path, seq);
    file.unlock().ok();
    result
}
