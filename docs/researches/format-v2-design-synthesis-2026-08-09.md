# Managed File Format v2 设计整合文档（v0.5 "Format Renewal"）

> **文档性质**：三视角格式重设计方案的整合落盘（调研设计文档），是后续 `docs/dev/format-v2/` 五文档（spec/design/bdd/tdd/impl_plan）与 review 的唯一设计输入。只读本文即可理解全部设计决策。
>
> **来源索引**：
> - 权威规格来源：计划文件《Managed File Format 全面重设计（v0.5 "Format Renewal"）》
> - 视角 A（简洁与可维护性）规划报告（任务 #3）
> - 视角 B（解析健壮性与扩展性）规划报告（任务 #4）
> - 视角 C（最小变更与风险控制）规划报告（任务 #5）
> - 需求基线：`docs/researches/research-repo-formats-2026-08-08.md`（§6 技术债清单）
> - 用户最高指令：`docs/ssot/adr/feedbacks/v0_feedbacks.md`（第 23、27 行）

---

## 1. 背景与用户裁决

### 1.1 问题背景

当前四种 managed 文件格式（profile / post / brief / contacts）在 v0.2–v0.4 演进中沉淀了多类 ad-hoc 构造：`---` 水平线 + "后 2 行内出现 H3 头"的复合前瞻边界、非 ASCII 消息头分隔符 `·`（U+00B7，且与 README/错误文案中的 `.` 不一致）、固定 4 反引号 fence、`—`（em dash）空值占位、`all` 魔法值，以及以 system 消息正文文本脆弱编码 title/participants。这些构造缺乏规范依据，与用户"正规简洁、严谨克制"的格式立场冲突。`docs/researches/research-repo-formats-2026-08-08.md` §6 的六项技术债清单（详见附录）构成本次重设计的问题基线。

### 1.2 用户对旧格式的批评要点

- 信息结构组织不够"正规"：存在多处自造 pattern（复合锚定边界、非 ASCII 结构符、魔法值占位），缺乏 Markdown 原生语义依据（来源：`v0_feedbacks.md` 第 23 行批评的直接依据）。
- system 消息以 `[Thread created: X | participants: Y]` 正文文本编码元数据，`post summary` 靠字符串切分反解，属脆弱编码（来源：`research-repo-formats-2026-08-08.md` §6.2）。
- `validate` 校验深度止于"能否被解析器接受"，seq 连续性与 fence 闭合校验已实现却未接入（来源：同上 §6.1）。
- 非 ASCII 格式元素 `·` 残留，与 v0.4 输出协议 ASCII 化方向不一致（来源：同上 §6.4）。

### 1.3 两项关键裁决（本轮已确认）

1. **元数据载体**：纯原生 Markdown（标题/段落/列表/表格/链接/围栏），**禁止** YAML frontmatter、禁止新增 YAML 依赖。
2. **兼容策略**：项目默认不向前兼容，**hard breaking v0.5**，CHANGELOG 附迁移指南即可，不做 migrate 命令。

### 1.4 用户最高指令原文引用

`docs/ssot/adr/feedbacks/v0_feedbacks.md` **第 23 行**（v0.2 feedback 第 2 条）：

> 既然我们选择使用md作为文件格式, 那么就以正规简洁的方式组织信息结构, 严谨克制, 但是自由灵活地使用各个标题, 列表等等语法.

同文件 **第 27 行**（第 3.3 条，**保留**）：

> 在我们的managed文件中, 以fenced code block形式包裹, 并设置为markdown block, 这样就能够让文件支持多层markdown了.

即：fence 包裹正文是用户本人的硬性要求，不可废除，只能让其更标准（定稿采用 CommonMark 规范内的动态围栏长度取代固定 4 反引号）。

---

## 2. 定稿格式规格（decision-complete）

> 本章全文收录自计划文件"新格式规格"章节，schema 代码围栏逐字保留，不得删改语义。

### 2.1 统一设计语言（四种格式共用）

1. **H1 = 文档身份**（名字/标题），H1 后首个 H2 之前的段落 = description（自由散文）。
2. **扁平标量属性** = 首个 H2 之前的列表项 `- key: value`（小写 ASCII 键；空/缺省即省略该行，废除 `—` 占位与 `all` 魔法值）。
3. **表格型数据** = GFM 表格（原生制表语义）。
4. **对其他 managed 文件的引用** = Markdown 链接（原生引用语义）。
5. **用户内容** = ` ```markdown ` 围栏块，**围栏长度动态** = max(3, 正文内最长连续反引号串 + 1)（CommonMark 规范内，取代固定 4 反引号）。
6. **记录单元**（消息/brief 条目）= H2 标题；解析全程 fence 感知（遵循 CommonMark：N 个反引号开的围栏只能被 ≥N 个反引号关闭）。
7. 结构符全 ASCII，废除 `·`（U+00B7）与 `—`；宽容解析（未知内容忽略）、CRLF 归一化（I11）保留。

### 2.2 profile（`*.profile.md`）

````markdown
# alice

Parser module implementer

- model: gpt-4o

## Scope

| permission | paths |
| --- | --- |
| read | src/** |
| write | src/parser/** |
| owns | src/parser/** |
````

规则：

- 必需：H1（name）、`- model:`；description 与 Scope 可选（空 scope = 省略整节）。
- Scope 表格一行一个 (permission, glob) 对；permission ∈ read/write/owns。

### 2.3 post/thread（`*.post.md`）

````markdown
# Daily Standup

## Participants

- alice
- bob

## #1 alice (2026-08-01T19:38:22Z)

Parser module is 80% done.（正文恒在 ```markdown 围栏内，此处示意）

## #2 bob (2026-08-01T19:38:22Z)

- reply-to: #1
- mentions: alice
- to: charlie

```markdown
Tests merged, all green.
```
````

规则：

- **preamble** = 首个 fence 感知 `^## #(\d+) (\S+) \((.+)\)$` H2 之前的全部内容：H1 = title；`## Participants` 下的 bullets = 参与者；其余忽略。
- 消息 = H2（seq + sender + RFC3339 时间戳，纯 ASCII 括号）；sender 校验为无空格/无括号 token。
- 可选属性列表：`reply-to:`、`mentions:`、`to:`（广播 = 省略 `to`，废除 `all`）；正文 = 动态长度 markdown 围栏。
- **彻底废除 system 消息**（清偿技术债 #2）：`post create` 命令删除；`post send` 新建文件时可带 `--title`（缺省 = 文件名主干）与 `--participants`，锁内首写 preamble。
- 尾部 O(1) seq 扫描正则改为 `^## #(\d+) `；64KB 单条上限、fs2 锁、append-only、`thread_edit` 三重约束全部保留。

### 2.4 brief（`*.brief.md`）

````markdown
# Codebase Onboarding

How to understand this project

- owner: alice
- created: 2026-08-01T19:40:36Z

## main.rs

- path: src/main.rs
- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540
- regex: fn main

Entry point
````

规则：

- 条目 = H2（title 即标题文本）+ 属性列表（path/hash/regex，hash 全量落盘不截断）+ 散文 note 段（取代 blockquote note 与 ```` ```regex ```` 之外的结构）。
- 复杂 regex（含换行/反引号/特殊字符）用 ` ```regex ` 围栏块（CommonMark 合法的 info-string 围栏，保留此逃生口）；`groups` 仍由命名捕获组派生不落盘。
- Fresh/Shifted/Stale 三态验证逻辑与字节级 SHA-256 不变（技术债 #5 仅在 spec 中文档化声明，不改行为）。

### 2.5 contacts（`*.contacts.md`）

````markdown
# Core Team

- [alice](agents/alice.profile.md)
- [bob](agents/bob.profile.md)
````

规则：

- 条目 = Markdown 链接：label 为 profile 名，destination 为路径；路径含空格/括号时序列化为 `[](<path>)` 形式，解析两种形式。
- 读取时即时增强 profile 简介的行为不变。

### 2.6 validate 强化（清偿技术债 #1、#3）

- `validate` 接入 `validate_seq_monotonicity`（seq 从 1 严格连续）与围栏闭合校验；`post send` 新增 `--to` 参数。

---

## 3. 三视角方案摘要与取舍记录

### 3.1 视角 A：简洁性与可维护性（Simplicity & Maintainability）

**方向概述**：把"文件级元数据"迁移到业界公认的 YAML frontmatter，把"记录级结构"统一到标准标题层级 + 标准列表；正文继续 fenced 包裹但改为 CommonMark 标准的自适应 fence 长度。核心取舍：引入一个 `serde_yaml` 依赖换取"零学习成本"的文件头元数据（同时根治 system 消息脆弱编码）；逐消息/逐条目的可选字段仍用 heading 下的 bullet（键名改为小写 ASCII、值不再强制反引号包裹）。

**schema 主张要点**（节选自视角 A 报告）：

````markdown
---
title: Daily Standup
participants: [alice, bob, charlie]
---

## #1 alice 2026-08-08T10:00:00Z

Parser module is 80% done.

## #2 bob 2026-08-08T10:05:00Z

- reply-to: #1
- mentions: alice

```markdown
正文可含 ```、---、###，fence 自动加长隔离
```
````

（profile 用 frontmatter 承载 model + `## Scope` 下三个 H3 列表；brief 用 frontmatter 承载 owner/created/description + blockquote note；contacts 仅 frontmatter title + 纯 bullet 路径。）

**核心取舍与风险**：以"新增 YAML 依赖、推翻『无 frontmatter』历史原则"换取元数据零学习成本与 system 消息债务清偿；风险在于与历史评审原则冲突、heading 即边界时 fence 内伪造 `## #N` 需 fence 感知防护。

**被采纳进定稿的元素**：

- H2 标题即消息边界（heading 本身即边界，废除 `---` + 前瞻算法）。
- 动态/自适应 fence 长度思想（正文含 fence 时自动加长）。
- 小写 ASCII 键的 bullet 元数据（废除大写 Key、反引号剥除、`—` 占位、`all` 魔法值）。
- frontmatter/preamble 承载 title/participants，废除 system 消息；`post summary` 直读元数据。
- fence 感知扫描作为 heading 切分的前提。

**被否决的元素及理由**：

- YAML frontmatter + `serde_yaml` 依赖：与用户裁决（纯原生 Markdown、禁 YAML）直接冲突。
- contacts 纯裸路径 bullet：定稿改用 Markdown 链接，获得原生引用语义与 label 简介锚点。
- brief 的 blockquote note：定稿改用散文 note 段（blockquote 原生语义是引用/旁注，不贴切）。

### 3.2 视角 B：解析健壮性、生态兼容与扩展性

**方向概述**：以"YAML frontmatter 标准 + CommonMark 标准构造 + 围栏 YAML 元数据块"替换全部自造 pattern，使每一处解析都有成熟规范（YAML 1.2 / CommonMark / GFM）背书，并用 serde 类型化解析天然获得未知字段前向兼容。核心取舍：引入 `serde_yaml` 依赖与文件头 frontmatter，换取解析无歧义、frontmatter 生态（VS Code/Obsidian/Hugo/gray-matter）直接兼容；正文 fence 长度改为"正文最长反引号串 + 1"的动态长度。并发性与 O(1) 尾读机制完全保留。

**schema 主张要点**（节选自视角 B 报告）：

`````markdown
---
type: post
version: 1
title: Daily Standup
participants: [alice, bob, charlie]
---

## #1 alice

```yaml
time: 2026-08-01T19:38:22Z
```

````markdown
Parser module is 80% done.
````

## #3 bob

```yaml
time: 2026-08-01T19:38:22Z
reply-to: 2
mentions: [alice]
```

````markdown
Tests merged, all green.
````
`````

（frontmatter 含 `type`/`version` 字段为格式演进预留；每条消息 = H2 边界头 + 可选 ```` ```yaml ```` 元数据围栏块 + 动态长度 markdown 正文围栏；brief 条目用 YAML 块标量承载复杂 regex，替代自造 ```` ```regex ````。）

**核心取舍与风险**：以双重 YAML 面（frontmatter + 逐消息元数据块）换取类型化扩展与生态兼容；风险包括 `serde_yaml` crate 处于维护模式、逐消息 yaml fence 视觉噪声大、推翻"无 frontmatter"原则与历史评审冲突。

**被采纳进定稿的元素**：

- 动态 fence 长度的精确算法：max(正文内最长连续反引号串 + 1, 下限)，定稿取 max(3, 最长串 + 1)。
- 尾部 O(1) seq 扫描正则改为 `^## #(\d+) `（grep 友好、O(1) 机制不变）。
- 首条 send 锁内写入文件头元数据（frontmatter 首写竞争由既有 fs2 锁覆盖）。
- sender 字符集校验（防注入破坏头文法）。
- `validate` 接入 `validate_seq_monotonicity` 与围栏闭合校验。
- brief hash 保持字节级 SHA-256 不变（技术债 #5 仅文档化）。

**被否决的元素及理由**：

- YAML frontmatter 与 `type`/`version` 字段：用户裁决禁 YAML；hard breaking 策略下格式版本字段无必要（以 spec 与 CHANGELOG 为准）。
- 逐消息 ```` ```yaml ```` 元数据围栏块：重新引入 YAML 且视觉噪声大，与裁决冲突；定稿用 H2 标题内联 seq/sender/时间戳 + 可选属性列表。
- 用 YAML 块标量承载复杂 regex 并删除 ```` ```regex ````：定稿保留 ```` ```regex ```` 逃生口（CommonMark 合法的 info-string 围栏）。

### 3.3 视角 C：最小变更与风险控制（Expand–Migrate–Contract）

**方向概述**：双版本共存。旧解析器整体冻结为 `format::v1`（只读、不改一行），新格式实现为独立 `format::v2`；读路径按确定性结构签名双版本分发，写路径只产 v2；存量文件用一次性、无损、roundtrip 可证的 `migrate` 操作转换。新格式本身遵循"每个语法元素只用 Markdown 原生语义、结构符全 ASCII"。取舍：放弃"彻底删除旧代码"的整洁性，换取每一步可独立验证、可回滚。

**schema 主张要点**（节选自视角 C 报告）：

````markdown
# Daily Standup

> participants: alice, bob, charlie
> created: 2026-08-01T19:38:03Z

## #2 alice (2026-08-01T19:38:22Z)

````markdown
Parser module is 80% done.
````

## #3 bob (2026-08-01T19:38:22Z)

> reply-to: #2
> mentions: alice

````markdown
Tests merged, all green.
````
````

（文件头 H1 + blockquote 承载 title/participants；消息 = 单个 `^## #(\d+) (.+) \((.+)\)$` 标题，全 ASCII；元数据用 blockquote `> key: value`；contacts 用 Markdown 链接 `[alice](agents/alice.profile.md)`；不加 frontmatter，采用确定性结构探测做版本分发。）

**核心取舍与风险**：以双版本解析代码翻倍、迁移语义变换风险（system #1 → 文件头是唯一真变换）换取零回归风险与可回滚性；探测歧义与野外存量文件写坏是主要风险面。

**被采纳进定稿的元素**：

- H2 标题承载消息 = seq + sender + 括号内时间戳（`^## #(\d+) (\S+) \((.+)\)$` 的形态来源）。
- Markdown 链接承载 contacts 引用（label = profile 名，destination = 路径）。
- 结构符全 ASCII、每个语法元素只用 Markdown 原生语义的设计立场。
- brief 条目结构：标题 + 字段 + 散文 note 的骨架（属性载体由 blockquote 改为 bullet 列表）。

**被否决的元素及理由**：

- 双版本共存 + `migrate` 命令：用户裁决 hard breaking、迁移指南即可；避免解析代码翻倍。
- blockquote `> key: value` 承载元数据：blockquote 原生语义是引用/旁注，属性列表 `- key: value` 语义更贴切。
- 确定性结构探测的版本分发机制：hard breaking 下无必要。

### 3.4 关键分叉点对照表

| 分叉点 | 视角 A（简洁） | 视角 B（健壮/扩展） | 视角 C（最小变更） | 最终裁决 |
|---|---|---|---|---|
| **元数据载体** | YAML frontmatter + bullet | YAML frontmatter + 逐消息 ```` ```yaml ```` 块 | 无 frontmatter；blockquote `> key: value` + 结构探测 | 纯原生 Markdown：preamble（H1/description/bullet 列表）+ GFM 表格 + Markdown 链接；禁 YAML |
| **消息边界** | H2 严格文法头（无括号时间戳） | H2 `^## #\d+ <sender>$`（时间戳进 yaml 块） | H2 `^## #(\d+) (.+) \((.+)\)$` | H2 = `^## #(\d+) (\S+) \((.+)\)$`，fence 感知；seq/sender/时间戳内联于标题 |
| **消息元数据** | heading 下 bullet（小写键） | ```` ```yaml ```` 围栏块（serde 类型化） | blockquote | H2 下可选属性列表 `- reply-to:/- mentions:/- to:`；广播 = 省略 `to` |
| **兼容策略** | hard breaking，可选 migrate | hard breaking，validate 提示迁移 | Expand–Migrate–Contract 双版本 + 无损 migrate | hard breaking v0.5，CHANGELOG 附迁移指南，不做 migrate |

---

## 4. Rejected Alternatives

（收录自计划文件，共 6 条）

1. **YAML frontmatter（方案 A/B 主张）**：生态兼容性最佳，但用户明确裁决采用纯原生 Markdown，且避免新增 YAML 依赖。
2. **双版本共存 + `migrate` 命令（方案 C 主张）**：安全性最佳，但用户裁决项目默认不向前兼容，迁移指南即可；避免解析代码翻倍。
3. **逐消息 ` ```yaml ` 元数据围栏块（方案 B）**：类型化扩展好，但重新引入 YAML 且视觉噪声大，与裁决冲突。
4. **blockquote `> key: value` 承载元数据（方案 C）**：blockquote 原生语义是引用/旁注，属性列表语义更贴切；blockquote 保留给历史 note 语义的替代（散文段）。
5. **保留 `---` 边界 + 固定 4 反引号**：即被用户否定的 ad-hoc 组合（前瞻锚定、非规范固定围栏长度）。
6. **格式版本字段/frontmatter 类型标记**：hard breaking 策略下无必要，格式版本以 spec 文档与 CHANGELOG 为准。

---

## 5. 风险与缓解

（收录自计划文件）

| 风险 | 缓解 |
|---|---|
| preamble 与消息同为 H2 可能歧义 | 边界正则 `^## #\d+ ` fence 感知匹配，非匹配 H2 一律归 preamble；对抗用例覆盖 |
| 动态围栏长度边界（正文含超长反引号串） | TDD 强制覆盖 3–6 连续反引号各一例；算法 = 最长串 + 1、下限 3 |
| sender/timestamp 注入破坏 H2 文法 | sender 字符集校验（无空格/括号/换行），错误信封带 fix/example |
| Windows 路径含空格破坏链接 | 序列化自动切 `[](<path>)` 形式；解析双形式 |
| hard breaking 伤及存量文件 | 用户已裁决；CHANGELOG 迁移指南含逐格式 before/after 对照 |
| 表格手写不便 | 文件主要由 CLI 序列化产出，手写场景仅要求行级增删，spec 中明示 |
| 违反实现流程原则抢跑 | 阶段 0/1 为硬门槛，文档未闭合不得进入阶段 2 |

---

## 6. 实施阶段概要

（收录自计划文件的阶段划分与依赖关系）

### 6.1 阶段 0：调研沉淀与五文档（实现门槛前置）

- 0.1 将三视角方案整合结论落盘为 `docs/researches/format-v2-design-synthesis-2026-08-09.md`（即本文档；含定稿规格全文、三方案取舍记录、Rejected Alternatives）。
- 0.2 依《实现流程原则》产出：`docs/dev/format-v2/{spec.md, design.md, bdd.md, tdd.md, impl_plan.md}` + `docs/roles/format-v2-implementer.md`（职责/原则/BOOTSTRAP）。内容以定稿规格为准，BDD 覆盖四格式全部 Given/When/Then 场景，TDD 列出测试清单（含边界用例）。

### 6.2 阶段 1：对抗性 review-rework loop（实现门槛）

- 1.1 派出多个 distinct 视角的批判性 review subagents（格式正统性/解析健壮性/流程符合性至少三视角）对六份文档做对抗 review。
- 1.2 依据 review 结论 rework 文档，循环直至全部闭合。用户反馈（v0_feedbacks）与 pillars 为最终裁决依据。

### 6.3 阶段 2：实现（文档闭合后）

- 2.1 **core format 层重写**（`repos/paperwork-core/src/format/`）：`mod.rs` 删除 `MESSAGE_HEADER_RE`/`find_message_boundaries`/`BULLET_KEY_RE` 中的 `—`/反引号剥除逻辑，新增共享 preamble 解析、动态围栏工具、表格解析；`thread.rs`/`profile.rs`/`manifest.rs`/`contacts.rs` 按新规格重写解析与序列化；`error.rs` 全部 fix/example 文案更新为新格式（纯 ASCII）。领域类型（`lib.rs`）仅增 `ThreadMeta{title, participants}`。
- 2.2 **core ops 层适配**（`repos/paperwork-core/src/ops/`）：`thread.rs` 的 `SEQ_RE` 改 `^## #(\d+) `、`thread_send` 锁内首写 preamble、`thread_edit`/`thread_summary` 适配；`profile.rs`/`manifest.rs`/`contacts.rs` 序列化适配。`output.rs` 信封协议零改动。
- 2.3 **CLI 适配**（`repos/paperwork-cli/src/`）：删除 `post create`；`post send` 增 `--title/--participants/--to`；`post summary` 读 preamble；`validate` 接入 seq/围栏校验；其余命令仅改输出组装。
- 2.4 **测试重写**：format 内联单测、`tests/ops_tests.rs`、`tests/cli_integration.rs` 全部改新格式字面量；新增边界用例：动态围栏（正文含 3/4/5/6 连续反引号）、fence 内伪造 `## #N`、sender 字符集违规、Windows 带空格路径链接、CRLF、Unicode 正文、preamble 变体、并发锁内首写竞争（沿用现有并发用例）。
- 2.5 **语料与文档**：新建 `test-v05/` 冒烟语料（含 garbage/broken 坏例）；旧 `test-v03/`、`test-v04/`、`_fix/` 保持原样作历史记录；重写 `README.md` 格式章节（一并消灭 `·`/`.` 文档偏差）；`CHANGELOG.md` 增 0.5.0 Breaking 段 + 迁移指南（新旧格式对照与手工迁移步骤）；核对 `.github/workflows/ci.yml` smoke 脚本断言。
- 2.6 版本：workspace 两个 crate bump 至 0.5.0；发布顺序仍 core → 30s → cli（`publish.ps1` 不变）。

### 6.4 阶段 3：验证与评审

- Verify：`cargo test --workspace`、`cargo clippy -- -D warnings` 三平台语义本地等价检查；Browser 不适用（无 Web UI）。
- Ultra Review：3 个独立 CodeReview subagents 分别按完整性/正确性/影响面评审，合并结论后修复回归。
- 在 `docs/reviews/` 产出 `v0.5-review-<date>.md` 评审书（沿用 review book 结构）。

### 6.5 依赖关系

```
0.1 → 0.2 → 1.1 ⇄ 1.2（loop）→ 2.1 → 2.2 → 2.3 → 2.4 →（2.5 可与 2.4 并行）→ 阶段 3
```

core 与 CLI 严格串行（同仓耦合）；文档/语料任务与测试任务可按模块并行。

### 6.6 实施假设（收录自计划文件）

- `--participants` 采用逗号分隔名单输入；title 缺省取文件名主干。
- brief hash 继续对目标文件原始字节计算，换行敏感性仅文档化不修复。
- `docs/dev/adr-v1.md` 中已废止条款（DM/notify）不在本次范围内修订。

---

## 7. 附录：需求基线索引（§6 技术债清偿对照）

需求基线：`docs/researches/research-repo-formats-2026-08-08.md` §6（第 288–295 行）六项技术债。定稿规格对每一项的清偿方式如下：

| # | 技术债（基线原文要点） | 定稿规格的清偿方式 |
|---|---|---|
| 1 | `validate_seq_monotonicity` 与 `validate_markdown` 已实现但未接入任何命令路径，seq gap/断 fence 不被 validate 捕获 | `validate` 接入 `validate_seq_monotonicity`（seq 从 1 严格连续）与围栏闭合校验（§2.6） |
| 2 | `post create` system 消息占 seq #1，title/participants 以 `[Thread created: X \| participants: Y]` 正文文本脆弱编码，summary 靠字符串切分反解 | 彻底废除 system 消息：`post create` 删除；title/participants 升格为 preamble（H1 + `## Participants` bullets），`post send` 锁内首写（§2.3） |
| 3 | `Message.to` 格式层存在但 CLI 无入口，恒为 `To: all` | `post send` 新增 `--to` 参数；消息可选属性 `to:`；广播 = 省略 `to`，废除 `all` 魔法值（§2.3、§2.6） |
| 4 | 消息头分隔符 `·`（U+00B7）非 ASCII，与 README/错误文案的 `.` 不一致 | 结构符全 ASCII：消息头改为 `## #N sender (timestamp)`（纯 ASCII 括号），废除 `·` 与 `—`（§2.1 第 7 条、§2.3） |
| 5 | brief hash 对字节敏感（含换行符），跨平台换行转换可能误报 Shifted | 行为不变（字节级 SHA-256 保留），仅在 spec 中文档化声明换行敏感性，不修复（§2.4、§6.6） |
| 6 | `docs/dev/adr-v1.md` 中 DM/notify 约定已被后续 feedback 废止 | 明确不在本次范围内修订，引用时以 CHANGELOG v0.2 与 v0_feedbacks 追加段为准（§6.6） |

---

**自查记录**：本文全部 schema 代码围栏均已配对闭合（外层围栏统一采用 4 反引号以安全包裹含 3 反引号的内层示例；视角 B 示例因含 4 反引号内层围栏采用 5 反引号外层）；章节 1–7 齐全，定稿规格逐字收录自计划文件，三视角取舍与 Rejected Alternatives、风险表、阶段划分均可追溯至对应来源文件。

---

## 8. 2026-08-09 owner 追裁：post 格式三项变更

> **本节为追记追加**：v0.5.0 "Format Renewal" 实现已完成（hard breaking、未发布 crates.io）后，owner 对 post/thread 格式给出三项**定稿级**裁决。三项裁决仅对 post/thread 生效；**profile/brief/contacts 三格式固定不动**。本节与正文 §2.3（post schema）、§6.3 阶段 2.3（CLI）等早期表述冲突处，以本节及已联动的 `docs/dev/format-v2/` 五文档（spec §5/§8/§9/§10/§11、design §8.5、bdd、tdd、impl_plan S3.0）为准。
>
> **版本决策：0.5.0 未发布，本轮变更并入 0.5.0，不 bump 版本。**

### 8.1 三项裁决原文要点

1. **废除 participants**：preamble 中 `- participants:` 属性行删除。理由：对话消息中已包含全部发言者，维护名单是冗余负担。participants 语义在需要时（如 summary）由消息 sender 集合派生。
2. **废除消息属性行 reply-to / mentions / to**：这些引用状态不再是结构化字段，而是正文文本引用：
   - `@somebody`（正文内出现）= mention；
   - `@#N`（正文内出现，N 为消息序号）= reply 引用；
   - 读取/统计（summary 等）时从正文文本实时派生，不落盘。
   - `to` 字段彻底删除（不再有定向发送的结构化概念；如需定向，用户在正文自行 @）。
3. **正文围栏简化**：` ```markdown ` 改为 ` ```md `。序列化严格写 `md`；解析宽容，`md` 与 `markdown` 均接受（info-string 前缀匹配、CommonMark 规则）。动态围栏长度算法不变（max(3, 正文最长连续反引号串+1)）。

### 8.2 leader 默认规则（写入文档即为规格）

- mentions 派生：扫描正文文本，正则形如 `@([^\s@()]+)`，按出现顺序去重；**排除 sender 本人**的自提及。
- reply-to 派生：扫描正文 `@#(\d+)`，取**首个**合法引用为 reply-to（其余忽略）；不校验引用目标是否存在（宽容）。
- preamble 仅剩 H1 标题（标题行后允许自由散文，解析忽略）。
- 消息头正则 `^##\s+#(\d+)\s+([^\s()]+)\s+\((.+)\)\s*$` 不变；seq 校验、64KB 上限、fs2 锁、fence 感知尾扫全部不变。
- `post send` 的 `--to`/`--participants` flag 删除（保留 `--title`）；新文件首写 preamble 仅剩标题。
- 版本决策：0.5.0 未发布，本轮变更并入 0.5.0，不 bump 版本。
- 规格完备化补充（五文档落实时追认）：mentions 派生中 `@#N` 形态 token 归 reply 引用不计入 mentions；`--reply-to`/`--mention` 糖衣 flag 默认保留、语义改为正文 token 注入（spec §11 OQ-4）。

### 8.3 Rejected：结构化属性行方案

即维持原 §2.3 定稿（preamble `- participants:` + 消息 `- reply-to:/- mentions:/- to:` 属性行）的方案，被三项裁决整体否决：

1. participants 名单与消息 sender 双源共存，需人工同步且必然漂移；对话消息本身已包含全部发言者，名单是冗余负担。
2. reply/mention 本质是正文语境中的文本引用，结构化属性行是正文语义的冗余投影：写侧双重维护（正文写 @ 同时还要同步属性行）、读侧双通道一致性无保障。
3. 结构化字段的类型安全收益在本项目不成立：引用目标本就不校验存在性（宽容解析立场不变），属性行徒增解析/序列化/校验代码面。
4. 正文文本引用（`@somebody`/`@#N`）零新语法、单源自洽，与消息头 `#N` token 族同形互证；围栏 info 缩写 `md` 是社区惯例，写侧更短且前缀匹配保留对存量 `markdown` 围栏的读取宽容。

### 8.4 对本文早期文本的处置

- §2.3 post/thread schema 与规则、§2.6 中 `post send` 新增 `--to` 的表述、§6.3 阶段 2.3（CLI `--participants/--to`）、§6.6 实施假设（`--participants` 逗号名单）、§7 技术债 #3 清偿方式（`--to` 参数）：均以本节与 spec.md 现行为准，不再作为实现输入。
- 其余三格式（profile/brief/contacts）相关章节不受影响。
