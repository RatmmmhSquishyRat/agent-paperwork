//! Integration tests for the stateless paperwork CLI.
//!
//! Merged baseline: v0.5.0 positional grammar (PATH first; NAME second for
//! post send/edit; content last) on top of Managed File Format v2 (owner
//! rulings D1/D2/D3): H1-title-only preamble, `## #N sender (timestamp)`
//! headers with ```md fences, reference state as `@name` / `@#N` body
//! tokens, no `post create` (thread creation folded into the first send).
//! v0.6 grammar (this file): PATH is the only positional argument; every
//! required payload is a named flag (--author/-a, --message/-m, --seq,
//! --name, --title, --entry, --entry-title, --profile).
//! Usage errors (clap parse failures) render as the `usage` envelope, exit 2;
//! runtime errors keep the six categories, exit 1.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("paperwork").unwrap()
}

/// Minimal valid single-message thread corpus (Format v2).
///
/// Header `## #N sender (timestamp)` + ```md fence; reference state lives in
/// the body as `@#N` / `@name` tokens (D2). The `to` parameter is vestigial:
/// the To attribute line was deleted by D1/D2 and is ignored here.
fn thread_message(seq: u64, sender: &str, _to: &str, reply_to: Option<u64>, mentions: &[&str], body: &str) -> String {
    let mut tokens: Vec<String> = Vec::new();
    if let Some(r) = reply_to {
        tokens.push(format!("@#{}", r));
    }
    for m in mentions {
        tokens.push(format!("@{}", m));
    }
    let body = if tokens.is_empty() {
        body.to_string()
    } else {
        format!("{}\n\n{}", tokens.join(" "), body)
    };
    format!("## #{} {} (2026-01-15T10:30:00Z)\n\n```md\n{}\n```\n\n", seq, sender, body)
}

// --- Profile ---

#[test]
fn profile_create_and_show() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("alice.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "--name", "alice", "--model", "gpt-4"])
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
        .args(["--json", "profile", "create", path.to_str().unwrap(), "--name", "bob"])
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
        .args(["profile", "create", path.to_str().unwrap(), "--name", "x"])
        .assert()
        .success();

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "--name", "y"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error already-exists:"));
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

    cmd().args(["profile", "create", p1.to_str().unwrap(), "--name", "a"]).assert().success();
    cmd().args(["profile", "create", p2.to_str().unwrap(), "--name", "b"]).assert().success();

    cmd()
        .args(["--json", "profile", "list", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("a.profile.md"))
        .stdout(predicate::str::contains("b.profile.md"));
}

// --- Post ---

#[test]
fn post_send_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("thread.md");

    // First send creates the thread and writes the preamble (no separate
    // create subcommand; Format v2 folds creation into the first send).
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--title", "Design Discussion", "--message", "I think we should use Rust."])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.send"))
        .stdout(predicate::str::contains("seq: 1"));

    let actual_path = dir.path().join("thread.post.md");
    let content = std::fs::read_to_string(&actual_path).unwrap();
    // First-write preamble is the H1 title only (D1): no participants line
    assert!(content.starts_with("# Design Discussion\n\n## #1 alice ("));
    assert!(!content.contains("- participants:"));
    // Body fence info is `md` on the write side (D3)
    assert!(content.contains("```md"));

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--message", "Agreed, Rust it is."])
        .assert()
        .success()
        .stdout(predicate::str::contains("seq: 2"));

    // Read starts at #1 (no placeholder first message)
    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.read"))
        .stdout(predicate::str::contains("#1 alice"))
        .stdout(predicate::str::contains("Rust"))
        .stdout(predicate::str::contains("#2 bob"));
}

#[test]
fn post_send_stdin() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("stdin-thread.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--stdin"])
        .write_stdin("Message from stdin")
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.send"));

    // Default title: strip .md from the original path argument (spec §5.7)
    let content = std::fs::read_to_string(dir.path().join("stdin-thread.post.md")).unwrap();
    assert!(content.contains("# stdin-thread"));

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
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--message", "   "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"));
}

#[test]
fn post_edit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("edit-thread.md");

    cmd().args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--message", "original"]).assert().success();

    // First real message is seq 1 (placeholder creation message abolished)
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--author", "bob", "--seq", "1", "--message", "edited"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.edit"));

    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--from", "1", "--to", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("edited"));
}

#[test]
fn post_summary() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("sum.md");

    cmd().args(["post", "send", path.to_str().unwrap(), "--author", "x", "--message", "hello"]).assert().success();

    // Title from the H1 preamble (default derived from the path); messages
    // counts real messages only (no placeholder creation message).
    cmd()
        .args(["--json", "post", "summary", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\":\"sum\""))
        .stdout(predicate::str::contains("\"participants\":\"x\""))
        .stdout(predicate::str::contains("\"messages\":1"));
}

// --- Brief ---

#[test]
fn brief_create_add_read() {
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("brief.md");
    let entry_file = dir.path().join("notes.txt");

    std::fs::write(&entry_file, "some content").unwrap();

    cmd()
        .args(["brief", "create", brief_path.to_str().unwrap(), "--title", "My Brief"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok brief.create"));

    cmd()
        .args(["brief", "add", brief_path.to_str().unwrap(), "--entry", "notes.txt"])
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

    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "--title", "B"]).assert().success();
    cmd().args(["brief", "add", brief_path.to_str().unwrap(), "--entry", "e.txt"]).assert().success();

    cmd()
        .args(["brief", "remove", brief_path.to_str().unwrap(), "--entry-title", "e.txt"])
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

    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "--title", "V"]).assert().success();
    cmd().args(["brief", "add", brief_path.to_str().unwrap(), "--entry", "src.txt"]).assert().success();

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

    cmd().args(["profile", "create", profile_path.to_str().unwrap(), "--name", "agent"]).assert().success();

    cmd()
        .args(["contacts", "create", contacts_path.to_str().unwrap(), "--title", "Team"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok contacts.create"));

    cmd()
        .args(["contacts", "add", contacts_path.to_str().unwrap(), "--profile", profile_path.to_str().unwrap()])
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

    cmd().args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--message", "hello"]).assert().success();

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
        .args(["--quiet", "profile", "create", path.to_str().unwrap(), "--name", "q"])
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
    // S-SEND-08: only PATH given, --author and --message both missing
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap()])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("example:"))
        .stderr(predicate::str::contains("paperwork post send standup.post.md --author alice --message"));
}

#[test]
fn name_body_confusion_single_string() {
    // S-SEND-15 (tdd 1b-C flip): a single extra string after PATH is a clap
    // unexpected-argument usage error (exit 2); the v0.5 silent-write path
    // (validation exit 1 via the NAME slot) is structurally gone.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "some body text"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));
}

#[test]
fn usage_old_grammar_send_from() {
    // S-SEND-13: old-grammar --from flag -> usage envelope, canonical example
    // must not carry user argv values (F2 ruling); v0.6 named form.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--from", "alice", "--message", "body"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork post send standup.post.md --author alice --message"));
}

#[test]
fn usage_v05_grammar_profile_create_positional() {
    // S-PROF-03 (tdd 1b-A flip): --name is re-legalized in v0.6, so the old
    // trigger became valid; the migration trigger is now the v0.5 positional
    // NAME (extra positional -> usage exit 2).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("a.md");

    cmd()
        .args(["profile", "create", path.to_str().unwrap(), "alice"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork profile create agents/alice --name alice"));
}

#[test]
fn usage_v05_grammar_brief_add_positional() {
    // S-BRIEF-04 (tdd 1b-A flip): --entry re-legalized; trigger is now the
    // v0.5 positional entry path (extra positional -> usage exit 2).
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("b.md");
    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "--title", "B"]).assert().success();

    cmd()
        .args(["brief", "add", brief_path.to_str().unwrap(), "e.txt"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork brief add onboarding.brief.md --entry src/main.rs"));
}

#[test]
fn usage_v05_grammar_contacts_add_positional() {
    // S-CONTACTS-04 (tdd 1b-A flip): --profile re-legalized; trigger is now
    // the v0.5 positional profile path (extra positional -> usage exit 2).
    let dir = TempDir::new().unwrap();
    let contacts_path = dir.path().join("c.md");
    cmd().args(["contacts", "create", contacts_path.to_str().unwrap()]).assert().success();

    cmd()
        .args(["contacts", "add", contacts_path.to_str().unwrap(), "x.profile.md"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork contacts add team.contacts.md"));
}

#[test]
fn usage_identity_flag_post_edit_from() {
    // S-SEND-13 migration chain (tdd 1b-A rewrite): --seq is re-legalized in
    // v0.6, so the case keeps its usage exit 2 only through the still-illegal
    // identity flag --from.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--seq", "1", "--from", "alice", "--message", "new"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork post edit standup.post.md --author alice --seq 3"));
}

#[test]
fn usage_seq_not_numeric() {
    // S-EDIT-06: --seq value must parse as u64
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--author", "alice", "--seq", "abc", "--message", "new"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));
}

#[test]
fn usage_extra_positional_send() {
    // S-SEND-13: four positional values for send (v0.6 keeps PATH only)
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
        err.contains("the following required arguments were not provided: --author <AUTHOR>")
            && err.contains("--message <MESSAGE>"),
        "message must list the missing arguments: {}",
        err
    );

    // --json carries the same complete message
    cmd()
        .args(["--json", "post", "send", path.to_str().unwrap()])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(
            "the following required arguments were not provided: --author <AUTHOR>",
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
        .args(["post", "send", bare.to_str().unwrap(), "--author", "alice", "--message", "landing content"])
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
        .args(["post", "send", notes.to_str().unwrap(), "--author", "alice", "--message", "attempt"])
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
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--reply-to", "1", "--message", "reply body"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention: alice"));

    // JSON key with the same singular name (carol replies to #2 -> bob)
    cmd()
        .args(["--json", "post", "send", path.to_str().unwrap(), "--author", "carol", "--reply-to", "2", "--message", "again"])
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
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--reply-to", "1", "--message", "self reply"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention").not());

    // (b) explicit mention already covers the original sender
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--reply-to", "1", "--mention", "alice", "--message", "explicit"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention").not());

    // (c) reply-to seq does not exist
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--reply-to", "99", "--message", "ghost reply"])
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
    cmd().args(["profile", "create", path.to_str().unwrap(), "--name", "alice"]).assert().success();
    let actual = dir.path().join("alice.profile.md");

    cmd()
        .args(["validate", actual.to_str().unwrap(), "--type", "post"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error format:"));
}

// --- dash-headed bodies (v0.6: direct via --message, allow_hyphen_values) ---

#[test]
fn dash_body_direct_via_message_send_and_edit() {
    // S-SEND-10 / S-EDIT-05 (tdd 1b-B flip): a body starting with '-' is
    // passed DIRECTLY via --message. PINNED DEPENDENCY: this test relies on
    // `allow_hyphen_values = true` on --message for BOTH post send and
    // post edit; do not remove that attribute without flipping this case.
    // The v0.5 `--` boundary form is retired.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--message", "-fix flag text"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok post.send"));

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("-fix flag text"));

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--author", "alice", "--seq", "2", "--message", "-edited dash"])
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
fn bare_dash_token_teaches_message_flag() {
    // S-SEND-11 (tdd 1b-B flip): a bare dash-headed token (not handed to
    // --message) is a clap unknown-argument usage error (exit 2); the fix
    // guides the value into --message and the canonical example shows the
    // `--message "-fix flag text"` form.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "-fix", "flag", "text"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("pass it via --message"))
        .stderr(predicate::str::contains("--message \"-fix flag text\""));
}

// --- NF-3 supplementary cases ---

#[test]
fn post_create_removed_is_usage() {
    // Format v2: `post create` no longer exists; invoking it is a clap usage
    // error (exit 2) and nothing is written.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.md");

    cmd()
        .args(["post", "create", path.to_str().unwrap(), "T"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));

    assert!(!dir.path().join("t.post.md").exists());
}

#[test]
fn post_send_title_ignored_on_existing_thread() {
    // OQ-1: --title is only honoured on first write; on a non-empty thread
    // it is silently ignored (exit 0, preamble title unchanged).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--title", "First Title", "--message", "first"])
        .assert()
        .success();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--title", "Second Title", "--message", "second"])
        .assert()
        .success();

    cmd()
        .args(["--json", "post", "summary", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\":\"First Title\""));
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
        .stderr(predicate::str::contains("paperwork profile create agents/alice --name alice --model gpt-4o"));
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

    cmd().args(["brief", "create", brief_path.to_str().unwrap(), "--title", "B"]).assert().success();

    cmd()
        .args(["brief", "add", brief_path.to_str().unwrap(), "--entry", "src/main.rs"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok brief.add"));

    // remove by the ORIGINAL path misses (stored title is the basename)
    cmd()
        .args(["brief", "remove", brief_path.to_str().unwrap(), "--entry-title", "src/main.rs"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-found:"));

    // remove by basename succeeds
    cmd()
        .args(["brief", "remove", brief_path.to_str().unwrap(), "--entry-title", "main.rs"])
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
    for flag in ["--author", "--message", "--stdin", "--reply-to", "--mention"] {
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
    // v0.6: --name is re-legalized as the required named flag (was the v0.5
    // positional NAME slot).
    assert!(profile_create_help.contains("--name"), "profile create must gain --name");

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

    // Format v2 flag surface (D1/D2): send keeps the sugar flags but never
    // gains --to / --participants; --title is the preamble carrier.
    assert!(send_help.contains("--title"), "post send must keep --title");
    assert!(!send_help.contains("--to\n"), "post send must not gain --to");
    assert!(!send_help.contains("--participants"), "post send must not gain --participants");
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
fn send_message_and_stdin_conflict_is_usage() {
    // S-SEND-07 (tdd 1b-C flip): --message + --stdin -> clap conflicts_with
    // -> usage exit 2 (was v0.5 validation exit 1); no file write.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--message", "body", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("--stdin"));
}

#[test]
fn send_missing_message_no_stdin_is_usage() {
    // S-SEND-06 (tdd 1b-C flip): PATH + --author but neither --message nor
    // --stdin -> clap required_unless_present -> usage exit 2 (was v0.5
    // validation exit 1).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("--message <MESSAGE>"));
}

#[test]
fn edit_triple_guardrail_cli() {
    // S-EDIT-02: wrong sender / not most recent / not final -> not-allowed,
    // examples carry the v0.6 named grammar
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    let mut content = thread_message(1, "alice", "all", None, &[], "first");
    content.push_str(&thread_message(2, "alice", "all", None, &[], "second"));
    content.push_str(&thread_message(3, "bob", "all", None, &[], "third"));
    std::fs::write(&path, content).unwrap();

    // (a) wrong sender: #3 was sent by bob
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--author", "alice", "--seq", "3", "--message", "x"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-allowed:"))
        .stderr(predicate::str::contains("sent by 'bob', not 'alice'"))
        .stderr(predicate::str::contains("paperwork post edit"));

    // (b) not sender's most recent: alice's last is #2
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--author", "alice", "--seq", "1", "--message", "x"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error not-allowed:"))
        .stderr(predicate::str::contains("not your most recent"));

    // (c) not final: #2 is alice's latest but bob's #3 ends the thread
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--author", "alice", "--seq", "2", "--message", "x"])
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
        .stdout(predicate::str::contains("## #1 alice"))
        .stdout(predicate::str::contains("ok post.read").not());
}

#[test]
fn single_letter_aliases_work() {
    // p/b/c/v aliases resolve to their canonical groups
    let dir = TempDir::new().unwrap();

    let profile_path = dir.path().join("a.md");
    cmd()
        .args(["p", "create", profile_path.to_str().unwrap(), "--name", "alice"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok profile.create"));

    let brief_path = dir.path().join("b.md");
    cmd()
        .args(["b", "create", brief_path.to_str().unwrap(), "--title", "B"])
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
    for verb in ["send", "read", "summary", "edit"] {
        assert!(help.contains(verb), "group help must list verb `{}`", verb);
    }
    // Format v2 removed `post create`
    assert!(!help.contains("  create"), "group help must not list removed verb `create`");
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
    // The auto-added mention lands in the body as an `@name` token (D2).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--reply-to", "1", "--message", "reply"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("implicit-mention: alice"));

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("@#1 @alice\n\nreply"),
        "implicit mention token not persisted to body: {}",
        content
    );
}

// =====================================================================
// Format v2 (merged baseline) additions: D1/D2/D3 + OQ-1/OQ-4 behaviour
// =====================================================================

#[test]
fn post_send_to_and_participants_flags_removed() {
    // Owner rulings D1/D2: `--to` and `--participants` flags are deleted;
    // passing them is a clap usage error and no file is ever created.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("directed.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--to", "charlie", "--message", "Hi"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--participants", "bob,charlie", "--message", "Hi"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));

    assert!(!dir.path().join("directed.post.md").exists());
}

#[test]
fn post_send_mention_injects_body_tokens() {
    // OQ-4: --mention a,b injects `@a @b` tokens at the body head.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("mention.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--mention", "charlie,dave", "--message", "hello team"])
        .assert()
        .success()
        .stdout(predicate::str::contains("seq: 1"));

    let content = std::fs::read_to_string(dir.path().join("mention.post.md")).unwrap();
    assert!(content.contains("@charlie @dave\n\nhello team"));

    // Derived mentions show up in the default read output
    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("mentions:charlie,dave"));

    // Read filter matches on the derived value
    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--mention", "dave"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 messages"));
}

#[test]
fn post_send_mention_rejects_malformed_values() {
    // MJ-2: --mention values are validated at the flag layer; shapes that
    // the derivation rules would silently mangle or drop are rejected with
    // a Validation envelope and no file is written.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("strict.md");

    // 1. reply-shaped value (#<digits>) belongs to --reply-to, not --mention
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--mention", "#5", "--message", "hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"))
        .stderr(predicate::str::contains("invalid --mention value '#5'"));

    // 2. whitespace inside the value would be truncated by the token scan
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--mention", "two words", "--message", "hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"));

    // 3. mentioning the sender itself is silently dropped by derivation
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--mention", "alice", "--message", "hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"));

    // No file was created by any rejected invocation
    assert!(!dir.path().join("strict.post.md").exists());
}

#[test]
fn post_send_reply_to_injects_body_tokens() {
    // OQ-4: --reply-to N injects `@#N` plus the implicit @original-sender.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("reply.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--message", "first"])
        .assert()
        .success();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--reply-to", "1", "--message", "agreed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("seq: 2"));

    let content = std::fs::read_to_string(dir.path().join("reply.post.md")).unwrap();
    assert!(content.contains("@#1 @alice\n\nagreed"));

    // JSON read exposes the parse-time derived fields (no `to` field)
    cmd()
        .args(["--json", "post", "read", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"reply_to\":1"))
        .stdout(predicate::str::contains("\"mentions\":[\"alice\"]"))
        .stdout(predicate::str::contains("\"to\"").not());

    // Read filter matches on the derived reply reference
    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--reply-to", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 messages"))
        .stdout(predicate::str::contains("#2 bob"));
}

#[test]
fn post_send_reply_token_dedup() {
    // Implicit @ original sender never duplicates: self-reply skips it and
    // an explicit --mention of the same name is injected exactly once.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("dedup.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--message", "first"])
        .assert()
        .success();

    // Self-reply: only the `@#1` token, no `@alice`
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--reply-to", "1", "--message", "follow-up"])
        .assert()
        .success();

    // Reply + explicit mention of the same sender: single `@alice` token
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--reply-to", "1", "--mention", "alice,alice", "--message", "also this"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join("dedup.post.md")).unwrap();
    assert!(content.contains("@#1\n\nfollow-up"));
    assert!(content.contains("@#1 @alice\n\nalso this"));
    assert_eq!(content.matches("@alice").count(), 1);
}

#[test]
fn post_send_oversized_body_after_injection() {
    // The 64KB cap applies to the final body AFTER token injection.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("big.md");

    // Delivered via --stdin to stay under the OS command-line length limit.
    let huge = "a".repeat(65 * 1024);
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--mention", "bob", "--stdin"])
        .write_stdin(huge)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"));
}

#[test]
fn post_read_plain_no_preamble() {
    // Subset output is messages-only serialization (no preamble, POST-31)
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("plain.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--title", "Plain Check", "--message", "one"])
        .assert()
        .success();
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--message", "two"])
        .assert()
        .success();

    cmd()
        .args(["--plain", "post", "read", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("## #1 alice"))
        .stdout(predicate::str::contains("## #2 bob"))
        .stdout(predicate::str::contains("Plain Check").not())
        .stdout(predicate::str::contains("- participants:").not());
}

#[test]
fn post_send_appends_to_file_missing_trailing_newline() {
    // A thread whose final byte is the closing fence (external edits can
    // strip the trailing newline). The next send must repair the boundary
    // instead of gluing the new header onto the fence line.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("noeol.post.md");
    std::fs::write(
        &path,
        "# NoEol\n\n\
         ## #1 alice (2026-08-09T03:50:00Z)\n\n\
         ```md\nfirst\n```",
    )
    .unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "bob", "--message", "second"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.send"))
        .stdout(predicate::str::contains("seq: 2"));

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("\n## #2 bob ("));
    assert!(!content.contains("```##"));

    // Both messages read back intact (previously #2 vanished into #1).
    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 messages"))
        .stdout(predicate::str::contains("#1 alice"))
        .stdout(predicate::str::contains("#2 bob"));
}

#[test]
fn post_edit_missing_message_is_usage() {
    // Review F3 flipped (tdd 1b): missing body channel is now caught by clap
    // (required_unless_present) as usage exit 2; the envelope example is the
    // edit-shaped v0.6 canonical form (never a send-shaped example).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("thread.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "alice", "--message", "first"])
        .assert()
        .success();

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--author", "alice", "--seq", "1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("paperwork post edit"))
        .stderr(predicate::str::contains("paperwork post send").not());
}

// --- Format v2 validate pipeline (seq / fence / heuristic / empty) ---

#[test]
fn validate_seq_gap() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("gap.post.md");
    std::fs::write(
        &path,
        "# Gap Thread\n\n\
         ## #1 alice (2026-01-15T10:30:00Z)\n\n\
         ```md\none\n```\n\n\
         ## #3 bob (2026-01-15T10:31:00Z)\n\n\
         ```md\nthree\n```\n",
    )
    .unwrap();

    // seq failure surfaces as Validation directly (category validation, R10)
    cmd()
        .args(["--json", "validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"status\":\"error\""))
        .stdout(predicate::str::contains("\"category\":\"validation\""))
        .stdout(predicate::str::contains("sequence"));
}

#[test]
fn validate_unclosed_fence() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("broken.post.md");
    std::fs::write(
        &path,
        "# Broken Fence\n\n\
         ## #1 alice (2026-01-15T10:30:00Z)\n\n\
         ```md\nbody\n```\n\n\
         ```text\nunclosed tail\n",
    )
    .unwrap();

    cmd()
        .args(["--json", "validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"status\":\"error\""))
        .stdout(predicate::str::contains("\"category\":\"format\""))
        .stdout(predicate::str::contains("unclosed"));
}

#[test]
fn validate_garbage() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("garbage.post.md");
    std::fs::write(&path, "just some garbage text\nno headers here\n").unwrap();

    cmd()
        .args(["--json", "validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"status\":\"error\""))
        .stdout(predicate::str::contains("\"category\":\"format\""))
        .stdout(predicate::str::contains("dynamic md fences"));
}

#[test]
fn validate_empty_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("empty.post.md");
    std::fs::write(&path, "").unwrap();

    // Empty-file exemption removed (spec §8, VAL-07): zero messages -> Parse
    cmd()
        .args(["--json", "validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"status\":\"error\""))
        .stdout(predicate::str::contains("\"category\":\"format\""));
}

#[test]
fn validate_suspected_header_warning() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("suspect.post.md");
    std::fs::write(
        &path,
        "# Suspect\n\n\
         ## #1 alice (2026-01-15T10:30:00Z)\n\n\
         ```md\nok\n```\n\n\
         ## #2 bob (2026-01-15T10:31:00Z\n",
    )
    .unwrap();

    // Conclusion stays ok; the malformed header line gets a warning + fix
    cmd()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok validate"))
        .stdout(predicate::str::contains("warning:"))
        .stdout(predicate::str::contains("expected format: ## #<seq> <sender> (<timestamp>)"));
}

#[test]
fn validate_suspected_header_multi_space_warning() {
    // N2 regression: `##  #1 alice` (double space + missing timestamp) fails
    // the strict grammar but MUST trip the whitespace-lenient heuristic.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("multispace.post.md");
    std::fs::write(
        &path,
        "# Suspect\n\n\
         ## #1 alice (2026-01-15T10:30:00Z)\n\n\
         ```md\nok\n```\n\n\
         ##  #1 alice\n",
    )
    .unwrap();

    cmd()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok validate"))
        .stdout(predicate::str::contains("suspected message header: ##  #1 alice"))
        .stdout(predicate::str::contains("expected format: ## #<seq> <sender> (<timestamp>)"));
}

// =====================================================================
// v0.6 new cases (tdd §4)
// =====================================================================

#[test]
fn short_forms_equivalent_to_long_flags() {
    // S-SEND-02 / S-SHORT-01: -a / -m are verbatim equivalents of
    // --author / --message (F3: the short set is exactly {-a, -m, -q}).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "-a", "alice", "-m", "short form body"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok post.send"))
        .stdout(predicate::str::contains("seq: 1"));

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "-a", "alice", "--seq", "1", "-m", "short form edit"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok post.edit"));

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("short form edit"));
}

#[test]
fn send_missing_author_is_usage() {
    // S-SEND-05: --message given but --author missing -> usage exit 2
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--message", "body"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"))
        .stderr(predicate::str::contains("--author <AUTHOR>"));
}

#[test]
fn edit_stdin_only_succeeds() {
    // S-EDIT-04 boundary: --stdin satisfies the body channel without
    // --message (required_unless_present).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--author", "alice", "--seq", "1", "--stdin"])
        .write_stdin("stdin edit body")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ok post.edit"));

    cmd()
        .args(["post", "read", path.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("stdin edit body"));
}

#[test]
fn read_mention_has_no_short_form() {
    // S-READ-04 (F3): the short set is exactly {-a, -m, -q}; read filters
    // never gain short forms.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &["bob"], "hi")).unwrap();

    cmd()
        .args(["post", "read", path.to_str().unwrap(), "-m", "bob"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error usage:"));

    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--mention", "bob"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("showing: 1/1"));
}

#[test]
fn send_empty_author_is_validation() {
    // S-SEND-18: whitespace-only --author value -> validation exit 1
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.post.md");
    std::fs::write(&path, thread_message(1, "alice", "all", None, &[], "hi")).unwrap();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--author", "   ", "--message", "body"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("error validation:"))
        .stderr(predicate::str::contains("--author"));
}
