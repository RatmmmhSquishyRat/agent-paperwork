# paperwork

A simple, fast and stateless file-based collaboration toolkit for AI agents.

[Installation](#installation) • [Usage](#usage) • [Output Protocol](#output-protocol) • [File Formats](#file-formats)

## Features

- **Zero setup**: no init, no login, no config, no workspace. Works from any path on any file.
- **Agent-first output**: structured `ok`/`error` envelope protocol. Errors carry `fix:` + `example:` for one-retry self-correction.
- **Append-only threads**: async communication via post files. Reply, @mention, filter — all in one primitive.
- **Staleness detection**: brief files track regex anchors + SHA-256 hashes. Verify if knowledge is fresh, shifted, or stale.
- **Pure ASCII output**: no Unicode symbols, no ANSI. Parseable by any agent in any environment.
- **Human-readable files**: all managed files are richly-marked Markdown with fenced code blocks.
- **Cross-platform**: single static binary. Linux, macOS, Windows.

## Usage

```bash
# Profiles
paperwork profile create alice --name alice --model gpt-4o
paperwork profile show alice
paperwork profile list .

# Threads (auto-creates .post.md on first send)
paperwork post create standup --title "Daily Standup" --participants alice,bob
paperwork post send standup --from alice "Proposing Rust."
paperwork post send standup --from bob --reply-to 2 "Agreed."
paperwork post send standup --from alice --stdin < report.md
paperwork post read standup --mention alice
paperwork post summary standup

# Briefs (reading guides with staleness detection)
paperwork brief create onboarding --title "Onboarding" --owner alice
paperwork brief add onboarding --entry "src/main.rs" --regex "fn main"
paperwork brief verify onboarding

# Contacts
paperwork contacts create team --title "Team"
paperwork contacts add team --profile ./alice.profile.md
paperwork contacts read team

# Validation
paperwork validate standup.post.md
```

## Output Protocol

Every command outputs a structured envelope. Pure ASCII, machine-parseable without JSON.

```
ok post.send #4 -> standup.post.md
seq: 4
path: standup.post.md
sender: alice
```

On failure (stderr, exit 1):

```
error not-found: thread 'standup.post.md' does not exist
fix: send a message to auto-create, or run post create
example: paperwork post send standup --from alice "first message"
```

| Flag | Behavior |
|------|----------|
| (default) | Structured envelope |
| `--json` | JSON with `"status": "ok"/"error"` |
| `--plain` | Raw file content |
| `-q` | Suppress status line, keep fields |

## File Formats

Managed files use type suffixes: `.profile.md`, `.post.md`, `.brief.md`, `.contacts.md`.

Message bodies are wrapped in 4-backtick fences — any Markdown inside is safe:

```markdown
---

### #2 alice . 2026-08-01T10:00:05Z

- To: all
- Reply-To: #1
- Mentions: bob

````markdown
I propose we use Rust.

Here's a code block inside:
```rust
fn main() {}
```
````
```

## Installation

### From crates.io

```bash
cargo install paperwork-cli
```

### From release binaries

Precompiled binaries for Linux, macOS, and Windows are available on the [Releases page](https://github.com/RatmmmhSquishyRat/agent-paperwork/releases).

### From source

```bash
git clone https://github.com/RatmmmhSquishyRat/agent-paperwork
cd agent-paperwork
cargo build --release
cargo install --path repos/paperwork-cli
```

Requires Rust 1.74+.

## Development

```bash
git clone https://github.com/RatmmmhSquishyRat/agent-paperwork
cd agent-paperwork

# Build
cargo build

# Test
cargo test

# Lint
cargo clippy --all-targets -- -D warnings
```

## Architecture

```
repos/
  paperwork-core/    Library: format parsers + path-explicit operations
  paperwork-cli/     Binary: thin CLI (clap -> core -> envelope output)
```

[paperwork-core](https://crates.io/crates/paperwork-core) is a standalone library consumable by IDE plugins, agent harnesses, or other tools.

## License

MIT — see [LICENSE](./LICENSE).
