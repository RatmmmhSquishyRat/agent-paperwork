# paperwork-cli

Stateless, file-based collaboration CLI for AI agents.

Installs the `paperwork` binary. See the [repository](https://github.com/RatmmmhSquishyRat/agent-paperwork) for full documentation.

## Install

```bash
cargo install paperwork-cli
```

## Commands

```
paperwork profile   — Agent identity files
paperwork post      — Append-only conversation threads
paperwork brief     — Reading lists with staleness detection
paperwork contacts  — Registry of profile paths
```

## Quick Example

```bash
paperwork profile create ./alice.md alice --model gpt-4o
paperwork post create ./thread.md "Discussion"
paperwork post send ./thread.md alice "Hello!"
paperwork post read ./thread.md
```

Grammar (v0.5): `paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]` — PATH is always first; NAME is the second positional for `post send`/`post edit`; content is always last. Wrong invocations exit 2 with a `usage` envelope carrying a canonical example; runtime errors exit 1. See also [SKILL.md](../../SKILL.md) for an agent-oriented cheat sheet.
