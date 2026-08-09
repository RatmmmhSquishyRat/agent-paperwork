# Agent Paperwork 仓库调研报告

**调研日期**：2026-08-08
**调研对象**：`c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork`（当前版本 v0.4.0）
**调研方式**：直接阅读 `repos/paperwork-core`、`repos/paperwork-cli` 全部源码、测试文件、测试语料（`test-v03/`、`test-v04/`、`_fix/`）及 `docs/` 下全部评审与 ADR 文档。所有结论均以实际代码/文件为依据，并标注来源路径。

---

## 1. 项目概览与整体架构

### 1.1 项目定位

Agent Paperwork 是一个面向 AI agent 的**无状态、基于文件的协作原语 CLI 工具**（`README.md` 第 3、14 行）：无服务器、无数据库、无登录、无 workspace，每条命令接收显式文件路径，文件本身即 SSOT（single source of truth）。三支柱为：

| 支柱 | 命令 | 文件类型 | 用途 |
|------|------|----------|------|
| Identity（Role Profiles） | `profile` | `*.profile.md` | agent 的名称、模型、职责范围 |
| Communication（File-Based） | `post` | `*.post.md` | append-only 消息线程，支持 reply 与 @mention |
| Knowledge（Read Manifests） | `brief` | `*.brief.md` | 带 staleness 检测的阅读清单 |
| Directory（辅助） | `contacts` | `*.contacts.md` | profile 路径列表 |

（来源：`README.md` 第 50–55 行）

### 1.2 Cargo workspace 组织

- 根 `Cargo.toml`：仅声明 workspace，成员为 `repos/paperwork-core` 与 `repos/paperwork-cli`，resolver = "2"。
- `repos/paperwork-core`（v0.4.0，库）：依赖 `regex`、`chrono(serde)`、`thiserror`、`serde`、`sha2`、`fs2`；dev 依赖 `tempfile`（`repos/paperwork-core/Cargo.toml`）。定位为可独立发布的库，供 IDE 插件 / agent harness 复用（`README.md` 第 247 行）。
- `repos/paperwork-cli`（v0.4.0，二进制名 `paperwork`）：依赖 `paperwork-core`（path 引用）、`clap(derive)`、`serde_json`、`anyhow`、`chrono`；dev 依赖 `tempfile`、`assert_cmd`、`predicates`（`repos/paperwork-cli/Cargo.toml`）。
- 发布顺序由 `publish.ps1` 固化：先发 `paperwork-core`，等待 30 秒让 crates.io 索引更新，再发 `paperwork-cli`。
- CI（`.github/workflows/ci.yml`）：Linux/macOS/Windows 三平台矩阵，跑 `cargo test --workspace` + clippy（`-D warnings`）+ release 构建后的端到端 smoke 脚本（含错误信封、空正文拒绝、garbage validate 拒绝等断言）。

### 1.3 paperwork-core 分层

`repos/paperwork-core/src/lib.rs` 定义全部领域类型并导出四个模块：

- **领域类型**（lib.rs）：`Profile`、`ContactEntry`、`Message{seq, sender, timestamp, to, reply_to, mentions, body}`、`ThreadSummary`、`ManifestEntry{title, path, hash, regex, groups, note}`、`Manifest`、三态枚举 `VerifyResult{Fresh, Shifted, Stale}`。
- **`format/` 层**（纯字符串解析/序列化，不碰文件系统）：`mod.rs`（共享工具）、`profile.rs`、`thread.rs`、`manifest.rs`、`contacts.rs`。这是全部格式的权威实现。
- **`ops/` 层**（无状态、路径显式的文件系统操作）：`mod.rs` 开头注释即声明 "No workspace root, no init, no state. Files are independent — no cross-references managed by the CLI."；含 `thread.rs`、`profile.rs`、`manifest.rs`（brief 操作）、`contacts.rs`。
- **`error.rs`**：统一错误类型 `PaperworkError`（详见 §4.4）。
- **`hash.rs`**：SHA-256 blob 哈希（`hash_bytes` / `hash_file`，小写 hex），注释注明用于 manifest 条目校验（invariant I7）。

### 1.4 paperwork-cli 分层

- `src/main.rs`：clap 入口，5 个子命令 `profile(p)`、`post`、`brief(b)`、`contacts(c)`、`validate(v)`；全局 flag `--json`、`--plain`、`-q`；错误时 downcast 为 `PaperworkError` 输出结构化错误信封，exit 1。
- `src/output.rs`：统一输出信封协议。成功：`ok <command> <conclusion>` + `key: value` 字段行 + 可选 `---` 分隔的 body 行；失败：`error <category>: <message>` + `fix:` + `example:`；`--json` 包装为 `{"status","command","conclusion",...}`。
- `src/cmd/mod.rs`：`Context{mode, quiet}` 与 `ensure_suffix()`——路径不以类型后缀结尾时自动补 `.profile.md/.post.md/.brief.md/.contacts.md`（裸 `.md` 会被替换）。
- `src/cmd/*.rs`：每个子命令的参数解析与输出组装，逻辑全部委托给 core ops。

### 1.5 测试语料的用途

- `test-v03/`：v0.3 评审会话的手工冒烟产物（`docs/reviews/v0.3-review-2026-08-01.md` 的测试现场），含正常样例（`alice.profile.md`、`standup.post.md`、`onboarding.brief.md`、`team.contacts.md`、自动建线程的 `quick-chat.post.md`）与坏例（`broken.post.md` = "this is not valid"、`garbage.profile.md` = "random text no structure"）；`src/main.rs`、`src/lib.rs` 是 brief 条目指向的被哈希文件。
- `test-v04/`：v0.4 评审冒烟产物（对应 `docs/reviews/v0.4-review-2026-08-01.md`），结构同上，另含 `auto-thread.post.md`（验证 send 自动建文件）、`garbage.post.md` 坏例。
- `_fix/`：一次修复验证会话的产物（`guide.brief.md` 引用同目录 `main.rs`，`standup.post.md` 为双消息线程）。
- 自动化测试：`repos/paperwork-core/tests/ops_tests.rs`（595 行，覆盖 profile/thread/brief/contacts ops 与并发锁）、`repos/paperwork-cli/tests/cli_integration.rs`（382 行，assert_cmd 端到端断言输出信封）。README 称全量 119 个测试。

---

## 2. Managed File Format 逐一详解

> **重要核实结论（修正历史约定）**：当前代码中**不存在 YAML frontmatter**。历史记忆中"线程文件用 YAML `---` 作为消息分隔符"的说法，在 v0.3 之后的实现里已演化为：`---` 作为 **Markdown 水平线**承担消息边界角色，且解析是"边界锚定 + fence 感知"的（并非 YAML 多文档解析；v0.3 review 仅称其风格"YAML-style"，见 `docs/reviews/v0.3-review-2026-08-01.md` 第 148 行）。"正文内分割线用 `***`"只是 v0.3 时期的**使用侧约定**（见该 review 第 24 行），代码中没有任何 `***` 相关处理——因为 v0.3 起正文被 4 反引号 fence 包裹，正文内的 `---` 本身已安全。

### 2.1 Profile 文件（`*.profile.md`）

**权威实现**：`repos/paperwork-core/src/format/profile.rs`；操作：`repos/paperwork-core/src/ops/profile.rs`；CLI：`repos/paperwork-cli/src/cmd/profile.rs`。

**Schema**（format/profile.rs 头部文档注释 + 解析逻辑）：

```markdown
# <name>                    ← H1，agent 名（必需）

- Model: <model-id>         ← 必需
- Description: <free-text>  ← 可选，缺省为空串

## Scope                    ← H2 字面量 "## Scope" 开启 scope 段

- Read: `<glob>`, `<glob>`  ← 反引号包裹、逗号分隔；空用 —（em dash）
- Write: `<glob>`, ...
- Owns: `<glob>`, ...
```

示例（`test-v04/alice.profile.md`）：

```markdown
# alice

- Model: gpt-4o
- Description: Parser module implementer

## Scope

- Read: `src/**`
- Write: `src/parser/**`
- Owns: `src/parser/**`
```

**解析规则**（`parse_profile`）：
1. 先 `normalize_line_endings`（CRLF/CR → LF，format/mod.rs 第 15 行，注释标注 invariant I11，所有 parser 强制先调用）。
2. 逐行扫描：`# ` 开头（非 `## `）→ name；`## Scope` 字面量进入 scope 段，任何其他 `## ` 退出 scope 段；`- Key: value` 行由共享正则 `^- ([^:]+):\s*(.*)$`（format/mod.rs `BULLET_KEY_RE`）提取。
3. `Read/Write/Owns` 仅在 scope 段内生效；值经 `parse_scope_globs` 解析：`—` 或空 → 空列表；否则按逗号切分并剥去首尾反引号（也宽容接受无反引号值）。
4. **解析是宽容的**：未知 key、多余内容一律忽略；只有 name 与 Model 缺失才报错。

**校验逻辑**：
- 缺 H1 → `Parse { "missing agent name heading (# <name>)", fix, example: "# alice" }`。
- 缺 Model → `Parse { "missing - Model: line for profile '<name>'" }`。
- 坏例对照：`test-v04/garbage.profile.md`（内容 "random text no structure"）无 H1 无 Model，`validate` 命令会报 `error format: ... missing agent name heading`（v0.4 review §2.5 实测确认）。
- 单测还覆盖 CRLF、Unicode（emoji/非 ASCII 名字均可解析）（format/profile.rs 测试段）。

**Ops**：`create_profile`（拒绝覆盖，AlreadyExists；自动建父目录）、`show_profile`、`edit_profile`（只更新 `Some` 字段，读-改-全量重写）。CLI 另有 `profile list <dir>`：只扫描 `.profile.md` 后缀文件，逐个解析，坏文件优雅降级为 `(unreadable)`。

### 2.2 Post/Thread 文件（`*.post.md`）——格式核心

**权威实现**：`repos/paperwork-core/src/format/thread.rs` + `format/mod.rs` 的边界检测；操作：`repos/paperwork-core/src/ops/thread.rs`；CLI：`repos/paperwork-cli/src/cmd/post.rs`。

**文件整体结构**：线程文件**没有任何文件头 frontmatter**——文件即消息序列，从第一条消息的 `---` 边界开始。线程标题/参与者元数据不是结构化字段，而是 `post create` 注入的 system 消息正文（见下）。

**单条消息结构**（`serialize_message`，format/thread.rs 第 203–236 行）：

```markdown
---                                    ← 边界：水平线
                                       ← 空行
### #<seq> <sender> · <timestamp>      ← H3 头：seq、sender、ISO-8601 时间，以「·」(U+00B7) 分隔

- To: all | alice, bob                 ← 元数据 bullet（广播序列化为 "all"）
- Reply-To: #<seq>                     ← 可选
- Mentions: name1, name2               ← 可选

````markdown                           ← 4 反引号 fence 包裹正文
<任意 Markdown 正文>
````
```

真实样例见 `test-v04/standup.post.md`（6 条消息，含 Reply-To/#2、Mentions/alice 的 #3）。

**边界检测算法**（format/mod.rs `find_message_boundaries`，第 67–103 行）：
- 边界 = 一行恰为 `---`（trim 后），且其后 **2 行以内**出现合法 H3 头（正则 `^### #(\d+) (.+) · (.+)$`，`MESSAGE_HEADER_RE`）。不满足此组合的孤立 `---` 视为正文内容（单测 `test_find_message_boundaries_lone_hr` 验证）。
- **fence 感知**：以 ```` ```` ````（4 反引号）开头的一行切换 fence 状态；fence 内部的 `---` 永远不是边界（单测 `test_parse_message_body_with_hr` 明确守护此语义）。
- 第一条边界之前的内容被忽略；空文件/纯空白 → 空消息列表。

**消息内容解析**（`parse_message_content`）：
- fence 外、正文开始前，bullet 元数据只识别三个 key：`To`（`all`/空 → 空 Vec 表示广播；否则逗号分隔名单）、`Reply-To`（`#5` → `Some(5)`；`—`/空 → None）、`Mentions`（逗号分隔名单）。
- fence 内所有行原样收集为 body，首尾空行被裁剪，`join("\n")`。

**正文自由度**：body 是任意 Markdown，可含标题、列表、`**bold**`、三反引号代码块、甚至 `---` 与伪造的 `###` 头——全部被 4 反引号 fence 隔离（format/thread.rs 测试 `test_body_with_triple_backtick_fence`、`test_parse_message_body_with_h3`）。这正是 v0.2 feedback 第 3.3 条的直接落地（`docs/ssot/adr/feedbacks/v0_feedbacks.md`："在我们的managed文件中, 以fenced code block形式包裹, 并设置为markdown block"）。

**时间戳**：先 RFC3339，失败则按 `%Y-%m-%dT%H:%M:%S` 假定 UTC（`parse_timestamp`）。

**一致性校验**：`validate_seq_monotonicity`（首条必须 seq 1、严格连续无 gap）已实现并有单测，**但当前未被任何 ops/CLI 调用**（全库 grep 仅见定义与测试引用）——seq 完整性实际由 append 机制保证而非事后校验。同理 `validate_markdown`（未闭合 fence 检测）也仅被单测引用。

**Append 逻辑**（ops/thread.rs `thread_send`，第 41–126 行）：
1. 自动创建父目录；以 `append + create + read` 模式打开文件（首条消息自动建文件，无前置 create）。
2. `fs2::lock_exclusive()` 独占文件锁，阻塞并发写者。
3. **锁内**读最后 seq：`read_last_seq_locked` 从文件尾部反向扫描最多 64KB+256 字节缓冲，用正则 `### #(\d+) ` 取最大 seq（O(1) 不随文件增长）。
4. new_seq = last_seq + 1；时间戳取 `Utc::now()`。
5. 序列化后检查 **64KB 硬上限**（`MAX_MESSAGE_SIZE`），超限报 `MessageTooLarge`。
6. 单次 `write_all` 原子追加，随后解锁。

**唯一的非 append 操作——`thread_edit`**（第 222–380 行）：自编辑受三重约束——① 只能改自己发的消息（sender 匹配）；② 必须是该 sender 最近一条；③ 必须是全线程最后一条。通过后锁内读全文 → 解析 → 替换 body（保留全部元数据）→ `set_len(0)` 全量重写。这是 append-only 语义的唯一受控例外。

**其余 ops**：`thread_read`（seq 闭区间过滤）、`thread_summary`（计数、末条 sender/时间、末 3 条各取前 50 字符 snippet）。

**CLI 行为要点**（cmd/post.rs）：
- `post create` 以 sender=`system` 发送 `[Thread created: <title> | participants: ...]` 作为 seq #1（评审长期指出这是设计债，见 §5）；`post summary` 反向从该 system 消息正文中提取 title/participants。
- `post send`：拒绝空/纯空白正文（`Validation` 错误）；`--stdin` 从管道读正文；**reply 隐式 @**：`--reply-to N` 时自动把被回复消息的 sender 加入 mentions（非自己且未重复时）。
- `--to` 参数不存在：CLI 永远传空 to 列表，文件中恒为 `- To: all`（To 字段是格式层能力，CLI 未暴露）。
- `post read`：支持 `--from/--to`（seq 区间）、`--mention`、`--reply-to` 过滤与 `--limit`（默认 20，取尾部 N 条，超出时输出 `showing: N/M`）；`--plain` 输出过滤后消息重新序列化的原始格式。

### 2.3 Brief 文件（`*.brief.md`）＝ Read Manifest

**权威实现**：`repos/paperwork-core/src/format/manifest.rs`（内部类型仍叫 Manifest，命令面叫 brief，v0.2 更名）；操作：`ops/manifest.rs`；CLI：`cmd/brief.rs`。

**Schema**（format/manifest.rs 头部注释 + 解析）：

```markdown
# <title>                      ← H1（兼容旧 "# Manifest: X" 前缀，解析时剥除）

- Owner: <agent>               ← 必需（兼容旧 key "Author"）
- Created: <ISO-8601>          ← 必需，解析失败即整体 Parse 错误
- Description: <text>          ← 可选

## Entries                     ← 字面量段头

### <entry-title>              ← 每个条目一个 H3（条目间可选 --- 分隔）

- Path: `<relative-path-or-glob>`   ← 反引号包裹，解析时剥除
- Hash: `<sha256-hex>`              ← 反引号包裹
- Regex: `<pattern>` | —            ← 内联反引号 或 "—" 表示无
> note 行（blockquote，可多行，拼接为 note）
```

复杂正则改用围栏块：`- Regex:` 后跟 ` ```regex ... ``` `（含换行或反引号的 pattern 序列化时自动切换此形式）。

真实样例见 `test-v04/onboarding.brief.md` 与 `_fix/guide.brief.md`（后者展示空 Description 也合法）。

**解析细节**：
- `groups` 字段不落盘：解析时由 regex 命名捕获组 `(?<name>...)` 自动提取（`extract_regex_groups`）。
- 条目 title 缺 path/hash 时落空串（`EntryBuilder::build`），不报错；整体文件仅 title/Owner/Created 缺失才报 Parse。
- `brief add`：title 取条目路径的**文件名**；重复 title 报 AlreadyExists；hash 由 `hash::hash_file` 对目标文件计算（路径先按 CWD 解析，不存在则相对 brief 所在目录）。
- `brief remove`：按 title 删除，未找到报 NotFound；`brief read --full` 输出 hash 前 12 位、regex、note。

**Hash 版本化过期检测**（`brief_verify` / `verify_entry`，ops/manifest.rs 第 216–257 行）——三态判定，regex 优先：

| 条件 | 结果 |
|------|------|
| 文件读取失败（缺失） | `Stale` |
| 有 regex 且编译失败或不匹配 | `Stale` |
| regex 匹配（或无 regex）且 SHA-256 与记录一致 | `Fresh` |
| regex 匹配（或无 regex）但 hash 不同 | `Shifted` |

语义（lib.rs `VerifyResult` 注释）：Fresh = 内容未变；Shifted = 结构锚点（regex）仍在但内容有改动；Stale = 锚点失效，brief 知识已过期，需重读。hash 是对文件**原始字节**的 SHA-256（hash.rs `hash_file` = `fs::read` + `hash_bytes`），不做任何归一化，因此换行符变化也会判 Shifted。CLI `brief verify [--base-dir]` 默认以 brief 所在目录解析条目相对路径，输出 `<title>: fresh|shifted|stale` 与 `N/M fresh` 结论。

### 2.4 Contacts 文件（`*.contacts.md`）

**权威实现**：`format/contacts.rs`、`ops/contacts.rs`、`cmd/contacts.rs`。设计定位见 v0_feedbacks："contact就是一个特殊的brief"。

**Schema**（极简 bullet 列表）：

```markdown
# <title>          ← H1 必需（parse_contacts_title 单独提取）

- <profile-path>   ← 每行一个 profile 路径，无 key、无 summary 列
```

真实样例 `test-v04/team.contacts.md`：存的是 Windows 绝对路径。

**解析**：跳过空行与 `#` 标题行；`- ` 前缀行整体作为 `profile_path`，`summary` 字段恒为空串（`ContactEntry.summary` 在格式层已废弃，仅存于类型定义）。v0.2 时期的 Markdown 表格格式（见 v0.2 review §6 样例）已被 bullet 列表取代。

**Ops**：`contacts_create`（拒绝覆盖）、`contacts_add`（**幂等**：路径已存在则静默 Ok）、`contacts_read`。CLI `contacts read` 的 summary 信息不在文件中，而是**读取时即时增强**：逐条调用 `show_profile` 解析被引用的 profile，输出 `path: name (description)`，解析失败显示 `(unreadable)`（修复了 v0.2/v0.3 的 BUG-4/5）。

### 2.5 Validate 命令（跨格式结构校验）

**实现**：`repos/paperwork-cli/src/cmd/validate.rs`。
1. 按路径后缀判定类型：`.post.md` / `.profile.md` / `.brief.md` / `.contacts.md`；未知后缀直接 Parse 错误。
2. 调用对应 format 解析器做**真实解析**（v0.4 修复了 v0.3 "no-op validator" 关键 bug）。
3. Post 特有规则：解析成功但 0 条消息且内容非空 → 报 "no valid message boundaries found"（纯垃圾文本无边界、非空内容 thus 被拒）。
4. 失败统一包装为 `Parse { "<path> is not a valid <type> file: <detail>", fix, example }`。
5. 注意：`validate` **不做** seq 连续性校验（`validate_seq_monotonicity` 未接入），也不做 fence 闭合校验（`validate_markdown` 未接入）——当前校验深度止于"能否被解析器接受"。

---

## 3. 跨格式共同设计原则

1. **Markdown + bullet-key 而非 YAML/JSON**：源于初版技术选型（`docs/ssot/adr/初版技术选型.md`）——"managed文件格式推荐使用带丰富标记的markdown, 来保证through cli/file两种情况都可轻松阅读"。结构化元数据统一用 `- Key: value` bullet（v0.3 从 bold-key `**Key**: value` 迁移而来），共享一个正则解析器；自由内容用 fence 隔离。人与 agent 双可读是第一目标。
2. **无 frontmatter，标题即元数据**：四种格式都以 H1 开头承载标识（profile 名 / brief 标题 / contacts 标题），thread 干脆以消息边界开头。避免了 YAML 解析依赖，`---` 只承担 Markdown 水平线这一种语义。
3. **Append-only**：thread 追加用"独占锁 + 锁内取 seq + 单次 write"，天然支持并发安全（v0.2 引入 fs2，修复了 v0.2 review BUG-3 的锁外取 seq 问题）；append-only 保证历史不可篡改、任何 agent 可增量消费。唯一编辑口 `thread_edit` 以三重约束模拟"撤回重发"而不破坏历史语义。
4. **Fence 边界隔离**：v0.2 feedback 指出的核心矛盾（"使用者输入的内容也需要是markdown"）以 4 反引号 fenced block 解决——外层用 4 反引号，内层 3 反引号代码块、`---`、`###`、`***` 全部安全（v0.3 review §4.1）。`***` 作为正文分割线的旧约定因此降级为可选习惯，代码不再依赖。
5. **类型后缀即类型系统**：文件名后缀（`.profile.md` 等）是唯一的类型判别依据——validate 按后缀分发解析器、`profile list` 按后缀过滤、CLI 自动补后缀。文件"by format, not by location"（ADR-v1 原则表的 format-matching）。
6. **Hash 完整性**：brief 用 SHA-256 字节哈希 + regex 锚点实现轻量级"知识新鲜度"，三态 Fresh/Shifted/Stale 区分"没变/小改/结构性过期"，服务于 agent 的阅读清单维护，而非安全级完整性校验。
7. **错误处理风格**（`error.rs`）：单一枚举 `PaperworkError` 覆盖全部失败模式，每类都携带 `fix` + `example` 两个自纠错字段，支撑"agent 一次重试即可自修复"的目标（v0.4 CHANGELOG）。类别到错误信封 category 的映射：

   | 错误变体 | category | 对应格式违规场景 |
   |---|---|---|
   | `Parse` | `format` | 结构违规：缺 H1/Model/Owner/Created、非法消息头、非法时间戳、无消息边界 |
   | `Validation` | `validation` | 语义违规：空正文、seq gap（预留）、同时给 positional 与 --stdin |
   | `MessageTooLarge` | `validation` | 单条消息超 64KB |
   | `NotFound` / `AlreadyExists` | `not-found` / `already-exists` | 文件存在性：读不存在文件、create 拒绝覆盖 |
   | `NotAllowed` | `not-allowed` | 状态约束：编辑他人/非末条消息 |
   | `IoContext` / `Io` | `io` | 带路径上下文的 IO 失败 |

8. **宽容解析、严格校验**：解析器对未知 key、多余内容静默忽略，对 CRLF 先归一化（I11），对 Unicode 全兼容；只有结构性必需字段缺失才失败。向后兼容显式保留（manifest 的 `Author`→`Owner`、`# Manifest:` 前缀）。
9. **无状态与文件独立**：CLI 不维护任何跨文件引用（contacts 里路径读不到只显示 unreadable，不报错中断）；`ops/mod.rs` 与 ADR-v1 均明文规定。

---

## 4. 与历史约定的核实对照（结论）

| 历史约定 | 当前代码事实 | 证据 |
|---|---|---|
| 线程用 YAML `---` 作消息分隔符 | `---` 仍是消息分隔符，但是 Markdown 水平线语义 + "后 2 行内必须出现 H3 头"的锚定解析，非 YAML 解析；无 frontmatter | format/mod.rs `find_message_boundaries`；v0.3 review 第 18、148 行 |
| 正文分割线用 `***` | 代码无任何 `***` 处理；正文被 4 反引号 fence 包裹后 `---` 也安全，`***` 仅为文档中的使用习惯 | 全库 grep `***` 仅命中 docs/reviews 两处；format/thread.rs 测试 |
| append-only 语义 | 成立：锁内追加 + 单次写入；仅 `thread_edit` 可重写且限"本人最后一条即全线程最后一条" | ops/thread.rs 第 41–126、222–380 行 |
| manifest 用 blob hash 做过期检测 | 成立且更精细：SHA-256 字节哈希 + regex 锚点 → Fresh/Shifted/Stale 三态 | hash.rs、ops/manifest.rs `verify_entry` |
| 消息头分隔符 | 实际是 `·`（U+00B7）；README 示例渲染为 `.`、错误 fix 文案也写 `.`，与实际正则不一致——是一处文档/代码小偏差 | format/mod.rs 第 25 行 vs README 第 172 行 |

---

## 5. 版本演进摘要

**v0.1 → v0.2（架构纠偏，2026-07-30）**：owner feedback（`docs/ssot/adr/feedbacks/v0_feedbacks.md`）否决 `.paperwork/` 托管目录 + init/login 模型，确立无状态、路径显式、文件独立（固化为 `docs/dev/adr-v1.md` ADR-011）；删除 `init/invite/dm/notify/layout`，统一 `post` 为唯一通信原语（DM 被追加 feedback 明确删除），manifest 更名 brief，引入 fs2 文件锁与 `--json/--plain`。当时格式为 bold-key 元数据 + 裸正文（v0.2 review §6 留样）。

**v0.2 → v0.3（格式重设计，2026-08-01）**：落实 v0.2 feedback 三条——① 类型后缀自动补全；② 严谨克制的 Markdown 结构（bullet-list 元数据取代 bold-key）；③ 正文以 4 反引号 markdown fence 包裹，解决边界歧义；`---` 成为唯一消息分隔符；contacts 从表格改 bullet；新增 `validate` 命令（但初版是 no-op）。遗留 4 个 bug，其中"validate 接受垃圾"为 Critical。

**v0.3 → v0.4（输出协议与校验修复，2026-08-01）**：全部输出重构为 ASCII 信封协议（`ok/error` + category + `fix:`/`example:`，解决 Windows 下 Unicode 乱码）；validate 接入真实解析；拒绝空正文；新增 `--stdin`；`post read` 默认显示时间戳与内联元数据且 `--plain` 修复区间过滤；contacts read 增强 profile 简介；`profile list` 结构化。v0.4 review 确认 v0.3 四个 bug 全部修复，残留低优问题：system 消息占据 #1（持续设计债，UX review 建议废除 `post create` 改为首条 send 携带 `--title/--participants`）、`--from` 身份/区间语义冲突（建议 `--as`）、`PAPERWORK_AGENT` 环境变量提案等（`docs/reviews/v0.4-ux-review-2026-08-01.md` 优先级矩阵）。

**格式维度的演进主线**：bold-key 裸正文（v0.2，脆弱）→ bullet 元数据 + fenced 正文 + 后缀类型系统（v0.3，稳健）→ 格式稳定不动，改进全部发生在输出协议与校验严格性上（v0.4）。

---

## 6. 已识别的技术债与风险点（供后续任务参考）

1. `validate_seq_monotonicity` 与 `validate_markdown` 已实现但未接入任何命令路径——外部手工篡改文件产生的 seq gap 或断 fence 不会被 validate 捕获。
2. `post create` 的 system 消息占据 seq #1：title/participants 以正文文本编码（`[Thread created: X | participants: Y]`），`post summary` 靠字符串切分反解，属脆弱编码。
3. `Message.to` 字段在格式层存在但 CLI 无 `--to` 入口，恒为 `To: all`。
4. 消息头分隔符 `·`（非 ASCII）与 README/错误文案中的 `.` 不一致；输出协议整体 ASCII 化后此处是残留的非 ASCII 格式元素。
5. brief 的 hash 对字节敏感（含换行符），跨平台 git checkout 换行转换可能误报 Shifted（与 I11 的解析归一化是两套路径，hash 先于归一化）。
6. `docs/dev/adr-v1.md` 中的 DM 目录/notify 约定已被后续 feedback 废止（DM 删除、notify 移除），引用时须以 CHANGELOG v0.2 与 v0_feedbacks 追加段为准。

---

**关键源文件索引**（绝对路径）：

- 格式层：`c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\repos\paperwork-core\src\format\{mod.rs, profile.rs, thread.rs, manifest.rs, contacts.rs}`
- 操作层：`c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\repos\paperwork-core\src\ops\{thread.rs, profile.rs, manifest.rs, contacts.rs}`
- 错误/哈希/类型：`...\paperwork-core\src\{error.rs, hash.rs, lib.rs}`
- CLI：`...\paperwork-cli\src\{main.rs, output.rs, cmd\*.rs}`
- 测试：`...\paperwork-core\tests\ops_tests.rs`、`...\paperwork-cli\tests\cli_integration.rs`
- 语料：`test-v03\`、`test-v04\`、`_fix\`
- 文档：`CHANGELOG.md`、`docs\reviews\v0.{2,3,4}-review-*.md`、`docs\reviews\v0.4-ux-review-2026-08-01.md`、`docs\dev\adr-v1.md`、`docs\ssot\adr\{初版技术选型.md, agent-ux-qol.md, feedbacks\v0_feedbacks.md}`
