//! Integration tests for the stateless paperwork CLI.
//!
//! v0.5.0 grammar: PATH is always the first required positional argument;
//! NAME is the second positional for post send/edit; content is last.
//! Usage errors (clap parse failures) render as the `usage` envelope, exit 2;
//! runtime errors keep the six categories, exit 1.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("paperwork").unwrap()
}

/// Minimal valid single-message thread corpus.
fn thread_message(seq: u64, sender: &str, to: &str, reply_to: Option<u64>, mentions: &[&str], body: &str) -> String {
    let mut out = format!("---\n\n### #{} {} · 2026-01-15T10:30:00Z\n\n- To: {}\n", seq, sender, to);
    if let Some(r) = reply_to {
        out.push_str(&format!("- Reply-To: #{}\n", r));
    }
    if !mentions.is_empty() {
        out.push_str(&format!("- Mentions: {}\n", mentions.join(", ")));
    }
    out.push_str(&format!("\n````markdown\n{}\n````\n\n", body));
    out
}

// --- Profile ---

#[test]
fn profile_create_and_show() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("alice.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "alice", "--model", "gpt-4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok profile.create"))
        .stdout(predicate::str::contains("name: alice"));

    cmd()
        .args(["profile", "show", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok profile.show"))
        .stdout(predicate::str::contains("name: alice"));
}

#[test]
fn profile_create_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bob.md");

    cmd()
        .args(["--json", "profile", "create", path.to_str().unwrap(), "bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\":\"bob\""))
        .stdout(predicate::str::contains("\"status\":\"ok\""));
}

#[test]
fn profile_create_duplicate_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("dup.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "x"])
        .assert()
        .success();

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "y"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error already-exists:"));
}

#[test]
fn profile_edit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("edit.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "agent"])
        .assert()
        .success();

    cmd()
        .args(["profile", "edit", path.to_str().unwrap(), "--model", "claude-3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok profile.edit"))
        .stdout(predicate::str::contains("changed: model"));

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

    cmd().args(["profile", "create", p1.to_str().unwrap(), "a"]).assert().success();
    cmd().args(["profile", "create", p2.to_str().unwrap(), "b"]).assert().success();

    cmd()
        .args(["--json", "profile", "list", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("a.profile.md"))
        .stdout(predicate::str::contains("b.profile.md"));
}

// --- Post ---

#[test]
fn post_create_send_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("thread.md");

    cmd()
        .args(["post", "create", path.to_str().unwrap(), "Design Discussion"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.create"));

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice", "I think we should use Rust."])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.send"));

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.read"))
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn post_send_stdin() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("stdin-thread.md");

    cmd()
        .args(["post", "create", path.to_str().unwrap(), "T"])
        .assert()
        .success();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice", "--stdin"])
        .write_stdin("Message from stdin")
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.send"));

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Message from stdin"));
}

#[test]
fn post_send_empty_body_rejected() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("empty.md");

    cmd()
        .args(["post", "create", path.to_str().unwrap(), "T"])
        .assert()
        .success();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice", "   "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"));
}

#[test]
fn post_edit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("edit-thread.md");

    cmd().args(["post", "create", path.to_str().unwrap(), "T"]).assert().success();
    cmd().args(["post", "send", path.to_str().unwrap(), "bob", "original"]).assert().success();

    // Edit the last message (seq 2, since create is seq 1)
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "bob", "2", "edited"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.edit"));

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

    cmd().args(["post", "create", path.to_str().unwrap(), "S"]).assert().success();
    cmd().args(["post", "send", path.to_str().unwrap(), "x", "hello"]).assert().success();

    cmd()
        .args(["--json", "post", "summary", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"messages\":2"));
}

// --- Brief ---

#[test]
fn brief_create_add_read() {
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("brief.md");
    let entry_file = dir.path().join("notes.txt");

    std::fs::write(&entry_file, "some content").unwrap();

    cmd()
        .args(["brief", "create", brief_path.to_str().unwrap(), "My Brief"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok brief.create"));

    cmd()
        .args(["brief", "add", brief_path.to_str().unwrap(), "notes.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok brief.add"));

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

    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "B"]).assert().success();
    cmd().args(["brief", "add", brief_path.to_str().unwrap(), "e.txt"]).assert().success();

    cmd()
        .args(["brief", "remove", brief_path.to_str().unwrap(), "e.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok brief.remove"));

    cmd()
        .args(["--json", "brief", "read", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entries\":[]"));
}

#[test]
fn brief_verify() {
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("v.md");
    let entry_file = dir.path().join("src.txt");

    std::fs::write(&entry_file, "original").unwrap();

    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "V"]).assert().success();
    cmd().args(["brief", "add", brief_path.to_str().unwrap(), "src.txt"]).assert().success();

    // Verify fresh
    cmd()
        .args(["--json", "brief", "verify", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("fresh"));

    // Modify file -> shifted
    std::fs::write(&entry_file, "modified").unwrap();
    cmd()
        .args(["--json", "brief", "verify", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("shifted"));
}

// --- Contacts ---

#[test]
fn contacts_create_add_read() {
    let dir = TempDir::new().unwrap();
    let contacts_path = dir.path().join("contacts.md");
    let profile_path = dir.path().join("agent.md");

    cmd().args(["profile", "create", profile_path.to_str().unwrap(), "agent"]).assert().success();

    cmd()
        .args(["contacts", "create", contacts_path.to_str().unwrap(), "--title", "Team"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok contacts.create"));

    cmd()
        .args(["contacts", "add", contacts_path.to_str().unwrap(), profile_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok contacts.add"));

    cmd()
        .args(["--json", "contacts", "read", contacts_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent"));
}

// --- Validate ---

#[test]
fn validate_post_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("thread.md");

    cmd().args(["post", "create", path.to_str().unwrap(), "T"]).assert().success();

    // The actual file created has .post.md suffix
    let actual_path = dir.path().join("thread.post.md");
    cmd()
        .args(["validate", actual_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok validate"));
}

#[test]
fn validate_unknown_suffix() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("random.txt");
    std::fs::write(&path, "hello").unwrap();

    cmd()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error format:"));
}

// --- Global flags ---

#[test]
fn quiet_suppresses_status_line() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("q.md");

    // Quiet suppresses the "ok" status line but still outputs fields
    cmd()
        .args(["--quiet", "profile", "create", path.to_str().unwrap(), "q"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: q"))
        .stdout(predicate::str::contains("ok").not());
}

#[test]
fn error_exit_code_1() {
    cmd()
        .args(["profile", "show", "nonexistent/path/file.md"])
        .assert()
        .code(1);
}

#[test]
fn json_error_on_stdout() {
    cmd()
        .args(["--json", "profile", "show", "nonexistent/path/file.md"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"status\":\"error\""))
        .stdout(predicate::str::contains("\"exit_code\":1"));
}

// =====================================================================
// v0.5.0 new cases (tdd §3)
// =====================================================================

// --- Usage envelope (seventh category, exit 2) ---

#[test]
fn usage_missing_body_post_send() {
    // S-SEND-08: only PATH given, NAME and BODY both missing
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap()])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("example:"))
        .stderr(predicate::str::contains("paperwork post send standup.post.md alice"));
}

#[test]
fn name_body_confusion_single_string() {
    // S-SEND-12: PATH + one string -> falls into validation (empty NAME slot),
    // exit 1, fix teaches `--` (F1 ruling: not a usage error)
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "some body text"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error validation:"))
        .stderr(predicate::str::contains("NAME"))
        .stderr(predicate::str::contains("--"));
}

#[test]
fn usage_old_grammar_send_from() {
    // S-SEND-09: old-grammar --from flag -> usage envelope, canonical example
    // must not carry user argv values (F2 ruling)
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--from", "alice", "body"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork post send standup.post.md alice"));
}

#[test]
fn usage_old_grammar_profile_create_name() {
    // S-PROF-03
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("a.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "--name", "alice"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork profile create agents/alice alice"));
}

#[test]
fn usage_old_grammar_brief_add_entry() {
    // S-BRIEF-03
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("b.md");
    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "B"]).assert().success();

    cmd()
        .args(["brief", "add", brief_path.to_str().unwrap(), "--entry", "e.txt"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork brief add onboarding.brief.md src/main.rs"));
}

#[test]
fn usage_old_grammar_contacts_add_profile() {
    // S-CONTACTS-03
    let dir = TempDir::new().unwrap();
    let contacts_path = dir.path().join("c.md");
    cmd().args(["contacts", "create", contacts_path.to_str().unwrap()]).assert().success();

    cmd()
        .args(["contacts", "add", contacts_path.to_str().unwrap(), "--profile", "x.profile.md"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork contacts add team.contacts.md"));
}

#[test]
fn usage_old_grammar_post_edit_seq() {
    // S-EDIT-04
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--seq", "1", "--from", "alice", "new"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork post edit standup.post.md alice 3"));
}

#[test]
fn usage_seq_not_numeric() {
    // S-EDIT-03
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "alice", "abc", "new"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));
}

#[test]
fn usage_extra_positional_send() {
    // S-SEND-13: four positional values for send
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice", "body", "extra"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));
}

#[test]
fn json_usage_error_on_stdout() {
    // S-OUT-03: --json usage error -> stdout JSON, category usage, exit_code 2
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["--json", "post", "send", path.to_str().unwrap()])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"category\":\"usage\""))
        .stdout(predicate::str::contains("\"command\":\"post.send\""))
        .stdout(predicate::str::contains("\"example\""))
        .stdout(predicate::str::contains("\"exit_code\":2"));
}

#[test]
fn json_runtime_error_has_command_field() {
    // S-OUT-02: runtime error JSON gains the `command` field (additive)
    cmd()
        .args(["--json", "post", "read", "nonexistent/path/file.md"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"status\":\"error\""))
        .stdout(predicate::str::contains("\"command\":\"post.read\""))
        .stdout(predicate::str::contains("\"exit_code\":1"));
}

#[test]
fn top_level_parse_failure_command_usage() {
    // S-OUT-06: group/verb layer failure -> command identifier is "usage"
    cmd()
        .args(["post"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));

    cmd()
        .args(["--json", "post"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"command\":\"usage\""))
        .stdout(predicate::str::contains("\"category\":\"usage\""))
        .stdout(predicate::str::contains("\"exit_code\":2"));
}

#[test]
fn missing_subcommand_message_shape() {
    // B1 (Ryan W1 / QA BUG-2): bare group and top-level invocations must
    // carry an explicit "missing subcommand" message (never about text),
    // identical in the default and --json envelopes.
    let group_out = cmd().args(["post"]).assert().code(2);
    let group_err = String::from_utf8_lossy(&group_out.get_output().stderr).to_string();
    assert!(
        group_err.contains("missing subcommand for group 'post'"),
        "group-level message wrong: {}",
        group_err
    );

    cmd()
        .args(["--json", "post"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("missing subcommand for group 'post'"));

    let top_out = cmd().output().unwrap();
    assert_eq!(top_out.status.code(), Some(2));
    let top_err = String::from_utf8_lossy(&top_out.stderr).to_string();
    assert!(
        top_err.contains("missing subcommand: expected one of profile, post, brief, contacts, validate"),
        "top-level message wrong: {}",
        top_err
    );

    cmd()
        .args(["--json"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(
            "missing subcommand: expected one of profile, post, brief, contacts, validate",
        ));
}

#[test]
fn usage_missing_required_argument_full_message() {
    // B1 (Ryan W1): the missing-argument list must survive into the message
    // (no truncation at the first rendered line).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    let out = cmd()
        .args(["post", "send", path.to_str().unwrap()])
        .assert()
        .code(2)
        .get_output()
        .clone();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        err.contains("the following required arguments were not provided: <NAME>"),
        "message must list the missing arguments: {}",
        err
    );

    // --json carries the same complete message
    cmd()
        .args(["--json", "post", "send", path.to_str().unwrap()])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(
            "the following required arguments were not provided: <NAME>",
        ));
}

#[test]
fn help_and_version_pass_through_exit_0() {
    // S-OUT-07 (F5 freeze): --help at all levels and -V keep exit 0
    cmd()
        .arg("--help")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Usage:"));

    cmd()
        .args(["post", "send", "--help"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Examples:"));

    cmd()
        .arg("-V")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("paperwork"));
}

// --- Three-stage path resolution ---

#[test]
fn path_original_file_wins() {
    // S-PATH-01: an existing bare x.md is used as-is, never rewritten
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("x.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "bare file content")).unwrap();

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("bare file content"));
}

#[test]
fn path_suffix_fallback() {
    // S-PATH-02: bare name resolves to the suffixed variant when only it exists
    let dir = TempDir::new().unwrap();
    let suffixed = dir.path().join("standup.post.md");
    std::fs::write(&suffixed, thread_message(1, "alice", "all", None, &[], "suffixed content")).unwrap();

    cmd()
        .args(["post", "read", dir.path().join("standup").to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("suffixed content"));
}

#[test]
fn path_both_exist_original_wins() {
    // S-PATH-05: x.md and x.post.md both exist -> x.md wins
    let dir = TempDir::new().unwrap();
    let original = dir.path().join("x.md");
    let suffixed = dir.path().join("x.post.md");
    std::fs::write(&original, thread_message(1, "alice", "all", None, &[], "original content")).unwrap();
    std::fs::write(&suffixed, thread_message(1, "bob", "all", None, &[], "suffixed content")).unwrap();

    cmd()
        .args(["post", "read", original.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("original content"))
        .stdout(predicate::str::contains("suffixed content").not());
}

#[test]
fn path_send_creates_suffixed_landing() {
    // S-PATH-06: neither path exists -> send creates the suffixed variant
    let dir = TempDir::new().unwrap();
    let bare = dir.path().join("newthread");

    cmd()
        .args(["post", "send", bare.to_str().unwrap(), "alice", "landing content"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok post.send"));

    assert!(dir.path().join("newthread.post.md").is_file());
    assert!(!dir.path().join("newthread").exists());
}

#[test]
fn path_stage1_foreign_file_format_error_no_reroute() {
    // S-PATH-07 (F4): stage-1 hit on a non-thread file -> format error,
    // no re-routing, no notes.post.md creation
    let dir = TempDir::new().unwrap();
    let notes = dir.path().join("notes.md");
    std::fs::write(&notes, "just some plain notes").unwrap();

    cmd()
        .args(["post", "send", notes.to_str().unwrap(), "alice", "attempt"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error format:"));

    assert!(!dir.path().join("notes.post.md").exists());
}

#[test]
fn path_directory_never_matches_stage1() {
    // S-PATH-08: an existing directory is not is_file() -> read reports
    // not-found and nothing is created
    let dir = TempDir::new().unwrap();
    let subdir = dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    cmd()
        .args(["post", "read", subdir.to_str().unwrap()])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-found:"))
        .stderr(predicate::str::contains("subdir.post.md"));

    assert!(!dir.path().join("subdir.post.md").exists());
}

#[test]
fn path_both_missing_not_found_names_suffixed_path() {
    // S-PATH-04: neither variant exists -> not-found; the error names the
    // suffixed landing path (not the bare input)
    let dir = TempDir::new().unwrap();

    cmd()
        .args(["post", "read", dir.path().join("no-such").to_str().unwrap()])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-found:"))
        .stderr(predicate::str::contains("no-such.post.md"));
}

// --- implicit-mention (singular, additive) ---

#[test]
fn implicit_mention_triggered_on_reply() {
    // S-SEND-03: replying auto-mentions the original sender (singular field)
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "bob", "--reply-to", "1", "reply body"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention: alice"));

    // JSON key with the same singular name (carol replies to #2 -> bob)
    cmd()
        .args(["--json", "post", "send", path.to_str().unwrap(), "carol", "--reply-to", "2", "again"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"implicit-mention\":\"bob\""));
}

#[test]
fn implicit_mention_not_triggered_boundaries() {
    // S-SEND-10b / S-SEND-11: self-reply, explicit mention, missing seq
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    // (a) self-reply: original sender == sender
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice", "--reply-to", "1", "self reply"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention").not());

    // (b) explicit mention already covers the original sender
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "bob", "--reply-to", "1", "--mention", "alice", "explicit"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention").not());

    // (c) reply-to seq does not exist
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "bob", "--reply-to", "99", "ghost reply"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention").not());
}

// --- read: showing / window always-on fields ---

#[test]
fn read_showing_window_small_thread() {
    // S-READ-01 / S-READ-02: 6 messages -> showing 6/6, window #1-#6
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    let mut content = String::new();
    for i in 1..=6 {
        content.push_str(&thread_message(i, "alice", "all", None, &[], &format!("msg {}", i)));
    }
    std::fs::write(&path, content).unwrap();

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("showing: 6/6"))
        .stdout(predicate::str::contains("window: #1-#6"));
}

#[test]
fn read_showing_window_over_limit() {
    // S-READ-06: 50 messages, default limit 20 -> showing 20/50, window #31-#50
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    let mut content = String::new();
    for i in 1..=50 {
        content.push_str(&thread_message(i, "alice", "all", None, &[], &format!("msg {}", i)));
    }
    std::fs::write(&path, content).unwrap();

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("showing: 20/50"))
        .stdout(predicate::str::contains("window: #31-#50"));
}

#[test]
fn read_empty_thread_showing_zero_no_window() {
    // S-READ-02 boundary: empty thread -> showing 0/0, NO window field
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, "").unwrap();

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("showing: 0/0"))
        .stdout(predicate::str::contains("window").not());
}

#[test]
fn read_filter_then_limit_total_semantics() {
    // S-READ-07 (F3): total is the post-filter count, before limit.
    // 50 messages, 25 mention alice -> --mention alice --limit 20 = 20/25
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    let mut content = String::new();
    for i in 1..=50u64 {
        if i % 2 == 1 {
            content.push_str(&thread_message(i, "bob", "all", None, &["alice"], &format!("msg {}", i)));
        } else {
            content.push_str(&thread_message(i, "bob", "all", None, &[], &format!("msg {}", i)));
        }
    }
    std::fs::write(&path, content).unwrap();

    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--mention", "alice", "--limit", "20"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("showing: 20/25"))
        .stdout(predicate::str::contains("showing: 20/50").not());
}

// --- validate --type ---

#[test]
fn validate_type_overrides_suffix() {
    // S-VAL-02: unknown suffix + --type post -> validated as a thread
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("myfile.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["validate", path.to_str().unwrap(), "--type", "post"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok validate"));
}

#[test]
fn validate_unknown_suffix_no_type_format_error() {
    // S-VAL-03: unknown suffix without --type -> error format:, fix mentions --type
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("myfile.md");
    std::fs::write(&path, "garbage").unwrap();

    cmd()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error format:"))
        .stderr(predicate::str::contains("--type"));
}

#[test]
fn validate_type_bogus_is_usage() {
    // S-VAL-05: --type with an invalid enum value -> usage exit 2
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("myfile.md");
    std::fs::write(&path, "x").unwrap();

    cmd()
        .args(["validate", path.to_str().unwrap(), "--type", "bogus"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));
}

#[test]
fn validate_type_mismatch_format_error() {
    // S-VAL-06: a profile file validated --type post -> format error
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("alice.md");
    cmd().args(["profile", "create", path.to_str().unwrap(), "alice"]).assert().success();
    let actual = dir.path().join("alice.profile.md");

    cmd()
        .args(["validate", actual.to_str().unwrap(), "--type", "post"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error format:"));
}

// --- `--` boundary ---

#[test]
fn dash_body_with_double_dash_send_and_edit() {
    // S-SEND-07 / S-EDIT-05: body starting with '-' placed after `--`
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice", "--", "-fix flag text"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok post.send"));

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("-fix flag text"));

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "alice", "2", "--", "-edited dash"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok post.edit"));

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("-edited dash"));
}

#[test]
fn dash_body_without_double_dash_is_usage() {
    // S-SEND-14 (NF-2): clap treats -fix as an unknown flag -> usage exit 2,
    // fix teaches `--`, example shows the `--` boundary form
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice", "-fix flag text"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("--"))
        .stderr(predicate::str::contains("paperwork post send standup.post.md alice -- \"-fix flag text\""));
}

// --- NF-3 supplementary cases ---

#[test]
fn post_create_missing_title_usage() {
    // S-CREATE-02
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.md");

    cmd()
        .args(["post", "create", path.to_str().unwrap()])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork post create standup \"Daily Standup\""));
}

#[test]
fn post_create_duplicate_already_exists() {
    // S-CREATE-03
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.md");

    cmd().args(["post", "create", path.to_str().unwrap(), "T"]).assert().success();

    cmd()
        .args(["post", "create", path.to_str().unwrap(), "T2"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error already-exists:"));
}

#[test]
fn profile_create_missing_name_usage() {
    // S-PROF-02
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("a.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap()])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork profile create agents/alice alice --model gpt-4o"));
}

#[test]
fn read_from_identity_value_is_usage() {
    // S-READ-04: --from only accepts a seq number (u64)
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--from", "alice"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("--from 5 --to 20"));
}

#[test]
fn brief_add_remove_basename_mapping() {
    // S-BRIEF-07: entry stored under its basename; remove by original path fails
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("b.md");
    let src_dir = dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "B"]).assert().success();

    cmd()
        .args(["brief", "add", brief_path.to_str().unwrap(), "src/main.rs"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok brief.add"));

    // remove by the ORIGINAL path misses (stored title is the basename)
    cmd()
        .args(["brief", "remove", brief_path.to_str().unwrap(), "src/main.rs"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-found:"));

    // remove by basename succeeds
    cmd()
        .args(["brief", "remove", brief_path.to_str().unwrap(), "main.rs"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok brief.remove"));
}

#[test]
fn contacts_create_title_positional_misuse() {
    // S-CONTACTS-05: title stays an optional flag for contacts create;
    // passing it positionally is an extra positional -> usage exit 2.
    // --help documents the flag (default "Contacts").
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("c.md");

    cmd()
        .args(["contacts", "create", path.to_str().unwrap(), "Team"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));

    cmd()
        .args(["contacts", "create", "--help"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("--title"))
        .stdout(predicate::str::contains("Contacts"));
}

// --- Hidden alias & naming policy ---

#[test]
fn po_hidden_alias_equivalent_to_post() {
    // S-ALIAS-01: po read == post read; po never appears in --help
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "alias content")).unwrap();

    cmd()
        .args(["po", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok post.read"))
        .stdout(predicate::str::contains("alias content"));

    // "po" as a standalone listed command must not appear ("post" does)
    cmd()
        .arg("--help")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("  po ").not());
}

#[test]
fn naming_policy_whitelist() {
    // SOTA C6: top-level groups are exactly {profile, post, brief, contacts,
    // validate}; hidden aliases never surface in help output.
    let assert_out = cmd().arg("--help").assert().success();
    let output = assert_out.get_output();
    let help = String::from_utf8_lossy(&output.stdout).to_string();

    for group in ["profile", "post", "brief", "contacts", "validate"] {
        assert!(help.contains(group), "help must list group `{}`", group);
    }
    // hidden aliases must not appear as listed commands
    for alias in ["  po ", "  p ", "  b ", "  c ", "  v ", "[p]", "[po]", "[b]", "[c]", "[v]"] {
        assert!(!help.contains(alias), "hidden alias `{}` leaked into help", alias.trim());
    }
}

#[test]
fn flag_inventory_matches_spec() {
    // SOTA C6: retained flags surface in the per-verb help text
    let send_help = {
        let out = cmd().args(["post", "send", "--help"]).assert().success();
        String::from_utf8_lossy(&out.get_output().stdout).to_string()
    };
    for flag in ["--stdin", "--reply-to", "--mention"] {
        assert!(send_help.contains(flag), "post send must keep {}", flag);
    }
    assert!(!send_help.contains("--from"), "post send must not keep --from");

    let read_help = {
        let out = cmd().args(["post", "read", "--help"]).assert().success();
        String::from_utf8_lossy(&out.get_output().stdout).to_string()
    };
    for flag in ["--from", "--to", "--mention", "--reply-to", "--limit"] {
        assert!(read_help.contains(flag), "post read must keep {}", flag);
    }

    let brief_help = {
        let out = cmd().args(["brief", "add", "--help"]).assert().success();
        String::from_utf8_lossy(&out.get_output().stdout).to_string()
    };
    for flag in ["--regex", "--note"] {
        assert!(brief_help.contains(flag), "brief add must keep {}", flag);
    }

    // Review round C4: remaining verb flag inventories
    let profile_create_help = {
        let out = cmd().args(["profile", "create", "--help"]).assert().success();
        String::from_utf8_lossy(&out.get_output().stdout).to_string()
    };
    assert!(profile_create_help.contains("--model"), "profile create must keep --model");
    assert!(!profile_create_help.contains("--name"), "profile create must not keep --name");

    let brief_create_help = {
        let out = cmd().args(["brief", "create", "--help"]).assert().success();
        String::from_utf8_lossy(&out.get_output().stdout).to_string()
    };
    assert!(brief_create_help.contains("--owner"), "brief create must keep --owner");

    let contacts_create_help = {
        let out = cmd().args(["contacts", "create", "--help"]).assert().success();
        String::from_utf8_lossy(&out.get_output().stdout).to_string()
    };
    assert!(contacts_create_help.contains("--title"), "contacts create must keep --title");

    let validate_help = {
        let out = cmd().args(["validate", "--help"]).assert().success();
        String::from_utf8_lossy(&out.get_output().stdout).to_string()
    };
    assert!(validate_help.contains("--type"), "validate must keep --type");

    let post_create_help = {
        let out = cmd().args(["post", "create", "--help"]).assert().success();
        String::from_utf8_lossy(&out.get_output().stdout).to_string()
    };
    assert!(post_create_help.contains("--participants"), "post create must keep --participants");
    assert!(!post_create_help.contains("--title"), "post create must not keep --title");
}

// =====================================================================
// Review-round additions (A2 / B4 / C3 / C6)
// =====================================================================

#[test]
fn ascii_output_contract_guard() {
    // A2 (pins R-09): usage error and runtime error stderr must be pure
    // ASCII at the raw-byte level
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    let usage_out = cmd()
        .args(["post", "send", path.to_str().unwrap()])
        .assert()
        .code(2)
        .get_output()
        .clone();
    assert!(
        usage_out.stderr.iter().all(u8::is_ascii),
        "usage error stderr contains non-ASCII bytes"
    );

    let runtime_out = cmd()
        .args(["post", "read", "nonexistent/path/file.md"])
        .assert()
        .code(1)
        .get_output()
        .clone();
    assert!(
        runtime_out.stderr.iter().all(u8::is_ascii),
        "runtime error stderr contains non-ASCII bytes"
    );
}

#[test]
fn send_body_and_stdin_mutually_exclusive() {
    // S-SEND-04 (B4): positional body + --stdin -> validation error, exit 1
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice", "body", "--stdin"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error validation:"))
        .stderr(predicate::str::contains("--stdin"));
}

#[test]
fn send_missing_body_no_stdin_is_validation() {
    // S-SEND-05 (standalone): PATH + NAME but no body/--stdin -> validation
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "alice"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error validation:"))
        .stderr(predicate::str::contains("paperwork post send"));
}

#[test]
fn edit_triple_guardrail_cli() {
    // S-EDIT-02: wrong sender / not most recent / not final -> not-allowed,
    // examples carry the v0.5 positional grammar
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    let mut content = thread_message(1, "alice", "all", None, &[], "first");
    content.push_str(&thread_message(2, "alice", "all", None, &[], "second"));
    content.push_str(&thread_message(3, "bob", "all", None, &[], "third"));
    std::fs::write(&path, content).unwrap();

    // (a) wrong sender: #3 was sent by bob
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "alice", "3", "x"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-allowed:"))
        .stderr(predicate::str::contains("sent by 'bob', not 'alice'"))
        .stderr(predicate::str::contains("paperwork post edit"));

    // (b) not sender's most recent: alice's last is #2
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "alice", "1", "x"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-allowed:"))
        .stderr(predicate::str::contains("not your most recent"));

    // (c) not final: #2 is alice's latest but bob's #3 ends the thread
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "alice", "2", "x"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-allowed:"))
        .stderr(predicate::str::contains("not the final message"))
        .stderr(predicate::str::contains("\"corrected body\""));
}

#[test]
fn quiet_read_keeps_showing_and_window() {
    // -q suppresses only the status line; showing/window fields survive
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    let mut content = thread_message(1, "alice", "all", None, &[], "a");
    content.push_str(&thread_message(2, "bob", "all", None, &[], "b"));
    std::fs::write(&path, content).unwrap();

    cmd()
        .args(["-q", "post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("showing: 2/2"))
        .stdout(predicate::str::contains("window: #1-#2"))
        .stdout(predicate::str::contains("ok post.read").not());
}

#[test]
fn plain_read_outputs_file_format() {
    // --plain emits the serialized thread (file format), no envelope
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "plain body")).unwrap();

    cmd()
        .args(["--plain", "post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("### #1 alice"))
        .stdout(predicate::str::contains("ok post.read").not());
}

#[test]
fn single_letter_aliases_work() {
    // p/b/c/v aliases resolve to their canonical groups
    let dir = TempDir::new().unwrap();

    let profile_path = dir.path().join("a.md");
    cmd()
        .args(["p", "create", profile_path.to_str().unwrap(), "alice"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok profile.create"));

    let brief_path = dir.path().join("b.md");
    cmd()
        .args(["b", "create", brief_path.to_str().unwrap(), "B"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok brief.create"));

    let contacts_path = dir.path().join("c.md");
    cmd()
        .args(["c", "create", contacts_path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok contacts.create"));

    let thread_path = dir.path().join("t.post.md");
    std::fs::write(&thread_path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();
    cmd()
        .args(["v", thread_path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok validate"));
}

#[test]
fn post_group_help_lists_verbs() {
    // `paperwork post --help`: group-level help lists all verbs, exit 0
    let out = cmd().args(["post", "--help"]).assert().code(0);
    let help = String::from_utf8_lossy(&out.get_output().stdout).to_string();
    assert!(help.contains("post"), "group help must mention the group");
    assert!(help.contains("<COMMAND>"), "group help must show the subcommand slot");
    for verb in ["create", "send", "read", "summary", "edit"] {
        assert!(help.contains(verb), "group help must list verb `{}`", verb);
    }
}

#[test]
fn read_mention_filter_zero_hits_on_nonempty_thread() {
    // Filter miss on a non-empty thread: showing 0/0, no window, exit 0
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    let mut content = thread_message(1, "alice", "all", None, &[], "a");
    content.push_str(&thread_message(2, "bob", "all", None, &["carol"], "b"));
    std::fs::write(&path, content).unwrap();

    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--mention", "nobody"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("showing: 0/0"))
        .stdout(predicate::str::contains("window").not());
}

#[test]
fn implicit_mention_persisted_to_file() {
    // The auto-added mention must land in the thread file (- Mentions: line)
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "bob", "--reply-to", "1", "reply"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention: alice"));

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("- Mentions: alice"),
        "implicit mention not persisted to file"
    );
}
