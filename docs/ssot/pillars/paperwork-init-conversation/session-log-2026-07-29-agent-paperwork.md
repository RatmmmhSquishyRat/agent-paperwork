# Session Log — Agent Paperwork Design Discussion

**Date**: 2026-07-29  
**Participants**: User (owner), Assistant (delivery)  
**Purpose**: Conceptual design discussion for `agent paperwork` — a file-based agent collaboration CLI toolkit.

---

## User Message 1

你如何理解基于文件的agent上下文渐进式披露?

## Assistant Message 1

This is a great architectural question. Here's my understanding of **file-based agent context progressive disclosure**:

### Core Idea

It's a design pattern where an agent's context (instructions, knowledge, capabilities) is organized as files on disk, and revealed to the agent **incrementally based on need** rather than loaded all at once.

### Why It Exists

The fundamental tension: agents have finite context windows, but real projects have unbounded complexity. You can't stuff everything into the prompt upfront — it's wasteful, noisy, and eventually impossible.

### The Layered Structure

Typically it looks like a pyramid:

1. **Always-on layer** — minimal identity, rules, tool schemas. Loaded every turn. Small and stable.
2. **Index/overview layer** — a table of contents, module tree, or memory overview. Tells the agent *what exists* without revealing details.
3. **On-demand layer** — actual file contents, skill definitions, code modules. Loaded only when the task touches them.
4. **Deep-dive layer** — full source, historical context, related memories. Retrieved via explicit search/read actions.

### Key Mechanisms

| Mechanism | Role |
|-----------|------|
| Glob/Grep/Read | Agent actively pulls file content when needed |
| Index files (e.g., `SKILL.md`, `SERVER_METADATA.json`) | Lightweight descriptors that let the agent decide *whether* to load more |
| Semantic search | Bridges "I need X" → "it's in file Y" without the agent knowing paths upfront |
| Memory/knowledge trees | Hierarchical pointers that can be explored before fetching full content |

### The Design Insight

The file system itself becomes the **context management infrastructure**:

- **Discoverability**: directory structure + naming conventions let the agent navigate without a global map.
- **Granularity**: one concept per file means you load exactly what's relevant.
- **Composability**: skills, rules, configs are independent files that can be combined per-task.
- **Persistence**: unlike in-memory context, files survive across sessions and can be versioned.

### The "Progressive" Part

The agent's workflow mirrors how a human developer works:

> See project structure → read a relevant file → follow a reference → search for a symbol → read its definition → ...

Each step **narrows uncertainty** before spending context budget. The agent doesn't read all 500 files; it reads 3–5 that matter for the current sub-task.

### Trade-offs

- **Pro**: Scales to arbitrarily large projects; context stays fresh and relevant.
- **Pro**: Files are inspectable/debuggable by humans (unlike opaque embeddings).
- **Con**: Requires good indexing/naming — if the agent can't *find* the right file, the context is effectively lost.
- **Con**: Multi-hop retrieval costs latency (search → read → search again).

In short: it treats the file system as a **lazy-loading context database** where the agent is both the query engine and the consumer, and "progressive disclosure" is the strategy that keeps the context window a scarce, well-spent resource.

---

## User Message 2

如果我把一些常用的经常一起阅读的文件, 通过cli输出一次性给到agent, 这对于长上下文模型来说, 是反模式还是有益模式?

## Assistant Message 2

This is a genuinely nuanced question — the answer is **it depends on the granularity and stability of the grouping**, and both outcomes are real.

### When It's Beneficial

**Tightly-coupled, small, stable bundles** — e.g.:

```bash
cat src/types/order.ts src/services/order.ts src/services/order.test.ts
```

This works well because:

- **Eliminates retrieval overhead**: 3 search-read cycles → 1 tool call. For files that are *always* co-read, the discovery step is pure waste.
- **Preserves relational context**: The agent sees type + impl + test simultaneously, enabling cross-file reasoning without "memory" of a previous read.
- **Long-context models handle this well**: A few thousand tokens of tightly-related code is well within the "effective attention" range.
- **Reduces agent decision fatigue**: The agent doesn't need to figure out "what else should I read?" — the bundle encodes domain knowledge about coupling.

### When It Becomes an Anti-Pattern

**Large, loosely-related dumps** — e.g.:

```bash
cat src/**/*.ts  # "here's the whole module"
```

Problems:

| Issue | Why it matters |
|-------|----------------|
| **Lost-in-the-middle** | Even 200K models show degraded attention to content in the middle of very long inputs. Critical details get buried. |
| **Signal dilution** | 80% of the bundle may be irrelevant to the specific sub-task. The agent must now *filter* instead of *retrieve*. |
| **False economy** | You saved 3 tool calls but spent 20K tokens of context budget that could have been used for deeper reasoning or more iterations. |
| **Rigidity** | The bundle encodes a *static* assumption about what's needed. Tasks are dynamic — the agent might need file X instead of file Y today. |
| **Anchoring bias** | Pre-loaded context can over-anchor the agent's approach, even when the task calls for a different angle. |

### The Heuristic

I'd frame it as:

> **Bundle by coupling, not by directory.**

Good bundle criteria:
- Files that **reference each other's symbols** directly (type ↔ impl ↔ test)
- Total size stays under ~3–5K tokens
- The grouping is **stable across tasks** (not task-dependent)
- You'd feel confident saying "you can't understand A without B"

Bad bundle signals:
- "These files are in the same folder"
- "They're all related to feature X" (too broad)
- The bundle exceeds ~10K tokens
- Different sub-tasks within the feature need different subsets

### A Hybrid Pattern That Works Well

```bash
# Layer 1: Always provide a compact index
cat ARCHITECTURE.md  # 50-line overview with file roles

# Layer 2: Bundle the tight core
cat src/types/order.ts src/services/order.ts

# Layer 3: Let the agent pull the rest on demand
# (agent uses grep/read for edge cases, tests, configs)
```

This gives the agent **orientation + core context + agency** without over-committing the context window.

### Bottom Line

For long-context models, the question isn't "can it handle the tokens?" (it can). The question is **"does this information earn its position in the attention budget?"** Small, coupled bundles — yes, genuinely helpful. Large thematic dumps — you're trading retrieval latency for reasoning quality, and that trade is usually worse than you'd expect.

---

## User Message 3

好吧, 我说说我想做的东西 - 我想做的是一个agent之间基于文件的合作util toolkit cli, 叫做 agent paparwork:
几个主要功能: 
1. 完全基于文件的agent role profile, 描述自己是谁, 有什么模型配置, 将会干什么, 以及最重要的涉及读写文件范围和只读文件范围.
2. 完全基于文件的通讯录(各个profile以及对应私聊文件夹的路径), 私聊(DM)和群聊(meeting vis GDM), 全append无阻碍编辑.
3. 完全基于文件路径列表(也支持相对路径和blob, 乃至行号/hashline范围)的, 一个快速阅读文件清单. 不再需要agent渐进式披露, agent可以自己总结固化阅读的最佳实践, 落盘到read manifest 文件, 让其他agent能正确快速掌握需要理解的文本文件. agent读取时候会先读到目录, 然后可以选择直接全量阅读, 或者根据路径自己手动选择性阅读.

这些基本上是collaboration软件中我认为对于agent来说最有价值和值得做的部分. 

你如何理解这个项目, 你认为是否有任何我没有想到的地方, 或者需要改进的?

## Assistant Message 3

This is a well-scoped project. Let me share my understanding and then the gaps I see.

### My Understanding

**Agent Paperwork** is essentially: *Unix philosophy applied to agent collaboration.* Everything is a file, everything is append-only, everything is human-readable. The CLI is a thin orchestration layer over filesystem primitives.

The three pillars map to:

| Pillar | Analogy | Core value |
|--------|---------|------------|
| Role Profile | Unix `/etc/passwd` + capabilities | Identity + **permission boundary** |
| Contacts + DM/Meeting | Email + mailing list | Async, append-only communication |
| Read Manifest | `Makefile` dependencies / `.gitattributes` | **Codified reading expertise** — the most novel part |

The read manifest is the standout idea. It inverts the usual pattern: instead of each agent rediscovering "you need to read A, then B, then lines 40–80 of C," an experienced agent **publishes** that knowledge as a reusable artifact. This is essentially *institutional knowledge* for agent teams.

### Things Worth Considering

#### 1. Read State / "What's New?"

Append-only solves *write* contention, but not *read* tracking. Agent A appends 5 messages. Agent B comes back later — how does it know where it left off?

Options:
- Per-reader offset file: `.paperwork/dm/alice-bob/.bob-offset` (byte or line count)
- Or: each agent tracks its own high-water mark in its profile/state

Without this, agents either re-read everything (wasteful) or miss updates (dangerous).

#### 2. Message Framing in Append-Only Files

Multiple agents appending to a meeting file need **unambiguous delimiters**. Raw text appending can interleave at the OS level (two `>>` writes racing). Consider:

- A structured envelope per message (e.g., YAML front-matter block with sender, timestamp, seq)
- Or: the CLI mediates writes with `flock` / advisory locking
- Minimum: atomic-line-append guarantee (each message is a single logical unit the CLI writes in one `write()` syscall)

#### 3. Read Manifest Staleness

A manifest says "read lines 40–80 of `parser.ts`." Someone refactors `parser.ts`. Now the manifest points at garbage.

Mitigations:
- Store a **content hash** (or git SHA) alongside each entry. CLI can flag "stale: 3/7 entries drifted."
- Support `--verify` that checks hashes and reports which entries need regeneration.
- Allow manifests to be regenerated by an agent and re-published (versioned).

#### 4. Profile: Declared vs. Enforced Scope

The read/write scope in a profile is a **contract**. But is it:
- Honor-system (agent self-declares, trusts others to respect)?
- CLI-enforced (the `read`/`write` subcommands actually validate against scope)?

I'd argue: **declare in profile, enforce in CLI.** The `paperwork read <agent> <path>` command checks scope before returning content. This makes the boundary real rather than aspirational. Cost is low, trust gain is high.

#### 5. Task Handoff Protocol

Communication (DM/meeting) handles *discussion*, but not *work transfer*. "I finished implementing X, please review" needs structure:

- A lightweight `handoff` file convention: what was done, which files changed, acceptance criteria, status (`pending` / `accepted` / `rejected`)
- Or: keep it minimal — just a message convention (structured YAML in DM) rather than a new file type

Without this, "collaboration" stays at the chat level and doesn't reach the *work* level.

#### 6. Discovery / Routing

Contacts list tells you *who exists*. But in a team of 10 agents, **who should I ask?** Consider adding to profiles:
- `capabilities: [rust, code-review, architecture]`
- `owns: [src/parser/**, src/lexer/**]`

Then the CLI could support `paperwork who --owns src/parser/` → returns the responsible agent.

#### 7. Growth / Archival

Append-only files grow forever. A long-running meeting file becomes a 10K-line monster that no agent wants to read.

- Support `paperwork archive <meeting>` → moves to `archive/2026-07.md`, resets the active file with a pointer.
- Or: time-based rotation built into the CLI.

#### 8. Bootstrapping

`paperwork init` needs to be really smooth. First agent in a workspace should be able to:
```bash
paperwork init --name alice --scope "src/**:rw,docs/**:r"
paperwork invite bob  # creates bob's profile stub + DM folder
```

The cold-start experience will determine adoption.

### One Structural Suggestion

Consider making the **directory layout itself** part of the spec:

```
.paperwork/
├── profiles/
│   ├── alice.yaml
│   └── bob.yaml
├── contacts.yaml          # registry: name → profile path + dm path
├── dm/
│   ├── alice-bob/
│   │   ├── .meta.yaml     # participants, created
│   │   └── thread.md      # append-only
│   └── ...
├── meetings/
│   ├── standup-0729/
│   │   ├── .meta.yaml
│   │   └── log.md         # append-only, multi-party
│   └── ...
├── manifests/
│   ├── onboarding-alice.md
│   └── parser-module.md
└── state/
    ├── alice.offset.yaml  # read high-water marks
    └── ...
```

If this layout is **the spec** (not just an implementation detail), then any tool/agent that understands the layout can interoperate — even without your CLI. The CLI becomes a convenience, not a dependency.

### Verdict

The core instincts are right: file-based, append-only, human-readable, agent-publishes-knowledge-for-other-agents. The read manifest idea is genuinely novel in the agent-tooling space — most tools focus on *giving* context, not on *agents teaching each other how to read*.

The main risks are: (1) staleness of manifests, (2) read-state tracking, and (3) the gap between "chat collaboration" and "work collaboration." Nail those three and this is a very useful primitive.

---

## User Message 4

1. agent不需要显式记录自己读到哪里, 因为DM文件本来就是append语义的, 而tool call又是agent自己产生的, 所以它只需要接着上次的msg序号往下读就好了. 并且, GDM还可以被读取状态概要, 标题, 一共多少个msg, 上次更新时间和用户, 各个msg的摘取速览等等. 因此, agent在读取具体消息时候, 需要做的只有指定序号范围就够了, 和读取文件行号范围是类似的.
1.5. 我想到DM中应该允许agent@别的profile, 这些@消息会被记录于profile中, 并通过cli提醒主人, 提醒后归入历史通知记录, 存储于在单独文件中.
1.75. 并且, 这里我还想到可以根据@profile或者reply的消息来筛选查看相关message, 然后reply默认带有@, 和主流DM应用一致.
2. 所有这个cli相关的文件, 都是managed, 这属于core负责的机制.
3. 我的建议是, 既不是行号, 也不是hashline, 而是实现正则表达式抽取文字, 甚至允许多组匹配输出. 失败情况下, 则就可以知道文件被更新; 另外, 确实有可能改变但是失败静默, 或者预料内持续成功, 这种情况下文件版本应当单独建模, 也就是在具体路径后, 记录blob path的hash即可, 调用时候校验即可.
4. cli不会也无法enforce读写范围, 所以profile中的范围都是自主负责, 动态协商调整的.
5. DM只是append onlu, 无法进行插入和删除而已, 而这两种操作在工单中实际上都是污染. 因此, 有append only, 有@, 有编辑自己消息能力, 这些功能足够工单使用了, 只不过是cli harness层级没有语义说明罢了.
6. 有道理, paperwork应当支持查询指定路径范围A内的profile谁读写/只读/owns(这个语义不错)指定路径范围内的文件.
7. 这要看你到底关注哪一点, 是可读性还是体积, 文件太大需要全部归档, agent自己来操作就行; 可读性不好, 可以让post owner/users给summary, 尤其是过长的消息, 过长的消息序列段, 给summary就行.
8. 有道理, 先流畅轻松甚至stub创建, 然后再细致编辑, cli的全程UX都应该遵循这种体验.

我觉得你提的殿都很有道理, 因此上述我进行了完整的补全.

## Assistant Message 4

Your resolutions are clean and each one dissolves the problem rather than over-engineering around it. A few reflections on the refined design:

### What's Particularly Sharp

**Regex-as-anchor in manifests** (point 3) is the best answer to staleness. It reframes the question from "is line 47 still relevant?" to "does the *thing I care about* still exist and say what I expect?" — which is how a human would verify too. The three-state outcome (match success / match failure / silent drift) maps naturally to:

- Match succeeds + hash matches → fresh, use directly
- Match succeeds + hash differs → content shifted but structure holds, probably fine
- Match fails → **stale, needs re-curation**

This gives the reading agent a confidence signal, not just raw content.

**Append-only as work semantics** (point 5) — you're right that insert/delete in a collaboration log is *lying about history*. A ticket system that only appends + allows self-edit is essentially a git commit log for conversations. The "lack of semantic layer" is actually a feature: the CLI stays dumb, agents impose meaning.

**@-mention → profile notification → history file** (point 1.5) gives you an async interrupt mechanism without any daemon or polling infrastructure. The CLI just checks "does your notification file have unread entries?" on next invocation. Very Unix.

### One Small Thought on Point 6

The query `paperwork who --scope src/parser/** --access owns` implies profiles declare path patterns. If two agents both `owns: src/parser/**`, that's either a conflict or a shared ownership. You might want the CLI to surface overlaps:

```bash
paperwork who --owns "src/parser/**"
# alice (owns)
# bob (owns)   ← overlap warning?
```

Not a blocker — just worth deciding whether overlap is legal (yes, probably, like CODEOWNERS with multiple entries) or should warn.

### The Design Feels Complete

The core loop is:

```
profile (identity + scope)
  → contacts (discovery)
    → DM/GDM (communication, append-only, @/reply)
      → manifest (codified reading knowledge, regex-anchored, hash-versioned)
```

Each layer is a file, each operation is a CLI subcommand, each artifact is human-readable. No database, no server, no daemon. An agent team can be bootstrapped with `paperwork init` and a few `cat >` commands, and scales to complex multi-agent workflows without the infrastructure changing.

The one thing I'd nail down early in implementation: the **managed file format spec** (point 2). Since the CLI owns structure, the exact envelope format for messages (sequence number, sender, timestamp, @mentions, reply-to) is effectively your wire protocol. Get that stable and everything else composes around it.

Solid project. The read-manifest-as-published-expertise idea alone makes this worth building — I haven't seen it done as a first-class primitive elsewhere.

---

## User Message 5

1. There is no core loop in design, the DM, the profile, the manifest-baed brief, they can be all relevant to each other, but no hierarchy, no fixed usage loop, each forms a individual usage tool themselves.
2. To be a owner, I wont down into implementation details, protocols or whaterver, at most just adrs. I write ssot and pillars, I play your delivery and then give feedback, that will be the primary intervention form.

Now generate a session log file with all messages verbatimly, losslessly, as an ssot conversation record.

---

*End of session log.*
