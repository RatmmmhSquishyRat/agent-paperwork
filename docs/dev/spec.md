# Agent Paperwork — Technical Specification

## 1. Scope

This spec defines the complete behavior of:
- **paperwork-core**: Rust library crate — file format, parsing, operations, validation
- **paperwork-cli**: Rust binary crate — command interface, UX, output formatting

Governed by: `docs/ssot/adr/初版技术选型.md`, `docs/ssot/pillars/paperwork-init-conversation/`, ADR-001…010.

---

## 2. Managed File Format (Richly-Marked Markdown)

All managed files use `.md` extension with rich Markdown markup. Parsing is regex/heading-based, not YAML/JSON.

### 2.1 Profile (`profiles/<name>.md`)

```markdown
# <name>

**Model**: <model-id>  
**Description**: <free-text>

## Scope

**Read**: `<glob>`, `<glob>`, ...  
**Write**: `<glob>`, `<glob>`, ...  
**Owns**: `<glob>`, `<glob>`, ...
```

**Parsing rules**:
- H1 (`# `) → agent name (must match filename stem)
- `**Model**: ` line → model identifier (free-form string)
- `**Description**: ` line → description (rest of line)
- Under `## Scope`: `**Read**: `, `**Write**: `, `**Owns**: ` lines
- Scope values: comma-separated backtick-quoted glob patterns
- Empty scope line (`**Read**: —`) means no declared scope

### 2.2 Contacts (`contacts.md`)

```markdown
# Contacts

| Agent | Profile |
|-------|--------|
| alice | profiles/alice.md |
| bob | profiles/bob.md |
```

**Parsing rules**:
- Markdown table with fixed columns: Agent, Profile
- One row per registered agent
- DM paths are NOT stored; they are derived via ADR-008 convention: `dm/<sorted-a>--<sorted-b>/`
- To find the DM folder between any two agents, sort their names alphabetically and join with `--`

### 2.3 DM Thread (`dm/<pair>/thread.md`)

```markdown
---

### #<seq> — <sender> · <ISO-8601>

**To**: <recipient>  
**Reply-To**: #<seq> | —

<body: free-form Markdown, multi-line>

---
```

**Parsing rules**:
- **Message boundary**: `---` on its own line **immediately followed** (within 2 lines) by a valid H3 header matching `### #\d+ — .+ · .+`. A lone `---` NOT followed by this pattern is body content, not a boundary.
- Header: `### #<seq> — <sender> · <timestamp>` (H3)
- Metadata lines: `**To**: `, `**Reply-To**: ` (bold key pattern)
- Body: everything between metadata block and next valid message boundary
- Seq: monotonically increasing integer, starts at 1
- Timestamp: ISO-8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`)
- Reply-To: `#<seq>` reference or `—` (em dash) for none
- **Body safety**: Bodies may contain `---`, H3 headings, and bold text freely. Only the exact boundary+header pair triggers a split.
- **Concurrency**: Append operations use advisory file locking (`flock`/`LockFileEx`) around read-seq + write to prevent seq collision.

### 2.4 DM Meta (`dm/<pair>/meta.md`)

```markdown
# DM: <agent-a> ↔ <agent-b>

**Created**: <ISO-8601>  
**Participants**: <agent-a>, <agent-b>
```

### 2.5 Post/GDM Log (`posts/<name>/log.md`)

Same format as DM thread (§2.3), except:
- `**To**: ` may list multiple recipients (comma-separated) or the literal `all` (meaning all participants)
- Multiple senders appear in sequence
- In the `Message` struct, `to: Vec<String>` — an empty Vec serializes as `all`

### 2.6 Post Meta (`posts/<name>/meta.md`)

```markdown
# Post: <name>

**Created**: <ISO-8601>  
**Participants**: <agent-a>, <agent-b>, <agent-c>  
**Title**: <human-readable title>
```

### 2.7 Manifest (`manifests/<name>.md`)

```markdown
# Manifest: <name>

**Author**: <agent>  
**Created**: <ISO-8601>  
**Description**: <what this manifest helps you understand>

## Entries

### <entry-title>

**Path**: `<relative-path-or-glob>`  
**Hash**: `<sha256-hex>`  
**Regex**: `<pattern>` | —  
**Groups**: <group1>, <group2> | —

> Optional note about why this entry matters.

---
```

**Parsing rules**:
- Each entry is an H3 section under `## Entries`
- `**Path**: ` — relative path or glob pattern (backtick-quoted)
- `**Hash**: ` — SHA-256 hex digest of the file blob at curation time
- `**Regex**: ` — optional extraction pattern stored in a fenced code block (```regex ... ```) to handle all special characters; `—` if none
- `**Groups**: ` — derived automatically from regex named captures at parse time; `—` if regex is absent
- Blockquote (`>`) = optional human note

### 2.8 Notifications (`notifications/<name>/unread.md`, `history.md`)

```markdown
# Notifications: <name>

---

### <ISO-8601> — from <sender>

**In**: <thread-path>  
**Seq**: #<seq>  
**Type**: mention | reply

> <snippet of the triggering message>

---
```

---

## 3. Core Library API (`paperwork-core`)

### 3.1 Module Structure

```
paperwork-core/src/
├── lib.rs
├── format/
│   ├── mod.rs          # format trait + shared parsing utils
│   ├── profile.rs      # profile parse/serialize
│   ├── contacts.rs     # contacts table parse/serialize
│   ├── thread.rs       # DM/GDM message parse/serialize/append
│   ├── manifest.rs     # manifest parse/serialize
│   └── notification.rs # notification parse/serialize
├── ops/
│   ├── mod.rs
│   ├── profile.rs      # create/edit/show/list profiles
│   ├── contacts.rs     # invite, list, who-queries
│   ├── thread.rs       # send, read-range, summary
│   ├── manifest.rs     # create, edit, read, verify
│   └── notify.rs       # push notification, list unread, ack
├── layout.rs           # .paperwork/ directory structure management
├── hash.rs             # SHA-256 blob hashing
└── error.rs            # unified error type
```

### 3.2 Key Types

```rust
/// Glob pattern for scope declarations. Validated on parse.
pub type GlobPattern = String;

pub struct Profile {
    pub name: String,
    pub model: String,
    pub description: String,
    pub scope_read: Vec<GlobPattern>,
    pub scope_write: Vec<GlobPattern>,
    pub scope_owns: Vec<GlobPattern>,
}

pub struct ContactEntry {
    pub agent: String,
    pub profile_path: String,  // relative to .paperwork/
}

pub struct Message {
    pub seq: u64,
    pub sender: String,
    pub timestamp: DateTime<Utc>,
    pub to: Vec<String>,       // empty Vec = "all" (post broadcast)
    pub reply_to: Option<u64>,
    pub body: String,
}

pub struct ThreadSummary {
    pub thread_path: String,
    pub message_count: u64,
    pub last_sender: Option<String>,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub snippets: Vec<String>,  // preview of recent messages
}

pub struct ManifestEntry {
    pub title: String,
    pub path: String,       // glob or relative path
    pub hash: String,       // SHA-256 hex
    pub regex: Option<String>,
    pub groups: Vec<String>,
    pub note: Option<String>,
}

pub enum VerifyResult {
    Fresh,    // regex matches + hash matches (or no regex + hash matches)
    Shifted,  // regex matches + hash differs (or no regex + hash differs)
    Stale,    // regex fails
}

pub struct Notification {
    pub timestamp: DateTime<Utc>,
    pub from: String,
    pub thread_path: String,  // relative path to thread/log file
    pub seq: u64,
    pub notify_type: NotifyType,
    pub snippet: String,
}

pub enum NotifyType {
    Mention,
    Reply,
}

pub enum Access {
    Owns,
    Read,
    Write,
}
```

### 3.3 Core Operations

| Operation | Signature | Semantics |
|-----------|-----------|-----------|
| `init` | `init(root: &Path, name: &str, model: &str) -> Result<()>` | Create `.paperwork/` skeleton + initial profile |
| `profile_create` | `create_profile(root: &Path, p: &Profile) -> Result<()>` | Write profile .md, update contacts |
| `profile_parse` | `parse_profile(content: &str) -> Result<Profile>` | Parse richly-marked Markdown |
| `thread_append` | `append_msg(root: &Path, thread: &str, msg: &Message) -> Result<()>` | Atomic append (O_APPEND, single write) |
| `thread_read_range` | `read_range(root: &Path, thread: &str, from: u64, to: u64) -> Result<Vec<Message>>` | Parse and filter by seq |
| `thread_summary` | `summary(root: &Path, thread: &str) -> Result<ThreadSummary>` | Count, last update, snippet previews |
| `thread_self_edit` | `edit_msg(root: &Path, thread: &str, seq: u64, sender: &str, new_body: &str) -> Result<()>` | In-place rewrite of own message body only |
| `manifest_verify` | `verify(root: &Path, name: &str) -> Result<Vec<(ManifestEntry, VerifyResult)>>` | Check regex + hash per entry |
| `who_owns` | `who(root: &Path, pattern: &str, access: Access) -> Result<Vec<Profile>>` | Scan profiles for scope match |
| `notify_push` | `push_notify(root: &Path, target: &str, n: &Notification) -> Result<()>` | Append to unread.md |
| `notify_ack` | `ack_notify(root: &Path, agent: &str) -> Result<Vec<Notification>>` | Move unread → history |

### 3.4 Atomic Append Contract

- Advisory file lock acquired (`fs2::FileExt::lock_exclusive()`) before any thread mutation
- Last seq determined via reverse-scan of final 64KB (O(1) regardless of file size)
- File opened with `O_APPEND`; entire serialized message written in **one** `write()` syscall
- Lock released after write completes
- Message size MUST be < 64KB (hard limit; typical < 4KB)
- Same locking protocol applies to `self_edit` and notification operations

---

## 4. CLI Command Structure (`paperwork-cli`)

### 4.1 Command Tree

```
paperwork
├── init [--name <agent>] [--model <id>] [--scope <spec>]
├── profile
│   ├── create <name> [--model <id>] [--description <text>] [--scope <spec>]
│   ├── edit <name> [--model | --description | --scope-read | --scope-write | --scope-owns]
│   ├── show <name>
│   └── list
├── invite <name> [--model <id>]
├── contacts
├── who --owns|--reads|--writes <glob>
├── dm <agent>
│   ├── send <body> [--mention <agent>] [--reply-to <seq>]
│   ├── read [--from <seq>] [--to <seq>]
│   ├── edit <seq> <new-body>   # self-edit own message only
│   └── summary
├── post
│   ├── create <name> --participants <a,b,c> [--title <text>]
│   ├── <name> send <body> [--mention <agent>] [--reply-to <seq>]
│   ├── <name> read [--from <seq>] [--to <seq>]
│   ├── <name> summary
│   └── list
├── manifest
│   ├── create <name> [--description <text>]
│   ├── <name> add --path <p> [--regex <r>] [--groups <g>] [--note <n>]
│   ├── <name> remove <entry-title>
│   ├── <name> read [--full | --selective]
│   ├── <name> verify
│   └── list
└── notify
    ├── [--agent <name>]
    └── --ack
```

### 4.2 UX Principles

1. **Stub-first**: Every `create` produces a working minimal artifact immediately
2. **Progressive refinement**: `edit` / `add` commands enhance stubs incrementally
3. **Readable output**: CLI output mirrors the Markdown format (rich terminal rendering)
4. **Agent-friendly**: All commands support `--json` for machine consumption
5. **No interactive prompts**: Every command is fully non-interactive (agent-safe)
6. **Error clarity**: Errors state what went wrong + what to do next

### 4.3 Output Modes

| Flag | Behavior |
|------|----------|
| (default) | Rich Markdown rendering to terminal |
| `--json` | Structured JSON output for machine parsing |
| `--plain` | Raw file content (cat-equivalent) |

---

## 5. Invariants & Constraints

| ID | Invariant |
|----|-----------|
| I1 | Seq numbers are monotonically increasing per thread, no gaps. File locking guarantees no collision under concurrent writes |
| I2 | Append-only: no insert, no delete, no reorder of messages |
| I3 | Self-edit: sender may update body of their own most recent message, ONLY if it is the final message in the thread. Requires file lock. In-place rewrite of that block only |
| I4 | All timestamps are UTC ISO-8601 |
| I5 | DM pair folder name = alphabetically sorted names joined by `--` |
| I6 | Profile name must match filename stem |
| I7 | Manifest hash is SHA-256 of raw file bytes at curation time |
| I8 | All managed files are valid Markdown (parseable by commonmark) |
| I9 | CLI never enforces scope; scope is advisory only |
| I10 | No network access; all operations are local filesystem |
| I11 | Canonical line ending is LF (`\n`). Parser normalizes CRLF→LF on read; serializer always emits LF. `.paperwork/.gitattributes` enforces `eol=lf` |
| I12 | Message boundary = `---` + valid H3 header pair. Lone `---` in body is content, not delimiter |

---

## 6. Repository Layout

```
repos/
├── paperwork-core/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── format/
│   │   ├── ops/
│   │   ├── layout.rs
│   │   ├── hash.rs
│   │   └── error.rs
│   └── tests/
│       ├── format_tests.rs
│       ├── ops_tests.rs
│       └── fixtures/
└── paperwork-cli/
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs
    │   ├── cmd/
    │   │   ├── mod.rs
    │   │   ├── init.rs
    │   │   ├── profile.rs
    │   │   ├── invite.rs
    │   │   ├── contacts.rs
    │   │   ├── who.rs
    │   │   ├── dm.rs
    │   │   ├── post.rs
    │   │   ├── manifest.rs
    │   │   └── notify.rs
    │   └── output.rs
    └── tests/
        └── cli_integration.rs
```
