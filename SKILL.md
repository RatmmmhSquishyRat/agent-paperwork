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
   `team` -> `team.contacts.md`. An existing file at the given path always wins.
5. A body starting with `-` is passed directly via `--message`:
   `paperwork post send standup.post.md --author alice --message "-fix flag text"`.

Global flags: `--json` (JSON output), `--plain` (raw file content),
`-q/--quiet` (drop the status line, keep fields).

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
paperwork post send standup --author bob --reply-to 1 --mention alice --message "On it"
paperwork post send standup.post.md --author alice --stdin < report.md
paperwork post read standup.post.md --from 5 --to 20
paperwork post read standup.post.md --mention alice --limit 20
paperwork post summary standup.post.md
paperwork post edit standup.post.md --author alice --seq 3 --message "corrected body"
```

The first `send` creates the thread (the `--title` flag sets the preamble H1;
on an existing thread `--title` is silently ignored). There is no
`post create` verb anymore. Replies auto-mention the original sender
(`implicit-mention` field appears in the output only when triggered).
`post read` always reports `showing: <displayed>/<total>` and, when non-empty,
`window: #first-#last`.

### brief — knowledge with staleness detection (`*.brief.md`)

```bash
paperwork brief create onboarding --title "Codebase Onboarding" --owner alice
paperwork brief add onboarding.brief.md --entry src/main.rs --regex "fn main" --note "Entry point"
paperwork brief verify onboarding.brief.md          # fresh | shifted | stale
paperwork brief read onboarding.brief.md --full
paperwork brief remove onboarding.brief.md --entry-title main.rs  # remove by stored basename
```

Entry titles are stored as basenames: add `src/main.rs`, remove `main.rs`.

### contacts — registry of profiles (`*.contacts.md`)

```bash
paperwork contacts create team --title "Core Team"  # --title is optional here (default "Contacts")
paperwork contacts add team.contacts.md --profile agents/alice.profile.md
paperwork contacts read team.contacts.md
```

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
