# CLI 文法 v0.6: BDD（行为场景）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布）
- 文档性质：行为规范的行为化表述（Given/When/Then），覆盖全部命令的正常路径与错误路径
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令）
  - `docs/ssot/adr/feedbacks/v0.7_feedbacks.md`（本轮 owner 指令：contacts CRUD + 锁统一 + 渐进阅读；本轮新增场景 S-BRIEF-07~09、S-CONTACTS-06~11、§12 锁行为场景的依据）
  - `docs/researches/cli-grammar-v06-reassessment-2026-08-09.md`（混淆面矩阵与错误等级标注）
  - `docs/dev/adr-v1.md`（ADR-011）、`docs/ssot/adr/feedbacks/v0_feedbacks.md`
  - `docs/dev/owner-rulings-2026-08-15.md`（2026-08-15 owner 四项裁决：写侧糖衣 flag 撤销与读侧过滤器保留；S-SEND-04/20 改写、S-SEND-22/23、S-EDIT-10、S-CONTACTS-16/17 与 S-CONTACTS-14 追加、S-SHORT-02 枚举收窄的依据）
- 契约基准：本目录 `spec.md`（签名、信封、退出码语义均以 spec 为准）；信封结构沿用 v0.5 spec §4（继承冻结）

约定：`<exit N>` 表示进程退出码；「信封」指 v0.5 spec §4 定义的 envelope（v0.6 零变更）；示例路径均为相对示例，行为与 cwd 无关（无状态）。v0.5 bdd 中未列于本文的场景（并发 seq 无间隙、--json/--plain/-q 三档、--help/-V 穿透、别名、ensure_suffix 三级解析）行为冻结，按 v0.5 bdd 对应场景回归，仅命令示例换 v0.6 文法。**编号约定（rework 补录，Nora ISSUE-m5）**：本文场景编号为 v0.6 独立编号，与 v0.5 bdd 存在同名异义撞号（如 S-SEND-09/10/12、S-EDIT-02/03 等）；凡引用 v0.5 场景一律带 v0.5 前缀；跨文档引用未带前缀的 S-xxx 一律指本文（v0.6）编号。

---

## 1. post send

### S-SEND-01 v0.6 文法成功发送

- **Given** 存在线程文件 `standup.post.md`
- **When** 执行 `paperwork post send standup.post.md --author alice --message "Parser done"`
- **Then** exit 0；stdout 首行 `ok post.send #N -> <path>`；字段区含 `seq`、`path`、`sender: alice`；文件追加一条 alice 的消息。

### S-SEND-02 短形式与全称等价

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md -a alice -m "short form"`
- **Then** exit 0；行为与 S-SEND-01 逐字等价（sender: alice，正文一致）；`--author/-a`、`--message/-m` 为同一 Arg 两面。

### S-SEND-03 线程不存在时自动创建

- **Given** `quick-chat.post.md` 不存在
- **When** 执行 `paperwork post send quick-chat --author alice --message "Hey"`
- **Then** exit 0；`quick-chat.post.md` 被创建且首条消息即该消息（#1，ensure_suffix 第(3)级落点，行为沿用 v0.5）。

### S-SEND-04 正文 @#N reply 语义的 implicit-mention 显式化（2026-08-15 owner 裁决改写：原依赖写侧 `--reply-to` 糖衣 flag，改为正文直书形态）

- **Given** 线程中 #2 为 alice 所发
- **When** 执行 `paperwork post send standup.post.md --author bob --message "@#2 Sure"`（reply 语义由 agent 正文直书 `@#N` token 表达，spec §3.1 撤销声明）
- **Then** exit 0；字段区含 `implicit-mention: alice`（单数字段、仅触发时出现、三种不触发边界均沿用 v0.5 bdd S-SEND-10b/S-SEND-11；derive 机制自正文 `@#N` token 派生，冻结不变）；`--json` 模式含同名 key。

### S-SEND-05 缺 --author（usage）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md --message "Hello"`
- **Then** exit 2；stderr 首行 `error usage:`（required flag 未提供）；example 为含 `--author` 与 `--message` 完整必填形态的规范示例（具体值）；不产生任何文件写入。

### S-SEND-06 缺正文通道（usage）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md --author alice`（无 `--message` 且无 `--stdin`）
- **Then** exit 2；`error usage:`；example 为单一静态规范可执行示例（采 `--message` 通道形态，具体值）；「二选一」指引由 message/fix 文案承担，不在 example 中表达（rework 裁定 F5）；不产生文件写入。

### S-SEND-07 --message 与 --stdin 同给（usage）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md --author alice --message "body" --stdin`
- **Then** exit 2；stderr 首行 `error usage:`（clap conflicts 判定，先于任何 I/O）；message 说明两者不可同时给出；example 为单一静态规范可执行示例（rework 裁定 F5，同 S-SEND-06 口径）；不产生文件写入。（v0.5 该冲突为 validation exit 1，本版层级提升，spec §5 第 1 条。）

### S-SEND-08 仅 --stdin 成功

- **Given** 存在线程文件，stdin 内容为多行正文
- **When** 执行 `echo multi-line | paperwork post send standup.post.md --author alice --stdin`
- **Then** exit 0；正文逐字为 stdin 内容；行为沿用 v0.5。

### S-SEND-09 空正文被拒绝（validation）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md --author alice --message "   "`
- **Then** exit 1；stderr 首行 `error validation:`（trim 后为空）；example 为 v0.6 完整形态。

### S-SEND-10 `--message` 值以 `-` 开头（flag 值直传，无需 `--` 边界）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md --author alice --message "-fix flag text"`
- **Then** exit 0；正文逐字为 `-fix flag text`；无需 `--` 边界（v0.5 S-SEND-07 的 `--` 形态废止，spec §5 第 3 条）。

### S-SEND-11 裸 `-` 开头 token 未走 --message（usage 教学）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md --author alice -fix`（意图发正文但误写为裸 token）
- **Then** exit 2；`error usage:`（clap 把 `-fix` 解析为未知 flag）；fix 文案引导正文经 `--message` 传递；example 为 `--message "-fix flag text"` 规范形态（静态示例，不携带用户原参数值）。

### S-SEND-12 v0.5 位置文法调用落入 usage 信封（迁移教学）

- **Given** 存在线程文件 `x.post.md`
- **When** 执行 `paperwork post send x.post.md alice "Hi"`（v0.5 文法：NAME/BODY 位置参数）
- **Then** exit 2；stderr 首行 `error usage:`（多余位置参数，v0.6 位置槽仅剩 PATH）；example 为 post send 的 v0.6 规范形态示例（含 `--author/--message`，具体可执行值，不携带用户原参数值）；无文件写入。

### S-SEND-13 v0.4 旧 flag 调用落入 usage 信封（迁移链延伸）

- **When** 执行 `paperwork post send x.post.md --from alice "hi"`
- **Then** exit 2；`error usage:`（`--from` 在 send 中不存在）；example 为 v0.6 规范形态示例。

### S-SEND-14 并发 send seq 无间隙（冻结回归）

- **Given** 存在线程文件
- **When** 两个进程并发执行 v0.6 文法 send
- **Then** 两条消息 seq 连续无间隙（文件锁行为不变）。

### S-SEND-15 NAME/BODY 混淆面消亡确认

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md "body text"`（v0.5 混淆面形态：单字符串）
- **Then** exit 2；`error usage:`（多余位置参数；该字符串不再可能被绑定为署名）；example 为 v0.6 完整形态。v0.5 bdd S-SEND-12 的「静默写入错误 sender」路径（研究文档矩阵第 3 行）结构性不可达。

### S-SEND-17 既有线程附 --title 静默忽略（冻结契约钉住，rework 裁定 F6，基线勘误后缩减）

- **Given** 存在线程文件 `standup.post.md`，标题为 `Old Title`
- **When** 执行 `paperwork post send standup.post.md --author alice --message "new msg" --title "New Title"`
- **Then** exit 0；消息正常追加；标题仍为 `Old Title`，`--title` 被静默忽略（仅首次写入、锁内 size==0 时生效，spec §3.1 行为登记，OQ-1；本轮不改运行时行为）。基线勘误：本场景原覆盖 `--title/--participants/--to` 三 flag，`--participants/--to` 已随 owner 追裁 D1/D2 删除，场景相应缩减为仅 `--title`（原 S-SEND-16 一并删除）。

### S-SEND-18 --author 空值拒绝（validation，rework 补录 Pete N2-(1)）

- **When** 执行 `paperwork post send standup.post.md --author "   " --message "hi"`
- **Then** exit 1；`error validation:`（trim 后为空，spec §3.1 条款的场景承载）；无文件写入。

### S-SEND-19 缺 PATH（usage，rework 补录 Pete N2-(3)）

- **When** 执行 `paperwork post send --author alice --message "hi"`（v0.6 文法集内缺唯一位置参数）
- **Then** exit 2；`error usage:`；example 为 v0.6 完整形态；无文件写入。

### S-SEND-20 正文 @#N/@name 语义往返（2026-08-15 owner 裁决改写：原为写侧糖衣 flag token 注入场景，改为正文直书 + 读侧 derive 往返）

- **Given** `newtopic.post.md` 不存在
- **When** 首次执行 `paperwork post send newtopic.post.md --author alice --message "first"`，再执行 `paperwork post send newtopic.post.md --author bob --message "@#1 @carol agreed"`（reply/mention 语义由 agent 正文直书，spec §3.1）
- **Then** 两次均 exit 0；第二条消息落盘正文逐字含 `@#1 @carol`（正文即语义载体，无任何注入变换）；`post read --reply-to 1` 与 `--mention carol` 过滤均命中第二条（读取时 derive，机制不变，读侧过滤器保留声明 spec §3.3/§10）；preamble 无 `- participants:` 行（D1）；全文件无 `--to`/`--participants` 属性行残留。
- **撤销声明**：写侧糖衣 flag `--reply-to`/`--mention` 已撤销，传入落 usage exit 2（见 S-SEND-22/S-SEND-23/S-EDIT-10；backlog B-01/U-04 问题面消解）。

### S-SEND-21 首次 send 建线程 preamble 仅 H1 标题（基线勘误后新增，format-v2 D1/OQ-1 行为补齐）

- **Given** `daily.post.md` 不存在
- **When** 执行 `paperwork post send daily.post.md --author alice --message "hi" --title "Daily Standup"`
- **Then** exit 0；落盘文件以 `# Daily Standup` H1 行起始，其后直接是 `## #1 alice (...)` 消息头（无占位创建消息、无属性行）；缺 `--title` 时标题取路径剥 `.post.md` 后缀（spec §3.1 糖衣表）。

### S-SEND-22 写侧已撤销 flag `--reply-to` 落入 usage（2026-08-15 owner 裁决新增）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md --author bob --message "Sure" --reply-to 2`
- **Then** exit 2；`error usage:`（`--reply-to` 在 send 中不存在，按未知 flag 路径）；fix/example 引导 reply 语义在正文直书 `@#2`（如 `paperwork post send standup.post.md --author bob --message "@#2 Sure"`，静态规范示例口径，spec §3.1）；无文件写入。

### S-SEND-23 写侧已撤销 flag `--mention` 落入 usage（2026-08-15 owner 裁决新增）

- **Given** 存在线程文件
- **When** 执行 `paperwork post send standup.post.md --author alice --message "hi" --mention carol`
- **Then** exit 2；`error usage:`；fix/example 引导 mention 语义在正文直书 `@carol`；无文件写入。

### S-EDIT-10 写命令传入已撤销 flag（usage，2026-08-15 owner 裁决新增；「写命令」外延防线）

- **When** 执行 `paperwork post edit standup.post.md --author bob --seq 3 --message "x" --reply-to 2`（edit 本无该 flag，撤销口径按写命令外延声明，spec §2）
- **Then** exit 2；`error usage:`（未知 flag）；example 为 edit 完整必填形态规范示例；无文件写入。

## 2. post edit

### S-EDIT-01 v0.6 文法成功编辑

- **Given** 线程中 #3 为 bob 所发且是 bob 的最新消息、也是线程最后一条
- **When** 执行 `paperwork post edit standup.post.md --author bob --seq 3 --message "edited"`
- **Then** exit 0；stdout 首行 `ok post.edit #3`；#3 正文变为 edited。

### S-EDIT-02 缺 --author（usage）

- **When** 执行 `paperwork post edit standup.post.md --seq 3 --message "x"`
- **Then** exit 2；`error usage:`；example 为含 `--author/--seq/--message` 完整必填形态的规范示例；无文件写入。

### S-EDIT-03 缺 --seq（usage）

- **When** 执行 `paperwork post edit standup.post.md --author bob --message "x"`
- **Then** exit 2；`error usage:`；example 为完整必填形态规范示例。

### S-EDIT-04 缺正文通道（usage）

- **When** 执行 `paperwork post edit standup.post.md --author bob --seq 3`
- **Then** exit 2；`error usage:`；example 为单一静态规范可执行示例（采 `--message` 通道形态，rework 裁定 F5，同 S-SEND-06 口径）；无文件写入。

### S-EDIT-05 --message 与 --stdin 同给（usage）

- **When** 执行 `paperwork post edit standup.post.md --author bob --seq 3 --message "x" --stdin`
- **Then** exit 2；`error usage:`（conflicts）；example 为单一静态规范可执行示例（rework 裁定 F5）；不产生文件写入。

### S-EDIT-06 SEQ 非数字（usage）

- **When** 执行 `paperwork post edit standup.post.md --author bob --seq abc --message "x"`
- **Then** exit 2；`error usage:`（u64 解析失败）；example 给出合法 `--seq` 形态。

### S-EDIT-07 三重护栏（not-allowed，冻结回归）

- **Given** #3 为 bob 所发
- **When** 执行 `paperwork post edit standup.post.md --author alice --seq 3 --message "x"`
- **Then** exit 1；`error not-allowed:`，message 精确指出 `sent by 'bob', not 'alice'`；example 为 v0.6 具体可执行命令（如 `paperwork post edit standup.post.md --author bob --seq 3 --message "x"`）。

### S-EDIT-08 v0.5 位置文法调用（usage）

- **When** 执行 `paperwork post edit standup.post.md bob 3 "edited"`
- **Then** exit 2；`error usage:`（多余位置参数）；example 为 v0.6 规范形态示例。

### S-EDIT-09 仅 --stdin 成功（rework 补录 Pete N2-(2)）

- **Given** 线程中 #3 为 bob 所发且满足三重护栏，stdin 内容为新正文
- **When** 执行 `echo edited | paperwork post edit standup.post.md --author bob --seq 3 --stdin`
- **Then** exit 0；#3 正文逐字为 stdin 内容（edit 侧 stdin 通道与 send 侧 S-SEND-08 对等）。

## 3. post read / summary（冻结回归，示例换 v0.6 文法）

### S-READ-01 窗口字段恒显（冻结）

- **Given** 线程共 6 条消息
- **When** 执行 `paperwork post read standup.post.md`
- **Then** exit 0；字段区含 `showing: 6/6` 与 `window: #1-#6`（行为沿用 v0.5 bdd S-READ-01/02/06/07）。total 口径与空 window 形态的对等场景见下文 S-READ-06/07（rework 补录 Nora ISSUE-m3）。

### S-READ-02 seq 范围过滤（冻结）

- **When** 执行 `paperwork post read standup.post.md --from 2 --to 3`
- **Then** 仅显示 #2、#3；`--from/--to` 仅表 seq 范围。

### S-READ-03 --from 传身份值（usage）

- **When** 执行 `paperwork post read standup.post.md --from alice`
- **Then** exit 2；`error usage:`（--from 只接受 u64）；example 示范 seq 范围用法。

### S-READ-04 --mention 无短形式

- **When** 执行 `paperwork post read standup.post.md -m alice`
- **Then** exit 2；`error usage:`（`-m` 在 read 中不存在，刻意避让 `--message` 短形式）；`--mention alice` 全称形态 exit 0。

### S-READ-05 文件不存在（冻结）

- **When** 执行 `paperwork post read no-such.post.md`
- **Then** exit 1；`error not-found:`；fix/example 为 v0.6 send 建线程命令（`--author/--message` 形态）。

### S-READ-06 过滤后零命中的 total 口径与空 window（rework 补录 Nora ISSUE-m3，对等 v0.5 bdd S-READ-06）

- **Given** 线程共 4 条消息，发送者均非 carol
- **When** 执行 `paperwork post read standup.post.md --mention carol`
- **Then** exit 0；`showing: 0/0`（total 为过滤后口径，与 S-READ-07 同口径；冻结基线实测即为 0/0，初稿误写 0/4，按实现收口，fix-ledger A-01）；`window` 字段不显示（空 window 冻结行为，与 v0.5 bdd S-READ-06 一致）；body 为空。

### S-READ-07 过滤+limit 的 total 口径（rework 补录 Nora ISSUE-m3，对等 v0.5 bdd S-READ-07）

- **Given** 线程共 50 条消息，其中 25 条含 bob
- **When** 执行 `paperwork post read standup.post.md --mention bob --limit 20`
- **Then** exit 0；`showing: 20/25`（total 为过滤后口径而非原始 50，冻结语义）；`window` 为实际显示区间的 #a-#b。

### S-READ-08 read --to 身份值落入 usage（类型防线，基线勘误后独立成立）

- **When** 执行 `paperwork post read standup.post.md --to bob`
- **Then** exit 2；`error usage:`（read `--to` 为 u64 seq 上限，clap 类型解析失败，spec §1.4）；example 示范 seq 范围用法；无文件写入。基线勘误：原与 S-SEND-16 构成对偶，S-SEND-16 已随 send `--to` flag 删除而删除，本场景作为 read 侧类型防线独立保留。

### S-READ-09 read --author 习惯迁移（usage，rework 补录 Pete N2-(4)）

- **When** 执行 `paperwork post read standup.post.md --author alice`（把 send 的 --author 习惯带进 read）
- **Then** exit 2；`error usage:`（read 无 `--author` flag，按发送者过滤请用 `--mention`）；fix 文案点名 `--mention` 替代路径；无文件写入。

### S-READ-10 H1 宽容面：缺 H1 / 双 H1 线程读与 validate 均放行（2026-08-15 修复波 F2 登记，audit-robustness-round2 B-8 / T-05/T-06）

- **Given** 两个 post 线程夹具：其一缺 H1 标题（直接以 `## #1` 消息头开始），其二含两个 H1 标题行；两者消息结构均合法
- **When** 分别对两者执行 `paperwork post read <PATH>` 与 `paperwork validate <PATH> --type post`
- **Then** 全部 exit 0；read 的消息计数与正文解析不受 H1 存在性影响；validate 的结构检查面（消息边界、seq 单调、fence 闭合）照常执行；无文件写入。语义裁定注记见 spec §3.3/§3.7；测试钉住 cli_integration `h1_leniency_missing_and_duplicate_h1_read_cleanly`（钉住现行行为，不改行为）。

### S-SUM-01 summary 行为不变

- **When** 执行 `paperwork post summary standup.post.md`
- **Then** exit 0；字段含 title/participants/messages/last.sender/last.time/last.snippet（字段集与 v0.5 相同；基线勘误后 title 取自 H1 preamble、participants 由消息 sender 集合派生，D1）；缺线程报 not-found 形态同 read。

## 4. profile

### S-PROF-01 create v0.6 文法成功

- **When** 执行 `paperwork profile create agents/alice --name alice --model gpt-4o`
- **Then** exit 0；`agents/alice.profile.md` 创建；首行 `ok profile.create <path>`；字段含 `name: alice`。

### S-PROF-02 create 缺 --name（usage）

- **When** 执行 `paperwork profile create agents/alice`
- **Then** exit 2；`error usage:`；example 形如 `paperwork profile create agents/alice --name alice --model gpt-4o`。

### S-PROF-03 v0.5 位置文法调用（usage）

- **When** 执行 `paperwork profile create agents/alice alice`（v0.5 文法：NAME 位置参数）
- **Then** exit 2；`error usage:`（多余位置参数）；example 为 `--name` 形态规范示例。

### S-PROF-04 重复 create（冻结）

- **Given** `agents/alice.profile.md` 已存在
- **When** 再次执行 `paperwork profile create agents/alice --name alice`
- **Then** exit 1；`error already-exists:`。

### S-PROF-05 show / edit / list 不变

- **Then** 输出结构与 v0.5 完全一致（ok 首行、字段、`(unreadable)` 容错）。

## 5. brief

### S-BRIEF-01 create v0.6 文法

- **When** 执行 `paperwork brief create onboarding --title "Codebase Onboarding" --owner alice`
- **Then** exit 0；`onboarding.brief.md` 创建；`ok brief.create <path>`。

### S-BRIEF-02 add v0.6 文法

- **Given** brief 存在，且相对路径 `src/main.rs` 的文件存在
- **When** 执行 `paperwork brief add onboarding.brief.md --entry src/main.rs --regex "fn main" --note "Entry point"`
- **Then** exit 0；`ok brief.add src/main.rs -> <brief路径>`；条目 hash 已快照。

### S-BRIEF-03 缺必填 flag（usage）

- **When** 分别执行 `paperwork brief create onboarding`、`paperwork brief add onboarding.brief.md`、`paperwork brief remove onboarding.brief.md`
- **Then** 三者均 exit 2；`error usage:`；example 分别为含 `--title` / `--entry` / `--entry-title` 的规范形态示例。

### S-BRIEF-04 v0.5 位置文法调用（usage）

- **When** 执行 `paperwork brief add onboarding.brief.md src/main.rs`（v0.5 文法：ENTRY 位置参数）
- **Then** exit 2；`error usage:`（多余位置参数）；example 为 `--entry` 形态规范示例。

### S-BRIEF-05 remove 与 basename 推导（冻结）

- **Given** brief 存在，相对路径 `src/main.rs` 的文件存在
- **When** 先执行 `paperwork brief add onboarding.brief.md --entry src/main.rs`，再执行 `paperwork brief remove onboarding.brief.md --entry-title main.rs`
- **Then** 两步均 exit 0（存储标题为 basename）；remove 传 `src/main.rs` 则 `error not-found:`（推导规则沿用 v0.5 spec §3.3）。

### S-BRIEF-06 read / verify 不变

- **Then** 输出样貌与三态判定（fresh/shifted/stale、conclusion `N/M fresh`）与 v0.5 完全一致。

### S-BRIEF-07 read `--entry-title` 选择性详情（本轮新增，渐进阅读第三档）

- **Given** brief 存在，共 2 条目（`main.rs` 与 `lib.rs`）
- **When** 执行 `paperwork brief read onboarding.brief.md --entry-title main.rs`
- **Then** exit 0；stdout 首行 `ok brief.read 2 entries`（conclusion 为全量条目数 `N entries` 形态，与现状冻结口径一致，worktree cmd/brief.rs L171/L197 实测；rework 修订 Daniel M-2：初版首行断言 `ok brief.read <path>` 与 S-BRIEF-06 冻结声明矛盾，改为现状形态）；输出仅含 `main.rs` 条目的详情字段（path/hash/regex/note，即 `--full` 档字段）；不含 `lib.rs` 条目详情；`--json` 模式 entries 数组仅含该条目，且命中条目含 path/hash/regex/note（`--entry-title` 命中即按 `--full` 档字段输出，不受 `--full` 门控，spec §3.5 字段面口径，Daniel m-4）。

### S-BRIEF-08 read `--entry-title` 无匹配（not-found）

- **When** 执行 `paperwork brief read onboarding.brief.md --entry-title no-such.rs`
- **Then** exit 1；stderr 首行 `error not-found:`（resource Brief entry）；fix 引导 `paperwork brief read onboarding.brief.md` 列出条目；无文件写入。

### S-BRIEF-09 read `--entry-title` 与 `--full` 组合

- **When** 执行 `paperwork brief read onboarding.brief.md --full --entry-title main.rs`
- **Then** exit 0；行为与 S-BRIEF-07 一致（单条目详情，两 flag 组合合法无冲突）；未给 `--entry-title` 时 TOC / `--full` 两档行为冻结不变（S-BRIEF-06）。

### S-BRIEF-10 read `--entry-title` 空值守栏（评审轮新增，F1）

- **Given** brief 存在，含任意条目
- **When** 执行 `paperwork brief read onboarding.brief.md --entry-title ""`（或全空白值）
- **Then** exit 1；stderr 首行 `error validation:`；message 逐字 `entry title (--entry-title) is empty`；fix 逐字 `provide a non-empty --entry-title value`；example 逐字 `paperwork brief read onboarding.brief.md --entry-title main.rs`（纯 ASCII，镜像 post send 空值判定先例）；无文件写入。属行为变更登记：护栏前该形态落入 not-found（空键是「无键」而非未命中，spec §3.5 空值守栏）。

## 6. contacts

### S-CONTACTS-01 create 不变（--title 保留可选 flag）

- **When** 执行 `paperwork contacts create team --title "Core Team"`
- **Then** exit 0；`team.contacts.md` 创建，title 为 Core Team。

### S-CONTACTS-02 add v0.6 文法

- **Given** contacts 文件与 `alice.profile.md` 存在
- **When** 执行 `paperwork contacts add team.contacts.md --profile alice.profile.md`
- **Then** exit 0；`ok contacts.add <profile> -> <contacts路径>`。

### S-CONTACTS-03 add 缺 --profile（usage）

- **When** 执行 `paperwork contacts add team.contacts.md`
- **Then** exit 2；`error usage:`；example 为 `--profile` 形态规范示例。

### S-CONTACTS-04 v0.5 位置文法调用（usage）

- **When** 执行 `paperwork contacts add team.contacts.md alice.profile.md`（v0.5 文法：PROFILE-PATH 位置参数）
- **Then** exit 2；`error usage:`（多余位置参数）；example 为 `--profile` 形态规范示例。

### S-CONTACTS-05 read 富化不变

- **Then** body 行为 `<路径>: <name> (<description>)`，与 v0.5 一致。

### S-CONTACTS-06 remove 成功（本轮新增）

- **Given** contacts 文件 `team.contacts.md` 含条目 alice、bob（顺序在先）
- **When** 执行 `paperwork contacts remove team.contacts.md --profile alice.profile.md`
- **Then** exit 0；stdout 首行 `ok contacts.remove alice.profile.md -> team.contacts.md`；字段区含 `contacts: team.contacts.md`、`removed: alice.profile.md`；文件中 alice 条目消失，bob 条目保留且标题（H1）不变；`--json` 模式含同名 key（command `contacts.remove`）。

### S-CONTACTS-07 remove 未命中（not-found）

- **Given** contacts 文件不含 `ghost.profile.md` 条目，但含名为 alice 的条目（label = alice，键 = 其 profile 路径）
- **When** 执行 `paperwork contacts remove team.contacts.md --profile ghost.profile.md`
- **Then** exit 1；stderr 首行 `error not-found:`（resource Contacts entry）；fix 引导 `paperwork contacts read team.contacts.md` 核对条目清单，并含键口径教学句 `the key is the profile path as stored in the contacts file, not the label`（纯 ASCII，rework 补录 Ryan m-3）；文件内容不变。
- **And label-as-key 触发形态（rework 补录，Ryan m-3）**：执行 `paperwork contacts remove team.contacts.md --profile alice`（把 label 当键）同样 exit 1 `error not-found:`（键为存储路径字符串精确匹配，`alice` 非任何条目的 destination）；fix/example 同上，agent 经 contacts read 一步自纠。

### S-CONTACTS-08 update 成功（label 重派生 + 顺序保留）

- **Given** contacts 文件含条目 [alice, bob]；`carol.profile.md` 存在且 H1 为 `carol`
- **When** 执行 `paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md`
- **Then** exit 0；stdout 首行 `ok contacts.update alice.profile.md -> carol.profile.md`；字段区含 `contacts`、`updated: alice.profile.md -> carol.profile.md`（编排层裁定维持箭头串形态，值格式逐字钉住为 `<OLD> -> <NEW>` 单空格三段拼接，spec §3.6，rework 补录 Ryan m-4）；文件中 alice 条目被原地替换为 `[carol](carol.profile.md)`（label 依 R11 对 NEW 重派生），bob 条目位置不变（顺序保留）；`--json` 模式含同名 key（command `contacts.update`）。

### S-CONTACTS-09 update 错误路径（not-found / already-exists）

- **Given** contacts 文件含条目 [alice, bob]
- **When** 分别执行 `paperwork contacts update team.contacts.md --profile ghost.profile.md --new-profile carol.profile.md` 与 `paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile bob.profile.md`
- **Then** 前者 exit 1 `error not-found:`（OLD 未命中）；后者 exit 1 `error already-exists:`（NEW 已存在于清单，fix 引导先 remove 或改用既有条目）；两次均无文件写入；update 不支持改 label（无任何 label 类 flag，label 为 R11 派生数据）。

### S-CONTACTS-10 remove/update 缺必填 flag（usage）

- **When** 分别执行 `paperwork contacts remove team.contacts.md`、`paperwork contacts update team.contacts.md`、`paperwork contacts update team.contacts.md --profile alice.profile.md`
- **Then** 三者均 exit 2；`error usage:`；example 逐字钉住（rework 补录 Ryan m-2，spec §5 第 2 条同源）：remove 缺 flag 的 example 为 `paperwork contacts remove team.contacts.md --profile alice.profile.md`；update 两形态的 example 均为 `paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md`（静态规范示例，不携带用户原参数值）；无文件写入。

### S-CONTACTS-11 旧文法不受影响与幂等回归

- **When** 执行 `paperwork contacts remove team.contacts.md alice.profile.md`（误把 profile 路径作位置参数）
- **Then** exit 2；`error usage:`（多余位置参数，v0.6 位置槽仅剩 PATH）；example 为 `--profile` 形态规范示例。
- **And** contacts add 幂等语义不变：对已存在条目再次 add 仍 exit 0 no-op（spec §3.6，行为冻结）；contacts create/add/read 既有行为（S-CONTACTS-01~05）逐条冻结。

### S-CONTACTS-12 remove 最后一条目后的文件形态（rework 补录，Daniel M-4）

- **Given** contacts 文件 `solo.contacts.md` 仅含一条目（`alice.profile.md`）
- **When** 执行 `paperwork contacts remove solo.contacts.md --profile alice.profile.md`
- **Then** exit 0；落盘文件仅剩 title H1 + 空行（`serialize_contacts(title, &[])` 产物，与 contacts create 初态同形）；`paperwork validate solo.contacts.md --type contacts` 判定合法；再对同键执行 remove -> not-found exit 1（文件不变）。

### S-CONTACTS-13 特殊字符路径的 remove/update 往返（rework 补录，Daniel M-4）

- **Given** contacts 文件含 destination 需转义的条目（路径含空格/括号，如 `my profile (v2).profile.md`，序列化走 `<...>` 形态）
- **When** 执行 `paperwork contacts update team.contacts.md --profile "my profile (v2).profile.md" --new-profile "new dir/prof.profile.md"`
- **Then** exit 0；键匹配以**未转义原串**为准命中；新 destination 经序列化后走 angle-bracket 形态；往返后其余条目字节不变；再对同键执行 remove 仍命中（二次操作键匹配不受转义形态影响）；全程 validate 合法。

### S-CONTACTS-14 update 到不存在 NEW 的静默成功面（rework 补录，Ryan M-3；仿 S-SEND-17 三件套形态）

- **声明面**：update 到不存在/不可读的 NEW 仍 exit 0 静默写入（destination 按原值落盘、label 依 R11 回退文件名主干），与 format-v2 R11 及 add 现状一致，属已知静默面非缺陷（spec §3.6 行为契约，本轮不改运行时）。
- **Given** contacts 文件含条目 alice；`carol` 文件不存在（忘写 `.profile.md` 后缀的路径笔误形态）
- **When** 执行 `paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol`
- **Then** exit 0；条目落为 `[carol](carol)`（label 回退文件名主干）；`updated` 字段回显 `alice.profile.md -> carol`（原值）；下一轮 `contacts read` 该条目显示 `(unreadable)` 类容错形态。
- **agent 自救指引**：fix/example 与文档面引导写入前先 `paperwork contacts read <PATH>` 核对既有条目、或 `paperwork validate carol.profile.md` 确认目标 profile 合法；候选增强「写前 destination 存在性校验/回显」登记 backlog B-02，~~本轮不实现~~，2026-08-15 owner 裁决落地为非阻塞 advisory 校验（本场景 exit 0 静默写入语义不变，其上叠加 `advisory` 信封字段，见 S-CONTACTS-16/17；spec §3.6「destination advisory 校验契约」）。

### S-CONTACTS-15 add/update 空键护栏（评审轮新增，F1；Kim M-1 + QA BUG-1）

- **Given** contacts 文件 `team.contacts.md` 含条目 alice
- **When** 分别执行 `paperwork contacts add team.contacts.md --profile ""`、`paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile ""`（或全空白值）
- **Then** 两者均 exit 1；stderr 首行 `error validation:`；message 逐字分别为 `profile path (--profile) is empty` 与 `new profile path (--new-profile) is empty`；fix 逐字分别为 `provide a non-empty --profile value` 与 `provide a non-empty --new-profile value`；example 为各动词规范示例（S-CONTACTS-10 同源逐字形态，纯 ASCII，镜像 post send `--message`/`--author` 空值判定先例）；两次均无文件写入；护栏先于文件存在性判定（目标文件不存在亦落 validation）；库直调（core `contacts_add`/`contacts_update`）同受护栏。
- **行为变更登记**：护栏前 add 空键写入不可解析 bullet `- []()`（下次解析静默消失，validate 判结构损坏，属静默数据损坏）；update 空 `--new-profile` 把既有条目替换成该退化 bullet。护栏后既有条目不可经空键破坏；非空但不可读的路径仍依 S-CONTACTS-14 / add 静默回退判例（护栏仅针对「无键」，不改变「不可读路径」行为面）。

### S-CONTACTS-16 add destination advisory 非阻塞校验（2026-08-15 owner 裁决新增；spec §3.6 advisory 契约）

- **Given** contacts 文件 `team.contacts.md` 存在；`ghost.profile.md` 不存在
- **When** 执行 `paperwork contacts add team.contacts.md --profile ghost.profile.md`
- **Then** exit 0（destination 问题不阻塞写入，永不因 destination 问题 exit≠0）；条目照常落盘，label 依 R11 回退文件名主干（`[ghost](ghost.profile.md)`）；ok 信封字段区含 `advisory`（值提示 destination 不存在，建议文案 `destination 'ghost.profile.md' does not exist`，本例逐字节纯 ASCII；ASCII 声明口径收窄见 spec §3.6：文案模板恒纯 ASCII，destination 原文插值回显，整行 ASCII 仅当路径为 ASCII（Ray S-1）；2026-08-15 任务 #36 实施定稿冻结：建议形态逐字采用）；`--json` 模式含同名 key `advisory`（只增不改协议）。
- **And 不触发形态**：destination 为存在且合法的 profile 文件时（如 S-CONTACTS-02 的 alice.profile.md），信封**不含** `advisory` 字段（仅触发时出现，避免噪音）；S-CONTACTS-02 既有断言面冻结不变。
- **And 格式非法触发形态**：destination 存在但非合法 profile（内容损坏）时仍 exit 0 照常写入，`advisory` 提示格式非法（2026-08-15 任务 #36 实施定稿冻结：`destination '<P>' is not a valid profile file`）。

### S-CONTACTS-17 update destination advisory 触发形态（2026-08-15 owner 裁决新增；与 S-CONTACTS-14 叠加）

- **Given** contacts 文件含条目 [alice]；`carol` 文件不存在（忘后缀笔误形态，同 S-CONTACTS-14 Given）
- **When** 执行 `paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol`
- **Then** exit 0；行为面与 S-CONTACTS-14 逐条一致（条目落为 `[carol](carol)`、`updated` 回显 `alice.profile.md -> carol`）；叠加断言：ok 信封含 `advisory` 字段（destination 不存在提示）；`--json` 模式同时含 `updated` 与 `advisory` 两 key；其余条目字节不变。

## 7. validate（冻结回归，示例换 v0.6 文法）

### S-VAL-01 按后缀推断成功 / S-VAL-02 --type 覆盖后缀 / S-VAL-03 未知后缀 / S-VAL-05 --type 非法值 / S-VAL-06 --type 与后缀交叉

- **Then** 五场景行为与 v0.5 bdd S-VAL-01~06 逐条一致（exit 0 / exit 0 / format exit 1 / usage exit 2 / format exit 1）。

### S-VAL-04 垃圾内容示例换 v0.6 文法

- **Given** `garbage.post.md` 内容非法
- **When** 执行 `paperwork validate garbage.post.md`
- **Then** exit 1；`error format:`；example 为 v0.6 形态 `paperwork post send myfile --author alice --message "hello"`（与 validate.rs 实际输出逐字一致：example 用无后缀的 `myfile`，初稿误带 `.post.md`，按实现收口，fix-ledger A-02）。

## 8. 横切场景：路径解析（冻结回归）

ensure_suffix 三级解析行为逐条沿用 v0.5 bdd S-PATH-01~08（原路径优先 / 补后缀 / create 补后缀 / 两者皆无 not-found / x.md 与 x.post.md 并存用 x.md / send 自动创建落点 / 异型文件 format 不改道 / 目录不命中），仅其中命令示例换 v0.6 文法（如 `paperwork post send quick --author alice --message "Hey"`）。

## 9. 横切场景：输出模式与 ASCII 契约

### S-OUT-01 --json 成功（冻结）

- **When** 执行 `paperwork --json post send standup --author alice --message "hi"`
- **Then** stdout 单行 JSON，含 `status:"ok"`、`command:"post.send"`、`conclusion`、`seq`、`sender` 等既有 key；exit 0。

### S-OUT-02 --json 运行时错误（冻结）

- **Then** JSON 错误对象含 `status:"error"`、`category`、`command:"post.read"`、`exit_code:1`（行为沿用 v0.5 bdd S-OUT-02）。

### S-OUT-03 --json usage 错误

- **When** 执行 `paperwork --json post send x.post.md alice "hi"`（v0.5 位置文法）
- **Then** stdout 单行 JSON 错误对象，`category:"usage"`、含 `command` 与 v0.6 规范形态 `example`，`"exit_code":2`；进程 exit 2；--json 感知仍为 argv 扫描（机制冻结）。

### S-OUT-04 -q / --plain / --help / -V / 顶层缺子命令（冻结）

- **Then** 五场景行为与 v0.5 bdd S-OUT-04~07 逐条一致，含 `--help` 各层级与 `-V`（-q 隐首行留字段；--plain 原始字节；--help/-V exit 0 穿透不进信封；缺子命令 exit 2 且 command 填 `usage`）（rework 修正 Nora 观察 2：明确 --help 层级覆盖与 -V 均在冻结范围）。

### S-OUT-05 ASCII 输出契约（冻结防线）

- **Given** 任意 usage 错误与运行时错误场景
- **When** 捕获 stderr 原始字节
- **Then** 逐字节 `is_ascii` 全真（`ascii_output_contract_guard` 级别断言沿用 v0.5 修复轮确立的防线，覆盖 usage 信封与七类运行时错误两档）。

### S-OUT-06 --json 与 --plain 同给（usage，rework 补录 Pete N2-(5)）

- **When** 执行 `paperwork --json --plain post read standup.post.md`
- **Then** exit 2；stdout 单行 JSON 错误对象（argv 扫描感知 `--json`，机制冻结），`category:"usage"`（clap conflicts 判定），`"exit_code":2`；无文件写入。

## 10. 横切场景：别名（冻结回归）

### S-ALIAS-01 po 隐藏别名与既有别名

- **When** 执行 `paperwork po read standup.post.md` 与 `p show` / `b read` / `c read` / `v` 各形态
- **Then** 行为与 v0.5 一致；`po` 与 `p/b/c/v` 均不出现在 `--help` 命令列表。

## 11. 横切场景：短形式全表一致性

### S-SHORT-01 短形式与全称等价（逐 flag）

- **When** 对 spec §4 短形式全表中每个有短形式的 flag（rework 裁定 F3 收窄后精确为 `-a/--author`、`-m/--message`、`-q` 全局三项），分别以短形式与全称执行同一命令
- **Then** 两次行为逐字等价（exit 码、信封、文件效果一致）。

### S-SHORT-02 命名政策白名单断言（冻结，SOTA C6；本轮 additive 同步更新）

- **When** 检查 `--help` 输出
- **Then** 组集合精确等于 {profile,post,brief,contacts,validate}（隐藏别名不出现）；contacts 组动词集合精确等于 {create,add,remove,update,read}（本轮 additive：remove/update 新增，既有三动词不变；**断言面落点（rework 补录，Daniel M-3）**：现状 cli_integration 组级动词断言仅 post 组存在（`post_group_help_lists_verbs` 先例），contacts 组动词集合断言为本轮**新建**，仿该先例体例并含反向断言（不出现清单外动词），`update` 已随本轮经 owner CRUD 指令授权扩容纳入 SOTA C6 白名单，v0.7_feedbacks §2.5）；全 CLI flag 名集合与 spec §4 全表一致；短形式集合精确等于 {-a, -m, -q}（rework 裁定 F3，本轮不变）；spec §4「其余全部 flag」行枚举的全量清单逐一断言无短形式：`--seq`、`--stdin`、`--title`、`--to`、`--from`、`--entry`、`--entry-title`、`--profile`、`--new-profile`、`--name`、`--model`、`--description`、`--owner`、`--note`、`--regex`、`--scope-read/--scope-write/--scope-owns`（--scope-* 三 flag）、`--full`、`--limit`、`--base-dir`、`--type`、`--json`、`--plain`，另含 post read `--reply-to` 与 post read `--mention`（均无短形式；**2026-08-15 owner 裁决后口径收窄：写侧 send `--reply-to`/`--mention` 已撤销，post 侧补充项仅剩 read 侧两项，send 侧探针移除**（原「post send/read 两侧 --reply-to」表述废止）；**总数不写死，以下枚举为准**（分项口径：spec §4 全表逐 flag 负向断言 + post read 侧 `--reply-to`/`--mention` 两项补充；任务 #34 勘误：原「共 26 项」计数口径含糊，改为枚举口径；2026-08-15 裁决后枚举净减两项，原 26 项口径作废，以现行枚举逐项断言为准）；基线勘误删除 `--participants`，其余与 spec §4 枚举逐字对齐并保留原清单 --name；本轮 additive 新增 `--new-profile` 一项；rework 修订：原「共 25 项」为 stale 计数，Mark M-3/Ryan m-1/Daniel m-1 定案后以本枚举为准；**断言面落点（Daniel M-3）**：现状仅 6 个一次性负向探针（`-s/-l/-n/-t/-e/-p`），逐 flag 负向清单为本轮**新建/扩展**，非「追加」；后续短形式增删以本枚举为唯一对账口径，不再维护硬编码总数）（F3 收窄后本白名单断言重获完整防线意义）。

## 12. 横切场景：写路径锁（本轮新增，spec §3.9）

### S-LOCK-01 多进程并发写不丢失（contacts/brief）

- **Given** contacts 文件与 brief 文件均存在；**预创建 N 个互不相同的 entry 目标文件（与 N 个 brief 条目一一对应；brief add 须对 entry 目标文件做 SHA-256 快照，文件缺失即 io 错误 exit 1，rework 补录 Daniel m-2）**；contacts 侧 N 个 profile 路径可为不存在路径（add 不校验目标存在性，label 回退，不阻塞并发断言）
- **When** N 个独立进程并发执行 `paperwork contacts add team.contacts.md --profile p<i>.profile.md`（互不相同的 N 个 profile 路径），另一组 N 个进程并发执行 `brief add`（互不相同的条目，指向上述预创建文件）
- **Then** 全部 exit 0（锁阻塞串行化，无丢失、无 fast fail 误报）；contacts 文件条目集合 = N 个路径的并集；brief 条目集合 = N 个条目的并集；两文件均可被 `validate` 判定合法（格式未损坏）。

### S-LOCK-02 profile edit 并发串行化（rework 修订，Daniel M-1）

- **Given** `agents/alice.profile.md` 存在（model 与 description 各有旧值）
- **When** 两个进程并发执行 `paperwork profile edit agents/alice.profile.md --model X` 与 `paperwork profile edit agents/alice.profile.md --description "D"`（两字段**不重叠**）
- **Then** 两者均 exit 0（锁内串行）；最终文件为合法 profile（validate 通过），**终态为两次编辑的字段并集**：model 取 `X` 且 description 取 `D`（后一编辑经锁内读改写读到前一编辑的落盘结果再施加自身变更，无丢失写）；无交错损坏字节；禁止出现仅一侧生效的丢写终态。若采用「两进程改同一字段」变体，则断言最后写入者胜（两终态之一，以集合口径断言），二选一由测试实施方选定并在用例内写清；本场景主形态为不重叠字段并集断言。

### S-LOCK-03 fast fail 无降级防线（代码级不变量）

- **Given** 六写路径（contacts add/remove/update、brief add/remove、profile edit）的实现
- **When** `lock_exclusive` 获取失败（IO/锁错误）
- **Then** exit 1；`error io:`，fix 含 `another process may hold the lock; retry shortly` 语义；代码层不变量：**不存在任何无锁降级写入路径**（本条以 code review + 锁调用点位盘点断言为准，集成测试不强制模拟 OS 级锁失败；thread 写路径既有并发用例 S-SEND-14 冻结回归同批执行）。

备注（本节通用，rework 补录 Ryan M-2）：锁阻塞等待无内建超时（fs2 语义，命令可能长时间乃至无限期不返回），契约全文见 spec §3.9「agent 可见阻塞行为契约」；agent 编排侧可对 paperwork 进程施加进程级超时，超时后杀进程重试（幂等/先读后写路径重试安全）。
