# Managed File Format v2 规格（Normative）

> **文档性质**：v0.5 "Format Renewal" 格式规范，Normative（规范性）。实现必须逐条符合本文。
>
> **上游依据**：
> - 定稿设计整合文档（唯一设计输入）：`docs/researches/format-v2-design-synthesis-2026-08-09.md`（下称 synthesis）
> - 用户最高指令：`docs/ssot/adr/feedbacks/v0_feedbacks.md` 第 23、27 行
> - 技术选型约束：`docs/ssot/adr/初版技术选型.md`（纯原生 Markdown，禁 YAML frontmatter，禁新增 YAML 依赖）
>
> **版本说明**：本文为阶段 1 对抗性评审后按 leader 裁决（R1–R15，规格级；C1–C10，补全级）的 rework 定稿。裁决论证与被否决替代记录见 design.md §8（评审裁决记录）；与 synthesis 的差异以本文为准。
>
> **2026-08-09 owner 追裁（D1–D3，定稿级，仅涉 post/thread）**：废除 preamble `participants`（D1）；废除消息属性行 `reply-to`/`mentions`/`to`，引用状态改为正文文本引用 `@somebody`/`@#N` 并在读取/统计时实时派生、不落盘（D2）；正文围栏 info 由 `markdown` 简化为 `md`——写严格 `md`、解析宽容接受 `md` 与 `markdown`（D3）。profile/brief/contacts 三格式语义不变。0.5.0 未发布，本轮变更并入 0.5.0，**不 bump 版本**。追裁记录见 design.md §8.5 与 synthesis 末章；本文与追裁冲突处以本文为准。
>
> **配套文档**：design.md（Informative 论证）、bdd.md（行为场景编号，下文以 `BDD:<编号>` 引用）、tdd.md（测试清单）、impl_plan.md（实施计划）。
>
> **章节编号约定**：本文 §1–§11；BDD 场景编号形如 `POST-05`；TDD 条目编号形如 `T-CORE-03`。

---

## §1 范围与术语

1. 本规格约束四种 managed file format：`*.profile.md`（§4）、`*.post.md`（§5）、`*.brief.md`（§6）、`*.contacts.md`（§7），以及 `validate` 命令语义（§8）。
2. **preamble**：thread/brief/profile/contacts 文件中、首个记录单元（消息 H2 / 条目 H2 / Scope H2）之前的全部内容，承载文档级元数据。
3. **记录单元**：post 的消息（H2）与 brief 的条目（H2）。profile 的 `## Scope` H2 为 profile schema 的识别边界，其节体为 scope 属性行列表（§4.2）。
4. **fence 感知**：解析全程遵循 CommonMark **反引号围栏**规则的如下立场（R13）——N 个反引号打开的围栏，只能被长度 ≥ N、无 info string 的反引号行关闭（synthesis §2.1 第 6 条）；围栏行允许 ≤3 空格前导空白，**≥4 空格缩进的反引号行按缩进代码块内容处理，不具围栏语义**；tilde 围栏 `~~~` **不识别，按普通行处理**。
5. 关键词"必须（MUST）/禁止（MUST NOT）/应当（SHOULD）/可以（MAY）"按 RFC 2119 语义理解。
6. 兼容性：**hard breaking v0.5**。本规格不定义任何 v0.2–v0.4 旧构造（`---` 边界、`·` 分隔符、`—` 占位、`all` 魔法值、system 消息、固定 4 反引号围栏）的解析行为；解析器遇到旧构造时按"宽容解析"规则忽略或拒绝（§3.6）。

## §2 统一设计语言（四种格式共用）

以下 6 条为定稿规格（synthesis §2.1 经评审裁决 R2/R3/R4/R15 修订，裁决记录见 design.md §8），逐字约束：

1. **H1 = 文档身份**（名字/标题），H1 后首个 H2 之前的段落 = description（自由散文）。**对 profile/brief/contacts 生效**；post 例外：owner 追裁 D1 后 post preamble 仅剩 H1 标题，标题行后允许自由散文但**解析忽略**（§5.2）。
2. **扁平标量属性** = 属性行 `- key: value`（小写 ASCII 键；空/缺省即省略该行，废除 `—` 占位与 `all` 魔法值）。属性行仅在**两处有效区**生效（R4；原"post 消息属性区"已由 2026-08-09 owner 追裁 D2 废除，post preamble 亦不再有属性语义，D1）：**preamble 区**（首个记录单元 H2 之前，仅对 profile/brief 生效）、**brief 条目属性区**（条目 H2 之后至首个非属性非空行，其后同形行归 note）；其余位置出现的同形行是普通正文或被忽略（post preamble/消息区出现的同形行一律忽略，§5.2/§5.4）。例外：profile `## Scope` 节体按 §4.2 schema 识别为 scope 属性行列表（同形文法、由 profile 解析器独立识别，R3）。
3. **对其他 managed 文件的引用** = Markdown 链接（原生引用语义）。
4. **用户内容** = ` ```md ` 围栏块（owner 追裁 D3：写侧严格输出 info `md`；解析侧宽容，`md` 与 `markdown` 均接受——info-string 前缀匹配，CommonMark 规则，见 §5.4），**围栏长度动态** = max(3, 正文内最长连续反引号串 + 1)（CommonMark 规范内，取代固定 4 反引号）。**适用域裁决（R15）**：fence 包裹仅适用于 **post 消息正文**；brief note 与 profile description 属文档元叙述，为裸散文，不过 fence。
5. **记录单元**（消息/brief 条目）= H2 标题（有正文的记录单元；contacts 条目无正文，退化为链接 bullet——contacts 例外）；解析全程 fence 感知。
6. 结构符全 ASCII，废除 `·`（U+00B7）与 `—`；宽容解析（未知内容忽略）、CRLF 归一化（I11）保留。

> 原 synthesis 第 3 条"表格型数据 = GFM 表格"已被裁决 R3 废除：本规格无任何表格构造（唯一使用点 profile Scope 改为属性行列表）。否决理由（GFM 表格非 CommonMark、手写不便、属性行文法零新构造）见 design.md §8。

## §3 共享解析与序列化约定

### §3.1 CRLF 归一化（不变量 I11）

所有解析器必须在处理前将 `\r\n` 与孤立 `\r` 归一化为 `\n`（对应 `format/mod.rs::normalize_line_endings`）。序列化输出统一使用 `\n`。

### §3.2 属性行（attribute line）

文法（精确正则）：

```text
^- ([a-z][a-z0-9-]*):\s*(.*)$
```

- 键必须为小写 ASCII（可含数字与连字符），首字符为字母；大写键（旧格式 `- Model:`、`- To:` 等）不再识别，按未知内容忽略（§3.6）。
- 值取冒号后 trim 结果；值可为空字符串。
- 属性行仅在 §2 第 2 条所列两处有效区生效（owner 追裁 D2 废除原"post 消息属性区"；post preamble 仅 H1 标题，D1）：
  1. **preamble 区**：首个记录单元 H2 之前（仅对 profile/brief 生效）；
  2. **brief 条目属性区**：条目 H2 之后，延伸至首个**非属性的非空行**为止——空行不终止属性区；首个非空且非属性行的行开始 note 段，其后出现的属性行同形行一律归 note（不再具有属性语义，BDD:BRIEF-12）。
- 其余位置出现的同形行是普通正文（如 brief note 内、profile description 内的 bullet 同形行，BDD:PROF-11）；post preamble 与消息区出现的同形行不具属性语义，解析忽略（§5.2/§5.4，D1/D2）。
- profile `## Scope` 节体是上述文法的独立识别区（§4.2），不属于两处有效区。

### §3.3 fence 感知扫描

- 逐行状态机：**行首前导空白 ≤3 空格**之后为连续反引号串时进入/退出围栏（对齐 CommonMark：≥4 空格缩进是缩进代码块，其内反引号行不作围栏，BDD:POST-24）。
- tilde 围栏 `~~~` 不识别，按普通行处理（不翻转围栏状态）。
- 开启行：反引号串长度记为 N（N ≥ 3），可带任意 info string（如 `md`、`markdown`、`regex`，或无 info）。
- 关闭行：反引号串长度 ≥ N 且整行仅由反引号（与空白）构成，无 info string。
- 围栏内部的一切行（含形如 `---`、`## #N ...` 的行）都不是结构边界（见 BDD:POST-05、BDD:BRIEF-03）。

### §3.4 动态围栏长度算法

序列化用户内容时：

```text
fence_len(body) = max(3, body 内最长连续反引号串长度 + 1)
```

- 开启行 = fence_len 个反引号 + info string `md`（D3：写侧严格输出 `md`；解析侧 `md` 与 `markdown` 均接受，§5.4）；关闭行 = fence_len 个反引号。
- body 为空时 fence_len = 3。
- 解析侧必须接受关闭行长度 ≥ 开启长度（CommonMark），不得假定等长（见 BDD:POST-06）。
- TDD 强制覆盖 body 含 3/4/5/6 连续反引号各一例（synthesis §5 风险表）。

### §3.5 时间戳

- 序列化格式：RFC 3339 UTC，`%Y-%m-%dT%H:%M:%SZ`。
- 解析接受 RFC 3339（任意时区偏移，归一化为 UTC）；无时区的 `%Y-%m-%dT%H:%M:%S` 视为 UTC（保留现有 `parse_timestamp` 行为）。
- 解析失败产生 `PaperworkError::Parse`（§9.2）。后果披露见 design.md §3（消息头时间戳解析失败 → 整文件 Parse，全线程不可读）。

### §3.6 宽容解析

- 未识别的标题、列表项、段落一律忽略，不报错。
- 必需结构缺失（如无 H1、无 `- model:`）才报 `Parse` 错误（各格式章节列明）。
- 旧格式构造（`·` 头、`---` 边界、大写键、`—`、`all`）不获得任何特殊语义：`·` 头不匹配新消息头正则，故归入 preamble/正文或被忽略——旧文件在新解析器下解析为空/缺字段，由 `validate` 拒绝（§8）。

## §4 profile（`*.profile.md`）

### §4.1 schema 范例

````markdown
# alice

Parser module implementer

- model: gpt-4o

## Scope

- read: src/**
- write: src/parser/**
- owns: src/parser/**
````

### §4.2 结构规则

- preamble = H1（name）+ 可选 description 散文段 + 属性行。
- **必需**：H1（name）、`- model:`。description 与 `## Scope` 节可选；**空 scope = 省略整节**（禁止 `—` 占位行）。
- `## Scope` 节体为**属性行列表**（R3）：一行一个 (permission, glob) 对，形如 `- <permission>: <glob>`；键 `read`/`write`/`owns` 可重复。
- permission ∈ { `read`, `write`, `owns` }；未知 permission 的行忽略（§3.6）。
- glob 为裸文本（不再反引号包裹），取冒号后 trim 值。
- 同一 permission 可出现多行（多个 glob），解析按行序保序聚合。

### §4.3 序列化

```text
# <name>\n
\n
<description>\n          ← 仅当 description 非空，其后跟空行
\n
- model: <model>\n
\n
## Scope\n               ← 仅当 scope 非空
\n
- <perm>: <glob>\n…      ← 每个 (permission, glob) 一行，顺序 read → write → owns
```

空行规则：各块之间恰好一个空行；文件末尾恰好一个 `\n`。roundtrip 必须成立（BDD:PROF-10）。

### §4.4 错误

- 无 H1 → `Parse { message: "missing agent name heading (# <name>)", fix, example }`。
- 无 `- model:` → `Parse { message: "missing - model: line for profile '<name>'", ... }`。
- fix/example 文案纯 ASCII，见 §9.2。

## §5 post / thread（`*.post.md`）

### §5.1 schema 范例

````markdown
# Daily Standup

## #1 alice (2026-08-01T19:38:22Z)

```md
Parser module is 80% done.
```

## #2 bob (2026-08-01T19:38:22Z)

```md
@alice tests merged, all green.（@#1 = 回复引用 #1，@somebody = mention，均在正文文本内，§5.4）
```
````

### §5.2 preamble

- preamble = 首个 fence 感知匹配消息头（§5.3 正则）的 H2 之前的全部内容。
- **preamble 仅剩 H1 标题**（owner 追裁 D1）：H1 = title；标题行后允许自由散文，解析忽略；preamble 中出现的属性行（含历史形态 `- participants:`）与非匹配 H2 一律归 preamble 忽略（宽容解析，§3.6）。
- **participants 废除**（D1）：理由——对话消息中已包含全部发言者，维护名单是冗余负担；participants 语义在需要时（如 summary）由消息 sender 集合派生（§5.4 派生规则），不落盘。
- preamble 解析结果映射为 `ThreadMeta { title }`（`lib.rs` 类型；原 participants 字段随 D1 删除）。**`ThreadMeta` 仅用于解析读取**：`thread_edit` 重写不依赖其再序列化（§5.7）。
- 无 H1 的 preamble：title 取空字符串（宽容解析），但 CLI 写入路径必须产出 H1。

### §5.3 消息头（精确正则）

```text
^##\s+#(\d+)\s+([^\s()]+)\s+\((.+)\)\s*$
```

- 捕获组：seq（十进制无符号）、sender、timestamp 字符串。
- sender 必须是无空格、无括号的 token：解析侧由 `([^\s()]+)` 强制，写入侧由 §5.6 字符集校验一致拒绝括号 sender（R1）。
- **空白宽容**（R9）：字段间 `\s+`、行尾 `\s*$`（容忍尾随空白）；序列化仍输出规范单空格形态（§5.9）。
- 消息头**必须顶格**：`##` 之前的任何前导空白使其退化为 preamble/正文，不构成消息边界。
- timestamp 为纯 ASCII 圆括号包裹的 RFC 3339（§3.5）；贪婪捕获下头行尾部垃圾（如 `(ts) (备注)`）导致时间戳解析失败 → 整文件 Parse（BDD:POST-28，后果披露见 design.md §3）。
- 不匹配该正则的任何 H2 都不是消息边界。
- 边界扫描必须 fence 感知（§3.3）。

### §5.4 消息正文与引用派生

- 消息 = H2 消息头 + 正文围栏，**无消息属性区**（owner 追裁 D2：`- reply-to:`/`- mentions:`/`- to:` 属性行废除）。消息头与首个围栏之间的任何内容（历史属性行同形行、散文等）解析忽略（宽容）。
- **正文提取（规范性规则，R12）**：body = 围栏开启行与关闭行之间的行序列，**去除首尾空白行**后以 `\n` 连接；roundtrip 保证仅对规范化后 body 成立（BDD:POST-23）。
- 正文围栏的 info string 解析宽容（D3 + C2）：`md` 与 `markdown` 均接受（info-string 前缀匹配，CommonMark 规则）；其余任意 info（含无 info）亦接受为正文围栏；**写入侧统一严格输出 `md`**。
- 一条消息出现多个围栏时：**取首个**为正文，其余忽略。
- 围栏缺失时正文为空字符串（宽容解析）。
- **引用状态不再落盘**（D2）：mention / reply-to 是正文文本引用，读取/统计（summary、read 过滤等）时从正文文本实时派生，序列化不输出任何派生结果；`to` 字段彻底删除（不再有定向发送的结构化概念；如需定向，用户在正文自行 `@`）。`Message` 持久化模型不含 reply_to/mentions/to 字段。

**派生算法（leader 默认规则，规范性）**：

- **mentions 派生**：扫描正文文本，正则形如 `@([^\s@()]+)`，按出现顺序去重；**排除 sender 本人**的自提及；捕获值形如 `#<纯数字>` 的 token 是 reply 引用（见下条），不计入 mentions。
- **reply-to 派生**：扫描正文 `@#(\d+)`，取**首个**合法引用为 reply-to（其余忽略）；**不校验引用目标是否存在**（宽容）。
- **participants 派生**（需要时，如 summary，D1）：由全部消息 sender 集合派生，按消息首次出现顺序去重。
- 派生仅作用于读取/统计路径；正文中 `@` 未构成合法 token（如孤立的 `@`、`@ ` 后随空白/行尾、`@)` 等）时不派生任何结果且不报错（BDD:POST-33/34）。

### §5.5 尾部 O(1) seq 扫描（精确正则）

```text
(?m)^##[ \t]+#(\d+)
```

- 用于 `read_last_seq_locked` 的反向尾扫（缓冲区 64KB+256B 不变）；多行模式使 `^` 匹配行首；`[ \t]+` 为 §5.3 字段间 `\s+` 的行内等价（避免 `\s` 跨行）。
- **尾扫 fence 感知（缓冲区内，R6）**：对缓冲区内围栏行按 CommonMark 长度规则（§3.3）做开合追踪；处于开启围栏内部的候选头跳过，不参与 seq 取值。
- **残留限制（明文声明）**：缓冲区起点之前的围栏奇偶状态不可知（缓冲起点可能切断围栏）；该构造下 fence 内候选头仍可能污染 seq，由 `validate` 的 seq 连续性校验（§8）兜底暴露（BDD:CONC-03）。
- **丢弃首行规则（R7）**：仅当 `read_start > 0` 时检查前一字节，非 `\n` 才截到第一个 `\n` 之后（丢弃不完整首行）；`read_start == 0`（缓冲区覆盖整个文件）时**不丢弃任何行**（首行即完整行，丢弃将吞掉首个消息头造成 seq 重复）。

### §5.6 sender 字符集校验（写入侧）

- `thread_send` 必须校验 sender：非空、无空白字符（空格/tab/换行）、无 `(` `)`。
- 违规 → `PaperworkError::Validation`（§9.2，BDD:POST-17）。
- sender 允许非 ASCII token（字符集 `[^\s()]+` 按 Unicode 类理解），无长度上限（受 64KB 单条上限隐式约束，BDD:POST-13）。
- 正文文本引用 token（`@somebody`/`@#N`）不做写入侧校验（宽容；派生规则见 §5.4，D2）。

### §5.7 system 消息废除与 preamble 首写

- `post create` 命令删除；seq #1 直接是首条真实消息。
- `post send` 保留 `--title`（缺省算法：**剥 `.post.md` 后缀，否则剥 `.md` 后缀，否则取原名**）；**`--to`/`--participants` flag 删除**（owner 追裁 D1/D2）；`--reply-to`/`--mention` 糖衣 flag 的去留见 §11 OQ-4（**2026-08-15 裁决指针：该默认已被 owner 裁决撤销，写侧糖标志已从 CLI 面全量移除，reply/mention 语义改为正文 token 直书（`@#N`/`@name`）、读侧派生；以 cli-grammar-v0.6 spec 与 docs/dev/owner-rulings-2026-08-15.md 为准**）。**仅当文件在锁内为空（size = 0）时**，先写入 preamble（仅 H1 title，D1），再追加消息；文件非空时 `--title` 忽略（见 §11 OQ-1）。
- preamble + 首条消息必须在同一次锁内写入。**preamble 不受 64KB 单条上限约束**（该上限仅约束单条消息，§5.8）。
- `thread_edit` 全文件重写时必须保留 preamble：**对首个消息头之前的字节区间原样搬运，不做规范化重序列化**（手写散文、额外 H2 节等 preamble 内容在 edit 后逐字节保留，BDD:POST-16/POST-29）。
- **崩溃窗口声明**：`thread_edit` 锁内"截断 + 重写"存在崩溃窗口——断电/进程杀会丢失全文件（含 preamble）；接受该窗口：fs2 锁已排除并发写者，仅断电/杀进程暴露（后续加固方向为锁内 temp+rename，本次不做）。

### §5.8 保留约束

append-only、fs2 排他锁内分配 seq、单条序列化后 ≤ 64KB、单次 `write_all`、`thread_edit` 三重约束（本人消息 / 本人最新 / 线程末条）全部保留，行为不变（§10）。**64KB 上限同时约束 `thread_send` 与 `thread_edit`**：`thread_edit` 对新 body 序列化后执行 ≤64KB 校验，超限 `MessageTooLarge` 且文件不变（R8，BDD:POST-30）。

### §5.9 序列化格式（精确）

```text
serialize_preamble(meta):
  # <title>\n
  \n

serialize_message(msg):
  ## #<seq> <sender> (<RFC3339-Z>)\n
  \n
  <fence_len 个反引号>md\n        ← D3：严格输出 md
  <body>\n
  <fence_len 个反引号>\n
  \n

serialize_thread(meta, messages) = serialize_preamble + 各 serialize_message 顺序连接
```

- 消息头序列化输出规范单空格形态（§5.3 空白宽容仅适用于解析侧）。
- 消息头与围栏之间不输出任何属性行（D2）；preamble 不输出 `- participants:` 行（D1）。
- 空 body 仍必须包裹围栏（` ```md\n ``` `，fence_len=3）。
- `serialize_preamble` 仅承载 `ThreadMeta` 的 title 字段；手写 preamble 额外内容的保留由 §5.7 原样搬运机制保证，不经再序列化。

## §6 brief（`*.brief.md`）

### §6.1 schema 范例

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

### §6.2 结构规则

- preamble = H1（title）+ 可选 description 散文段 + 属性行。
- **必需**：H1、`- owner:`、`- created:`（RFC 3339，§3.5）。description 可选。
- 条目 = H2（条目 title 即标题文本）+ 属性列表 + 可选散文 note 段：
  - `- path: <相对路径或 glob>`（裸文本，无反引号包裹）
  - `- hash: <sha256-hex>`（**全量 64 位小写 hex，不截断**）
  - `- regex: <pattern>`（简单模式内联；无 regex = 省略该行）
- **条目属性区边界**（§3.2 两处有效区之二）：条目 H2 之后至首个非属性非空行为属性区（空行不终止）；首个非空且非属性的行开始 note，其后同形属性行归 note（BDD:BRIEF-12）。
- 复杂 regex（含换行、反引号或不便内联的字符）使用 ` ```regex ` 围栏块替代 `- regex:` 行（CommonMark 合法的 info-string 围栏，保留此逃生口；BDD:BRIEF-03）。
- note 为裸散文段（取代旧 blockquote note；适用域裁决 R15：note 属文档元叙述，不过 fence）；无 note = 省略。
- `groups` 仍由命名捕获组 `(?<name>...)` 派生，不落盘。
- 不再有 `## Entries` 包装节：条目 H2 直接位于 preamble 之后。

### §6.3 序列化

```text
# <title>\n\n
<description>\n\n        ← 仅当非空
- owner: <author>\n
- created: <RFC3339-Z>\n\n
## <entry title>\n\n
- path: <path>\n
- hash: <hash>\n
- regex: <pattern>\n     ← 简单模式；复杂模式改用 ```regex 围栏；无则省略
\n
<note 散文段>\n\n        ← 仅当 note 非空
```

### §6.4 验证三态（行为不变）

Fresh / Shifted / Stale 语义与字节级 SHA-256（I7）保持不变；技术债 #5 仅文档化：**hash 对目标文件原始字节计算，换行符差异会导致 Shifted**，不修复（BDD:BRIEF-09）。

### §6.5 错误

- 无 H1 → `Parse { message: "missing title heading (# <title>)", ... }`。
- 无 `- owner:` / `- created:` → `Parse`（fix/example 使用小写键文案，§9.2）。

## §7 contacts（`*.contacts.md`）

### §7.1 schema 范例

````markdown
# Core Team

- [alice](agents/alice.profile.md)
- [bob](agents/bob.profile.md)
````

### §7.2 结构规则

- preamble：H1 = contacts 标题（必需，缺失报 `Parse`）。
- 条目 = Markdown 链接 bullet：`- [<label>](<destination>)`。label 为 profile 名，destination 为 profile 文件路径。
- 解析必须接受两种 destination 形式：
  1. 裸路径：`[alice](agents/alice.profile.md)`
  2. 尖括号路径：`[alice](<agents/my profile.md>)`（destination 以 `<` 开始、`>` 结束，内容取出后反转义 `\<`/`\>`）
- `[label](path "title")` 形式：title 语法**不接受**，解析忽略 title 部分、照常提取 destination（宽容）。
- 解析侧必须反转义 label 中的 `\]`（与 §7.3 序列化转义构成 roundtrip，BDD:CONT-08）。
- 非链接的普通 bullet（裸路径）不再识别为条目，忽略（§3.6）。
- 读取时即时增强 profile 简介（CLI `enrich_profile`）行为不变。

### §7.3 序列化与转义规则

- 序列化形式：`- [<label>](<path>)\n`。
- **转义规则**：path 含空格、tab、`(`、`)`、`<`、`>` 中任一字符时，destination 必须序列化为尖括号形式 `[label](<path>)`（其中 `<`/`>` 字符转义为 `\<`/`\>`）；否则用裸形式。
- label 序列化时若含 `]` 需反斜杠转义 `\]`（防御性规则；正常 profile 名不触发）。
- **label 来源**（R11）：写入时读取目标 profile 的 H1 作为 label；读取失败回退文件名主干——主干算法：**先剥 `.profile.md` 后缀，再剥 `.md` 后缀，否则取原名**（如 `alice.profile.md` → `alice`）。见 §11 OQ-2。

### §7.4 错误

- 无 H1 → `Parse { message: "missing contacts title heading (# <title>)", ... }`。

## §8 validate 语义（清偿技术债 #1、#3）

`paperwork validate <path>` 按后缀分发（`.post.md` / `.profile.md` / `.brief.md` / `.contacts.md`），未知后缀报 `Parse`。

对 `.post.md`，依次执行且全部通过才为 ok：

1. `parse_messages` 成功且消息数 ≥ 1（**空内容或非空文件零消息 → `Parse`**；注意空文件拒绝是**对 v0.4 现状的行为变更**——现状豁免空文件，本规格删除该豁免，BDD:VAL-07）；
2. `validate_seq_monotonicity`：seq 从 1 开始、严格连续无 gap（本次**接入**，此前未接线）；
3. `validate_markdown`（围栏闭合校验，升级为动态长度 fence 感知，§3.3）；
4. **疑似消息头启发式**（R9）：形似 `## #<数字>` 开头但不严格匹配 §5.3 文法、且不在围栏内的行，报 **warning + fix**；warning 不改变 ok/error 结论（把静默吞消息转为显式提示，BDD:VAL-08）。

**错误信封形态（R10）**：validate 直接透出各步骤的底层错误变体，**不统一重包为 `Parse`**——步骤 1/3 失败 → `Parse`（category `format`）；步骤 2 失败 → `Validation`（category `validation`，BDD:VAL-02）。

对其余三种格式：对应 parser 成功 + `validate_markdown` 围栏闭合校验（错误信封同上直出）。

**技术债 #3 的清偿方式变更（owner 追裁 D2）**：`to` 字段彻底删除，不再存在定向发送的结构化概念；如需定向，用户在正文自行 `@`。`post send` 的 `--to` 参数随之删除；`--participants` 亦随 D1 删除。

## §9 错误分类（对齐 `error.rs::PaperworkError`）

### §9.1 变体与 category 对照

| 变体 | category() | 本规格触发点 |
| --- | --- | --- |
| `Parse` | `format` | §4.4、§5.3/§5.4 头/时间戳解析失败、§6.5、§7.4、§8 步骤 1/3 各拒绝项（含空文件） |
| `Validation` | `validation` | seq 单调性（§8 步骤 2，validate 直出此信封）、sender 字符集（§5.6）、空 body（CLI 保留） |
| `MessageTooLarge` | `validation` | 单条 > 64KB（§5.8，`thread_send` 与 `thread_edit` 同守） |
| `NotFound` / `AlreadyExists` / `NotAllowed` / `IoContext` / `Io` | 不变 | 行为与现有一致，仅 example 文案更新为新命令形态 |

### §9.2 关键 fix/example 文案（纯 ASCII，实现时逐字采用）

| 场景 | fix | example |
| --- | --- | --- |
| 非法消息头 | `expected format: ## #<seq> <sender> (<timestamp>)` | `## #1 alice (2026-01-15T10:30:00Z)` |
| 疑似消息头（validate warning） | `expected format: ## #<seq> <sender> (<timestamp>)` | `## #1 alice (2026-01-15T10:30:00Z)` |
| 非法时间戳 | `use RFC 3339 format: YYYY-MM-DDTHH:MM:SSZ` | `2026-01-15T10:30:00Z` |
| sender 字符集违规 | `sender must be a single token without spaces or parentheses` | `paperwork post send standup --from alice "Hello"` |
| profile 缺 model | `add a '- model: <model-id>' bullet line` | `- model: gpt-4o` |
| brief 缺 owner | `add a '- owner: <agent>' bullet line` | `- owner: alice` |
| brief 缺 created | `add a '- created: <RFC3339>' bullet line` | `- created: 2026-01-15T10:00:00Z` |
| seq 首值非 1 | `thread messages must start at seq 1` | （空） |
| seq gap | `message sequence numbers must be consecutive with no gaps` | （空） |
| validate 零消息 | `expected '## #<seq> <sender> (<timestamp>)' headers with dynamic md fences` | `paperwork post send myfile --from alice "hello"` |

错误信封协议（`ok/error` + `fix:` + `example:`，`output.rs`）**零改动**。

## §10 不变量清单

| 编号 | 不变量 | 实现锚点（现状） |
| --- | --- | --- |
| I1 | thread append-only：`thread_send` 仅追加；唯一例外是 `thread_edit` 锁内全文件重写（且三重约束） | `ops/thread.rs` |
| I2 | seq 在 fs2 `lock_exclusive` 锁内分配；尾扫 O(1)（缓冲区 64KB+256B，缓冲区内 fence 感知，§5.5） | `read_last_seq_locked` |
| I3 | 单条消息序列化后 ≤ 64KB（`MAX_MESSAGE_SIZE`），超限 `MessageTooLarge`；`thread_send` 与 `thread_edit` 同守 | `thread_send`、`thread_edit` |
| I4 | 追加路径锁内合并为一次 `write_all`（避免交错写；真实互斥来自 fs2 锁，非 syscall 原子性） | `thread_send` |
| I5 | fence 感知解析（CommonMark 反引号围栏子集立场：N 反引号围栏仅被 ≥N 反引号关闭；≤3 空格缩进；tilde 不识别） | §3.3 |
| I6 | 动态围栏长度 = max(3, 正文最长连续反引号串 + 1) | §3.4 |
| I7 | brief hash = 目标文件原始字节的 SHA-256（小写 hex，全量不截断，换行敏感仅文档化） | `hash.rs` |
| I8 | 结构符全 ASCII；解析前 CRLF 归一化（**I11**，沿用代码注释编号）；序列化输出 LF | `format/mod.rs::normalize_line_endings` |
| I9 | preamble 与消息在同一次锁内首写；preamble 在 `thread_edit` 重写后**原文字节保留**（§5.7） | §5.7 |
| I10 | 引用状态（mentions/reply-to）为正文文本引用（`@somebody`/`@#N`），读取/统计时实时派生、**不落盘**；participants 由 sender 集合派生（D1/D2）；正文围栏 info 写严格 `md`、解析宽容接受 `md`/`markdown`（D3） | §5.4、§5.9 |

## §11 OPEN-QUESTION 汇总

> 规格内部歧义点。均已给出本文档采用的确定性默认，实现前应由 leader/用户确认；实现者不得擅自更改默认。

- **OQ-1（`--title` 对存量线程的行为）**：synthesis §2.3 仅规定"`post send` 新建文件时锁内首写 preamble"。对已存在且非空的文件传入 `--title` 时的行为未定义。本文默认：**锁内文件非空时忽略 `--title`（静默）**，不引入新告警/错误信封。涉及章节：§5.7。（原 `--participants` 部分随 owner 追裁 D1 废除，flag 已删除。）
- **OQ-2（contacts link label 来源）**：synthesis §2.5 规定 label 为 profile 名，但未规定序列化时取目标 profile 的 H1 还是文件名主干。本文裁决（R11）：**写入时读取目标 profile 的 H1 作为 label，读取失败回退文件名主干**（先剥 `.profile.md` 再剥 `.md`，否则原名）。此为新增派生规则，与读取侧 `enrich_profile` 的命名直觉对齐（现状序列化输出裸路径、不存在 label 派生逻辑，原"既有行为一致"论据失实，已更正）。涉及章节：§7.3。
- **OQ-3（已失效）**：原"`- to:` 多值语义"随 owner 追裁 D2 整体失效——`to` 字段与 `- to:` 属性行彻底删除，无多值语义可言。保留编号以免引用漂移。
- **OQ-4（`--reply-to`/`--mention` 糖衣 flag 的去留，2026-08-09 追裁新增）**：D2 删除的是**结构化字段**；leader 明示删除的 flag 仅 `--to`/`--participants`（保留 `--title`），未提及 `--reply-to`/`--mention`。本文默认：**保留**这两个 flag（与 cli-ux-redesign 签名 `post send <PATH> <NAME> [BODY] [--stdin] [--reply-to N] [--mention a,b]` 一致），语义变更为**正文 token 注入**——CLI 在 send 序列化时于 body 首部注入 `@#N` 与各 `@name` token（空格分隔，其后换行接原 body），使 §5.4 派生规则可还原；reply 隐式 @ 原发送者逻辑同理改为注入。此为 CLI 层行为，不影响文件格式。实现者不得擅改默认；如 leader 另有裁决以裁决为准。涉及章节：§5.4、§5.7。（**2026-08-15 裁决指针注记，任务 #45 F3 / S2-03**：owner 于 2026-08-15 裁决（docs/dev/owner-rulings-2026-08-15.md，实施链：任务 #35 spec 增量修订 + 任务 #36 代码实施 9821933/f94b65f）**撤销**本 OQ 的「保留」默认：`--reply-to`/`--mention` 写侧糖标志已从 CLI 面全量移除；reply/mention 语义由用户在正文直书 token（`@#N`/`@name`）、读侧按 §5.4 派生；文件格式不变，§5.4 派生规则不变。本 OQ 保留编号与历史文本供溯源，现行行为以 cli-grammar-v0.6 spec（docs/ssot/specs/cli-grammar-v0.6/spec.md）为准。）
