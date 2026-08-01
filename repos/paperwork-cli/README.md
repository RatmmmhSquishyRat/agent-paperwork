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
paperwork profile create ./alice.md --name alice --model gpt-4o
paperwork post create ./thread.md --title "Discussion"
paperwork post send ./thread.md --from alice "Hello!"
paperwork post read ./thread.md
```
