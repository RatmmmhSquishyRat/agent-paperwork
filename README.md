# paperwork

Stateless, file-based collaboration primitives for AI agents. Identity, messaging, and knowledge briefs — all as plain Markdown files, operated by one CLI.

<p>
  <a href="https://github.com/RatmmmhSquishyRat/agent-paperwork/actions/workflows/ci.yml"><img src="https://github.com/RatmmmhSquishyRat/agent-paperwork/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/paperwork-cli"><img src="https://img.shields.io/crates/v/paperwork-cli.svg?label=paperwork-cli" alt="crates.io paperwork-cli"></a>
  <a href="https://crates.io/crates/paperwork-core"><img src="https://img.shields.io/crates/v/paperwork-core.svg?label=paperwork-core" alt="crates.io paperwork-core"></a>
  <a href="https://crates.io/crates/paperwork-cli"><img src="https://img.shields.io/crates/d/paperwork-cli.svg" alt="downloads"></a>
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey" alt="platforms">
  <a href="./LICENSE"><img src="https://img.shields.io/crates/l/paperwork-cli.svg" alt="license"></a>
</p>

No server. No database. No daemon. No login. No workspace. Every command takes an explicit file path and works from anywhere — the files are the source of truth.

---

## Quick start

```bash
cargo install paperwork-cli
```

Then, from any directory:

```bash
$ paperwork post send standup --title "Daily Standup" --from alice "Proposing Rust for the backend."
ok post.send #1 -> standup.post.md
seq: 1
path: standup.post.md
sender: alice
```

That's it. `standup.post.md` is created on first send — the preamble (the H1 title) is written in the same lock as message #1 — and the reply is one line:

```bash
$ paperwork post send standup --from bob --reply-to 1 "Agreed. Ship it."
ok post.send #2 -> standup.post.md
seq: 2
path: standup.post.md
sender: bob
```

---

## Why

AI agents that work together need three minimal things: **who they are**, **a way to talk**, and **a way to hand off knowledge**. `paperwork` provides exactly those three as filesystem primitives, so any harness can adopt them with zero infrastructure:

| Primitive | Command | File | Purpose |
|-----------|---------|------|---------|
| Identity | `profile` | `*.profile.md` | An agent's name, model, and scope |
| Communication | `post` | `*.post.md` | Append-only threads with reply + @mention |
| Knowledge | `brief` | `*.brief.md` | Reading lists with staleness detection |
| Directory | `contacts` | `*.contacts.md` | A list of agent profiles as Markdown links |

Every output is a **structured, ASCII-only envelope** built for machine parsing — an agent can detect success/failure from the first line and self-correct from the error's `fix:` and `example:` fields without reading docs.

---

## Install

**From [crates.io](https://crates.io/crates/paperwork-cli)** (recommended):

```bash
cargo install paperwork-cli
```

**From release binaries** — precompiled for Linux, macOS, and Windows on the [Releases page](https://github.com/RatmmmhSquishyRat/agent-paperwork/releases).

**From source:**

```bash
git clone https://github.com/RatmmmhSquishyRat/agent-paperwork
cd agent-paperwork
cargo install --path repos/paperwork-cli
```

Requires Rust 1.74+.

---

## Commands

### `profile` — agent identity

```bash
paperwork profile create alice --name alice --model gpt-4o --description "Parser owner"
paperwork profile show alice
paperwork profile edit alice --scope-read "src/**" --scope-owns "src/parser/**"
paperwork profile list .
```

### `post` — append-only threads

```bash
paperwork post send standup --title "Daily Standup" --from alice "Status update"
paperwork post send standup --from bob --reply-to 1 --mention alice "On it"
paperwork post send standup --from alice --stdin < report.md   # multi-line via pipe
paperwork post read standup --mention alice                    # filter by @mention
paperwork post read standup --reply-to 1                       # filter by reply
paperwork post summary standup
```

`--reply-to N` and `--mention a,b` are sugar: they inject `@#N` / `@name` tokens at the head of the body before writing. References live only in the body text — reply/mention state is re-derived from it on every read.

### `brief` — knowledge with staleness detection

```bash
paperwork brief create onboarding --title "Onboarding" --owner alice
paperwork brief add onboarding --entry "src/main.rs" --regex "fn main"
paperwork brief verify onboarding                              # fresh | shifted | stale
paperwork brief read onboarding --full
```

### `contacts` — registry of profiles

```bash
paperwork contacts create team --title "Team"
paperwork contacts add team --profile ./alice.profile.md
paperwork contacts read team                                   # shows name + description
```

### `validate` — structural integrity check

```bash
paperwork validate standup.post.md
```

Parses by type suffix; for posts it additionally enforces consecutive seq numbers starting at 1 and closed code fences. Warnings (suspected malformed message headers) are reported without failing the check.

---

## Output protocol

All output is pure ASCII — no color, no Unicode symbols. Parseable without JSON.

**Success** (stdout, exit 0):

```
ok post.send #2 -> standup.post.md
seq: 2
path: standup.post.md
sender: bob
```

**Failure** (stderr, exit 1) — always tells you how to fix it:

```
error not-found: Thread 'standup.post.md' not found
fix: send a message first to create the thread
example: paperwork post send standup.post.md --from <name> <body>
```

**Modes:**

| Flag | Output |
|------|--------|
| _(default)_ | Structured envelope above |
| `--json` | JSON with `"status": "ok" \| "error"` |
| `--plain` | Raw file content |
| `-q` | Drop the status line, keep fields |

---

## File formats

Managed files are plain Markdown named by type suffix. One shared design language: H1 is the document identity, free prose after the H1 is the description, flat attributes are lowercase `- key: value` bullet lines (omit a line instead of writing a placeholder), references to other managed files are Markdown links, and message bodies sit inside ` ```md ` fences whose length is dynamic — always one backtick longer than the longest backtick run inside, so any Markdown inside them (headings, lists, triple-backtick blocks, `---`) is safe. All structural characters are ASCII.

**`standup.post.md`** — preamble (the H1 title only) followed by H2 messages `## #<seq> <sender> (<RFC3339>)`; no attribute lines — replies and mentions are body-text tokens (`@#N` = reply to message N, `@name` = mention) derived on read:

`````markdown
# Daily Standup

## #1 alice (2026-08-01T10:00:05Z)

```md
Proposing Rust for the backend.
```

## #2 bob (2026-08-01T10:01:00Z)

```md
@#1 @alice Agreed. Ship it.
```
`````

**`alice.profile.md`** — H1 name, prose description, `- model:` attribute, optional `## Scope` section of `- <permission>: <glob>` lines:

```markdown
# alice

Parser owner

- model: gpt-4o

## Scope

- read: src/**
- owns: src/parser/**
```

**`onboarding.brief.md`** — H1 title, prose description, `- owner:` / `- created:` attributes, then entry H2 sections with `path` / `hash` / `regex` attributes and a prose note (complex regexes use a ` ```regex ` fence):

```markdown
# Onboarding

Reading list for new agents

- owner: alice
- created: 2026-08-01T10:00:00Z

## main.rs

- path: src/main.rs
- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540
- regex: fn main

Entry point
```

**`team.contacts.md`** — H1 title and one Markdown link bullet per profile (paths containing spaces are wrapped in angle brackets):

```markdown
# Team

- [alice](alice.profile.md)
- [bob](bob.profile.md)
```

---

## Development

```bash
git clone https://github.com/RatmmmhSquishyRat/agent-paperwork
cd agent-paperwork

cargo build                                     # build
cargo test                                      # 154 tests
cargo clippy --all-targets -- -D warnings       # lint
```

CI runs the full test suite **and** an end-to-end smoke test on Linux, macOS, and Windows on every push.

## Architecture

```
repos/
  paperwork-core/   Library: format parsers + path-explicit operations
  paperwork-cli/    Binary: thin CLI (clap -> core -> envelope output)
```

[paperwork-core](https://crates.io/crates/paperwork-core) is a standalone library for IDE plugins, agent harnesses, or other tools.

## License

[MIT](./LICENSE)
