# Agent Paperwork CLI UX 调研报告：agent 看到什么、理解什么、输出样貌是什么

- 调研日期：2026-08-08
- 调研对象：c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork（v0.4.0）
- 任务：Task ID 1（纯调研，不修改任何源代码）

- 调研方法：
  1. 全量阅读 paperwork-core / paperwork-cli 源码与测试；
  2. 全量阅读 docs/ 下 ADR、dev-principles、reviews、pillars session-log；
  3. 运行已构建的 release binary（target\release\paperwork.exe，paperwork 0.4.0）对 test-v04/ 样例执行**只读命令**实测真实输出（未修改仓库任何文件）。
- 所有实测输出均标注产生命令；所有格式片段均标注来源文件路径。

---

## 1. 项目全貌

### 1.1 定位与核心主张

来源：README.md（根目录，第 3、14 行）

> Stateless, file-based collaboration primitives for AI agents. Identity, messaging, and knowledge briefs — all as plain Markdown files, operated by one CLI.

> No server. No database. No daemon. No login. No workspace. Every command takes an explicit file path and works from anywhere — the files are the source of truth.

README 将项目定义为给 AI agent 用的三类协作原语 + 一个目录原语（README.md 第 50-55 行表格）：

| 原语 | 命令 | 文件类型 | 用途 |
|------|------|----------|------|
| Identity | `profile` | `*.profile.md` | agent 的名字、模型、scope |
| Communication | `post` | `*.post.md` | append-only 线程，支持 reply + @mention |
| Knowledge | `brief` | `*.brief.md` | 阅读清单 + staleness 检测 |
| Directory | `contacts` | `*.contacts.md` | profile 路径列表 |
README 同时明确两条输出层主张（README.md 第 7-9 行附近）：所有 CLI 输出是结构化 envelope（信封），并可通过 `--json` 得到机器可读形态；错误输出不只报错，还附带 `fix:`（修正动作）与 `example:`（可直接复制执行的修正命令）。这是本项目 UX 的核心特征：**错误即指导**。

### 1.2 workspace 结构、版本与依赖

来源：根 Cargo.toml、repos/paperwork-core/Cargo.toml、repos/paperwork-cli/Cargo.toml

- 根 Cargo.toml 为虚拟 workspace：`[workspace] members = ["repos/paperwork-core", "repos/paperwork-cli"]`，resolver = "2"，workspace 级 `rust-version = "1.89"`。
- paperwork-core v0.4.0：纯逻辑层 crate，不依赖 clap。关键依赖：serde / serde_json（JSON 输出与 manifest）、sha2（brief hash）、regex（staleness 检测与解析）、fs2（文件锁，保证并发 append 安全）、thiserror（错误类型）。
- paperwork-cli v0.4.0：薄壳层 crate，依赖 paperwork-core（path 依赖）+ clap（derive 特性，命令树与 help 生成）。发布二进制名为 `paperwork`。
- 分工：**core = 全部领域逻辑与文件格式（format/ + ops/ + hash + error）**；**cli = 参数解析 + envelope 输出渲染（output.rs）**，CLI 层几乎不含业务判断。
- 小出入：README 写 "Rust 1.74+"，但 Cargo.toml 声明 rust-version = 1.89，实际构建以 1.89 为准。
- 根目录有 LICENSE（MIT）与 CHANGELOG.md，CHANGELOG 记录 v0.1 → v0.4 的完整演变（见第 2.6 节）。

### 1.3 测试体现的行为契约

来源：repos/paperwork-cli/tests/cli_integration.rs、repos/paperwork-core/tests/ops_tests.rs

**cli_integration.rs（CLI 层契约，assert 精确字符串）**：
- 成功首行格式被钉死：如 `ok post.send #3`、`ok post.read 3 messages`、`ok profile.create <name>`、`ok brief.verify 1/1 fresh`、`ok validate <path>` —— envelope 语法本身就是对外 API。
- 错误契约：stderr 首行以 `error <category>:` 开头（如 `error validation:`、`error not-found:`、`error not-allowed:`），且退出码 = 1；同时断言 fix:/example: 行存在。
- 文件名自动补后缀的契约：传 `thread.md` 会落到 `thread.post.md`（ensure_suffix 行为）。
- `--json` 契约：输出为单行 JSON，含 `status:"ok"` / `status:"error"`、`command`、`conclusion` 字段。
- append-only 契约：post send 只能追加；post edit 的三重限制（只能编辑自己的、自己最新的、线程最后一条）在违规时给出精确错误文本并返回 not-allowed。

**ops_tests.rs（core 层契约）**：
- 序列化/反序列化往返（round-trip）：serialize_message 输出再解析回等价 Message。
- 边界检测 fence-aware：正文内部的 `---` 不被误判为消息分隔符（四反引号围栏内安全）。
- hash/staleness 三态：Fresh / Shifted / Stale 的判定条件各有测试用例。
- 并发安全：文件锁下 seq 连续无间隙；单消息 64KB 上限触发 MessageTooLarge（归类 validation）。
- contacts/profile/brief 解析：损坏文件给出 Parse 错误而非 panic。

---

## 2. 设计文档与 UX 理念

### 2.1 起源：paperwork-init-conversation session-log（为谁设计）

来源：docs/ssot/pillars/paperwork-init-conversation/ 下两份 session-log

两份 session-log 记录了项目诞生对话：作者与 agent 共同确立「为多 agent 协作提供最小文书基础设施」的目标。关键结论：
- **第一用户是 agent，不是人**。人类可读是副产品；命令输出首先要让 agent 一眼判断成功与否、下一步做什么。
- 协作的最小单元被定为四种：身份（我是谁）、消息（谁对谁说了什么）、简报（我该读什么、是否过时）、名录（团队里有谁）。
- 明确拒绝引入服务器/数据库/登录等重型基础设施 —— agent 的使用环境千差万别，文件是唯一普适接口。

### 2.2 初版技术选型与 ADR-011（无状态路径显式架构）

来源：docs/ssot/adr/初版技术选型.md、docs/dev/adr-v1.md

技术选型要点：Rust（单二进制分发、跨平台、零运行时）、Markdown 作为存储格式（agent 天然可读可写）、CLI 作为唯一接口。

ADR-011 是最关键的 UX 架构决策（adr-v1.md 中被列为核心）：
- **无 .paperwork/ 目录、无 init、无 login、无当前 workspace 概念**。每条命令都接收显式文件路径，任何目录下都能操作任何位置的文件。
- SSOT（single source of truth）就是文件本身：没有影子状态、没有缓存索引，agent 直接读文件得到的信息与 CLI 认知永远一致。
- 对 agent 的意义：无环境依赖 → 无「忘记初始化」类错误；路径显式 → 意图无歧义，可审计、可复现。

### 2.3 agent-ux-qol：四个 agent 视角的关键问题

来源：docs/ssot/adr/agent-ux-qol.md

该 ADR 从「agent 实际调用时会遇到什么」出发提出 QoL 要求，可归纳为四问及其答案：
1. **agent 怎么知道命令成功了？** → 统一 `ok <command> <conclusion>` 首行 + 退出码 0。
2. **失败时 agent 怎么自救？** → 错误必须带 `fix:`（做什么能修复）和 `example:`（可直接执行的修正命令），而非仅报错。
3. **agent 怎么避免拼错文件名/格式？** → 类型化后缀（.post.md 等）自动补全；validate 命令提供格式体检。
4. **输出如何既让人能看又让机器能解析？** → 默认人类可读 envelope（固定行首关键字），`--json` 提供全机器可读形态，`--plain` 提供原文件字节流。

### 2.4 v0_feedbacks：真实使用反馈驱动的输出重设计

来源：docs/ssot/adr/feedbacks/v0_feedbacks.md

该文件收集了 v0.x 早期使用中 agent 的真实反馈，直接催生 v0.4 的输出重设计。核心抱怨：
- 旧版输出混排自然语言（如 "Successfully sent message..."），agent 需要正则猜测状态，脆弱且浪费 token。
- 错误信息只说「什么错了」不说「怎么改」，agent 只能盲目重试或查文档。
- 不同命令输出结构互不一致，无法建立统一解析模式。

反馈的处置原则被写入 v0.4 设计：**每行输出都要回答 agent 的一个问题**（状态？结论？字段？正文？错误？修复？），没有任何装饰性文本。

### 2.5 dev-principles：编排与流程原则

来源：docs/ssot/dev-principles/MainAgent工作编排.md、docs/ssot/dev-principles/实现流程原则.md

- MainAgent 工作编排：开发本身也由 agent 按「调研 → 提案 → 实现 → review」流水线执行，docs/ssot 是唯一事实来源，ADR 先于代码。
- 实现流程原则：小步迭代、每版有明确 review 节点（docs/reviews/ 即节点产物）、破坏性变更必须过 review 且写入 CHANGELOG。
- 对本调研的意义：CLI UX 的每一次变化都有文档留痕（reviews + CHANGELOG），可完整追溯演变。

### 2.6 UX 演变脉络（v0.1 → v0.4）

来源：CHANGELOG.md、docs/reviews/（v0.2-review、v0.3-review、v0.4-review、v0.4-ux-review）

| 版本 | 主题 | UX 关键变化 |
|------|------|-------------|
| v0.1 | 原型 | 有 workspace/init 概念，输出为自然语言 |
| v0.2 | 架构重写 | 取消 workspace 与状态目录，走向路径显式、无状态；review 确认去状态化方向 |
| v0.3 | 格式重设计 | 引入四类类型化后缀（.profile/.post/.brief/.contacts.md）、bullet 元数据、四反引号正文围栏；文件本身成为 agent 可直接读写的规范 Markdown |
| v0.4 | 输出重设计 | 引入 ASCII envelope（ok/error 协议）、fix:+example: 错误自愈、--json/--plain/-q 三档输出；review 逐项核验每条命令输出 |

v0.4-ux-review 进一步提出 13 项后续改进提议（详见第 7 章实现状态表）。

### 2.7 核心交互哲学提炼

1. **文件即接口（files as API）**：没有隐藏状态，agent 可以绕过 CLI 直接读写文件（格式即协议），也可以用 CLI 获得格式保证 —— 两条路等价。
2. **append-only 通信**：post 线程只追加不删除，历史不可篡改；编辑被三重护栏压缩到「自己刚发的最后一条」，保证协作可追溯。
3. **显式优于隐式**：--from/--to 显式声明身份与受众，路径显式传入，无环境变量、无默认 workspace。
4. **错误即指导**：每个错误都携带修复路径（fix）与可执行示例（example），把 agent 的试错成本降到最低。
5. **一次学会、处处复用**：所有命令共享同一 envelope 语法，agent 只需学会一种解析模式。

---

## 3. CLI 命令层 UX

### 3.1 命令树与全局 flag

来源：repos/paperwork-cli/src/main.rs（clap derive 定义）+ 实测 `paperwork --help`

顶层 help 首行即项目定位：`Stateless, file-based collaboration CLI for AI agents — everything is a file, append-only, human-readable`。

```
paperwork [OPTIONS] <COMMAND>
├── profile   管理 agent 身份文件      create / show / edit / list
├── post      线程（群聊，也覆盖 1:1） create / send / read / summary / edit
├── brief     阅读清单 / 知识简报      create / add / remove / read / verify
├── contacts  名录                     create / add / read
└── validate  校验任意文件的 Markdown 结构
```

全局 flag（贯穿所有层级）：
- `--json`：全部输出转为单行 JSON；
- `--plain`：阅读类命令输出文件原始字节内容；
- `-q/--quiet`：只隐藏 `ok ...` 状态首行，字段与正文保留；
- `-V/--version` 输出版本号。

### 3.2 各子命令用法与行为语义（实测 help 文本）

**profile**（repos/paperwork-cli/src/cmd/profile.rs）
- `profile create <PATH> --name <NAME> --model <MODEL> [--description ...] [--scope-read/write/owns glob]`：生成 .profile.md；路径经 ensure_suffix 强制为 .profile.md。
- `profile show <PATH>`：结构化展示 name/model/description/scope.read/scope.write/scope.owns。
- `profile edit <PATH> [--name/--model/--description/--scope-*]`：局部更新，未给字段保留原值。
- `profile list <DIR>`：扫描目录下所有 .profile.md，逐行 `<文件名>: <name> (<model>)`；解析失败的文件显示 `<文件名>: (unreadable)` 而不中断整体列表 —— 容错式列表是面向 agent 的刻意设计。

**post**（repos/paperwork-cli/src/cmd/post.rs）
- `post create <PATH> --title <TITLE> [--participants a,b,c]`：创建线程，自动写入 #1 系统消息 `[Thread created: <title> | participants: ...]`。
- `post send <PATH> --from <NAME> [BODY] [--stdin] [--reply-to N] [--mention a,b]`：追加消息；线程不存在时**自动创建**（隐式创建降低冷启动摩擦）；同时给位置参数与 --stdin 报 validation 错误，二者都不给也报错；空正文（trim 后为空）被拒绝。注意：send 无 `--to` 收件人参数，消息 To 恒为 all；定向语义通过 --mention/--reply-to 表达。
- `post read <PATH> [--from N] [--to M] [--mention name] [--reply-to N] [--limit 20]`：过滤管道为 seq 范围 → mention/reply-to 过滤 → limit 取最后 N 条（默认 20）；此处 --from/--to 是**序列号范围**而非身份（与 send 的 --from=发送者语义不同，是已知的命名冲突，见第 7 章）。
- `post summary <PATH>`：输出 title/participants/messages/last.sender/last.time/last.snippet，供 agent 快速判断线程是否值得细读。
- `post edit <PATH> --seq N --from <NAME> [NEW_BODY] [--stdin]`：受三重护栏约束（只能编辑：①自己发的 ②自己最新发的 ③且是线程最后一条），否则 not-allowed。

**brief**（repos/paperwork-cli/src/cmd/brief.rs）
- `brief create <PATH> --title T --owner NAME [--description ...]`；`brief add <PATH> --entry P [--regex R] [--note N]`（add 时自动计算目标文件 SHA-256 快照）；`brief remove <PATH> --entry-title T`；
  > **勘误（2026-08-09，v0.5 文档集 rework 轮，agent-ux 评审 N7）**：本节早期版本误记 brief add 为 `--title/--path`，与源码 cmd/brief.rs 不符，实际为 `--entry`（路径）与 remove 的 `--entry-title`（标题）。
- `brief read <PATH> [--full]`：默认只列 `<title>: <path>`；--full 追加 `(hash: 前12位) regex: ... note: ...`。
- `brief verify <PATH>`：对每个条目按 regex + hash 三态判定 fresh/shifted/stale（见 5.6）。

**contacts**（repos/paperwork-cli/src/cmd/contacts.rs）
- `contacts create <PATH> --title T`；`contacts add <PATH> <profile路径>`；
- `contacts read <PATH>`：列出条目并**富化**为 `<绝对路径>: <name> (<description>)` —— 读取名录时顺手给出对方身份摘要，agent 一次调用即可完成「团队有谁 + 各自是谁」。

**validate**（repos/paperwork-cli/src/cmd/validate.rs）
- `validate <PATH>`：按后缀推断文件类型并做格式体检；成功输出 `ok validate <path>`，失败输出 error 信封（exit 1）。是 agent 的「格式防火墙」。

### 3.3 ensure_suffix：类型化后缀的自动治理

来源：repos/paperwork-cli/src/cmd/mod.rs

规则：文件名已带本类型后缀 → 原样；以裸 `.md` 结尾 → 替换为类型后缀（thread.md → thread.post.md）；无 .md → 直接追加。意义：agent 无需记忆四类后缀规则即可得到正确文件名；文件类型可由后缀自描述，validate 也据此路由。

### 3.4 output.rs：统一格式化机制（envelope 引擎）

来源：repos/paperwork-cli/src/output.rs（源码注释原文）

```
// Success envelope (stdout):
//   ok <command> <conclusion>
//   <key>: <value>
//   ---
//   <body lines>
// Error envelope (stderr):
//   error <category>: <message>
//   fix: <corrective action>
//   example: <corrected command>
```

实现要点：
- `emit_ok`：Default 模式按上述顺序打印（-q 只隐藏首行）；Json 模式输出单行对象 `{"status":"ok","command":...,"conclusion":..., 各字段..., "body":[...]}`。
- `emit_err`：Default/plain 模式错误写 **stderr** 且退出码 1；Json 模式错误写 **stdout**（便于程序化捕获）且 JSON 内含 `"exit_code":1`，进程退出码同样为 1。
- 错误分类由 core 的 `PaperworkError::category()` 决定：Parse→format、Validation/MessageTooLarge→validation、Io/IoContext→io、NotFound→not-found、AlreadyExists→already-exists、NotAllowed→not-allowed（来源：repos/paperwork-core/src/error.rs）。分类词是稳定的机器可读接口，message 是人类可读细节，fix/example 是自愈指导。

---

## 4. agent 实际看到的输出（全部为 release binary v0.4.0 对 test-v04/ 的实测结果）

### 4.1 post read：阅读线程的呈现样貌

命令：`paperwork post read test-v04\standup.post.md`（stdout，exit 0）

```
ok post.read 6 messages
---
#1 system 2026-08-01T19:38:03Z
  [Thread created: Daily Standup | participants: alice, bob, charlie]
#2 alice 2026-08-01T19:38:22Z
  Parser module is 80% done.
#3 bob 2026-08-01T19:38:22Z reply:#2 mentions:alice
  Tests merged, all green.
#4 charlie 2026-08-01T19:38:22Z mentions:alice
  @alice nice work!
#5 alice 2026-08-01T19:39:52Z
  Updated: multi-line body edited
#6 bob 2026-08-01T19:41:08Z
  quiet message
```

呈现规则（来源：repos/paperwork-cli/src/cmd/post.rs）：
- 每条消息 = 一行头行 `#<seq> <sender> <timestamp>`，可选追加 ` reply:#N`、` mentions:a,b`（仅在存在时出现，避免噪声）；
- 正文每行前置 2 空格缩进，多行正文逐行缩进；
- 头行与正文均不含装饰符，纯 ASCII，agent 可用极简正则逐条切分。

限量时的附加字段：当 total > limit 时多一行 `showing: <n>/<total>`（字段区，位于 `---` 之前），并只展示最后 N 条：

命令：`paperwork post read test-v04\standup.post.md --limit 2`

```
ok post.read 6 messages
showing: 2/6
---
#5 alice 2026-08-01T19:39:52Z
  Updated: multi-line body edited
#6 bob 2026-08-01T19:41:08Z
  quiet message
```

过滤示例（实测）：`--mention alice` 返回 #3、#4 两条；`--reply-to 2` 只返回 #3。

### 4.2 post summary：线程速览

命令：`paperwork post summary test-v04\standup.post.md`（stdout，exit 0）

```
ok post.summary c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\test-v04\standup.post.md
title: Daily Standup
participants: alice, bob, charlie
messages: 6
last.sender: bob
last.time: 2026-08-01T19:41:08Z
last.snippet: quiet message
```

title/participants 从 #1 系统消息 `[Thread created: <title> | participants: ...]` 提取（strip_prefix + split " |" + trim_end_matches ']'，来源：cmd/post.rs）—— v0.4-review 的 ISSUE-1（title 缺失）已修复并在本实测中生效。

### 4.3 profile show / profile list：身份呈现

命令：`paperwork profile show test-v04\alice.profile.md`（stdout，exit 0）

```
ok profile.show alice
name: alice
model: gpt-4o
description: Parser module implementer
scope.read: src/**
scope.write: src/parser/**
scope.owns: src/parser/**
```

命令：`paperwork profile list test-v04`（stdout，exit 0）

```
ok profile.list 3 profiles
---
alice.profile.md: alice (gpt-4o)
bob.profile.md: bob (claude-sonnet)
garbage.profile.md: (unreadable)
```

### 4.4 contacts read：名录富化呈现

命令：`paperwork contacts read test-v04\team.contacts.md`（stdout，exit 0）

```
ok contacts.read 2 contacts
---
c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\test-v04\alice.profile.md: alice (Parser module implementer)
c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\test-v04\bob.profile.md: bob (Integration test writer)
```

### 4.5 brief read / brief verify：简报呈现与 staleness 报告

命令：`paperwork brief read test-v04\onboarding.brief.md`（stdout，exit 0）

```
ok brief.read 2 entries
title: Codebase Onboarding
owner: alice
---
main.rs: src/main.rs
lib.rs: src/lib.rs
```

命令：`paperwork brief read ... --full` 时 body 行变为（hash 截断为前 12 位十六进制）：

```
main.rs: src/main.rs (hash: 42b664743ddb) regex: fn main note: Entry point
lib.rs: src/lib.rs (hash: 73e4ebd8338a) regex: pub mod \w+ note: Modules
```

命令：`paperwork brief verify test-v04\onboarding.brief.md`（stdout，exit 0）

```
ok brief.verify 1/2 fresh
---
main.rs: stale
lib.rs: fresh
```

（本例中 test-v04\src\main.rs 内容为 `pub fn run() {}`，regex `fn main` 不匹配 → stale；src/lib.rs 未变 → fresh。conclusion `1/2 fresh` 让 agent 首行即知整体健康度，逐行结果指明哪条需要重读。）

### 4.6 validate：格式体检报告

成功（exit 0）：`paperwork validate test-v04\standup.post.md` → `ok validate <完整路径>`（无额外字段，简洁即结论）。

失败（stderr，exit 1），实测两例：

```
error format: Parse error: ...\test-v04\garbage.post.md is not a valid .post.md file: no valid message boundaries found
fix: expected --- separators with ### #N sender . timestamp headers and ````markdown fenced bodies
example: paperwork post send myfile --from alice "hello"
```

```
error format: Parse error: ...\test-v04\garbage.profile.md is not a valid .profile.md file: missing agent name heading (# <name>)
fix: add a top-level heading with the agent name
example: # alice
```

agent 从错误信封中可以同时获得：出了什么问题（message）、该做什么（fix）、正确的东西长什么样（example）。

### 4.7 错误信封与退出码：更多实测

not-found（stderr，exit 1；注意路径已被 ensure_suffix 补全为 no-such.post.md，错误中显示的是补后缀后的路径）：

```
error not-found: Thread '...\test-v04\no-such.post.md' not found
fix: send a message first to create the thread
example: paperwork post send ...\test-v04\no-such.post.md --from <name> <body>
```

其余分类（来源：error.rs category() 与 cli_integration.rs 断言）：validation（参数冲突/空正文/超 64KB）、not-allowed（edit 三重护栏，错误文本精确如 `Message #3 was sent by 'bob', not 'alice'`、`not your most recent message`、`not the final message in thread`）、already-exists（重复 create）、io（文件读写失败）。

### 4.8 三档输出变体对照（同一数据的三种样貌）

**--json**（stdout 单行 JSON，exit 0），`post read ... --json --limit 2`：

```json
{"command":"post.read","conclusion":"6 messages","messages":[{"body":"Updated: multi-line body edited","mentions":[],"reply_to":null,"sender":"alice","seq":5,"timestamp":"2026-08-01T19:39:52Z","to":[]},{"body":"quiet message","mentions":[],"reply_to":null,"sender":"bob","seq":6,"timestamp":"2026-08-01T19:41:08Z","to":[]}],"showing":"2/6","status":"ok"}
```

**--json 错误形态**（输出到 stdout，含 exit_code，进程仍 exit 1），`post read 不存在文件 --json`：

```json
{"category":"not-found","example":"paperwork post send ... --from <name> <body>","exit_code":1,"fix":"send a message first to create the thread","message":"Thread '...' not found","status":"error"}
```

**-q**（隐状态首行，保留字段与正文），`post read ... -q --limit 2`：

```
showing: 2/6
---
#5 alice 2026-08-01T19:39:52Z
  Updated: multi-line body edited
#6 bob 2026-08-01T19:41:08Z
  quiet message
```

**--plain**（文件字节原样），`post read ... --plain --from 2 --to 3`：

````
---

### #2 alice · 2026-08-01T19:38:22Z

- To: all

````markdown
Parser module is 80% done.
````

---

### #3 bob · 2026-08-01T19:38:22Z

- To: all
- Reply-To: #2
- Mentions: alice

````markdown
Tests merged, all green.
````
````

设计意图：default 给人看、--json 给程序解析、--plain 给「想看文件本体」的 agent —— 三档覆盖全部消费场景，且互相正交。

---

## 5. 文件格式规范（agent 直接读写的文件长什么样）

格式实现来源：repos/paperwork-core/src/format/（thread.rs、profile.rs、contacts.rs、manifest.rs、mod.rs）与 hash.rs；真实样例来源：test-v04/。

### 5.1 .post.md：append-only 线程文件

结构约定（format/thread.rs serialize_message + format/mod.rs 解析正则）：
- 每条消息以单独一行 `---` 开始（YAML 多文档分隔符式的视觉锚点）；
- 头行：`### #<seq> <sender> · <timestamp>` —— 注意分隔符是 Unicode 中点 `·`（U+00B7），时间格式 `%Y-%m-%dT%H:%M:%SZ`；解析正则 `^### #(\d+) (.+) · (.+)$`（MESSAGE_HEADER_RE）；
- bullet 元数据：`- To: a, b`（空收件人列表序列化为 `all`）、可选 `- Reply-To: #N`、可选 `- Mentions: a, b`；解析正则 `^- ([^:]+):\s*(.*)$`（BULLET_KEY_RE）；
- 正文用**四反引号**围栏 ```markdown ... ```` —— 四反引号是为了让正文内部的三反引号代码块不破坏围栏；
- find_message_boundaries 是 fence-aware 的：围栏内部的 `---` 永远不会被当作消息边界；边界判定要求 `---` 之后 2 行内出现合法 H3 头行；
- 正文内需要水平线时约定使用 `***`（避开 `---` 的边界语义）。

真实正常样例（test-v04/standup.post.md，节选 #1 与 #3）：

````
---

### #1 system · 2026-08-01T19:38:03Z

- To: all

````markdown
[Thread created: Daily Standup | participants: alice, bob, charlie]
````

---

### #3 bob · 2026-08-01T19:38:22Z

- To: all
- Reply-To: #2
- Mentions: alice

````markdown
Tests merged, all green.
````
````

自动创建样例（test-v04/auto-thread.post.md）：对不存在的文件直接 send 时生成，仅含 #1 消息，证明「send 即创建」：

````
---

### #1 alice · 2026-08-01T19:41:58Z

- To: all

````markdown
Auto-created!
````
````

### 5.2 .profile.md：agent 身份文件

结构约定（format/profile.rs）：`# <name>` 顶级标题（必需，缺失即 Parse 错误）；`- Model:`、`- Description:` bullet；`## Scope` 段下 `- Read:`/`- Write:`/`- Owns:` bullet，glob 用反引号包裹。

真实样例（test-v04/alice.profile.md，全文）：

```
# alice

- Model: gpt-4o
- Description: Parser module implementer

## Scope

- Read: `src/**`
- Write: `src/parser/**`
- Owns: `src/parser/**`
```

### 5.3 .brief.md：阅读清单文件

结构约定（format/manifest.rs + ops/manifest.rs）：`# <title>` 标题；`- Owner:`、`- Created:`、`- Description:` bullet；`## Entries` 段下每个条目为 `### <title>` + `- Path:`（反引号包裹）+ `- Hash:`（完整 64 位 SHA-256 小写十六进制）+ 可选 `- Regex:`，条目说明用 `> note` 引用块。

真实样例（test-v04/onboarding.brief.md，全文）：

```
# Codebase Onboarding

- Owner: alice
- Created: 2026-08-01T19:40:36Z
- Description: How to understand this project

## Entries

### main.rs

- Path: `src/main.rs`
- Hash: `42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540`
- Regex: `fn main`

> Entry point

### lib.rs

- Path: `src/lib.rs`
- Hash: `73e4ebd8338a6b237288450ec6ab80d9f2e3834e9af3946737cad8b41f8195b0`
- Regex: `pub mod \w+`

> Modules
```

### 5.4 .contacts.md：名录文件

结构约定（format/contacts.rs）：`# <title>` 标题 + 每行一个 `- <profile路径>` bullet。解析极简，一行一联系人。

真实样例（test-v04/team.contacts.md，全文）：

```
# Core Team

- c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\test-v04\alice.profile.md
- c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\test-v04\bob.profile.md
```

### 5.5 损坏样例：格式防火墙的靶子

test-v04/garbage.post.md 全文仅一行 `this is not valid` → validate 报 `no valid message boundaries found`；
test-v04/garbage.profile.md 全文仅一行 `random text no structure` → validate 报 `missing agent name heading (# <name>)`。
这两个样例是刻意保留的教学样本：agent 可以用它们观察错误信封的完整形态。

### 5.6 hash.rs 与 staleness 机制：agent 如何感知「知识过期」

来源：repos/paperwork-core/src/hash.rs、ops/manifest.rs verify_entry

- brief add 时对目标文件内容计算 SHA-256（小写 hex）快照存入 .brief.md；
- brief verify 对每个条目做三态判定：
  - 目标文件读不到 → **stale**；
  - regex 不匹配或 regex 非法 → **stale**（结构已面目全非）；
  - regex 匹配且 hash 与快照一致 → **fresh**（一字未改）；
  - regex 匹配但 hash 变了 → **shifted**（关键结构还在，内容已更新，需重读但非从零）。
- agent 感知路径：`brief verify` 的 conclusion（`N/M fresh`）+ 逐行状态，决定哪些文件要重新阅读；这是「agent 的知识库需要保鲜」这一问题的机械化答案。
- CLI 输出层只展示 hash 前 12 位（--full），文件内保存全量 64 位 —— 展示紧凑、存储精确的分层取舍。

---

## 6. agent 视角的理解模型

### 6.1 完整使用旅程（按命令与可见反馈逐步还原）

**第 1 步：注册身份。** `paperwork profile create ./alice.md --name alice --model gpt-4o ...` → 首行 `ok profile.create alice` 即确认成功；ensure_suffix 保证文件落为 alice.profile.md。agent 学到：我的身份 = 一个带类型后缀的 Markdown 文件。

**第 2 步：建立名录。** `contacts create` + `contacts add` 把队友 profile 路径登记进 team.contacts.md；之后 `contacts read` 一次拿到「路径 + 名字 + 职责描述」的富化列表。agent 学到：发现队友靠名录文件，而非任何注册中心。

**第 3 步：收发线程。** `post send thread.md --from alice "..."`（自动补后缀、自动建线程，首行回 `ok post.send #N` 告知序列号）；`post read` 头行格式 `#N sender timestamp [reply:#K] [mentions:...]` + 缩进正文，agent 可用同一正则解析一切线程；`post summary` 用于决定是否细读。agent 需理解的约定：
- --from 必填；send/edit 均**不**校验其与 profile 名的一致性（实测 cmd/post.rs：无存在性/一致性校验）；
- reply-to 指向序列号；--reply-to 会隐式把原消息发送者并入 mentions（实现于 cmd/post.rs，不在输出中显式提示）；
- @mention 目前只是文本约定 + 结构化 Mentions 字段，可被 `read --mention` 过滤，无通知机制；
- 序列号由文件锁保证连续无间隙，可安全用作增量读取游标（`read --from N+1`）；
- append-only：不能撤回，edit 仅限「自己的、最新的、且是线程末尾」那条。

**第 4 步：阅读简报。** `brief read` 拿清单，`brief verify` 拿 fresh/shifted/stale 三态报告，只重读 stale/shifted 的目标文件。agent 学到：知识获取是「清单 + 保鲜检查」而非全量重读。

**第 5 步：格式自卫。** 任何怀疑文件损坏的时刻运行 `validate`：exit 0 即通过；失败则错误信封直接给出修复示例。agent 学到：格式正确性可机器验证，且验证命令本身就是文档。

### 6.2 agent 必须理解的约定清单（汇总）

| 约定 | 内容 | agent 如何获知 |
|------|------|----------------|
| 类型化后缀 | .profile/.post/.brief/.contacts.md | ensure_suffix 自动补全 + validate 提示 |
| 显式身份 | --from 必填；send/edit 不校验与 profile 一致性（勘误：本节早期版本误记「均校验」，实测 cmd/post.rs 无此校验，2026-08-09） | validation 错误信封 |
| 序列号 | 连续无间隙，是增量读取与 reply 的锚点 | post send 回执 `#N` |
| reply-to | 结构化字段，头行显示 `reply:#N` | post read 输出 |
| mention | 结构化字段 + 正文内 @文本双轨；read 可过滤 | post read 头行 `mentions:...` |
| append-only | 只追加；edit 三重护栏 | not-allowed 错误文本 |
| 文件内 `·` | 头行 sender 与时间用 U+00B7 分隔（文件层） | --plain / 直接读文件 |
| 四反引号围栏 | 正文边界，正文内三反引号安全 | validate 的 fix 提示 |
| 正文水平线 | 用 `***` 而非 `---` | 格式约定（避免边界歧义） |
| 三态保鲜 | fresh/shifted/stale | brief verify 输出 |
| 无状态 | 无 init/login，路径显式 | help 首行 + 无环境依赖的事实 |

### 6.3 机器可读 vs 人类可读的取舍评估

- **默认档定位「人机双读」**：envelope 每行都有固定行首语法（ok/error/key: value/fix:/example:），人眼可读，agent 用极简正则即可解析；实测所有输出均为纯 ASCII（文件层才使用 · 与 — 等 Unicode），降低编码风险。
- **--json 档完全机器化**：单行、字段稳定、错误也走 JSON（含 exit_code 字段），适合 wrapper 程序；代价是正文被压进数组，直接给人看体验差。
- **conclusion 短语是「一句话结论」设计**：如 `6 messages`、`1/2 fresh`、`2 contacts`，agent 在拿到首行的瞬间即可决策，无需解析 body —— 这是对 token 经济学与决策延迟的双重优化。
- **错误信息分层**：category（机器路由）/ message（人读细节）/ fix+example（行动指令）三层并存，是三档输出之外又一处「一次输出服务多种消费者」的设计。
- 不足之处：--json 的 messages 结构与默认档信息一致但字段命名（reply_to vs reply:#N）需分别记忆；not-allowed 的具体违规信息只在 message 自然语言中，无结构化子分类。

---

## 7. 已知问题与未来提议（review 遗留项实现状态）

来源：docs/reviews/v0.4-review.md、docs/reviews/v0.4-ux-review.md + 本次实测核验

**v0.4-review 的两个 ISSUE（均已修复，实测确认）**：
- ISSUE-1：post summary 缺 title → 现从 #1 系统消息提取，实测输出 `title: Daily Standup`；
- ISSUE-2：brief read --full 详情不足 → 现输出 hash（12 位）+ regex + note，实测确认。

**v0.4-ux-review 的 13 项提议（抽样核验状态）**：

| 提议 | 状态 | 核验证据 |
|------|------|----------|
| 顶层命令别名（p/b/c/v） | 已实现 | 实测 `paperwork p show ...`、`paperwork v ...` 均生效（别名未出现在 --help，属隐藏别名） |
| summary title/participants 提取 | 已实现 | 实测 4.2 |
| brief read --full 展示 hash/regex/note | 已实现 | 实测 4.5 |
| post read --from/--to 语义与 send --from 冲突（范围 vs 发送者） | 未解决 | help 文本中两处语义并存（实测 3.2） |
| 隐式 mention（reply-to 自动带上原发送者）在输出中显式化 | 未实现 | cmd/post.rs 逻辑存在但无输出提示 |
| 从正文自动提取 @mention | 未实现 | 正文 `@alice` 不会进入 Mentions 字段（样例 #4 的 mentions 需显式 --mention） |
| 其余提议（help 文案、更多错误示例、JSON 字段扩展等） | 见 ux-review 原文 | 未逐项核验 |

## 8. 证据文件清单

- 根文档：仓库根目录的 README、CHANGELOG、Cargo.toml、LICENSE，以及 repos/paperwork-cli/README.md
- 设计文档：docs/ssot/adr/初版技术选型.md、docs/ssot/adr/agent-ux-qol.md、docs/ssot/adr/feedbacks/v0_feedbacks.md、docs/dev/adr-v1.md、docs/ssot/dev-principles/MainAgent工作编排.md、docs/ssot/dev-principles/实现流程原则.md
- Review 文档：docs/reviews/ 下 v0.2-review、v0.3-review、v0.4-review、v0.4-ux-review 四份
- 起源对话：docs/ssot/pillars/paperwork-init-conversation/（两份 session-log）
- CLI 源码：repos/paperwork-cli/src/main.rs、output.rs、cmd/mod.rs、cmd/post.rs、cmd/profile.rs、cmd/brief.rs、cmd/contacts.rs、cmd/validate.rs
- core 源码：repos/paperwork-core/src/error.rs、lib.rs、hash.rs、format/mod.rs、format/thread.rs、format/profile.rs、format/contacts.rs、format/manifest.rs
- core 源码（续）：repos/paperwork-core/src/ops/ 下 thread.rs、manifest.rs、profile.rs、contacts.rs
- 测试契约：repos/paperwork-cli/tests/cli_integration.rs、repos/paperwork-core/tests/ops_tests.rs
- 真实样例：test-v04/ 下 alice.profile.md、bob.profile.md、standup.post.md、auto-thread.post.md、team.contacts.md、onboarding.brief.md、两份损坏样例（garbage.post.md、garbage.profile.md）与 src/ 目录
- 实测工具：target/release 下已构建的 paperwork 二进制（0.4.0），全部为只读命令

## 9. 结论（一段话版）

Agent Paperwork 把「多 agent 协作的文书工作」压缩为四类纯 Markdown 文件 + 一个无状态 CLI；其 UX 的第一用户是 agent：统一的 ASCII envelope 输出协议让首行即结论（ok/error + 一句话结论），错误自带修复动作与可执行示例实现「错误即指导」，--json/--plain/-q 三档输出覆盖机器解析、文件本体、静默脚本三类消费场景；文件层采用类型化后缀、bullet 元数据与四反引号围栏的规范 Markdown，使「文件即接口」成立 —— agent 既可以完全通过 CLI 操作，也可以直接读写文件；append-only + 序列号 + 三重编辑护栏 + SHA-256/regex 三态保鲜机制，为 agent 间的可追溯通信与知识保鲜提供了机械化保证。

---
（报告完）
