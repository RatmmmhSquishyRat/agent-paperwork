# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.5.0] - 2026-08-09

### Changed (Breaking)

CLI grammar redesign: required values are positional; optional values stay flags.
Grammar: `paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]` —
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

### Added

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
