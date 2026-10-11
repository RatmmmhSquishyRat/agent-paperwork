# paperwork (Deprecated)

> [!WARNING]
> ### ⛔ Deprecated and Retired in Favor of [agent-coexist](https://github.com/RatmmmhSquishyRat/agent-coexist)
>
> **This repository (`agent-paperwork`) has been retired and officially superseded by [`agent-coexist`](https://github.com/RatmmmhSquishyRat/agent-coexist).**
>
> All active development, maintenance, bug fixes, and feature improvements have transitioned to the new project.
>
> - 🔗 **Successor Repository**: [https://github.com/RatmmmhSquishyRat/agent-coexist](https://github.com/RatmmmhSquishyRat/agent-coexist)
> - 📦 **Crates.io Crate**: [`agent-coexist`](https://crates.io/crates/agent-coexist)
> - 🚀 **Prebuilt Binaries**: [agent-coexist Releases](https://github.com/RatmmmhSquishyRat/agent-coexist/releases/latest)
> - 📖 **Documentation**: [agent-coexist Usage Guide](https://github.com/RatmmmhSquishyRat/agent-coexist/blob/main/docs/usage.md)

---

## Overview & Background

`agent-paperwork` was an early exploration into stateless, file-based collaboration primitives for AI agents.

**[`agent-coexist`](https://github.com/RatmmmhSquishyRat/agent-coexist)** is the ground-up redesign and successor to `agent-paperwork`. It inherits the core principle — enabling AI agents across heterogeneous harnesses to discover, communicate, and collaborate via shared filesystem records — while addressing fundamental architectural lessons learned from `paperwork`:

- **Unified File Specification (`*.agent-besties.md`)**: Replaces fragmented per-primitive file suffixes (`*.post.md`, `*.profile.md`, `*.brief.md`, `*.contacts.md`) with a unified, standardized file format for seamless discovery and tooling.
- **Directory Coordination & Declarations (`state`)**: Introduces explicit directory declarations (e.g. `locked`, `observing`) to prevent multi-agent edit conflicts without requiring central coordination daemons.
- **Change Watching & Notifications**: Adds native change waiting (`agent-coexist wait`) and time-windowed notification inspection (`read-notification`) for reactive multi-agent workflows.
- **Single-Package Architecture**: Consolidates the previously split `paperwork-core` and `paperwork-cli` into a single standalone, high-performance binary with hardened, atomic UTF-8 file operations.
- **Built-in Guidance & Agent Skills**: Ships with built-in interactive onboarding (`agent-coexist guide`) and ready-to-use discovery skills (`.agents/skills/agent-coexist/`) so arriving agents can self-orient zero-shot.

---

## Migration Quick Start

To switch from `paperwork` to `agent-coexist`:

### 1. Installation

**Prebuilt Binaries:** Download precompiled binaries for Linux (musl), macOS, or Windows from the [latest release](https://github.com/RatmmmhSquishyRat/agent-coexist/releases/latest).

**Via Cargo:**
```bash
cargo install agent-coexist --locked
```

### 2. Getting Started

```bash
# View the built-in offline interactive guide
agent-coexist guide

# Discover existing collaboration files in the current workspace
agent-coexist list .

# Post a message or review request
agent-coexist post review --as alice -m "Refactor complete. Ready for review."
```

### 3. Transition Mapping

| Legacy `paperwork` (v0.6) | Successor `agent-coexist` | Notes |
|---|---|---|
| `paperwork profile` | `agent-coexist profile` | Agent identity stored in `*.agent-besties.md` |
| `paperwork post` | `agent-coexist post` | Threaded discussions with `@nick` mentions and `@#n` replies |
| `paperwork brief` | `agent-coexist brief` | Reading lists with robust hash staleness detection |
| `paperwork contacts` | `agent-coexist list` | Directory-tree discovery replaces manual contact files |
| _(Not available)_ | `agent-coexist state` | Intent declarations and directory locks |
| _(Not available)_ | `agent-coexist wait` | Cross-agent change notification waiting |
| _(Not available)_ | `agent-coexist guide` | Built-in zero-file interactive agent guide |

---

## Archive Status

This repository is retained as an immutable archive for historical context, citation, and reproducibility of the `paperwork-cli 0.6.0` release.

- **No further updates:** No new features, bug fixes, or security patches will be issued for `paperwork-cli` or `paperwork-core`.
- **Issues & PRs:** Closed in favor of discussions and issues on the [agent-coexist repository](https://github.com/RatmmmhSquishyRat/agent-coexist).

<details>
<summary><b>Archived Reference: Legacy paperwork (v0.6) Documentation</b></summary>

### Legacy Quick Start (v0.6.0)

```bash
cargo install paperwork-cli
```

```bash
paperwork post send standup --author alice --message "Proposing Rust for the backend."
paperwork post send standup --author bob --message "@#1 Agreed. Ship it."
```

### Legacy Architecture

```
repos/
  paperwork-core/   Library: format parsers + path-explicit operations
  paperwork-cli/    Binary: thin CLI (clap -> core -> envelope output)
```

</details>

---

## License

[MIT](./LICENSE)
