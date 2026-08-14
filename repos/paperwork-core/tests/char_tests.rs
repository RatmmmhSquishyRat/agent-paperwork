//! T1 behavior lock — characterization tests (core face): byte-identity
//! roundtrip corpus for all four managed formats (format v2).
//!
//! Every sample is a canonical legal document; parse → serialize must
//! reproduce the input BYTES exactly. These tests are the regression gate
//! for the upcoming structural refactor: any serializer drift fails here.
//!
//! Additive only — no existing source/test file is modified.

use paperwork_core::format::contacts::{parse_contacts, parse_contacts_title, serialize_contacts};
use paperwork_core::format::manifest::{parse_manifest, serialize_manifest};
use paperwork_core::format::profile::{parse_profile, serialize_profile};
use paperwork_core::format::thread::{parse_messages, parse_preamble, serialize_thread};

// ---------------------------------------------------------------------------
// Thread (.post.md) — 3 samples
// ---------------------------------------------------------------------------

/// Multi-message thread: mentions, `@#N` reply token, nested-backtick body
/// (dynamic fence grows to 4) and an empty-body message.
const THREAD_T1: &str = "# Design Discussion\n\n## #1 alice (2026-03-01T08:00:00Z)\n\n```md\nOpening proposal @bob @carol\n```\n\n## #2 bob (2026-03-01T08:05:00Z)\n\n````md\n@#1 @alice counter-proposal:\n```rust\nfn main() {}\n```\n````\n\n## #3 carol (2026-03-01T08:10:00Z)\n\n```md\n```\n\n";

#[test]
fn char_thread_roundtrip_t1_multi_message_refs_dynamic_fence() {
    let meta = parse_preamble(THREAD_T1);
    assert_eq!(meta.title, "Design Discussion");
    let messages = parse_messages(THREAD_T1).expect("parse t1");
    assert_eq!(messages.len(), 3);
    // derivation freeze (spec §5.4)
    assert_eq!(messages[0].mentions, vec!["bob", "carol"]);
    assert_eq!(messages[0].reply_to, None);
    assert_eq!(messages[1].reply_to, Some(1));
    assert_eq!(messages[1].mentions, vec!["alice"]);
    assert_eq!(messages[2].body, "");
    assert!(
        serialize_thread(&meta, &messages) == THREAD_T1,
        "t1 byte identity"
    );
}

/// Unicode thread: multi-line body, reply token without mention.
const THREAD_T2: &str = "# 团队讨论\n\n## #1 alicé (2026-03-02T12:00:00Z)\n\n```md\n第一行 @bob\n第二行 你好 🚀\n```\n\n## #2 bob (2026-03-02T12:05:00Z)\n\n```md\n@#1 收到\n```\n\n";

#[test]
fn char_thread_roundtrip_t2_unicode_multiline() {
    let meta = parse_preamble(THREAD_T2);
    let messages = parse_messages(THREAD_T2).expect("parse t2");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].sender, "alicé");
    assert_eq!(messages[0].mentions, vec!["bob"]);
    assert_eq!(messages[1].reply_to, Some(1));
    assert!(
        serialize_thread(&meta, &messages) == THREAD_T2,
        "t2 byte identity"
    );
}

/// Fence growth to 5: body carries a 4-backtick run.
const THREAD_T3: &str = "# Fence Growth\n\n## #1 alice (2026-03-03T13:00:00Z)\n\n`````md\nquoted fence:\n````\ninner\n````\n`````\n\n";

#[test]
fn char_thread_roundtrip_t3_fence_growth() {
    let meta = parse_preamble(THREAD_T3);
    let messages = parse_messages(THREAD_T3).expect("parse t3");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].body, "quoted fence:\n````\ninner\n````");
    assert!(
        serialize_thread(&meta, &messages) == THREAD_T3,
        "t3 byte identity"
    );
}

// ---------------------------------------------------------------------------
// Profile (.profile.md) — 3 samples
// ---------------------------------------------------------------------------

const PROFILE_P1: &str = "# alice\n\n- model: gpt-4o\n";

#[test]
fn char_profile_roundtrip_p1_minimal() {
    let profile = parse_profile(PROFILE_P1).expect("parse p1");
    assert_eq!(profile.name, "alice");
    assert_eq!(profile.description, "");
    assert!(
        serialize_profile(&profile) == PROFILE_P1,
        "p1 byte identity"
    );
}

/// Full profile: multi-line description, repeated scope keys (read before
/// write before owns — the canonical serialization order).
const PROFILE_P2: &str = "# bob\n\nReviewer for the parser module.\nSecond description line.\n\n- model: claude-4\n\n## Scope\n\n- read: src/**\n- read: docs/**\n- write: src/parser/**\n- owns: src/parser/**\n";

#[test]
fn char_profile_roundtrip_p2_full_scope() {
    let profile = parse_profile(PROFILE_P2).expect("parse p2");
    assert_eq!(
        profile.description,
        "Reviewer for the parser module.\nSecond description line."
    );
    assert_eq!(profile.scope_read, vec!["src/**", "docs/**"]);
    assert_eq!(profile.scope_write, vec!["src/parser/**"]);
    assert_eq!(profile.scope_owns, vec!["src/parser/**"]);
    assert!(
        serialize_profile(&profile) == PROFILE_P2,
        "p2 byte identity"
    );
}

const PROFILE_P3: &str = "# carol\n\nDocs owner.\n\n- model: m3\n";

#[test]
fn char_profile_roundtrip_p3_description_no_scope() {
    let profile = parse_profile(PROFILE_P3).expect("parse p3");
    assert!(profile.scope_read.is_empty());
    assert!(
        serialize_profile(&profile) == PROFILE_P3,
        "p3 byte identity"
    );
}

// ---------------------------------------------------------------------------
// Brief (.brief.md) — 3 samples
// ---------------------------------------------------------------------------

const HEX64: &str = "42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540";

/// Description preamble + inline regex + prose note.
const BRIEF_B1: &str = "# Onboarding\n\nHow to read this repo.\n\n- owner: alice\n- created: 2026-02-01T09:00:00Z\n\n## main.rs\n\n- path: src/main.rs\n- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540\n- regex: fn main\n\nEntry point.\n";

#[test]
fn char_brief_roundtrip_b1_inline_regex_note() {
    let manifest = parse_manifest(BRIEF_B1).expect("parse b1");
    assert_eq!(manifest.name, "Onboarding");
    assert_eq!(manifest.author, "alice");
    assert_eq!(manifest.description, "How to read this repo.");
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(manifest.entries[0].regex, Some("fn main".to_string()));
    assert_eq!(manifest.entries[0].note, Some("Entry point.".to_string()));
    assert!(
        serialize_manifest(&manifest) == BRIEF_B1,
        "b1 byte identity"
    );
}

/// Two entries: fenced multi-line regex (backtick inside → dynamic fence)
/// with derived capture groups, followed by a regex-less entry.
const BRIEF_B2: &str = "# Log Brief\n\n- owner: bob\n- created: 2026-02-02T10:00:00Z\n\n## log\n\n- path: data.log\n- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540\n```regex\n(?<year>\\d{4})-(?<month>\\d{2})\nwith `backtick` line\n```\n\n## cfg\n\n- path: config.toml\n- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540\n";

#[test]
fn char_brief_roundtrip_b2_fenced_regex_multi_entry() {
    let manifest = parse_manifest(BRIEF_B2).expect("parse b2");
    assert_eq!(manifest.entries.len(), 2);
    assert_eq!(
        manifest.entries[0].regex,
        Some("(?<year>\\d{4})-(?<month>\\d{2})\nwith `backtick` line".to_string())
    );
    assert_eq!(manifest.entries[0].groups, vec!["year", "month"]);
    assert_eq!(manifest.entries[1].regex, None);
    assert!(
        serialize_manifest(&manifest) == BRIEF_B2,
        "b2 byte identity"
    );
}

/// Attribute-zone boundary: a blank line does not terminate it; an
/// attribute-shaped line AFTER prose belongs to the note verbatim.
const BRIEF_B3: &str = "# Notes Brief\n\n- owner: carol\n- created: 2026-02-03T11:00:00Z\n\n## notes\n\n- path: a.md\n- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540\n\nProse first line.\n- not: absorbed\n";

#[test]
fn char_brief_roundtrip_b3_attribute_zone_boundary() {
    let manifest = parse_manifest(BRIEF_B3).expect("parse b3");
    let entry = &manifest.entries[0];
    assert_eq!(entry.path, "a.md");
    assert_eq!(entry.hash, HEX64);
    assert_eq!(
        entry.note,
        Some("Prose first line.\n- not: absorbed".to_string())
    );
    assert!(
        serialize_manifest(&manifest) == BRIEF_B3,
        "b3 byte identity"
    );
}

// ---------------------------------------------------------------------------
// Contacts (.contacts.md) — 3 samples
// ---------------------------------------------------------------------------

const CONTACTS_C1: &str =
    "# Core Team\n\n- [alice](agents/alice.profile.md)\n- [bob](agents/bob.profile.md)\n";

#[test]
fn char_contacts_roundtrip_c1_plain_links() {
    let contacts = parse_contacts(CONTACTS_C1).expect("parse c1");
    assert_eq!(contacts.len(), 2);
    assert_eq!(
        parse_contacts_title(CONTACTS_C1).expect("title"),
        "Core Team"
    );
    assert!(
        serialize_contacts("Core Team", &contacts) == CONTACTS_C1,
        "c1 byte identity"
    );
}

/// Angle-bracket destinations, escaped label `]`, escaped `\<`/`\>`.
const CONTACTS_C2: &str = "# Mixed\n\n- [alice](<team docs/alice.profile.md>)\n- [we\\]ird](<a\\<b\\>c.md>)\n- [plain](simple.md)\n";

#[test]
fn char_contacts_roundtrip_c2_escapes() {
    let contacts = parse_contacts(CONTACTS_C2).expect("parse c2");
    assert_eq!(contacts.len(), 3);
    assert_eq!(contacts[1].label, "we]ird");
    assert_eq!(contacts[1].profile_path, "a<b>c.md");
    assert!(
        serialize_contacts("Mixed", &contacts) == CONTACTS_C2,
        "c2 byte identity"
    );
}

/// Windows-style path: backslashes + a space (angle form, `\\` escaping).
const CONTACTS_C3: &str = "# Windows\n\n- [win](<C:\\\\team docs\\\\alice.profile.md>)\n";

#[test]
fn char_contacts_roundtrip_c3_windows_path() {
    let contacts = parse_contacts(CONTACTS_C3).expect("parse c3");
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].profile_path, "C:\\team docs\\alice.profile.md");
    assert!(
        serialize_contacts("Windows", &contacts) == CONTACTS_C3,
        "c3 byte identity"
    );
}
