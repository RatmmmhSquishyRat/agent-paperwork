# paperwork

Stateless, file-based collaboration toolkit for AI agents.

[![crates.io](https://img.shields.io/crates/v/paperwork-cli.svg)](https://crates.io/crates/paperwork-cli)
[![crates.io](https://img.shields.io/crates/v/paperwork-core.svg)](https://crates.io/crates/paperwork-core)
[![license](https://img.shields.io/crates/l/paperwork-cli.svg)](./LICENSE)

Everything is a file. Everything is append-only. Everything is human-readable.
No server, no database, no daemon, no login, no workspace.

## Install

```bash
cargo install paperwork-cli
```

Provides the `paperwork` binary. Requires Rust 1.74+.

## Commands

```
paperwork profile    Agent identity files
paperwork post       Append-only conversation threads
paperwork brief      Reading lists with staleness detection
paperwork contacts   Registry of profile paths
paperwork validate   Check file structure integrity
```

## Quick Start

```bash
# Profiles (auto-suffixes to .profile.md)
paperwork profile create alice --name alice --model gpt-4o
paperwork profile create bob --name bob --model claude-4

# Thread (auto-suffixes to .post.md, auto-creates on first send)
paperwork post create standup --title "Daily Standup" --participants alice,bob
paperwork post send standup --from alice "Proposing Rust for the backend."
paperwork post send standup --from bob --reply-to 2 "Agreed."

# Reply carries implicit @mention of original sender
paperwork post send standup --from alice --reply-to 3 --mention bob "clap derive."

# Multi-line content via stdin
cat report.md | paperwork post send standup --from alice --stdin

# Filter
paperwork post read standup --mention alice
paperwork post read standup --reply-to 2

# Brief (reading guide with staleness detection)
paperwork brief create onboarding --title "Project Onboarding" --owner alice
paperwork brief add onboarding --entry "src/main.rs" --regex "fn main"
paperwork brief verify onboarding

# Contacts (reads profiles for summaries)
paperwork contacts create team --title "Team"
paperwork contacts add team --profile ./alice.profile.md
paperwork contacts read team

# Validate file structure
paperwork validate standup.post.md
```

## Output Protocol

Designed for agent consumption. Pure ASCII, no ANSI, no Unicode symbols.

### Success (stdout)

```
ok post.send #4 -> standup.post.md
seq: 4
path: standup.post.md
sender: alice
```

### Error (stderr)

```
error not-found: thread 'standup.post.md' does not exist
fix: send a message to auto-create, or run post create
example: paperwork post send standup --from alice "first message"
```

### Rules

- Line 1 is always `ok` or `error` — instant status
- Fields are `key: value` — machine-parseable without JSON
- Body separator `---` only in read commands
- Errors carry `fix:` + `example:` — self-correct in one retry

### Modes

| Flag | Behavior |
|------|----------|
| (default) | Structured envelope (above) |
| `--json` | JSON with same fields + `"status": "ok"/"error"` |
| `--plain` | Raw file content |
| `-q` | Suppress status line, keep fields/body |

## File Formats

All managed files use type suffixes: `.profile.md`, `.post.md`, `.brief.md`, `.contacts.md`.

### Post thread (`standup.post.md`)

```markdown
---

### #1 system . 2026-08-01T10:00:00Z

- To: all

````markdown
[Thread created: Daily Standup | participants: alice, bob]
````

---

### #2 alice . 2026-08-01T10:00:05Z

- To: all

````markdown
Proposing Rust for the backend.
````

---

### #3 bob . 2026-08-01T10:01:00Z

- To: all
- Reply-To: #2
- Mentions: alice

````markdown
Agreed. What about the CLI framework?
````
```

Message bodies are wrapped in 4-backtick fenced code blocks.
This makes parsing unambiguous — any Markdown inside (headings, lists, triple-backtick fences, `---`) is safe.

### Profile (`alice.profile.md`)

```markdown
# alice

- Model: gpt-4o
- Description: Parser module implementer

## Scope

- Read: `src/**`, `docs/**`
- Write: `src/parser/**`
- Owns: `src/parser/**`
```

### Brief (`onboarding.brief.md`)

```markdown
# Project Onboarding

- Owner: alice
- Created: 2026-08-01T10:00:00Z
- Description: How to read this project

## Entries

### main.rs

- Path: `src/main.rs`
- Hash: `42b664743ddb6056...`
- Regex: `fn main`

> Entry point of the application.
```

### Contacts (`team.contacts.md`)

```markdown
# Team

- ./alice.profile.md
- ./bob.profile.md
```

## Brief Verification

Each entry stores a regex anchor + SHA-256 hash. `paperwork brief verify` reports:

| State | Meaning |
|-------|---------|
| fresh | Regex matches + hash matches — use directly |
| shifted | Regex matches + hash differs — content changed, structure holds |
| stale | Regex fails — needs re-curation |

## Architecture

```
repos/
  paperwork-core/    Library: format parsers + path-explicit operations
  paperwork-cli/     Binary: thin CLI (clap -> core -> envelope output)
```

- [paperwork-core](https://crates.io/crates/paperwork-core) — Pure Rust library. Consumable by IDE plugins, agent harnesses, other tools.
- [paperwork-cli](https://crates.io/crates/paperwork-cli) — Thin binary. Installs the `paperwork` command.

## Design Principles

| Principle | Meaning |
|-----------|---------|
| Stateless | No config, no workspace, no memory. SSOT = the files |
| Path-explicit | Every command takes explicit file paths |
| Independent | No CLI-managed cross-references between files |
| Append-only | Threads grow; no insert, no delete |
| Human-readable | All files are richly-marked Markdown |
| Agent-first | Structured output, actionable errors, bounded responses |

## What This Is NOT

- Not a chat app (no real-time, no server)
- Not a project manager (no tasks, no boards)
- Not enforced access control (scope is honor-system)
- Not stateful (no `.paperwork/` folder, no init, no login)

## License

MIT
