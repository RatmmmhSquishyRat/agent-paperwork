# Agent Paperwork — Implementation Plan

## Execution Strategy

Per SSOT《MainAgent工作编排》: aggressive decomposition, parallel Role Agents, independent threads.
Per SSOT《实现流程原则》: this document + role docs must pass adversarial review before implementation begins.

---

## Module Decomposition (Role Agent Assignment)

### Role A: `paperwork-core` — Format Layer

**Scope**: `format/` module — all parsing and serialization.

**Deliverables**:
- `format/mod.rs` — shared parsing utilities (bold-key extraction, H3 header matching, `---` splitting)
- `format/profile.rs` — Profile parse/serialize
- `format/contacts.rs` — Contacts table parse/serialize
- `format/thread.rs` — Message parse/serialize (single + multi)
- `format/manifest.rs` — Manifest + ManifestEntry parse/serialize
- `format/notification.rs` — Notification parse/serialize
- Unit tests: all roundtrip + edge cases from TDD Layer 1

**Dependencies**: None (leaf module).
**Estimated complexity**: Medium. Pure functions, no I/O.

---

### Role B: `paperwork-core` — Operations Layer

**Scope**: `ops/` module + `layout.rs` + `hash.rs` — all filesystem operations.

**Deliverables**:
- `layout.rs` — `.paperwork/` skeleton creation, path resolution
- `hash.rs` — SHA-256 blob hashing
- `ops/profile.rs` — create, edit, show, list
- `ops/contacts.rs` — invite, contacts list, who-query
- `ops/thread.rs` — append (atomic), read-range, summary, self-edit
- `ops/manifest.rs` — create, add entry, remove entry, verify
- `ops/notify.rs` — push, list unread, ack
- `error.rs` — unified error type with actionable messages
- Integration tests: all from TDD Layer 2

**Dependencies**: Role A (format layer) must be complete.
**Estimated complexity**: High. Filesystem I/O, atomicity, concurrency.

---

### Role C: `paperwork-cli` — Command Interface

**Scope**: `paperwork-cli` crate — clap command tree, dispatch, output formatting.

**Deliverables**:
- `main.rs` — entry point, global flags
- `cmd/init.rs` — init command
- `cmd/profile.rs` — profile subcommands
- `cmd/invite.rs` — invite command
- `cmd/contacts.rs` — contacts listing command
- `cmd/who.rs` — who scope-query command
- `cmd/dm.rs` — DM subcommands (send, read, edit, summary)
- `cmd/post.rs` — post subcommands
- `cmd/manifest.rs` — manifest subcommands
- `cmd/notify.rs` — notify subcommands
- `output.rs` — Markdown / JSON / Plain output renderers
- CLI integration tests: all from TDD Layer 3

**Dependencies**: Role B (ops layer) must be complete.
**Estimated complexity**: Medium-High. UX design, error presentation.

---

### Role D: CLI UX Design (Document-Only)

**Scope**: Design the CLI's natural flow semantics, user journey, expression, QoL.

**Deliverables**:
- `docs/dev/cli-ux-design.md` — command flow diagrams, output examples, error voice, agent ergonomics
- Must be completed BEFORE Role C implementation begins

**Dependencies**: PRD + ADR + spec must be closed.
**Estimated complexity**: Low-Medium. Design work, no code.

---

## Execution Order & Parallelism

```
Phase 1 (parallel):
  ├── Role D: CLI UX Design document
  └── Role A: Format layer implementation

Phase 2 (after Role A closes):
  └── Role B: Operations layer implementation

Phase 3 (after Role B + Role D close):
  └── Role C: CLI implementation

Phase 4 (after Role C):
  └── Main Agent: Full review, e2e verification, acceptance
```

**Parallelism opportunities**:
- Role A and Role D are fully independent → run simultaneously
- Within Role A: profile/contacts/thread/manifest/notification parsers are independent → sub-parallelizable
- Within Role B: ops modules depend on format but are mutually independent → sub-parallelizable after format closes

---

## Acceptance Criteria

| Gate | Criteria |
|------|----------|
| Format complete | All TDD Layer 1 tests pass, roundtrip verified |
| Ops complete | All TDD Layer 2 tests pass, atomic append verified under concurrency |
| CLI complete | All TDD Layer 3 tests pass, `--json` valid for every command |
| Final acceptance | Full workflow: init → invite → dm send/read → post create/send → manifest create/add/verify → notify → who. All via CLI. All files human-readable. |

---

## Risk Register

| Risk | Mitigation |
|------|------------|
| Regex parsing fragility | Extensive edge-case fixtures; format is controlled (we define it) |
| Windows atomic append | Dedicated Windows test; `FILE_APPEND_DATA` well-documented |
| Seq collision under concurrency | Mitigated by ADR-007 file locking (`fs2`); implemented and tested with 10-thread contention |
| Manifest glob-vs-path ambiguity | Clear spec: path = exact or glob; regex = content anchor |

---

## Definition of Done (per module)

1. Code compiles with zero warnings (`cargo clippy`)
2. All specified tests pass (`cargo test`)
3. Public API documented with rustdoc
4. No `unwrap()` in library code (proper error propagation)
5. Format output matches spec §2 byte-for-byte
