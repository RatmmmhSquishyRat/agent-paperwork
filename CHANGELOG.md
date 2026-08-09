# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.5.0] - 2026-08-09

### Changed (Breaking) — Format Renewal

Hard breaking rewrite of all four managed file formats on pure CommonMark-native constructs: H1 identity + prose description, lowercase `- key: value` attribute lines, dynamic-length message fences, Markdown-link references. All non-ASCII structural characters are removed. Old constructs (`---` message boundaries, `·` header separators, fixed 4-backtick fences, `—` placeholders, `all` magic value, system messages, capitalized attribute keys) receive no parsing support; **there is no `migrate` command** — migrate by hand per the guide below.

The finalized post spec additionally abolishes all persisted reference state: the post preamble is the H1 title only (no `- participants:` line — participants derive from the message sender set at read time), messages carry no attribute lines (`- reply-to:` / `- mentions:` / `- to:` deleted; replies are `@#N` and mentions are `@name` body-text tokens, re-derived on every read), and body fences use the `md` info string on write (the parser still accepts legacy `markdown`).

Command surface: `post create` is deleted (the preamble is written by the first `post send --title` inside the same lock); `post send --to` and `--participants` are deleted; `--reply-to N` / `--mention a,b` remain as sugar that injects `@#N` / `@name` tokens at the head of the body; `validate` now enforces seq continuity and fence closure.

**post** — before:

`````markdown
---

### #1 system · 2026-08-01T19:38:03Z

- To: all

````markdown
[Thread created: Daily Standup | participants: alice, bob]
````

---

### #2 alice · 2026-08-01T19:38:22Z

- To: all

````markdown
Parser module is 80% done.
````

---

### #3 bob · 2026-08-01T19:40:10Z

- Reply-To: #2
- Mentions: alice

````markdown
Agreed. Ship it.
````
`````

after:

`````markdown
# Daily Standup

## #1 alice (2026-08-01T19:38:22Z)

```md
Parser module is 80% done.
```

## #2 bob (2026-08-01T19:40:10Z)

```md
@#1 @alice Agreed. Ship it.
```
`````

**profile** — before:

`````markdown
# alice

- Model: gpt-4o
- Description: Parser module implementer

## Scope

- Read: `src/**`
- Owns: `src/parser/**`
`````

after:

`````markdown
# alice

Parser module implementer

- model: gpt-4o

## Scope

- read: src/**
- owns: src/parser/**
`````

**brief** — before:

`````markdown
# Codebase Onboarding

- Owner: alice
- Created: 2026-08-01T19:40:36Z
- Description: How to understand this project

## Entries

### main.rs

- Path: `src/main.rs`
- Hash: `42b6647…`
- Regex: `fn main`

> Entry point
`````

after:

`````markdown
# Codebase Onboarding

How to understand this project

- owner: alice
- created: 2026-08-01T19:40:36Z

## main.rs

- path: src/main.rs
- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540
- regex: fn main

Entry point
`````

**contacts** — before:

`````markdown
# Core Team

- agents/alice.profile.md
- agents/bob.profile.md
`````

after:

`````markdown
# Core Team

- [alice](agents/alice.profile.md)
- [bob](agents/bob.profile.md)
`````

### Migration guide (manual)

1. **post**: delete the `---` boundary lines; rewrite each message header `### #N sender · ts` as `## #N sender (ts)`; delete the seq #1 system message and lift its `[Thread created: X | participants: Y]` payload into the preamble as a bare `# X` H1 (the `- participants: Y` list is dropped — participants derive from the senders), renumbering the remaining messages consecutively from 1; convert every message attribute block into body-text tokens on the first body line: `- Reply-To: #N` -> `@#N`, `- Mentions: a,b` -> `@a @b` (space-separated, then a blank line before the original body); drop `- To:` lines entirely — directed messages no longer exist as a structured concept, write `@name` in the body instead; replace fixed 4-backtick body fences with the minimal fence longer than any backtick run in the body (usually 3) and change the fence info `markdown` to `md`.
2. **profile**: lowercase `- Model:`; move `- Description:` text into a prose paragraph after the H1; rewrite the `## Scope` bullets as `- <permission>: <glob>` with bare globs (no backticks); delete `—` placeholder lines (empty scope = omit the section).
3. **brief**: lowercase `- Owner:` / `- Created:`; move `- Description:` text into prose after the H1; drop the `## Entries` wrapper and promote each entry H3 to H2; unwrap backticks from `- Path:` / `- Regex:`; write the hash in full (64 hex chars, no truncation); rewrite blockquote notes as bare prose paragraphs.
4. **contacts**: turn each bare-path bullet into a Markdown link `- [name](path)`.

Known downgrade after the break (expected, not a bug): old-format profiles (capitalized `- Model:` key) parse as missing their model, so `profile list` shows them as `(unreadable)` and `contacts read` skips their enrichment. Convert the file as above to restore it.

Behavior contract changes worth knowing before upgrading:

- `validate` now rejects empty `.post.md` files (`error format:`, exit 1); v0.4 passed them (`ok`).
- `post send --to` and `--participants` no longer exist; replies and mentions are written into the body as `@#N` / `@name` tokens (also what `--reply-to` / `--mention` now do), and `post read --mention` / `--reply-to` filters match those derived tokens instead of stored fields.
- `post summary` derives participants from the message sender set (first-appearance order) instead of reading a `- participants:` line; the title still comes from the preamble H1.
- Body fences are written with the `md` info string; files still carrying ` ```markdown ` fences keep parsing fine (lenient read, strict write).
- `post send --from` sender tokens are validated on the write side: non-empty, no whitespace, no `(`/`)`. Multi-word display names accepted by v0.4 are now rejected (`error validation:`).
- `brief read --full` prints the full 64-hex-char hash on stdout (the body-line `(hash: …)` was truncated to 12 chars in v0.4).
- Reading an unmigrated v0.4 file is a **silent empty result**, not an error: `post read` returns `ok` with 0 messages, `contacts read` returns an empty list (old constructs receive no parsing support). Use `paperwork validate <file>` as the machine check for migration completeness — it reports `error format:` until the file is fully migrated.
- `post send` into an unmigrated v0.4 thread is **rejected** (`error format:`) instead of appending — appending would silently produce a mixed-format corrupt file. Migrate the file per the guide above first.
- Files written by early 0.5.0 builds that still carry `- reply-to:` / `- mentions:` attribute lines read fine: those attribute lines are ignored (references derive from the body `@` tokens only), and any rewrite operation (`post edit`) drops them.

### Rust API (paperwork-core) breaking

Compile-level breaks for library consumers (IDE plugins, agent harnesses, other tools embedding `paperwork-core`):

- `ops::thread::thread_send` signature changed: `(path, from, to, body, reply_to, mentions)` → `(path, from, body, preamble: Option<&ThreadMeta>)` (preamble written only when the file is empty inside the lock, §5.7), and it now rejects appending into unmigrated legacy threads (new `Parse` failure path).
- `ThreadMeta`: brand-new type in 0.5.0 (`ThreadMeta { title }`) — v0.4 had no thread-preamble type at all, so nothing was removed here.
- `ThreadSummary`: gained a `participants: Vec<String>` field (derived from the message sender set, first-appearance order) — struct-literal construction breaks at compile time, and `post summary --json` payloads carry a new `participants` key.
- `Message`: the `to` field is deleted; `reply_to` / `mentions` survive as parse-time derivations from body tokens (`@#N` / `@name`) and are never serialized back (Serialize/Deserialize shape change for `post read --json` payloads).
- `ContactEntry`: the `summary` field is removed and replaced by `label` (Serialize/Deserialize shape change; `contacts read --json` entries carry `label` instead of `summary`).
- `format::thread::serialize_thread` signature changed: `(&[Message])` → `(&ThreadMeta, &[Message])`.
- Removed public functions: `extract_bullet_key`, `parse_message_header`, `is_boundary_line`, `find_message_boundaries`, `parse_scope_globs`, `serialize_scope_globs`.

### Added

- `post send --title`: preamble (H1 title only) written atomically with the first message
- `post send --reply-to` / `--mention`: sugar flags injecting `@#N` / `@name` tokens at the body head; replies implicitly `@` the original sender
- `post summary`: title read from the preamble H1; participants derived from the message sender set
- `validate`: seq monotonicity (starts at 1, no gaps) and fence-closure checks wired in; suspected malformed message headers reported as warnings with a fix hint
- New smoke corpus `test-v05/` in the new format

### Technical debt repaid

- **system message abolished** (debt #2): no more `[Thread created: …]` body-text encoding and string-split round trips; thread metadata lives in the preamble
- **`validate` deepened** (debt #1): `validate_seq_monotonicity` and fence closure were implemented since v0.3 but never wired to any command; both are live now, with error envelopes surfacing the real category (`format` vs `validation`)
- **`--to` closed out** (debt #3): `Message.to` existed in the format layer with no CLI entry point and serialized as `To: all` forever; the field is now deleted outright — directed messages no longer exist as a structured concept, and the `all` magic value is gone
- **Non-ASCII separators purged** (debt #4): the `·` (U+00B7) message-header separator — which also drifted from the `.` shown in docs — and the `—` placeholder are removed; every structural character is ASCII

## [0.4.0] - 2026-08-01

### Changed (Breaking)

- All CLI output redesigned: unified envelope protocol (`ok`/`error` + `key: value` fields)
- Pure ASCII output — removed all Unicode symbols and ANSI codes
- `--json` output wrapped with `"status"`, `"command"`, `"conclusion"` fields
- `-q` now suppresses only the status line; fields and body still output

### Added

- `--stdin` flag for `post send` and `post edit` (pipe multi-line content)
- `post read`: default limit 20 messages with `showing:` field
- `post read`: timestamps and inline metadata in default mode
- `validate`: actual format parsing by file type suffix, rejects invalid files
- `contacts read`: enriches output with profile name + description
- `profile list`: structured output with name and model per entry
- All errors carry `fix:` and `example:` fields for one-retry self-correction
- Empty message body rejection with actionable error

### Fixed

- `post summary`: clean title extraction from system message
- `brief read --full`: shows hash, regex, note per entry
- `post read --plain`: respects `--from`/`--to` range filtering
- `--mention` flag: uses `value_delimiter` instead of greedy `num_args`

## [0.3.0] - 2026-08-01

### Changed (Breaking)

- File naming: auto-appends type suffix (`.profile.md`, `.post.md`, `.brief.md`, `.contacts.md`)
- File format: bullet-list metadata (`- Key: value`) replaces bold-key (`**Key**: value`)
- Message bodies wrapped in 4-backtick fenced code blocks (`````markdown`)
- Fence-aware boundary detection (`---` inside fence ignored)

### Added

- `paperwork validate <path>` command
- `--mention` comma-separated flag for `post send`

## [0.2.0] - 2026-07-30

### Changed (Breaking)

- Complete architecture rewrite: stateless, path-explicit, no workspace
- Removed: `init`, `invite`, `dm`, `notify`, `layout`, `.paperwork/` folder
- Unified `post` as sole communication primitive (replaces DM + GDM)
- @mention and reply-to are fields/filters within `post read`

### Added

- `brief` command (renamed from `manifest`)
- `contacts` command
- File locking (fs2) for concurrent append safety
- `--json` and `--plain` output modes

## [0.1.0] - 2026-07-29

Initial implementation (superseded by v0.2 architecture correction).
