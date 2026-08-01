# paperwork-core

Core library for [Agent Paperwork](https://github.com/RatmmmhSquishyRat/agent-paperwork) — stateless, file-based collaboration primitives for AI agents.

## What it provides

- **Format parsers/serializers** for richly-marked Markdown files (profiles, threads, briefs, contacts)
- **Path-explicit operations**: create, read, append, edit, verify — all taking explicit file paths
- **Thread safety**: file locking (fs2) for concurrent append
- **Staleness detection**: regex-anchored + SHA-256 hash verification for briefs

## Usage

```rust
use paperwork_core::ops::thread;
use std::path::Path;

// Send a message (auto-creates file)
let seq = thread::thread_send(
    Path::new("./discussion.md"),
    "alice",        // sender
    &[],            // to (empty = all)
    "Hello world",  // body
    None,           // reply_to
    &[],            // mentions
)?;

// Read messages
let messages = thread::thread_read(Path::new("./discussion.md"), None, None)?;
```

## Design

- **Stateless**: no workspace, no config, no init
- **Path-explicit**: every function takes a concrete file path
- **Append-only threads**: messages grow, never deleted
- **Rich Markdown**: all files human-readable without tooling

See the [repository](https://github.com/RatmmmhSquishyRat/agent-paperwork) for full documentation.
