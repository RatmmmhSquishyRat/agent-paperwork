# paperwork — Agent Skill Card

Stateless, file-based collaboration primitives for AI agents. No server, no
workspace, no login — every command takes an explicit file path. Files are the
source of truth.

## Grammar (v0.5)

```
paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]
```

Rules to remember:

1. **PATH is always the first positional argument** (right after the verb).
2. For `post send` / `post edit`, **NAME** (the signing actor) is the **second**
   positional argument.
3. **Content (BODY / NEW_BODY) is always the last positional argument.**
4. Bare paths resolve to type-suffixed files: `standup` -> `standup.post.md`,
   `alice` -> `alice.profile.md`, `guide` -> `guide.brief.md`,
   `team` -> `team.contacts.md`. An existing file at the given path always wins.
5. A body starting with `-` must be placed after `--`:
   `paperwork post send standup.post.md alice -- "-fix flag text"`.

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
values, and retry once. There are no `--from`/`--name`/`--title`/`--entry`/
`--seq` flags for required values anymore — they are positional now.

## Tools & typical calls

### profile — agent identity (`*.profile.md`)

```bash
paperwork profile create agents/alice alice --model gpt-4o --description "Parser owner"
paperwork profile show agents/alice.profile.md
paperwork profile edit agents/alice.profile.md --model gpt-4o --scope-read "src/**"
paperwork profile list agents
```

### post — append-only threads (`*.post.md`)

```bash
paperwork post create standup "Daily Standup" --participants alice,bob
paperwork post send standup alice "Parser module is 80% done."
paperwork post send standup bob --reply-to 1 --mention alice "On it"
paperwork post send standup.post.md alice --stdin < report.md
paperwork post read standup.post.md --from 5 --to 20
paperwork post read standup.post.md --mention alice --limit 20
paperwork post summary standup.post.md
paperwork post edit standup.post.md alice 3 "corrected body"
```

Replies auto-mention the original sender (`implicit-mention` field appears in
the output only when triggered). `post read` always reports
`showing: <displayed>/<total>` and, when non-empty, `window: #first-#last`.

### brief — knowledge with staleness detection (`*.brief.md`)

```bash
paperwork brief create onboarding "Codebase Onboarding" --owner alice
paperwork brief add onboarding.brief.md src/main.rs --regex "fn main" --note "Entry point"
paperwork brief verify onboarding.brief.md          # fresh | shifted | stale
paperwork brief read onboarding.brief.md --full
paperwork brief remove onboarding.brief.md main.rs  # remove by stored basename
```

Entry titles are stored as basenames: add `src/main.rs`, remove `main.rs`.

### contacts — registry of profiles (`*.contacts.md`)

```bash
paperwork contacts create team --title "Core Team"  # --title stays a flag here
paperwork contacts add team.contacts.md agents/alice.profile.md
paperwork contacts read team.contacts.md
```

### validate — structural integrity check

```bash
paperwork validate standup.post.md
paperwork validate mystery.md --type post           # post|profile|brief|contacts
```

## If a call fails

- `error usage:` (exit 2): the invocation shape is wrong. Use the printed
  `example:` as the template; required values are positional.
- `error not-found:`: the file (or suffixed variant) does not exist. Write
  commands (`send`/`create`/`add`) create files; read-only commands do not.
- `error format:`: the target exists but is not a valid paperwork file. Run
  `paperwork validate <path> --type <kind>` to diagnose.
- `error already-exists:`: use `send` on existing threads instead of `create`.
- With `--json`, errors are single-line JSON objects on stdout carrying
  `status`, `category`, `command`, `fix`, `example`, and `exit_code`.
