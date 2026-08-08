# CLI UX 重设计 v0.5.0 — BDD（行为场景）

- 日期：2026-08-09
- 版本：v0.5.0
- 文档性质：行为规范的行为化表述（Given/When/Then），覆盖全部命令的正常路径与错误路径
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.5_feedbacks.md`（owner 指令）
  - `docs/dev/adr-v1.md`（ADR-011）、`docs/ssot/adr/feedbacks/v0_feedbacks.md`
  - `docs/researches/cli-ux-agent-visible-output-research-2026-08-08.md`
  - `docs/researches/ux-open-items-backlog-2026-08-08.md`
- 契约基准：本目录 `spec.md`（签名、信封、退出码语义均以 spec 为准）

约定：`<exit N>` 表示进程退出码；「信封」指 spec §4 定义的 envelope；示例中的路径均为相对示例，实际行为与 cwd 无关（无状态）。

---

## 1. post send

### S-SEND-01 新文法成功发送

- **Given** 存在线程文件 `standup.post.md`
- **When** 执行 `paperwork post send standup.post.md alice "Parser done"`
- **Then** exit 0；stdout 首行 `ok post.send #N -> <path>`；字段区含 `seq`、`path`、`sender: alice`；文件追加一条 alice 的消息。

### S-SEND-02 线程不存在时自动创建

- **Given** `quick-chat.post.md` 不存在
- **When** 执行 `paperwork post send quick-chat alice "Hey"`
- **Then** exit 0；`quick-chat.post.md` 被创建且首条消息即该消息（#1）。

### S-SEND-03 reply-to 隐式 mention 显式化（U-10）

- **Given** 线程中 #2 为 alice 所发
- **When** 执行 `paperwork post send standup.post.md bob "Sure" --reply-to 2`
- **Then** exit 0；字段区含 `implicit-mention: alice`（单数字段，仅触发时出现）；文件内该消息 Mentions 含 alice；`--json` 模式 JSON 含 `implicit-mention` key（与默认档一致）；未触发 reply-to 隐式 mention 的 send 输出不含该字段。

### S-SEND-04 --stdin 与位置 body 互斥

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md alice "body" --stdin`
- **Then** exit 1；stderr 首行 `error validation:`；message 说明两者不可同时给出；example 为新文法命令。

### S-SEND-05 缺 body 且无 --stdin

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md alice`
- **Then** exit 1；stderr 首行 `error validation:`；example 形如 `paperwork post send standup.post.md alice "Hello"`。

### S-SEND-06 空正文被拒绝

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md alice "   "`
- **Then** exit 1；stderr 首行 `error validation:`（行为同 v0.4，文法为新）。

### S-SEND-07 body 以 `-` 开头（`--` 边界）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md alice -- "-fix flag text"`
- **Then** exit 0；正文逐字为 `-fix flag text`，不被 clap 当作 flag 解析。

### S-SEND-08 缺必填位置参数（仅 PATH，usage 错误）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md`（仅 PATH，NAME 必填位缺失）
- **Then** exit 2；stderr 首行 `error usage:`；example 为该命令的规范可执行示例（含 PATH/NAME/BODY 完整形态，具体值）；不产生任何文件写入。

### S-SEND-12 NAME/BODY 混淆面：PATH+单字符串→validation（F1 裁定）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md "body text"`（单字符串必然绑定 NAME 槽、BODY 缺省）
- **Then** exit 1；stderr 首行 `error validation:`（无正文）；message 提示若已给出正文请检查是否遗漏 NAME 槽位；example 为含 NAME 槽的完整命令形态（具体值，如 `paperwork post send standup.post.md alice "body text"`）；fix 含 `--` 边界用法；不产生文件写入。

### S-SEND-09 旧文法调用落入 usage 信封（迁移教学）

- **Given** 存在线程文件 `x.post.md`
- **When** 执行 `paperwork post send x.post.md --from alice "hi"`
- **Then** exit 2；stderr 首行 `error usage:`（`--from` 为未知 flag）；example 为 post send 的规范形态示例（具体可执行值，不携带用户原参数值，见 design §2.6）；无文件写入。

### S-SEND-10 并发 send seq 无间隙

- **Given** 存在线程文件
- **When** 两个进程并发执行新文法 send
- **Then** 两条消息 seq 连续无间隙（文件锁行为不变）。

### S-SEND-13 多余位置参数（usage）

- **When** 执行 `paperwork post send standup.post.md alice "b" "extra"`（四个位置值）
- **Then** exit 2；`error usage:`（unexpected argument）；example 为 post send 规范形态示例。

### S-SEND-14 body 以 `-` 开头未加 `--`（usage 负形态，闭合复核 NF-2 补录）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md alice -fix`（意图发送 `-fix` 开头的正文，未加 `--`）
- **Then** exit 2；`error usage:`（clap 把 `-fix` 解析为未知 flag）；fix 文案提示 `--` 边界（正文以 `-` 开头时须置于 `--` 之后）；example 示范 `--` 用法形态 `paperwork post send standup.post.md alice -- "-fix flag text"`（预置静态示例，不携带用户原参数值）。

### S-SEND-10b implicit-mention 不触发边界：自回复

- **Given** 线程中 #2 为 alice 所发
- **When** 执行 `paperwork post send standup.post.md alice "again" --reply-to 2`
- **Then** exit 0；字段区**不**含 `implicit-mention`（原发送者即本人）；文件 Mentions 不新增 alice。

### S-SEND-11 implicit-mention 不触发边界：已显式 mention / reply-to 不存在

- **Given** 线程中 #2 为 alice 所发、线程共 3 条
- **When** 分别执行 `paperwork post send standup.post.md bob "x" --reply-to 2 --mention alice` 与 `paperwork post send standup.post.md bob "y" --reply-to 99`
- **Then** 两次均 exit 0；字段区均**不**含 `implicit-mention`（前者已显式 mention，后者 seq 不存在静默跳过）；行为沿用 v0.4。

## 2. post edit

### S-EDIT-01 新文法成功编辑

- **Given** 线程中 #3 为 bob 所发且是 bob 的最新消息、也是线程最后一条
- **When** 执行 `paperwork post edit standup.post.md bob 3 "edited"`
- **Then** exit 0；stdout 首行 `ok post.edit #3`；#3 正文变为 edited。

### S-EDIT-02 三重护栏（not-allowed）

- **Given** #3 为 bob 所发
- **When** 执行 `paperwork post edit standup.post.md alice 3 "x"`
- **Then** exit 1；stderr `error not-allowed:`，message 精确指出 `sent by 'bob', not 'alice'`；example 为新文法具体可执行命令（如 `paperwork post edit standup.post.md bob 3 "x"`）。

### S-EDIT-03 SEQ 非数字（usage 错误）

- **Given** 存在线程文件
- **When** 执行 `paperwork post edit standup.post.md bob abc "x"`
- **Then** exit 2；`error usage:`（u64 解析失败）；example 给出合法 SEQ 形态。

### S-EDIT-04 旧文法调用

- **When** 执行 `paperwork post edit standup.post.md --seq 2 --from bob "edited"`
- **Then** exit 2；`error usage:`；example 为 post edit 的规范形态示例（具体可执行值，不携带用户原参数值，见 design §2.6）。

### S-EDIT-05 `--` 边界：NEW_BODY 以 `-` 开头

- **Given** 线程中 #3 为 bob 所发且可编辑
- **When** 执行 `paperwork post edit standup.post.md bob 3 -- "-fix flag text"`
- **Then** exit 0；#3 正文逐字为 `-fix flag text`。

## 3. post read / summary / create

### S-READ-01 全量读取（窗口字段恒显，U-11）

- **Given** 线程共 6 条消息（≤ 默认 limit 20）
- **When** 执行 `paperwork post read standup.post.md`
- **Then** exit 0；首行 `ok post.read 6 messages`；字段区含 `showing: 6/6` 与 `window: #1-#6`（字段区形态，不放 conclusion 行）；body 为 6 条消息。

### S-READ-02 限量读取窗口指示

- **Given** 线程共 50 条消息
- **When** 执行 `paperwork post read standup.post.md --limit 20`
- **Then** 字段区含 `showing: 20/50` 与 `window: #31-#50`（按实际展示的第一条与最后一条 seq）；body 仅最后 20 条。

### S-READ-03 seq 范围过滤语义不变

- **Given** 线程存在
- **When** 执行 `paperwork post read standup.post.md --from 2 --to 3`
- **Then** 仅显示 #2、#3；`--from/--to` 仅表 seq 范围（全 CLI 唯一语义）。

### S-READ-04 read 上传身份值（旧语义误用落入 usage）

- **When** 执行 `paperwork post read standup.post.md --from alice`
- **Then** exit 2；`error usage:`（--from 只接受 u64）；example 示范 seq 范围用法。

### S-READ-05 文件不存在

- **When** 执行 `paperwork post read no-such.post.md`
- **Then** exit 1；`error not-found:`；fix/example 为新文法 send 建线程命令。

### S-READ-06 空线程不显示 window

- **Given** 线程存在但过滤后无任何消息（如 `--mention` 无命中）
- **When** 执行 `paperwork post read standup.post.md --mention nobody`
- **Then** exit 0；字段区含 `showing: 0/0`（total = 过滤后、limit 前）；**不**显示 `window` 字段；body 为空。

### S-READ-07 过滤 + limit 组合（total 口径，F3 裁定）

- **Given** 线程共 50 条，其中 alice 被 mention 25 次（`--mention` 过滤基于 mention-list，而非发言者）
- **When** 执行 `paperwork post read standup.post.md --mention alice --limit 20`
- **Then** 字段区含 `showing: 20/25`（total 为过滤后、limit 截断前，而非线程物理总数 50）；`window` 按实际展示的首末 seq（线程基准）；body 仅过滤结果的最后 20 条。

### S-SUM-01 summary 行为不变

- **Given** 线程存在
- **When** 执行 `paperwork post summary standup.post.md`
- **Then** exit 0；字段含 title/participants/messages/last.sender/last.time/last.snippet（与 v0.4 相同）。

### S-CREATE-01 create 新文法

- **When** 执行 `paperwork post create design "Design Discussion" --participants alice,bob`
- **Then** exit 0；`design.post.md` 创建（ensure_suffix），#1 为系统消息，TITLE 即 "Design Discussion"。

### S-CREATE-02 create 缺 TITLE（usage）

- **When** 执行 `paperwork post create design`
- **Then** exit 2；`error usage:`；example 为 post create 规范形态示例（含 TITLE 槽具体值）。

### S-CREATE-03 post create 重复（already-exists）

- **Given** `design.post.md` 已存在
- **When** 再次执行 `paperwork post create design "Another Title"`
- **Then** exit 1；`error already-exists:`。

## 4. profile

### S-PROF-01 create 新文法成功

- **When** 执行 `paperwork profile create agents/alice alice --model gpt-4o`
- **Then** exit 0；`agents/alice.profile.md` 创建；首行 `ok profile.create <path>`；字段含 `name: alice`。

### S-PROF-02 create 缺 NAME（usage）

- **When** 执行 `paperwork profile create agents/alice`
- **Then** exit 2；`error usage:`；example 形如 `paperwork profile create agents/alice alice --model gpt-4o`。

### S-PROF-03 旧文法 `--name` 落入 usage

- **When** 执行 `paperwork profile create agents/alice --name alice --model gpt-4o`
- **Then** exit 2；`error usage:`；example 为 profile create 的规范形态示例（NAME 位置化，不携带用户原参数值）。

### S-PROF-04 重复 create

- **Given** `agents/alice.profile.md` 已存在
- **When** 再次执行 `paperwork profile create agents/alice alice`
- **Then** exit 1；`error already-exists:`。

### S-PROF-05 show / edit / list 不变

- **Given** profile 存在
- **When** 分别执行 `profile show <PATH>`、`profile edit <PATH> --model x`、`profile list <DIR>`
- **Then** 输出结构与 v0.4 完全一致（ok 首行、字段、`(unreadable)` 容错）。

## 5. brief

### S-BRIEF-01 create 新文法

- **When** 执行 `paperwork brief create onboarding "Codebase Onboarding" --owner alice`
- **Then** exit 0；`onboarding.brief.md` 创建；`ok brief.create <path>`。

### S-BRIEF-02 add 新文法（U-07）

- **Given** brief 存在，且相对路径 `src/main.rs` 的文件存在
- **When** 执行 `paperwork brief add onboarding.brief.md src/main.rs --regex "fn main" --note "Entry point"`
- **Then** exit 0；`ok brief.add src/main.rs -> <brief路径>`；条目 hash 已快照。

### S-BRIEF-03 旧文法 `--entry` 落入 usage

- **When** 执行 `paperwork brief add onboarding.brief.md --entry src/main.rs`
- **Then** exit 2；`error usage:`；example 为 brief add 的规范形态示例（不携带用户原参数值）。

### S-BRIEF-07 add 与 remove 的参数映射（basename 推导）

- **Given** brief 存在，相对路径 `src/main.rs` 的文件存在
- **When** 先执行 `paperwork brief add onboarding.brief.md src/main.rs`，再执行 `paperwork brief remove onboarding.brief.md main.rs`
- **Then** 两步均 exit 0；条目存储标题为 basename `main.rs`，故 remove 传 basename 成功；若 remove 传原相对路径 `src/main.rs` 则 `error not-found:`（spec §3.3 推导规则）。

### S-BRIEF-04 remove 新文法

- **Given** brief 含条目 `main.rs`
- **When** 执行 `paperwork brief remove onboarding.brief.md main.rs`
- **Then** exit 0；`ok brief.remove main.rs`。

### S-BRIEF-05 remove 不存在的条目

- **When** 执行 `paperwork brief remove onboarding.brief.md no-such`
- **Then** exit 1；`error not-found:`；example 为新文法具体可执行命令（如 `paperwork brief remove onboarding.brief.md main.rs`）。

### S-BRIEF-06 read / verify 不变

- **Given** brief 存在
- **When** 执行 `brief read <PATH> [--full]` 与 `brief verify <PATH> [--base-dir DIR]`
- **Then** 输出样貌与三态判定与 v0.4 完全一致。

## 6. contacts

### S-CONTACTS-01 create 不变（--title 保留 flag）

- **When** 执行 `paperwork contacts create team --title "Core Team"`
- **Then** exit 0；`team.contacts.md` 创建，title 为 Core Team。

### S-CONTACTS-02 add 新文法（U-07）

- **Given** contacts 文件与 `alice.profile.md` 存在
- **When** 执行 `paperwork contacts add team.contacts.md alice.profile.md`
- **Then** exit 0；`ok contacts.add <profile> -> <contacts路径>`。

### S-CONTACTS-03 旧文法 `--profile` 落入 usage

- **When** 执行 `paperwork contacts add team.contacts.md --profile alice.profile.md`
- **Then** exit 2；`error usage:`；example 为 contacts add 的规范形态示例（不携带用户原参数值）。

### S-CONTACTS-05 contacts create 的 title 位置化误用（usage）

- **When** 执行 `paperwork contacts create team "Core Team"`（从 post/brief create 负迁移）
- **Then** exit 2；`error usage:`（多余位置参数）；after_help 注记 title 在 contacts create 为可选 flag（默认 Contacts）。

### S-CONTACTS-04 read 富化不变

- **Given** contacts 含可读 profile
- **When** 执行 `paperwork contacts read team.contacts.md`
- **Then** body 行为 `<路径>: <name> (<description>)`，与 v0.4 一致。

## 7. validate

### S-VAL-01 按后缀推断成功

- **Given** `standup.post.md` 为合法线程
- **When** 执行 `paperwork validate standup.post.md`
- **Then** exit 0；`ok validate <path>`。

### S-VAL-02 --type 覆盖后缀（U-15）

- **Given** 文件 `mystery.md` 内容为合法 post 线程但后缀非 `.post.md`
- **When** 执行 `paperwork validate mystery.md --type post`
- **Then** exit 0；按 post 解析器通过。

### S-VAL-03 未知后缀且未给 --type

- **Given** `random.txt` 存在
- **When** 执行 `paperwork validate random.txt`
- **Then** exit 1；`error format:`；fix 提示后缀要求或 `--type`；example 含 `--type` 用法。

### S-VAL-04 垃圾内容

- **Given** `garbage.post.md` 内容非法
- **When** 执行 `paperwork validate garbage.post.md`
- **Then** exit 1；`error format:`；example 为新文法 `paperwork post send myfile alice "hello"`。

### S-VAL-05 --type 非法值（usage）

- **When** 执行 `paperwork validate standup.post.md --type bogus`
- **Then** exit 2；`error usage:`（枚举值非法）；example 为含合法 `--type post` 的规范示例。

### S-VAL-06 --type 与后缀交叉

- **Given** `x.profile.md` 为合法 profile
- **When** 执行 `paperwork validate x.profile.md --type post`
- **Then** exit 1；`error format:`（按 post 解析器解析 profile 内容失败）——--type 显式覆盖后缀推断（spec §3.5）。

## 8. 横切场景：路径解析（U-14/N-02）

### S-PATH-01 原路径优先——存在的裸 .md 不被改写

- **Given** 存在文件 `standup.md`（恰好是一个合法线程，文件名无类型后缀）
- **When** 执行 `paperwork post read standup.md`
- **Then** exit 0；读取的是 `standup.md` 本体，**不**被改写为 `standup.post.md` 后报 not-found。

### S-PATH-02 原路径不存在时补后缀

- **Given** 不存在 `standup`、不存在 `standup.md`，但存在 `standup.post.md`
- **When** 执行 `paperwork post read standup`
- **Then** exit 0；实际解析为 `standup.post.md`。

### S-PATH-03 create 类命令补后缀

- **Given** `alice.profile.md` 尚不存在
- **When** 执行 `paperwork profile create alice alice`
- **Then** 文件落为 `alice.profile.md`（目标不存在，走补后缀分支）。

### S-PATH-04 两者都不存在

- **Given** `no-such` 与 `no-such.post.md` 均不存在
- **When** 执行 `paperwork post read no-such`
- **Then** exit 1；`error not-found:`，错误中路径为补后缀后的 `no-such.post.md`（与 v0.4 报错形态一致）。

### S-PATH-05 x.md 与 x.post.md 同时存在时用 x.md

- **Given** `x.md` 与 `x.post.md` 同时存在（内容不同，均为合法线程）
- **When** 执行 `paperwork post read x.md`
- **Then** exit 0；读取的是 `x.md` 本体（三级解析第①级：原路径原样存在即用原路径），不受 `x.post.md` 影响。

### S-PATH-06 send 自动创建落点（三级解析第③级，路径决策语义）

- **Given** `quick` 与 `quick.post.md` 均不存在
- **When** 执行 `paperwork post send quick alice "Hey"`
- **Then** exit 0；线程创建在补后缀路径 `quick.post.md`（第③级：都不存在 → 以补后缀路径为操作落点；物理创建仅因 send 是写命令而发生）。

### S-PATH-07 第①级命中异型文件（format 错误，不改道）

- **Given** `notes.md` 存在但为普通笔记（非 paperwork 线程格式），`notes.post.md` 不存在
- **When** 执行 `paperwork post send notes.md alice "hi"`
- **Then** exit 1；`error format:`（第①级命中原路径，按线程解析器解析失败）；**不**自动改道创建 `notes.post.md`（与 v0.4 无条件改写的行为差异，spec §5）；example 引导 `paperwork validate notes.md --type post` 或换一个线程路径。

### S-PATH-08 传入路径为已存在目录（第①级不命中）

- **Given** 目录 `threads/` 存在，`threads.post.md` 不存在
- **When** 执行 `paperwork post read threads`
- **Then** 第①级判据为 `is_file()`，目录不命中；后续级别亦无文件 → exit 1；`error not-found:`（路径为补后缀形态）；不产生任何文件/目录创建。

## 9. 横切场景：输出模式

### S-OUT-01 --json 成功

- **When** 执行 `paperwork --json post send standup alice "hi"`
- **Then** stdout 单行 JSON，含 `status:"ok"`、`command:"post.send"`、`conclusion`、`seq`、`sender` 等既有 key；exit 0。

### S-OUT-02 --json 运行时错误带 command 字段

- **When** 执行 `paperwork --json post read no-such.post.md`
- **Then** stdout 单行 JSON 错误对象，含 `status:"error"`、`category:"not-found"`、`command:"post.read"`（新增 key）、`exit_code`；进程 exit 1。

### S-OUT-03 --json usage 错误

- **When** 执行 `paperwork --json post send x.post.md --from alice "hi"`
- **Then** stdout 单行 JSON 错误对象，`category:"usage"`、含 `command` 字段与规范形态 `example`（不携带用户原参数值），`"exit_code":2`（如实反映进程退出码）；进程 exit 2；--json 感知通过 argv 扫描实现（try_parse 失败时尚无解析结果，spec §4.3）。

### S-OUT-04 -q 场景

- **When** 执行 `paperwork -q post read standup.post.md`
- **Then** 不打印 `ok ...` 首行；字段（含 showing/window）与 body 保留；exit 0。

### S-OUT-05 --plain 不变

- **When** 执行 `paperwork post read standup.post.md --plain --from 2 --to 3`
- **Then** 输出文件内该范围的原始字节形态（`---` 边界、`### #N sender · timestamp`、四反引号围栏），与 v0.4 一致。

### S-OUT-06 顶层解析失败（command 标识填 usage）

- **When** 执行 `paperwork`（不带任何组/动词）或 `paperwork --json`（同样缺子命令）
- **Then** exit 2；默认档 stderr 首行形如 `error usage: missing subcommand`；`--json` 模式 stdout JSON 含 `"command":"usage"`、`"category":"usage"`、`"exit_code":2`。

### S-OUT-07 --help/-V 穿透（冻结用例，F5 裁定）

- **When** 分别执行 `paperwork --help`、`paperwork post --help`、`paperwork -V`
- **Then** 三者均 exit 0；按 clap 原样输出帮助/版本（stdout）；**不**进 usage 信封、**不** exit 2（DisplayHelp/DisplayVersion 穿透条款，spec §4.3；守住 spec §6.3 全局 flag 语义不变）。

## 10. 横切场景：别名

### S-ALIAS-01 po 隐藏别名

- **When** 执行 `paperwork po read standup.post.md`
- **Then** 等价于 `paperwork post read ...`；exit 0；`po` 不出现在 `--help` 命令列表。

### S-ALIAS-02 既有别名不变

- **When** 分别执行 `paperwork p show ...`、`b read ...`、`c read ...`、`v ...`
- **Then** 行为与 v0.4 一致。
