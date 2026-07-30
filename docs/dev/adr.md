# Architecture Decision Records

## ADR-001: Implementation Language — Rust

**Status**: Accepted (owner directive: `docs/ssot/adr/初版技术选型.md`)  
**Context**: The CLI must distribute as a single static binary with no runtime dependencies, support Windows/macOS/Linux, and perform file I/O, YAML parsing, regex matching, and SHA hashing.

**Decision**: Rust.

**Rationale**:
- Owner directive: "初版技术选型使用rust技术栈实现即可"
- Single static binary distribution (no runtime install)
- `clap` (CLI), `serde` (serialization), `regex` (manifest anchors), `sha2` (blob hash), `glob` (path patterns) — mature ecosystem
- `O_APPEND` + single `write()` syscall guarantees atomic append on all target platforms
- Performance: <50ms per subcommand trivially achievable
- Cross-compilation via `cross` or GitHub Actions matrix

**Alternatives rejected**:
- Go: viable but larger binaries, less expressive type system for envelope modeling
- Node/TypeScript: requires runtime — violates zero-dep constraint
- Python: requires runtime — violates zero-dep constraint
- Shell scripts: fragile, no structured regex support, poor cross-platform

---

## ADR-002: Managed File Format — Richly-Marked Markdown

**Status**: Accepted (owner directive: `docs/ssot/adr/初版技术选型.md`)  
**Context**: All managed files must be readable both through CLI output AND as raw files on disk. Need unambiguous message delimiting, machine-parseable metadata, and human readability.

**Decision**: Richly-marked Markdown as the universal managed file format. Use headings, bold keys, lists, and `---` delimiters for structure. No YAML/JSON for user-facing managed files.

**Owner directive**: "managed文件格式推荐使用带丰富标记的markdown, 来保证through cli/file两种情况都可轻松阅读."

**Thread format (DM/GDM)**:
```markdown
---

### #1 — alice · 2026-07-29T14:30:00Z

**To**: bob  
**Reply-To**: —

Message body in free-form Markdown.

---

### #2 — bob · 2026-07-29T14:31:00Z

**To**: alice  
**Reply-To**: #1

Reply body here.

---
```

**Profile format**:
```markdown
# alice

**Model**: gpt-4o  
**Description**: Parser module implementer

## Scope

**Read**: `src/parser/**`, `docs/**`  
**Write**: `src/parser/**`  
**Owns**: `src/parser/**`
```

**Rationale**:
- Markdown renders beautifully in terminals (via CLI formatting), editors, and GitHub
- Raw file is immediately human-readable without any tooling
- Machine-parseable via heading/list/bold conventions (regex-tractable)
- `---` + `### #seq` gives unambiguous message boundaries
- Agents can also write/read these files directly without CLI

**Alternatives rejected**:
- YAML front-matter: less readable as raw file, requires YAML awareness
- JSON lines: not human-friendly, no multi-line body
- SQLite: violates file-based, human-readable constraint

---

## ADR-003: Managed Directory Layout as Specification

**Status**: Accepted  
**Context**: Interoperability requires that any tool understanding the layout can work with paperwork files, even without the CLI.

**Decision**: The `.paperwork/` directory structure IS the specification.

```
.paperwork/
├── profiles/
│   └── <name>.md
├── contacts.md
├── dm/
│   └── <agent-a>--<agent-b>/
│       ├── meta.md
│       └── thread.md
├── posts/
│   └── <post-name>/
│       ├── meta.md
│       └── log.md
├── manifests/
│   └── <name>.md
└── notifications/
    └── <name>/
        ├── unread.md
        └── history.md
```

**Rationale**:
- Directory layout is self-documenting
- Any agent/tool can navigate without CLI
- CLI is convenience, not dependency
- All files are `.md` (richly-marked Markdown per ADR-002)
- Naming conventions (alphabetically-sorted DM pair names) prevent duplication

---

## ADR-004: Staleness Detection — Regex Anchor + Blob Hash

**Status**: Accepted  
**Context**: Read manifests reference file content that may change. Need a confidence signal for reading agents.

**Decision**: Each manifest entry stores: `path`, optional `regex` (with named groups), and `hash` (SHA-256 of file blob at curation time).

**Three-state verification**:
| Regex | Hash | Verdict |
|-------|------|--------|
| Match succeeds | Hash matches | **Fresh** — use directly |
| Match succeeds | Hash differs | **Shifted** — structure holds, content changed |
| Match fails | (any) | **Stale** — needs re-curation |
| No regex defined | Hash matches | **Fresh** — use directly |
| No regex defined | Hash differs | **Shifted** — content changed, no structure to confirm |

**Rationale**:
- Regex answers "does the thing I care about still exist?"
- Hash answers "has anything changed at all?"
- Combined: confidence signal, not just raw content
- No line numbers (fragile), no hashline (non-standard)

---

## ADR-005: Scope — Honor System, No CLI Enforcement

**Status**: Accepted (owner directive)  
**Context**: Profile declares read/write/owns scope. Should CLI enforce?

**Decision**: No. Scope is declarative and self-governed. Agents dynamically negotiate.

**Rationale** (owner): "cli不会也无法enforce读写范围, 所以profile中的范围都是自主负责, 动态协商调整的."
- Enforcement adds complexity without real security (agents can bypass CLI)
- Honor system keeps CLI thin and composable
- `who` query provides discovery, not gating

---

## ADR-006: No Core Loop — Independent Tool Composition

**Status**: Accepted (owner directive)  
**Context**: Should DM, profiles, and manifests form a hierarchical pipeline?

**Decision**: No hierarchy, no fixed usage loop. Each pillar is an independent tool.

**Rationale** (owner): "There is no core loop in design... each forms an individual usage tool themselves."
- Agents compose tools freely based on task needs
- No forced workflow reduces friction
- Each pillar is useful standalone

---

## ADR-007: Atomic Append Strategy — File Locking + O_APPEND

**Status**: Accepted  
**Context**: Multiple agents may append to the same GDM file concurrently. Need to prevent seq collision and torn writes.

**Decision**: Advisory file locking (`fs2` crate: `lock_exclusive()`) around read-seq + append, combined with `O_APPEND` for EOF-targeting writes.

**Protocol**:
1. Open file with append mode
2. Acquire exclusive advisory lock (`flock` Unix / `LockFileEx` Windows)
3. Reverse-scan last 4KB for last seq (O(1))
4. Serialize + single `write()` call
5. Release lock

**Rationale**:
- File locking eliminates seq collision entirely (no race)
- `O_APPEND` ensures write targets EOF even if file grew between lock and write
- Advisory locking sufficient: all writers use CLI/core (cooperative model)
- Hard limit: message size MUST be < 64KB (CLI rejects larger)
- `fs2` crate provides cross-platform locking

**Correction**: Earlier draft claimed PIPE_BUF atomicity for regular files — this is incorrect (PIPE_BUF applies to pipes only). File locking is the correct mechanism.

---

## ADR-008: DM Pair Naming Convention

**Status**: Proposed  
**Context**: DM folders need deterministic naming to avoid duplicates (alice-bob vs bob-alice).

**Decision**: Sort agent names alphabetically, join with `--`: `alice--bob`.

**Rationale**:
- Deterministic: both agents compute the same path
- No collision: unique pair → unique folder name
- Human-readable
- `--` separator avoids collision with agent names containing `-`

---

## ADR-009: Repository Separation — Core + CLI

**Status**: Accepted (owner directive: `docs/ssot/adr/初版技术选型.md`)  
**Context**: The core file-format/operations library and the CLI user interface have distinct concerns: core is about data integrity and format compliance; CLI is about UX, flow, expression, and quality-of-life.

**Decision**: Two separate repositories.

| Repo | Responsibility |
|------|----------------|
| `paperwork-core` | File format spec, read/write/append operations, parsing, validation, staleness detection. Library crate. |
| `paperwork-cli` | Command structure, UX flow, output formatting, QoL features. Binary crate. Depends on `paperwork-core`. |

**Owner directive**: "paperwork core和cli分repo来实现, 并且cli的自然流程操作语义, 动线, 表述, UX&Qol需单独设计和落地."

**Rationale**:
- Core can be consumed as a library by other tools (IDE plugins, agent harnesses)
- CLI UX design is a distinct discipline requiring separate iteration
- Clean API boundary forces good core abstractions
- Independent versioning and release cadence

**Implication**: CLI's natural flow semantics, user journey, expression style, and UX/QoL must be independently designed and documented before implementation.

---

## ADR-010: Terminology — "meeting" Deprecated in Favor of "post" / GDM

**Status**: Accepted (owner directive)  
**Context**: The original design used "meeting" for multi-party group conversations. This implies synchronous, scheduled interaction — misaligned with the async, append-only nature of the system.

**Decision**: Deprecate "meeting". The canonical terms are:
- **post** — user-facing noun for a multi-party conversation thread
- **GDM** (Group DM) — technical/internal term for the same concept

**Rationale**:
- "post" better reflects async, append-only semantics (like a forum post thread)
- "GDM" is the natural generalization of DM to N participants
- "meeting" implies real-time presence, which this system explicitly does not require
- CLI subcommands: `paperwork post create/send/read/summary` (not `meeting`)
- Directory: `.paperwork/posts/<name>/` (not `meetings/`)

**Migration**: All documents and code use `post`/`GDM`. "meeting" is not used anywhere in implementation.
