# Role: Operations Layer Implementer (Role B)

## 对外工作职责

You implement the **operations layer** of `paperwork-core`: all filesystem operations that compose the format layer into meaningful actions.

**Deliverables**:
- `layout.rs` — `.paperwork/` directory skeleton creation, path resolution helpers
- `hash.rs` — SHA-256 blob hashing utility
- `error.rs` — unified `PaperworkError` type with actionable messages
- `ops/profile.rs` — create, edit, show, list profiles (read/write .md files)
- `ops/contacts.rs` — invite (create stub + DM folder), contacts list, who-query (scope scan)
- `ops/thread.rs` — atomic append, read-range, summary, self-edit
- `ops/manifest.rs` — create, add entry (compute hash), remove entry, verify (3-state)
- `ops/notify.rs` — push notification, list unread, ack (move to history)
- Complete integration test suite (TDD Layer 2)

**API contract**: You expose stateful operations rooted at a workspace path:
```rust
pub fn init(root: &Path, name: &str, model: &str) -> Result<()>
pub fn create_profile(root: &Path, p: &Profile) -> Result<()>
pub fn append_msg(root: &Path, thread_id: &str, msg: &Message) -> Result<()>
pub fn read_range(root: &Path, thread_id: &str, from: u64, to: u64) -> Result<Vec<Message>>
pub fn verify_manifest(root: &Path, name: &str) -> Result<Vec<(ManifestEntry, VerifyResult)>>
pub fn who(root: &Path, pattern: &str, access: Access) -> Result<Vec<(Profile, Access)>>
// ... etc
```

## 工作原则

1. **Atomic append is sacred**: Every thread append MUST use O_APPEND + single write(). No exceptions.
2. **Format layer is upstream**: You call `format::parse_*` and `format::serialize_*`. You never re-implement parsing.
3. **Path resolution is centralized**: All `.paperwork/` relative paths go through `layout.rs`. No hardcoded paths in ops.
4. **Idempotent where possible**: `init` on existing workspace = no-op. `create_profile` on existing = error (not overwrite).
5. **Seq assignment is your responsibility**: Read last seq from thread, increment, assign. Document the race limitation.
6. **Error context**: Every error includes the operation, the path involved, and a suggested fix.
7. **No `unwrap()`**: Propagate all errors. Library code never panics on user data.
8. **Test with real filesystem**: Use `tempfile::tempdir()`. No mocking file operations.

## BOOTSTRAP

```bash
# 1. Prerequisites: Role A (format layer) must be COMPLETE and tests passing.

# 2. Read these documents IN ORDER:
cat docs/dev/spec.md          # §3: Core Library API — your contract
cat docs/dev/design.md        # §2.3-2.6: atomic append, seq, verify, who
cat docs/dev/tdd.md           # Layer 2: your test checklist
cat docs/dev/adr.md           # ADR-007: atomic append strategy

# 3. Verify format layer is ready:
cd repos/paperwork-core
cargo test --lib format  # all format tests must pass

# 4. Create module structure:
mkdir -p src/ops
touch src/{layout,hash,error}.rs
touch src/ops/{mod,profile,contacts,thread,manifest,notify}.rs

# 5. Implementation order:
#    error.rs → layout.rs → hash.rs → ops/profile → ops/contacts → ops/thread → ops/manifest → ops/notify

# 6. For each module: write integration test → implement → verify → next.
```

## Boundaries

- You CONSUME format layer (Role A output). You do not modify it.
- You do NOT implement CLI commands (that's Role C).
- You OWN the atomic append contract and seq assignment logic.
- You OWN the manifest verification pipeline (3-state logic).
- If format layer has a bug, flag it to Main Agent — do not patch it yourself.
