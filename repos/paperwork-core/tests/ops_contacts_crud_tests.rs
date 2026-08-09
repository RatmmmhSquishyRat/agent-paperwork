//! Core tests for the contacts CRUD round (spec cli-grammar-v0.6 §3.6/§3.9).
//!
//! New file by design (tdd §8.4): ops_tests.rs stays byte-for-byte frozen;
//! all new behavior is pinned here. Lock assertions are result-oriented
//! (entry sets / byte invariants), never coupled to lock internals.

use std::collections::BTreeSet;

use paperwork_core::format::contacts::{parse_contacts, parse_contacts_title, serialize_contacts};
use paperwork_core::format::manifest::serialize_manifest;
use paperwork_core::format::profile::serialize_profile;
use paperwork_core::ops::contacts::{
    contacts_add, contacts_create, contacts_read, contacts_remove, contacts_update,
};
use paperwork_core::ops::manifest::{brief_add_entry, brief_create, brief_read, brief_remove_entry};
use paperwork_core::ops::profile::{create_profile, edit_profile, show_profile};
use paperwork_core::PaperworkError;
use tempfile::TempDir;

/// Create a contacts file with the given profile entries (labels derived
/// from real profile files when they exist, else file-name stems).
fn setup_contacts(dir: &TempDir, title: &str, profiles: &[&str]) -> std::path::PathBuf {
    let path = dir.path().join("team.contacts.md");
    contacts_create(&path, title).unwrap();
    for p in profiles {
        contacts_add(&path, p).unwrap();
    }
    path
}

fn write_profile(dir: &TempDir, file_name: &str, name: &str) -> std::path::PathBuf {
    let p = dir.path().join(file_name);
    create_profile(&p, name, "gpt-4o", "").unwrap();
    p
}

// --- contacts_remove ---

#[test]
fn contacts_remove_hit_preserves_title_and_order() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md", "bob.profile.md", "carol.profile.md"]);

    contacts_remove(&path, "bob.profile.md").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(parse_contacts_title(&content).unwrap(), "Core Team");
    let entries = parse_contacts(&content).unwrap();
    let paths: Vec<&str> = entries.iter().map(|e| e.profile_path.as_str()).collect();
    assert_eq!(paths, vec!["alice.profile.md", "carol.profile.md"]);
}

#[test]
fn contacts_remove_miss_is_not_found_and_file_unchanged() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);
    let before = std::fs::read(&path).unwrap();

    let err = contacts_remove(&path, "ghost.profile.md").unwrap_err();
    match err {
        PaperworkError::NotFound { resource, name, .. } => {
            assert_eq!(resource, "Contacts entry");
            assert_eq!(name, "ghost.profile.md");
        }
        other => panic!("expected NotFound, got {:?}", other),
    }
    assert_eq!(std::fs::read(&path).unwrap(), before, "file bytes must not change");
}

#[test]
fn contacts_remove_missing_file_is_not_found_contacts() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("absent.contacts.md");

    let err = contacts_remove(&path, "alice.profile.md").unwrap_err();
    match err {
        PaperworkError::NotFound { resource, .. } => assert_eq!(resource, "Contacts"),
        other => panic!("expected NotFound, got {:?}", other),
    }
}

// --- contacts_update ---

#[test]
fn contacts_update_hit_redrives_label_and_keeps_order() {
    let dir = TempDir::new().unwrap();
    write_profile(&dir, "carol.profile.md", "carol");
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md", "bob.profile.md"]);

    contacts_update(&path, "alice.profile.md", "carol.profile.md").unwrap();

    let entries = contacts_read(&path).unwrap();
    assert_eq!(entries.len(), 2);
    // In-place replacement at index 0: order preserved.
    assert_eq!(entries[0].profile_path, "carol.profile.md");
    assert_eq!(entries[0].label, "carol", "label must be re-derived from NEW profile H1 (R11)");
    assert_eq!(entries[1].profile_path, "bob.profile.md");
}

#[test]
fn contacts_update_label_fallback_to_stem_when_new_unreadable() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);

    contacts_update(&path, "alice.profile.md", "ghost.profile.md").unwrap();

    let entries = contacts_read(&path).unwrap();
    assert_eq!(entries[0].profile_path, "ghost.profile.md");
    assert_eq!(entries[0].label, "ghost", "label must fall back to the file-name stem");
}

#[test]
fn contacts_update_old_miss_is_not_found_and_file_unchanged() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);
    let before = std::fs::read(&path).unwrap();

    let err = contacts_update(&path, "ghost.profile.md", "carol.profile.md").unwrap_err();
    match err {
        PaperworkError::NotFound { resource, name, .. } => {
            assert_eq!(resource, "Contacts entry");
            assert_eq!(name, "ghost.profile.md");
        }
        other => panic!("expected NotFound, got {:?}", other),
    }
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn contacts_update_missing_file_is_not_found_contacts() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("absent.contacts.md");

    let err = contacts_update(&path, "alice.profile.md", "carol.profile.md").unwrap_err();
    match err {
        PaperworkError::NotFound { resource, .. } => assert_eq!(resource, "Contacts"),
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[test]
fn contacts_update_new_already_exists_is_already_exists() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md", "bob.profile.md"]);
    let before = std::fs::read(&path).unwrap();

    let err = contacts_update(&path, "alice.profile.md", "bob.profile.md").unwrap_err();
    match err {
        PaperworkError::AlreadyExists { resource, name, .. } => {
            assert_eq!(resource, "Contacts entry");
            assert_eq!(name, "bob.profile.md");
        }
        other => panic!("expected AlreadyExists, got {:?}", other),
    }
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn contacts_update_old_equals_new_follows_judgment_order() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);

    // OLD == NEW and OLD is present -> AlreadyExists (OLD hit first, then
    // NEW-exists check fires).
    let err = contacts_update(&path, "alice.profile.md", "alice.profile.md").unwrap_err();
    assert!(matches!(err, PaperworkError::AlreadyExists { .. }), "OLD==NEW with OLD hit must be AlreadyExists");

    // OLD == NEW and OLD absent -> NotFound (OLD miss precedes NEW check).
    let err = contacts_update(&path, "ghost.profile.md", "ghost.profile.md").unwrap_err();
    assert!(matches!(err, PaperworkError::NotFound { .. }), "OLD==NEW with OLD miss must be NotFound");
}

#[test]
fn contacts_update_nonexistent_new_is_silent_success() {
    // spec §3.6 frozen silent surface (S-CONTACTS-14): destination written
    // as given, label falls back to the file-name stem, exit Ok.
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);

    contacts_update(&path, "alice.profile.md", "carol").unwrap();

    let entries = contacts_read(&path).unwrap();
    assert_eq!(entries[0].profile_path, "carol");
    assert_eq!(entries[0].label, "carol");
}

#[test]
fn contacts_remove_last_entry_matches_create_initial_shape() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Solo", &["alice.profile.md"]);

    contacts_remove(&path, "alice.profile.md").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, serialize_contacts("Solo", &[]), "must equal the create initial shape");
    assert!(parse_contacts(&content).unwrap().is_empty());

    // Removing the same key again: not-found, file unchanged.
    let before = std::fs::read(&path).unwrap();
    let err = contacts_remove(&path, "alice.profile.md").unwrap_err();
    assert!(matches!(err, PaperworkError::NotFound { .. }));
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn remove_update_roundtrip_with_special_character_paths() {
    let dir = TempDir::new().unwrap();
    let special = "my profile (v2).profile.md";
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md", special]);

    // Key matches on the UNESCAPED original string.
    contacts_update(&path, special, "new dir/prof.profile.md").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let entries = parse_contacts(&content).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].profile_path, "alice.profile.md");
    assert_eq!(entries[1].profile_path, "new dir/prof.profile.md");
    // New destination carries spaces -> angle-bracket serialized form.
    assert!(content.contains("<new dir/prof.profile.md>"), "space path must serialize in angle-bracket form");

    // Second operation on the same (unescaped) key still hits.
    contacts_remove(&path, "new dir/prof.profile.md").unwrap();
    let entries = contacts_read(&path).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].profile_path, "alice.profile.md");
}

#[test]
fn contacts_add_idempotent_after_lock_migration() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);
    let before = std::fs::read(&path).unwrap();

    contacts_add(&path, "alice.profile.md").unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), before, "idempotent add must be a byte-level no-op");
    assert_eq!(contacts_read(&path).unwrap().len(), 1);
}

#[test]
fn locked_writes_produce_bytes_identical_to_serialize_functions() {
    let dir = TempDir::new().unwrap();

    // contacts add: file bytes == serialize_contacts over the parsed view.
    let c_path = setup_contacts(&dir, "Team", &["alice.profile.md", "bob.profile.md"]);
    let content = std::fs::read_to_string(&c_path).unwrap();
    let expected = serialize_contacts("Team", &parse_contacts(&content).unwrap());
    assert_eq!(content, expected, "contacts add output must equal serialize_contacts");

    // contacts remove: same invariant after a removal.
    contacts_remove(&c_path, "alice.profile.md").unwrap();
    let content = std::fs::read_to_string(&c_path).unwrap();
    let expected = serialize_contacts("Team", &parse_contacts(&content).unwrap());
    assert_eq!(content, expected, "contacts remove output must equal serialize_contacts");

    // contacts update: same invariant after an in-place replacement.
    contacts_update(&c_path, "bob.profile.md", "carol.profile.md").unwrap();
    let content = std::fs::read_to_string(&c_path).unwrap();
    let expected = serialize_contacts("Team", &parse_contacts(&content).unwrap());
    assert_eq!(content, expected, "contacts update output must equal serialize_contacts");

    // brief add/remove: file bytes == serialize_manifest over the parsed view.
    let entry_file = dir.path().join("main.rs");
    std::fs::write(&entry_file, "fn main() {}\n").unwrap();
    let b_path = dir.path().join("onboarding.brief.md");
    brief_create(&b_path, "Onboarding", Some("alice"), "").unwrap();
    brief_add_entry(&b_path, "main.rs", None, None).unwrap();
    let manifest = brief_read(&b_path).unwrap();
    assert_eq!(
        std::fs::read_to_string(&b_path).unwrap(),
        serialize_manifest(&manifest),
        "brief add output must equal serialize_manifest"
    );
    let entry_file2 = dir.path().join("lib.rs");
    std::fs::write(&entry_file2, "pub fn lib() {}\n").unwrap();
    brief_add_entry(&b_path, "lib.rs", None, None).unwrap();
    brief_remove_entry(&b_path, "main.rs").unwrap();
    let manifest = brief_read(&b_path).unwrap();
    assert_eq!(
        std::fs::read_to_string(&b_path).unwrap(),
        serialize_manifest(&manifest),
        "brief remove output must equal serialize_manifest"
    );

    // profile edit: file bytes == serialize_profile over the parsed view.
    let p_path = dir.path().join("alice.profile.md");
    create_profile(&p_path, "alice", "gpt-4o", "old desc").unwrap();
    edit_profile(&p_path, Some("claude-3"), None, None, None, None).unwrap();
    let profile = show_profile(&p_path).unwrap();
    assert_eq!(
        std::fs::read_to_string(&p_path).unwrap(),
        serialize_profile(&profile),
        "profile edit output must equal serialize_profile"
    );
}

#[test]
fn multithread_concurrent_add_remove_loses_no_entries() {
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Team", &[]);
    let n = 8usize;

    // N concurrent adds with distinct profile paths.
    let mut handles = Vec::new();
    for i in 0..n {
        let p = path.clone();
        handles.push(std::thread::spawn(move || {
            contacts_add(&p, &format!("p{}.profile.md", i)).unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    let entries = contacts_read(&path).unwrap();
    assert_eq!(entries.len(), n, "no add may be lost");
    let got: BTreeSet<String> = entries.iter().map(|e| e.profile_path.clone()).collect();
    let want: BTreeSet<String> = (0..n).map(|i| format!("p{}.profile.md", i)).collect();
    assert_eq!(got, want, "entry set must equal the expected set");

    // Concurrent removes over the first half; the result must still parse.
    let mut handles = Vec::new();
    for i in 0..(n / 2) {
        let p = path.clone();
        handles.push(std::thread::spawn(move || {
            contacts_remove(&p, &format!("p{}.profile.md", i)).unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    let content = std::fs::read_to_string(&path).unwrap();
    let entries = parse_contacts(&content).expect("file must parse after concurrent writes");
    let got: BTreeSet<String> = entries.iter().map(|e| e.profile_path.clone()).collect();
    let want: BTreeSet<String> = (n / 2..n).map(|i| format!("p{}.profile.md", i)).collect();
    assert_eq!(got, want, "remaining entry set must equal the expected set");
}

// --- empty-key validation guard (review Kim M-1 / QA BUG-1) ---

#[test]
fn contacts_add_empty_profile_is_validation_error() {
    // Library-direct guard: an empty/whitespace-only profile path is
    // refused before any file access (previously it silently wrote the
    // unparseable bullet `- []()`).
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);
    let before = std::fs::read(&path).unwrap();

    for empty in ["", "   "] {
        let err = contacts_add(&path, empty).unwrap_err();
        match err {
            PaperworkError::Validation { message, .. } => {
                assert_eq!(message, "profile path (--profile) is empty");
            }
            other => panic!("expected Validation, got {:?}", other),
        }
    }
    assert_eq!(std::fs::read(&path).unwrap(), before, "refused call must not write");
    assert_eq!(contacts_read(&path).unwrap().len(), 1);
}

#[test]
fn contacts_update_empty_keys_are_validation_errors() {
    // Both --profile and --new-profile are guarded at the core entry.
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);
    let before = std::fs::read(&path).unwrap();

    let err = contacts_update(&path, "  ", "carol.profile.md").unwrap_err();
    match err {
        PaperworkError::Validation { message, .. } => {
            assert_eq!(message, "profile path (--profile) is empty");
        }
        other => panic!("expected Validation, got {:?}", other),
    }

    let err = contacts_update(&path, "alice.profile.md", "").unwrap_err();
    match err {
        PaperworkError::Validation { message, .. } => {
            assert_eq!(message, "new profile path (--new-profile) is empty");
        }
        other => panic!("expected Validation, got {:?}", other),
    }

    assert_eq!(std::fs::read(&path).unwrap(), before, "refused calls must not write");
}

// --- lock helper no-change skip (review Kim m-1 / Oscar M-2) ---

#[test]
fn idempotent_add_keeps_bytes_and_mtime_stable() {
    // The locked helper skips truncate+rewrite when the closure returns
    // unchanged content: an idempotent add must be a true zero-write
    // (bytes AND mtime stable), restoring the pre-lock baseline semantics.
    let dir = TempDir::new().unwrap();
    let path = setup_contacts(&dir, "Core Team", &["alice.profile.md"]);

    let meta_before = std::fs::metadata(&path).unwrap();
    let bytes_before = std::fs::read(&path).unwrap();
    let mtime_before = meta_before.modified().unwrap();

    contacts_add(&path, "alice.profile.md").unwrap();

    let meta_after = std::fs::metadata(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        bytes_before,
        "idempotent add must keep file bytes identical"
    );
    assert_eq!(
        meta_after.modified().unwrap(),
        mtime_before,
        "idempotent add must not rewrite the file (mtime must stay stable)"
    );
}
