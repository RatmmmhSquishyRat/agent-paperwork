# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

The v0.5 perfection round (debt-closure batch). All CLI output contracts stay byte-frozen: golden envelope snapshots, frozen error wordings, JSON key names and key order (see `repos/paperwork-cli/tests/char_tests.rs`). Everything below is internal consolidation plus explicitly disclosed behavior additions — **zero behavior change for legitimate v0.5 files, except the one limitation explicitly disclosed under Fixed: the `--reply-to` implicit-mention sender lookup reuses the bounded tail-scan window, so a reply target beyond that window silently loses the implicit mention**. (Ultra Review F5 disclosure; the previous "zero behavior change" wording was overstated.)

### Changed (internal)

DRY consolidation across core and CLI, behavior-locked end to end:

- Shared fence scanner family in `format/mod.rs` (`for_each_outside_fence` / `first_outside_fence` / `collect_outside_fence`): 7 line-level fence sites converged to the shared scanner; 4 sites share the fence predicates by documented exemption (the 2 byte-level scans keep their byte loops; the 2 content collectors keep their own loops) — pinned by differential corpora (CRLF / indented fence / tilde / broken fence / lone-`\r`). Ultra Review F6: the count previously read "8 converged"; the predicate-sharing sites are exemptions, not scanner consumers — the per-site ruling is recorded in the closure ledger.
- Head-family regexes centralized in the format layer; the redundant `SEQ_RE` deleted in favor of the `header_seq` predicate
- ~31 seven-line `IoContext` boilerplate blocks collapsed onto the `io_ctx` helper; the 5 read-modify-write lock sequences unified behind the `LockedFile` RAII guard (no manual early-exit unlock paths left)
- `ops/thread.rs` split into `thread.rs` (send/edit orchestration) + `thread_read.rs` (read/summary) + `thread_scan.rs` (legacy guard / tail scans); the public re-export surface is unchanged
- 9 command-side hand-rolled JSON payloads converged onto the `output.rs` `JsonBuilder`. Key order is decided by `serde_json::Map` — backed by a `BTreeMap` (ALPHABETICAL order), because the `preserve_order` feature is NOT enabled — which is byte-identical to the pre-replacement output and pinned by the char_tests snapshots (Ultra Review F4: the previous "insertion order kept" wording described a mechanism that does not exist here; a unit test now pins alphabetical-not-insertion serialization)
- Single-pass `normalize_line_endings` with one top-level normalization and Cow hand-down (the validate path no longer re-normalizes 3–4 times)
- Streaming SHA-256 `hash_file` (64KB chunks; digest bit-identical to the one-shot form)
- `thread_edit` incremental last-message rewrite (byte-identical to the full rewrite, pinned by a differential corpus)
- Single-pass `hex_encode`
- `--reply-to` sender lookup via bounded reverse tail scan instead of a whole-file re-parse

### Added

Behavior additions (write-side guards and hardening; disclosed per the closure rules):

- Write-side injection guards: single-line fields (thread title, profile name/model, contacts title/label/path, brief title/owner) reject `\n` / `\r`; prose carrying a dangerous attribute-shaped line (`- model:` / `- owner:` / `- created:` / `- path:` / `- hash:` / `- regex:`) is rejected, because it would shadow the real structural attribute on re-parse
- Brief partial-migration residue guard: a brief that already uses lowercase keys but still carries the v0.4 `## Entries` wrapper heading or `### ` entry headers is refused at parse (and by `validate`) — it would otherwise parse silently and be destroyed by the next read-modify-write
- `profile create` / `brief create` / `contacts create` are atomic: `create_new` replaces the two-step `exists()` + write, closing the race window where two racing creators could both succeed
- `brief verify` surfaces genuine IO failures (permission denied, interrupted reads) as real `io` errors instead of collapsing them into `Stale`; a MISSING target stays `Stale` — that is the frozen spec three-state contract, not error swallowing
- `contacts read` enrichment uses the core two-level path resolver (entry path as given first, then relative to the contacts file's directory) shared with the write-side `derive_label` — no more drift between the two resolution paths
- `parse_contacts_title` is fence-aware: a pseudo-title inside a fenced code block is no longer adopted
- Write/read symmetry for the brief residue guard (Ultra Review F1): the write side now refuses exactly what the read-side guard refuses — a `## Entries` or `### ...` line OUTSIDE any code fence in brief description / entry-note prose (and profile description via the shared prose guard) is rejected as a reserved heading shape at Validation; occurrences INSIDE a fence remain legal quoted content. The parse-side residue message additionally names this self-produced cause
- Rust API additions (Ultra Review F8 disclosure; CLI binary behavior unchanged): `format::for_each_outside_fence`, `format::unclosed_fence`, `format::dedup_preserve_order`, `format::strip_title_suffix`, `format::strip_label_suffix`, `format::check_single_line`, `format::first_line_representation_issue`, `format::prose_representation_issue`, `format::contains_dangerous_attribute_line`, `format::contains_reserved_heading_shape`, `ops::contacts::resolve_contact_path`, `ops::profile::create_profile_full`, `ops::thread::find_message_sender`; the `thread.rs` split re-exports `thread_meta` / `thread_read` / `thread_summary` from `thread_read.rs` (names and signatures unchanged)

### Removed

- `PaperworkError::Io` dead variant (Rust API disclosure: a public enum variant of `paperwork-core` is removed). Verified before deletion: no construction site and no implicit `?` conversion depended on it — every IO site constructs `IoContext` explicitly, and the variant carried no pinned default wording. Downstream consumers matching `PaperworkError` exhaustively need to drop the arm.

### Fixed

- `ensure_suffix` no longer routes paths through `to_string_lossy`: non-UTF-8 file names survive suffix enforcement untouched (native `OsStr` concatenation)
- `profile create` with a scope section no longer opens and locks the target file twice — single write pass
- The reply-to whole-file re-parse is gone: the implicit-mention sender lookup is a bounded reverse tail scan. Two lock-topology facts, restated accurately (Ultra Review F5): `find_message_sender` takes and releases its OWN lock, then `thread_send` takes the lock again for the write — two independent acquisitions, NOT one already-held lock; and the lookup reuses the spec §5.5 64KB + 256B tail-scan window, so on a legitimately large thread a reply target beyond the window silently gets no implicit `@sender` mention (the send itself still succeeds; disclosed per the closure rules instead of moving the lookup inside the send lock)
- Degenerate suffix inputs no longer drift from spec (Ultra Review F2): default post title strips `.post.md` then `.md` (spec §5.7) and contacts label fallback strips `.profile.md` then `.md` (spec §7.3 R11) via two separate chains — the merged superset over-stripped `x.profile.md` as a post path (title `x` instead of `x.profile`) and `x.post.md` as a contacts entry (label `x` instead of `x.post`)

### Known downgrade re-check

The v0.4 legacy stance is unchanged: reading an unmigrated v0.4 file stays a silent empty result (`post read` -> 0 messages, `contacts read` -> empty list), and writing into one stays refused (`post send` / `contacts add` / `brief add` -> `error format:`). The perfection round added the brief residue guard to that same stance; nothing about the silent-empty-read / write-refusal position changed.

### Version

No version bump: this batch stays 0.5.0 pending the owner's release decision.

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
