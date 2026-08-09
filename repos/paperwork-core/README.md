# paperwork-core

Core library for [Agent Paperwork](https://github.com/RatmmmhSquishyRat/agent-paperwork) — stateless, file-based collaboration primitives for AI agents.

## What it provides

- **Format parsers/serializers** for pure-Markdown managed files (profiles, threads, briefs, contacts): H1 identity + prose description, lowercase `- key: value` attribute lines, dynamic-length ` ```md ` message fences, Markdown-link references
- **Path-explicit operations**: create, read, append, edit, verify — all taking explicit file paths
- **Thread safety**: file locking (fs2) for concurrent append; the thread preamble (`ThreadMeta`: the H1 title only) is written in the same lock as the first message. Replies and mentions are body-text tokens (`@#N` / `@name`) derived on read — never stored; participants derive from the message sender set
- **Staleness detection**: regex-anchored + SHA-256 hash verification for briefs

## Usage

```rust
use paperwork_core::{ops::thread, ThreadMeta};
use std::path::Path;

// Send a message (first send creates the file and writes the preamble)
let meta = ThreadMeta {
    title: "Discussion".to_string(),
};
let seq = thread::thread_send(
    Path::new("./discussion.post.md"),
    "alice",          // sender
    "Hello world",    // body
    Some(&meta),      // preamble (ignored once the file is non-empty)
)?;

// Read messages
let messages = thread::thread_read(Path::new("./discussion.post.md"), None, None)?;

// Contacts entries carry a display label derived from the profile
let entries = paperwork_core::ops::contacts::contacts_read(Path::new("./team.contacts.md"))?;
for entry in &entries {
    println!("{} -> {}", entry.label, entry.profile_path);
}
```

## Design

- **Stateless**: no workspace, no config, no init
- **Path-explicit**: every function takes a concrete file path
- **Append-only threads**: messages grow, never deleted
- **Pure Markdown**: all files human-readable without tooling; ASCII-only structural characters

See the [repository](https://github.com/RatmmmhSquishyRat/agent-paperwork) for full documentation.
