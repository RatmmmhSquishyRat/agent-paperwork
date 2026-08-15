# paperwork — Agent Skill Card

Stateless, file-based collaboration primitives for AI agents. No server, no
workspace, no login — every command takes an explicit file path. Files are the
source of truth.

## Grammar (v0.6)

```
paperwork [global flags] <group> <verb> <PATH> --required-flag ... [--optional-flag ...]
```

Rules to remember:

1. **PATH is the only positional argument** (right after the verb).
2. Every required payload is a **named flag**: `--author` (signing actor) and
   `--message` or `--stdin` (body, exactly one) for `post send` / `post edit`,
   plus `--seq` (the message to edit) for `post edit`.
3. Short forms exist only for `-a` (--author), `-m` (--message), `-q`
   (--quiet). Everything else is long-form only.
4. Bare paths resolve to type-suffixed files: `standup` -> `standup.post.md`,
   `alice` -> `alice.profile.md`, `guide` -> `guide.brief.md`,
   `team` -> `team.contacts.md`. A path ending in bare `.md` is rewritten to
   the type suffix (`notes.md` -> `notes.post.md`). An existing file at the
   given path always wins.
5. A body starting with `-` is passed directly via `--message`:
   `paperwork post send standup.post.md --author alice --message "-fix flag text"`.
   Note: with the space form (`-m <value>` / `--message <value>`) the NEXT
   token is always consumed as the body — a body that looks like a flag
   (e.g. the literal `--stdin`) is written verbatim. To make the intent
   explicit, use the equals form: `-m="--stdin"` / `--message="--stdin"`.

Global flags: `--json` (JSON output), `--plain` (raw file content),
`-q/--quiet` (drop the status line, keep fields).

## Output encoding

Every byte paperwork writes — stdout and stderr, all modes (`default` /
`--json` / `--plain`, with or without `--quiet`) — is **valid UTF-8; decode it
as UTF-8**. The envelope structure (status line, category, `fix:`,
`example:`) is pure ASCII, but `io` error `message` fields may embed
OS-localized text (e.g. Chinese on zh-CN Windows). On Windows PowerShell set
`[Console]::OutputEncoding = [System.Text.Encoding]::UTF8` before capturing
output (a default cp936 session mis-decodes captured stderr). Prefer `--json`
for structured consumption: error envelopes go to stdout as single-line JSON.

## Write commands and file locks

Write commands (`post send` / `post edit`, `brief add` / `brief remove`,
`contacts add` / `contacts update` / `contacts remove`, `profile edit`) are
serialized through an exclusive file lock. The locked critical section is a
read-modify-write of milliseconds, so blocking is normally brief — but waiting
has **no built-in timeout**. If a writer may be stalled, apply your own
process-level timeout, kill the process, and retry: that is safe (`contacts
add` is idempotent; the rest are read-modify-write). A lock is released
automatically when the holding process exits or crashes.

## Exit codes & error self-healing

| Exit | Meaning | Envelope |
|------|---------|----------|
| 0 | Success | `ok <command> <conclusion>` + `key: value` fields |
| 1 | Runtime error | `error <category>: <message>` + `fix:` + `example:` |
| 2 | Usage error (wrong invocation) | `error usage: ...` + `fix:` + canonical `example:` |

Error categories (exit 1): `format`, `validation`, `io`, `not-found`,
`already-exists`, `not-allowed`. On exit 2, copy the `example:` line, adapt the
values, and retry once. Required values are always named flags — positional
NAME/BODY slots from v0.5 are gone; if you see "unexpected argument" for a bare
value, hand it to its named flag (`--author`, `--message`, `--name`, `--seq`,
`--title`, `--entry`, `--entry-title`, `--profile`).

## Tools & typical calls

### profile — agent identity (`*.profile.md`)

```bash
paperwork profile create agents/alice --name alice --model gpt-4o --description "Parser owner"
paperwork profile show agents/alice.profile.md
paperwork profile edit agents/alice.profile.md --model gpt-4o --scope-read "src/**"
paperwork profile list agents
```

### post — append-only threads (`*.post.md`)

```bash
paperwork post send standup --author alice --title "Daily Standup" --message "Parser module is 80% done."
paperwork post send standup --author bob --message "@#1 On it, @alice"
paperwork post send standup.post.md --author alice --stdin < report.md
paperwork post read standup.post.md --from 5 --to 20
paperwork post read standup.post.md --mention alice --limit 20
paperwork post summary standup.post.md
paperwork post edit standup.post.md --author alice --seq 3 --message "corrected body"
```

The first `send` creates the thread (the `--title` flag sets the preamble H1;
on an existing thread `--title` is silently ignored). There is no
`post create` verb anymore. Reply/mention references live in the message
body itself: write an `@#N` token (reply to seq N) and/or `@name` tokens
(mentions) directly in `--message`; the CLI writes the body verbatim and
re-derives the relations on every read. Replying to someone's message
auto-mentions the original sender (`implicit-mention` field appears in the
output only when triggered). The former write-side sugar flags
`--reply-to`/`--mention` are revoked (2026-08-15): passing them is a usage
error (exit 2) whose `fix:` teaches the body-token form; `post read` keeps
its `--mention`/`--reply-to` filters.
`post read` always reports `showing: <displayed>/<total>` and, when non-empty,
`window: #first-#last`.

### brief — knowledge with staleness detection (`*.brief.md`)

```bash
paperwork brief create onboarding --title "Codebase Onboarding" --owner alice
paperwork brief add onboarding.brief.md --entry src/main.rs --regex "fn main" --note "Entry point"
paperwork brief verify onboarding.brief.md          # fresh | shifted | stale
paperwork brief read onboarding.brief.md --full
paperwork brief read onboarding.brief.md --entry-title main.rs  # single-entry details (path/hash/regex/note)
paperwork brief remove onboarding.brief.md --entry-title main.rs  # remove by stored basename
```

Entry titles are stored as basenames: add `src/main.rs`, remove `main.rs`.
`brief read --entry-title <T>` prints only that entry with the full detail
field set; a miss exits 1 with `not-found`.

### contacts — registry of profiles (`*.contacts.md`)

```bash
paperwork contacts create team --title "Core Team"  # --title is optional here (default "Contacts")
paperwork contacts add team.contacts.md --profile agents/alice.profile.md
paperwork contacts remove team.contacts.md --profile agents/alice.profile.md
paperwork contacts update team.contacts.md --profile agents/alice.profile.md --new-profile agents/carol.profile.md
paperwork contacts read team.contacts.md
```

The key for `remove`/`update` is the profile path exactly as stored in the
contacts file, not the link label (`contacts read` lists the stored paths).
`contacts update` re-binds an entry's destination path; it is not an `edit`
(there is no `edit` verb in this group: `edit` means changing a file's own
content, `update` means swapping the entry's target profile).
Destination problems never block a write (2026-08-15 agent-first ruling):
`contacts add`/`update` succeed with exit 0 even when the destination
profile is missing, unreadable, or not a valid profile file; the ok
envelope then carries a single-line `advisory` field (e.g.
`advisory: destination 'ghost.profile.md' does not exist`) in both the
default and `--json` output.

### validate — structural integrity check

```bash
paperwork validate standup.post.md
paperwork validate mystery.md --type post           # post|profile|brief|contacts
```

## If a call fails

- `error usage:` (exit 2): the invocation shape is wrong. Use the printed
  `example:` as the template; required values are named flags, PATH is the
  only positional argument.
- `error not-found:`: the file (or suffixed variant) does not exist. Write
  commands (`send`/`create`/`add`) create files; read-only commands do not.
- `error format:`: the target exists but is not a valid paperwork file. Run
  `paperwork validate <path> --type <kind>` to diagnose.
- `error already-exists:`: use `send` on existing files instead of `create`.
- With `--json`, errors are single-line JSON objects on stdout carrying
  `status`, `category`, `command`, `fix`, `example`, and `exit_code`.
