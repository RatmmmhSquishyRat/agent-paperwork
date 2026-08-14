//! T6 follow-up wiring tests (additive; char_tests stays untouched).
//!
//! - NEW-4: contacts read enrichment resolves entry paths through core's
//!   two-level `resolve_contact_path` (as-given first, then relative to the
//!   contacts file's own directory) — locked here with a scenario where the
//!   entry path is only resolvable relative to the contacts directory.
//! - Sam-S3: scoped `profile create` is a single create_new write — an
//!   existing target is still refused without being touched.

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
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    std::fs::write(path, content).expect("write fixture");
}

fn read_file(dir: &TempDir, rel: &str) -> String {
    std::fs::read_to_string(dir.path().join(rel)).expect("read fixture")
}

const PROFILE_ALICE: &str = "# alice\n\nReviewer.\n\n- model: m1\n";

/// Contacts file lives in a subdirectory; its entry path `alice.profile.md`
/// does NOT exist relative to the CWD — only relative to the contacts file's
/// own directory. Pre-NEW-4 enrichment reported `(unreadable)` here.
#[test]
fn t6_contacts_read_enriches_relative_to_contacts_dir() {
    let dir = TempDir::new().unwrap();
    write(&dir, "team/alice.profile.md", PROFILE_ALICE);
    write(
        &dir,
        "team/roster.contacts.md",
        "# Contacts\n\n- [alice](alice.profile.md)\n",
    );

    let r = run(&dir, &["contacts", "read", "team/roster"]);
    assert_eq!(r.code, 0);
    assert_eq!(
        r.stdout, "ok contacts.read 1 contacts\n---\nalice.profile.md: alice (Reviewer.)\n",
        "entry must be enriched via contacts-directory-relative resolution, not (unreadable)"
    );
}

#[test]
fn t6_contacts_read_json_enriches_relative_to_contacts_dir() {
    let dir = TempDir::new().unwrap();
    write(&dir, "team/alice.profile.md", PROFILE_ALICE);
    write(
        &dir,
        "team/roster.contacts.md",
        "# Contacts\n\n- [alice](alice.profile.md)\n",
    );

    let r = run(&dir, &["--json", "contacts", "read", "team/roster"]);
    assert_eq!(r.code, 0);
    assert_eq!(
        r.stdout,
        "{\"command\":\"contacts.read\",\"conclusion\":\"1 contacts\",\"contacts\":[{\"description\":\"Reviewer.\",\"label\":\"alice\",\"name\":\"alice\",\"path\":\"alice.profile.md\"}],\"status\":\"ok\"}\n"
    );
}

/// An entry that resolves neither as-given nor contacts-relative still
/// degrades to `(unreadable)` (enrichment never fails the read).
#[test]
fn t6_contacts_read_unresolvable_entry_stays_unreadable() {
    let dir = TempDir::new().unwrap();
    write(&dir, "team/alice.profile.md", PROFILE_ALICE);
    write(
        &dir,
        "team/roster.contacts.md",
        "# Contacts\n\n- [alice](alice.profile.md)\n- [ghost](ghost.profile.md)\n",
    );

    let r = run(&dir, &["contacts", "read", "team/roster"]);
    assert_eq!(r.code, 0);
    assert_eq!(
        r.stdout,
        "ok contacts.read 2 contacts\n---\nalice.profile.md: alice (Reviewer.)\nghost.profile.md: (unreadable)\n"
    );
}

/// Sam-S3 behavior face: scoped create is a single create_new write — an
/// existing target is refused (exit 1) and left byte-identical.
#[test]
fn t6_profile_create_scoped_refuses_existing_file() {
    let dir = TempDir::new().unwrap();
    write(&dir, "alice.profile.md", PROFILE_ALICE);

    let r = run(
        &dir,
        &[
            "profile",
            "create",
            "alice",
            "--name",
            "mallory",
            "--model",
            "m9",
            "--scope-read",
            "src/**",
        ],
    );
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.starts_with("error already-exists:"),
        "stderr reports already-exists: {}",
        r.stderr
    );
    assert_eq!(read_file(&dir, "alice.profile.md"), PROFILE_ALICE);
}
