# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added — write-side injection guardrails (P-2 batch)

- All create ops (`profile create`, `brief create`, `contacts create`) now create files atomically (`create_new`): two racing creators can no longer both succeed — exactly one wins, every loser receives the `already-exists` envelope, and the winner's bytes always survive.
- Single-line fields reject embedded line breaks with a `validation` envelope before anything touches disk: post preamble title, profile name/model, brief title/owner/entry path, contacts title/profile path. This closes structure-injection vectors (e.g. a title carrying `\n## injected (...)`).
- Preamble prose (profile / brief description) now refuses attribute-shaped lines with known structural keys (`- model:` / `- owner:` / `- created:` / `- path:` / `- hash:` / `- regex:`): such a line would shadow the real attribute on the next parse. Legal multi-line prose is unaffected.
- `contacts` title parsing is fence-aware: an H1 inside a code fence is quoted example content, never the file title.
- Briefs carrying legacy v0.4 residue (an `## Entries` wrapper heading or `### ` entry headers outside fences) are refused at parse time with a `format` envelope and a migration pointer, instead of being silently corrupted by the next write op. A `### ` line inside a note fence stays legal.
- `brief verify` distinguishes a missing entry target (stays `Stale` per the spec three-state contract) from a genuine read failure (permission denied etc.), which now surfaces as an `io` error envelope instead of collapsing into `Stale`.
- New core API `create_profile_full`: one-shot creation of a complete profile (including scopes) in a single atomic write.

### Removed — Rust API (P-4 batch, SAM-5)

- **Breaking for direct Rust consumers only (CLI output unchanged):** the dead `PaperworkError::Io(std::io::Error)` variant is removed from the public error enum. Every IO failure now surfaces as `PaperworkError::IoContext` with an explicit path, fix hint, and example — the bare variant had no reachable construction site left after the io-error envelope unification. Crate consumers matching `PaperworkError::Io` must migrate to the `IoContext` arm; `category()` still reports `"io"` for both historical shapes.

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

1. **post**: delete the `---` boundary lines; rewrite each message header `### #N sender · ts` as `## #N sender (ts)`; delete the seq #1 system message and lift its `[Thread created: X | participants: Y]` payload into the preamble as a bare `# X` H1 (the `- participants: Y` list is dropped — participants derive from the senders), renumbering the remaining messages consecutively from 1; after renumbering, remap every `@#N` reply token in body text to the target message's new seq (old seq numbers are stale once messages shift — e.g. if the old #3 becomes the new #2 and it replied to old #2 which became the new #1, rewrite `@#2` as `@#1`); convert every message attribute block into body-text tokens written on the same first line of the body as the original text: `- Reply-To: #N` -> `@#N`, `- Mentions: a,b` -> `@a @b` (space-separated, directly before the original body text — see the `@#1 @alice Agreed. Ship it.` line in the example above); drop `- To:` lines entirely — directed messages no longer exist as a structured concept, write `@name` in the body instead; replace fixed 4-backtick body fences with the minimal fence longer than any backtick run in the body (usually 3) and change the fence info `markdown` to `md`.
2. **profile**: lowercase `- Model:`; move `- Description:` text into a prose paragraph after the H1; rewrite the `## Scope` bullets as `- <permission>: <glob>` with bare globs (no backticks); delete `—` placeholder lines (empty scope = omit the section).
3. **brief**: lowercase `- Owner:` / `- Created:`; move `- Description:` text into prose after the H1; drop the `## Entries` wrapper and promote each entry H3 to H2; unwrap backticks from `- Path:` / `- Regex:`; write the hash in full (64 hex chars, no truncation); rewrite blockquote notes as bare prose paragraphs. Migration is not atomic — apply all steps to a brief file in one pass before running any write command (`brief add` / `brief remove`) on it, so no intermediate state is exposed to writers; do the structural moves (drop the `## Entries` wrapper, promote H3s) first, then lowercase the attribute keys.
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
- `contacts add` into an unmigrated v0.4 contacts file is **rejected** (`error format:`) instead of rewriting — v0.5 parsing ignores bare-path bullets, so the read-modify-rewrite would silently drop every existing entry. Migrate the file per the guide above first.
- Files written by early 0.5.0 builds that still carry `- reply-to:` / `- mentions:` attribute lines read fine: those attribute lines are ignored (references derive from the body `@` tokens only), and any rewrite operation (`post edit`) drops them.

### Rust API (paperwork-core) breaking

Compile-level breaks for library consumers (IDE plugins, agent harnesses, other tools embedding `paperwork-core`):

- `ops::thread::thread_send` signature changed: `(path, from, to, body, reply_to, mentions)` → `(path, from, body, preamble: Option<&ThreadMeta>)` (preamble written only when the file is empty inside the lock, §5.7), and it now rejects appending into unmigrated legacy threads (new `Parse` failure path).
- `ThreadMeta`: brand-new type in 0.5.0 (`ThreadMeta { title }`) — v0.4 had no thread-preamble type at all, so nothing was removed here.
- `ThreadSummary`: gained a `participants: Vec<String>` field (derived from the message sender set, first-appearance order) — struct-literal construction breaks at compile time, and `post summary --json` payloads carry a new `participants` key.
- `Message`: the `to` field is deleted; `reply_to` / `mentions` survive as parse-time derivations from body tokens (`@#N` / `@name`) and are never serialized back (Serialize/Deserialize shape change for `post read --json` payloads).
- `ContactEntry`: `contacts read --json` entries gain a new `label` key (the core `ContactEntry` serde shape replaces `summary` with `label`).
- `format::thread::serialize_thread` signature changed: `(&[Message])` → `(&ThreadMeta, &[Message])`.
- Removed public functions: `extract_bullet_key`, `parse_message_header`, `is_boundary_line`, `find_message_boundaries`, `parse_scope_globs`, `serialize_scope_globs`.

### Added

- `post send --title`: preamble (H1 title only) written atomically with the first message
- `post send --reply-to` / `--mention`: sugar flags injecting `@#N` / `@name` tokens at the body head; replies implicitly `@` the original sender
- `post summary`: title read from the preamble H1; participants derived from the message sender set
- `validate`: seq monotonicity (starts at 1, no gaps) and fence-closure checks wired in; suspected malformed message headers reported as warnings with a fix hint
- Local smoke corpus `test-v05/` in the new format (not tracked by git; see docs/dev/format-v2/impl_plan.md S5.1)

### Technical debt repaid

- **system message abolished** (debt #2): no more `[Thread created: …]` body-text encoding and string-split round trips; thread metadata lives in the preamble
- **`validate` deepened** (debt #1): `validate_seq_monotonicity` and fence closure were implemented since v0.3 but never wired to any command; both are live now, with error envelopes surfacing the real category (`format` vs `validation`)
- **`--to` closed out** (debt #3): `Message.to` existed in the format layer with no CLI entry point and serialized as `To: all` forever; the field is now deleted outright — directed messages no longer exist as a structured concept, and the `all` magic value is gone
- **Non-ASCII separators purged** (debt #4): the `·` (U+00B7) message-header separator — which also drifted from the `.` shown in docs — and the `—` placeholder are removed; every structural character is ASCII

### Also in the 0.5.0 line — CLI grammar redesign (superseded by v0.6, recorded as released fact)

The published 0.5.0 shipped the old (pre-format-v2) file formats together with a
positional-argument CLI grammar redesign. This subsection records that released
grammar as-is. **It is already superseded by the v0.6 named-flag grammar redesign
(in progress, not yet released): on the v0.6 working line every required value
below is a named flag again, and `post create` no longer exists (thread creation
is folded into the first `post send`). Do not migrate against this table on a
v0.6 build — use the named-flag grammar shown by `paperwork <command> --help`.**

Grammar as released in 0.5.0: `paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]` —
PATH is always the first positional argument; content is always last.

**Migration table (old flag -> new positional):**

| Command | v0.4 | v0.5 |
|---|---|---|
| `profile create` | `profile create <PATH> --name <NAME>` | `profile create <PATH> <NAME>` |
| `post create` | `post create <PATH> --title <TITLE>` | `post create <PATH> <TITLE>` |
| `post send` | `post send <PATH> --from <NAME> <BODY>` | `post send <PATH> <NAME> <BODY>` |
| `post edit` | `post edit <PATH> --seq <N> --from <NAME> <BODY>` | `post edit <PATH> <NAME> <SEQ> <BODY>` |
| `brief create` | `brief create <PATH> --title <TITLE>` | `brief create <PATH> <TITLE>` |
| `brief add` | `brief add <PATH> --entry <ENTRY>` | `brief add <PATH> <ENTRY>` |
| `brief remove` | `brief remove <PATH> --entry-title <TITLE>` | `brief remove <PATH> <ENTRY-TITLE>` |
| `contacts add` | `contacts add <PATH> --profile <PROFILE>` | `contacts add <PATH> <PROFILE-PATH>` |

Unchanged: `post read --from/--to` (seq range), `contacts create --title`
(optional flag with default), all other optional flags (`--model`, `--reply-to`,
`--mention`, `--limit`, `--stdin`, `--regex`, `--note`, `--owner`, scopes).

Path resolution is now three-stage: (1) the given path wins if it exists as a
file; (2) otherwise the type-suffixed variant is used if it exists; (3) if
neither exists, the suffixed path becomes the landing point — physical creation
still happens only in write commands (send/create/add); read-only commands
report not-found. Hitting an existing foreign (non-paperwork) file at stage 1
now reports `error format:` instead of silently appending (v0.4 behavior).

**Consumer-visible behavior changes beyond the argument layer (migration notes):**

1. `post read` field `showing: n/total` is now always emitted (previously only
   when the default limit was exceeded); `total` counts post-filter messages
   before the limit.
2. Usage errors (wrong invocation shape) still exit **2** — the exit code
   follows clap's default and is unchanged from v0.4. What changed: stderr
   now carries the structured `error usage:` envelope instead of clap's
   free-form text / usage synopsis, and `--json` usage errors are new
   (single-line JSON on stdout). Runtime errors keep exit 1.
3. Error category vocabulary gains a seventh category `usage` (existing six —
   `format`, `validation`, `io`, `not-found`, `already-exists`, `not-allowed` —
   unchanged).
4. Three additive output fields: `implicit-mention` (singular; `post send`
   only) — new output field surfacing the reply auto-mention behavior that
   already existed in v0.4 (replies auto-add the original sender to
   mentions); `window` (`post read`, `#first-#last` of the displayed range,
   absent for empty threads); and `command` inside `--json` error objects.
5. `post create` on an existing thread no longer silently appends: v0.4
   appended a new thread-creation message with exit 0; v0.5 reports
   `already-exists` with exit 1. Send to the existing thread instead.

**Also added in the 0.5.0 CLI redesign:**

- `error usage:` envelope (exit 2) for all clap-level parse failures, carrying
  `fix:` and a canonical copy-paste `example:`; `--help`/`-V` keep exit 0
- `--json` usage errors: single-line JSON on stdout with
  `category:"usage"`, `command`, `example`, `exit_code:2`
- `command` field in `--json` runtime error objects (additive)
- `implicit-mention` field: replies auto-add the original sender to mentions
- `post read`: always-on `showing:` and `window:` fields
- `validate --type post|profile|brief|contacts`: explicit parser selection,
  overriding suffix inference
- `post` hidden alias `po` (does not appear in `--help`)
- Bodies starting with `-` supported via the `--` boundary
- `SKILL.md`: agent-oriented grammar cheat sheet with per-tool examples and
  error self-healing hints
- English `after_help` examples on every subcommand (including `--` teaching)

### Deprecated

- None.

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
