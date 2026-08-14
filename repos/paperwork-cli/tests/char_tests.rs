//! T1 behavior lock — characterization tests (CLI face).
//!
//! Golden-envelope snapshots of the whole command surface: stdout/stderr
//! byte-exact (LF-only) plus exit codes. These tests are the regression
//! gate for the upcoming structural refactor; every literal below is the
//! frozen v0.5.0 output contract.
//!
//! Determinism strategy:
//! - every command runs with `current_dir` = a fresh tempdir and takes
//!   RELATIVE paths, so no machine-specific absolute path ever leaks into
//!   an envelope;
//! - read/summary/validate fixtures are hand-written files with fixed
//!   timestamps (never produced by the clock);
//! - `post send` envelopes carry no clock value; where a produced FILE
//!   must be pinned, the wall-clock timestamp is masked to `(TS)` and
//!   everything else is byte-exact;
//! - Rust stdout is byte-level `\n` (no CRLF translation) — each test also
//!   asserts the capture carries no `\r`.
//!
//! Additive only — no existing source/test file is modified.

use std::sync::LazyLock;

use assert_cmd::Command;
use regex::Regex;
use tempfile::TempDir;

/// SHA-256 of the exact bytes `fn main() {}\n` (fixture entry file).
const H_MAIN: &str = "536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4";
/// SHA-256 of the exact bytes `pub fn lib() {}\n` (fixture entry file).
const H_LIB: &str = "0db36e0c52631e298f490f390d09308f9db27b8310b0aed0ad6650714455a69e";

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

fn run_stdin(dir: &TempDir, args: &[&str], stdin: &str) -> Run {
    let out = Command::cargo_bin("paperwork")
        .expect("binary built")
        .current_dir(dir.path())
        .args(args)
        .write_stdin(stdin)
        .output()
        .expect("spawn paperwork");
    Run {
        stdout: String::from_utf8(out.stdout).expect("stdout is utf8"),
        stderr: String::from_utf8(out.stderr).expect("stderr is utf8"),
        code: out.status.code().expect("exit code"),
    }
}

/// Byte-exact golden assertion for a stream.
fn gold(actual: &str, expected: &str) {
    assert!(
        !actual.contains('\r'),
        "CR byte leaked into captured output: {:?}",
        actual
    );
    assert_eq!(actual, expected, "golden snapshot mismatch");
}

fn write(dir: &TempDir, rel: &str, content: &str) {
    std::fs::write(dir.path().join(rel), content).expect("write fixture");
}

fn read_file(dir: &TempDir, rel: &str) -> String {
    std::fs::read_to_string(dir.path().join(rel)).expect("read fixture")
}

/// Mask wall-clock message timestamps `(YYYY-MM-DDTHH:MM:SSZ)` → `(TS)`.
fn mask_ts(text: &str) -> String {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z\)").expect("valid regex")
    });
    RE.replace_all(text, "(TS)").into_owned()
}

/// Mask the brief `- created:` wall-clock line.
fn mask_created(text: &str) -> String {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?m)^- created: .*$").expect("valid regex"));
    RE.replace_all(text, "- created: (TS)").into_owned()
}

/// Canonical 3-message thread fixture (fixed timestamps, LF).
const CHAT: &str = "# Chat Log\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nping @bob about the parser\n```\n\n## #2 bob (2026-01-15T10:31:00Z)\n\n```md\n@alice @#1 merged the fix\n```\n\n## #3 alice (2026-01-15T10:32:00Z)\n\n```md\nclosing out the thread\n```\n\n";

/// 4-message interleaved thread (alice's last is not the thread's last).
const MIXED: &str = "# Mixed\n\n## #1 alice (2026-01-16T09:00:00Z)\n\n```md\na1\n```\n\n## #2 bob (2026-01-16T09:01:00Z)\n\n```md\nb1\n```\n\n## #3 alice (2026-01-16T09:02:00Z)\n\n```md\na2\n```\n\n## #4 bob (2026-01-16T09:03:00Z)\n\n```md\nb2\n```\n\n";

/// Hand-written brief fixture (fixed created stamp; hashes pinned above).
const GUIDE: &str = "# Guide Brief\n\nReading list for the module.\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n## main.rs\n\n- path: main.rs\n- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4\n- regex: fn main\n\nEntry point of the tool.\n\n## lib.rs\n\n- path: lib.rs\n- hash: 0db36e0c52631e298f490f390d09308f9db27b8310b0aed0ad6650714455a69e\n- regex: zzz\n";

/// v0.4 legacy thread shape (H3 headers, no v0.5 headers).
const LEGACY_POST: &str =
    "# Legacy Thread\n\n### #1 alice (2025-12-31T23:59:59Z)\n\nOld v0.4 body text.\n";

/// v0.4 legacy contacts shape (bare path bullets).
const LEGACY_CONTACTS: &str = "# Legacy\n\n- agents/alice.profile.md\n";

fn setup_guide(dir: &TempDir) {
    write(dir, "guide.brief.md", GUIDE);
    write(dir, "main.rs", "fn main() {}\n");
    write(dir, "lib.rs", "pub fn lib() {}\n");
}

// ===========================================================================
// post send — golden envelopes (no clock value in the envelope)
// ===========================================================================

#[test]
fn char_post_send_default_json_quiet_envelopes() {
    // default mode
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &["post", "send", "chat", "--from", "alice", "Hello world"],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok post.send #1 -> chat.post.md\nseq: 1\npath: chat.post.md\nsender: alice\n",
    );
    gold(&r.stderr, "");
    // produced file: preamble from default title + canonical message
    gold(
        &mask_ts(&read_file(&dir, "chat.post.md")),
        "# chat\n\n## #1 alice (TS)\n\n```md\nHello world\n```\n\n",
    );

    // json mode
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "--json",
            "post",
            "send",
            "chat",
            "--from",
            "alice",
            "Hello world",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "{\"command\":\"post.send\",\"conclusion\":\"#1 -> chat.post.md\",\"path\":\"chat.post.md\",\"sender\":\"alice\",\"seq\":\"1\",\"status\":\"ok\"}\n",
    );

    // quiet mode suppresses only the status line
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "--quiet",
            "post",
            "send",
            "chat",
            "--from",
            "alice",
            "Hello world",
        ],
    );
    assert_eq!(r.code, 0);
    gold(&r.stdout, "seq: 1\npath: chat.post.md\nsender: alice\n");
}

#[test]
fn char_post_send_seq_increments_and_title_flag() {
    let dir = TempDir::new().unwrap();
    run(&dir, &["post", "send", "chat", "--from", "alice", "one"]);
    let r = run(&dir, &["post", "send", "chat", "--from", "bob", "two"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok post.send #2 -> chat.post.md\nseq: 2\npath: chat.post.md\nsender: bob\n",
    );

    // --title overrides the preamble on first write
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--from",
            "alice",
            "--title",
            "My Thread",
            "Hello world",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        &mask_ts(&read_file(&dir, "chat.post.md")),
        "# My Thread\n\n## #1 alice (TS)\n\n```md\nHello world\n```\n\n",
    );
}

#[test]
fn char_post_send_stdin_body() {
    let dir = TempDir::new().unwrap();
    let r = run_stdin(
        &dir,
        &["post", "send", "notes", "--from", "alice", "--stdin"],
        "stdin body line\n",
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok post.send #1 -> notes.post.md\nseq: 1\npath: notes.post.md\nsender: alice\n",
    );
    gold(
        &mask_ts(&read_file(&dir, "notes.post.md")),
        "# notes\n\n## #1 alice (TS)\n\n```md\nstdin body line\n\n```\n\n",
    );
}

#[test]
fn char_post_send_mention_reply_token_injection() {
    let dir = TempDir::new().unwrap();
    run(&dir, &["post", "send", "chat", "--from", "alice", "start"]);
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--from",
            "carol",
            "--reply-to",
            "1",
            "--mention",
            "alice,bob",
            "follow up",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok post.send #2 -> chat.post.md\nseq: 2\npath: chat.post.md\nsender: carol\n",
    );
    // Token line: `@#N` first, then `@name` in order; implicit original-
    // sender mention (alice) already listed → no duplicate.
    gold(
        &mask_ts(&read_file(&dir, "chat.post.md")),
        "# chat\n\n## #1 alice (TS)\n\n```md\nstart\n```\n\n## #2 carol (TS)\n\n```md\n@#1 @alice @bob\n\nfollow up\n```\n\n",
    );
}

// ===========================================================================
// post read — four output modes + filters, byte-exact
// ===========================================================================

#[test]
fn char_post_read_default() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(&dir, &["post", "read", "chat"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok post.read 3 messages\n---\n#1 alice 2026-01-15T10:30:00Z mentions:bob\n  ping @bob about the parser\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n#3 alice 2026-01-15T10:32:00Z\n  closing out the thread\n",
    );
    gold(&r.stderr, "");
}

#[test]
fn char_post_read_json_shape() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(&dir, &["--json", "post", "read", "chat"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "{\"command\":\"post.read\",\"conclusion\":\"3 messages\",\"messages\":[{\"body\":\"ping @bob about the parser\",\"mentions\":[\"bob\"],\"reply_to\":null,\"sender\":\"alice\",\"seq\":1,\"timestamp\":\"2026-01-15T10:30:00Z\"},{\"body\":\"@alice @#1 merged the fix\",\"mentions\":[\"alice\"],\"reply_to\":1,\"sender\":\"bob\",\"seq\":2,\"timestamp\":\"2026-01-15T10:31:00Z\"},{\"body\":\"closing out the thread\",\"mentions\":[],\"reply_to\":null,\"sender\":\"alice\",\"seq\":3,\"timestamp\":\"2026-01-15T10:32:00Z\"}],\"status\":\"ok\"}\n",
    );
}

#[test]
fn char_post_read_plain_and_quiet() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);

    // plain = serialized messages only, no preamble (BDD:POST-31)
    let r = run(&dir, &["--plain", "post", "read", "chat"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nping @bob about the parser\n```\n\n## #2 bob (2026-01-15T10:31:00Z)\n\n```md\n@alice @#1 merged the fix\n```\n\n## #3 alice (2026-01-15T10:32:00Z)\n\n```md\nclosing out the thread\n```\n\n",
    );

    // quiet drops only the `ok` status line
    let r = run(&dir, &["--quiet", "post", "read", "chat"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "---\n#1 alice 2026-01-15T10:30:00Z mentions:bob\n  ping @bob about the parser\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n#3 alice 2026-01-15T10:32:00Z\n  closing out the thread\n",
    );
}

#[test]
fn char_post_read_filters_and_limit() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);

    let r = run(&dir, &["post", "read", "chat", "--from", "2", "--to", "2"]);
    gold(
        &r.stdout,
        "ok post.read 1 messages\n---\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n",
    );

    let r = run(&dir, &["post", "read", "chat", "--mention", "alice"]);
    gold(
        &r.stdout,
        "ok post.read 1 messages\n---\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n",
    );

    let r = run(&dir, &["post", "read", "chat", "--reply-to", "1"]);
    gold(
        &r.stdout,
        "ok post.read 1 messages\n---\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n",
    );

    // limit keeps the LAST N and reports `showing`
    let r = run(&dir, &["post", "read", "chat", "--limit", "2"]);
    gold(
        &r.stdout,
        "ok post.read 3 messages\nshowing: 2/3\n---\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n#3 alice 2026-01-15T10:32:00Z\n  closing out the thread\n",
    );

    // plain subset carries no preamble either
    let r = run(
        &dir,
        &[
            "--plain", "post", "read", "chat", "--from", "2", "--to", "3",
        ],
    );
    gold(
        &r.stdout,
        "## #2 bob (2026-01-15T10:31:00Z)\n\n```md\n@alice @#1 merged the fix\n```\n\n## #3 alice (2026-01-15T10:32:00Z)\n\n```md\nclosing out the thread\n```\n\n",
    );
}

#[test]
fn char_post_read_missing_thread_error() {
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["post", "read", "ghost"]);
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error not-found: Thread 'ghost.post.md' not found\nfix: send a message first to create the thread\nexample: paperwork post send ghost.post.md --from <name> <body>\n",
    );
}

// ===========================================================================
// post summary
// ===========================================================================

#[test]
fn char_post_summary_default_and_quiet() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(&dir, &["post", "summary", "chat"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok post.summary chat.post.md\ntitle: Chat Log\nparticipants: alice, bob\nmessages: 3\nlast.sender: alice\nlast.time: 2026-01-15T10:32:00Z\nlast.snippet: closing out the thread\n",
    );

    let r = run(&dir, &["--quiet", "post", "summary", "chat"]);
    gold(
        &r.stdout,
        "title: Chat Log\nparticipants: alice, bob\nmessages: 3\nlast.sender: alice\nlast.time: 2026-01-15T10:32:00Z\nlast.snippet: closing out the thread\n",
    );
}

#[test]
fn char_post_summary_json_shape() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(&dir, &["--json", "post", "summary", "chat"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "{\"command\":\"post.summary\",\"conclusion\":\"chat.post.md\",\"last.sender\":\"alice\",\"last.snippet\":\"closing out the thread\",\"last.time\":\"2026-01-15T10:32:00Z\",\"messages\":3,\"participants\":\"alice, bob\",\"status\":\"ok\",\"title\":\"Chat Log\"}\n",
    );
}

// ===========================================================================
// post edit — success rewrites the whole file byte-exactly
// ===========================================================================

#[test]
fn char_post_edit_success_rewrites_file() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(
        &dir,
        &[
            "post",
            "edit",
            "chat",
            "--seq",
            "3",
            "--from",
            "alice",
            "edited closing line",
        ],
    );
    assert_eq!(r.code, 0);
    gold(&r.stdout, "ok post.edit #3\nseq: 3\npath: chat.post.md\n");
    gold(&r.stderr, "");
    // Preamble carried over verbatim + canonical re-serialization.
    gold(
        &read_file(&dir, "chat.post.md"),
        "# Chat Log\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nping @bob about the parser\n```\n\n## #2 bob (2026-01-15T10:31:00Z)\n\n```md\n@alice @#1 merged the fix\n```\n\n## #3 alice (2026-01-15T10:32:00Z)\n\n```md\nedited closing line\n```\n\n",
    );
}

#[test]
fn char_post_edit_not_owned_rejected() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(
        &dir,
        &[
            "post", "edit", "chat", "--seq", "3", "--from", "bob", "hijack",
        ],
    );
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error not-allowed: thread_edit: Message #3 was sent by 'alice', not 'bob'\nfix: you can only edit your own messages\nexample: paperwork post edit chat.post.md --seq 3 --from alice <body>\n",
    );
}

#[test]
fn char_post_edit_not_most_recent_rejected() {
    let dir = TempDir::new().unwrap();
    write(&dir, "mixed.post.md", MIXED);
    let r = run(
        &dir,
        &[
            "post", "edit", "mixed", "--seq", "1", "--from", "alice", "old",
        ],
    );
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error not-allowed: thread_edit: Message #1 is not your most recent message (your last is #3)\nfix: you can only edit your most recent message\nexample: paperwork post edit mixed.post.md --seq 3 --from alice <body>\n",
    );
}

#[test]
fn char_post_edit_not_final_rejected() {
    let dir = TempDir::new().unwrap();
    write(&dir, "mixed.post.md", MIXED);
    let r = run(
        &dir,
        &[
            "post", "edit", "mixed", "--seq", "3", "--from", "alice", "not last",
        ],
    );
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error not-allowed: thread_edit: Message #3 is not the final message in thread (last is #4)\nfix: you can only edit the final message in a thread\nexample: paperwork post edit mixed.post.md --seq 4 --from alice <body>\n",
    );
}

// ===========================================================================
// post send — frozen error paths
// ===========================================================================

#[test]
fn char_post_send_reply_to_zero_rejected() {
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--from",
            "alice",
            "--reply-to",
            "0",
            "Hello",
        ],
    );
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error validation: Validation error: reply-to must be >= 1\nfix: pass the seq number of an existing message (seq numbers start at 1)\nexample: paperwork post send chat.post.md --from alice --reply-to 1 \"Hello\"\n",
    );
}

#[test]
fn char_post_send_empty_body_rejected() {
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["post", "send", "chat", "--from", "alice", "   "]);
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error validation: Validation error: message body is empty\nfix: provide a non-empty message body\nexample: paperwork post send chat.post.md --from alice \"Hello\"\n",
    );
}

#[test]
fn char_post_send_legacy_thread_write_guard() {
    let dir = TempDir::new().unwrap();
    write(&dir, "legacy.post.md", LEGACY_POST);
    let r = run(&dir, &["post", "send", "legacy", "--from", "bob", "hi"]);
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error format: Parse error: thread file contains legacy v0.4 message headers but no v0.5 message headers\nfix: this file is in the v0.4 legacy format; v0.5 is not forward compatible - migrate it by hand per the CHANGELOG migration guide before writing\nexample: see CHANGELOG.md, [0.5.0] 'Migration guide (manual)', step 1 (post)\n",
    );
    // file untouched
    gold(&read_file(&dir, "legacy.post.md"), LEGACY_POST);
}

// ===========================================================================
// brief — create / add / read / verify / remove + M1 note guard
// ===========================================================================

/// Preamble-only brief (fixed created stamp) used by the add/remove tests.
const BRIEF_EMPTY: &str = "# Notes\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n";

#[test]
fn char_brief_create_default_envelope_and_file() {
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "brief",
            "create",
            "notes",
            "--title",
            "Team Notes",
            "--owner",
            "alice",
            "--description",
            "Reading list.",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok brief.create notes.brief.md\npath: notes.brief.md\ntitle: Team Notes\n",
    );
    gold(&r.stderr, "");
    gold(
        &mask_created(&read_file(&dir, "notes.brief.md")),
        "# Team Notes\n\nReading list.\n\n- owner: alice\n- created: (TS)\n",
    );
}

#[test]
fn char_brief_add_entry_with_regex_and_note() {
    let dir = TempDir::new().unwrap();
    write(&dir, "notes.brief.md", BRIEF_EMPTY);
    write(&dir, "main.rs", "fn main() {}\n");
    let r = run(
        &dir,
        &[
            "brief",
            "add",
            "notes",
            "--entry",
            "main.rs",
            "--regex",
            "fn main",
            "--note",
            "Entry point.",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok brief.add main.rs -> notes.brief.md\nbrief: notes.brief.md\nentry: main.rs\n",
    );
    gold(&r.stderr, "");
    gold(
        &read_file(&dir, "notes.brief.md"),
        &format!(
            "# Notes\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n## main.rs\n\n- path: main.rs\n- hash: {}\n- regex: fn main\n\nEntry point.\n",
            H_MAIN
        ),
    );
}

#[test]
fn char_brief_add_attribute_shaped_note_is_clap_rejected() {
    // CLI-surface lock: an attribute-shaped note value starts with `-`, so
    // clap refuses it as a flag-shaped token BEFORE the M1 guard can run
    // (exit 2, usage error). The core-level M1 refusal itself is locked by
    // the fence-opening variant below and by paperwork-core tests.
    let dir = TempDir::new().unwrap();
    write(&dir, "notes.brief.md", BRIEF_EMPTY);
    write(&dir, "main.rs", "fn main() {}\n");
    let r = run(
        &dir,
        &[
            "brief",
            "add",
            "notes",
            "--entry",
            "main.rs",
            "--note",
            "- key: value",
        ],
    );
    assert_eq!(r.code, 2, "clap usage error must exit 2");
    gold(&r.stdout, "");
    assert!(
        !r.stderr.is_empty(),
        "clap usage error must write to stderr"
    );
    // nothing written
    gold(&read_file(&dir, "notes.brief.md"), BRIEF_EMPTY);
}

#[test]
fn char_brief_add_note_guard_regex_fence_first_line() {
    let dir = TempDir::new().unwrap();
    write(&dir, "notes.brief.md", BRIEF_EMPTY);
    write(&dir, "main.rs", "fn main() {}\n");
    let r = run(
        &dir,
        &[
            "brief",
            "add",
            "notes",
            "--entry",
            "main.rs",
            "--note",
            "```regex\nx\n```",
        ],
    );
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error validation: Validation error: note is not representable in brief format: note starts with a ```regex fence opening line\nfix: start the note with a plain prose line; attribute-shaped '- key: value' first lines and ```regex fence openings are reserved for entry attributes\nexample: paperwork brief add notes.brief.md --entry main.rs --note \"Reading notes for this file\"\n",
    );
    gold(&read_file(&dir, "notes.brief.md"), BRIEF_EMPTY);
}

#[test]
fn char_brief_read_default_full_quiet_plain() {
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);

    let r = run(&dir, &["brief", "read", "guide"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok brief.read 2 entries\ntitle: Guide Brief\nowner: alice\n---\nmain.rs: main.rs\nlib.rs: lib.rs\n",
    );
    gold(&r.stderr, "");

    let r = run(&dir, &["brief", "read", "guide", "--full"]);
    gold(
        &r.stdout,
        &format!(
            "ok brief.read 2 entries\ntitle: Guide Brief\nowner: alice\n---\nmain.rs: main.rs (hash: {}) regex: fn main note: Entry point of the tool.\nlib.rs: lib.rs (hash: {}) regex: zzz\n",
            H_MAIN, H_LIB
        ),
    );

    let r = run(&dir, &["--quiet", "brief", "read", "guide"]);
    gold(
        &r.stdout,
        "title: Guide Brief\nowner: alice\n---\nmain.rs: main.rs\nlib.rs: lib.rs\n",
    );

    let r = run(&dir, &["--plain", "brief", "read", "guide"]);
    gold(&r.stdout, GUIDE);
}

#[test]
fn char_brief_read_json_shape() {
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);

    let r = run(&dir, &["--json", "brief", "read", "guide"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        &format!(
            "{{\"command\":\"brief.read\",\"conclusion\":\"2 entries\",\"entries\":[{{\"hash\":\"{}\",\"path\":\"main.rs\",\"title\":\"main.rs\"}},{{\"hash\":\"{}\",\"path\":\"lib.rs\",\"title\":\"lib.rs\"}}],\"owner\":\"alice\",\"status\":\"ok\",\"title\":\"Guide Brief\"}}\n",
            H_MAIN, H_LIB
        ),
    );

    let r = run(&dir, &["--json", "brief", "read", "guide", "--full"]);
    gold(
        &r.stdout,
        &format!(
            "{{\"command\":\"brief.read\",\"conclusion\":\"2 entries\",\"entries\":[{{\"hash\":\"{}\",\"note\":\"Entry point of the tool.\",\"path\":\"main.rs\",\"regex\":\"fn main\",\"title\":\"main.rs\"}},{{\"hash\":\"{}\",\"path\":\"lib.rs\",\"regex\":\"zzz\",\"title\":\"lib.rs\"}}],\"owner\":\"alice\",\"status\":\"ok\",\"title\":\"Guide Brief\"}}\n",
            H_MAIN, H_LIB
        ),
    );
}

#[test]
fn char_brief_verify_default_and_json() {
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);

    let r = run(&dir, &["brief", "verify", "guide"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok brief.verify 1/2 fresh\n---\nmain.rs: fresh\nlib.rs: stale\n",
    );

    let r = run(&dir, &["--json", "brief", "verify", "guide"]);
    gold(
        &r.stdout,
        "{\"command\":\"brief.verify\",\"conclusion\":\"1/2 fresh\",\"results\":[{\"path\":\"main.rs\",\"status\":\"fresh\",\"title\":\"main.rs\"},{\"path\":\"lib.rs\",\"status\":\"stale\",\"title\":\"lib.rs\"}],\"status\":\"ok\"}\n",
    );
}

#[test]
fn char_brief_remove_rewrites_file() {
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(
        &dir,
        &["brief", "remove", "guide", "--entry-title", "lib.rs"],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok brief.remove lib.rs\nbrief: guide.brief.md\nremoved: lib.rs\n",
    );
    gold(
        &read_file(&dir, "guide.brief.md"),
        &format!(
            "# Guide Brief\n\nReading list for the module.\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n## main.rs\n\n- path: main.rs\n- hash: {}\n- regex: fn main\n\nEntry point of the tool.\n",
            H_MAIN
        ),
    );
}

// ===========================================================================
// contacts — create / add / read (+ JSON enrichment) + B1 legacy guard
// ===========================================================================

const PROFILE_ALICE: &str = "# alice\n\n- model: m1\n";
const PROFILE_BOB: &str = "# bob\n\nReviewer.\n\n- model: m2\n";

#[test]
fn char_contacts_create_add_read_default() {
    let dir = TempDir::new().unwrap();
    write(&dir, "alice.profile.md", PROFILE_ALICE);
    write(&dir, "bob.profile.md", PROFILE_BOB);

    let r = run(&dir, &["contacts", "create", "team"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok contacts.create team.contacts.md\npath: team.contacts.md\ntitle: Contacts\n",
    );
    gold(&r.stderr, "");
    gold(&read_file(&dir, "team.contacts.md"), "# Contacts\n\n");

    let r = run(
        &dir,
        &["contacts", "add", "team", "--profile", "alice.profile.md"],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok contacts.add alice.profile.md -> team.contacts.md\ncontacts: team.contacts.md\nprofile: alice.profile.md\n",
    );
    let r = run(
        &dir,
        &["contacts", "add", "team", "--profile", "bob.profile.md"],
    );
    assert_eq!(r.code, 0);
    gold(
        &read_file(&dir, "team.contacts.md"),
        "# Contacts\n\n- [alice](alice.profile.md)\n- [bob](bob.profile.md)\n",
    );

    // read: label derived from the profile H1; description enrichment
    let r = run(&dir, &["contacts", "read", "team"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok contacts.read 2 contacts\n---\nalice.profile.md: alice\nbob.profile.md: bob (Reviewer.)\n",
    );
    gold(&r.stderr, "");
}

#[test]
fn char_contacts_read_json_shape_enriched() {
    let dir = TempDir::new().unwrap();
    write(&dir, "alice.profile.md", PROFILE_ALICE);
    write(&dir, "bob.profile.md", PROFILE_BOB);
    write(
        &dir,
        "team.contacts.md",
        "# Contacts\n\n- [alice](alice.profile.md)\n- [bob](bob.profile.md)\n",
    );
    let r = run(&dir, &["--json", "contacts", "read", "team"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "{\"command\":\"contacts.read\",\"conclusion\":\"2 contacts\",\"contacts\":[{\"description\":\"\",\"label\":\"alice\",\"name\":\"alice\",\"path\":\"alice.profile.md\"},{\"description\":\"Reviewer.\",\"label\":\"bob\",\"name\":\"bob\",\"path\":\"bob.profile.md\"}],\"status\":\"ok\"}\n",
    );
}

#[test]
fn char_contacts_add_legacy_bare_bullet_guard() {
    let dir = TempDir::new().unwrap();
    write(&dir, "legacy.contacts.md", LEGACY_CONTACTS);
    let r = run(
        &dir,
        &["contacts", "add", "legacy", "--profile", "bob.profile.md"],
    );
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error format: Parse error: contacts file contains legacy bare-path bullets that v0.5 parsing ignores\nfix: this file is in the v0.4 legacy format; v0.5 is not forward compatible - migrate it by hand per the CHANGELOG migration guide before adding entries\nexample: see CHANGELOG.md, [0.5.0] 'Migration guide (manual)', contacts\n",
    );
    // file untouched
    gold(&read_file(&dir, "legacy.contacts.md"), LEGACY_CONTACTS);
}

// ===========================================================================
// profile — create (scoped) / edit / show / list
// ===========================================================================

#[test]
fn char_profile_create_with_scope_edit_show() {
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "profile",
            "create",
            "alice",
            "--name",
            "alice",
            "--model",
            "m1",
            "--description",
            "Core agent.",
            "--scope-read",
            "src/**",
            "docs/**",
            "--scope-write",
            "src/parser/**",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok profile.create alice.profile.md\npath: alice.profile.md\nname: alice\n",
    );
    gold(&r.stderr, "");
    gold(
        &read_file(&dir, "alice.profile.md"),
        "# alice\n\nCore agent.\n\n- model: m1\n\n## Scope\n\n- read: src/**\n- read: docs/**\n- write: src/parser/**\n",
    );

    // edit replaces model + description; scopes survive verbatim
    let r = run(
        &dir,
        &[
            "profile",
            "edit",
            "alice",
            "--model",
            "m2",
            "--description",
            "Updated.",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok profile.edit alice.profile.md\nchanged: model, description\n",
    );
    gold(
        &read_file(&dir, "alice.profile.md"),
        "# alice\n\nUpdated.\n\n- model: m2\n\n## Scope\n\n- read: src/**\n- read: docs/**\n- write: src/parser/**\n",
    );

    // show default envelope
    let r = run(&dir, &["profile", "show", "alice"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok profile.show alice\nname: alice\nmodel: m2\ndescription: Updated.\nscope.read: src/**, docs/**\nscope.write: src/parser/**\n",
    );

    // show plain = raw file bytes
    let r = run(&dir, &["--plain", "profile", "show", "alice"]);
    gold(
        &r.stdout,
        "# alice\n\nUpdated.\n\n- model: m2\n\n## Scope\n\n- read: src/**\n- read: docs/**\n- write: src/parser/**\n",
    );
}

#[test]
fn char_profile_show_json_shape() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "alice.profile.md",
        "# alice\n\nUpdated.\n\n- model: m2\n\n## Scope\n\n- read: src/**\n- write: src/parser/**\n",
    );
    let r = run(&dir, &["--json", "profile", "show", "alice"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "{\"command\":\"profile.show\",\"conclusion\":\"alice\",\"description\":\"Updated.\",\"model\":\"m2\",\"name\":\"alice\",\"scope.read\":\"src/**\",\"scope.write\":\"src/parser/**\",\"status\":\"ok\"}\n",
    );
}

#[test]
fn char_profile_list_default_and_json() {
    let dir = TempDir::new().unwrap();
    write(&dir, "alice.profile.md", PROFILE_ALICE);
    write(&dir, "bob.profile.md", PROFILE_BOB);

    let r = run(&dir, &["profile", "list", "."]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok profile.list 2 profiles\n---\nalice.profile.md: alice (m1)\nbob.profile.md: bob (m2)\n",
    );

    let r = run(&dir, &["--json", "profile", "list", "."]);
    gold(
        &r.stdout,
        "{\"command\":\"profile.list\",\"conclusion\":\"2 profiles\",\"profiles\":[{\"model\":\"m1\",\"name\":\"alice\",\"path\":\"alice.profile.md\"},{\"model\":\"m2\",\"name\":\"bob\",\"path\":\"bob.profile.md\"}],\"status\":\"ok\"}\n",
    );
}

// ===========================================================================
// validate — four formats ok / suspected-header warning / legacy contacts
// ===========================================================================

#[test]
fn char_validate_four_formats_ok() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    write(&dir, "alice.profile.md", PROFILE_ALICE);
    write(&dir, "guide.brief.md", GUIDE);
    write(
        &dir,
        "team.contacts.md",
        "# Contacts\n\n- [alice](alice.profile.md)\n",
    );
    for f in [
        "chat.post.md",
        "alice.profile.md",
        "guide.brief.md",
        "team.contacts.md",
    ] {
        let r = run(&dir, &["validate", f]);
        assert_eq!(r.code, 0, "validate {f}");
        gold(&r.stdout, &format!("ok validate {f}\n"));
        gold(&r.stderr, "");
    }
}

/// Valid thread + one flush-left line that LOOKS like a message header but
/// violates the strict grammar (double space, malformed tail) — line 9.
const WARN_POST: &str =
    "# T\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nhi\n```\n\n##  #2 alice oops\n";

#[test]
fn char_validate_suspected_header_warning_default_and_json() {
    let dir = TempDir::new().unwrap();
    write(&dir, "warn.post.md", WARN_POST);

    // warning rides in the body; conclusion stays ok, exit 0
    let r = run(&dir, &["validate", "warn.post.md"]);
    assert_eq!(r.code, 0);
    gold(
        &r.stdout,
        "ok validate warn.post.md\n---\nwarning: line 9: suspected message header: ##  #2 alice oops\nfix: expected format: ## #<seq> <sender> (<timestamp>)\nexample: ## #1 alice (2026-01-15T10:30:00Z)\n",
    );

    let r = run(&dir, &["--json", "validate", "warn.post.md"]);
    gold(
        &r.stdout,
        "{\"body\":[\"warning: line 9: suspected message header: ##  #2 alice oops\",\"fix: expected format: ## #<seq> <sender> (<timestamp>)\",\"example: ## #1 alice (2026-01-15T10:30:00Z)\"],\"command\":\"validate\",\"conclusion\":\"warn.post.md\",\"status\":\"ok\"}\n",
    );
}

#[test]
fn char_validate_legacy_contacts_format_error() {
    let dir = TempDir::new().unwrap();
    write(&dir, "legacy.contacts.md", LEGACY_CONTACTS);
    let r = run(&dir, &["validate", "legacy.contacts.md"]);
    assert_eq!(r.code, 1);
    gold(&r.stdout, "");
    gold(
        &r.stderr,
        "error format: Parse error: contacts file contains legacy bare-path bullets but no link bullets\nfix: this file is in the v0.4 legacy format; migrate it by hand per the CHANGELOG migration guide: wrap each path in a Markdown link bullet '- [label](path)'\nexample: - [alice](agents/alice.profile.md)\n",
    );
}

// ===========================================================================
// output.rs — ok/err JSON envelope shapes + global flag conflict
// ===========================================================================

#[test]
fn char_output_err_json_shape() {
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["--json", "post", "read", "ghost"]);
    assert_eq!(r.code, 1);
    gold(&r.stderr, "");
    gold(
        &r.stdout,
        "{\"category\":\"not-found\",\"example\":\"paperwork post send ghost.post.md --from <name> <body>\",\"exit_code\":1,\"fix\":\"send a message first to create the thread\",\"message\":\"Thread 'ghost.post.md' not found\",\"status\":\"error\"}\n",
    );
}

#[test]
fn char_json_plain_conflict_is_clap_exit_2() {
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["--json", "--plain", "post", "read", "chat"]);
    assert_eq!(r.code, 2, "clap usage error must exit 2");
    gold(&r.stdout, "");
    assert!(
        r.stderr.contains("--json"),
        "stderr names --json: {}",
        r.stderr
    );
    assert!(
        r.stderr.contains("--plain"),
        "stderr names --plain: {}",
        r.stderr
    );
}
