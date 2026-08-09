//! Integration tests for the stateless paperwork CLI - Managed File Format v2.
//!
//! Covers tdd.md §4 (T-CLI-01..24) plus the global-flag envelope tests.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("paperwork").unwrap()
}

// --- Profile (T-CLI-01..05) ---

#[test]
fn profile_create_and_show() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("alice.md");

    cmd()
        .args([
            "profile", "create", path.to_str().unwrap(),
            "--name", "alice",
            "--model", "gpt-4o",
            "--description", "Parser module implementer",
            "--scope-write", "src/parser/**",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok profile.create"))
        .stdout(predicate::str::contains("name: alice"));

    cmd()
        .args(["profile", "show", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok profile.show"))
        .stdout(predicate::str::contains("name: alice"))
        .stdout(predicate::str::contains("model: gpt-4o"))
        .stdout(predicate::str::contains("description: Parser module implementer"))
        .stdout(predicate::str::contains("scope.write: src/parser/**"));

    // On-disk format: H1 identity + lowercase attribute lines + Scope lines
    let content = std::fs::read_to_string(dir.path().join("alice.profile.md")).unwrap();
    assert!(content.contains("# alice"));
    assert!(content.contains("- model: gpt-4o"));
    assert!(content.contains("- write: src/parser/**"));
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
        .args([
            "profile", "edit", path.to_str().unwrap(),
            "--model", "claude-3",
            "--scope-write", "src/**",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok profile.edit"))
        .stdout(predicate::str::contains("changed: model, scope.write"));

    cmd()
        .args(["--json", "profile", "show", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude-3"))
        .stdout(predicate::str::contains("src/**"));

    // Scope is serialized as an attribute line list (R3)
    let content = std::fs::read_to_string(dir.path().join("edit.profile.md")).unwrap();
    assert!(content.contains("- write: src/**"));
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

// --- Post (T-CLI-06..12) ---

#[test]
fn post_send_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("thread.md");

    // First send creates the thread and writes the preamble (no separate create subcommand)
    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice",
            "--title", "Design Discussion",
            "I think we should use Rust.",
        ])
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
        .args(["post", "send", path.to_str().unwrap(), "--from", "bob", "Agreed, Rust it is."])
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
        .args(["post", "send", path.to_str().unwrap(), "--from", "alice", "--stdin"])
        .write_stdin("Message from stdin")
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.send"));

    // Default title: strip .md from the original path argument
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
        .args(["post", "send", path.to_str().unwrap(), "--from", "alice", "   "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"));
}

#[test]
fn post_send_to_and_participants_flags_removed() {
    // Owner rulings D1/D2: `--to` and `--participants` flags are deleted;
    // passing them is a clap usage error and no attribute line ever lands.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("directed.md");

    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "bob", "--to", "charlie", "Hi",
        ])
        .assert()
        .failure();

    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "bob", "--participants", "bob,charlie", "Hi",
        ])
        .assert()
        .failure();

    // No file was created by the rejected invocations
    assert!(!dir.path().join("directed.post.md").exists());
}

#[test]
fn post_send_mention_injects_body_tokens() {
    // OQ-4: --mention a,b injects `@a @b` tokens at the body head.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("mention.md");

    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice", "--mention", "charlie,dave",
            "hello team",
        ])
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
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice", "--mention", "#5",
            "hello",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"))
        .stderr(predicate::str::contains("invalid --mention value '#5'"));

    // 2. whitespace inside the value would be truncated by the token scan
    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice", "--mention", "two words",
            "hello",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"));

    // 3. mentioning the sender itself is silently dropped by derivation
    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice", "--mention", "alice",
            "hello",
        ])
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
        .args(["post", "send", path.to_str().unwrap(), "--from", "alice", "first"])
        .assert()
        .success();

    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "bob", "--reply-to", "1",
            "agreed",
        ])
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
        .args(["post", "send", path.to_str().unwrap(), "--from", "alice", "first"])
        .assert()
        .success();

    // Self-reply: only the `@#1` token, no `@alice`
    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice", "--reply-to", "1", "follow-up",
        ])
        .assert()
        .success();

    // Reply + explicit mention of the same sender: single `@alice` token
    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "bob", "--reply-to", "1", "--mention", "alice,alice",
            "also this",
        ])
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
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice", "--mention", "bob", "--stdin",
        ])
        .write_stdin(huge)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error validation:"));
}

#[test]
fn post_edit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("edit-thread.md");

    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "bob", "--title", "My Thread",
            "original",
        ])
        .assert()
        .success();

    // First real message is seq 1 (placeholder creation message abolished)
    cmd()
        .args(["post", "edit", path.to_str().unwrap(), "--seq", "1", "--from", "bob", "edited"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok post.edit"));

    cmd()
        .args(["post", "read", path.to_str().unwrap(), "--from", "1", "--to", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("edited"));

    // Edit rewrites the file but preserves the preamble verbatim
    let content = std::fs::read_to_string(dir.path().join("edit-thread.post.md")).unwrap();
    assert!(content.contains("# My Thread"));
    assert!(!content.contains("- participants:"));
    assert!(content.contains("edited"));
    assert!(!content.contains("original"));
}

#[test]
fn post_summary() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("sum.md");

    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice",
            "--title", "Daily Standup",
            "hello",
        ])
        .assert()
        .success();

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--from", "bob", "hi"])
        .assert()
        .success();

    // Title from the preamble; participants DERIVED from the sender set
    // (D1) — the preamble carries no participant list anymore.
    cmd()
        .args(["--json", "post", "summary", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\":\"Daily Standup\""))
        .stdout(predicate::str::contains("\"participants\":\"alice, bob\""))
        .stdout(predicate::str::contains("\"messages\":2"));
}

#[test]
fn post_create_removed() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("x.md");

    cmd()
        .args(["post", "create", path.to_str().unwrap(), "--title", "T"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// --- Brief (T-CLI-13..15) ---

#[test]
fn brief_create_add_read() {
    let dir = TempDir::new().unwrap();
    let brief_path = dir.path().join("brief.md");
    let entry_file = dir.path().join("notes.txt");

    std::fs::write(&entry_file, "some content").unwrap();

    cmd()
        .args([
            "brief", "create", brief_path.to_str().unwrap(),
            "--title", "My Brief", "--owner", "alice",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok brief.create"));

    cmd()
        .args([
            "brief", "add", brief_path.to_str().unwrap(),
            "--entry", "notes.txt",
            "--regex", "some",
            "--note", "Entry point",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok brief.add"));

    cmd()
        .args(["brief", "read", brief_path.to_str().unwrap(), "--full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("notes.txt"));

    // New-format literals: lowercase keys, bare prose note, full 64-hex hash
    let content = std::fs::read_to_string(dir.path().join("brief.brief.md")).unwrap();
    assert!(content.contains("- owner: alice"));
    assert!(content.contains("- path: notes.txt"));
    assert!(content.contains("- regex: some"));
    assert!(content.contains("Entry point"));
    assert!(!content.contains("> Entry point"));
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
    cmd()
        .args([
            "brief", "add", brief_path.to_str().unwrap(),
            "--entry", "src.txt", "--regex", "original",
        ])
        .assert()
        .success();

    // Fresh: regex matches + hash matches
    cmd()
        .args(["--json", "brief", "verify", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("fresh"));

    // Shifted: regex still matches, hash differs
    std::fs::write(&entry_file, "original plus more").unwrap();
    cmd()
        .args(["--json", "brief", "verify", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("shifted"));

    // Stale: regex fails to match
    std::fs::write(&entry_file, "nothing to see").unwrap();
    cmd()
        .args(["--json", "brief", "verify", brief_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("stale"));
}

// --- Contacts (T-CLI-16) ---

#[test]
fn contacts_create_add_read() {
    let dir = TempDir::new().unwrap();
    let contacts_path = dir.path().join("team.md");
    let profile_path = dir.path().join("agent.md");

    cmd()
        .args(["profile", "create", profile_path.to_str().unwrap(), "--name", "agent", "--model", "m"])
        .assert()
        .success();

    cmd()
        .args(["contacts", "create", contacts_path.to_str().unwrap(), "--title", "Team"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok contacts.create"));

    let actual_profile = dir.path().join("agent.profile.md");
    cmd()
        .args(["contacts", "add", contacts_path.to_str().unwrap(), "--profile", actual_profile.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok contacts.add"));

    // Entries are Markdown link bullets; label = target profile H1
    let content = std::fs::read_to_string(dir.path().join("team.contacts.md")).unwrap();
    assert!(content.contains("- [agent]("));

    cmd()
        .args(["--json", "contacts", "read", contacts_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"label\":\"agent\""))
        .stdout(predicate::str::contains("agent"));
}

// --- Validate (T-CLI-17..24) ---

#[test]
fn validate_ok() {
    let dir = TempDir::new().unwrap();

    // Valid .profile.md
    let profile = dir.path().join("alice.profile.md");
    cmd().args(["profile", "create", profile.to_str().unwrap(), "--name", "alice", "--model", "m"]).assert().success();

    // Valid .post.md
    let post = dir.path().join("standup.post.md");
    cmd().args(["post", "send", post.to_str().unwrap(), "--from", "alice", "--title", "S", "hello"]).assert().success();

    // Valid .brief.md
    let entry_file = dir.path().join("notes.txt");
    std::fs::write(&entry_file, "content").unwrap();
    let brief = dir.path().join("onboarding.brief.md");
    cmd().args(["brief", "create", brief.to_str().unwrap(), "--title", "B"]).assert().success();
    cmd().args(["brief", "add", brief.to_str().unwrap(), "--entry", "notes.txt"]).assert().success();

    // Valid .contacts.md
    let contacts = dir.path().join("team.contacts.md");
    cmd().args(["contacts", "create", contacts.to_str().unwrap(), "--title", "Team"]).assert().success();
    cmd().args(["contacts", "add", contacts.to_str().unwrap(), "--profile", profile.to_str().unwrap()]).assert().success();

    for path in [&profile, &post, &brief, &contacts] {
        cmd()
            .args(["validate", path.to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains("ok validate"));
    }
}

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

#[test]
fn post_read_plain_no_preamble() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("plain.md");

    cmd()
        .args([
            "post", "send", path.to_str().unwrap(),
            "--from", "alice", "--title", "Plain Check",
            "one",
        ])
        .assert()
        .success();
    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--from", "bob", "two"])
        .assert()
        .success();

    // Subset output is messages-only serialization (no preamble, POST-31)
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

// --- Append guard: missing trailing newline (review F1) ---

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
        .args(["post", "send", path.to_str().unwrap(), "--from", "bob", "second"])
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
fn post_edit_missing_body_example_shows_edit_form() {
    // Review F3: the one-retry example must match the failing command
    // (edit previously showed a send-shaped example).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("thread.md");

    cmd()
        .args(["post", "send", path.to_str().unwrap(), "--from", "alice", "first"])
        .assert()
        .success();

    cmd()
        .args([
            "post", "edit", path.to_str().unwrap(),
            "--seq", "1", "--from", "alice",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("example: paperwork post edit"))
        .stderr(predicate::str::contains("example: paperwork post send").not());
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
