//! Integration tests for the stateless, path-explicit operations layer.
//!
//! All fixtures use Managed File Format v2 (spec.md); test mapping per
//! tdd.md §3 (T-OPS-01..30).

use std::fs;
use std::sync::{Arc, Barrier};
use std::thread as std_thread;

use tempfile::tempdir;

use paperwork_core::ops::{contacts, manifest, profile, thread};
use paperwork_core::{PaperworkError, ThreadMeta, VerifyResult};

/// Reverse-scan buffer size (spec §5.5: 64KB + 256B).
const SCAN: u64 = 64 * 1024 + 256;

fn meta(title: &str) -> ThreadMeta {
    ThreadMeta {
        title: title.to_string(),
    }
}

/// Manual message serialization with a fixed timestamp (for size-precise
/// fixtures). Matches the spec §5.9 canonical shape.
fn manual_msg(seq: u64, sender: &str, ts: &str, body: &str) -> String {
    format!(
        "## #{} {} ({})\n\n```md\n{}\n```\n\n",
        seq, sender, ts, body
    )
}

// ============================================================================
// Profile Ops Tests (T-OPS-01..06)
// ============================================================================

#[test]
fn create_profile_writes_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.profile.md");

    profile::create_profile(&path, "alice", "gpt-4", "Test agent").expect("create_profile failed");

    assert!(path.is_file());
    let content = fs::read_to_string(&path).expect("read failed");
    assert!(content.contains("# alice"));
    assert!(content.contains("- model: gpt-4"));
    assert!(content.contains("Test agent"));
    // empty scope → whole section omitted; no legacy constructs
    assert!(!content.contains("## Scope"));
    assert!(!content.contains('—'));
    assert!(!content.contains("- Description:"));
}

#[test]
fn create_profile_creates_parent_dirs() {
    let dir = tempdir().expect("tempdir");
    let path = dir
        .path()
        .join("nested")
        .join("deep")
        .join("alice.profile.md");

    profile::create_profile(&path, "alice", "gpt-4", "").expect("create_profile failed");

    assert!(path.is_file());
}

#[test]
fn create_profile_rejects_overwrite() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.profile.md");

    profile::create_profile(&path, "alice", "gpt-4", "").expect("first create");
    let result = profile::create_profile(&path, "alice", "gpt-4", "");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn show_profile_reads_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.profile.md");

    // new-format fixture (spec §4.1)
    fs::write(
        &path,
        "# alice\n\nParser module implementer\n\n- model: gpt-4o\n\n## Scope\n\n- read: src/**\n- write: src/parser/**\n",
    )
    .expect("write");

    let p = profile::show_profile(&path).expect("show_profile failed");
    assert_eq!(p.name, "alice");
    assert_eq!(p.model, "gpt-4o");
    assert_eq!(p.description, "Parser module implementer");
    assert_eq!(p.scope_read, vec!["src/**"]);
    assert_eq!(p.scope_write, vec!["src/parser/**"]);
}

#[test]
fn show_profile_not_found() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("nonexistent.profile.md");

    let result = profile::show_profile(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn edit_profile_updates_fields() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.profile.md");

    profile::create_profile(&path, "alice", "gpt-4", "Original").expect("create failed");

    profile::edit_profile(
        &path,
        Some("claude-3"),
        Some("Updated description"),
        Some(vec!["docs/**".to_string()]),
        None,
        Some(vec!["src/core/**".to_string()]),
    )
    .expect("edit_profile failed");

    let p = profile::show_profile(&path).expect("show failed");
    assert_eq!(p.model, "claude-3");
    assert_eq!(p.description, "Updated description");
    assert_eq!(p.scope_read, vec!["docs/**"]);
    assert_eq!(p.scope_owns, vec!["src/core/**"]);
    assert!(p.scope_write.is_empty());

    // on-disk scope is an attribute-line list (R3)
    let content = fs::read_to_string(&path).expect("read");
    assert!(content.contains("## Scope"));
    assert!(content.contains("- read: docs/**"));
    assert!(content.contains("- owns: src/core/**"));
    assert!(!content.contains("| ")); // no GFM table
}

// ============================================================================
// Thread Ops Tests (T-OPS-07..18, 26)
// ============================================================================

#[test]
fn thread_send_creates_file_and_returns_seq() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("standup.post.md");

    let m = meta("Daily Standup");
    let seq = thread::thread_send(&path, "alice", "@bob Hello, Bob!", Some(&m))
        .expect("thread_send failed");

    // seq 1 is the first real message (no system message, spec §5.7)
    assert_eq!(seq, 1);
    assert!(path.is_file());

    let content = fs::read_to_string(&path).expect("read");
    assert!(content.starts_with("# Daily Standup\n"));
    // D1: preamble carries the H1 title only — no participants line
    assert!(!content.contains("- participants:"));
    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 1);
}

#[test]
fn thread_send_creates_parent_dirs() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.dm").join("bob.post.md");

    let seq = thread::thread_send(&path, "alice", "Hi", None).expect("thread_send failed");

    assert_eq!(seq, 1);
    assert!(path.is_file());
}

#[test]
fn thread_send_increments_seq() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    let m = meta("t");
    for i in 0..3 {
        let sender = if i % 2 == 0 { "alice" } else { "bob" };
        let seq = thread::thread_send(&path, sender, &format!("Message {}", i + 1), Some(&m))
            .expect("thread_send failed");
        assert_eq!(seq, (i + 1) as u64);
    }

    let content = fs::read_to_string(&path).expect("read");
    assert!(content.contains("## #1 alice ("));
    assert!(content.contains("## #2 bob ("));
    assert!(content.contains("## #3 alice ("));

    let messages = thread::thread_read(&path, None, None).expect("read failed");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].seq, 1);
    assert_eq!(messages[1].seq, 2);
    assert_eq!(messages[2].seq, 3);
}

#[test]
fn thread_read_range_subset() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    for i in 1..=5 {
        thread::thread_send(&path, "alice", &format!("Msg {}", i), None).expect("send failed");
    }

    let messages = thread::thread_read(&path, Some(2), Some(4)).expect("read failed");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].seq, 2);
    assert_eq!(messages[2].seq, 4);
}

#[test]
fn thread_read_not_found() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("nonexistent.post.md");

    let result = thread::thread_read(&path, None, None);
    assert!(result.is_err());
}

#[test]
fn thread_summary_correct() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    let m = meta("t");
    thread::thread_send(&path, "alice", "First", Some(&m)).expect("send");
    thread::thread_send(&path, "bob", "Second", Some(&m)).expect("send");
    thread::thread_send(&path, "alice", "Third", Some(&m)).expect("send");

    let summary = thread::thread_summary(&path).expect("summary failed");
    assert_eq!(summary.message_count, 3);
    // M8: the preamble title rides along in the same parse pass
    assert_eq!(summary.title, "t");
    // D1: participants derived from senders, first-appearance order
    assert_eq!(summary.participants, vec!["alice", "bob"]);
    assert_eq!(summary.last_sender, Some("alice".to_string()));
    assert!(!summary.snippets.is_empty());
}

#[test]
fn thread_summary_empty_for_missing_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.post.md");

    let summary = thread::thread_summary(&path).expect("summary failed");
    assert_eq!(summary.message_count, 0);
    assert_eq!(summary.title, "");
    assert_eq!(summary.last_sender, None);
}

#[test]
fn thread_send_body_text_refs_derived() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    thread::thread_send(&path, "alice", "Hello", None).expect("send");
    // D2 / OQ-4: reference tokens are injected into the body by the caller
    // (CLI layer); core derives reply_to / mentions at read time.
    thread::thread_send(&path, "bob", "@#1 @alice Reply", None).expect("send");

    let content = fs::read_to_string(&path).expect("read");
    // no attribute lines on disk (D2)
    assert!(!content.contains("- reply-to:"));
    assert!(!content.contains("- to:"));
    assert!(!content.contains("- mentions:"));

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages[1].reply_to, Some(1));
    assert_eq!(messages[1].mentions, vec!["alice"]);
}

#[test]
fn concurrent_thread_send_safety() {
    let dir = tempdir().expect("tempdir");
    let path = Arc::new(dir.path().join("thread.post.md"));
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    for i in 0..10 {
        let path = Arc::clone(&path);
        let barrier = Arc::clone(&barrier);

        let handle = std_thread::spawn(move || {
            barrier.wait();
            thread::thread_send(
                &path,
                &format!("agent{}", i),
                &format!("Concurrent message {}", i),
                None,
            )
        });
        handles.push(handle);
    }

    for handle in handles {
        handle
            .join()
            .expect("thread panicked")
            .expect("send failed");
    }

    let messages = thread::thread_read(&path, None, None).expect("read failed");
    assert_eq!(messages.len(), 10);

    // seqs 1..=10 with no gaps
    for (i, msg) in messages.iter().enumerate() {
        assert_eq!(msg.seq, (i + 1) as u64);
    }
    // bodies intact
    for i in 0..10 {
        let needle = format!("Concurrent message {}", i);
        assert!(messages.iter().any(|m| m.body == needle));
    }
}

#[test]
fn thread_meta_reads_preamble() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    // missing file → default meta, no error
    let m = thread::thread_meta(&path).expect("meta on missing file");
    assert_eq!(m, ThreadMeta::default());

    let m = meta("Daily Standup");
    thread::thread_send(&path, "alice", "hi", Some(&m)).expect("send");

    let read = thread::thread_meta(&path).expect("meta failed");
    assert_eq!(read.title, "Daily Standup");
}

#[test]
fn thread_send_rejects_invalid_sender() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    for bad in ["two words", "bob(x)", "line\nbreak", ""] {
        let result = thread::thread_send(&path, bad, "body", None);
        assert!(result.is_err(), "sender {:?} must be rejected", bad);
        assert_eq!(result.unwrap_err().category(), "validation");
    }
    // file never created
    assert!(!path.exists());
}

#[test]
fn thread_send_rejects_oversized() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    thread::thread_send(&path, "alice", "seed", None).expect("seed");
    let before = fs::read_to_string(&path).expect("read");

    let huge = "x".repeat(70_000);
    let result = thread::thread_send(&path, "alice", &huge, None);
    let err = result.expect_err("must reject oversized");
    assert!(matches!(err, PaperworkError::MessageTooLarge { .. }));

    // file did not grow
    let after = fs::read_to_string(&path).expect("read");
    assert_eq!(before, after);
}

#[test]
fn thread_send_on_preamble_only_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    // existing preamble-only file (handwritten)
    fs::write(
        &path,
        "# Custom Title\n\nHandwritten prose.\n\n- participants: carol\n",
    )
    .expect("write");

    // --title is ignored for a non-empty file (OQ-1); the historical
    // `- participants:` line is plain ignored preamble content (D1)
    let m = meta("Other Title");
    let seq = thread::thread_send(&path, "alice", "first real message", Some(&m)).expect("send");
    assert_eq!(seq, 1);

    let content = fs::read_to_string(&path).expect("read");
    // original preamble untouched
    assert!(content.starts_with("# Custom Title\n\nHandwritten prose.\n\n- participants: carol\n"));
    assert!(!content.contains("Other Title"));

    let read = thread::thread_meta(&path).expect("meta");
    assert_eq!(read.title, "Custom Title");
}

// ============================================================================
// Thread Edit Tests (T-OPS-19, 20, 27 + retained)
// ============================================================================

#[test]
fn thread_edit_own_message() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    thread::thread_send(&path, "alice", "Original body", None).expect("send");

    thread::thread_edit(&path, 1, "alice", "Edited body").expect("edit failed");

    let messages = thread::thread_read(&path, Some(1), Some(1)).expect("read");
    assert_eq!(messages[0].body, "Edited body");
    assert_eq!(messages[0].sender, "alice");
}

#[test]
fn thread_edit_rejects_other_sender() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    thread::thread_send(&path, "alice", "Alice's message", None).expect("send");

    let result = thread::thread_edit(&path, 1, "bob", "Hacked!");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("sent by 'alice', not 'bob'"));
}

#[test]
fn thread_edit_constraints() {
    let dir = tempdir().expect("tempdir");

    // not the sender's most recent message
    let path = dir.path().join("a.post.md");
    thread::thread_send(&path, "alice", "Alice 1", None).expect("send");
    thread::thread_send(&path, "bob", "Bob 1", None).expect("send");
    thread::thread_send(&path, "alice", "Alice 2", None).expect("send");
    let err = thread::thread_edit(&path, 1, "alice", "Edited").unwrap_err();
    assert!(err.to_string().contains("not your most recent"));

    // sender's most recent, but not the final message
    let path = dir.path().join("b.post.md");
    thread::thread_send(&path, "alice", "Alice 1", None).expect("send");
    thread::thread_send(&path, "bob", "Bob 1", None).expect("send");
    let err = thread::thread_edit(&path, 1, "alice", "Edited").unwrap_err();
    assert!(err.to_string().contains("not the final message"));

    // not found thread
    let path = dir.path().join("missing.post.md");
    assert!(thread::thread_edit(&path, 1, "alice", "test").is_err());
}

#[test]
fn thread_edit_preserves_preamble_verbatim() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    // handwritten preamble: description prose + an extra H2 section
    let preamble = "# Standup\n\nHandwritten description prose.\n\n## Notes\n\nExtra section content.\n\n- participants: alice, bob\n\n";
    let msg1 = manual_msg(1, "alice", "2026-01-15T10:30:00Z", "first");
    let msg2 = manual_msg(2, "bob", "2026-01-15T10:31:00Z", "second");
    fs::write(&path, format!("{}{}{}", preamble, msg1, msg2)).expect("write");

    thread::thread_edit(&path, 2, "bob", "edited second").expect("edit");

    let content = fs::read_to_string(&path).expect("read");
    // byte-for-byte preamble preservation (R5)
    assert!(content.starts_with(preamble));
    assert!(content.contains("edited second"));
    assert!(content.contains("## Notes\n\nExtra section content."));

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].body, "first");
    assert_eq!(messages[1].body, "edited second");
}

#[test]
fn thread_edit_rejects_oversized() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    thread::thread_send(&path, "alice", "seed", None).expect("send");
    let before = fs::read_to_string(&path).expect("read");

    let huge = "y".repeat(70_000);
    let err = thread::thread_edit(&path, 1, "alice", &huge).expect_err("must reject");
    assert!(matches!(err, PaperworkError::MessageTooLarge { .. }));

    // file unchanged
    assert_eq!(before, fs::read_to_string(&path).expect("read"));
}

// B1 regression: lone `\r` between preamble and first header (invariant I11
// input class) must not lose the preamble on edit.
#[test]
fn thread_edit_preserves_preamble_lone_cr() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("cr.post.md");

    // preamble terminated by a lone `\r` right before the first header
    let content = "# Title\r## #1 alice (2026-01-15T10:30:00Z)\n\n```markdown\nold\n```\n";
    fs::write(&path, content).expect("write");

    thread::thread_edit(&path, 1, "alice", "newbody").expect("edit");

    let after = fs::read_to_string(&path).expect("read");
    // preamble bytes carried over verbatim (byte-for-byte, R5 / I9)
    assert!(after.starts_with("# Title\r"), "preamble lost: {:?}", after);
    assert!(after.contains("newbody"));

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].body, "newbody");
}

// MJ-1 regression: preamble pseudo-headers (seq-0 and overflowing-seq H2s)
// are preamble content per the parse-side predicate — the edit carry-over
// boundary must agree, or those lines plus following prose are silently
// deleted.
#[test]
fn thread_edit_preserves_preamble_pseudo_headers() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("pseudo.post.md");

    // Preamble carrying BOTH pseudo-header shapes, each followed by prose.
    let preamble = "# t\n\n## #0 alice (2026-01-15T10:30:00Z)\n\nprose after the seq-0 pseudo header\n\n## #99999999999999999999999999 mallory (2026-01-15T10:30:00Z)\n\nprose after the overflow pseudo header\n\n";
    let content = format!(
        "{}{}",
        preamble,
        manual_msg(1, "alice", "2026-01-15T10:30:00Z", "original")
    );
    fs::write(&path, content).expect("write");

    // parse-side sanity: both H2s fell into the preamble
    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 1);

    thread::thread_edit(&path, 1, "alice", "edited").expect("edit");

    let after = fs::read_to_string(&path).expect("read");
    // byte-for-byte preamble preservation incl. pseudo headers and prose
    assert!(after.starts_with(preamble), "preamble lost: {:?}", after);
    assert!(after.contains("prose after the seq-0 pseudo header"));
    assert!(after.contains("prose after the overflow pseudo header"));
    assert!(after.contains("edited"));

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].body, "edited");
}

// B1 regression variant: the preamble fence's closing line glued to the
// following content by a lone `\r`.
#[test]
fn thread_edit_preserves_preamble_fence_close_lone_cr() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("crfence.post.md");

    let content = "# t\n\n```markdown\nexample\n```\r## #1 alice (2026-01-15T10:30:00Z)\n\n```markdown\nold\n```\n";
    fs::write(&path, content).expect("write");

    thread::thread_edit(&path, 1, "alice", "newbody").expect("edit");

    let after = fs::read_to_string(&path).expect("read");
    assert!(
        after.starts_with("# t\n\n```markdown\nexample\n```\r"),
        "preamble lost: {:?}",
        after
    );
    assert!(after.contains("newbody"));

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].body, "newbody");
}

// M2 regression: sending into an unmigrated v0.4 thread is refused and the
// file stays untouched.
#[test]
fn thread_send_rejects_legacy_v04_thread() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("legacy.post.md");

    // v0.4 shape: `---` boundaries + `### #N sender · ts` headers
    let legacy = "---\n\n### #1 alice \u{00B7} 2026-08-01T19:38:22Z\n\n- To: all\n\n````markdown\nold body\n````\n\n---\n\n### #2 bob \u{00B7} 2026-08-01T19:39:00Z\n\n- To: all\n\n````markdown\nsecond\n````\n";
    fs::write(&path, legacy).expect("write");

    let err = thread::thread_send(&path, "bob", "new message", None)
        .expect_err("legacy thread must be rejected");
    assert_eq!(err.category(), "format");
    assert!(err.fix().contains("v0.4 legacy format"));
    assert!(err.fix().contains("CHANGELOG migration guide"));

    // file byte-for-byte unchanged
    assert_eq!(fs::read_to_string(&path).expect("read"), legacy);
}

// M2 regression: a legitimate preamble-only new file (no legacy traces)
// accepts sends normally.
#[test]
fn thread_send_preamble_only_file_ok() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("preamble.post.md");

    let preamble = "# Standup\n\n- participants: alice\n";
    fs::write(&path, preamble).expect("write");

    let seq = thread::thread_send(&path, "alice", "hello", None)
        .expect("preamble-only file accepts sends");
    assert_eq!(seq, 1);

    let content = fs::read_to_string(&path).expect("read");
    assert!(content.starts_with(preamble));
    assert!(content.contains("## #1 alice"));
}

// mn-4 regression: a `### #N` line INSIDE a preamble fence is quoted
// content, not a legacy v0.4 trace — the write refusal must not trigger.
#[test]
fn thread_send_allows_legacy_shaped_line_inside_preamble_fence() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("fenced.post.md");

    // preamble-only file whose prose fence quotes a legacy-shaped line
    let preamble = "# Standup\n\nExample of the old format, quoted:\n\n```markdown\n### #1 alice (legacy example)\n```\n";
    fs::write(&path, preamble).expect("write");

    let seq = thread::thread_send(&path, "alice", "first real message", None)
        .expect("fenced legacy-shaped line must not block the send");
    assert_eq!(seq, 1);

    let content = fs::read_to_string(&path).expect("read");
    assert!(content.starts_with(preamble));
    assert!(content.contains("## #1 alice"));
}

// ============================================================================
// Concurrency: first write (T-OPS-21, 29, 30)
// ============================================================================

#[test]
fn concurrent_first_write_single_preamble() {
    let dir = tempdir().expect("tempdir");
    let path = Arc::new(dir.path().join("thread.post.md"));
    let m = Arc::new(meta("Race Title"));
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = vec![];

    for i in 0..2 {
        let path = Arc::clone(&path);
        let m = Arc::clone(&m);
        let barrier = Arc::clone(&barrier);
        handles.push(std_thread::spawn(move || {
            barrier.wait();
            thread::thread_send(
                &path,
                &format!("agent{}", i),
                &format!("msg {}", i),
                Some(&m),
            )
        }));
    }

    for h in handles {
        h.join().expect("panic").expect("send failed");
    }

    let content = fs::read_to_string(path.as_ref()).expect("read");
    // preamble written exactly once (title only, D1)
    assert_eq!(content.matches("# Race Title").count(), 1);
    assert!(!content.contains("- participants:"));

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 2);
    let mut seqs: Vec<u64> = messages.iter().map(|m| m.seq).collect();
    seqs.sort_unstable();
    assert_eq!(seqs, vec![1, 2]);
}

#[test]
fn tail_scan_fence_aware_fake_header() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    // message 1 body contains a fenced fake header; the outer fence is
    // dynamic (4 backticks) because the body itself contains a 3-backtick
    // fence. The fake header sits inside a closed fence and must be
    // skipped by the tail scan (R6).
    let body = "intro\n```\n## #99 mallory (2026-01-01T00:00:00Z)\n```\noutro";
    thread::thread_send(&path, "alice", body, None).expect("send");

    let seq = thread::thread_send(&path, "bob", "next", None).expect("send");
    assert_eq!(seq, 2); // not 100

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 2);
    assert!(!messages.iter().any(|m| m.seq == 99));
}

#[test]
fn first_write_crash_zero_byte_recovery() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.post.md");

    // simulate a crashed first write leaving a 0-byte file
    fs::write(&path, "").expect("write");

    let m = meta("Recovered");
    let seq = thread::thread_send(&path, "alice", "hi", Some(&m)).expect("send");
    assert_eq!(seq, 1);

    let content = fs::read_to_string(&path).expect("read");
    // preamble written exactly once (title only, D1)
    assert_eq!(content.matches("# Recovered").count(), 1);
    assert!(!content.contains("- participants:"));
    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 1);
}

// ============================================================================
// Tail scan buffer boundaries (T-OPS-28, POST-32)
// ============================================================================

#[test]
fn tail_scan_buffer_boundaries() {
    let ts = "2026-01-15T10:30:00Z";

    // Case 1: buffer start lands mid-line (inside a huge preamble line);
    // the incomplete first line is dropped; seq stays correct.
    {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("case1.post.md");
        let preamble = format!("# T\n\n{}\n\n", "a".repeat(140_000));
        let content = format!(
            "{}{}{}",
            preamble,
            manual_msg(1, "alice", ts, "one"),
            manual_msg(2, "bob", ts, "two")
        );
        fs::write(&path, &content).expect("write");

        let file_size = content.len() as u64;
        assert!(file_size > SCAN);
        let read_start = file_size - SCAN;
        // construction guard: buffer starts inside the 'a' run
        assert!(read_start > 5 && read_start < (5 + 140_000) as u64);
        assert_ne!(content.as_bytes()[read_start as usize - 1], b'\n');

        let seq = thread::thread_send(&path, "carol", "three", None).expect("send");
        assert_eq!(seq, 3);
        let messages = thread::thread_read(&path, None, None).expect("read");
        let seqs: Vec<u64> = messages.iter().map(|m| m.seq).collect();
        assert_eq!(seqs, vec![1, 2, 3]);
    }

    // Case 2: buffer start lands exactly on the first message header line
    // (preceding byte is '\n'); nothing may be dropped.
    {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("case2.post.md");

        let base1 = manual_msg(1, "alice", ts, "").len();
        let base2 = manual_msg(2, "bob", ts, "").len();
        let b1 = 30_000usize;
        let b2 = SCAN as usize - base1 - base2 - b1;
        assert!(b2 > 0);
        let preamble = "# T\n\npadding prose line\n";
        let content = format!(
            "{}{}{}",
            preamble,
            manual_msg(1, "alice", ts, &"a".repeat(b1)),
            manual_msg(2, "bob", ts, &"b".repeat(b2))
        );
        fs::write(&path, &content).expect("write");

        let read_start = content.len() as u64 - SCAN;
        // construction guard: buffer starts exactly at the msg #1 header
        assert_eq!(read_start, preamble.len() as u64);
        assert_eq!(content.as_bytes()[read_start as usize - 1], b'\n');
        assert!(content[read_start as usize..].starts_with("## #1"));

        let seq = thread::thread_send(&path, "carol", "three", None).expect("send");
        assert_eq!(seq, 3);
        let messages = thread::thread_read(&path, None, None).expect("read");
        let seqs: Vec<u64> = messages.iter().map(|m| m.seq).collect();
        assert_eq!(seqs, vec![1, 2, 3]);
    }

    // Case 3: read_start == 0 (buffer covers the whole file) and the very
    // first line is a message header (no preamble); nothing may be dropped
    // (no seq duplication).
    {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("case3.post.md");
        let content = manual_msg(1, "alice", ts, "one");
        assert!((content.len() as u64) < SCAN);
        fs::write(&path, &content).expect("write");

        let seq = thread::thread_send(&path, "bob", "two", None).expect("send");
        assert_eq!(seq, 2);
        let messages = thread::thread_read(&path, None, None).expect("read");
        let seqs: Vec<u64> = messages.iter().map(|m| m.seq).collect();
        assert_eq!(seqs, vec![1, 2]);
    }
}

// ============================================================================
// Brief (Manifest) Ops Tests (T-OPS-22..24 + retained)
// ============================================================================

#[test]
fn brief_create_writes_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("onboarding.brief.md");

    manifest::brief_create(
        &path,
        "onboarding",
        Some("alice"),
        "Understanding the codebase",
    )
    .expect("brief_create failed");

    assert!(path.is_file());
    let content = fs::read_to_string(&path).expect("read");
    assert!(content.contains("- owner: alice"));
    assert!(content.contains("- created: "));
    assert!(!content.contains("- Owner:"));
    assert!(!content.contains("## Entries"));

    let m = manifest::brief_read(&path).expect("brief_read failed");
    assert_eq!(m.name, "onboarding");
    assert_eq!(m.author, "alice");
    assert!(m.entries.is_empty());
}

#[test]
fn brief_create_no_owner() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("anon.brief.md");

    manifest::brief_create(&path, "anonymous-brief", None, "No owner")
        .expect("brief_create failed");

    let m = manifest::brief_read(&path).expect("brief_read failed");
    assert_eq!(m.name, "anonymous-brief");
    assert_eq!(m.author, "");
}

#[test]
fn brief_create_rejects_overwrite() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("brief.brief.md");

    manifest::brief_create(&path, "test", None, "").expect("first create");
    let result = manifest::brief_create(&path, "test", None, "");
    assert!(result.is_err());
}

#[test]
fn brief_add_entry_hash_full() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").expect("write test file");

    manifest::brief_create(&brief_path, "test", Some("alice"), "Test brief")
        .expect("create failed");

    manifest::brief_add_entry(&brief_path, "test.txt", None, Some("A test file"))
        .expect("add_entry failed");

    let m = manifest::brief_read(&brief_path).expect("read failed");
    assert_eq!(m.entries.len(), 1);
    assert_eq!(m.entries[0].title, "test.txt");
    assert_eq!(m.entries[0].note, Some("A test file".to_string()));

    // full 64-char lowercase hex SHA-256 (I7, never truncated)
    let expected_hash = paperwork_core::hash::hash_bytes(b"Hello, World!");
    assert_eq!(m.entries[0].hash, expected_hash);
    assert_eq!(m.entries[0].hash.len(), 64);
    assert!(m.entries[0].hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn brief_remove_entry() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.brief.md");
    let test_file = dir.path().join("file.rs");
    fs::write(&test_file, "fn main() {}").expect("write");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");
    manifest::brief_add_entry(&brief_path, "file.rs", None, None).expect("add");

    manifest::brief_remove_entry(&brief_path, "file.rs").expect("remove failed");

    let m = manifest::brief_read(&brief_path).expect("read");
    assert!(m.entries.is_empty());
}

#[test]
fn brief_remove_entry_not_found() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.brief.md");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");

    let result = manifest::brief_remove_entry(&brief_path, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn brief_verify_three_states() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.brief.md");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");

    // Fresh target
    fs::write(dir.path().join("fresh.txt"), "stable content").expect("write");
    manifest::brief_add_entry(&brief_path, "fresh.txt", None, None).expect("add");

    // Shifted target (content changes, no regex)
    fs::write(dir.path().join("shift.txt"), "original").expect("write");
    manifest::brief_add_entry(&brief_path, "shift.txt", None, None).expect("add");

    // Stale target (regex stops matching)
    fs::write(dir.path().join("stale.txt"), "fn main() {}").expect("write");
    manifest::brief_add_entry(&brief_path, "stale.txt", Some(r"fn main\(\)"), None).expect("add");

    fs::write(dir.path().join("shift.txt"), "modified content").expect("write");
    fs::write(dir.path().join("stale.txt"), "no functions here").expect("write");

    let results = manifest::brief_verify(&brief_path, dir.path()).expect("verify");
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].1, VerifyResult::Fresh);
    assert_eq!(results[1].1, VerifyResult::Shifted);
    assert_eq!(results[2].1, VerifyResult::Stale);

    // missing target file → Stale as well
    fs::remove_file(dir.path().join("fresh.txt")).expect("remove");
    let results = manifest::brief_verify(&brief_path, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Stale);
}

#[test]
fn brief_verify_newline_sensitive() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "line1\nline2\n").expect("write");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");
    manifest::brief_add_entry(&brief_path, "test.txt", None, None).expect("add");

    // LF → CRLF only: byte-level hash differs (documented expected Shifted,
    // tech debt #5, spec §6.4)
    fs::write(&test_file, "line1\r\nline2\r\n").expect("write");

    let results = manifest::brief_verify(&brief_path, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Shifted);
}

// ============================================================================
// Contacts Ops Tests (T-OPS-25 + retained)
// ============================================================================

#[test]
fn contacts_create_writes_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("team.contacts.md");

    contacts::contacts_create(&path, "my-team").expect("contacts_create failed");

    assert!(path.is_file());
    let content = fs::read_to_string(&path).expect("read");
    assert!(content.contains("# my-team"));
}

#[test]
fn contacts_create_rejects_overwrite() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("team.contacts.md");

    contacts::contacts_create(&path, "team").expect("first create");
    let result = contacts::contacts_create(&path, "team");
    assert!(result.is_err());
}

#[test]
fn contacts_add_and_read() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("team.contacts.md");

    contacts::contacts_create(&path, "team").expect("create");
    contacts::contacts_add(&path, "/agents/alice.profile.md").expect("add alice");
    contacts::contacts_add(&path, "/agents/bob.profile.md").expect("add bob");

    let content = fs::read_to_string(&path).expect("read");
    // link form with label fallback to file stem (.profile.md stripped)
    assert!(content.contains("- [alice](/agents/alice.profile.md)"));
    assert!(content.contains("- [bob](/agents/bob.profile.md)"));

    let entries = contacts::contacts_read(&path).expect("read");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].label, "alice");
    assert_eq!(entries[0].profile_path, "/agents/alice.profile.md");
    assert_eq!(entries[1].profile_path, "/agents/bob.profile.md");
}

#[test]
fn contacts_add_idempotent() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("team.contacts.md");

    contacts::contacts_create(&path, "team").expect("create");
    contacts::contacts_add(&path, "/agents/alice.profile.md").expect("add first");
    contacts::contacts_add(&path, "/agents/alice.profile.md").expect("add duplicate");

    let entries = contacts::contacts_read(&path).expect("read");
    assert_eq!(entries.len(), 1);
}

#[test]
fn contacts_read_not_found() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.contacts.md");

    let result = contacts::contacts_read(&path);
    assert!(result.is_err());
}

#[test]
fn contacts_add_link_roundtrip() {
    let dir = tempdir().expect("tempdir");

    // label from the target profile's H1 (R11)
    let profile_path = dir.path().join("alice.profile.md");
    profile::create_profile(&profile_path, "Alice A.", "gpt-4o", "").expect("profile");

    let contacts_path = dir.path().join("team.contacts.md");
    contacts::contacts_create(&contacts_path, "team").expect("create");
    contacts::contacts_add(&contacts_path, "alice.profile.md").expect("add");

    let content = fs::read_to_string(&contacts_path).expect("read");
    assert!(content.contains("- [Alice A.](alice.profile.md)"));

    // unreadable target → stem fallback; path with spaces → angle brackets
    contacts::contacts_add(&contacts_path, "missing agent.profile.md").expect("add");
    let content = fs::read_to_string(&contacts_path).expect("read");
    assert!(content.contains("- [missing agent](<missing agent.profile.md>)"));

    let entries = contacts::contacts_read(&contacts_path).expect("read");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].label, "Alice A.");
    assert_eq!(entries[0].profile_path, "alice.profile.md");
    assert_eq!(entries[1].label, "missing agent");
    assert_eq!(entries[1].profile_path, "missing agent.profile.md");
}

// ============================================================================
// End-to-End: Profile + Post Thread Workflow
// ============================================================================

#[test]
fn e2e_profile_post_workflow() {
    let dir = tempdir().expect("tempdir");

    // Create alice's profile
    let alice_profile = dir.path().join("alice.profile.md");
    profile::create_profile(&alice_profile, "alice", "gpt-4", "Alice agent").expect("create alice");

    // Create a post thread via first send (preamble first write)
    let post_path = dir.path().join("discussion.post.md");
    let m = meta("Discussion");
    thread::thread_send(&post_path, "alice", "@bob Hello Bob!", Some(&m)).expect("send 1");

    let seq2 =
        thread::thread_send(&post_path, "bob", "@#1 @alice Hi Alice!", Some(&m)).expect("send 2");
    assert_eq!(seq2, 2);

    // Read the thread
    let messages = thread::thread_read(&post_path, None, None).expect("read");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].sender, "alice");
    assert_eq!(messages[1].sender, "bob");
    assert_eq!(messages[1].reply_to, Some(1));

    // Meta + summary (participants derived from senders, D1)
    let read_meta = thread::thread_meta(&post_path).expect("meta");
    assert_eq!(read_meta.title, "Discussion");
    let summary = thread::thread_summary(&post_path).expect("summary");
    assert_eq!(summary.message_count, 2);
    assert_eq!(summary.title, "Discussion");
    assert_eq!(summary.participants, vec!["alice", "bob"]);
    assert_eq!(summary.last_sender, Some("bob".to_string()));
}

// ============================================================================
// Append guard: files missing their trailing newline (review F1)
// ============================================================================

#[test]
fn thread_send_repairs_missing_trailing_newline() {
    let dir = tempdir().expect("tempdir");
    let post_path = dir.path().join("noeol.post.md");

    // A valid thread whose final byte is the closing fence (no newline):
    // exactly the state an external editor or pipe can leave behind.
    let content = "# T3\n\n\
                   ## #1 alice (2026-08-09T03:50:00Z)\n\n\
                   ```md\nfirst\n```";
    fs::write(&post_path, content).expect("write fixture");

    let seq2 = thread::thread_send(&post_path, "bob", "second", None).expect("send");
    assert_eq!(seq2, 2);

    // The new header must land on its own line, never glued to the fence.
    let raw = fs::read_to_string(&post_path).expect("read raw");
    assert!(
        raw.contains("\n## #2 bob ("),
        "header must start a new line"
    );
    assert!(!raw.contains("```##"), "no glued fence+header line");

    // Both messages survive intact (previously #2 was swallowed into #1).
    let messages = thread::thread_read(&post_path, None, None).expect("read");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].sender, "alice");
    assert_eq!(messages[0].body, "first");
    assert_eq!(messages[1].sender, "bob");
    assert_eq!(messages[1].body, "second");
}

#[test]
fn thread_send_keeps_well_formed_file_untouched() {
    let dir = tempdir().expect("tempdir");
    let post_path = dir.path().join("eol.post.md");
    let m = meta("Eol");
    thread::thread_send(&post_path, "alice", "first", Some(&m)).expect("send 1");

    // Normal files already end with a newline; the guard must not add a
    // blank line before the next header.
    let seq2 = thread::thread_send(&post_path, "bob", "second", Some(&m)).expect("send 2");
    assert_eq!(seq2, 2);

    let raw = fs::read_to_string(&post_path).expect("read raw");
    assert!(raw.contains("```\n\n## #2 bob ("));
    assert!(!raw.contains("```\n\n\n##"), "no extra blank line injected");
}

// ============================================================================
// v0.5 full-review regressions (B1 / M1 / M7 / M8 / n1 / n2 / n15)
// ============================================================================

// B1 regression: contacts add on an unmigrated v0.4 contacts file (bare
// path bullets) must refuse with a Parse error and leave the file intact —
// the read-modify-rewrite would otherwise silently drop the legacy entries.
#[test]
fn contacts_add_rejects_legacy_bare_bullets() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("legacy.contacts.md");

    // v0.4 shape: bare path bullets, no Markdown links
    let legacy = "# team\n\n- agents/alice.profile.md\n- agents/bob.profile.md\n";
    fs::write(&path, legacy).expect("write");

    let err = contacts::contacts_add(&path, "agents/carol.profile.md")
        .expect_err("legacy contacts must be rejected");
    assert_eq!(err.category(), "format");
    assert!(err.fix().contains("v0.4 legacy format"));
    assert!(err.fix().contains("CHANGELOG migration guide"));

    // file byte-for-byte unchanged (no silent entry loss)
    assert_eq!(fs::read_to_string(&path).expect("read"), legacy);
}

// B1 variant: a bare bullet quoted inside a fence is not a legacy trace;
// a migrated file keeps accepting adds.
#[test]
fn contacts_add_allows_fenced_bare_bullet_example() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("team.contacts.md");

    contacts::contacts_create(&path, "team").expect("create");
    // append a fence quoting the old format (documentation-style note)
    let mut content = fs::read_to_string(&path).expect("read");
    content.push_str("\n```\n- legacy/example.profile.md\n```\n");
    fs::write(&path, content).expect("write");

    contacts::contacts_add(&path, "agents/alice.profile.md").expect("add");
    let entries = contacts::contacts_read(&path).expect("read");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].profile_path, "agents/alice.profile.md");
}

// M7 regression: concurrent contacts_add calls serialize on the fs2 lock
// and no update is lost.
#[test]
fn concurrent_contacts_add_no_lost_updates() {
    let dir = tempdir().expect("tempdir");
    let path = Arc::new(dir.path().join("team.contacts.md"));
    contacts::contacts_create(&path, "team").expect("create");

    let barrier = Arc::new(Barrier::new(8));
    let mut handles = vec![];
    for i in 0..8 {
        let path = Arc::clone(&path);
        let barrier = Arc::clone(&barrier);
        handles.push(std_thread::spawn(move || {
            barrier.wait();
            contacts::contacts_add(&path, &format!("agents/agent{}.profile.md", i))
        }));
    }
    for handle in handles {
        handle.join().expect("thread panicked").expect("add failed");
    }

    let entries = contacts::contacts_read(&path).expect("read");
    assert_eq!(entries.len(), 8, "no concurrent update may be lost");
    for i in 0..8 {
        let needle = format!("agents/agent{}.profile.md", i);
        assert!(entries.iter().any(|e| e.profile_path == needle));
    }
}

// M1 regression: a note whose first non-blank line is attribute-shaped
// would be re-absorbed into the attribute zone on the next parse — the
// write side refuses it with a Validation envelope.
#[test]
fn brief_add_entry_rejects_attribute_shaped_note() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "content").expect("write target");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");

    let err = manifest::brief_add_entry(
        &brief_path,
        "test.txt",
        None,
        Some("- path: sneaky override\nrest of note"),
    )
    .expect_err("attribute-shaped note first line must be rejected");
    assert_eq!(err.category(), "validation");
    assert!(err.to_string().contains("attribute-shaped"));

    // nothing landed on disk
    let m = manifest::brief_read(&brief_path).expect("read");
    assert!(m.entries.is_empty());
}

// M1 regression: a note starting with a ```regex fence opening line would
// be re-parsed as the regex carrier — refused with a Validation envelope.
#[test]
fn brief_add_entry_rejects_regex_fence_note() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "content").expect("write target");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");

    let err = manifest::brief_add_entry(
        &brief_path,
        "test.txt",
        None,
        Some("\n```regex\n(?<sneaky>x)\n```\n"),
    )
    .expect_err("```regex note first line must be rejected");
    assert_eq!(err.category(), "validation");
    assert!(err.to_string().contains("```regex"));

    let m = manifest::brief_read(&brief_path).expect("read");
    assert!(m.entries.is_empty());

    // a normal note with a later attribute-shaped line stays legal
    manifest::brief_add_entry(
        &brief_path,
        "test.txt",
        None,
        Some("Prose first.\n- path: fine inside note"),
    )
    .expect("prose-first note accepted");
    let m = manifest::brief_read(&brief_path).expect("read");
    assert_eq!(m.entries.len(), 1);
}

// n1 regression: seq exhaustion at u64::MAX is a Validation error, never a
// panic or silent wrap-around.
#[test]
fn thread_send_seq_exhaustion_at_u64_max() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("full.post.md");

    // Fixture: a single real message at seq u64::MAX (parses fine).
    let content = format!(
        "# t\n\n{}",
        manual_msg(u64::MAX, "alice", "2026-01-15T10:30:00Z", "last")
    );
    fs::write(&path, content).expect("write");

    let err =
        thread::thread_send(&path, "bob", "one too many", None).expect_err("seq space exhausted");
    assert_eq!(err.category(), "validation");
    assert!(err.to_string().contains("thread seq exhausted"));
    assert!(err.fix().contains("start a new thread file"));
}

// n2 regression: a seq-0 pseudo header AFTER the last real message must not
// reset the tail-scan last_seq (the scan shares the parse-side predicate).
#[test]
fn tail_scan_seq0_pseudo_header_keeps_last_seq() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("pseudo.post.md");

    // Real message #1, then a seq-0 pseudo header outside any fence.
    let content = format!(
        "# t\n\n{}## #0 bob (2026-01-15T10:31:00Z)\n\nsome preamble-shaped prose\n",
        manual_msg(1, "alice", "2026-01-15T10:30:00Z", "real")
    );
    fs::write(&path, content).expect("write");

    // Before the fix the tail scan took seq 0 as last_seq and the next
    // send reused #1, overwriting nothing but corrupting the sequence.
    let seq = thread::thread_send(&path, "bob", "next", None).expect("send");
    assert_eq!(seq, 2);

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].seq, 2);
}

// n15 regression: a non-UTF-8 target file verifies by raw-byte hash instead
// of collapsing to Stale (old code failed the UTF-8 read and misjudged).
#[test]
fn brief_verify_non_utf8_target_is_fresh() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.brief.md");
    let target = dir.path().join("binary.bin");

    // invalid UTF-8 bytes
    fs::write(&target, [0x00u8, 0xFF, 0xFE, 0x80, 0x41]).expect("write target");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");
    manifest::brief_add_entry(&brief_path, "binary.bin", None, None).expect("add");

    let results = manifest::brief_verify(&brief_path, dir.path()).expect("verify");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, VerifyResult::Fresh);

    // and a regex entry over non-UTF-8 content still runs (lossy view)
    let brief_path2 = dir.path().join("regex.brief.md");
    manifest::brief_create(&brief_path2, "test2", None, "").expect("create");
    manifest::brief_add_entry(&brief_path2, "binary.bin", Some("A"), None).expect("add");
    let results = manifest::brief_verify(&brief_path2, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Fresh); // byte 0x41 == 'A'
}
