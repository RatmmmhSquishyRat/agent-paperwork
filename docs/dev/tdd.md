# Agent Paperwork — TDD Plan

## Test Philosophy

- **Red-Green-Refactor**: Write failing test → minimal impl → refactor
- **Layered**: format unit tests → ops integration tests → CLI e2e tests
- **Fixture-driven**: Golden Markdown files for parse/serialize roundtrip validation
- **No mocks for filesystem**: use `tempfile` crate for real FS operations

---

## Layer 1: Format Parsing & Serialization (Unit Tests)

### 1.1 Profile Format

| Test | Input | Expected |
|------|-------|----------|
| `parse_profile_basic` | Valid profile .md | Profile struct with all fields |
| `parse_profile_empty_scope` | Profile with `—` scopes | Empty Vec for scope fields |
| `parse_profile_multi_glob` | Multiple comma-separated globs | Vec with all patterns |
| `serialize_profile_roundtrip` | Profile struct | Serialize → parse → identical struct |
| `parse_profile_invalid_no_h1` | Missing `# name` | Error: "missing agent name heading" |

### 1.2 Thread Format

| Test | Input | Expected |
|------|-------|----------|
| `parse_single_message` | One message block | Message with seq=1, correct fields |
| `parse_multi_message` | 3 message blocks | Vec of 3 Messages, ordered |
| `parse_message_with_reply` | Reply-To: #1 | reply_to = Some(1) |
| `parse_message_no_reply` | Reply-To: — | reply_to = None |
| `parse_message_multiline_body` | Body with 5 lines | Full body preserved |
| `serialize_message_roundtrip` | Message struct | Serialize → parse → identical |
| `parse_empty_thread` | Empty string | Empty Vec |
| `seq_monotonicity_check` | Messages 1,2,3 | Passes validation |
| `seq_gap_detection` | Messages 1,3 (gap) | Warning/error |

### 1.3 Manifest Format

| Test | Input | Expected |
|------|-------|----------|
| `parse_manifest_entry_full` | Entry with path, hash, regex, groups, note | Complete ManifestEntry |
| `parse_manifest_no_regex` | Entry with `—` for regex | regex = None |
| `parse_manifest_multi_entry` | 3 entries | Vec of 3 |
| `serialize_manifest_roundtrip` | Manifest struct | Serialize → parse → identical |

### 1.4 Contacts Format

| Test | Input | Expected |
|------|-------|----------|
| `parse_contacts_table` | Valid table with 3 rows | Vec of ContactEntry |
| `serialize_contacts_roundtrip` | Contacts struct | Serialize → parse → identical |

### 1.5 Notification Format

| Test | Input | Expected |
|------|-------|----------|
| `parse_notification_entry` | Valid notification block | Notification struct |
| `parse_empty_notifications` | Header only, no entries | Empty Vec |

---

## Layer 2: Operations (Integration Tests with tempdir)

### 2.1 Init & Layout

| Test | Setup | Action | Assert |
|------|-------|--------|--------|
| `init_creates_skeleton` | Empty tempdir | `init(root, "alice")` | All dirs + files exist |
| `init_idempotent` | Initialized dir | `init(root, "alice")` again | No error, no changes |
| `init_second_agent` | alice initialized | `init(root, "bob")` | bob profile added, alice unchanged |

### 2.2 Profile Ops

| Test | Setup | Action | Assert |
|------|-------|--------|--------|
| `create_profile_writes_file` | Initialized | `create_profile(bob)` | File exists, contacts updated |
| `edit_profile_scope` | bob exists | `edit_profile(bob, scope)` | File updated correctly |
| `list_profiles` | alice + bob | `list_profiles()` | Returns both |

### 2.3 Thread Ops

| Test | Setup | Action | Assert |
|------|-------|--------|--------|
| `append_first_message` | Empty thread | `append_msg(seq=1)` | File has one message |
| `append_increments_seq` | Thread with msg 1 | `append_msg()` | New msg has seq=2 |
| `read_range_subset` | Thread with 5 msgs | `read_range(2, 4)` | Returns msgs 2,3,4 |
| `read_range_all` | Thread with 3 msgs | `read_range(1, 3)` | Returns all 3 |
| `summary_correct` | Thread with msgs | `summary()` | Count, last sender correct |
| `concurrent_append_safety` | Empty thread | 10 parallel appends | All 10 present, no interleaving |
| `self_edit_own_message` | alice sent msg #3 | `edit_msg(seq=3, sender="alice", new_body)` | Body updated, metadata unchanged |
| `self_edit_preserves_seq` | alice sent msg #3 | `edit_msg(seq=3, ...)` | Seq, sender, timestamp unchanged |
| `self_edit_rejects_other_sender` | alice sent msg #3 | `edit_msg(seq=3, sender="bob", ...)` | Error: not message owner |
| `self_edit_only_last_own` | alice sent #1,#3; bob sent #2 | `edit_msg(seq=1, sender="alice", ...)` | Error: not last own message |

### 2.4 Manifest Ops

| Test | Setup | Action | Assert |
|------|-------|--------|--------|
| `create_manifest` | Initialized | `create_manifest("onboarding")` | File exists with header |
| `add_entry_computes_hash` | File exists at path | `add_entry(path, regex)` | Hash matches actual file |
| `verify_fresh` | Unchanged file | `verify()` | Returns Fresh |
| `verify_stale` | Regex won't match | `verify()` | Returns Stale |
| `verify_shifted` | File modified, regex matches | `verify()` | Returns Shifted |

### 2.5 Notification Ops

| Test | Setup | Action | Assert |
|------|-------|--------|--------|
| `push_notification` | Agent exists | `push_notify(bob, notif)` | unread.md has entry |
| `ack_moves_to_history` | 2 unread | `ack_notify(bob)` | unread empty, history has 2 |

### 2.6 Who Query

| Test | Setup | Action | Assert |
|------|-------|--------|--------|
| `who_owns_match` | alice owns src/** | `who("src/main.rs", Owns)` | Returns alice |
| `who_owns_no_match` | alice owns src/** | `who("docs/readme.md", Owns)` | Returns empty |
| `who_multi_match` | alice+bob own src/** | `who("src/lib.rs", Owns)` | Returns both |
| `who_reads_match` | alice reads docs/** | `who("docs/guide.md", Read)` | Returns alice |
| `who_writes_match` | bob writes src/lexer/** | `who("src/lexer/token.rs", Write)` | Returns bob |

### 2.7 Invite & Contacts Ops

| Test | Setup | Action | Assert |
|------|-------|--------|--------|
| `invite_creates_profile_and_dm` | alice initialized | `invite("bob")` | bob profile exists, dm/alice--bob/ exists with meta.md + thread.md |
| `invite_dm_folder_alphabetical` | zara initialized | `invite("alice")` | Folder is dm/alice--zara/ (sorted) |
| `invite_updates_contacts` | alice initialized | `invite("bob")` | contacts.md includes bob row |
| `contacts_list` | alice + bob exist | `list_contacts()` | Returns both entries with profile paths |

---

## Layer 3: CLI End-to-End Tests

| Test | Command | Assert |
|------|---------|--------|
| `cli_init` | `paperwork init --name test` | Exit 0, .paperwork/ exists |
| `cli_init_idempotent` | `paperwork init --name test` twice | No error, no changes |
| `cli_profile_create_show` | create then show | Output contains name + model |
| `cli_profile_list` | create 2 agents, list | Both names in output |
| `cli_invite` | `paperwork invite bob` | Profile + DM folder created |
| `cli_contacts` | after invite | Table with both agents |
| `cli_dm_send_read` | send then read | Output contains message body |
| `cli_dm_reply` | send --reply-to 1 | Reply-To: #1 in output |
| `cli_dm_mention_notify` | send --mention bob | notifications/bob/unread.md has entry |
| `cli_dm_summary` | send 3 msgs, summary | Count=3, last sender shown |
| `cli_dm_self_edit` | send, then edit seq | Body updated, metadata same |
| `cli_post_lifecycle` | create → send → summary | Summary shows correct count |
| `cli_post_read_range` | send 5, read --from 2 --to 4 | Only msgs 2-4 shown |
| `cli_post_list` | create 2 posts, list | Both names in output |
| `cli_manifest_verify` | create → add → verify | Output shows "Fresh" |
| `cli_manifest_remove` | add entry, remove it | Entry gone from file |
| `cli_manifest_list` | create 2, list | Both names in output |
| `cli_who_owns` | set scope, who --owns | Correct agent shown |
| `cli_who_reads_writes` | set scope, who --reads/--writes | Correct agent shown |
| `cli_notify_ack` | mention, then notify --ack | unread empty, history has entry |
| `cli_json_output` | Any command + `--json` | Valid JSON parseable |
| `cli_error_actionable` | `paperwork dm nobody send "x"` | Error message suggests fix |

---

## Test Infrastructure

```rust
// Shared test helper
fn setup_workspace() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    paperwork_core::ops::init(&root, "test-agent").unwrap();
    (dir, root)
}
```

## Coverage Target

- Format layer: 100% branch coverage (all parse paths)
- Ops layer: all happy paths + key error paths
- CLI layer: smoke tests for every subcommand
- Concurrency: dedicated stress test (not in CI critical path)
