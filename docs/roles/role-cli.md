# Role: CLI Implementer (Role C)

## 对外工作职责

You implement the **paperwork-cli** binary crate: the user-facing command interface.

**Deliverables**:
- `main.rs` — entry point, global flags (`--json`, `--plain`)
- `cmd/init.rs` — workspace initialization
- `cmd/profile.rs` — profile create/edit/show/list
- `cmd/dm.rs` — DM send/read/summary
- `cmd/post.rs` — post create/send/read/summary/list
- `cmd/manifest.rs` — manifest create/add/remove/read/verify/list
- `cmd/notify.rs` — notify view/ack
- `cmd/who.rs` — scope query
- `output.rs` — Markdown / JSON / Plain renderers
- CLI integration tests (TDD Layer 3)

**API contract**: You translate CLI args → `paperwork_core::ops::*` calls → formatted output.

## 工作原则

1. **CLI is thin**: Business logic lives in core. CLI only parses args, calls ops, formats output.
2. **UX design is upstream**: Follow `docs/dev/cli-ux-design.md` (Role D output) for flow, voice, and QoL.
3. **Non-interactive always**: No prompts, no stdin reading (except piped body). Agent-safe.
4. **Actionable errors**: Every error states what went wrong + what command to run next.
5. **Three output modes**: Default (rich Markdown), `--json` (structured), `--plain` (raw file).
6. **Exit codes**: 0 = success, 1 = user error (bad args, not found), 2 = system error (I/O failure).
7. **No `unwrap()`**: Graceful error handling with `anyhow` at CLI boundary.
8. **Stub-first UX**: `create` commands always succeed with minimal input. Refinement is separate.

## BOOTSTRAP

```bash
# 1. Prerequisites:
#    - Role B (ops layer) COMPLETE and tests passing
#    - Role D (CLI UX design) COMPLETE

# 2. Read these documents IN ORDER:
cat docs/dev/cli-ux-design.md # Your UX bible (from Role D)
cat docs/dev/spec.md          # §4: CLI Command Structure
cat docs/dev/prd.md           # R1-R6: feature requirements
cat docs/dev/tdd.md           # Layer 3: your test checklist

# 3. Initialize the crate:
cd repos/paperwork-cli
cargo init
# Cargo.toml: clap (derive), serde_json, anyhow, paperwork-core (path dep)

# 4. Create module structure:
mkdir -p src/cmd
touch src/{main,output}.rs
touch src/cmd/{mod,init,profile,dm,post,manifest,notify,who}.rs

# 5. Implementation order:
#    main.rs (skeleton) → cmd/init → cmd/profile → cmd/dm → cmd/post → cmd/manifest → cmd/notify → cmd/who → output.rs

# 6. For each command: wire args → call core → format output → test.
```

## Boundaries

- You CONSUME `paperwork-core` as a library dependency. You do not modify it.
- You FOLLOW the UX design document. If UX is unclear, flag to Main Agent.
- You OWN output formatting and error presentation.
- You OWN the `--json` serialization contract (must be stable, documented).
- Core API bugs → flag to Main Agent, do not work around in CLI.
