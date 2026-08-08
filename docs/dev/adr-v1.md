# ADR-011: Stateless Path-Based Architecture (v0 Feedback Correction)

> **Superseded-by note (v0.5.0)**: 本文 CLI Command Model 示例为 v0.4 及更早文法；v0.5.0 文法以 `docs/ssot/specs/cli-ux-redesign/spec.md` 为准。（The CLI Command Model examples in this document reflect v0.4-and-earlier grammar; the v0.5.0 grammar is authoritative in `docs/ssot/specs/cli-ux-redesign/spec.md`. Historical content below is immutable.）

**Status**: Accepted (owner directive: `docs/ssot/adr/feedbacks/v0_feedbacks.md`)

## Context

v0 implementation created a `.paperwork/` centralized workspace folder with init/login semantics. This contradicts the owner's actual design intent.

## Owner Directives (verbatim key points)

1. "我说的托管文件(managed file), 是针对于单个, 或者单个小组文件而言的, 不是直接把所有文件都放到托管文件夹中"
2. "在任意路径启动, 都能够对于任意路径的格式匹配的文件进行操作管理, 保证最大的任意场景临场可用性, 做到尽可能的无状态"
3. "各个文件之间, 也不应该通过managed center进行互相引用"
4. "没有传统意义上的登录语义...GDM给名字就够了"
5. "没有所谓.paperwork文件夹. 有的就是能够通过cli一个个创建文件, 修改文件, 使用文件"

## Decision

**The CLI is a stateless, path-explicit file tool.** No workspace, no init, no login, no central folder.

### Architecture Principles

| Principle | Meaning |
|-----------|---------|
| Stateless | CLI has no config, no state, no memory. SSOT = the files themselves |
| Path-explicit | Every command takes explicit file path(s). No path discovery |
| Independent files | No CLI-managed cross-references between files |
| Format-matching | CLI recognizes files by their format, not their location |
| Any-path | Works from any CWD, operates on any path |

### Managed File Types (independent, standalone)

| Type | What it is | Adjacent files |
|------|-----------|----------------|
| **Profile** | Single .md describing an agent | DM folder in SAME directory (managed name) |
| **DM thread** | Append-only 1:1 conversation file | Lives in profile's adjacent DM folder |
| **Post/GDM** | Standalone append-only group thread | None |
| **Brief** (manifest) | Standalone reading list / knowledge brief | None. At most an owner name field |
| **Contacts** | A special brief listing profile paths | None. Just paths + summaries |

### What Does NOT Exist

- ❌ `.paperwork/` folder
- ❌ `paperwork init`
- ❌ "current agent" / login / session
- ❌ `paperwork invite` (as connection establishment)
- ❌ Central contacts registry that files depend on
- ❌ CLI-managed references between files

### CLI Command Model

```
paperwork profile create <path> --name <n> [--model <m>]
paperwork profile show <path>
paperwork profile edit <path> [fields...]

paperwork dm send <thread-path> --from <name> --to <name> <body>
paperwork dm read <thread-path> [--from N] [--to M]
paperwork dm summary <thread-path>

paperwork post create <path> --title <t> [--participants a,b,c]
paperwork post send <path> --from <name> <body>
paperwork post read <path> [--from N] [--to M]
paperwork post summary <path>

paperwork brief create <path> --title <t> [--owner <name>]
paperwork brief add <path> --entry-path <p> [--regex <r>]
paperwork brief read <path> [--full]
paperwork brief verify <path>

paperwork contacts read <path>
paperwork contacts add <path> --profile <profile-path>
```

### DM Folder Convention

A profile at `any/path/alice.md` has its DM folder at `any/path/alice.dm/`:
```
any/path/
├── alice.md          # profile
└── alice.dm/         # DM folder (managed name = <stem>.dm/)
    ├── bob.md        # DM thread with bob (filename = other party name)
    └── charlie.md    # DM thread with charlie
```

DM folder is auto-created on first `dm send` if absent. No pre-registration needed.

### Notification Convention

@-mentions in a thread can be recorded as notification entries appended to the target profile's adjacent notification file: `any/path/alice.notify.md`. This is append-only like threads.

## Supersedes

- ADR-003 (managed directory layout) — **DELETED**. No managed directory.
- ADR-008 (DM pair naming) — **REVISED**. DM files live in profile-adjacent folder, named by other party.
- ADR-009 (repo separation) — Still valid (core + cli separate crates).
