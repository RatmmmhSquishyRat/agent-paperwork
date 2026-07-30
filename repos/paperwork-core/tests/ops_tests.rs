//! Integration tests for the operations layer.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread as std_thread;

use chrono::Utc;
use tempfile::{tempdir, TempDir};

use paperwork_core::ops::{contacts, manifest, notify, profile, thread};
use paperwork_core::{Access, Message, Notification, NotifyType, Profile, VerifyResult};

/// Setup helper: create initialized workspace.
fn setup_workspace(name: &str) -> (TempDir, PathBuf) {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path().to_path_buf();
    paperwork_core::ops::init(&root, name, "test-model").expect("init failed");
    (dir, root)
}

// ============================================================================
// 2.1 Init & Layout Tests
// ============================================================================

#[test]
fn init_creates_skeleton() {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    paperwork_core::ops::init(root, "alice", "gpt-4").expect("init failed");

    // Check directories exist
    assert!(root.join(".paperwork").is_dir());
    assert!(root.join(".paperwork/profiles").is_dir());
    assert!(root.join(".paperwork/dm").is_dir());
    assert!(root.join(".paperwork/posts").is_dir());
    assert!(root.join(".paperwork/manifests").is_dir());
    assert!(root.join(".paperwork/notifications").is_dir());

    // Check files exist
    assert!(root.join(".paperwork/contacts.md").is_file());
    assert!(root.join(".paperwork/.gitattributes").is_file());
    assert!(root.join(".paperwork/profiles/alice.md").is_file());
}

#[test]
fn init_idempotent() {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    paperwork_core::ops::init(root, "alice", "gpt-4").expect("first init failed");

    // Read profile content
    let content_before =
        fs::read_to_string(root.join(".paperwork/profiles/alice.md")).expect("read failed");

    // Init again
    paperwork_core::ops::init(root, "alice", "gpt-4").expect("second init failed");

    // Content should be unchanged
    let content_after =
        fs::read_to_string(root.join(".paperwork/profiles/alice.md")).expect("read failed");

    assert_eq!(content_before, content_after);
}

#[test]
fn init_second_agent() {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    paperwork_core::ops::init(root, "alice", "gpt-4").expect("alice init failed");
    paperwork_core::ops::init(root, "bob", "claude-3").expect("bob init failed");

    // Both profiles should exist
    assert!(root.join(".paperwork/profiles/alice.md").is_file());
    assert!(root.join(".paperwork/profiles/bob.md").is_file());

    // Contacts should have both
    let contacts_content =
        fs::read_to_string(root.join(".paperwork/contacts.md")).expect("read failed");
    assert!(contacts_content.contains("alice"));
    assert!(contacts_content.contains("bob"));
}

// ============================================================================
// 2.2 Profile Ops Tests
// ============================================================================

#[test]
fn create_profile_writes_file() {
    let (_dir, root) = setup_workspace("alice");

    let bob = Profile {
        name: "bob".to_string(),
        model: "claude-3".to_string(),
        description: "Test agent".to_string(),
        scope_read: vec!["src/**".to_string()],
        scope_write: vec![],
        scope_owns: vec![],
    };

    profile::create_profile(&root, &bob).expect("create_profile failed");

    // File should exist
    assert!(root.join(".paperwork/profiles/bob.md").is_file());

    // Contacts should be updated
    let contacts = contacts::contacts_list(&root).expect("contacts_list failed");
    assert!(contacts.iter().any(|c| c.agent == "bob"));
}

#[test]
fn edit_profile_scope() {
    let (_dir, root) = setup_workspace("alice");

    profile::edit_profile(
        &root,
        "alice",
        None,
        Some("Updated description"),
        Some(vec!["docs/**".to_string()]),
        None,
        Some(vec!["src/core/**".to_string()]),
    )
    .expect("edit_profile failed");

    let profile = profile::show_profile(&root, "alice").expect("show_profile failed");
    assert_eq!(profile.description, "Updated description");
    assert_eq!(profile.scope_read, vec!["docs/**"]);
    assert_eq!(profile.scope_owns, vec!["src/core/**"]);
}

#[test]
fn list_profiles() {
    let (_dir, root) = setup_workspace("alice");

    let bob = Profile {
        name: "bob".to_string(),
        model: "claude-3".to_string(),
        description: String::new(),
        scope_read: vec![],
        scope_write: vec![],
        scope_owns: vec![],
    };
    profile::create_profile(&root, &bob).expect("create bob failed");

    let profiles = profile::list_profiles(&root).expect("list_profiles failed");
    assert_eq!(profiles.len(), 2);
    assert!(profiles.iter().any(|p| p.name == "alice"));
    assert!(profiles.iter().any(|p| p.name == "bob"));
}

// ============================================================================
// 2.3 Thread Ops Tests
// ============================================================================

#[test]
fn append_first_message() {
    let (_dir, root) = setup_workspace("alice");

    // Create DM folder first
    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    let msg = Message {
        seq: 0, // Will be overwritten
        sender: "alice".to_string(),
        timestamp: Utc::now(),
        to: vec!["bob".to_string()],
        reply_to: None,
        body: "Hello, Bob!".to_string(),
    };

    thread::append_msg(&root, "dm/alice--bob/thread.md", &msg).expect("append failed");

    let messages = thread::read_range(&root, "dm/alice--bob/thread.md", 1, 100)
        .expect("read_range failed");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].seq, 1);
    assert_eq!(messages[0].body, "Hello, Bob!");
}

#[test]
fn append_increments_seq() {
    let (_dir, root) = setup_workspace("alice");
    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    for i in 0..3 {
        let msg = Message {
            seq: 0,
            sender: if i % 2 == 0 { "alice" } else { "bob" }.to_string(),
            timestamp: Utc::now(),
            to: vec![],
            reply_to: None,
            body: format!("Message {}", i + 1),
        };
        thread::append_msg(&root, "dm/alice--bob/thread.md", &msg).expect("append failed");
    }

    let messages = thread::read_range(&root, "dm/alice--bob/thread.md", 1, 100)
        .expect("read_range failed");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].seq, 1);
    assert_eq!(messages[1].seq, 2);
    assert_eq!(messages[2].seq, 3);
}

#[test]
fn read_range_subset() {
    let (_dir, root) = setup_workspace("alice");
    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    for i in 1..=5 {
        let msg = Message {
            seq: 0,
            sender: "alice".to_string(),
            timestamp: Utc::now(),
            to: vec![],
            reply_to: None,
            body: format!("Message {}", i),
        };
        thread::append_msg(&root, "dm/alice--bob/thread.md", &msg).expect("append failed");
    }

    let messages = thread::read_range(&root, "dm/alice--bob/thread.md", 2, 4)
        .expect("read_range failed");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].seq, 2);
    assert_eq!(messages[1].seq, 3);
    assert_eq!(messages[2].seq, 4);
}

#[test]
fn summary_correct() {
    let (_dir, root) = setup_workspace("alice");
    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    for i in 1..=3 {
        let msg = Message {
            seq: 0,
            sender: if i == 3 { "bob" } else { "alice" }.to_string(),
            timestamp: Utc::now(),
            to: vec![],
            reply_to: None,
            body: format!("Message {}", i),
        };
        thread::append_msg(&root, "dm/alice--bob/thread.md", &msg).expect("append failed");
    }

    let summary = thread::summary(&root, "dm/alice--bob/thread.md").expect("summary failed");
    assert_eq!(summary.message_count, 3);
    assert_eq!(summary.last_sender, Some("bob".to_string()));
    assert!(!summary.snippets.is_empty());
}

#[test]
fn concurrent_append_safety() {
    let (_dir, root) = setup_workspace("alice");
    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    let root = Arc::new(root);
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    for i in 0..10 {
        let root = Arc::clone(&root);
        let barrier = Arc::clone(&barrier);

        let handle = std_thread::spawn(move || {
            barrier.wait();
            let msg = Message {
                seq: 0,
                sender: format!("agent{}", i),
                timestamp: Utc::now(),
                to: vec![],
                reply_to: None,
                body: format!("Concurrent message {}", i),
            };
            thread::append_msg(&root, "dm/alice--bob/thread.md", &msg)
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("thread panicked").expect("append failed");
    }

    // All 10 messages should be present with unique seqs
    let messages = thread::read_range(&root, "dm/alice--bob/thread.md", 1, 100)
        .expect("read_range failed");
    assert_eq!(messages.len(), 10);

    // Verify seqs are 1-10 with no gaps
    for (i, msg) in messages.iter().enumerate() {
        assert_eq!(msg.seq, (i + 1) as u64);
    }
}

#[test]
fn self_edit_own_message() {
    let (_dir, root) = setup_workspace("alice");
    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    let msg = Message {
        seq: 0,
        sender: "alice".to_string(),
        timestamp: Utc::now(),
        to: vec!["bob".to_string()],
        reply_to: None,
        body: "Original body".to_string(),
    };
    thread::append_msg(&root, "dm/alice--bob/thread.md", &msg).expect("append failed");

    thread::self_edit(&root, "dm/alice--bob/thread.md", 1, "alice", "Edited body")
        .expect("self_edit failed");

    let messages = thread::read_range(&root, "dm/alice--bob/thread.md", 1, 1)
        .expect("read_range failed");
    assert_eq!(messages[0].body, "Edited body");
    assert_eq!(messages[0].sender, "alice"); // Metadata unchanged
}

#[test]
fn self_edit_rejects_other_sender() {
    let (_dir, root) = setup_workspace("alice");
    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    let msg = Message {
        seq: 0,
        sender: "alice".to_string(),
        timestamp: Utc::now(),
        to: vec![],
        reply_to: None,
        body: "Alice's message".to_string(),
    };
    thread::append_msg(&root, "dm/alice--bob/thread.md", &msg).expect("append failed");

    let result = thread::self_edit(&root, "dm/alice--bob/thread.md", 1, "bob", "Hacked!");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("sent by 'alice', not 'bob'") || err.contains("only edit your own"));
}

#[test]
fn self_edit_only_last_own() {
    let (_dir, root) = setup_workspace("alice");
    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    // alice sends #1
    let msg1 = Message {
        seq: 0,
        sender: "alice".to_string(),
        timestamp: Utc::now(),
        to: vec![],
        reply_to: None,
        body: "Alice 1".to_string(),
    };
    thread::append_msg(&root, "dm/alice--bob/thread.md", &msg1).expect("append failed");

    // bob sends #2
    let msg2 = Message {
        seq: 0,
        sender: "bob".to_string(),
        timestamp: Utc::now(),
        to: vec![],
        reply_to: None,
        body: "Bob 1".to_string(),
    };
    thread::append_msg(&root, "dm/alice--bob/thread.md", &msg2).expect("append failed");

    // alice sends #3
    let msg3 = Message {
        seq: 0,
        sender: "alice".to_string(),
        timestamp: Utc::now(),
        to: vec![],
        reply_to: None,
        body: "Alice 2".to_string(),
    };
    thread::append_msg(&root, "dm/alice--bob/thread.md", &msg3).expect("append failed");

    // Try to edit #1 (not alice's most recent)
    let result = thread::self_edit(&root, "dm/alice--bob/thread.md", 1, "alice", "Edited");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not your most recent") || err.contains("not the final") || err.contains("#3"));
}

// ============================================================================
// 2.4 Manifest Ops Tests
// ============================================================================

#[test]
fn create_manifest() {
    let (_dir, root) = setup_workspace("alice");

    manifest::create_manifest(&root, "onboarding", "alice", "Understanding the codebase")
        .expect("create_manifest failed");

    assert!(root.join(".paperwork/manifests/onboarding.md").is_file());

    let m = manifest::read_manifest(&root, "onboarding").expect("read_manifest failed");
    assert_eq!(m.name, "onboarding");
    assert_eq!(m.author, "alice");
    assert!(m.entries.is_empty());
}

#[test]
fn add_entry_computes_hash() {
    let (_dir, root) = setup_workspace("alice");

    // Create a test file
    let test_file = root.join("test.txt");
    fs::write(&test_file, "Hello, World!").expect("write test file failed");

    manifest::create_manifest(&root, "test", "alice", "Test manifest")
        .expect("create_manifest failed");

    manifest::add_entry(&root, "test", "Test File", "test.txt", None, Some("A test file"))
        .expect("add_entry failed");

    let m = manifest::read_manifest(&root, "test").expect("read_manifest failed");
    assert_eq!(m.entries.len(), 1);
    assert_eq!(m.entries[0].title, "Test File");
    assert!(!m.entries[0].hash.is_empty());

    // Verify hash is correct SHA-256
    let expected_hash = paperwork_core::hash::hash_bytes(b"Hello, World!");
    assert_eq!(m.entries[0].hash, expected_hash);
}

#[test]
fn verify_fresh() {
    let (_dir, root) = setup_workspace("alice");

    let test_file = root.join("test.txt");
    fs::write(&test_file, "content").expect("write failed");

    manifest::create_manifest(&root, "test", "alice", "Test").expect("create failed");
    manifest::add_entry(&root, "test", "Entry", "test.txt", None, None).expect("add failed");

    let results = manifest::verify_manifest(&root, "test").expect("verify failed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, VerifyResult::Fresh);
}

#[test]
fn verify_shifted() {
    let (_dir, root) = setup_workspace("alice");

    let test_file = root.join("test.txt");
    fs::write(&test_file, "original content").expect("write failed");

    manifest::create_manifest(&root, "test", "alice", "Test").expect("create failed");
    manifest::add_entry(&root, "test", "Entry", "test.txt", None, None).expect("add failed");

    // Modify the file
    fs::write(&test_file, "modified content").expect("write failed");

    let results = manifest::verify_manifest(&root, "test").expect("verify failed");
    assert_eq!(results[0].1, VerifyResult::Shifted);
}

#[test]
fn verify_stale() {
    let (_dir, root) = setup_workspace("alice");

    let test_file = root.join("test.txt");
    fs::write(&test_file, "fn main() {}").expect("write failed");

    manifest::create_manifest(&root, "test", "alice", "Test").expect("create failed");
    manifest::add_entry(
        &root,
        "test",
        "Entry",
        "test.txt",
        Some(r"fn nonexistent\(\)"),
        None,
    )
    .expect("add failed");

    let results = manifest::verify_manifest(&root, "test").expect("verify failed");
    assert_eq!(results[0].1, VerifyResult::Stale);
}

// ============================================================================
// 2.5 Notification Ops Tests
// ============================================================================

#[test]
fn push_notification() {
    let (_dir, root) = setup_workspace("alice");

    let notif = Notification {
        timestamp: Utc::now(),
        from: "alice".to_string(),
        thread_path: "dm/alice--bob/thread.md".to_string(),
        seq: 1,
        notify_type: NotifyType::Mention,
        snippet: "Hey @bob!".to_string(),
    };

    notify::push_notify(&root, "bob", &notif).expect("push_notify failed");

    let unread = notify::list_unread(&root, "bob").expect("list_unread failed");
    assert_eq!(unread.len(), 1);
    assert_eq!(unread[0].from, "alice");
    assert_eq!(unread[0].snippet, "Hey @bob!");
}

#[test]
fn ack_moves_to_history() {
    let (_dir, root) = setup_workspace("alice");

    // Push 2 notifications
    for i in 1..=2 {
        let notif = Notification {
            timestamp: Utc::now(),
            from: "alice".to_string(),
            thread_path: "dm/alice--bob/thread.md".to_string(),
            seq: i,
            notify_type: NotifyType::Mention,
            snippet: format!("Notification {}", i),
        };
        notify::push_notify(&root, "bob", &notif).expect("push_notify failed");
    }

    // Ack
    let acked = notify::ack_notify(&root, "bob").expect("ack_notify failed");
    assert_eq!(acked.len(), 2);

    // Unread should be empty
    let unread = notify::list_unread(&root, "bob").expect("list_unread failed");
    assert!(unread.is_empty());

    // History should have 2
    let history = notify::list_history(&root, "bob").expect("list_history failed");
    assert_eq!(history.len(), 2);
}

// ============================================================================
// 2.6 Who Query Tests
// ============================================================================

#[test]
fn who_owns_match() {
    let (_dir, root) = setup_workspace("alice");

    profile::edit_profile(
        &root,
        "alice",
        None,
        None,
        None,
        None,
        Some(vec!["src/**".to_string()]),
    )
    .expect("edit failed");

    let matches = contacts::who_query(&root, "src/main.rs", Access::Owns)
        .expect("who_query failed");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "alice");
}

#[test]
fn who_owns_no_match() {
    let (_dir, root) = setup_workspace("alice");

    profile::edit_profile(
        &root,
        "alice",
        None,
        None,
        None,
        None,
        Some(vec!["src/**".to_string()]),
    )
    .expect("edit failed");

    let matches = contacts::who_query(&root, "docs/readme.md", Access::Owns)
        .expect("who_query failed");
    assert!(matches.is_empty());
}

#[test]
fn who_reads_match() {
    let (_dir, root) = setup_workspace("alice");

    profile::edit_profile(
        &root,
        "alice",
        None,
        None,
        Some(vec!["docs/**".to_string()]),
        None,
        None,
    )
    .expect("edit failed");

    let matches = contacts::who_query(&root, "docs/guide.md", Access::Read)
        .expect("who_query failed");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "alice");
}

#[test]
fn who_writes_match() {
    let (_dir, root) = setup_workspace("alice");

    let bob = Profile {
        name: "bob".to_string(),
        model: "claude-3".to_string(),
        description: String::new(),
        scope_read: vec![],
        scope_write: vec!["src/lexer/**".to_string()],
        scope_owns: vec![],
    };
    profile::create_profile(&root, &bob).expect("create bob failed");

    let matches = contacts::who_query(&root, "src/lexer/token.rs", Access::Write)
        .expect("who_query failed");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "bob");
}

// ============================================================================
// 2.7 Invite & Contacts Ops Tests
// ============================================================================

#[test]
fn invite_creates_profile_and_dm() {
    let (_dir, root) = setup_workspace("alice");

    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    // Profile exists
    assert!(root.join(".paperwork/profiles/bob.md").is_file());

    // DM folder exists (alphabetically sorted)
    assert!(root.join(".paperwork/dm/alice--bob").is_dir());
    assert!(root.join(".paperwork/dm/alice--bob/meta.md").is_file());
    assert!(root.join(".paperwork/dm/alice--bob/thread.md").is_file());
}

#[test]
fn invite_dm_folder_alphabetical() {
    let (_dir, root) = setup_workspace("zara");

    contacts::invite(&root, "zara", "alice", "gpt-4").expect("invite failed");

    // Folder should be alice--zara (sorted)
    assert!(root.join(".paperwork/dm/alice--zara").is_dir());
    assert!(!root.join(".paperwork/dm/zara--alice").exists());
}

#[test]
fn invite_updates_contacts() {
    let (_dir, root) = setup_workspace("alice");

    contacts::invite(&root, "alice", "bob", "claude-3").expect("invite failed");

    let contacts = contacts::contacts_list(&root).expect("contacts_list failed");
    assert!(contacts.iter().any(|c| c.agent == "bob"));
}

#[test]
fn contacts_list() {
    let (_dir, root) = setup_workspace("alice");

    let bob = Profile {
        name: "bob".to_string(),
        model: "claude-3".to_string(),
        description: String::new(),
        scope_read: vec![],
        scope_write: vec![],
        scope_owns: vec![],
    };
    profile::create_profile(&root, &bob).expect("create bob failed");

    let contacts = contacts::contacts_list(&root).expect("contacts_list failed");
    assert_eq!(contacts.len(), 2);
    assert!(contacts.iter().any(|c| c.agent == "alice"));
    assert!(contacts.iter().any(|c| c.agent == "bob"));
}
