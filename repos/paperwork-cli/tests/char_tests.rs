//! P-5 behavior lock — characterization tests (CLI face), re-frozen on the
//! v0.6 named-flag grammar (merged master @ 3829fd9 lineage).
//!
//! Golden-envelope snapshots of the whole command surface: stdout/stderr
//! byte-exact (LF-only) plus exit codes. Every literal in `frozen()` is the
//! frozen v0.6 output contract; the gate protects the P-2..P-7 refactor
//! batches against byte drift.
//!
//! Determinism strategy (carried over from the wip-era T1 suite):
//! - every command runs with `current_dir` = a fresh tempdir and takes
//!   RELATIVE paths, so no machine-specific absolute path ever leaks into
//!   an envelope;
//! - read/summary/validate fixtures are hand-written files with fixed
//!   timestamps (never produced by the clock);
//! - where a produced FILE must be pinned, the wall-clock timestamp is
//!   masked to `(TS)` (message headers and brief `- created:` lines);
//! - Rust stdout is byte-level `\n` (no CRLF translation) — each assertion
//!   also checks the capture carries no `\r`.
//!
//! Recording mode: with `PAPERWORK_CHAR_RECORD=1` every `gold` call appends
//! `("label", <escaped actual>)` to `_char_record.txt` instead of asserting
//! — used once to regenerate the freeze table. The committed table below is
//! the normative contract; record mode is a maintenance tool only.

use std::collections::HashMap;
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

static RECORD: LazyLock<bool> = LazyLock::new(|| std::env::var("PAPERWORK_CHAR_RECORD").is_ok());

/// Mask the OS-localized part of io error envelopes.
///
/// `PaperworkError::IoContext` Display embeds `std::io::Error`'s message,
/// which is platform- AND locale-dependent (`系统找不到指定的文件。 (os error 2)`
/// on zh-CN Windows, `No such file or directory (os error 2)` on Linux).
/// The CI matrix spans three OSes, so the localized tail is collapsed to a
/// deterministic token; our own wording (path, fix, example) stays frozen
/// byte-exact.
fn mask_os(text: &str) -> String {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r": [^\n]*\(os error \d+\)").expect("valid regex"));
    RE.replace_all(text, ": (OS ERROR)").into_owned()
}

/// Byte-exact golden assertion for a stream (or a record entry).
///
/// With `PAPERWORK_CHAR_RECORD=1` entries are written to
/// `_char_record.txt`: a `CHARLEN(<label byte len>)` line, the raw label
/// line, then the `{:?}`-escaped value line — so the splice step can
/// extract entries byte-exactly without console re-encoding. Run record
/// mode single-threaded (`--test-threads=1`) to keep appends ordered.
fn gold(label: &str, actual_raw: &str) {
    assert!(
        !actual_raw.contains('\r'),
        "CR byte leaked into captured output [{label}]: {actual_raw:?}"
    );
    let actual_owned = mask_os(actual_raw);
    let actual = actual_owned.as_str();
    if *RECORD {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("_char_record.txt")
            .expect("open record file");
        writeln!(f, "CHARLEN({})", label.len()).expect("write len");
        f.write_all(label.as_bytes()).expect("write label");
        f.write_all(b"\n").expect("write nl");
        writeln!(f, "{:?}", actual).expect("write value");
        return;
    }
    let expected = frozen()
        .get(label)
        .unwrap_or_else(|| panic!("missing freeze entry for label '{label}'"));
    assert_eq!(actual, *expected, "golden snapshot mismatch [{label}]");
}

fn write(dir: &TempDir, rel: &str, content: &str) {
    if let Some(parent) = std::path::Path::new(rel).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(dir.path().join(parent)).expect("mkdir fixture dir");
        }
    }
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

/// Foreign (non-managed) Markdown shape.
const FOREIGN: &str = "## Just a heading\n\n- a bullet\n";

fn setup_guide(dir: &TempDir) {
    write(dir, "guide.brief.md", GUIDE);
    write(dir, "main.rs", "fn main() {}\n");
    write(dir, "lib.rs", "pub fn lib() {}\n");
}

// ===========================================================================
// post send
// ===========================================================================

#[test]
fn char_post_send_modes() {
    // default mode
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--message",
            "Hello world",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_send_default_stdout", &r.stdout);
    gold("post_send_default_stderr", &r.stderr);
    gold(
        "post_send_default_file",
        &mask_ts(&read_file(&dir, "chat.post.md")),
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
            "--author",
            "alice",
            "--message",
            "Hello world",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_send_json_stdout", &r.stdout);
    gold("post_send_json_stderr", &r.stderr);

    // quiet mode suppresses only the status line
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "--quiet",
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--message",
            "Hello world",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_send_quiet_stdout", &r.stdout);

    // plain mode on a write verb
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "--plain",
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--message",
            "Hello world",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_send_plain_stdout", &r.stdout);
}

#[test]
fn char_post_send_seq_increments_and_title_flag() {
    let dir = TempDir::new().unwrap();
    run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--message",
            "one",
        ],
    );
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "bob",
            "--message",
            "two",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_send_seq2_stdout", &r.stdout);

    // --title overrides the preamble on first write
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--title",
            "My Thread",
            "--message",
            "Hello world",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        "post_send_title_file",
        &mask_ts(&read_file(&dir, "chat.post.md")),
    );
}

#[test]
fn char_post_send_stdin_body() {
    let dir = TempDir::new().unwrap();
    let r = run_stdin(
        &dir,
        &["post", "send", "chat", "--author", "alice", "--stdin"],
        "line one\nline two\n",
    );
    assert_eq!(r.code, 0);
    gold("post_send_stdin_stdout", &r.stdout);
    gold(
        "post_send_stdin_file",
        &mask_ts(&read_file(&dir, "chat.post.md")),
    );
}

#[test]
fn char_post_send_reply_to_implicit_mention() {
    // seed: alice #1, then bob replies to #1 -> implicit mention of alice
    let dir = TempDir::new().unwrap();
    run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--message",
            "opening",
        ],
    );
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "bob",
            "--reply-to",
            "1",
            "--message",
            "the reply",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_send_implicit_mention_stdout", &r.stdout);
    gold("post_send_implicit_mention_json_stdout", &{
        let dir2 = TempDir::new().unwrap();
        run(
            &dir2,
            &[
                "post",
                "send",
                "chat",
                "--author",
                "alice",
                "--message",
                "opening",
            ],
        );
        run(
            &dir2,
            &[
                "--json",
                "post",
                "send",
                "chat",
                "--author",
                "bob",
                "--reply-to",
                "1",
                "--message",
                "the reply",
            ],
        )
        .stdout
    });
    gold(
        "post_send_implicit_mention_file",
        &mask_ts(&read_file(&dir, "chat.post.md")),
    );
}

// NEW-12 (ported from wip, v0.6 grammar): reply-to pointing at a missing
// seq keeps the historical envelope — the send succeeds, the `@#N` token is
// injected, and NO implicit mention appears (the bounded tail-scan lookup
// returns None exactly like the old whole-file read returned an empty
// filter).
#[test]
fn char_post_send_reply_to_missing_seq_envelope_unchanged() {
    let dir = TempDir::new().unwrap();
    run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--message",
            "start",
        ],
    );
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "bob",
            "--reply-to",
            "5",
            "--message",
            "ping",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_send_reply_missing_seq_stdout", &r.stdout);
    gold("post_send_reply_missing_seq_stderr", &r.stderr);
    gold(
        "post_send_reply_missing_seq_file",
        &mask_ts(&read_file(&dir, "chat.post.md")),
    );
}

#[test]
fn char_post_send_mention_flag_injection() {
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--mention",
            "bob,carol",
            "--message",
            "heads up",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_send_mention_stdout", &r.stdout);
    gold(
        "post_send_mention_file",
        &mask_ts(&read_file(&dir, "chat.post.md")),
    );
}

#[test]
fn char_post_send_errors() {
    // legacy v0.4 thread: write guard refuses (format envelope)
    let dir = TempDir::new().unwrap();
    write(&dir, "old.post.md", LEGACY_POST);
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "old.post.md",
            "--author",
            "alice",
            "--message",
            "x",
        ],
    );
    assert_eq!(r.code, 1);
    gold("post_send_legacy_stderr", &r.stderr);
    gold("post_send_legacy_json_stdout", &{
        let dir2 = TempDir::new().unwrap();
        write(&dir2, "old.post.md", LEGACY_POST);
        run(
            &dir2,
            &[
                "--json",
                "post",
                "send",
                "old.post.md",
                "--author",
                "alice",
                "--message",
                "x",
            ],
        )
        .stdout
    });

    // foreign file format
    let dir = TempDir::new().unwrap();
    write(&dir, "foreign.post.md", FOREIGN);
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "foreign.post.md",
            "--author",
            "alice",
            "--message",
            "x",
        ],
    );
    assert_eq!(r.code, 1);
    gold("post_send_foreign_stderr", &r.stderr);

    // invalid sender characters (validation envelope)
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "bad name!",
            "--message",
            "x",
        ],
    );
    assert_eq!(r.code, 1);
    gold("post_send_bad_author_stderr", &r.stderr);
}

// ===========================================================================
// post edit
// ===========================================================================

#[test]
fn char_post_edit_modes() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(
        &dir,
        &[
            "post",
            "edit",
            "chat.post.md",
            "--author",
            "alice",
            "--seq",
            "3",
            "--message",
            "edited closing",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_edit_default_stdout", &r.stdout);
    gold("post_edit_default_stderr", &r.stderr);

    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(
        &dir,
        &[
            "--json",
            "post",
            "edit",
            "chat.post.md",
            "--author",
            "alice",
            "--seq",
            "3",
            "--message",
            "edited closing",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_edit_json_stdout", &r.stdout);

    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(
        &dir,
        &[
            "--quiet",
            "post",
            "edit",
            "chat.post.md",
            "--author",
            "alice",
            "--seq",
            "3",
            "--message",
            "edited closing",
        ],
    );
    assert_eq!(r.code, 0);
    gold("post_edit_quiet_stdout", &r.stdout);
}

#[test]
fn char_post_edit_errors() {
    // wrong owner
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(
        &dir,
        &[
            "post",
            "edit",
            "chat.post.md",
            "--author",
            "bob",
            "--seq",
            "3",
            "--message",
            "x",
        ],
    );
    assert_eq!(r.code, 1);
    gold("post_edit_wrong_owner_stderr", &r.stderr);

    // not the sender's most recent message
    let dir = TempDir::new().unwrap();
    write(&dir, "mixed.post.md", MIXED);
    let r = run(
        &dir,
        &[
            "post",
            "edit",
            "mixed.post.md",
            "--author",
            "alice",
            "--seq",
            "1",
            "--message",
            "x",
        ],
    );
    assert_eq!(r.code, 1);
    gold("post_edit_not_recent_stderr", &r.stderr);

    // not the final message in the thread
    let dir = TempDir::new().unwrap();
    write(&dir, "mixed.post.md", MIXED);
    let r = run(
        &dir,
        &[
            "post",
            "edit",
            "mixed.post.md",
            "--author",
            "bob",
            "--seq",
            "2",
            "--message",
            "x",
        ],
    );
    assert_eq!(r.code, 1);
    gold("post_edit_not_final_stderr", &r.stderr);

    // unknown seq
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(
        &dir,
        &[
            "post",
            "edit",
            "chat.post.md",
            "--author",
            "alice",
            "--seq",
            "99",
            "--message",
            "x",
        ],
    );
    assert_eq!(r.code, 1);
    gold("post_edit_unknown_seq_stderr", &r.stderr);

    // missing file
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "post",
            "edit",
            "absent.post.md",
            "--author",
            "alice",
            "--seq",
            "1",
            "--message",
            "x",
        ],
    );
    assert_eq!(r.code, 1);
    gold("post_edit_missing_stderr", &r.stderr);
}

// ===========================================================================
// post read / summary
// ===========================================================================

#[test]
fn char_post_read_modes() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(&dir, &["post", "read", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("post_read_default_stdout", &r.stdout);
    gold("post_read_default_stderr", &r.stderr);

    let r = run(&dir, &["--json", "post", "read", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("post_read_json_stdout", &r.stdout);

    let r = run(&dir, &["--plain", "post", "read", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("post_read_plain_stdout", &r.stdout);

    let r = run(&dir, &["--quiet", "post", "read", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("post_read_quiet_stdout", &r.stdout);
}

#[test]
fn char_post_read_filters_window() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    // window smaller than the message count -> showing + window fields
    let r = run(&dir, &["post", "read", "chat.post.md", "--limit", "2"]);
    assert_eq!(r.code, 0);
    gold("post_read_window_default_stdout", &r.stdout);
    let r = run(
        &dir,
        &["--json", "post", "read", "chat.post.md", "--limit", "2"],
    );
    assert_eq!(r.code, 0);
    gold("post_read_window_json_stdout", &r.stdout);

    // --from/--to range
    let r = run(
        &dir,
        &["post", "read", "chat.post.md", "--from", "2", "--to", "3"],
    );
    assert_eq!(r.code, 0);
    gold("post_read_range_stdout", &r.stdout);

    // --mention filter
    let r = run(&dir, &["post", "read", "chat.post.md", "--mention", "bob"]);
    assert_eq!(r.code, 0);
    gold("post_read_mention_stdout", &r.stdout);

    // --reply-to filter
    let r = run(&dir, &["post", "read", "chat.post.md", "--reply-to", "1"]);
    assert_eq!(r.code, 0);
    gold("post_read_replyto_stdout", &r.stdout);
}

#[test]
fn char_post_read_errors() {
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["post", "read", "absent.post.md"]);
    assert_eq!(r.code, 1);
    gold("post_read_missing_stderr", &r.stderr);

    let dir = TempDir::new().unwrap();
    write(&dir, "foreign.post.md", FOREIGN);
    let r = run(&dir, &["post", "read", "foreign.post.md"]);
    assert_eq!(r.code, 1);
    gold("post_read_foreign_stderr", &r.stderr);
}

#[test]
fn char_post_summary_modes() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(&dir, &["post", "summary", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("post_summary_default_stdout", &r.stdout);
    gold("post_summary_default_stderr", &r.stderr);

    let r = run(&dir, &["--json", "post", "summary", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("post_summary_json_stdout", &r.stdout);

    let r = run(&dir, &["--plain", "post", "summary", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("post_summary_plain_stdout", &r.stdout);

    let r = run(&dir, &["--quiet", "post", "summary", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("post_summary_quiet_stdout", &r.stdout);

    // missing file -> not-found error envelope
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["post", "summary", "absent.post.md"]);
    assert_eq!(r.code, 1);
    gold("post_summary_missing_stderr", &r.stderr);
}

#[test]
fn char_post_summary_foreign() {
    let dir = TempDir::new().unwrap();
    write(&dir, "foreign.post.md", FOREIGN);
    let r = run(&dir, &["post", "summary", "foreign.post.md"]);
    assert_eq!(r.code, 1);
    gold("post_summary_foreign_stderr", &r.stderr);
}

// ===========================================================================
// profile
// ===========================================================================

#[test]
fn char_profile_create_modes() {
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
            "gpt-4o",
            "--description",
            "Parser implementer",
        ],
    );
    assert_eq!(r.code, 0);
    gold("profile_create_default_stdout", &r.stdout);
    gold("profile_create_default_stderr", &r.stderr);
    gold("profile_create_file", &read_file(&dir, "alice.profile.md"));

    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &["--json", "profile", "create", "alice", "--name", "alice"],
    );
    assert_eq!(r.code, 0);
    gold("profile_create_json_stdout", &r.stdout);

    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &["--quiet", "profile", "create", "alice", "--name", "alice"],
    );
    assert_eq!(r.code, 0);
    gold("profile_create_quiet_stdout", &r.stdout);

    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &["--plain", "profile", "create", "alice", "--name", "alice"],
    );
    assert_eq!(r.code, 0);
    gold("profile_create_plain_stdout", &r.stdout);
}

#[test]
fn char_profile_create_scope_and_duplicate() {
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "profile",
            "create",
            "scoped",
            "--name",
            "scoped",
            "--scope-read",
            "src/**",
            "docs/**",
            "--scope-write",
            "src/parser/**",
            "--scope-owns",
            "src/parser/**",
        ],
    );
    assert_eq!(r.code, 0);
    gold(
        "profile_create_scope_file",
        &read_file(&dir, "scoped.profile.md"),
    );

    // duplicate -> already-exists envelope
    let r = run(&dir, &["profile", "create", "scoped", "--name", "other"]);
    assert_eq!(r.code, 1);
    gold("profile_create_duplicate_stderr", &r.stderr);
}

#[test]
fn char_profile_show_modes() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "alice.profile.md",
        "# alice\n\nParser implementer.\n\n- model: gpt-4o\n",
    );
    let r = run(&dir, &["profile", "show", "alice.profile.md"]);
    assert_eq!(r.code, 0);
    gold("profile_show_default_stdout", &r.stdout);
    gold("profile_show_default_stderr", &r.stderr);

    let r = run(&dir, &["--json", "profile", "show", "alice.profile.md"]);
    assert_eq!(r.code, 0);
    gold("profile_show_json_stdout", &r.stdout);

    let r = run(&dir, &["--plain", "profile", "show", "alice.profile.md"]);
    assert_eq!(r.code, 0);
    gold("profile_show_plain_stdout", &r.stdout);

    let r = run(&dir, &["--quiet", "profile", "show", "alice.profile.md"]);
    assert_eq!(r.code, 0);
    gold("profile_show_quiet_stdout", &r.stdout);
}

#[test]
fn char_profile_show_errors() {
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["profile", "show", "absent.profile.md"]);
    assert_eq!(r.code, 1);
    gold("profile_show_missing_stderr", &r.stderr);

    let dir = TempDir::new().unwrap();
    write(&dir, "garbage.profile.md", "no heading at all\n");
    let r = run(&dir, &["profile", "show", "garbage.profile.md"]);
    assert_eq!(r.code, 1);
    gold("profile_show_format_stderr", &r.stderr);
}

#[test]
fn char_profile_edit_modes() {
    let dir = TempDir::new().unwrap();
    write(&dir, "alice.profile.md", "# alice\n\n- model: gpt-4o\n");
    let r = run(
        &dir,
        &[
            "profile",
            "edit",
            "alice.profile.md",
            "--model",
            "claude-4",
            "--description",
            "Now reviews",
        ],
    );
    assert_eq!(r.code, 0);
    gold("profile_edit_default_stdout", &r.stdout);
    gold("profile_edit_file", &read_file(&dir, "alice.profile.md"));

    let dir = TempDir::new().unwrap();
    write(&dir, "alice.profile.md", "# alice\n\n- model: gpt-4o\n");
    let r = run(
        &dir,
        &[
            "--json",
            "profile",
            "edit",
            "alice.profile.md",
            "--model",
            "claude-4",
        ],
    );
    assert_eq!(r.code, 0);
    gold("profile_edit_json_stdout", &r.stdout);

    // missing target
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &["profile", "edit", "absent.profile.md", "--model", "x"],
    );
    assert_eq!(r.code, 1);
    gold("profile_edit_missing_stderr", &r.stderr);
}

#[test]
fn char_profile_list_modes() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "agents/alice.profile.md",
        "# alice\n\n- model: gpt-4o\n",
    );
    write(&dir, "agents/bob.profile.md", "# bob\n\n- model: m3\n");
    let r = run(&dir, &["profile", "list", "agents"]);
    assert_eq!(r.code, 0);
    gold("profile_list_default_stdout", &r.stdout);
    gold("profile_list_default_stderr", &r.stderr);

    let r = run(&dir, &["--json", "profile", "list", "agents"]);
    assert_eq!(r.code, 0);
    gold("profile_list_json_stdout", &r.stdout);

    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["profile", "list", "nowhere"]);
    assert_eq!(r.code, 1);
    gold("profile_list_missing_dir_stderr", &r.stderr);
}

// ===========================================================================
// brief
// ===========================================================================

#[test]
fn char_brief_create_modes() {
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &[
            "brief",
            "create",
            "onboarding",
            "--title",
            "Codebase Onboarding",
            "--owner",
            "alice",
            "--description",
            "Reading list",
        ],
    );
    assert_eq!(r.code, 0);
    gold("brief_create_default_stdout", &r.stdout);
    gold("brief_create_default_stderr", &r.stderr);
    gold(
        "brief_create_file",
        &mask_created(&read_file(&dir, "onboarding.brief.md")),
    );

    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &["--json", "brief", "create", "onboarding", "--title", "T"],
    );
    assert_eq!(r.code, 0);
    gold("brief_create_json_stdout", &r.stdout);

    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &["--quiet", "brief", "create", "onboarding", "--title", "T"],
    );
    assert_eq!(r.code, 0);
    gold("brief_create_quiet_stdout", &r.stdout);
}

#[test]
fn char_brief_add_modes() {
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(
        &dir,
        &[
            "brief",
            "add",
            "new.brief.md",
            "--entry",
            "main.rs",
            "--regex",
            "fn main",
            "--note",
            "Entry point",
        ],
    );
    // brief add to a missing brief: behavior pinned whatever it is
    gold("brief_add_missing_brief_stdout", &r.stdout);
    gold("brief_add_missing_brief_stderr", &r.stderr);
    assert!(r.code == 0 || r.code == 1);

    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(
        &dir,
        &["brief", "add", "guide.brief.md", "--entry", "src/tool.rs"],
    );
    // entry file absent -> hash failure or not-found, pinned
    gold("brief_add_missing_entry_stderr", &r.stderr);
    assert_eq!(r.code, 1);

    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(
        &dir,
        &["brief", "add", "guide.brief.md", "--entry", "extra.rs"],
    );
    assert_eq!(r.code, 1);
    gold("brief_add_entry_not_found_stderr", &r.stderr);

    write(&dir, "extra.rs", "fn extra() {}\n");
    let r = run(
        &dir,
        &[
            "brief",
            "add",
            "guide.brief.md",
            "--entry",
            "extra.rs",
            "--note",
            "Extra module",
        ],
    );
    assert_eq!(r.code, 0);
    gold("brief_add_default_stdout", &r.stdout);
    gold("brief_add_default_stderr", &r.stderr);

    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(
        &dir,
        &[
            "--json",
            "brief",
            "add",
            "guide.brief.md",
            "--entry",
            "second.rs",
        ],
    );
    // second.rs missing -> json error envelope
    assert_eq!(r.code, 1);
    gold("brief_add_json_error_stdout", &r.stdout);
}

#[test]
fn char_brief_remove_modes() {
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(
        &dir,
        &[
            "brief",
            "remove",
            "guide.brief.md",
            "--entry-title",
            "main.rs",
        ],
    );
    assert_eq!(r.code, 0);
    gold("brief_remove_default_stdout", &r.stdout);
    gold("brief_remove_default_stderr", &r.stderr);
    gold("brief_remove_file", &read_file(&dir, "guide.brief.md"));

    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(
        &dir,
        &[
            "--json",
            "brief",
            "remove",
            "guide.brief.md",
            "--entry-title",
            "lib.rs",
        ],
    );
    assert_eq!(r.code, 0);
    gold("brief_remove_json_stdout", &r.stdout);

    // unknown entry title
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(
        &dir,
        &[
            "brief",
            "remove",
            "guide.brief.md",
            "--entry-title",
            "nope.rs",
        ],
    );
    assert_eq!(r.code, 1);
    gold("brief_remove_unknown_stderr", &r.stderr);
}

#[test]
fn char_brief_read_modes() {
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(&dir, &["brief", "read", "guide.brief.md"]);
    assert_eq!(r.code, 0);
    gold("brief_read_default_stdout", &r.stdout);
    gold("brief_read_default_stderr", &r.stderr);

    let r = run(&dir, &["--json", "brief", "read", "guide.brief.md"]);
    assert_eq!(r.code, 0);
    gold("brief_read_json_stdout", &r.stdout);

    let r = run(&dir, &["--plain", "brief", "read", "guide.brief.md"]);
    assert_eq!(r.code, 0);
    gold("brief_read_plain_stdout", &r.stdout);

    let r = run(&dir, &["--quiet", "brief", "read", "guide.brief.md"]);
    assert_eq!(r.code, 0);
    gold("brief_read_quiet_stdout", &r.stdout);

    // v0.6 face: --entry-title single-entry detail
    let r = run(
        &dir,
        &[
            "brief",
            "read",
            "guide.brief.md",
            "--entry-title",
            "main.rs",
        ],
    );
    assert_eq!(r.code, 0);
    gold("brief_read_entry_title_stdout", &r.stdout);

    // --full
    let r = run(&dir, &["brief", "read", "guide.brief.md", "--full"]);
    assert_eq!(r.code, 0);
    gold("brief_read_full_stdout", &r.stdout);

    // missing brief
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["brief", "read", "absent.brief.md"]);
    assert_eq!(r.code, 1);
    gold("brief_read_missing_stderr", &r.stderr);
}

#[test]
fn char_brief_verify_modes() {
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(&dir, &["brief", "verify", "guide.brief.md"]);
    // lib.rs content matches H_LIB but regex zzz finds nothing -> pinned
    gold("brief_verify_default_stdout", &r.stdout);
    gold("brief_verify_default_stderr", &r.stderr);
    assert!(r.code == 0 || r.code == 1);

    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(&dir, &["--json", "brief", "verify", "guide.brief.md"]);
    gold("brief_verify_json_stdout", &r.stdout);

    // hash mismatch: corrupt main.rs after the brief pinned its hash;
    // verify still exits 0 and reports the failure per entry in the envelope
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    write(&dir, "main.rs", "fn changed() {}\n");
    let r = run(&dir, &["brief", "verify", "guide.brief.md"]);
    assert_eq!(r.code, 0);
    gold("brief_verify_hash_mismatch_stdout", &r.stdout);
    gold("brief_verify_hash_mismatch_stderr", &r.stderr);
}

// ===========================================================================
// contacts
// ===========================================================================

#[test]
fn char_contacts_create_modes() {
    let dir = TempDir::new().unwrap();
    let r = run(
        &dir,
        &["contacts", "create", "team", "--title", "Core Team"],
    );
    assert_eq!(r.code, 0);
    gold("contacts_create_default_stdout", &r.stdout);
    gold("contacts_create_default_stderr", &r.stderr);
    gold("contacts_create_file", &read_file(&dir, "team.contacts.md"));

    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["--json", "contacts", "create", "team"]);
    assert_eq!(r.code, 0);
    gold("contacts_create_json_stdout", &r.stdout);

    // duplicate -> already-exists
    let dir = TempDir::new().unwrap();
    run(&dir, &["contacts", "create", "team"]);
    let r = run(&dir, &["contacts", "create", "team"]);
    assert_eq!(r.code, 1);
    gold("contacts_create_duplicate_stderr", &r.stderr);
}

#[test]
fn char_contacts_add_modes() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "agents/alice.profile.md",
        "# alice\n\n- model: gpt-4o\n",
    );
    run(
        &dir,
        &["contacts", "create", "team", "--title", "Core Team"],
    );
    let r = run(
        &dir,
        &[
            "contacts",
            "add",
            "team.contacts.md",
            "--profile",
            "agents/alice.profile.md",
        ],
    );
    assert_eq!(r.code, 0);
    gold("contacts_add_default_stdout", &r.stdout);
    gold("contacts_add_default_stderr", &r.stderr);
    gold("contacts_add_file", &read_file(&dir, "team.contacts.md"));

    let r = run(
        &dir,
        &[
            "--json",
            "contacts",
            "add",
            "team.contacts.md",
            "--profile",
            "agents/bob.profile.md",
        ],
    );
    // the target profile file is NOT required to exist: contacts only
    // records the path (stateless roster semantics) -> success envelope
    assert_eq!(r.code, 0);
    gold("contacts_add_second_json_stdout", &r.stdout);

    // legacy contacts shape: bare path bullets refuse writes (B1 guard)
    let dir = TempDir::new().unwrap();
    write(&dir, "legacy.contacts.md", LEGACY_CONTACTS);
    write(
        &dir,
        "agents/alice.profile.md",
        "# alice\n\n- model: gpt-4o\n",
    );
    let r = run(
        &dir,
        &[
            "contacts",
            "add",
            "legacy.contacts.md",
            "--profile",
            "agents/alice.profile.md",
        ],
    );
    assert_eq!(r.code, 1);
    gold("contacts_add_legacy_stderr", &r.stderr);
}

#[test]
fn char_contacts_remove_update_modes() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "agents/alice.profile.md",
        "# alice\n\n- model: gpt-4o\n",
    );
    write(&dir, "agents/carol.profile.md", "# carol\n\n- model: m3\n");
    run(&dir, &["contacts", "create", "team"]);
    run(
        &dir,
        &[
            "contacts",
            "add",
            "team.contacts.md",
            "--profile",
            "agents/alice.profile.md",
        ],
    );

    // update re-binds the stored path
    let r = run(
        &dir,
        &[
            "contacts",
            "update",
            "team.contacts.md",
            "--profile",
            "agents/alice.profile.md",
            "--new-profile",
            "agents/carol.profile.md",
        ],
    );
    assert_eq!(r.code, 0);
    gold("contacts_update_default_stdout", &r.stdout);
    gold("contacts_update_file", &read_file(&dir, "team.contacts.md"));

    // remove by stored path
    let r = run(
        &dir,
        &[
            "contacts",
            "remove",
            "team.contacts.md",
            "--profile",
            "agents/carol.profile.md",
        ],
    );
    assert_eq!(r.code, 0);
    gold("contacts_remove_default_stdout", &r.stdout);
    gold("contacts_remove_file", &read_file(&dir, "team.contacts.md"));

    // remove of an unknown entry -> pinned error
    let r = run(
        &dir,
        &[
            "contacts",
            "remove",
            "team.contacts.md",
            "--profile",
            "ghost.profile.md",
        ],
    );
    assert_eq!(r.code, 1);
    gold("contacts_remove_unknown_stderr", &r.stderr);

    // update of an unknown entry -> pinned error
    let r = run(
        &dir,
        &[
            "contacts",
            "update",
            "team.contacts.md",
            "--profile",
            "ghost.profile.md",
            "--new-profile",
            "x.profile.md",
        ],
    );
    assert_eq!(r.code, 1);
    gold("contacts_update_unknown_stderr", &r.stderr);
}

#[test]
fn char_contacts_read_modes() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "team.contacts.md",
        "# Core Team\n\n- [alice](agents/alice.profile.md)\n- [bob](agents/bob.profile.md)\n",
    );
    let r = run(&dir, &["contacts", "read", "team.contacts.md"]);
    assert_eq!(r.code, 0);
    gold("contacts_read_default_stdout", &r.stdout);
    gold("contacts_read_default_stderr", &r.stderr);

    let r = run(&dir, &["--json", "contacts", "read", "team.contacts.md"]);
    assert_eq!(r.code, 0);
    gold("contacts_read_json_stdout", &r.stdout);

    let r = run(&dir, &["--plain", "contacts", "read", "team.contacts.md"]);
    assert_eq!(r.code, 0);
    gold("contacts_read_plain_stdout", &r.stdout);

    let r = run(&dir, &["--quiet", "contacts", "read", "team.contacts.md"]);
    assert_eq!(r.code, 0);
    gold("contacts_read_quiet_stdout", &r.stdout);

    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["contacts", "read", "absent.contacts.md"]);
    assert_eq!(r.code, 1);
    gold("contacts_read_missing_stderr", &r.stderr);
}

// ===========================================================================
// validate
// ===========================================================================

#[test]
fn char_validate_modes() {
    let dir = TempDir::new().unwrap();
    write(&dir, "chat.post.md", CHAT);
    let r = run(&dir, &["validate", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("validate_ok_default_stdout", &r.stdout);
    gold("validate_ok_default_stderr", &r.stderr);

    let r = run(&dir, &["--json", "validate", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("validate_ok_json_stdout", &r.stdout);

    let r = run(&dir, &["--quiet", "validate", "chat.post.md"]);
    assert_eq!(r.code, 0);
    gold("validate_ok_quiet_stdout", &r.stdout);

    // unclosed fence
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "broken.post.md",
        "# t\n\n## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nbody never closed\n",
    );
    let r = run(&dir, &["validate", "broken.post.md"]);
    assert_eq!(r.code, 1);
    gold("validate_unclosed_fence_stderr", &r.stderr);

    // type mismatch: brief file parsed as post
    let dir = TempDir::new().unwrap();
    setup_guide(&dir);
    let r = run(&dir, &["validate", "guide.brief.md", "--type", "post"]);
    assert_eq!(r.code, 1);
    gold("validate_type_mismatch_stderr", &r.stderr);

    // unknown suffix -> no inferred type
    let dir = TempDir::new().unwrap();
    write(&dir, "mystery.txt", "anything\n");
    let r = run(&dir, &["validate", "mystery.txt"]);
    assert_eq!(r.code, 1);
    gold("validate_unknown_suffix_stderr", &r.stderr);

    // missing file
    let dir = TempDir::new().unwrap();
    let r = run(&dir, &["validate", "absent.post.md"]);
    assert_eq!(r.code, 1);
    gold("validate_missing_stderr", &r.stderr);
}

// ===========================================================================
// usage envelopes (exit 2)
// ===========================================================================

#[test]
fn char_usage_envelopes() {
    let dir = TempDir::new().unwrap();

    // top-level missing subcommand
    let r = run(&dir, &[]);
    assert_eq!(r.code, 2);
    gold("usage_missing_subcommand_stderr", &r.stderr);

    // group-level missing subcommand
    let r = run(&dir, &["post"]);
    assert_eq!(r.code, 2);
    gold("usage_group_missing_subcommand_stderr", &r.stderr);

    // missing required argument (--author on post send)
    let r = run(&dir, &["post", "send", "chat", "--message", "hi"]);
    assert_eq!(r.code, 2);
    gold("usage_missing_author_stderr", &r.stderr);

    // unknown flag with migration teaching
    let r = run(&dir, &["post", "send", "chat", "--from", "alice"]);
    assert_eq!(r.code, 2);
    gold("usage_unknown_flag_stderr", &r.stderr);

    // post read --author special-case teaching
    let r = run(&dir, &["post", "read", "chat", "--author", "alice"]);
    assert_eq!(r.code, 2);
    gold("usage_post_read_author_stderr", &r.stderr);

    // dash-leading bare token on post send (body teaching)
    let r = run(
        &dir,
        &["post", "send", "chat", "--author", "alice", "-oops"],
    );
    assert_eq!(r.code, 2);
    gold("usage_dash_token_stderr", &r.stderr);

    // bare extra positional value
    let r = run(
        &dir,
        &[
            "post",
            "send",
            "chat",
            "--author",
            "alice",
            "--message",
            "hi",
            "extra",
        ],
    );
    assert_eq!(r.code, 2);
    gold("usage_extra_positional_stderr", &r.stderr);

    // usage envelope in json mode goes to stdout
    let r = run(&dir, &["--json", "post", "send", "chat", "--message", "hi"]);
    assert_eq!(r.code, 2);
    gold("usage_json_stdout", &r.stdout);
    gold("usage_json_stderr", &r.stderr);
}

// ===========================================================================
// Freeze table (generated by record mode; normative v0.6 contract)
// ===========================================================================

static FROZEN: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let entries: &[(&str, &str)] = &[
        // <FREEZE-BEGIN>
        ("brief_add_default_stderr", ""),
        ("brief_add_default_stdout", "ok brief.add extra.rs -> guide.brief.md\nbrief: guide.brief.md\nentry: extra.rs\n"),
        ("brief_add_entry_not_found_stderr", "error io: (OS ERROR)\nfix: check that the file exists and is readable\nexample: paperwork brief add onboarding.brief.md --entry src/main.rs\n"),
        ("brief_add_json_error_stdout", "{\"category\":\"io\",\"command\":\"brief.add\",\"example\":\"paperwork brief add onboarding.brief.md --entry src/main.rs\",\"exit_code\":1,\"fix\":\"check that the file exists and is readable\",\"message\":\"IO error at 'second.rs': (OS ERROR)\",\"status\":\"error\"}\n"),
        ("brief_add_missing_brief_stderr", "error not-found: Brief 'new.brief.md' not found\nfix: run `paperwork brief create new.brief.md --title \"My Brief\"` first\nexample: paperwork brief create new.brief.md --title \"My Brief\"\n"),
        ("brief_add_missing_brief_stdout", ""),
        ("brief_add_missing_entry_stderr", "error io: (OS ERROR)\nfix: check that the file exists and is readable\nexample: paperwork brief add onboarding.brief.md --entry src/main.rs\n"),
        ("brief_create_default_stderr", ""),
        ("brief_create_default_stdout", "ok brief.create onboarding.brief.md\npath: onboarding.brief.md\ntitle: Codebase Onboarding\n"),
        ("brief_create_file", "# Codebase Onboarding\n\nReading list\n\n- owner: alice\n- created: (TS)\n"),
        ("brief_create_json_stdout", "{\"command\":\"brief.create\",\"conclusion\":\"onboarding.brief.md\",\"path\":\"onboarding.brief.md\",\"status\":\"ok\",\"title\":\"T\"}\n"),
        ("brief_create_quiet_stdout", "path: onboarding.brief.md\ntitle: T\n"),
        ("brief_read_default_stderr", ""),
        ("brief_read_default_stdout", "ok brief.read 2 entries\ntitle: Guide Brief\nowner: alice\n---\nmain.rs: main.rs\nlib.rs: lib.rs\n"),
        ("brief_read_entry_title_stdout", "ok brief.read 2 entries\ntitle: Guide Brief\nowner: alice\n---\nmain.rs: main.rs (hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4) regex: fn main note: Entry point of the tool.\n"),
        ("brief_read_full_stdout", "ok brief.read 2 entries\ntitle: Guide Brief\nowner: alice\n---\nmain.rs: main.rs (hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4) regex: fn main note: Entry point of the tool.\nlib.rs: lib.rs (hash: 0db36e0c52631e298f490f390d09308f9db27b8310b0aed0ad6650714455a69e) regex: zzz\n"),
        ("brief_read_json_stdout", "{\"command\":\"brief.read\",\"conclusion\":\"2 entries\",\"entries\":[{\"hash\":\"536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4\",\"path\":\"main.rs\",\"title\":\"main.rs\"},{\"hash\":\"0db36e0c52631e298f490f390d09308f9db27b8310b0aed0ad6650714455a69e\",\"path\":\"lib.rs\",\"title\":\"lib.rs\"}],\"owner\":\"alice\",\"status\":\"ok\",\"title\":\"Guide Brief\"}\n"),
        ("brief_read_missing_stderr", "error not-found: Brief 'absent.brief.md' not found\nfix: run `paperwork brief create absent.brief.md --title \"My Brief\"` first\nexample: paperwork brief create absent.brief.md --title \"My Brief\"\n"),
        ("brief_read_plain_stdout", "# Guide Brief\n\nReading list for the module.\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n## main.rs\n\n- path: main.rs\n- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4\n- regex: fn main\n\nEntry point of the tool.\n\n## lib.rs\n\n- path: lib.rs\n- hash: 0db36e0c52631e298f490f390d09308f9db27b8310b0aed0ad6650714455a69e\n- regex: zzz\n"),
        ("brief_read_quiet_stdout", "title: Guide Brief\nowner: alice\n---\nmain.rs: main.rs\nlib.rs: lib.rs\n"),
        ("brief_remove_default_stderr", ""),
        ("brief_remove_default_stdout", "ok brief.remove main.rs\nbrief: guide.brief.md\nremoved: main.rs\n"),
        ("brief_remove_file", "# Guide Brief\n\nReading list for the module.\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n## lib.rs\n\n- path: lib.rs\n- hash: 0db36e0c52631e298f490f390d09308f9db27b8310b0aed0ad6650714455a69e\n- regex: zzz\n"),
        ("brief_remove_json_stdout", "{\"brief\":\"guide.brief.md\",\"command\":\"brief.remove\",\"conclusion\":\"lib.rs\",\"removed\":\"lib.rs\",\"status\":\"ok\"}\n"),
        ("brief_remove_unknown_stderr", "error not-found: Brief entry 'nope.rs' not found\nfix: run `paperwork brief read guide.brief.md` to see available entries\nexample: paperwork brief read guide.brief.md\n"),
        ("brief_verify_default_stderr", ""),
        ("brief_verify_default_stdout", "ok brief.verify 1/2 fresh\n---\nmain.rs: fresh\nlib.rs: stale\n"),
        ("brief_verify_hash_mismatch_stderr", ""),
        ("brief_verify_hash_mismatch_stdout", "ok brief.verify 0/2 fresh\n---\nmain.rs: stale\nlib.rs: stale\n"),
        ("brief_verify_json_stdout", "{\"command\":\"brief.verify\",\"conclusion\":\"1/2 fresh\",\"results\":[{\"path\":\"main.rs\",\"status\":\"fresh\",\"title\":\"main.rs\"},{\"path\":\"lib.rs\",\"status\":\"stale\",\"title\":\"lib.rs\"}],\"status\":\"ok\"}\n"),
        ("contacts_add_default_stderr", ""),
        ("contacts_add_default_stdout", "ok contacts.add agents/alice.profile.md -> team.contacts.md\ncontacts: team.contacts.md\nprofile: agents/alice.profile.md\n"),
        ("contacts_add_file", "# Core Team\n\n- [alice](agents/alice.profile.md)\n"),
        ("contacts_add_legacy_stderr", "error format: Parse error: contacts file contains legacy bare-path bullets that v0.5 parsing ignores\nfix: this file is in the v0.4 legacy format; v0.5 is not forward compatible - migrate it by hand per the CHANGELOG migration guide before adding entries\nexample: see CHANGELOG.md, [0.5.0] 'Migration guide (manual)', contacts\n"),
        ("contacts_add_second_json_stdout", "{\"command\":\"contacts.add\",\"conclusion\":\"agents/bob.profile.md -> team.contacts.md\",\"contacts\":\"team.contacts.md\",\"profile\":\"agents/bob.profile.md\",\"status\":\"ok\"}\n"),
        ("contacts_create_default_stderr", ""),
        ("contacts_create_default_stdout", "ok contacts.create team.contacts.md\npath: team.contacts.md\ntitle: Core Team\n"),
        ("contacts_create_duplicate_stderr", "error already-exists: Contacts 'team.contacts.md' already exists\nfix: use `paperwork contacts add` to add entries\nexample: paperwork contacts add team.contacts.md --profile agents/alice.profile.md\n"),
        ("contacts_create_file", "# Core Team\n\n"),
        ("contacts_create_json_stdout", "{\"command\":\"contacts.create\",\"conclusion\":\"team.contacts.md\",\"path\":\"team.contacts.md\",\"status\":\"ok\",\"title\":\"Contacts\"}\n"),
        ("contacts_read_default_stderr", ""),
        ("contacts_read_default_stdout", "ok contacts.read 2 contacts\n---\nagents/alice.profile.md: (unreadable)\nagents/bob.profile.md: (unreadable)\n"),
        ("contacts_read_json_stdout", "{\"command\":\"contacts.read\",\"conclusion\":\"2 contacts\",\"contacts\":[{\"description\":\"\",\"label\":\"alice\",\"name\":\"(unreadable)\",\"path\":\"agents/alice.profile.md\"},{\"description\":\"\",\"label\":\"bob\",\"name\":\"(unreadable)\",\"path\":\"agents/bob.profile.md\"}],\"status\":\"ok\"}\n"),
        ("contacts_read_missing_stderr", "error not-found: Contacts 'absent.contacts.md' not found\nfix: run `paperwork contacts create absent.contacts.md` first\nexample: paperwork contacts create absent.contacts.md\n"),
        ("contacts_read_plain_stdout", "# Core Team\n\n- [alice](agents/alice.profile.md)\n- [bob](agents/bob.profile.md)\n"),
        ("contacts_read_quiet_stdout", "---\nagents/alice.profile.md: (unreadable)\nagents/bob.profile.md: (unreadable)\n"),
        ("contacts_remove_default_stdout", "ok contacts.remove agents/carol.profile.md -> team.contacts.md\ncontacts: team.contacts.md\nremoved: agents/carol.profile.md\n"),
        ("contacts_remove_file", "# Contacts\n\n"),
        ("contacts_remove_unknown_stderr", "error not-found: Contacts entry 'ghost.profile.md' not found\nfix: run `paperwork contacts read team.contacts.md` to list entries; the key is the profile path as stored in the contacts file, not the label\nexample: paperwork contacts read team.contacts.md\n"),
        ("contacts_update_default_stdout", "ok contacts.update agents/alice.profile.md -> agents/carol.profile.md\ncontacts: team.contacts.md\nupdated: agents/alice.profile.md -> agents/carol.profile.md\n"),
        ("contacts_update_file", "# Contacts\n\n- [carol](agents/carol.profile.md)\n"),
        ("contacts_update_unknown_stderr", "error not-found: Contacts entry 'ghost.profile.md' not found\nfix: run `paperwork contacts read team.contacts.md` to list entries; the key is the profile path as stored in the contacts file, not the label\nexample: paperwork contacts read team.contacts.md\n"),
        ("post_edit_default_stderr", ""),
        ("post_edit_default_stdout", "ok post.edit #3\nseq: 3\npath: chat.post.md\n"),
        ("post_edit_json_stdout", "{\"command\":\"post.edit\",\"conclusion\":\"#3\",\"path\":\"chat.post.md\",\"seq\":\"3\",\"status\":\"ok\"}\n"),
        ("post_edit_missing_stderr", "error not-found: Thread 'absent.post.md' not found\nfix: cannot edit a non-existent thread\nexample: paperwork post send absent.post.md --author alice --message \"Hello\"\n"),
        ("post_edit_not_final_stderr", "error not-allowed: thread_edit: Message #2 is not your most recent message (your last is #4)\nfix: you can only edit your most recent message\nexample: paperwork post edit mixed.post.md --author bob --seq 4 --message \"corrected body\"\n"),
        ("post_edit_not_recent_stderr", "error not-allowed: thread_edit: Message #1 is not your most recent message (your last is #3)\nfix: you can only edit your most recent message\nexample: paperwork post edit mixed.post.md --author alice --seq 3 --message \"corrected body\"\n"),
        ("post_edit_quiet_stdout", "seq: 3\npath: chat.post.md\n"),
        ("post_edit_unknown_seq_stderr", "error not-found: Message '#99' not found\nfix: check the seq number with `paperwork post read`\nexample: paperwork post read chat.post.md\n"),
        ("post_edit_wrong_owner_stderr", "error not-allowed: thread_edit: Message #3 was sent by 'alice', not 'bob'\nfix: you can only edit your own messages\nexample: paperwork post edit chat.post.md --author alice --seq 3 --message \"corrected body\"\n"),
        ("post_read_default_stderr", ""),
        ("post_read_default_stdout", "ok post.read 3 messages\nshowing: 3/3\nwindow: #1-#3\n---\n#1 alice 2026-01-15T10:30:00Z mentions:bob\n  ping @bob about the parser\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n#3 alice 2026-01-15T10:32:00Z\n  closing out the thread\n"),
        ("post_read_foreign_stderr", "error format: Parse error: foreign.post.md is not a valid post thread: no valid message boundaries found\nfix: expected an H1 title preamble with `## #N sender timestamp` message headers; or validate it explicitly\nexample: paperwork validate foreign.post.md --type post\n"),
        ("post_read_json_stdout", "{\"command\":\"post.read\",\"conclusion\":\"3 messages\",\"messages\":[{\"body\":\"ping @bob about the parser\",\"mentions\":[\"bob\"],\"reply_to\":null,\"sender\":\"alice\",\"seq\":1,\"timestamp\":\"2026-01-15T10:30:00Z\"},{\"body\":\"@alice @#1 merged the fix\",\"mentions\":[\"alice\"],\"reply_to\":1,\"sender\":\"bob\",\"seq\":2,\"timestamp\":\"2026-01-15T10:31:00Z\"},{\"body\":\"closing out the thread\",\"mentions\":[],\"reply_to\":null,\"sender\":\"alice\",\"seq\":3,\"timestamp\":\"2026-01-15T10:32:00Z\"}],\"showing\":\"3/3\",\"status\":\"ok\",\"window\":\"#1-#3\"}\n"),
        ("post_read_mention_stdout", "ok post.read 1 messages\nshowing: 1/1\nwindow: #1-#1\n---\n#1 alice 2026-01-15T10:30:00Z mentions:bob\n  ping @bob about the parser\n"),
        ("post_read_missing_stderr", "error not-found: Thread 'absent.post.md' not found\nfix: send a message first to create the thread\nexample: paperwork post send absent.post.md --author alice --message \"Hello\"\n"),
        ("post_read_plain_stdout", "## #1 alice (2026-01-15T10:30:00Z)\n\n```md\nping @bob about the parser\n```\n\n## #2 bob (2026-01-15T10:31:00Z)\n\n```md\n@alice @#1 merged the fix\n```\n\n## #3 alice (2026-01-15T10:32:00Z)\n\n```md\nclosing out the thread\n```\n\n"),
        ("post_read_quiet_stdout", "showing: 3/3\nwindow: #1-#3\n---\n#1 alice 2026-01-15T10:30:00Z mentions:bob\n  ping @bob about the parser\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n#3 alice 2026-01-15T10:32:00Z\n  closing out the thread\n"),
        ("post_read_range_stdout", "ok post.read 2 messages\nshowing: 2/2\nwindow: #2-#3\n---\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n#3 alice 2026-01-15T10:32:00Z\n  closing out the thread\n"),
        ("post_read_replyto_stdout", "ok post.read 1 messages\nshowing: 1/1\nwindow: #2-#2\n---\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n"),
        ("post_read_window_default_stdout", "ok post.read 3 messages\nshowing: 2/3\nwindow: #2-#3\n---\n#2 bob 2026-01-15T10:31:00Z reply:#1 mentions:alice\n  @alice @#1 merged the fix\n#3 alice 2026-01-15T10:32:00Z\n  closing out the thread\n"),
        ("post_read_window_json_stdout", "{\"command\":\"post.read\",\"conclusion\":\"3 messages\",\"messages\":[{\"body\":\"@alice @#1 merged the fix\",\"mentions\":[\"alice\"],\"reply_to\":1,\"sender\":\"bob\",\"seq\":2,\"timestamp\":\"2026-01-15T10:31:00Z\"},{\"body\":\"closing out the thread\",\"mentions\":[],\"reply_to\":null,\"sender\":\"alice\",\"seq\":3,\"timestamp\":\"2026-01-15T10:32:00Z\"}],\"showing\":\"2/3\",\"status\":\"ok\",\"window\":\"#2-#3\"}\n"),
        ("post_send_bad_author_stderr", "error validation: Validation error: invalid sender 'bad name!': must be a single token without spaces or parentheses\nfix: sender must be a single token without spaces or parentheses\nexample: paperwork post send standup --author alice --message \"Hello\"\n"),
        ("post_send_default_file", "# chat\n\n## #1 alice (TS)\n\n```md\nHello world\n```\n\n"),
        ("post_send_default_stderr", ""),
        ("post_send_default_stdout", "ok post.send #1 -> chat.post.md\nseq: 1\npath: chat.post.md\nsender: alice\n"),
        ("post_send_foreign_stderr", "error format: Parse error: foreign.post.md is not a valid post thread: no valid message boundaries found\nfix: expected an H1 title preamble with `## #N sender timestamp` message headers; or validate it explicitly\nexample: paperwork validate foreign.post.md --type post\n"),
        ("post_send_implicit_mention_file", "# chat\n\n## #1 alice (TS)\n\n```md\nopening\n```\n\n## #2 bob (TS)\n\n```md\n@#1 @alice\n\nthe reply\n```\n\n"),
        ("post_send_implicit_mention_json_stdout", "{\"command\":\"post.send\",\"conclusion\":\"#2 -> chat.post.md\",\"implicit-mention\":\"alice\",\"path\":\"chat.post.md\",\"sender\":\"bob\",\"seq\":\"2\",\"status\":\"ok\"}\n"),
        ("post_send_implicit_mention_stdout", "ok post.send #2 -> chat.post.md\nseq: 2\npath: chat.post.md\nsender: bob\nimplicit-mention: alice\n"),
        ("post_send_json_stderr", ""),
        ("post_send_json_stdout", "{\"command\":\"post.send\",\"conclusion\":\"#1 -> chat.post.md\",\"path\":\"chat.post.md\",\"sender\":\"alice\",\"seq\":\"1\",\"status\":\"ok\"}\n"),
        ("post_send_legacy_json_stdout", "{\"category\":\"format\",\"command\":\"post.send\",\"example\":\"paperwork validate old.post.md --type post\",\"exit_code\":1,\"fix\":\"expected an H1 title preamble with `## #N sender timestamp` message headers; or validate it explicitly\",\"message\":\"Parse error: old.post.md is not a valid post thread: no valid message boundaries found\",\"status\":\"error\"}\n"),
        ("post_send_legacy_stderr", "error format: Parse error: old.post.md is not a valid post thread: no valid message boundaries found\nfix: expected an H1 title preamble with `## #N sender timestamp` message headers; or validate it explicitly\nexample: paperwork validate old.post.md --type post\n"),
        ("post_send_mention_file", "# chat\n\n## #1 alice (TS)\n\n```md\n@bob @carol\n\nheads up\n```\n\n"),
        ("post_send_mention_stdout", "ok post.send #1 -> chat.post.md\nseq: 1\npath: chat.post.md\nsender: alice\n"),
        ("post_send_plain_stdout", ""),
        ("post_send_quiet_stdout", "seq: 1\npath: chat.post.md\nsender: alice\n"),
        ("post_send_reply_missing_seq_file", "# chat\n\n## #1 alice (TS)\n\n```md\nstart\n```\n\n## #2 bob (TS)\n\n```md\n@#5\n\nping\n```\n\n"),
        ("post_send_reply_missing_seq_stderr", ""),
        ("post_send_reply_missing_seq_stdout", "ok post.send #2 -> chat.post.md\nseq: 2\npath: chat.post.md\nsender: bob\n"),
        ("post_send_seq2_stdout", "ok post.send #2 -> chat.post.md\nseq: 2\npath: chat.post.md\nsender: bob\n"),
        ("post_send_stdin_file", "# chat\n\n## #1 alice (TS)\n\n```md\nline one\nline two\n\n```\n\n"),
        ("post_send_stdin_stdout", "ok post.send #1 -> chat.post.md\nseq: 1\npath: chat.post.md\nsender: alice\n"),
        ("post_send_title_file", "# My Thread\n\n## #1 alice (TS)\n\n```md\nHello world\n```\n\n"),
        ("post_summary_default_stderr", ""),
        ("post_summary_default_stdout", "ok post.summary chat.post.md\ntitle: Chat Log\nparticipants: alice, bob\nmessages: 3\nlast.sender: alice\nlast.time: 2026-01-15T10:32:00Z\nlast.snippet: closing out the thread\n"),
        ("post_summary_foreign_stderr", "error format: Parse error: foreign.post.md is not a valid post thread: no valid message boundaries found\nfix: expected an H1 title preamble with `## #N sender timestamp` message headers; or validate it explicitly\nexample: paperwork validate foreign.post.md --type post\n"),
        ("post_summary_json_stdout", "{\"command\":\"post.summary\",\"conclusion\":\"chat.post.md\",\"last.sender\":\"alice\",\"last.snippet\":\"closing out the thread\",\"last.time\":\"2026-01-15T10:32:00Z\",\"messages\":3,\"participants\":\"alice, bob\",\"status\":\"ok\",\"title\":\"Chat Log\"}\n"),
        ("post_summary_missing_stderr", "error not-found: Thread 'absent.post.md' not found\nfix: send a message first to create the thread\nexample: paperwork post send absent.post.md --author alice --message \"Hello\"\n"),
        ("post_summary_plain_stdout", ""),
        ("post_summary_quiet_stdout", "title: Chat Log\nparticipants: alice, bob\nmessages: 3\nlast.sender: alice\nlast.time: 2026-01-15T10:32:00Z\nlast.snippet: closing out the thread\n"),
        ("profile_create_default_stderr", ""),
        ("profile_create_default_stdout", "ok profile.create alice.profile.md\npath: alice.profile.md\nname: alice\n"),
        ("profile_create_duplicate_stderr", "error already-exists: Profile 'scoped.profile.md' already exists\nfix: use `paperwork profile edit` to modify an existing profile\nexample: paperwork profile edit scoped.profile.md --model gpt-4o\n"),
        ("profile_create_file", "# alice\n\nParser implementer\n\n- model: gpt-4o\n"),
        ("profile_create_json_stdout", "{\"command\":\"profile.create\",\"conclusion\":\"alice.profile.md\",\"name\":\"alice\",\"path\":\"alice.profile.md\",\"status\":\"ok\"}\n"),
        ("profile_create_plain_stdout", ""),
        ("profile_create_quiet_stdout", "path: alice.profile.md\nname: alice\n"),
        ("profile_create_scope_file", "# scoped\n\n- model: \n\n## Scope\n\n- read: src/**\n- read: docs/**\n- write: src/parser/**\n- owns: src/parser/**\n"),
        ("profile_edit_default_stdout", "ok profile.edit alice.profile.md\nchanged: model, description\n"),
        ("profile_edit_file", "# alice\n\nNow reviews\n\n- model: claude-4\n"),
        ("profile_edit_json_stdout", "{\"changed\":\"model\",\"command\":\"profile.edit\",\"conclusion\":\"alice.profile.md\",\"status\":\"ok\"}\n"),
        ("profile_edit_missing_stderr", "error not-found: Profile 'absent.profile.md' not found\nfix: run `paperwork profile create absent.profile.md --name alice` first\nexample: paperwork profile create absent.profile.md --name alice\n"),
        ("profile_list_default_stderr", ""),
        ("profile_list_default_stdout", "ok profile.list 2 profiles\n---\nalice.profile.md: alice (gpt-4o)\nbob.profile.md: bob (m3)\n"),
        ("profile_list_json_stdout", "{\"command\":\"profile.list\",\"conclusion\":\"2 profiles\",\"profiles\":[{\"model\":\"gpt-4o\",\"name\":\"alice\",\"path\":\"alice.profile.md\"},{\"model\":\"m3\",\"name\":\"bob\",\"path\":\"bob.profile.md\"}],\"status\":\"ok\"}\n"),
        ("profile_list_missing_dir_stderr", "error not-found: Directory 'nowhere' not found\nfix: provide a valid directory path\nexample: paperwork profile list nowhere\n"),
        ("profile_show_default_stderr", ""),
        ("profile_show_default_stdout", "ok profile.show alice\nname: alice\nmodel: gpt-4o\ndescription: Parser implementer.\n"),
        ("profile_show_format_stderr", "error format: Parse error: missing agent name heading (# <name>)\nfix: add a top-level heading with the agent name\nexample: # alice\n"),
        ("profile_show_json_stdout", "{\"command\":\"profile.show\",\"conclusion\":\"alice\",\"description\":\"Parser implementer.\",\"model\":\"gpt-4o\",\"name\":\"alice\",\"status\":\"ok\"}\n"),
        ("profile_show_missing_stderr", "error not-found: Profile 'absent.profile.md' not found\nfix: run `paperwork profile create absent.profile.md --name alice` first\nexample: paperwork profile create absent.profile.md --name alice\n"),
        ("profile_show_plain_stdout", "# alice\n\nParser implementer.\n\n- model: gpt-4o\n"),
        ("profile_show_quiet_stdout", "name: alice\nmodel: gpt-4o\ndescription: Parser implementer.\n"),
        ("usage_dash_token_stderr", "error usage: unexpected argument '-o' found\nfix: required values are named flags (--author/--message for post send/edit); see the canonical example below; if a body value starts with '-', pass it via --message (e.g. paperwork post send standup.post.md --author alice --message \"-fix flag text\")\nexample: paperwork post send standup.post.md --author alice --message \"Hello\"\n"),
        ("usage_extra_positional_stderr", "error usage: unexpected argument 'extra' found\nfix: required values are named flags (--author/--message for post send/edit); see the canonical example below; values are given via their named flags, not as bare tokens\nexample: paperwork post send standup.post.md --author alice --message \"Hello\"\n"),
        ("usage_group_missing_subcommand_stderr", "error usage: missing subcommand for group 'post'; run 'paperwork post --help' to list its verbs\nfix: required values are named flags (--author/--message for post send/edit); see the canonical example below\nexample: paperwork post send standup.post.md --author alice --message \"Hello\"\n"),
        ("usage_json_stderr", ""),
        ("usage_json_stdout", "{\"category\":\"usage\",\"command\":\"post.send\",\"example\":\"paperwork post send standup.post.md --author alice --message \\\"Hello\\\"\",\"exit_code\":2,\"fix\":\"required values are named flags (--author/--message for post send/edit); see the canonical example below\",\"message\":\"the following required arguments were not provided: --author <AUTHOR>\",\"status\":\"error\"}\n"),
        ("usage_missing_author_stderr", "error usage: the following required arguments were not provided: --author <AUTHOR>\nfix: required values are named flags (--author/--message for post send/edit); see the canonical example below\nexample: paperwork post send standup.post.md --author alice --message \"Hello\"\n"),
        ("usage_missing_subcommand_stderr", "error usage: missing subcommand: expected one of profile, post, brief, contacts, validate\nfix: required values are named flags (--author/--message for post send/edit); see the canonical example below\nexample: paperwork post send standup.post.md --author alice --message \"Hello\"\n"),
        ("usage_post_read_author_stderr", "error usage: unexpected argument '--author' found\nfix: required values are named flags (--author/--message for post send/edit); see the canonical example below; post read has no --author flag; to locate messages by a sender, filter on mentions via --mention (e.g. paperwork post read standup.post.md --mention alice)\nexample: paperwork post read standup.post.md --from 5 --to 20\n"),
        ("usage_unknown_flag_stderr", "error usage: unexpected argument '--from' found\nfix: required values are named flags (--author/--message for post send/edit); see the canonical example below; this flag is not recognized; if it came from older grammar, give the value via the matching named flag\nexample: paperwork post send standup.post.md --author alice --message \"Hello\"\n"),
        ("validate_missing_stderr", "error io: (OS ERROR)\nfix: check that the file exists and is readable\nexample: paperwork validate absent.post.md\n"),
        ("validate_ok_default_stderr", ""),
        ("validate_ok_default_stdout", "ok validate chat.post.md\n"),
        ("validate_ok_json_stdout", "{\"command\":\"validate\",\"conclusion\":\"chat.post.md\",\"status\":\"ok\"}\n"),
        ("validate_ok_quiet_stdout", ""),
        ("validate_type_mismatch_stderr", "error format: Parse error: no valid messages found\nfix: expected '## #<seq> <sender> (<timestamp>)' headers with dynamic md fences\nexample: paperwork post send myfile --author alice --message \"hello\"\n"),
        ("validate_unclosed_fence_stderr", "error format: Parse error: unclosed code fence (3 backticks) opened at line 5\nfix: close every code fence with a backtick-only line at least as long as the opening fence\nexample: paperwork validate standup.post.md --type post\n"),
        ("validate_unknown_suffix_stderr", "error format: Parse error: unknown file type: mystery.txt\nfix: file must end with .post.md/.profile.md/.brief.md/.contacts.md, or pass --type\nexample: paperwork validate myfile.md --type post\n"),
        // <FREEZE-END>
    ];
    entries.iter().copied().collect()
});

fn frozen() -> &'static HashMap<&'static str, &'static str> {
    &FROZEN
}

// Referenced constants kept alive for future freeze entries.
#[allow(dead_code)]
fn _pinned_hashes() -> (&'static str, &'static str) {
    (H_MAIN, H_LIB)
}
