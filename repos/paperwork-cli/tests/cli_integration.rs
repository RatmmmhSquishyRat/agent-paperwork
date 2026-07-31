//! Integration tests for the stateless paperwork CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("paperwork").unwrap()
}

// ─── Profile ────────────────────────────────────────────────────────────────

#[test]
fn profile_create_and_show() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("alice.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "--name", "alice", "--model", "gpt-4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{2713}"));

    cmd()
        .args(["profile", "show", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"));
}

#[test]
fn profile_create_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bob.md");

    cmd()
        .args(["--json", "profile", "create", path.to_str().unwrap(), "--name", "bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"bob\""));
}

#[test]
fn profile_create_duplicate_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("dup.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "--name", "x"])
        .assert()
        .success();

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "--name", "y"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("\u{2717}"));
}

#[test]
fn profile_edit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("edit.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "--name", "agent"])
        .assert()
        .success();

    cmd()
        .args(["profile", "edit", path.to_str().unwrap(), "--model", "claude-3"])
        .assert()
        .success();

    cmd()
        .args(["--json", "profile", "show", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude-3"));
}

#[test]
fn profile_list() {
    let dir = TempDir::new().unwrap();
    let p1 = dir.path().join("a.md");
    let p2 = dir.path().join("b.md");

    cmd().args(["profile", "create", p1.to_str().unwrap(), "--name", "a"]).assert().success();
    cmd().args(["profile", "create", p2.to_str().unwrap(), "--name", "b"]).assert().success();

    cmd()
        .args(["--json", "profile", "list", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("a.md"))
        .stdout(predicate::str::contains("b.md"));
}

// ─── Post ───────────────────────────────────────────────────────────────────

#[test]
fn post_create_send_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("thread.md");

    cmd()
        .args(["post", "create", path.to_str().unwrap(), "--title", "Design Discussion"])
        .assert()
        .success();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--from", "alice", "I think we should use Rust."])
        .assert()
        .success();

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn post_edit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("edit-thread.md");

    cmd().args(["post", "create", path.to_str().unwrap(), "--title", "T"]).assert().success();
    cmd().args(["post", "send", path.to_str().unwrap(), "--from", "bob", "original"]).assert().success();

    // Edit the last message (seq 2, since create is seq 1)
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--seq", "2", "--from", "bob", "edited"])
        .assert()
        .success();

    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--from", "2", "--to", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("edited"));
}

#[test]
fn post_summary() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("sum.md");

    cmd().args(["post", "create", path.to_str().unwrap(), "--title", "S"]).assert().success();
    cmd().args(["post", "send", path.to_str().unwrap(), "--from", "x", "hello"]).assert().success();

    cmd()
        .args(["--json", "post", "summary", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"message_count\": 2"));
}

// ─── Brief ──────────────────────────────────────────────────────────────────

#[test]
fn brief_create_add_read() {
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("brief.md");
    let entry_file = dir.path().join("notes.txt");

    std::fs::write(&entry_file, "some content").unwrap();

    cmd()
        .args(["brief", "create", brief_path.to_str().unwrap(), "--title", "My Brief"])
        .assert()
        .success();

    cmd()
        .args(["brief", "add", brief_path.to_str().unwrap(), "--entry", "notes.txt"])
        .assert()
        .success();

    cmd()
        .args(["brief", "read", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("notes.txt"));
}

#[test]
fn brief_remove() {
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("b.md");
    let entry_file = dir.path().join("e.txt");

    std::fs::write(&entry_file, "data").unwrap();

    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "--title", "B"]).assert().success();
    cmd().args(["brief", "add", brief_path.to_str().unwrap(), "--entry", "e.txt"]).assert().success();

    cmd()
        .args(["brief", "remove", brief_path.to_str().unwrap(), "--entry-title", "e.txt"])
        .assert()
        .success();

    cmd()
        .args(["--json", "brief", "read", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entries\": []"));
}

#[test]
fn brief_verify() {
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("v.md");
    let entry_file = dir.path().join("src.txt");

    std::fs::write(&entry_file, "original").unwrap();

    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "--title", "V"]).assert().success();
    cmd().args(["brief", "add", brief_path.to_str().unwrap(), "--entry", "src.txt"]).assert().success();

    // Verify fresh
    cmd()
        .args(["--json", "brief", "verify", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fresh"));

    // Modify file → shifted
    std::fs::write(&entry_file, "modified").unwrap();
    cmd()
        .args(["--json", "brief", "verify", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Shifted"));
}

// ─── Contacts ───────────────────────────────────────────────────────────────

#[test]
fn contacts_create_add_read() {
    let dir = TempDir::new().unwrap();
    let contacts_path = dir.path().join("contacts.md");
    let profile_path = dir.path().join("agent.md");

    cmd().args(["profile", "create", profile_path.to_str().unwrap(), "--name", "agent"]).assert().success();

    cmd()
        .args(["contacts", "create", contacts_path.to_str().unwrap(), "--title", "Team"])
        .assert()
        .success();

    cmd()
        .args(["contacts", "add", contacts_path.to_str().unwrap(), "--profile", profile_path.to_str().unwrap()])
        .assert()
        .success();

    cmd()
        .args(["--json", "contacts", "read", contacts_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent.md"));
}

// ─── Notify ─────────────────────────────────────────────────────────────────

#[test]
fn notify_push_and_read() {
    let dir = TempDir::new().unwrap();
    let notify_path = dir.path().join("alice.notify.md");

    cmd()
        .args([
            "notify", "push", notify_path.to_str().unwrap(),
            "--from", "bob",
            "--thread", "some/thread.md",
            "--seq", "5",
            "--type", "mention",
            "--snippet", "Hey @alice",
        ])
        .assert()
        .success();

    cmd()
        .args(["--json", "notify", "read", notify_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("bob"))
        .stdout(predicate::str::contains("Mention"));
}

#[test]
fn notify_read_empty() {
    let dir = TempDir::new().unwrap();
    let notify_path = dir.path().join("empty.notify.md");

    cmd()
        .args(["--json", "notify", "read", notify_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

// ─── Global flags ───────────────────────────────────────────────────────────

#[test]
fn quiet_suppresses_success() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("q.md");

    cmd()
        .args(["--quiet", "profile", "create", path.to_str().unwrap(), "--name", "q"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn error_exit_code_1() {
    cmd()
        .args(["profile", "show", "nonexistent/path/file.md"])
        .assert()
        .code(1);
}
