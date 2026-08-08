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
$ paperwork post send standup alice "Proposing Rust for the backend."
ok post.send #1 -> standup.post.md
seq: 1
path: standup.post.md
sender: alice
```

That's it. `standup.post.md` is created on first send, and the reply is one line:

```bash
$ paperwork post send standup bob --reply-to 1 "Agreed. Ship it."
ok post.send #2 -> standup.post.md
seq: 2
path: standup.post.md
sender: bob
implicit-mention: alice
```

---

## Why

AI agents that work together need three minimal things: **who they are**, **a way to talk**, and **a way to hand off knowledge**. `paperwork` provides exactly those three as filesystem primitives, so any harness can adopt them with zero infrastructure:

| Primitive | Command | File | Purpose |
|-----------|---------|------|---------|
| Identity | `profile` | `*.profile.md` | An agent's name, model, and scope |
| Communication | `post` | `*.post.md` | Append-only threads with reply + @mention |
| Knowledge | `brief` | `*.brief.md` | Reading lists with staleness detection |
| Directory | `contacts` | `*.contacts.md` | A list of profile paths |

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
paperwork profile create alice alice --model gpt-4o --description "Parser owner"
paperwork profile show alice
paperwork profile edit alice --scope-read "src/**" --scope-owns "src/parser/**"
paperwork profile list .
```

### `post` — append-only threads

```bash
paperwork post create standup "Daily Standup" --participants alice,bob
paperwork post send standup alice "Status update"
paperwork post send standup bob --reply-to 1 --mention alice "On it"
paperwork post send standup alice --stdin < report.md          # multi-line via pipe
paperwork post read standup --mention alice                    # filter by @mention
paperwork post read standup --reply-to 1                       # filter by reply
paperwork post summary standup
```

### `brief` — knowledge with staleness detection

```bash
paperwork brief create onboarding "Onboarding" --owner alice
paperwork brief add onboarding src/main.rs --regex "fn main"
paperwork brief verify onboarding                              # fresh | shifted | stale
paperwork brief read onboarding --full
```

### `contacts` — registry of profiles

```bash
paperwork contacts create team --title "Team"
paperwork contacts add team ./alice.profile.md
paperwork contacts read team                                   # shows name + description
```

### `validate` — structural integrity check

```bash
paperwork validate standup.post.md
paperwork validate mystery.md --type post                      # explicit parser selection
```

---

## Output protocol

All output is pure ASCII — no color, no Unicode symbols. Parseable without JSON.

**Grammar (v0.5):** `paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]` — PATH is always the first positional argument; for `post send`/`post edit` NAME (the signing actor) is the second; content is always last.

**Success** (stdout, exit 0):

```
ok post.send #2 -> standup.post.md
seq: 2
path: standup.post.md
sender: bob
```

**Runtime failure** (stderr, exit 1) — always tells you how to fix it:

```
error not-found: thread 'standup.post.md' does not exist
fix: send a message to auto-create, or run post create
example: paperwork post send standup alice "first message"
```

**Usage failure** (stderr, exit 2) — wrong invocation (missing/unknown arguments), with a canonical copy-paste example:

```
error usage: required values are positional...
fix: required values are positional (PATH first; NAME second for post send/edit); see the canonical example below
example: paperwork post send standup.post.md alice "Parser module is 80% done."
```

A body starting with `-` must be placed after `--`: `paperwork post send standup.post.md alice -- "-fix flag text"`.

**Modes:**

| Flag | Output |
|------|--------|
| _(default)_ | Structured envelope above |
| `--json` | JSON with `"status": "ok" \| "error"` |
| `--plain` | Raw file content |
| `-q` | Drop the status line, keep fields |

---

## File formats

Managed files are richly-marked Markdown, named by type suffix. Message bodies sit inside 4-backtick fences, so any Markdown inside them (headings, lists, triple-backtick blocks, `---`) is safe.

**`standup.post.md`**

````markdown
---

### #1 alice . 2026-08-01T10:00:05Z

- To: all

```markdown
Proposing Rust for the backend.
```

---

### #2 bob . 2026-08-01T10:01:00Z

- To: all
- Reply-To: #1
- Mentions: alice

```markdown
Agreed. Ship it.
```
````

**`alice.profile.md`**

```markdown
# alice

- Model: gpt-4o
- Description: Parser owner

## Scope

- Read: `src/**`
- Owns: `src/parser/**`
```

**`onboarding.brief.md`**

```markdown
# Onboarding

- Owner: alice
- Created: 2026-08-01T10:00:00Z

## Entries

### main.rs

- Path: `src/main.rs`
- Hash: `42b6647…`
- Regex: `fn main`
```

---

## Development

```bash
git clone https://github.com/RatmmmhSquishyRat/agent-paperwork
cd agent-paperwork

cargo build                                     # build
cargo test                                      # 159 tests
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
