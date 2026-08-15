# CLI UX 重设计 v0.5.0 — Spec（行为规范）

> **Superseded-by note (v0.6)**: 本文文法层（位置参数文法三规则与命令签名）已被 v0.6 具名文法整体取代，现行文法以 `docs/ssot/specs/cli-grammar-v0.6/spec.md` 为准；本文保留为历史治理档案，正文不可改写。（Superseded by the v0.6 named-flag grammar, authoritative in `docs/ssot/specs/cli-grammar-v0.6/spec.md`. Historical content below is immutable.）

- 日期：2026-08-09
- 版本：v0.5.0
- 文档性质：行为规范（命令契约 + 输出协议），实现与测试的唯一验收基准
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.5_feedbacks.md`（owner 指令，最高优先级）
  - `docs/ssot/adr/feedbacks/v0_feedbacks.md`（v0 系列 owner ADR，#3.1 content 置末）
  - `docs/dev/adr-v1.md`（ADR-011：stateless / path-explicit / 无登录）
  - `docs/researches/cli-ux-agent-visible-output-research-2026-08-08.md`（现状全貌）
  - `docs/researches/ux-open-items-backlog-2026-08-08.md`（遗留项编号 U-xx/R-xx/F-xx/N-xx）
  - `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md`（业界基准）

---

## 1. 核心文法（总纲与三条规则）

### 1.1 总纲

```
paperwork [全局flag] <组> <动词> <PATH> [<NAME>] [<载荷>] [--可选修饰flag]
```

### 1.2 规则 1（槽位）

`<PATH>` 恒为第 1 必填位置参数；凡产生 agent 署名写入的动词（仅 post send / post edit），`<NAME>` 为第 2 必填位置参数；content 类参数恒为最后一个位置参数（v0_feedbacks #3.1）。

### 1.3 规则 2（唯一语义）

全 CLI 任何 flag 只有一种含义。`--from` 的身份语义随位置参数化消亡，`--from/--to` 从此仅存在于 post read，仅表 seq 范围，U-01 冲突结构性消解。

裁定（规则 2 边界）：`--mention/--reply-to` 在 send 中为「设置」、在 read 中为「过滤」，视为同一语义对象（mention 名单 / reply 锚点 seq）的同构延伸，**不构成双语义**（值语义同构：分别为名字与 seq）。

### 1.4 规则 3（判据）

必填即位置参数，可选才做 flag——未来新命令无需再讨论。

### 1.5 无 actor 命令规则

`<NAME>` 仅在动词产生可署名写入时出现（send/edit 两处）；只读/系统记录/目录级命令一律单位置参数，零特例。

---

## 2. 新文法全表（命令签名契约，与总方案逐字一致）

| 命令 | 新签名 | 变更 |
|---|---|---|
| post send | `post send <PATH> <NAME> [BODY] [--stdin] [--reply-to N] [--mention a,b]` | `--from`→位置参数 NAME（owner 指令直接落实） |
| post edit | `post edit <PATH> <NAME> <SEQ> [NEW_BODY] [--stdin]` | `--from`→NAME、`--seq`→SEQ 位置参数 |
| post read | `post read <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]` | 不变（--from/--to 成为全 CLI 唯一语义） |
| post create | `post create <PATH> <TITLE> [--participants a,b]` | `--title`→位置参数 |
| post summary | `post summary <PATH>` | 不变 |
| profile create | `profile create <PATH> <NAME> [--model] [--description] [--scope-read/write/owns]` | `--name`→位置参数 |
| profile show/edit/list | `<PATH>` / `<PATH> [--field..]` / `<DIR>` | 不变（无 actor） |
| brief create | `brief create <PATH> <TITLE> [--owner] [--description]` | `--title`→位置参数 |
| brief add | `brief add <PATH> <ENTRY> [--regex] [--note]` | `--entry`→位置参数 |
| brief remove | `brief remove <PATH> <ENTRY-TITLE>` | `--entry-title`→位置参数 |
| brief read/verify | `<PATH> [--full]` / `<PATH> [--base-dir]` | 不变 |
| contacts create | `contacts create <PATH> [--title]` | 不变（title 有默认值，属可选，按规则 3 保留 flag） |
| contacts add | `contacts add <PATH> <PROFILE-PATH>` | `--profile`→位置参数 |
| contacts read / validate | `<PATH>` | validate 新增可选 `--type`（U-15，additive） |

---

## 3. 逐命令契约

约定：`<X>` = 必填位置参数；`[X]` = 可选位置参数；`--x` = flag。默认值以 v0.4.0 现状为准，本版本不改默认值语义。

### 3.1 post 组

#### post send

```
paperwork post send <PATH> <NAME> [BODY] [--stdin] [--reply-to N] [--mention a,b]
```

| 位置 | 参数 | 必填 | 说明 |
|---|---|---|---|
| 1 | `PATH` | 是 | 线程文件路径；经 §5.1 后缀解析 |
| 2 | `NAME` | 是 | 发送者名字（署名），原 `--from`（U-01 消解） |
| 3 | `BODY` | 否（与 `--stdin` 二选一） | 消息正文；content 恒末位（v0_feedbacks #3.1） |

- flag：`--stdin`（从 stdin 读正文，与位置 BODY 互斥，同时给出报 validation 错误）；`--reply-to N`（u64，隐式把原消息发送者并入 mentions）；`--mention a,b`（逗号分隔）。
- 行为保留：线程不存在时自动创建；空正文（trim 后为空）拒绝（validation）。
- **NAME/BODY 混淆面（位置文法固有边界）**：`post send <PATH> "text"`（PATH + 单字符串）中该字符串必然绑定必填 NAME 槽、BODY 缺省——CLI 无法区分「漏 NAME」与「给了 NAME 缺 body」；该形态落 **validation exit 1**（非 usage），其 fix/example 必须展示含 NAME 槽的完整命令形态（并教 `--` 用法）；仅 `post send <PATH>`（缺必填位置参数）构成 usage exit 2。教学补偿三件套见 design.md §2.5。
- **NAME 校验语义（沿用 v0.4）**：NAME 不与 profile/contacts 做存在性/一致性校验；空串或 trim 后为空的 NAME 拒绝（validation）；NAME 可含空格；含逗号的 NAME 会与 `--mention a,b` 列表解析及 implicit-mention 比对产生歧义，不建议使用（无硬约束）。
- 输出增补（U-10，additive）：reply-to 隐式 mention 至多触发一人（原消息发送者）；仅当触发时，ok 信封字段区增补**单数字段** `implicit-mention: <name>`，不触发则不输出该字段；默认档与 `--json` 一致（JSON 同名 key）。**三种不触发边界（行为沿用 v0.4）**：① 原消息发送者即本次发送者本人（自回复）；② 原发送者已在显式 `--mention` 名单中；③ `--reply-to` 指向的 seq 不存在（静默跳过）。

#### post edit

```
paperwork post edit <PATH> <NAME> <SEQ> [NEW_BODY] [--stdin]
```

| 位置 | 参数 | 必填 | 说明 |
|---|---|---|---|
| 1 | `PATH` | 是 | 线程文件路径 |
| 2 | `NAME` | 是 | 编辑者名字，须与原消息发送者一致（原 `--from`） |
| 3 | `SEQ` | 是 | 目标消息序列号 u64（原 `--seq`） |
| 4 | `NEW_BODY` | 否（与 `--stdin` 二选一） | 新正文，content 末位 |

- 三重编辑护栏（只能编辑：自己的、自己最新的、线程最后一条）行为不变，违规 not-allowed。
- SEQ 位于 NEW_BODY 之前：非 content 必填参数前置（规则 1），content 恒末位。

#### post read（不变）

```
paperwork post read <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]
```

- `--from/--to` 仅表 seq 范围（规则 2 的唯一语义承载者）；`--limit` 默认 20。
- 输出增补（U-11，additive，字段区形态、不放 conclusion 行）：**恒显** `showing: <n>/<total>`（即使未超限）；`<total>` = 过滤后（--mention/--reply-to）、`--limit` 截断前的消息数（无过滤时为线程全部条数，与 conclusion 行 N 同口径，与 v0.4 现状一致）；并增补 `window: #<first>-#<last>`（取实际展示的第一条与最后一条 seq，线程基准；空线程不显示 window）。

#### post create

```
paperwork post create <PATH> <TITLE> [--participants a,b]
```

- TITLE 位置参数化（原 `--title` 必填 flag，规则 3）；`--participants` 逗号分隔可选。
- create 仍注入 #1 系统消息（线程创建双轨问题 U-03/R-01/N-03 裁决延后，本版不触碰文件格式层）。

#### post summary（不变）

```
paperwork post summary <PATH>
```

### 3.2 profile 组

#### profile create

```
paperwork profile create <PATH> <NAME> [--model] [--description] [--scope-read ...] [--scope-write ...] [--scope-owns ...]
```

| 位置 | 参数 | 必填 | 说明 |
|---|---|---|---|
| 1 | `PATH` | 是 | profile 文件路径；ensure_suffix 补 `.profile.md` |
| 2 | `NAME` | 是 | agent 名字（原 `--name` 必填 flag，U-06 消解） |

- flag：`--model`（默认空串）、`--description`（默认空串）、`--scope-read/--scope-write/--scope-owns`（多值，可选）。

#### profile show / edit / list（不变）

```
paperwork profile show <PATH>
paperwork profile edit <PATH> [--model] [--description] [--scope-read ...] [--scope-write ...] [--scope-owns ...]
paperwork profile list <DIR>
```

- 无 actor 语义；edit 未给字段保留原值，输出 `changed: <fields>`；list 容错式（解析失败显示 `(unreadable)`）。

### 3.3 brief 组

#### brief create

```
paperwork brief create <PATH> <TITLE> [--owner] [--description]
```

- TITLE 位置参数化（原 `--title` 必填 flag）；`--owner` 可选、`--description` 默认空串。

#### brief add

```
paperwork brief add <PATH> <ENTRY> [--regex] [--note]
```

- ENTRY（条目文件路径，相对 brief 目录）位置参数化（原 `--entry` 必填 flag，U-07 消解）；add 时自动计算目标文件 SHA-256 快照（行为不变）。

#### brief remove

```
paperwork brief remove <PATH> <ENTRY-TITLE>
```

- ENTRY-TITLE 位置参数化（原 `--entry-title` 必填 flag）。推导规则：add 的 ENTRY 是相对 brief 目录的**文件路径**，条目存储标题取其 **basename**（`src/main.rs` → `main.rs`）；故 remove 传的是该 basename，而非原相对路径。

#### brief read / verify（不变）

```
paperwork brief read <PATH> [--full]
paperwork brief verify <PATH> [--base-dir <DIR>]
```

- verify 三态（fresh/shifted/stale）判定与 conclusion `N/M fresh` 不变。

### 3.4 contacts 组

#### contacts create（不变）

```
paperwork contacts create <PATH> [--title]
```

- `--title` 有默认值 `Contacts`，属可选，按规则 3 保留 flag（总方案逐字规定）。

#### contacts add

```
paperwork contacts add <PATH> <PROFILE-PATH>
```

- PROFILE-PATH 位置参数化（原 `--profile` 必填 flag，U-07 消解）。

#### contacts read（不变）

```
paperwork contacts read <PATH>
```

- 富化输出（路径 + name + description）行为不变。

### 3.5 validate

```
paperwork validate <PATH> [--type post|profile|brief|contacts]
```

- `--type` 为新增可选 flag（U-15，additive）：给出时按指定类型解析，覆盖后缀推断；未给出时维持现状——按后缀推断，未知后缀报 format 错误。
- 成功输出 `ok validate <path>`，失败输出 error 信封（exit 1）。

### 3.6 别名与隐藏别名

- 现有隐藏别名 `p`(profile)/`b`(brief)/`c`(contacts)/`v`(validate) 不变。
- 新增 post 隐藏别名 `po`（U-08/N-01）。`p` 与 `po` 不冲突；不引入子命令级别名。

---

## 4. 输出 envelope 契约

### 4.1 成功信封（stdout，Default 模式，冻结不变）

```
ok <command> <conclusion>
<key>: <value>
---
<body lines>
```

- command 标识（`post.send`、`post.read`、`profile.create`、`validate` 等）**全部不变**，与参数文法解耦。
- conclusion 短语语义不变（如 `#3 -> <path>`、`6 messages`、`1/2 fresh`）。
- `-q` 仅隐藏 `ok ...` 首行，字段与正文保留（冻结不变）。

### 4.2 错误信封（stderr，七种 category——含新增 usage）

```
error <category>: <message>
fix: <corrective action>
example: <corrected command>
```

category 词表为**冻结枚举，仅可经评审流程扩展**：运行时六类（由 core `PaperworkError::category()` 决定）冻结不变——`format`（Parse）、`validation`（Validation/MessageTooLarge）、`io`（Io/IoContext）、`not-found`（NotFound）、`already-exists`（AlreadyExists）、`not-allowed`（NotAllowed）；本次经对抗评审流程确认扩展第七类 `usage`（见 §4.3）。

- 全部 example 字符串刷新为新文法（CLI 层与 core 层，见 impl_plan.md 步骤②④）。
- **example 书写约定**：所有 example 一律为**具体可复制执行**的命令值，禁用 `<path>`/`<name>` 类尖括号占位符（`<` 在 shell 中是重定向符，遵守 SOTA 报告结论 10「example 永远可复制执行」）。
- 运行时错误（上述六类）退出码保持 **1**。
- validation 类错误的 fix 文案须包含 `--` 边界教学：正文以 `-` 开头时须置于 `--` 之后（如 `... alice -- "-fix flag text"`）。

### 4.3 usage 错误信封（新增，additive）

- `usage` 为第七类 category（category 词表为冻结枚举、仅可经评审流程扩展，本次扩展已经本次对抗评审确认）：**专指 clap 层用法错误**（缺必填位置参数、未知 flag、参数数量/类型错误等），与运行时六类分层——运行时六类 exit 1，usage exit **2**。
- main.rs 改用 `Cli::try_parse()`：clap 用法错误不再走 clap 默认输出，而是渲染为标准错误信封 + 新文法规范示例。
- **穿透条款**：`--help/-h`（各层级）与 `-V` 同样以 Err 返回（DisplayHelp/DisplayVersion），须按 clap 原语义调 `error.print()` 后 **exit 0**，**不进 usage 信封**（守住 §6.3 冻结条款）；仅其余用法错误 kind 进 usage 信封 exit 2。
- **example 生成策略（静态规范示例）**：usage 信封输出该命令的**规范 usage 行 + 一条预置可执行示例**（具体值、可直接复制执行）；**不携带用户原参数值、不做 argv 值迁移重建**（旧文法迁移教学由规范示例 + SKILL.md + after_help 共同承担）。
- **疑似 flag 残留的 `--` 教学（闭合复核 NF-2 补录）**：usage 错误涉及疑似 flag 的 argv 残留（如未知 `-xxx`/`--xxx`）时，fix 文案须提示 `--` 边界（正文值以 `-` 开头时须置于 `--` 之后），信封示例同步示范 `--` 用法形态（见 bdd.md S-SEND-14）。
- 信封结构与 §4.2 一致：

```
error usage: <clap 用法错误描述>
fix: <修正动作>
example: <该命令的规范可执行示例>
```

- `--json` 模式下 usage 错误同样输出单行 JSON 错误对象（含 `command` 字段），JSON 内 `exit_code` 字段**如实反映进程退出码**，即 usage 错误填 **2**（运行时错误仍为 1）。**--json 感知机制**：try_parse 失败时尚未取得解析结果，实现须回退扫描 `std::env::args()` 判定 `--json` 是否出现（argv 扫描兜底）。
- **顶层解析失败**：组/动词层失败导致无法确定命令时（如缺子命令），信封与 JSON 的 command 标识统一填 **`usage`**（如 `error usage: missing subcommand`，JSON `"command":"usage"`）。
- 旧式调用（如 `post send x.post.md --from alice "hi"`）中 `--from` 已成未知 flag，自动落入 usage 信封并获得该命令的规范可执行示例——错误即指导承担迁移教学职责。

### 4.4 退出码语义

| 退出码 | 语义 |
|---|---|
| 0 | 成功 |
| 1 | 运行时错误（六类：format / validation / io / not-found / already-exists / not-allowed） |
| 2 | 用法错误（第七类 usage 信封，新增；clap 层） |

### 4.5 三档输出（冻结不变）

- `--json`：单行 JSON；成功对象含 `status:"ok"`、`command`、`conclusion`、各字段、`body`；错误对象含 `status:"error"`、`category`、`message`、`fix`、`example`、`command`（新增 key）、`exit_code`（**如实反映进程退出码**：运行时错误 1，usage 错误 2），输出到 stdout。
- `--plain`：阅读类命令输出文件原始字节内容。
- `-q/--quiet`：隐藏 ok 状态首行。

### 4.6 JSON 只增不改不删条款

- JSON 输出的**既有 key 全部保持不变**（含字段名、值格式）；本次仅允许**新增** key：
  - `post send` 成功对象新增 `implicit-mention`（单数；仅当 reply-to 触发隐式 mention 时出现）；
  - `post read` 成功对象 `showing` 由「超限才出现」改为恒出现（既有出现场景值格式不变；**注意：这是既有 key 出现语义的变化而非纯 additive**，以「showing 缺席 == 未截断」做判断的消费者将受影响，须在 CHANGELOG `Changed (Breaking)` 明示），并新增 `window`（`#<first>-#<last>`，空线程不出现；`<total>` 口径见 §3.1 post read）；
  - 错误 JSON 对象新增 `command` 字段（标识出错命令，如 `post.send`；顶层解析失败填 `usage`）。
- 不得改名、不得改类型、不得删除任何既有 key。

---

## 5. 附带行为变更清单

| 变更 | 内容 | 追溯 | 性质 |
|---|---|---|---|
| ensure_suffix 语义 | 改为**三级解析**：① 传入路径原样存在且为**文件**（判据 `is_file()`，目录不命中）→ 用原路径；② 否则，补类型后缀后的路径存在 → 用补后缀路径；③ 都不存在 → **以补后缀路径为操作落点**（三级解析只决定路径；物理创建仅发生在写命令 send/create/add，只读命令三级均无文件时报 not-found）。与 v0.4 行为差异声明：v0.4 无条件改写路径，v0.5 第①级命中异型文件（非 paperwork 格式）时按对应类型解析器报 format 错误，不再自动改道补后缀路径 | U-14/N-02 | 行为变更（含「x.md 与 x.post.md 同时存在时用 x.md」语义） |
| post send 输出 | 增补 `implicit-mention` 单数字段（reply-to 隐式 mention 的原发送者，至多一人，仅触发时出现） | U-10 | additive |
| post read 输出 | 恒显 `showing: <n>/<total>` + 增补 `window: #<first>-#<last>`（空线程不显示 window） | U-11 | additive |
| post 别名 | 新增隐藏别名 `po`；`p/b/c/v` 不变 | U-08/N-01 | additive |
| 错误 JSON | 增补 `command` 字段 | 总方案 | additive |
| usage 信封 | clap 用法错误渲染为标准信封（第七类 category），exit 2 | 总方案（Tina 方案） | additive |
| validate --type | 新增可选类型覆盖 flag | U-15 | additive |

---

## 6. 输出协议冻结条款

1. `ok <command> <conclusion>` 信封结构、运行时六种 error category、错误信封 `error/fix/example` 三行结构：全部不变（category 词表为冻结枚举、仅可经评审流程扩展：本次经对抗评审确认扩展第七类 `usage`，扩展不属于对既有六类的变更）。
2. command 标识（`post.send`/`post.read`/`post.summary`/`post.create`/`post.edit`/`profile.create`/`profile.show`/`profile.edit`/`profile.list`/`brief.create`/`brief.add`/`brief.remove`/`brief.read`/`brief.verify`/`contacts.create`/`contacts.add`/`contacts.read`/`validate`）：全部不变。
3. 全局 flag `--json/--plain/-q/-V`：语义不变。
4. JSON 既有 key：只增不改不删（§4.6）。
5. **core API 零变更**：`paperwork-core` 公开函数签名、返回类型、错误类型不变；**文件格式零变更**：四类托管文件的 Markdown 结构、四反引号围栏、`---` 边界、bullet 元数据、`·` 分隔符全部不变。
6. 本次重设计的破坏面**仅限命令参数文法**（flag → 位置参数、`--from` 移除、usage 退出码引入），其余一切对外接口冻结。
