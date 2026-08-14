//! T7 Ivy test-gap closure (G1–G5 superset) — CLI envelope face (additive).
//!
//! Closes the CLI-face negative / lenient-path gaps confirmed by the v0.5
//! review. Purely new tests: no existing source or test file is touched.
//!
//! Gap register covered here:
//! - G1 VAL-04: validate rejects a v0.4-shaped post (`### #N` header family)
//!   — default envelope wording + exit code;
//! - G2 VAL-05: validate rejects bad profile / brief / contacts fixtures
//!   (self-built tempdir fixtures; full default-envelope shape);
//! - G3: validate `--json` error envelope structure (keys + category);
//! - G4: post edit triple refusal — CLI envelope category + wording + file
//!   bytes unchanged (byte-level before/after compare; the golden char_tests
//!   pin the exact bytes but never assert byte-stability of the refusal);
//! - G5: lenient paths and mode combinations — read filter no-match empty
//!   envelope, summary on a missing file (pinned to the actual lenient
//!   empty-summary behavior), `--quiet` on an error path, CRLF input
//!   roundtrip, Unicode CLI roundtrip, injection-guard CLI regressions, and
//!   concurrent first-send contention (CONC-02 CLI face).
//!
//! Determinism strategy follows char_tests: every command runs with
//! `current_dir` = a fresh tempdir and takes RELATIVE paths, so no
//! machine-specific absolute path ever leaks into a pinned envelope.

use std::sync::{Arc, Barrier};

use assert_cmd::Command;
use tempfile::TempDir;

struct Run {
    stdout: String,
    stderr: String,
    code: i32,
}

fn run(dir: &TempDir, args: &[&str]) -> Run {
    let out = Command::cargo_bin("paperwork")
        .expect("binary built")
        .current_dir(dir.path())
        .args(args)
        .output()
        .expect("spawn paperwork");
    Run {
        stdout: String::from_utf8(out.stdout).expect("stdout is utf8"),
        stderr: String::from_utf8(out.stderr).expect("stderr is utf8"),
        code: out.status.code().expect("exit code"),
    }
}

fn write(dir: &TempDir, rel: &str, content: &str) {
    std::fs::write(dir.path().join(rel), content).expect("write fixture");
}

fn read_bytes(dir: &TempDir, rel: &str) -> Vec<u8> {
    std::fs::read(dir.path().join(rel)).expect("read fixture bytes")
}

fn read_file(dir: &TempDir, rel: &str) -> String {
    std::fs::read_to_string(dir.path().join(rel)).expect("read fixture")
}

// ===========================================================================
// G1 — VAL-04: validate rejects a v0.4-shaped post file (default envelope)
// ===========================================================================

/// v0.4 thread shape: `### #N` header family, no v0.5 `## #N` headers.
/// validate must surface the zero-message Parse envelope whose fix points at
/// the new header grammar + dynamic fences, exit 1.
#[test]
fn ivy_g1_validate_v04_legacy_post_default_envelope() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "legacy.post.md",
        "# Legacy Thread\n\n\
         ### #1 alice (2025-12-31T23:59:59Z)\n\n\
         Old v0.4 body text.\n\n\
         ### #2 bob (2026-01-01T00:00:00Z)\n\n\
         More old body text.\n",
    );

    let r = run(&dir, &["validate", "legacy.post.md"]);
    assert_eq!(r.code, 1, "validate of a v0.4 post must exit 1");
    assert!(r.stdout.is_empty(), "error envelope never touches stdout");
    assert_eq!(
        r.stderr,
        "error format: Parse error: no valid messages found\n\
         fix: expected '## #<seq> <sender> (<timestamp>)' headers with dynamic md fences\n\
         example: paperwork post send myfile --from alice \"hello\"\n",
        "G1: VAL-04 v0.4 legacy post envelope must stay byte-exact"
    );
}

// ===========================================================================
// G2 — VAL-05: three-format bad examples at the CLI validate face
// ===========================================================================

/// profile bad example: missing `- model:` line → error format: envelope.
#[test]
fn ivy_g2_validate_profile_missing_model_envelope() {
    let dir = TempDir::new().unwrap();
    write(&dir, "bob.profile.md", "# bob\n\nReviewer prose.\n");

    let r = run(&dir, &["validate", "bob.profile.md"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty());
    assert_eq!(
        r.stderr,
        "error format: Parse error: missing - model: line for profile 'bob'\n\
         fix: add a '- model: <model-id>' bullet line\n\
         example: - model: gpt-4o\n",
        "G2: profile missing-model envelope must stay byte-exact"
    );
}

/// brief bad examples: missing `- owner:` and missing `- created:` each
/// yield the dedicated Parse envelope (lowercase-key fix wording).
#[test]
fn ivy_g2_validate_brief_missing_owner_and_created_envelopes() {
    let dir = TempDir::new().unwrap();

    // Missing - owner:
    write(
        &dir,
        "noowner.brief.md",
        "# Guide\n\nReading list.\n\n## main.rs\n\n- path: main.rs\n- hash: ab\n",
    );
    let r = run(&dir, &["validate", "noowner.brief.md"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty());
    assert_eq!(
        r.stderr,
        "error format: Parse error: missing - owner: line for brief 'Guide'\n\
         fix: add a '- owner: <agent>' bullet line\n\
         example: - owner: alice\n",
        "G2: brief missing-owner envelope must stay byte-exact"
    );

    // Missing - created:
    write(
        &dir,
        "nocreated.brief.md",
        "# Guide\n\n- owner: alice\n\n## main.rs\n\n- path: main.rs\n- hash: ab\n",
    );
    let r = run(&dir, &["validate", "nocreated.brief.md"]);
    assert_eq!(r.code, 1);
    assert_eq!(
        r.stderr,
        "error format: Parse error: missing or invalid - created: line for brief 'Guide'\n\
         fix: add a '- created: <RFC3339>' bullet line\n\
         example: - created: 2026-01-15T10:00:00Z\n",
        "G2: brief missing-created envelope must stay byte-exact"
    );
}

/// contacts bad example: legacy bare-path bullets → the FULL default error
/// envelope shape (error line + fix + example), complementing M2's JSON-face
/// assertion in cli_integration.rs.
#[test]
fn ivy_g2_validate_contacts_legacy_full_envelope() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "team.contacts.md",
        "# Core Team\n\n- agents/alice.profile.md\n- agents/bob.profile.md\n",
    );

    let r = run(&dir, &["validate", "team.contacts.md"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty());
    assert_eq!(
        r.stderr,
        "error format: Parse error: contacts file contains legacy bare-path bullets but no link bullets\n\
         fix: this file is in the v0.4 legacy format; migrate it by hand per the CHANGELOG migration guide: wrap each path in a Markdown link bullet '- [label](path)'\n\
         example: - [alice](agents/alice.profile.md)\n",
        "G2: legacy contacts validate envelope must stay byte-exact"
    );
}

/// brief partial-migration residue (T2 guard CLI face, negative side):
/// lowercase keys but a residual `## Entries` wrapper OR `### ` entry
/// headers must be refused at validate, both variants.
#[test]
fn ivy_g2_validate_brief_partial_migration_residue_rejected() {
    let dir = TempDir::new().unwrap();

    // Variant 1: residual `## Entries` wrapper heading.
    write(
        &dir,
        "wrapper.brief.md",
        "# Guide\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n\
         ## Entries\n\n\
         ## main.rs\n\n- path: main.rs\n- hash: ab\n",
    );
    let r = run(&dir, &["validate", "wrapper.brief.md"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty());
    assert!(r.stderr.starts_with("error format:"));
    assert!(r.stderr.contains("legacy v0.4 residue"));
    assert!(r.stderr.contains("## Entries"));
    assert!(r
        .stderr
        .contains("fix: migrate this brief to the v0.5 entry layout"));

    // Variant 2: residual `### ` H3 entry headers.
    write(
        &dir,
        "h3.brief.md",
        "# Guide\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n\
         ### main.rs\n\n- path: main.rs\n- hash: ab\n",
    );
    let r = run(&dir, &["validate", "h3.brief.md"]);
    assert_eq!(r.code, 1);
    assert!(r.stderr.starts_with("error format:"));
    assert!(r.stderr.contains("legacy v0.4 residue"));
}

// ===========================================================================
// G3 — validate --json error envelope structure
// ===========================================================================

/// `--json` on any G2 bad example: stdout JSON envelope with
/// status=error / category / message / fix / example / exit_code=1 keys,
/// stderr empty, exit 1. Two representative bad examples (profile +
/// brief residue) pin the structure.
#[test]
fn ivy_g3_validate_json_error_envelope_structure() {
    let dir = TempDir::new().unwrap();
    write(&dir, "bob.profile.md", "# bob\n\nReviewer prose.\n");

    let r = run(&dir, &["--json", "validate", "bob.profile.md"]);
    assert_eq!(r.code, 1);
    assert!(r.stderr.is_empty(), "JSON errors go to stdout only");
    // Alphabetical BTreeMap key order is the frozen construction path.
    assert_eq!(
        r.stdout,
        "{\"category\":\"format\",\
         \"example\":\"- model: gpt-4o\",\
         \"exit_code\":1,\
         \"fix\":\"add a '- model: <model-id>' bullet line\",\
         \"message\":\"Parse error: missing - model: line for profile 'bob'\",\
         \"status\":\"error\"}\n",
        "G3: validate --json error envelope must stay byte-exact"
    );

    // Second bad example: brief partial-migration residue (multi-line
    // `example` field — keys and category pinned by substring).
    write(
        &dir,
        "residue.brief.md",
        "# Guide\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n\
         ## Entries\n\n\
         ### main.rs\n\n- path: main.rs\n- hash: ab\n",
    );
    let r = run(&dir, &["--json", "validate", "residue.brief.md"]);
    assert_eq!(r.code, 1);
    assert!(r.stderr.is_empty());
    assert!(r.stdout.contains("\"status\":\"error\""));
    assert!(r.stdout.contains("\"category\":\"format\""));
    assert!(r.stdout.contains("\"message\":"));
    assert!(r.stdout.contains("\"fix\":"));
    assert!(r.stdout.contains("\"example\":"));
    assert!(r.stdout.contains("\"exit_code\":1"));
}

// ===========================================================================
// G4 — post edit triple refusal: CLI envelope face + bytes unchanged
// ===========================================================================

/// Refusal 1: editing a message someone else sent.
#[test]
fn ivy_g4_edit_not_owned_cli_envelope_and_bytes_unchanged() {
    let dir = TempDir::new().unwrap();
    run(
        &dir,
        &[
            "post", "send", "g4a", "--from", "alice", "--title", "G4", "mine",
        ],
    );
    let before = read_bytes(&dir, "g4a.post.md");

    let r = run(
        &dir,
        &[
            "post", "edit", "g4a", "--seq", "1", "--from", "bob", "hijack",
        ],
    );
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty());
    assert!(r.stderr.starts_with("error not-allowed: thread_edit:"));
    assert!(r
        .stderr
        .contains("Message #1 was sent by 'alice', not 'bob'"));
    assert!(r
        .stderr
        .contains("fix: you can only edit your own messages"));
    assert_eq!(
        read_bytes(&dir, "g4a.post.md"),
        before,
        "G4: a refused edit must leave the file byte-identical"
    );
}

/// Refusal 2: editing an own message that is no longer the sender's most
/// recent one (the sender posted again afterwards).
#[test]
fn ivy_g4_edit_not_most_recent_cli_envelope_and_bytes_unchanged() {
    let dir = TempDir::new().unwrap();
    run(
        &dir,
        &[
            "post", "send", "g4b", "--from", "alice", "--title", "G4", "a1",
        ],
    );
    run(&dir, &["post", "send", "g4b", "--from", "bob", "b1"]);
    run(&dir, &["post", "send", "g4b", "--from", "alice", "a2"]);
    let before = read_bytes(&dir, "g4b.post.md");

    let r = run(
        &dir,
        &[
            "post", "edit", "g4b", "--seq", "1", "--from", "alice", "old",
        ],
    );
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty());
    assert!(r.stderr.starts_with("error not-allowed: thread_edit:"));
    assert!(r
        .stderr
        .contains("not your most recent message (your last is #3)"));
    assert!(r
        .stderr
        .contains("fix: you can only edit your most recent message"));
    assert_eq!(
        read_bytes(&dir, "g4b.post.md"),
        before,
        "G4: a refused edit must leave the file byte-identical"
    );
}

/// Refusal 3: editing an own most-recent message that is not the thread's
/// final message (someone else replied after it).
#[test]
fn ivy_g4_edit_not_final_cli_envelope_and_bytes_unchanged() {
    let dir = TempDir::new().unwrap();
    run(
        &dir,
        &[
            "post", "send", "g4c", "--from", "alice", "--title", "G4", "a1",
        ],
    );
    run(&dir, &["post", "send", "g4c", "--from", "bob", "b1"]);
    let before = read_bytes(&dir, "g4c.post.md");

    let r = run(
        &dir,
        &[
            "post", "edit", "g4c", "--seq", "1", "--from", "alice", "nope",
        ],
    );
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty());
    assert!(r.stderr.starts_with("error not-allowed: thread_edit:"));
    assert!(r
        .stderr
        .contains("not the final message in thread (last is #2)"));
    assert!(r
        .stderr
        .contains("fix: you can only edit the final message in a thread"));
    assert_eq!(
        read_bytes(&dir, "g4c.post.md"),
        before,
        "G4: a refused edit must leave the file byte-identical"
    );
}

// ===========================================================================
// G5 — lenient paths and mode combinations
// ===========================================================================

/// read filter negative face: --mention / --reply-to with no match yield
/// the empty-result envelope shape (0 messages, no body lines, `[]` in JSON).
#[test]
fn ivy_g5_read_filters_no_match_empty_envelope() {
    let dir = TempDir::new().unwrap();
    run(
        &dir,
        &[
            "post", "send", "f", "--from", "alice", "--title", "F", "hello",
        ],
    );
    run(&dir, &["post", "send", "f", "--from", "bob", "world"]);

    // Default mode: bare "0 messages" envelope, no body section.
    let r = run(&dir, &["post", "read", "f", "--mention", "ghost"]);
    assert_eq!(r.code, 0, "a no-match filter is a lenient success");
    assert_eq!(r.stdout, "ok post.read 0 messages\n");

    let r = run(&dir, &["post", "read", "f", "--reply-to", "99"]);
    assert_eq!(r.code, 0);
    assert_eq!(r.stdout, "ok post.read 0 messages\n");

    // JSON mode: empty messages array, status stays ok.
    let r = run(&dir, &["--json", "post", "read", "f", "--mention", "ghost"]);
    assert_eq!(r.code, 0);
    assert_eq!(
        r.stdout,
        "{\"command\":\"post.read\",\"conclusion\":\"0 messages\",\"messages\":[],\"status\":\"ok\"}\n"
    );
}

/// summary on a missing thread: pinned to the ACTUAL lenient behavior —
/// an empty summary with ok status (not a NotFound envelope).
#[test]
fn ivy_g5_summary_missing_file_lenient_empty_summary() {
    let dir = TempDir::new().unwrap();

    let r = run(&dir, &["post", "summary", "ghost"]);
    assert_eq!(r.code, 0, "summary of a missing thread stays a success");
    assert!(r.stderr.is_empty());
    assert_eq!(
        r.stdout,
        "ok post.summary ghost.post.md\n\
         title: \n\
         participants: \n\
         messages: 0\n\
         last.snippet: \n",
        "G5: missing-file summary must stay the lenient empty envelope"
    );

    let r = run(&dir, &["--json", "post", "summary", "ghost"]);
    assert_eq!(r.code, 0);
    assert_eq!(
        r.stdout,
        "{\"command\":\"post.summary\",\"conclusion\":\"ghost.post.md\",\
         \"last.snippet\":\"\",\"messages\":0,\"participants\":\"\",\
         \"status\":\"ok\",\"title\":\"\"}\n"
    );
}

/// Representative error-path `--quiet` combination: quiet never silences
/// the error envelope — stderr keeps the full envelope, stdout stays empty,
/// exit code stays 1.
#[test]
fn ivy_g5_quiet_error_envelope_unchanged() {
    let dir = TempDir::new().unwrap();
    write(&dir, "legacy.post.md", "# Legacy\n\n### #1 alice (ts)\n");

    let without_quiet = run(&dir, &["validate", "legacy.post.md"]);
    let with_quiet = run(&dir, &["--quiet", "validate", "legacy.post.md"]);

    assert_eq!(with_quiet.code, 1);
    assert!(
        with_quiet.stdout.is_empty(),
        "quiet error path keeps stdout empty"
    );
    assert_eq!(
        with_quiet.stderr, without_quiet.stderr,
        "G5: --quiet must not alter the error envelope bytes"
    );
    assert!(with_quiet.stderr.starts_with("error format:"));
}

/// CRLF input CLI face: a legal post file with CRLF endings reads back
/// clean, accepts an appended send, and validates — full roundtrip.
#[test]
fn ivy_g5_crlf_post_send_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "c.post.md",
        "# CRLF Chat\r\n\r\n\
         ## #1 alice (2026-01-15T10:30:00Z)\r\n\r\n\
         ```md\r\nhello crlf\r\n```\r\n",
    );

    // Read normalizes on the fly; no CR byte may leak into the envelope.
    let r = run(&dir, &["post", "read", "c"]);
    assert_eq!(r.code, 0);
    assert!(!r.stdout.contains('\r'));
    assert!(r.stdout.contains("ok post.read 1 messages"));
    assert!(r.stdout.contains("#1 alice"));
    assert!(r.stdout.contains("hello crlf"));

    // Append into the CRLF file and read both messages back intact.
    let r = run(&dir, &["post", "send", "c", "--from", "bob", "second"]);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("seq: 2"));

    let r = run(&dir, &["post", "read", "c"]);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("ok post.read 2 messages"));
    assert!(r.stdout.contains("hello crlf"));
    assert!(r.stdout.contains("second"));

    // The mixed-endings file still validates.
    let r = run(&dir, &["validate", "c.post.md"]);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("ok validate"));
}

/// Unicode CLI face: Chinese body + sender survive send → read and appear
/// verbatim in the JSON output (paperwork output is always UTF-8 bytes).
#[test]
fn ivy_g5_unicode_send_read_json_roundtrip() {
    let dir = TempDir::new().unwrap();

    let r = run(
        &dir,
        &[
            "post",
            "send",
            "uni",
            "--from",
            "小明",
            "--title",
            "中文标题",
            "今天天气不错。",
        ],
    );
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("ok post.send"));
    assert!(r.stdout.contains("seq: 1"));

    // On-disk preamble + body carry the Unicode verbatim.
    let content = read_file(&dir, "uni.post.md");
    assert!(content.contains("# 中文标题"));
    assert!(content.contains("## #1 小明 ("));
    assert!(content.contains("今天天气不错。"));

    // Default read roundtrip.
    let r = run(&dir, &["post", "read", "uni"]);
    assert_eq!(r.code, 0);
    assert!(!r.stdout.contains('\r'));
    assert!(r.stdout.contains("#1 小明"));
    assert!(r.stdout.contains("今天天气不错。"));

    // JSON read keeps the Unicode sender (no escaping loss).
    let r = run(&dir, &["--json", "post", "read", "uni"]);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("\"sender\":\"小明\""));
    assert!(r.stdout.contains("今天天气不错。"));

    // JSON summary derives participants from the Unicode sender set.
    let r = run(&dir, &["--json", "post", "summary", "uni"]);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("\"title\":\"中文标题\""));
    assert!(r.stdout.contains("\"participants\":\"小明\""));
}

/// Injection-guard CLI regression: literal newlines smuggled through CLI
/// arguments are refused at the flag layer — `--title` on post send and
/// `--model` on profile create — and nothing is written.
#[test]
fn ivy_g5_injection_guards_cli_literal_newline_refused() {
    let dir = TempDir::new().unwrap();

    // --title containing a literal newline would inject structure into the
    // preamble H1 — refuse.
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "inj",
            "--from",
            "alice",
            "--title",
            "bad\ntitle",
            "hi",
        ],
    );
    assert_eq!(r.code, 1);
    assert!(r.stderr.starts_with("error validation:"));
    assert!(r.stderr.contains("thread title contains a line break"));
    assert!(!dir.path().join("inj.post.md").exists());

    // --model containing a literal newline — same guard via profile create.
    let r = run(
        &dir,
        &[
            "profile", "create", "injp", "--name", "inj", "--model", "m1\nm2",
        ],
    );
    assert_eq!(r.code, 1);
    assert!(r.stderr.starts_with("error validation:"));
    assert!(r.stderr.contains("model contains a line break"));
    assert!(!dir.path().join("injp.profile.md").exists());
}

/// CONC-02 CLI face: two concurrent processes race the first send on a
/// non-existent thread — preamble written exactly once, seq {1, 2}, both
/// invocations succeed.
#[test]
fn ivy_g5_concurrent_first_send_cli_contention() {
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path().to_path_buf();
    let barrier = Arc::new(Barrier::new(2));

    let handles: Vec<_> = ["alice", "bob"]
        .into_iter()
        .map(|sender| {
            let barrier = Arc::clone(&barrier);
            let dir_path = dir_path.clone();
            std::thread::spawn(move || {
                barrier.wait();
                Command::cargo_bin("paperwork")
                    .expect("binary built")
                    .current_dir(&dir_path)
                    .args([
                        "post",
                        "send",
                        "race",
                        "--from",
                        sender,
                        "--title",
                        "Race Thread",
                        "hi from the racer",
                    ])
                    .output()
                    .expect("spawn paperwork")
            })
        })
        .collect();

    for out in handles.into_iter().map(|h| h.join().expect("join racer")) {
        assert!(
            out.status.success(),
            "both racers must win a serialized slot: {:?}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let content = read_file(&dir, "race.post.md");
    assert_eq!(
        content.matches("# Race Thread").count(),
        1,
        "preamble must be written exactly once under contention"
    );

    let r = run(&dir, &["post", "read", "race"]);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("ok post.read 2 messages"));
    assert!(r.stdout.contains("#1 "));
    assert!(r.stdout.contains("#2 "));

    let r = run(&dir, &["validate", "race.post.md"]);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("ok validate"));
}
