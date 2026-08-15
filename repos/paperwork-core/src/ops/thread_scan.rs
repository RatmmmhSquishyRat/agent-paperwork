//! Low-level byte scans for thread files (T5 split of the historical
//! monolithic `ops/thread.rs`).
//!
//! Three responsibilities, all read-only byte-level probes:
//! - the legacy v0.4 write guard ([`contains_legacy_headers`]);
//! - the reverse tail scan for the last seq ([`read_last_seq_locked`],
//!   spec §5.5);
//! - the fence-aware preamble offset probe
//!   ([`first_message_header_offset`]) used by `thread_edit`'s preamble
//!   carry-over (spec §5.7, R5).
//!
//! Concurrency: every scan runs inside the caller's `LockedFile` window
//! (the send/edit locks); none of these functions acquires or releases a
//! lock on behalf of a foreign guard.
//!
//! Crash windows: none of these scans writes; the truncate+rewrite crash
//! window lives in the `thread_edit` orchestration (`super::thread`).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::thread::{header_seq, LEGACY_HEADER_RE_FMT};
use crate::format::{
    fence_close_matches, fence_open_len, for_each_outside_fence, normalize_line_endings,
};

/// Size of reverse-scan buffer for finding last seq (64KB + 256B, spec §5.5).
pub(super) const REVERSE_SCAN_SIZE: u64 = (64 * 1024 + 256) as u64;

// The header-regex family lives in `format/thread.rs` (T4 unification):
// the legacy-header heuristic is the shared `LEGACY_HEADER_RE_FMT` static,
// and the tail scan below re-checks candidates with the parse-side
// `header_seq` predicate directly (the historical `SEQ_RE` prefilter was
// redundant — `header_seq` is the single authoritative gate).

/// Byte offset of the first fence-aware message header line in raw content.
///
/// Iterates line boundaries exactly like `normalize_line_endings` does
/// (`\n`, `\r\n` and lone `\r` — spec §3.1 / invariant I11), accumulating
/// RAW byte offsets while applying the fence/header predicates to the
/// terminator-stripped line. This keeps the byte-offset view consistent
/// with the normalized view used by `parse_messages`; a `split('\n')` scan
/// would glue lone-`\r`-terminated lines to their successor and silently
/// lose the whole preamble (review B1).
///
/// T4 byte-level exemption: this loop deliberately does NOT migrate onto
/// the shared line scanners (`format/mod.rs`, the fence-policy authority —
/// the loop reuses its `fence_open_len` / `fence_close_matches` predicates)
/// because the scanners hand out line slices of normalized content, while
/// this function must return RAW byte offsets where `\r\n` counts 2 bytes.
pub(super) fn first_message_header_offset(content: &str) -> Option<usize> {
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
///
/// T4: converged onto the shared scanner family ([`for_each_outside_fence`]).
pub(super) fn contains_legacy_headers(file: &File, path: &Path) -> Result<bool> {
    let mut file_ref = file;
    file_ref
        .seek(SeekFrom::Start(0))
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file handle validity", ""))?;
    let mut content = String::new();
    file_ref
        .read_to_string(&mut content)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file integrity", ""))?;
    let content = normalize_line_endings(&content);
    let mut found = false;
    for_each_outside_fence(&content, |_i, line| {
        if LEGACY_HEADER_RE_FMT.is_match(line) {
            found = true;
            return false; // early stop
        }
        true
    });
    Ok(found)
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
///
/// T4 byte-level exemption: this loop deliberately does NOT migrate onto
/// the shared line scanners (`format/mod.rs`, the fence-policy authority —
/// the loop reuses its `fence_open_len` / `fence_close_matches` predicates)
/// because it scans an UNNORMALIZED byte buffer (`String::from_utf8_lossy`
/// over the raw tail, lone `\r` boundaries included) under the R7
/// first-line-drop rule; normalizing first would shift every byte offset
/// the R7 probe reasons about.
pub(super) fn read_last_seq_locked(file: &File, path: &Path) -> Result<u64> {
    let metadata = file
        .metadata()
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file handle validity", ""))?;

    let file_size = metadata.len();
    if file_size == 0 {
        return Ok(0);
    }

    let read_start = file_size.saturating_sub(REVERSE_SCAN_SIZE);
    let read_len = (file_size - read_start) as usize;

    let mut file_ref = file;
    file_ref
        .seek(SeekFrom::Start(read_start))
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file handle validity", ""))?;

    let mut buffer = vec![0u8; read_len];
    file_ref
        .read_exact(&mut buffer)
        .map_err(|e| PaperworkError::io_ctx(path, e, "check file integrity", ""))?;

    // Incomplete-first-line rule (R7): only when the buffer does not cover
    // the whole file.
    let mut scan: &[u8] = &buffer;
    if read_start > 0 {
        let mut prev = [0u8; 1];
        file_ref
            .seek(SeekFrom::Start(read_start - 1))
            .map_err(|e| PaperworkError::io_ctx(path, e, "check file handle validity", ""))?;
        file_ref
            .read_exact(&mut prev)
            .map_err(|e| PaperworkError::io_ctx(path, e, "check file integrity", ""))?;
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
        // header_seq is the single authoritative gate (T4): the historical
        // `SEQ_RE` prefilter was redundant — seq-0 pseudo-headers and
        // overflowing seqs never reset last_seq, exactly like
        // `header_indices` on the read path (review n2 / MJ-1).
        if let Some(seq) = header_seq(line) {
            last_seq = seq;
        }
    }

    Ok(last_seq)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;

    // ========================================================================
    // T4 differential corpus: pin the fence-aware legacy-header scan
    // semantics BEFORE the migration onto the shared scanner family; the
    // same corpus must pass unchanged afterwards.
    // ========================================================================

    fn t4_legacy(content: &str) -> bool {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        let mut f = File::create(&path).expect("create");
        f.write_all(content.as_bytes()).expect("write");
        let f = OpenOptions::new().read(true).open(&path).expect("open");
        contains_legacy_headers(&f, &path).expect("scan")
    }

    #[test]
    fn test_t4_contains_legacy_headers_differential_corpus() {
        // plain legacy header
        assert!(t4_legacy("### #1 alice (2026-01-01T00:00:00Z)"));
        // v0.5 content carries no legacy traces
        assert!(!t4_legacy("# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n"));
        // legacy header inside a fence is quoted content
        assert!(!t4_legacy("```md\n### #1 alice\n```\n"));
        // <= 3 space indented fence is recognized
        assert!(!t4_legacy("   ```\n### #1 alice\n   ```"));
        // 4-space indent: no fence, the legacy line stays visible
        assert!(t4_legacy("    ```\n### #1 alice\n    ```"));
        // tilde fences are not recognized
        assert!(t4_legacy("~~~\n### #1 alice\n~~~"));
        // unclosed fence swallows the tail
        assert!(!t4_legacy("```\n### #1 alice"));
        // nested backtick length: shorter run does not close the fence
        assert!(!t4_legacy("````\n### #1 alice\n```\n"));
        // CRLF input behaves like LF
        assert!(t4_legacy("```md\r\n```\r\n### #1 alice\r\n"));
        // empty file
        assert!(!t4_legacy(""));
        // indented legacy line is not flush-left (regex stance)
        assert!(!t4_legacy(" ### #1 alice"));
    }
}
