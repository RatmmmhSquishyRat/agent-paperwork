# Agent Paperwork — Design Document

## 1. Architecture Overview

Two crates, clean dependency edge:

```
┌─────────────────────────────────────────────┐
│  paperwork-cli (binary)                     │
│  ┌─────────────────────────────────────┐    │
│  │ clap command tree                   │    │
│  │ output formatting (md/json/plain)   │    │
│  │ UX flow, QoL, error presentation    │    │
│  └──────────────┬──────────────────────┘    │
│                 │ depends on                 │
│  ┌──────────────▼──────────────────────┐    │
│  │ paperwork-core (library)            │    │
│  │ ┌─────────┐ ┌─────┐ ┌───────────┐  │    │
│  │ │ format/ │ │ ops/│ │ layout.rs │  │    │
│  │ └─────────┘ └─────┘ └───────────┘  │    │
│  │ ┌─────────┐ ┌──────────┐           │    │
│  │ │ hash.rs │ │ error.rs │           │    │
│  │ └─────────┘ └──────────┘           │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
         │
         ▼ filesystem only
┌─────────────────────┐
│  .paperwork/        │
│  (managed files)    │
└─────────────────────┘
```

No network. No daemon. No database. Pure filesystem operations.

---

## 2. Core Design Decisions

### 2.1 Format Layer: Regex-Based Markdown Parsing

**Why not a Markdown AST parser?** The managed format uses a *restricted subset* of Markdown (headings, bold keys, tables, horizontal rules). A full commonmark parser adds complexity without benefit — we control the format.

**Approach**: Boundary-anchored regex scanning:
1. Scan for **message boundaries**: `---` line immediately followed (within 2 lines) by H3 matching `### #(\d+) — (.+) · (.+)`. A lone `---` not followed by this pattern is body content.
2. Split content at validated boundaries only
3. Within each block, extract bold-key metadata lines: `\*\*(\w[\w-]*)\*\*: (.+)`
4. Remaining lines = body (may contain `---`, headings, bold text freely)
5. Normalize CRLF → LF before parsing (invariant I11)

This is fast (<1ms for typical files), deterministic, and body-safe.

### 2.2 Serialization: Template-Based Generation

Messages/profiles are serialized via format templates (not AST construction):

```rust
fn serialize_message(msg: &Message) -> String {
    let to_str = if msg.to.is_empty() {
        "all".to_string()
    } else {
        msg.to.join(", ")
    };
    format!(
        "---\n\n### #{} — {} · {}\n\n**To**: {}\n**Reply-To**: {}\n\n{}\n\n",
        msg.seq, msg.sender, msg.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
        to_str,
        msg.reply_to.map(|r| format!("#{}", r)).unwrap_or("—".into()),
        msg.body
    )
}
```

Guarantees: output is always valid managed Markdown, byte-for-byte reproducible.

### 2.3 Atomic Append with File Locking

Append operations are protected by advisory file locking to prevent seq collision:

```rust
fn locked_append(path: &Path, data: &[u8]) -> io::Result<()> {
    let f = OpenOptions::new().append(true).create(true).open(path)?;
    // Advisory lock: blocks concurrent writers
    #[cfg(unix)]
    {
        use fs2::FileExt;
        f.lock_exclusive()?;
    }
    #[cfg(windows)]
    {
        use fs2::FileExt;
        f.lock_exclusive()?;
    }
    // Single write() call for the message block
    f.write_all(data)?;
    f.unlock()?;
    Ok(())
}
```

**Guarantee**: The lock serializes concurrent appends, eliminating seq collision. The `O_APPEND` flag ensures the write targets EOF even if the file grew between lock acquisition and write. Message size limit: 64KB (hard cap; typical <4KB).

### 2.4 Seq Number Assignment

On append (within file lock):
1. Acquire exclusive lock on thread file
2. Reverse-scan last 4KB for last `### #N` header (O(1) regardless of file size)
3. New seq = N + 1 (or 1 if empty)
4. Serialize message + write via O_APPEND
5. Release lock

No race condition: the exclusive lock serializes all writers. Seq is always monotonically increasing with no gaps.

### 2.5 Manifest Verification Pipeline

Aligned with ADR-004 (including no-regex rows):

```
for each entry in manifest:
    file_bytes = read(entry.path)
    current_hash = sha256(file_bytes)
    
    if entry.regex.is_some():
        match = regex.find(file_content)
        if match.is_none():
            → Stale
        elif current_hash == entry.hash:
            → Fresh
        else:
            → Shifted
    else:
        if current_hash == entry.hash:
            → Fresh
        else:
            → Shifted  // no regex to confirm structure
```

### 2.6 Who-Query: Scope Scanning

```
for each profile in .paperwork/profiles/:
    parse profile
    match glob patterns in scope.{owns|read|write} against query pattern
    collect matches with access type
return sorted results
```

Glob matching uses the `glob` crate's `Pattern::matches()` for pattern-vs-pattern comparison.

---

## 3. CLI Design

### 3.1 Command Dispatch

`clap` derive-based:

```rust
#[derive(Parser)]
#[command(name = "paperwork", about = "File-based agent collaboration toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true)]
    plain: bool,
}

#[derive(Subcommand)]
enum Command {
    Init { name: String, ... },
    Profile { #[command(subcommand)] cmd: ProfileCmd },
    Invite { name: String, ... },
    Contacts,
    Who { owns: Option<String>, reads: Option<String>, writes: Option<String> },
    Dm { agent: String, #[command(subcommand)] cmd: DmCmd },
    Post { #[command(subcommand)] cmd: PostCmd },
    Manifest { #[command(subcommand)] cmd: ManifestCmd },
    Notify { ack: bool, agent: Option<String> },
}
```

### 3.2 Output Strategy

```rust
trait Output {
    fn render_profile(&self, p: &Profile) -> String;
    fn render_messages(&self, msgs: &[Message]) -> String;
    fn render_verify(&self, results: &[(ManifestEntry, VerifyResult)]) -> String;
    // ...
}

struct MarkdownOutput;  // default: rich terminal Markdown
struct JsonOutput;      // --json: serde_json
struct PlainOutput;     // --plain: raw file content
```

### 3.3 Error Handling UX

Errors are actionable:
```
Error: Profile 'charlie' not found.
  → Run `paperwork profile create charlie` or `paperwork invite charlie` first.
```

---

## 4. Testing Strategy

| Layer | Approach |
|-------|----------|
| `format/` | Unit tests: parse ↔ serialize roundtrip for every file type |
| `ops/` | Integration tests: temp-dir filesystem operations |
| `cli` | End-to-end: assert command output against golden files |
| Concurrency | Stress test: N parallel appends → verify no interleaving |

---

## 5. Dependency Graph (External Crates)

| Crate | Used By | Purpose |
|-------|---------|---------|
| `clap` (derive) | cli | Command parsing |
| `serde` + `serde_json` | core + cli | JSON output mode |
| `regex` | core | Markdown parsing, manifest anchors |
| `sha2` | core | Blob hashing |
| `glob` | core | Pattern matching for scope/who queries |
| `chrono` | core | Timestamp handling |
| `anyhow` / `thiserror` | core | Error types |
| `fs2` | core | Cross-platform advisory file locking |
| `tempfile` | tests | Temp directories for integration tests |

---

## 6. Build & Distribution

- `cargo build --release` → single binary
- Cross-platform via `cross` (Linux ARM/x64, macOS, Windows)
- No runtime dependencies; static linking via `musl` on Linux
- Version: semver, core and cli versioned independently
