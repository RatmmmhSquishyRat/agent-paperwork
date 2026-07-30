# Agent Paperwork — Product Requirements Document

## Vision

A zero-infrastructure, file-based collaboration CLI toolkit for AI agent teams.
Unix philosophy: everything is a file, everything is append-only, everything is human-readable.

## Problem Statement

Multi-agent workflows lack a minimal, composable collaboration primitive. Existing tools impose servers, databases, or proprietary protocols. Agents need:
- Identity & scope declaration
- Async communication (1:1 and group)
- Codified reading expertise transfer

All achievable with filesystem semantics alone.

## Product Scope

### In Scope

| Feature | Description |
|---------|-------------|
| Role Profiles | Richly-marked Markdown files: identity, model config, work description, read/write/owns scope |
| Contacts Registry | Maps agent names → profile path + DM folder path |
| DM (Direct Message) | 1:1 append-only thread with seq numbering, @-mentions, reply-to |
| Post / GDM (Group DM) | Multi-party append-only log with same semantics as DM |
| Notifications | @-mention → profile notification → CLI surfacing → history archive |
| Read Manifests | Path lists (glob, regex extraction, multi-group), blob-hash staleness detection |
| Scope Query | `who --owns/--reads/--writes <path-pattern>` across profiles |
| Managed Layout | `.paperwork/` directory as the spec; CLI owns structure |

### Out of Scope

- Scope enforcement (honor-system only; agents self-govern)
- Daemon / polling / server processes
- Semantic message layer (CLI stays dumb; agents impose meaning)
- Automatic archival (agent-driven, not CLI-driven)
- Network transport (local filesystem only)

## User Personas

1. **Agent** — primary user; reads/writes via CLI subcommands or raw file ops
2. **Human Operator** — inspects files, bootstraps workspaces, reviews threads
3. **Orchestrator (Main Agent)** — uses CLI to coordinate team, query ownership

## Core Requirements

### R1: Profile Management
- `paperwork profile create <name>` — stub-first creation, then refine
- `paperwork profile edit <name>` — update fields
- `paperwork profile show <name>` — display profile
- `paperwork profile list` — all registered agents
- Profile fields: `name`, `model`, `description`, `scope.read[]`, `scope.write[]`, `scope.owns[]`
- All profile data stored as richly-marked Markdown (per ADR-002)

### R2: Contacts & Discovery
- `paperwork invite <name>` — create profile stub + DM folder between inviter and invitee
- `paperwork contacts` — list all agents with paths
- `paperwork who --owns|--reads|--writes <glob>` — query scope declarations

### R3: DM / Post (GDM) Communication
- `paperwork dm <agent> send "message"` — append to 1:1 thread
- `paperwork dm <agent> read [--from N] [--to M]` — read by seq range
- `paperwork dm <agent> summary` — msg count, last update, snippet previews
- `paperwork dm <agent> edit <seq> "new body"` — self-edit own last message (in-place)
- `paperwork post create <name> --participants a,b,c` — create GDM thread
- `paperwork post <name> send "message"` — append to group log
- `paperwork post <name> read [--from N] [--to M]` — read by seq range
- `paperwork post <name> summary` — title, participant count, msg count, last update, snippet previews
- `paperwork post list` — list all posts
- Messages support: `--mention <agent>`, `--reply-to <seq>`
- Reply carries implicit @-mention of original sender

### R4: Notifications
- @-mention writes notification entry to target profile's notification file
- `paperwork notify` — show unread notifications for current agent
- `paperwork notify --ack` — mark as read, move to history file
- Filter: `--mention <agent>`, `--reply-to <seq>`

### R5: Read Manifests
- `paperwork manifest create <name>` — stub manifest
- `paperwork manifest <name> add --path <p> [--regex <r>] [--groups <g>] [--note <n>]` — add entry
- `paperwork manifest <name> remove <entry-title>` — remove entry
- `paperwork manifest <name> read [--full | --selective]` — TOC first, then selective
- `paperwork manifest <name> verify` — check regex matches + blob hashes
- `paperwork manifest list` — list all manifests
- Entry fields: `path` (relative/glob), `regex` (optional, multi-group), `hash` (blob SHA)
- Three-state verify result: fresh / shifted / stale

### R6: Workspace Bootstrap
- `paperwork init [--name <agent>] [--model <id>] [--scope <spec>]` — create `.paperwork/` layout
- Stub-first UX: everything creatable in one command, refinable later

## Non-Functional Requirements

| NFR | Target |
|-----|--------|
| Distribution | Single static binary, no runtime deps |
| Platform | Windows, macOS, Linux |
| Performance | <50ms for any single subcommand |
| Concurrency | Advisory file locking + O_APPEND + single write() per message (per ADR-007) |
| Human readability | All managed files are richly-marked Markdown (per ADR-002) |
| Interoperability | Layout is the spec; tools without CLI can interoperate |

## Success Criteria

1. Two agents can bootstrap, exchange messages, and read manifests using only the CLI
2. All `.paperwork/` files are human-readable richly-marked Markdown
3. Manifest staleness detection correctly identifies drifted files
4. No data loss under concurrent append (atomic write guarantee)
5. Full workflow achievable without network access
