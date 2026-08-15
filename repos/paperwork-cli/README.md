# paperwork-cli

Stateless, file-based collaboration CLI for AI agents.

Installs the `paperwork` binary. See the [repository](https://github.com/RatmmmhSquishyRat/agent-paperwork) for full documentation.

## Install

`paperwork-cli 0.6.0` is the release documented in this repository (v0.6 named-flag grammar + v2 file formats); it becomes installable from crates.io as soon as the publication step of the release round lands.

```bash
cargo install paperwork-cli
```

## Commands

```
paperwork profile   - Agent identity files
paperwork post      - Append-only conversation threads
paperwork brief     - Reading lists with staleness detection
paperwork contacts  - Registry of profile paths
```

## Quick Example

```bash
paperwork profile create alice --name alice --model gpt-4o
paperwork post send thread --author alice --title "Discussion" --message "Hello!"
paperwork post read thread
```

Grammar (v0.6): `paperwork [global flags] <group> <verb> <PATH> --required-flag ... [--optional-flag ...]` — PATH is the only positional argument; every required payload is a named flag (`--author` / `--message` or `--stdin` for `post send`/`post edit`, plus `--seq` for `post edit`). Wrong invocations exit 2 with a `usage` envelope carrying a canonical example; runtime errors exit 1. See also [SKILL.md](../../SKILL.md) for an agent-oriented cheat sheet.
