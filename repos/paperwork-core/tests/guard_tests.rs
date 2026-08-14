//! T2 correctness-guardrail regression tests (P-2 batch; ported from the
//! wip/v0.5-perfection-snapshot branch, adapted onto the merged master
//! lock layer — write ops stay on `locked_read_modify_write`).
//!
//! Covers:
//! - NEW-1 write-side injection guardrails (single-line fields, prose
//!   representability, dangerous structural-key lines in preamble prose);
//! - NEW-2 atomic `create_new` on all create ops (incl. an 8-thread race);
//! - NEW-4 `resolve_contact_path` two-level resolution helper;
//! - NEW-5 fence-aware `parse_contacts_title`;
//! - NEW-6 documented tail-scan fence-parity limitation (pin, not a fix);
//! - Sam-S1 legacy brief residue refusal at parse time;
//! - Sam-S3 one-shot `create_profile_full`;
//! - Sam-S5 (ruling A): missing target stays Stale per the frozen spec §6
//!   contract; genuine (non-NotFound) IO failures surface as io errors.

use std::fs;
use std::sync::{Arc, Barrier};
use std::thread as std_thread;

use tempfile::tempdir;

use paperwork_core::format::contacts::parse_contacts_title;
use paperwork_core::format::manifest::parse_manifest;
use paperwork_core::ops::{contacts, manifest, profile, thread};
use paperwork_core::{Profile, ThreadMeta, VerifyResult};

/// Reverse-scan buffer size (spec §5.5: 64KB + 256B) — mirrors
/// `read_last_seq_locked`'s REVERSE_SCAN_SIZE.
const SCAN: u64 = 64 * 1024 + 256;

fn meta(title: &str) -> ThreadMeta {
    ThreadMeta {
        title: title.to_string(),
    }
}

fn assert_validation(err: &paperwork_core::PaperworkError) {
    assert_eq!(err.category(), "validation");
}

// ============================================================================
// NEW-1: single-line field guardrails
// ============================================================================

#[test]
fn thread_send_rejects_multiline_preamble_title() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("t.post.md");

    let err = thread::thread_send(
        &path,
        "alice",
        "hello",
        Some(&meta("line one\n## injected (2026-01-01T00:00:00Z)")),
    )
    .expect_err("multiline title must be rejected");
    assert_validation(&err);
    assert!(err.to_string().contains("thread title"));

    // carriage return is equally refused
    let err = thread::thread_send(&path, "alice", "hello", Some(&meta("a\rb")))
        .expect_err("CR title must be rejected");
    assert_validation(&err);

    // nothing landed on disk
    assert!(!path.exists());

    // a legal single-line title still works
    let seq = thread::thread_send(&path, "alice", "hello", Some(&meta("fine title")))
        .expect("legal title accepted");
    assert_eq!(seq, 1);
}

#[test]
fn create_profile_rejects_multiline_name_and_model() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.profile.md");

    let err = profile::create_profile(&path, "ali\nce", "gpt-4", "")
        .expect_err("multiline name must be rejected");
    assert_validation(&err);
    assert!(err.to_string().contains("name"));

    let err = profile::create_profile(&path, "alice", "gpt\n- model: fake", "")
        .expect_err("multiline model must be rejected");
    assert_validation(&err);
    assert!(err.to_string().contains("model"));

    assert!(!path.exists());

    profile::create_profile(&path, "alice", "gpt-4", "Legal multi\nline description.").expect("ok");
}

#[test]
fn edit_profile_rejects_multiline_model_and_bad_description() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.profile.md");
    profile::create_profile(&path, "alice", "gpt-4", "Original.").expect("create");

    let err = profile::edit_profile(&path, Some("mod\nel"), None, None, None, None)
        .expect_err("multiline model must be rejected");
    assert_validation(&err);

    let err = profile::edit_profile(&path, None, Some("- model: fake"), None, None, None)
        .expect_err("attribute-shaped description must be rejected");
    assert_validation(&err);

    // file unchanged
    let p = profile::show_profile(&path).expect("read");
    assert_eq!(p.model, "gpt-4");
    assert_eq!(p.description, "Original.");

    // legal edits still work
    profile::edit_profile(
        &path,
        Some("gpt-5"),
        Some("New\ndescription."),
        None,
        None,
        None,
    )
    .expect("legal edit");
    let p = profile::show_profile(&path).expect("read");
    assert_eq!(p.model, "gpt-5");
    assert_eq!(p.description, "New\ndescription.");
}

#[test]
fn contacts_create_rejects_multiline_title() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("team.contacts.md");

    let err = contacts::contacts_create(&path, "team\n# injected")
        .expect_err("multiline title must be rejected");
    assert_validation(&err);
    assert!(!path.exists());

    contacts::contacts_create(&path, "team").expect("legal title accepted");
}

#[test]
fn contacts_add_rejects_multiline_profile_path() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("team.contacts.md");
    contacts::contacts_create(&path, "team").expect("create");

    let err = contacts::contacts_add(&path, "agents/alice.profile.md\n- [evil](e.md)")
        .expect_err("multiline profile path must be rejected");
    assert_validation(&err);
    assert!(err.to_string().contains("profile path"));

    let entries = contacts::contacts_read(&path).expect("read");
    assert!(entries.is_empty(), "nothing may land on disk");

    contacts::contacts_add(&path, "agents/alice.profile.md").expect("legal path accepted");
}

#[test]
fn brief_create_rejects_multiline_title_and_owner() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("b.brief.md");

    let err = manifest::brief_create(&path, "t\n## injected", None, "")
        .expect_err("multiline title must be rejected");
    assert_validation(&err);

    let err = manifest::brief_create(&path, "title", Some("own\ner"), "")
        .expect_err("multiline owner must be rejected");
    assert_validation(&err);
    assert!(!path.exists());

    manifest::brief_create(&path, "title", Some("alice"), "Legal\ndescription.").expect("ok");
}

#[test]
fn brief_add_entry_rejects_multiline_entry_path() {
    let dir = tempdir().expect("tempdir");
    let brief = dir.path().join("b.brief.md");
    manifest::brief_create(&brief, "t", None, "").expect("create");

    let err = manifest::brief_add_entry(&brief, "a.rs\n- path: evil", None, None)
        .expect_err("multiline entry path must be rejected");
    assert_validation(&err);

    let m = manifest::brief_read(&brief).expect("read");
    assert!(m.entries.is_empty());
}

// ============================================================================
// NEW-1: preamble prose representability (dangerous structural-key lines)
// ============================================================================

#[test]
fn profile_description_rejects_dangerous_key_line() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.profile.md");

    for desc in [
        "- model: fake",
        "Nice prose.\n- model: fake",
        "Nice prose.\n- owner: mallory",
        "Nice prose.\n- path: /etc/passwd",
        "Nice prose.\n- hash: deadbeef",
        "Nice prose.\n- regex: x",
    ] {
        let result = profile::create_profile(&path, "alice", "gpt-4", desc);
        assert!(result.is_err(), "desc {:?} must be rejected", desc);
        assert_validation(&result.unwrap_err());
    }
    assert!(!path.exists());

    // a dangerous line inside a fence is refused too: the preamble parsers
    // are not fence-aware, so the fence does not shield it
    let err = profile::create_profile(&path, "alice", "gpt-4", "Prose.\n```\n- model: quoted\n```")
        .expect_err("fenced dangerous line is not shielded");
    assert_validation(&err);
    assert!(!path.exists());

    // legal multi-line description still passes
    let path2 = dir.path().join("bob.profile.md");
    profile::create_profile(
        &path2,
        "bob",
        "gpt-4",
        "Line one.\nLine two.\n- unknown-key: fine",
    )
    .expect("legal multiline description accepted");
    let p = profile::show_profile(&path2).expect("read");
    assert_eq!(p.description, "Line one.\nLine two.");
}

#[test]
fn brief_description_rejects_dangerous_key_line() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("b.brief.md");

    let err = manifest::brief_create(&path, "t", Some("alice"), "Prose.\n- owner: mallory")
        .expect_err("dangerous key line in description must be rejected");
    assert_validation(&err);
    assert!(!path.exists());

    manifest::brief_create(&path, "t", Some("alice"), "Legal\ndescription.").expect("ok");
    let m = manifest::brief_read(&path).expect("read");
    assert_eq!(m.author, "alice");
    assert_eq!(m.description, "Legal\ndescription.");
}

#[test]
fn brief_note_later_attribute_line_stays_legal() {
    // Regression pin: notes sit AFTER the entry attribute zone, so a later
    // attribute-shaped line inside a note is verbatim content (BDD:BRIEF-12).
    let dir = tempdir().expect("tempdir");
    let brief = dir.path().join("b.brief.md");
    let target = dir.path().join("test.txt");
    fs::write(&target, "content").expect("write target");
    manifest::brief_create(&brief, "t", None, "").expect("create");

    manifest::brief_add_entry(
        &brief,
        "test.txt",
        None,
        Some("Prose first.\n- path: fine inside"),
    )
    .expect("later attribute-shaped note line stays legal");
    let m = manifest::brief_read(&brief).expect("read");
    assert_eq!(m.entries.len(), 1);
    assert_eq!(m.entries[0].path, "test.txt", "real path not shadowed");
}

// ============================================================================
// NEW-2: atomic create (TOCTOU) — single-thread repeat + 8-thread races
// ============================================================================

#[test]
fn create_ops_repeat_rejected_with_existing_envelope() {
    let dir = tempdir().expect("tempdir");

    let p = dir.path().join("alice.profile.md");
    profile::create_profile(&p, "alice", "gpt-4", "").expect("first");
    let err = profile::create_profile(&p, "mallory", "evil", "").unwrap_err();
    assert_eq!(err.category(), "already-exists");
    assert!(err.to_string().contains("already exists"));
    let original = fs::read_to_string(&p).expect("read");
    assert!(original.contains("# alice"), "first content must survive");

    let b = dir.path().join("b.brief.md");
    manifest::brief_create(&b, "first", None, "").expect("first");
    let err = manifest::brief_create(&b, "second", None, "").unwrap_err();
    assert_eq!(err.category(), "already-exists");
    assert!(fs::read_to_string(&b).unwrap().contains("# first"));

    let c = dir.path().join("team.contacts.md");
    contacts::contacts_create(&c, "first").expect("first");
    let err = contacts::contacts_create(&c, "second").unwrap_err();
    assert_eq!(err.category(), "already-exists");
    assert!(fs::read_to_string(&c).unwrap().contains("# first"));
}

fn concurrent_create_exactly_one_wins<F>(path: Arc<std::path::PathBuf>, create: F)
where
    F: Fn(&std::path::Path) -> paperwork_core::Result<()> + Send + Sync + 'static,
{
    let barrier = Arc::new(Barrier::new(8));
    let create = Arc::new(create);
    let mut handles = vec![];
    for _ in 0..8 {
        let path = Arc::clone(&path);
        let barrier = Arc::clone(&barrier);
        let create = Arc::clone(&create);
        handles.push(std_thread::spawn(move || {
            barrier.wait();
            create(&path)
        }));
    }
    let mut ok = 0;
    let mut already = 0;
    for h in handles {
        match h.join().expect("thread panicked") {
            Ok(()) => ok += 1,
            Err(e) => {
                assert_eq!(
                    e.category(),
                    "already-exists",
                    "losers must see the AlreadyExists envelope"
                );
                already += 1;
            }
        }
    }
    assert_eq!(ok, 1, "exactly one creator may win");
    assert_eq!(already, 7, "all losers must be refused");
}

#[test]
fn concurrent_create_profile_exactly_one_wins() {
    let dir = tempdir().expect("tempdir");
    let path = Arc::new(dir.path().join("racer.profile.md"));
    concurrent_create_exactly_one_wins(Arc::clone(&path), move |p| {
        profile::create_profile(p, "racer", "gpt-4", "")
    });
    assert!(fs::read_to_string(&*path).unwrap().contains("# racer"));
}

#[test]
fn concurrent_brief_create_exactly_one_wins() {
    let dir = tempdir().expect("tempdir");
    let path = Arc::new(dir.path().join("racer.brief.md"));
    concurrent_create_exactly_one_wins(Arc::clone(&path), move |p| {
        manifest::brief_create(p, "racer", None, "")
    });
    assert!(fs::read_to_string(&*path).unwrap().contains("# racer"));
}

#[test]
fn concurrent_contacts_create_exactly_one_wins() {
    let dir = tempdir().expect("tempdir");
    let path = Arc::new(dir.path().join("racer.contacts.md"));
    concurrent_create_exactly_one_wins(Arc::clone(&path), move |p| {
        contacts::contacts_create(p, "racer")
    });
    assert!(fs::read_to_string(&*path).unwrap().contains("# racer"));
}

// ============================================================================
// NEW-4: resolve_contact_path two-level resolution helper
// ============================================================================

#[test]
fn resolve_contact_path_two_levels() {
    let dir = tempdir().expect("tempdir");

    // Layout: <tmp>/team.contacts.md + <tmp>/agents/alice.profile.md,
    // CWD-independent relative entry path resolves against the contacts dir.
    let agents = dir.path().join("agents");
    fs::create_dir_all(&agents).expect("mkdir");
    fs::write(
        agents.join("alice.profile.md"),
        "# alice\n\n- model: gpt-4o\n",
    )
    .expect("write profile");
    let contacts_path = dir.path().join("team.contacts.md");

    // Level 2: not present as given -> contacts-directory-relative.
    let resolved = contacts::resolve_contact_path(&contacts_path, "agents/alice.profile.md");
    assert_eq!(resolved, agents.join("alice.profile.md"));
    assert!(resolved.exists());

    // Level 1: present as given wins (absolute path).
    let abs = agents.join("alice.profile.md");
    let resolved = contacts::resolve_contact_path(&contacts_path, abs.to_str().unwrap());
    assert_eq!(resolved, abs);

    // derive_label (which delegates to the helper) picks up the H1 name
    // through the same two-level resolution.
    contacts::contacts_create(&contacts_path, "team").expect("create");
    contacts::contacts_add(&contacts_path, "agents/alice.profile.md").expect("add");
    let entries = contacts::contacts_read(&contacts_path).expect("read");
    assert_eq!(entries[0].label, "alice");

    // Non-existent paths still resolve contacts-directory-relative.
    let resolved = contacts::resolve_contact_path(&contacts_path, "agents/ghost.profile.md");
    assert_eq!(resolved, dir.path().join("agents/ghost.profile.md"));
}

// ============================================================================
// NEW-5: fence-aware parse_contacts_title
// ============================================================================

#[test]
fn parse_contacts_title_fence_aware() {
    // an H1 inside a fence is quoted example content, never the title
    let content = "```markdown\n# fake title\n```\n\n# Real Title\n\n- [alice](a.md)\n";
    assert_eq!(parse_contacts_title(content).expect("title"), "Real Title");

    // fence-only file: no title found (error unchanged)
    let err = parse_contacts_title("```\n# fake\n```\n").unwrap_err();
    assert!(err.to_string().contains("missing contacts title heading"));

    // the fence before the real title may use a longer run
    let content = "````md\n# fake\n````\n# Real\n";
    assert_eq!(parse_contacts_title(content).expect("title"), "Real");

    // plain (no fence) behaviour preserved
    assert_eq!(parse_contacts_title("# T\n").expect("title"), "T");
}

// ============================================================================
// NEW-6: tail-scan fence-parity residual limitation (spec §5.5, pinned)
// ============================================================================

/// Spec §5.5 residual limitation (documented, pinned — NOT a fix): the
/// 64KB+256B reverse scan cannot know the fence parity before the buffer
/// start. When the buffer boundary cuts through an OPEN fence (the opening
/// line lies entirely before the buffer), the in-buffer fence tracker starts
/// closed and a fence-internal candidate header pollutes `last_seq`. The
/// validate op's seq-continuity check is the designed backstop.
#[test]
fn tail_scan_fence_parity_limitation_pinned() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("big.post.md");

    let head = "# t\n\n## #1 alice (2026-01-15T10:00:00Z)\n\n```md\nbody\n```\n\n";
    let fence_open = "```md\n"; // opening line sits entirely BEFORE the buffer
    let fake_header = "## #1000 mallory (2026-01-01T00:00:00Z)\n";
    let fence_close = "```\n\n";

    // The scan buffer is the last SCAN bytes; arrange for it to start with
    // the very first byte AFTER the fence opening line: read_start is then
    // preceded by '\n' (no R7 trimming) but the opening line itself is
    // outside the buffer, so the in-buffer fence tracker starts "closed"
    // and the fence-internal fake header looks like a real candidate.
    let inside_len = SCAN as usize - fake_header.len() - fence_close.len();
    let mut filler = "f".repeat(inside_len - 1);
    filler.push('\n');
    let content = format!(
        "{}{}{}{}{}",
        head, fence_open, fake_header, filler, fence_close
    );

    let buffer_start = content.len() - SCAN as usize;
    assert_eq!(
        buffer_start,
        head.len() + fence_open.len(),
        "fixture must start the buffer right after the fence opening line"
    );
    assert_eq!(content.as_bytes()[buffer_start - 1], b'\n', "R7 probe byte");
    assert!(content.len() as u64 > SCAN);
    fs::write(&path, content).expect("write fixture");

    // Full parse sees exactly the real message; the fake header is fence
    // content.
    let messages = thread::thread_read(&path, None, None).expect("read");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].seq, 1);

    // PINNED documented behavior (spec §5.5): the tail scan cannot know the
    // pre-buffer fence parity, reads the fake in-fence header, and assigns
    // 1001 instead of 2. validate's seq-continuity check is the designed
    // backstop that exposes such pollution.
    let seq = thread::thread_send(&path, "bob", "next", None).expect("send");
    assert_eq!(seq, 1001, "spec §5.5 residual limitation, pinned");
}

// ============================================================================
// Sam-S1: legacy brief residue guard
// ============================================================================

fn legal_brief() -> &'static str {
    "# Onboarding\n\nHow to read this repo\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n## main.rs\n\n- path: src/main.rs\n- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540\n\nEntry point\n"
}

#[test]
fn legacy_brief_residue_rejected_at_parse() {
    // residual `## Entries` wrapper (exact heading, fence-outside)
    let with_wrapper = format!("{}\n## Entries\n\n### main.rs\n", legal_brief());
    let err = parse_manifest(&with_wrapper).unwrap_err();
    assert_eq!(err.category(), "format");
    assert!(err.to_string().contains("legacy v0.4 residue"));
    assert!(err.fix().contains("CHANGELOG"));

    // residual H3 entry headers alone also trigger
    let with_h3 = format!("{}\n### sneaky\n", legal_brief());
    assert!(parse_manifest(&with_h3).is_err());

    // legal v0.5 brief unaffected (H2 entries are the v0.5 shape)
    let parsed = parse_manifest(legal_brief()).expect("legal brief parses");
    assert_eq!(parsed.entries.len(), 1);

    // `### ` inside a note fence is quoted content — no false positive
    let with_fenced_h3 = format!("{}\n```\n### quoted example\n```\n", legal_brief());
    // NB: the fenced block belongs to the entry note; still parses fine
    let parsed = parse_manifest(&with_fenced_h3).expect("fenced H3 is safe");
    assert!(parsed.entries[0]
        .note
        .as_ref()
        .unwrap()
        .contains("### quoted example"));

    // an `## EntriesX` heading is a plain v0.5 entry, not residue
    let not_residue =
        "# b\n\n- owner: a\n- created: 2026-01-15T10:00:00Z\n\n## EntriesX\n\n- path: a\n- hash: b\n";
    assert!(parse_manifest(not_residue).is_ok());
}

#[test]
fn legacy_brief_write_ops_refuse_and_leave_bytes_unchanged() {
    let dir = tempdir().expect("tempdir");
    let brief = dir.path().join("b.brief.md");
    let target = dir.path().join("t.txt");
    fs::write(&target, "content").expect("write target");

    // half-migrated file: lowercase keys but residual H3 entry headers
    let legacy = "# b\n\n- owner: alice\n- created: 2026-01-15T10:00:00Z\n\n### old-entry\n\n- Path: t.txt\n";
    fs::write(&brief, legacy).expect("write legacy brief");
    let before = fs::read(&brief).expect("read bytes");

    let err = manifest::brief_add_entry(&brief, "t.txt", None, Some("note"))
        .expect_err("add must refuse legacy residue");
    assert_eq!(err.category(), "format");
    assert!(err.to_string().contains("error"));

    let err = manifest::brief_remove_entry(&brief, "old-entry")
        .expect_err("remove must refuse legacy residue");
    assert_eq!(err.category(), "format");

    // byte-for-byte unchanged
    assert_eq!(fs::read(&brief).expect("read bytes"), before);

    // a legal brief remains fully operable
    let ok_brief = dir.path().join("ok.brief.md");
    fs::write(&ok_brief, legal_brief()).expect("write legal brief");
    manifest::brief_add_entry(&ok_brief, "t.txt", None, Some("note")).expect("legal add works");
    let m = manifest::brief_read(&ok_brief).expect("read");
    assert_eq!(m.entries.len(), 2);
}

// ============================================================================
// Sam-S3: one-shot profile creation with scopes
// ============================================================================

#[test]
fn create_profile_full_single_shot_with_scopes() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("alice.profile.md");

    let profile = Profile {
        name: "alice".to_string(),
        model: "gpt-4o".to_string(),
        description: "Full one-shot profile".to_string(),
        scope_read: vec!["src/**".to_string()],
        scope_write: vec!["src/**".to_string()],
        scope_owns: vec!["docs/**".to_string()],
    };
    profile::create_profile_full(&path, &profile).expect("one-shot create");

    // the file carries the scopes from the FIRST (and only) write
    let raw = fs::read_to_string(&path).expect("read");
    assert!(raw.contains("## Scope"));
    assert!(raw.contains("- read: src/**"));

    let parsed = profile::show_profile(&path).expect("read back");
    assert_eq!(parsed, profile);

    // refuses overwrite; original bytes survive
    let mut evil = profile.clone();
    evil.name = "mallory".to_string();
    let err = profile::create_profile_full(&path, &evil).unwrap_err();
    assert_eq!(err.category(), "already-exists");
    assert!(fs::read_to_string(&path).unwrap().contains("# alice"));

    // same injection guards as create_profile
    let mut bad = profile.clone();
    bad.name = "two\nlines".to_string();
    let err = profile::create_profile_full(&dir.path().join("x.profile.md"), &bad).unwrap_err();
    assert_validation(&err);
}

// ============================================================================
// Sam-S5: verify distinguishes IO failures from Stale
// ============================================================================

#[test]
fn brief_verify_missing_target_stays_stale_per_spec() {
    // Sam-S5 ruling A: "missing target -> Stale" is the frozen spec §6
    // three-state contract ("Stale: regex fails to match (or file
    // missing)") — intentional design, preserved.
    let dir = tempdir().expect("tempdir");
    let brief = dir.path().join("b.brief.md");
    let target = dir.path().join("target.txt");
    fs::write(&target, "content").expect("write target");

    manifest::brief_create(&brief, "t", None, "").expect("create");
    manifest::brief_add_entry(&brief, "target.txt", Some("content"), None).expect("add");

    // Fresh while the target exists and matches
    let results = manifest::brief_verify(&brief, dir.path()).expect("verify");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, VerifyResult::Fresh);

    // Remove the target: Stale per spec contract, NOT an error.
    fs::remove_file(&target).expect("remove target");
    let results = manifest::brief_verify(&brief, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Stale);
}

#[test]
fn brief_verify_real_io_failure_is_not_swallowed_as_stale() {
    // Sam-S5 ruling A: genuine read failures (anything other than
    // NotFound) surface as io-category errors instead of collapsing into
    // Stale. Portable construction: replace the target file with a
    // DIRECTORY of the same name — reading a directory fails with a
    // non-NotFound io error on both Windows (access denied) and Unix
    // (is-a-directory).
    let dir = tempdir().expect("tempdir");
    let brief = dir.path().join("b.brief.md");
    let target = dir.path().join("target.txt");
    fs::write(&target, "content").expect("write target");

    manifest::brief_create(&brief, "t", None, "").expect("create");
    manifest::brief_add_entry(&brief, "target.txt", None, None).expect("add");

    fs::remove_file(&target).expect("remove target");
    fs::create_dir(&target).expect("create dir in place of the target");

    let err = manifest::brief_verify(&brief, dir.path())
        .expect_err("a real read failure must not be swallowed into Stale");
    assert_eq!(err.category(), "io");
}

#[test]
fn brief_verify_semantic_drift_still_reports_states() {
    let dir = tempdir().expect("tempdir");
    let brief = dir.path().join("b.brief.md");
    let target = dir.path().join("target.txt");
    fs::write(&target, "content").expect("write target");

    manifest::brief_create(&brief, "t", None, "").expect("create");
    manifest::brief_add_entry(&brief, "target.txt", Some("content"), None).expect("add");

    // hash differs but regex still matches -> Shifted
    fs::write(&target, "content changed").expect("mutate target");
    let results = manifest::brief_verify(&brief, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Shifted);

    // regex no longer matches -> Stale (semantic drift stays three-state)
    fs::write(&target, "nothing to see").expect("mutate target");
    let results = manifest::brief_verify(&brief, dir.path()).expect("verify");
    assert_eq!(results[0].1, VerifyResult::Stale);
}
