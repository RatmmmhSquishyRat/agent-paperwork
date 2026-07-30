# CLI UX Design — Agent Paperwork

> **Role D deliverable.** Primary reference for Role C (CLI implementer).  
> Governed by: spec.md §4, prd.md R1–R6, ADR-006 (independent pillars), ADR-009 (repo separation), ADR-010 (post terminology).  
> Owner directive: "cli的自然流程操作语义, 动线, 表述, UX&Qol需单独设计和落地."  
> Owner UX principle: "先流畅轻松甚至stub创建, 然后再细致编辑."

---

## 1. Command Flow Semantics

Each pillar (profile, dm, post, manifest, notify) is an **independent tool** (ADR-006). There is no forced hierarchy or core loop. The flows below are *typical journeys*, not enforced sequences.

### 1.1 First-Time Setup

```
paperwork init --name alice --model gpt-4o
paperwork invite bob --model claude-sonnet
paperwork dm bob send "Hey, workspace is ready."
```

What happens:
1. `init` creates `.paperwork/` skeleton + `profiles/alice.md` stub + `contacts.md` with alice registered.
2. `invite` creates `profiles/bob.md` stub + `dm/alice--bob/` folder with `meta.md` + empty `thread.md`, registers bob in contacts.
3. `dm bob send` appends message #1 to `dm/alice--bob/thread.md`.

Minimum viable setup: **two commands** (`init` + `invite`) and you're chatting.

### 1.2 Daily Agent Use

```
paperwork notify                    # check what's new
paperwork dm bob read               # catch up (last 10 by default)
paperwork dm bob send "Done with parser." --reply-to 5
paperwork post standup read --from 12
paperwork post standup send "Shipped manifest verify." --mention alice
```

Pattern: **check → read → respond → broadcast**. Each step is independently useful.

### 1.3 Knowledge Sharing (Manifest Journey)

```
paperwork manifest create onboarding --description "How to understand this codebase"
paperwork manifest onboarding add --path "src/lib.rs" --note "Entry point"
paperwork manifest onboarding add --path "src/format/*.rs" --regex "pub fn parse_\w+" --note "All parsers"
paperwork manifest onboarding verify
paperwork dm bob send "Check out manifest 'onboarding' for codebase intro."
```

Pattern: **create stub → add entries incrementally → verify freshness → share reference**.

### 1.4 Discovery & Coordination

```
paperwork who --owns "src/parser/**"
paperwork contacts
paperwork profile show bob
```

Pattern: **find the right agent → inspect their scope → reach out**.

---

## 2. Expression & Voice

### 2.1 Tone Principles

| Principle | Rule |
|-----------|------|
| Terse | No filler words. No "Successfully created..." → just confirm the fact. |
| Actionable errors | Always: what went wrong + what to do next. |
| Agent-first | Output is parseable, predictable, and minimal. Beauty is secondary. |
| No chatter | No tips, suggestions, or "Did you mean?" unless explicitly helpful. |

### 2.2 Success Messages

Success output is **one line**, confirmatory, includes the artifact path:

```
✓ profile created: profiles/bob.md
✓ invited bob → dm/alice--bob/
✓ sent #3 → dm/alice--bob/thread.md
✓ post created: posts/standup/
✓ manifest created: manifests/onboarding.md
✓ entry added: "src/lib.rs" → manifests/onboarding.md
✓ 2 notifications acknowledged → history
```

Format: `✓ <past-tense verb> <key detail> → <path or ref>`

### 2.3 Error Messages

Two-line format: **what** + **fix**.

```
✗ profile "charlie" not found
  → run: paperwork invite charlie

✗ cannot send: dm folder dm/alice--charlie/ does not exist
  → run: paperwork invite charlie

✗ seq #99 out of range (thread has 12 messages)
  → valid range: 1–12

✗ manifest "onboarding" not found
  → run: paperwork manifest create onboarding

✗ edit denied: #7 was sent by bob (you are alice)
  → you can only edit your own messages

✗ message body exceeds 64KB limit (got 71KB)
  → split into multiple messages or reduce content
```

Format: `✗ <what went wrong>` + newline + `  → <what to do>`

### 2.4 Informational Output

Tables for lists, structured blocks for detail views. Always scannable. See §3 for full examples.

---

## 3. Output Design

Every command supports three output modes:

| Mode | Flag | Audience | Content |
|------|------|----------|---------|
| Default | (none) | Human / agent reading terminal | Rich Markdown rendering with color |
| JSON | `--json` | Machine / agent parsing | Structured JSON, one object or array |
| Plain | `--plain` | Piping / raw inspection | Raw file content (cat-equivalent) |

### 3.1 `profile show <name>`

**Default:**
```
# bob

**Model**: claude-sonnet
**Description**: Integration test writer

## Scope

**Read**: `src/**`, `tests/**`
**Write**: `tests/**`
**Owns**: `tests/**`
```

**`--json`:**
```json
{
  "name": "bob",
  "model": "claude-sonnet",
  "description": "Integration test writer",
  "scope": {
    "read": ["src/**", "tests/**"],
    "write": ["tests/**"],
    "owns": ["tests/**"]
  },
  "path": "profiles/bob.md"
}
```

**`--plain`:**
```
# bob

**Model**: claude-sonnet  
**Description**: Integration test writer

## Scope

**Read**: `src/**`, `tests/**`  
**Write**: `tests/**`  
**Owns**: `tests/**`
```

### 3.2 `profile list`

**Default:**
```
AGENT   MODEL           DESCRIPTION
alice   gpt-4o          Parser module implementer
bob     claude-sonnet   Integration test writer
```

**`--json`:**
```json
[
  {"name": "alice", "model": "gpt-4o", "description": "Parser module implementer", "path": "profiles/alice.md"},
  {"name": "bob", "model": "claude-sonnet", "description": "Integration test writer", "path": "profiles/bob.md"}
]
```

**`--plain`:**
```
| Agent | Profile |
|-------|--------|
| alice | profiles/alice.md |
| bob | profiles/bob.md |
```

### 3.3 `dm <agent> read`

**Default** (last 10 messages, rendered):
```
── dm/alice--bob/thread.md ── 12 messages ──

#3  alice → bob   2026-07-30T09:15:00Z
    Started on the parser module.

#4  bob → alice   2026-07-30T09:16:00Z  ↩#3
    Great. I'll set up test fixtures.

#5  alice → bob   2026-07-30T09:20:00Z
    @bob fixtures should cover edge cases in §2.3.
```

**`--json`:**
```json
{
  "thread": "dm/alice--bob/thread.md",
  "total": 12,
  "showing": {"from": 3, "to": 5},
  "messages": [
    {
      "seq": 3,
      "sender": "alice",
      "to": ["bob"],
      "timestamp": "2026-07-30T09:15:00Z",
      "reply_to": null,
      "body": "Started on the parser module."
    },
    {
      "seq": 4,
      "sender": "bob",
      "to": ["alice"],
      "timestamp": "2026-07-30T09:16:00Z",
      "reply_to": 3,
      "body": "Great. I'll set up test fixtures."
    },
    {
      "seq": 5,
      "sender": "alice",
      "to": ["bob"],
      "timestamp": "2026-07-30T09:20:00Z",
      "reply_to": null,
      "body": "@bob fixtures should cover edge cases in §2.3."
    }
  ]
}
```

**`--plain`:** Raw file content of `thread.md` (the full Markdown as stored on disk).

### 3.4 `dm <agent> summary`

**Default:**
```
dm/alice--bob — 12 messages
last: bob · 2026-07-30T09:16:00Z
recent:
  #10 alice: "Manifest verify is passing now."
  #11 bob: "Nice. Running full suite."
  #12 bob: "All green ✓"
```

**`--json`:**
```json
{
  "thread": "dm/alice--bob/thread.md",
  "message_count": 12,
  "last_sender": "bob",
  "last_timestamp": "2026-07-30T09:16:00Z",
  "snippets": [
    {"seq": 10, "sender": "alice", "preview": "Manifest verify is passing now."},
    {"seq": 11, "sender": "bob", "preview": "Nice. Running full suite."},
    {"seq": 12, "sender": "bob", "preview": "All green ✓"}
  ]
}
```

### 3.5 `post <name> read` / `post <name> summary`

**`post standup summary` default:**
```
post: standup — "Daily Standup"
participants: alice, bob, charlie (3)
messages: 47
last: charlie · 2026-07-30T08:00:00Z
recent:
  #45 alice: "Parser done, moving to CLI."
  #46 bob: "Tests for parser merged."
  #47 charlie: "Docs updated for manifest format."
```

**`post standup read --from 45 --to 47` default:**
```
── posts/standup/log.md ── 47 messages ── showing #45–#47 ──

#45  alice → all   2026-07-30T07:50:00Z
     Parser done, moving to CLI.

#46  bob → all   2026-07-30T07:55:00Z
     Tests for parser merged.

#47  charlie → all   2026-07-30T08:00:00Z
     Docs updated for manifest format.
```

**`--json`** follows same structure as DM read/summary, with added `"participants"` and `"title"` fields.

### 3.6 `manifest <name> verify`

**Default:**
```
manifest: onboarding (3 entries)

  FRESH    src/lib.rs              hash ✓  regex ✓
  SHIFTED  src/format/*.rs         hash ✗  regex ✓  (content changed, structure holds)
  STALE    docs/old-guide.md       regex ✗  (pattern no longer matches)

summary: 1 fresh · 1 shifted · 1 stale
```

Color coding (when enabled): FRESH=green, SHIFTED=yellow, STALE=red.

**`--json`:**
```json
{
  "manifest": "onboarding",
  "entries": [
    {"title": "src/lib.rs", "path": "src/lib.rs", "verdict": "fresh", "hash_match": true, "regex_match": true},
    {"title": "src/format/*.rs", "path": "src/format/*.rs", "verdict": "shifted", "hash_match": false, "regex_match": true},
    {"title": "docs/old-guide.md", "path": "docs/old-guide.md", "verdict": "stale", "hash_match": false, "regex_match": false}
  ],
  "summary": {"fresh": 1, "shifted": 1, "stale": 1}
}
```

### 3.7 `notify`

**Default (unread):**
```
notifications for alice — 2 unread

  2026-07-30T09:20:00Z  from bob    mention  in dm/alice--bob/thread.md #5
    "@alice fixtures should cover edge cases in §2.3."

  2026-07-30T09:45:00Z  from charlie  reply  in posts/standup/log.md #47
    "Re: your #45 — nice work on parser."
```

**`notify --ack` default:**
```
✓ 2 notifications acknowledged → notifications/alice/history.md
```

**`--json`:**
```json
{
  "agent": "alice",
  "unread_count": 2,
  "notifications": [
    {
      "timestamp": "2026-07-30T09:20:00Z",
      "from": "bob",
      "type": "mention",
      "thread": "dm/alice--bob/thread.md",
      "seq": 5,
      "snippet": "@alice fixtures should cover edge cases in §2.3."
    },
    {
      "timestamp": "2026-07-30T09:45:00Z",
      "from": "charlie",
      "type": "reply",
      "thread": "posts/standup/log.md",
      "seq": 47,
      "snippet": "Re: your #45 — nice work on parser."
    }
  ]
}
```

### 3.8 `who` Query

**`paperwork who --owns "src/parser/**"` default:**
```
owns "src/parser/**":
  alice   profiles/alice.md   scope.owns: `src/parser/**`
```

**`--json`:**
```json
{
  "query": {"access": "owns", "pattern": "src/parser/**"},
  "results": [
    {"name": "alice", "profile": "profiles/alice.md", "matched_scope": "src/parser/**"}
  ]
}
```

---

## 4. Quality-of-Life Features

### 4.1 Aliases

| Full Command | Alias | Notes |
|--------------|-------|-------|
| `paperwork profile` | `paperwork p` | |
| `paperwork dm` | `paperwork d` | |
| `paperwork post` | `paperwork g` | "g" for group |
| `paperwork manifest` | `paperwork m` | |
| `paperwork notify` | `paperwork n` | |
| `paperwork contacts` | `paperwork c` | |
| `paperwork who` | `paperwork w` | |

Aliases are single-character, no ambiguity. Full names always work.

### 4.2 Sensible Defaults

| Command | Default Behavior |
|---------|-----------------|
| `dm <agent> read` | Last 10 messages |
| `post <name> read` | Last 10 messages |
| `notify` | Show unread for current agent (from `init --name`) |
| `manifest <name> read` | TOC only (entry titles + paths); `--full` for complete content |
| `profile create <name>` | All fields empty/placeholder except name |
| `manifest create <name>` | Empty manifest with name + author + timestamp |
| `post create <name>` | Requires `--participants`; title defaults to name |

### 4.3 Tab Completion Hints

The CLI ships a `paperwork completions <shell>` subcommand (bash, zsh, fish, powershell). Completions cover:
- Subcommand names and aliases
- Agent names (from `contacts.md`) for `dm`, `invite`, `who`
- Post names (from `posts/`) for `post <name>`
- Manifest names (from `manifests/`) for `manifest <name>`
- Flag names for each subcommand

### 4.4 Color Output

- Color enabled by default when stdout is a TTY.
- Respects `NO_COLOR` environment variable (https://no-color.org/): if set (any value), disable all color.
- `--no-color` flag as explicit override.
- Color is **never** applied in `--json` or `--plain` modes.
- Semantic colors: green=success/fresh, yellow=warning/shifted, red=error/stale, dim=metadata, bold=agent names.

### 4.5 `--json` Universality

Every single command that produces output supports `--json`. This includes:
- `init` → `{"created": ".paperwork/", "profile": "profiles/alice.md"}`
- `invite` → `{"invited": "bob", "dm_folder": "dm/alice--bob/"}`
- `send` → `{"seq": 3, "thread": "dm/alice--bob/thread.md"}`
- `edit` → `{"edited": 7, "thread": "dm/alice--bob/thread.md"}`
- Errors → `{"error": "profile not found", "hint": "run: paperwork invite charlie", "code": 1}`

---

## 5. Agent Ergonomics

### 5.1 No Interactive Prompts — Ever

The CLI **never** reads from stdin for confirmation, selection, or input (except body via pipe, see §5.5). Every decision is made via flags or positional args. If required input is missing, the CLI errors immediately with a clear message.

```
✗ missing required argument: <name>
  → usage: paperwork profile create <name> [--model <id>] [--description <text>]
```

### 5.2 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Operation error (not found, validation failed, permission) |
| 2 | Usage error (bad args, missing required flag) |

No other codes. Agents can branch on `0` vs non-zero; `1` vs `2` distinguishes "tried but failed" from "didn't try."

### 5.3 Structured Error Output

With `--json`, errors are JSON on stdout:
```json
{"error": "seq #99 out of range", "detail": "thread has 12 messages", "hint": "valid range: 1-12", "exit_code": 1}
```

Without `--json`, errors go to **stderr** in the `✗` / `→` format (§2.3).

### 5.4 Idempotent Operations

| Operation | Idempotency |
|-----------|-------------|
| `init` | If `.paperwork/` exists, no-op with `✓ already initialized` |
| `invite <name>` | If profile + DM folder exist, no-op with `✓ bob already invited` |
| `profile create` | If profile exists, error (not idempotent — would overwrite) |
| `post create` | If post exists, error |
| `manifest create` | If manifest exists, error |
| `notify --ack` | If no unread, no-op with `✓ no unread notifications` |

Create operations that would destroy data are **not** idempotent (they error). Setup operations (`init`, `invite`) **are** idempotent (safe to re-run).

### 5.5 Body Input: Positional Arg OR Stdin Pipe

Message bodies and edit bodies accept input two ways:

```bash
# Positional argument (short messages)
paperwork dm bob send "Hello world"

# Stdin pipe (long/multi-line bodies)
cat report.md | paperwork dm bob send --stdin
echo "Quick note" | paperwork dm bob send --stdin

# Manifest note from file
cat note.txt | paperwork manifest onboarding add --path "src/lib.rs" --note-stdin
```

Rules:
- `--stdin` flag reads body from stdin until EOF.
- Positional arg and `--stdin` are mutually exclusive (error if both).
- If no body provided and stdin is not a TTY, implicitly read stdin (convenience for pipes).
- If no body and stdin IS a TTY → error (no interactive prompt).

### 5.6 Predictable Output Ordering

- Messages: always in seq order (ascending).
- Profile list / contacts: alphabetical by agent name.
- Manifest entries: in file order (insertion order).
- Notifications: chronological (oldest first).
- JSON arrays preserve the same ordering as default output.

---

## 6. Stub-First UX

> Owner directive: "先流畅轻松甚至stub创建, 然后再细致编辑, cli的全程UX都应该遵循这种体验."

Every `create` command produces a **working artifact immediately** with minimal input. Refinement is always a separate, optional step.

### 6.1 `init` — Minimum Viable Workspace

```bash
paperwork init --name alice
```

Creates:
```
.paperwork/
├── profiles/alice.md      ← stub (model: —, description: —, scope: all —)
├── contacts.md            ← one row: alice
├── dm/                    ← empty dir
├── posts/                 ← empty dir
├── manifests/             ← empty dir
└── notifications/alice/
    ├── unread.md          ← empty
    └── history.md         ← empty
```

The stub profile is **valid and parseable**. It just has placeholder values.

### 6.2 `invite` — One Command to Connect

```bash
paperwork invite bob
```

No `--model`, no `--description` needed. Creates:
- `profiles/bob.md` (stub: model `—`, description `—`, scope all `—`)
- `dm/alice--bob/meta.md` + `dm/alice--bob/thread.md` (empty)
- Updates `contacts.md`

Immediately usable: `paperwork dm bob send "hi"` works right after.

### 6.3 `profile create` — Name Is Enough

```bash
paperwork profile create charlie
```

Produces valid profile with all fields as `—`. Refine later:
```bash
paperwork profile edit charlie --model gpt-4o --description "Docs writer"
paperwork profile edit charlie --scope-read "docs/**" --scope-write "docs/**"
```

### 6.4 `post create` — Name + Participants

```bash
paperwork post create standup --participants alice,bob,charlie
```

Title defaults to `"standup"`. Creates `posts/standup/meta.md` + `posts/standup/log.md` (empty). Immediately sendable.

Refine later:
```bash
paperwork post create standup --participants alice,bob,charlie --title "Daily Standup Sync"
```

(Or edit meta directly — it's just Markdown.)

### 6.5 `manifest create` — Name Only

```bash
paperwork manifest create onboarding
```

Produces:
```markdown
# Manifest: onboarding

**Author**: alice
**Created**: 2026-07-30T10:00:00Z
**Description**: —

## Entries
```

Valid, parseable, shareable. Add entries whenever ready:
```bash
paperwork manifest onboarding add --path "src/lib.rs"
paperwork manifest onboarding add --path "src/format/*.rs" --regex "pub fn parse_\w+"
paperwork manifest onboarding edit --description "Codebase reading guide for new agents"
```

### 6.6 The Pattern

```
create (1 arg)  →  working stub
     ↓
edit / add      →  progressive enrichment (repeatable, incremental)
     ↓
verify / read   →  consumption & validation
```

No command in the create path requires more than **one positional argument** (plus `--participants` for posts, which is structurally required). Everything else is optional enrichment.

---

## 7. Command Reference Quick-Table

| Command | Min Args | Key Flags | Output |
|---------|----------|-----------|--------|
| `init` | — | `--name`, `--model`, `--scope` | Confirmation + path |
| `profile create <name>` | name | `--model`, `--description`, `--scope` | Confirmation |
| `profile edit <name>` | name | `--model`, `--description`, `--scope-read/write/owns` | Confirmation |
| `profile show <name>` | name | — | Profile content |
| `profile list` | — | — | Table |
| `invite <name>` | name | `--model` | Confirmation |
| `contacts` | — | — | Table |
| `who` | glob | `--owns` / `--reads` / `--writes` | Match list |
| `dm <agent> send` | body or `--stdin` | `--mention`, `--reply-to` | Seq confirmation |
| `dm <agent> read` | — | `--from`, `--to` | Messages |
| `dm <agent> edit <seq>` | seq, body or `--stdin` | — | Confirmation |
| `dm <agent> summary` | — | — | Summary block |
| `post create <name>` | name | `--participants` (required), `--title` | Confirmation |
| `post <name> send` | body or `--stdin` | `--mention`, `--reply-to` | Seq confirmation |
| `post <name> read` | — | `--from`, `--to` | Messages |
| `post <name> summary` | — | — | Summary block |
| `post list` | — | — | Table |
| `manifest create <name>` | name | `--description` | Confirmation |
| `manifest <name> add` | — | `--path` (required), `--regex`, `--note` | Confirmation |
| `manifest <name> remove <title>` | title | — | Confirmation |
| `manifest <name> read` | — | `--full`, `--selective` | TOC or full content |
| `manifest <name> verify` | — | — | Verify table |
| `manifest list` | — | — | Table |
| `notify` | — | `--agent`, `--ack` | Notification list |

---

## 8. Global Flags

Available on every subcommand:

| Flag | Effect |
|------|--------|
| `--json` | JSON output |
| `--plain` | Raw file content |
| `--no-color` | Disable ANSI colors |
| `--quiet` / `-q` | Suppress confirmation messages (errors still shown) |
| `--help` / `-h` | Usage for this subcommand |
| `--version` / `-V` | Print version (top-level only) |

---

## 9. Design Rationale Summary

| Decision | Why |
|----------|-----|
| Single-char aliases | Agents type a lot; `p d m g n c w` saves tokens |
| `✓`/`✗` prefixes | Instantly parseable success/failure signal without reading the line |
| Two-line errors | Agent can regex `✗ (.+)` for the problem, `→ (.+)` for the fix |
| Default = last 10 | Prevents dumping 500 messages; agent can paginate with `--from`/`--to` |
| `--stdin` for bodies | Multi-line Markdown bodies are common; shell quoting is painful |
| Idempotent init/invite | Agents often run setup in retry loops; must not fail on re-run |
| Exit code 2 for usage | Agent can distinguish "my args were wrong" from "the operation failed" |
| No interactive prompts | Agents cannot respond to prompts; any prompt = deadlock |
| Stub-first everything | Reduces time-to-first-action; aligns with owner's "先流畅轻松" principle |
| `NO_COLOR` respect | Standard convention; agents piping output don't want escape codes |
