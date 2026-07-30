//! Integration tests for paperwork-cli.
//!
//! Exercises the full workflow:
//! init → invite → dm send → dm read → post create → post send →
//! manifest create → manifest add → manifest verify → notify → who

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn paperwork() -> Command {
    Command::cargo_bin("paperwork").expect("binary exists")
}

#[test]
fn test_full_workflow() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_str().expect("valid path");

    // 1. Init
    paperwork()
        .args(["--root", root, "init", "--name", "alice", "--model", "gpt-4o"])
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"));

    // Verify .paperwork/ exists
    assert!(tmp.path().join(".paperwork").exists());
    assert!(tmp.path().join(".paperwork/profiles/alice.md").exists());
    assert!(tmp.path().join(".paperwork/contacts.md").exists());

    // 2. Init idempotent
    paperwork()
        .args(["--root", root, "init", "--name", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already initialized"));

    // 3. Invite bob
    paperwork()
        .args(["--root", root, "invite", "bob", "--model", "claude-sonnet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("invited bob"));

    // Verify bob profile and DM folder
    assert!(tmp.path().join(".paperwork/profiles/bob.md").exists());
    assert!(tmp.path().join(".paperwork/dm/alice--bob").exists());

    // 4. Invite idempotent
    paperwork()
        .args(["--root", root, "invite", "bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already invited"));

    // 5. Contacts
    paperwork()
        .args(["--root", root, "contacts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"))
        .stdout(predicate::str::contains("bob"));

    // 6. DM send
    paperwork()
        .args(["--root", root, "dm", "bob", "send", "Hey, workspace is ready."])
        .assert()
        .success()
        .stdout(predicate::str::contains("sent #1"));

    // 7. DM send second message
    paperwork()
        .args(["--root", root, "dm", "bob", "send", "Check the parser module."])
        .assert()
        .success()
        .stdout(predicate::str::contains("sent #2"));

    // 8. DM read
    paperwork()
        .args(["--root", root, "dm", "bob", "read"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hey, workspace is ready."))
        .stdout(predicate::str::contains("Check the parser module."));

    // 9. DM read --json
    paperwork()
        .args(["--root", root, "--json", "dm", "bob", "read"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\": 2"))
        .stdout(predicate::str::contains("\"seq\": 1"));

    // 10. DM summary
    paperwork()
        .args(["--root", root, "dm", "bob", "summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 messages"));

    // 11. DM send with mention (triggers notification)
    paperwork()
        .args([
            "--root", root, "dm", "bob", "send",
            "@bob fixtures should cover edge cases.",
            "--mention", "bob",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("sent #3"));

    // 12. Post create
    paperwork()
        .args([
            "--root", root, "post", "create", "standup",
            "--participants", "alice,bob",
            "--title", "Daily Standup",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("post created"));

    // 13. Post send
    paperwork()
        .args(["--root", root, "post", "standup", "send", "Shipped manifest verify."])
        .assert()
        .success()
        .stdout(predicate::str::contains("sent #1"));

    // 14. Post read
    paperwork()
        .args(["--root", root, "post", "standup", "read"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Shipped manifest verify."));

    // 15. Post summary
    paperwork()
        .args(["--root", root, "post", "standup", "summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Daily Standup"))
        .stdout(predicate::str::contains("messages: 1"));

    // 16. Post list
    paperwork()
        .args(["--root", root, "post", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("standup"));

    // 17. Manifest create
    paperwork()
        .args([
            "--root", root, "manifest", "create", "onboarding",
            "--description", "How to understand this codebase",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("manifest created"));

    // 18. Create a test file to add to manifest
    std::fs::write(tmp.path().join("test_file.rs"), "pub fn hello() {}").expect("write test file");

    // 19. Manifest add
    paperwork()
        .args([
            "--root", root, "manifest", "onboarding", "add",
            "--path", "test_file.rs",
            "--note", "Entry point",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("entry added"));

    // 20. Manifest read
    paperwork()
        .args(["--root", root, "manifest", "onboarding", "read"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test_file.rs"));

    // 21. Manifest verify (should be fresh)
    paperwork()
        .args(["--root", root, "manifest", "onboarding", "verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("FRESH"));

    // 22. Modify file → verify shows shifted
    std::fs::write(tmp.path().join("test_file.rs"), "pub fn hello() { println!(\"hi\"); }")
        .expect("modify test file");

    paperwork()
        .args(["--root", root, "manifest", "onboarding", "verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SHIFTED"));

    // 23. Manifest list
    paperwork()
        .args(["--root", root, "manifest", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("onboarding"));

    // 24. Notify view (bob has a notification from mention)
    paperwork()
        .args(["--root", root, "notify", "--agent", "bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 unread"));

    // 25. Notify ack
    paperwork()
        .args(["--root", root, "notify", "--agent", "bob", "--ack"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 notifications acknowledged"));

    // 26. Notify after ack (0 unread)
    paperwork()
        .args(["--root", root, "notify", "--agent", "bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 unread"));

    // 27. Profile edit + who query
    paperwork()
        .args([
            "--root", root, "profile", "edit", "alice",
            "--scope-owns", "src/parser/**",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("profile updated"));

    // 28. Who --owns
    paperwork()
        .args(["--root", root, "who", "--owns", "src/parser/**"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"));

    // 29. Profile show
    paperwork()
        .args(["--root", root, "profile", "show", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"))
        .stdout(predicate::str::contains("gpt-4o"));

    // 30. Profile list
    paperwork()
        .args(["--root", root, "profile", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"))
        .stdout(predicate::str::contains("bob"));
}

#[test]
fn test_json_output() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_str().expect("valid path");

    // Init with JSON
    paperwork()
        .args(["--root", root, "--json", "init", "--name", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"created\""))
        .stdout(predicate::str::contains("\"profile\""));

    // Invite with JSON
    paperwork()
        .args(["--root", root, "--json", "invite", "bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"invited\""))
        .stdout(predicate::str::contains("\"dm_folder\""));

    // DM send with JSON
    paperwork()
        .args(["--root", root, "--json", "dm", "bob", "send", "Hello JSON"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"seq\""))
        .stdout(predicate::str::contains("\"thread\""));

    // Contacts with JSON
    paperwork()
        .args(["--root", root, "--json", "contacts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"agent\""));
}

#[test]
fn test_error_handling() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_str().expect("valid path");

    // Operations on uninitialized workspace should fail
    paperwork()
        .args(["--root", root, "contacts"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));

    // Init first
    paperwork()
        .args(["--root", root, "init", "--name", "alice"])
        .assert()
        .success();

    // DM to non-invited agent should fail
    paperwork()
        .args(["--root", root, "dm", "charlie", "send", "Hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));

    // Profile show non-existent
    paperwork()
        .args(["--root", root, "profile", "show", "nobody"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));

    // Manifest operations on non-existent
    paperwork()
        .args(["--root", root, "manifest", "ghost", "read"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_dm_edit() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_str().expect("valid path");

    paperwork()
        .args(["--root", root, "init", "--name", "alice"])
        .assert()
        .success();

    paperwork()
        .args(["--root", root, "invite", "bob"])
        .assert()
        .success();

    // Send a message
    paperwork()
        .args(["--root", root, "dm", "bob", "send", "Original message"])
        .assert()
        .success();

    // Edit own last message
    paperwork()
        .args(["--root", root, "dm", "bob", "edit", "1", "Edited message"])
        .assert()
        .success()
        .stdout(predicate::str::contains("edited #1"));

    // Verify edit
    paperwork()
        .args(["--root", root, "dm", "bob", "read"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Edited message"));
}

#[test]
fn test_manifest_remove() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_str().expect("valid path");

    paperwork()
        .args(["--root", root, "init", "--name", "alice"])
        .assert()
        .success();

    // Create test file
    std::fs::write(tmp.path().join("lib.rs"), "fn main() {}").expect("write");

    paperwork()
        .args(["--root", root, "manifest", "create", "test-manifest"])
        .assert()
        .success();

    paperwork()
        .args([
            "--root", root, "manifest", "test-manifest", "add",
            "--path", "lib.rs",
        ])
        .assert()
        .success();

    // Remove entry
    paperwork()
        .args(["--root", root, "manifest", "test-manifest", "remove", "lib.rs"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entry removed"));

    // Verify removed
    paperwork()
        .args(["--root", root, "manifest", "test-manifest", "read"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 entries"));
}

#[test]
fn test_quiet_mode() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_str().expect("valid path");

    // Quiet mode suppresses success messages
    paperwork()
        .args(["--root", root, "-q", "init", "--name", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_plain_output() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_str().expect("valid path");

    paperwork()
        .args(["--root", root, "init", "--name", "alice"])
        .assert()
        .success();

    // Plain mode shows raw file content
    paperwork()
        .args(["--root", root, "--plain", "profile", "show", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# alice"));
}

#[test]
fn test_post_full_workflow_json() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_str().expect("valid path");

    paperwork()
        .args(["--root", root, "init", "--name", "alice"])
        .assert()
        .success();

    paperwork()
        .args(["--root", root, "invite", "bob"])
        .assert()
        .success();

    // Create post
    paperwork()
        .args([
            "--root", root, "--json", "post", "create", "dev",
            "--participants", "alice,bob",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"created\""));

    // Send to post
    paperwork()
        .args(["--root", root, "--json", "post", "dev", "send", "Hello team"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"seq\": 1"));

    // Read post JSON
    paperwork()
        .args(["--root", root, "--json", "post", "dev", "read"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"messages\""))
        .stdout(predicate::str::contains("Hello team"));

    // Summary JSON
    paperwork()
        .args(["--root", root, "--json", "post", "dev", "summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"message_count\": 1"));
}
