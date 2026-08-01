# paperwork

**Stateless, file-based collaboration toolkit for AI agents.**

Unix philosophy applied to agent coordination: everything is a file, everything is append-only, everything is human-readable. No server, no database, no daemon, no login.

## Why

AI agents working together need minimal primitives:
- **Identity** — who am I, what's my scope
- **Communication** — async append-only threads
- **Knowledge transfer** — codified reading lists with staleness detection

All achievable with filesystem semantics alone. `paperwork` provides a thin CLI over these file formats.

## Design Principles

| Principle | Meaning |
|-----------|---------|
| Stateless | No config, no workspace, no memory. SSOT = the files themselves |
| Path-explicit | Every command takes explicit file paths |
| Independent files | No CLI-managed cross-references between files |
| Append-only | Threads grow; no insert, no delete |
| Human-readable | All managed files are richly-marked Markdown |
| Agent-safe | Non-interactive, `--json` for machine consumption |

## Install

```bash
cargo install --path repos/paperwork-cli
```

Or build from source:
```bash
cargo build --release --manifest-path repos/paperwork-cli/Cargo.toml
```

## Commands

```
paperwork profile   — Agent identity files
paperwork post      — Append-only conversation threads (the only communication primitive)
paperwork brief     — Reading lists with regex-anchored staleness detection
paperwork contacts  — Registry of profile paths (a special brief)
```

## Quick Start

```bash
# Create agent profiles (just files, anywhere)
paperwork profile create ./agents/alice.md --name alice --model gpt-4o
paperwork profile create ./agents/bob.md --name bob --model claude-4

# Start a thread (auto-creates file)
paperwork post create ./threads/design.md --title "Architecture Discussion"
paperwork post send ./threads/design.md --from alice "I propose we use Rust."
paperwork post send ./threads/design.md --from bob --reply-to 1 "Agreed. What about the CLI framework?"

# Reply carries implicit @mention of original sender
paperwork post send ./threads/design.md --from alice --reply-to 2 "clap derive."

# Filter by mention or reply
paperwork post read ./threads/design.md --mention alice
paperwork post read ./threads/design.md --reply-to 1

# Create a reading guide (brief)
paperwork brief create ./guides/onboarding.md --title "Project Onboarding"
paperwork brief add ./guides/onboarding.md --entry "src/main.rs" --regex "fn main"
paperwork brief verify ./guides/onboarding.md --base-dir .

# Contacts: just a list of profile paths
paperwork contacts create ./team.md --title "Team"
paperwork contacts add ./team.md --profile ./agents/alice.md
paperwork contacts read ./team.md
```

## File Formats

All managed files are **richly-marked Markdown** — readable both raw and through CLI.

### Thread (post) file

```markdown
---

### #1 — alice · 2026-07-30T10:00:00Z

**To**: all
**Reply-To**: —

I propose we use Rust.

---

### #2 — bob · 2026-07-30T10:01:00Z

**To**: all
**Reply-To**: #1
**Mentions**: alice

Agreed. What about the CLI framework?

---
```

### Profile file

```markdown
# alice

**Model**: gpt-4o
**Description**: Parser module implementer

## Scope

**Read**: `src/**`, `docs/**`
**Write**: `src/parser/**`
**Owns**: `src/parser/**`
```

### Brief (manifest) file

```markdown
# Manifest: onboarding

**Author**: alice
**Created**: 2026-07-30T10:00:00Z
**Description**: How to read this project

## Entries

### main.rs

**Path**: `src/main.rs`
**Hash**: `42b6647...`
**Regex**: `fn main`

---
```

## Brief Verification (Staleness Detection)

Each brief entry stores a regex anchor + SHA-256 hash. `paperwork brief verify` reports:

| State | Meaning |
|-------|---------|
| **Fresh** | Regex matches + hash matches — use directly |
| **Shifted** | Regex matches + hash differs — structure holds, content changed |
| **Stale** | Regex fails — needs re-curation |

## Architecture

```
repos/
├── paperwork-core/   # Library: format parsing + path-explicit operations
└── paperwork-cli/    # Binary: thin CLI layer (clap → core → output)
```

- **paperwork-core**: Pure Rust library. Format parsers/serializers + filesystem operations. No CLI dependency. Consumable by other tools (IDE plugins, agent harnesses).
- **paperwork-cli**: Thin binary. Parses args → calls core → formats output. Three output modes: default (Markdown), `--json`, `--plain`.

## Global Flags

| Flag | Effect |
|------|--------|
| `--json` | Structured JSON output |
| `--plain` | Raw file content |
| `-q, --quiet` | Suppress confirmation messages |

## What This Is NOT

- ❌ Not a chat app (no real-time, no server)
- ❌ Not a project manager (no tasks, no boards)
- ❌ Not enforced access control (scope is honor-system)
- ❌ Not stateful (no `.paperwork/` folder, no init, no login)

## License

MIT
