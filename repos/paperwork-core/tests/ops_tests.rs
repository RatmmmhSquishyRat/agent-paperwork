//! Integration tests for the stateless, path-explicit operations layer.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread as std_thread;

use chrono::Utc;
use tempfile::tempdir;

use paperwork_core::ops::{contacts, manifest, notify, profile, thread};
use paperwork_core::{Notification, NotifyType, VerifyResult};

// ============================================================================
// Profile Ops Tests
// ============================================================================

#[test]
fn create_profile_writes_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.md");

    profile::create_profile(&path, "alice", "gpt-4", "Test agent")
        .expect("create_profile failed");

    assert!(path.is_file());
    let content = fs::read_to_string(&path).expect("read failed");
    assert!(content.contains("# alice"));
    assert!(content.contains("**Model**: gpt-4"));
}

#[test]
fn create_profile_creates_parent_dirs() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("nested").join("deep").join("alice.md");

    profile::create_profile(&path, "alice", "gpt-4", "")
        .expect("create_profile failed");

    assert!(path.is_file());
}

#[test]
fn create_profile_rejects_overwrite() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.md");

    profile::create_profile(&path, "alice", "gpt-4", "").expect("first create");
    let result = profile::create_profile(&path, "alice", "gpt-4", "");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn show_profile_reads_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.md");

    profile::create_profile(&path, "alice", "gpt-4", "Hello world")
        .expect("create failed");

    let p = profile::show_profile(&path).expect("show_profile failed");
    assert_eq!(p.name, "alice");
    assert_eq!(p.model, "gpt-4");
    assert_eq!(p.description, "Hello world");
}

#[test]
fn show_profile_not_found() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("nonexistent.md");

    let result = profile::show_profile(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn edit_profile_updates_fields() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.md");

    profile::create_profile(&path, "alice", "gpt-4", "Original")
        .expect("create failed");

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
    // scope_write unchanged (empty)
    assert!(p.scope_write.is_empty());
}

// ============================================================================
// Thread Ops Tests
// ============================================================================

#[test]
fn thread_send_creates_file_and_returns_seq() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.md");

    let seq = thread::thread_send(
        &path,
        "alice",
        &["bob".to_string()],
        "Hello, Bob!",
        None,
        &[],
    )
    .expect("thread_send failed");

    assert_eq!(seq, 1);
    assert!(path.is_file());
}

#[test]
fn thread_send_creates_parent_dirs() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.dm").join("bob.md");

    let seq = thread::thread_send(&path, "alice", &["bob".to_string()], "Hi", None, &[])
        .expect("thread_send failed");

    assert_eq!(seq, 1);
    assert!(path.is_file());
}

#[test]
fn thread_send_increments_seq() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.md");

    for i in 0..3 {
        let sender = if i % 2 == 0 { "alice" } else { "bob" };
        let seq = thread::thread_send(
            &path,
            sender,
            &[],
            &format!("Message {}", i + 1),
            None,
            &[],
        )
        .expect("thread_send failed");
        assert_eq!(seq, (i + 1) as u64);
    }

    let messages = thread::thread_read(&path, None, None).expect("read failed");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].seq, 1);
    assert_eq!(messages[1].seq, 2);
    assert_eq!(messages[2].seq, 3);
}

#[test]
fn thread_read_range_subset() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.md");

    for i in 1..=5 {
        thread::thread_send(&path, "alice", &[], &format!("Msg {}", i), None, &[])
            .expect("send failed");
    }

    let messages = thread::thread_read(&path, Some(2), Some(4)).expect("read failed");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].seq, 2);
    assert_eq!(messages[2].seq, 4);
}

#[test]
fn thread_read_not_found() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("nonexistent.md");

    let result = thread::thread_read(&path, None, None);
    assert!(result.is_err());
}

#[test]
fn thread_summary_correct() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.md");

    thread::thread_send(&path, "alice", &[], "First", None, &[]).expect("send");
    thread::thread_send(&path, "bob", &[], "Second", None, &[]).expect("send");
    thread::thread_send(&path, "alice", &[], "Third", None, &[]).expect("send");

    let summary = thread::thread_summary(&path).expect("summary failed");
    assert_eq!(summary.message_count, 3);
    assert_eq!(summary.last_sender, Some("alice".to_string()));
    assert!(!summary.snippets.is_empty());
}

#[test]
fn thread_summary_empty_for_missing_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.md");

    let summary = thread::thread_summary(&path).expect("summary failed");
    assert_eq!(summary.message_count, 0);
    assert_eq!(summary.last_sender, None);
}

#[test]
fn thread_send_with_reply_to() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.md");

    thread::thread_send(&path, "alice", &["bob".to_string()], "Hello", None, &[])
        .expect("send");
    thread::thread_send(
        &path,
        "bob",
        &["alice".to_string()],
        "Reply",
        Some(1),
        &[],
    )
    .expect("send");

    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages[1].reply_to, Some(1));
}

#[test]
fn concurrent_thread_send_safety() {
    let dir = tempdir().expect("tempdir");
    let path = Arc::new(dir.path().join("thread.md"));
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
                &[],
                &format!("Concurrent message {}", i),
                None,
                &[],
            )
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("thread panicked").expect("send failed");
    }

    let messages = thread::thread_read(&path, None, None).expect("read failed");
    assert_eq!(messages.len(), 10);

    // Verify seqs are 1-10 with no gaps
    for (i, msg) in messages.iter().enumerate() {
        assert_eq!(msg.seq, (i + 1) as u64);
    }
}

// ============================================================================
// Thread Edit Tests
// ============================================================================

#[test]
fn thread_edit_own_message() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.md");

    thread::thread_send(&path, "alice", &["bob".to_string()], "Original body", None, &[])
        .expect("send");

    thread::thread_edit(&path, 1, "alice", "Edited body").expect("edit failed");

    let messages = thread::thread_read(&path, Some(1), Some(1)).expect("read");
    assert_eq!(messages[0].body, "Edited body");
    assert_eq!(messages[0].sender, "alice");
}

#[test]
fn thread_edit_rejects_other_sender() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.md");

    thread::thread_send(&path, "alice", &[], "Alice's message", None, &[]).expect("send");

    let result = thread::thread_edit(&path, 1, "bob", "Hacked!");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("sent by 'alice', not 'bob'"));
}

#[test]
fn thread_edit_only_last_own() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("thread.md");

    thread::thread_send(&path, "alice", &[], "Alice 1", None, &[]).expect("send");
    thread::thread_send(&path, "bob", &[], "Bob 1", None, &[]).expect("send");
    thread::thread_send(&path, "alice", &[], "Alice 2", None, &[]).expect("send");

    // Try to edit #1 (not alice's most recent, and not final)
    let result = thread::thread_edit(&path, 1, "alice", "Edited");
    assert!(result.is_err());
}

#[test]
fn thread_edit_not_found_thread() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.md");

    let result = thread::thread_edit(&path, 1, "alice", "test");
    assert!(result.is_err());
}

// ============================================================================
// Brief (Manifest) Ops Tests
// ============================================================================

#[test]
fn brief_create_writes_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("onboarding.md");

    manifest::brief_create(&path, "onboarding", Some("alice"), "Understanding the codebase")
        .expect("brief_create failed");

    assert!(path.is_file());

    let m = manifest::brief_read(&path).expect("brief_read failed");
    assert_eq!(m.name, "onboarding");
    assert_eq!(m.author, "alice");
    assert!(m.entries.is_empty());
}

#[test]
fn brief_create_no_owner() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("anon.md");

    manifest::brief_create(&path, "anonymous-brief", None, "No owner")
        .expect("brief_create failed");

    let m = manifest::brief_read(&path).expect("brief_read failed");
    assert_eq!(m.name, "anonymous-brief");
    assert_eq!(m.author, "");
}

#[test]
fn brief_create_rejects_overwrite() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("brief.md");

    manifest::brief_create(&path, "test", None, "").expect("first create");
    let result = manifest::brief_create(&path, "test", None, "");
    assert!(result.is_err());
}

#[test]
fn brief_add_entry_computes_hash() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").expect("write test file");

    manifest::brief_create(&brief_path, "test", Some("alice"), "Test brief")
        .expect("create failed");

    manifest::brief_add_entry(&brief_path, "test.txt", None, Some("A test file"))
        .expect("add_entry failed");

    let m = manifest::brief_read(&brief_path).expect("read failed");
    assert_eq!(m.entries.len(), 1);
    assert_eq!(m.entries[0].title, "test.txt");
    assert!(!m.entries[0].hash.is_empty());

    let expected_hash = paperwork_core::hash::hash_bytes(b"Hello, World!");
    assert_eq!(m.entries[0].hash, expected_hash);
}

#[test]
fn brief_remove_entry() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.md");
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
    let brief_path = dir.path().join("brief.md");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");

    let result = manifest::brief_remove_entry(&brief_path, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn brief_verify_fresh() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "content").expect("write");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");
    manifest::brief_add_entry(&brief_path, "test.txt", None, None).expect("add");

    let results = manifest::brief_verify(&brief_path, dir.path()).expect("verify");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, VerifyResult::Fresh);
}

#[test]
fn brief_verify_shifted() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "original content").expect("write");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");
    manifest::brief_add_entry(&brief_path, "test.txt", None, None).expect("add");

    // Modify the file
    fs::write(&test_file, "modified content").expect("write");

    let results = manifest::brief_verify(&brief_path, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Shifted);
}

#[test]
fn brief_verify_stale_regex_mismatch() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "fn main() {}").expect("write");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");
    manifest::brief_add_entry(&brief_path, "test.txt", Some(r"fn nonexistent\(\)"), None)
        .expect("add");

    let results = manifest::brief_verify(&brief_path, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Stale);
}

#[test]
fn brief_verify_stale_file_missing() {
    let dir = tempdir().expect("tempdir");
    let brief_path = dir.path().join("brief.md");
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "content").expect("write");

    manifest::brief_create(&brief_path, "test", None, "").expect("create");
    manifest::brief_add_entry(&brief_path, "test.txt", None, None).expect("add");

    // Delete the file
    fs::remove_file(&test_file).expect("remove");

    let results = manifest::brief_verify(&brief_path, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Stale);
}

// ============================================================================
// Contacts Ops Tests
// ============================================================================

#[test]
fn contacts_create_writes_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("contacts.md");

    contacts::contacts_create(&path, "my-team").expect("contacts_create failed");

    assert!(path.is_file());
    let content = fs::read_to_string(&path).expect("read");
    assert!(content.contains("# Contacts: my-team"));
}

#[test]
fn contacts_create_rejects_overwrite() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("contacts.md");

    contacts::contacts_create(&path, "team").expect("first create");
    let result = contacts::contacts_create(&path, "team");
    assert!(result.is_err());
}

#[test]
fn contacts_add_and_read() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("contacts.md");

    contacts::contacts_create(&path, "team").expect("create");
    contacts::contacts_add(&path, "/agents/alice.md").expect("add alice");
    contacts::contacts_add(&path, "/agents/bob.md").expect("add bob");

    let entries = contacts::contacts_read(&path).expect("read");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].profile_path, "/agents/alice.md");
    assert_eq!(entries[1].profile_path, "/agents/bob.md");
}

#[test]
fn contacts_add_idempotent() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("contacts.md");

    contacts::contacts_create(&path, "team").expect("create");
    contacts::contacts_add(&path, "/agents/alice.md").expect("add first");
    contacts::contacts_add(&path, "/agents/alice.md").expect("add duplicate");

    let entries = contacts::contacts_read(&path).expect("read");
    assert_eq!(entries.len(), 1);
}

#[test]
fn contacts_read_not_found() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.md");

    let result = contacts::contacts_read(&path);
    assert!(result.is_err());
}

// ============================================================================
// Notification Ops Tests
// ============================================================================

#[test]
fn notify_push_and_read() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.notify.md");

    let notif = Notification {
        timestamp: Utc::now(),
        from: "bob".to_string(),
        thread_path: "/threads/dm.md".to_string(),
        seq: 1,
        notify_type: NotifyType::Mention,
        snippet: "Hey @alice!".to_string(),
    };

    notify::notify_push(&path, "alice", &notif).expect("push failed");

    let notifications = notify::notify_read(&path).expect("read failed");
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].from, "bob");
    assert_eq!(notifications[0].snippet, "Hey @alice!");
}

#[test]
fn notify_read_empty_for_missing_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.notify.md");

    let notifications = notify::notify_read(&path).expect("read failed");
    assert!(notifications.is_empty());
}

#[test]
fn notify_push_multiple() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.notify.md");

    for i in 1..=3 {
        let notif = Notification {
            timestamp: Utc::now(),
            from: format!("agent{}", i),
            thread_path: "/threads/t.md".to_string(),
            seq: i,
            notify_type: NotifyType::Mention,
            snippet: format!("Notification {}", i),
        };
        notify::notify_push(&path, "alice", &notif).expect("push");
    }

    let notifications = notify::notify_read(&path).expect("read");
    assert_eq!(notifications.len(), 3);
}

// ============================================================================
// DM Path Helper Tests
// ============================================================================

#[test]
fn dm_thread_path_convention() {
    let profile = PathBuf::from("/foo/alice.md");
    let dm = paperwork_core::ops::dm_thread_path(&profile, "bob");
    assert_eq!(dm, PathBuf::from("/foo/alice.dm/bob.md"));
}

#[test]
fn dm_thread_path_windows_style() {
    let profile = PathBuf::from(r"C:\agents\alice.md");
    let dm = paperwork_core::ops::dm_thread_path(&profile, "charlie");
    assert_eq!(dm, PathBuf::from(r"C:\agents\alice.dm\charlie.md"));
}

// ============================================================================
// End-to-End: Profile + DM Thread Workflow
// ============================================================================

#[test]
fn e2e_profile_dm_workflow() {
    let dir = tempdir().expect("tempdir");

    // Create alice's profile
    let alice_profile = dir.path().join("alice.md");
    profile::create_profile(&alice_profile, "alice", "gpt-4", "Alice agent")
        .expect("create alice");

    // Compute DM path with bob
    let dm_path = paperwork_core::ops::dm_thread_path(&alice_profile, "bob");
    assert_eq!(
        dm_path,
        dir.path().join("alice.dm").join("bob.md")
    );

    // Send messages (auto-creates alice.dm/bob.md)
    let seq1 = thread::thread_send(
        &dm_path,
        "alice",
        &["bob".to_string()],
        "Hello Bob!",
        None,
        &[],
    )
    .expect("send 1");
    assert_eq!(seq1, 1);

    let seq2 = thread::thread_send(
        &dm_path,
        "bob",
        &["alice".to_string()],
        "Hi Alice!",
        Some(1),
        &[],
    )
    .expect("send 2");
    assert_eq!(seq2, 2);

    // Read the thread
    let messages = thread::thread_read(&dm_path, None, None).expect("read");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].sender, "alice");
    assert_eq!(messages[1].sender, "bob");
    assert_eq!(messages[1].reply_to, Some(1));

    // Summary
    let summary = thread::thread_summary(&dm_path).expect("summary");
    assert_eq!(summary.message_count, 2);
    assert_eq!(summary.last_sender, Some("bob".to_string()));
}
