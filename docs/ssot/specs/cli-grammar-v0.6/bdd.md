# CLI 文法 v0.6: BDD（行为场景）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布）
- 文档性质：行为规范的行为化表述（Given/When/Then），覆盖全部命令的正常路径与错误路径
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令）
  - `docs/researches/cli-grammar-v06-reassessment-2026-08-09.md`（混淆面矩阵与错误等级标注）
  - `docs/dev/adr-v1.md`（ADR-011）、`docs/ssot/adr/feedbacks/v0_feedbacks.md`
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

### S-SEND-04 reply-to 隐式 mention 显式化（冻结回归）

- **Given** 线程中 #2 为 alice 所发
- **When** 执行 `paperwork post send standup.post.md --author bob --message "Sure" --reply-to 2`
- **Then** exit 0；字段区含 `implicit-mention: alice`（单数字段、仅触发时出现、三种不触发边界均沿用 v0.5 bdd S-SEND-10b/S-SEND-11）；`--json` 模式含同名 key。

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

### S-SEND-20 --reply-to/--mention 糖衣 flag token 注入（基线勘误后新增，format-v2 D2/OQ-4 行为补齐）

- **Given** `newtopic.post.md` 不存在
- **When** 首次执行 `paperwork post send newtopic.post.md --author alice --message "first"`，再执行 `paperwork post send newtopic.post.md --author bob --message "agreed" --reply-to 1 --mention carol`
- **Then** 两次均 exit 0；第二条消息落盘正文首行为 `@#1 @carol`，空行后接原正文（引用状态仅存于正文文本，D2）；`post read --reply-to 1` 与 `--mention carol` 过滤均命中第二条（读取时派生）；preamble 无 `- participants:` 行（D1）；全文件无 `--to`/`--participants` 属性行残留。

### S-SEND-21 首次 send 建线程 preamble 仅 H1 标题（基线勘误后新增，format-v2 D1/OQ-1 行为补齐）

- **Given** `daily.post.md` 不存在
- **When** 执行 `paperwork post send daily.post.md --author alice --message "hi" --title "Daily Standup"`
- **Then** exit 0；落盘文件以 `# Daily Standup` H1 行起始，其后直接是 `## #1 alice (...)` 消息头（无占位创建消息、无属性行）；缺 `--title` 时标题取路径剥 `.post.md` 后缀（spec §3.1 糖衣表）。

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
- **Then** exit 0；`showing: 0/4`（total 为过滤前全量口径，冻结语义）；`window` 字段不显示（空 window 冻结行为，与 v0.5 bdd S-READ-06 一致）；body 为空。

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

## 7. validate（冻结回归，示例换 v0.6 文法）

### S-VAL-01 按后缀推断成功 / S-VAL-02 --type 覆盖后缀 / S-VAL-03 未知后缀 / S-VAL-05 --type 非法值 / S-VAL-06 --type 与后缀交叉

- **Then** 五场景行为与 v0.5 bdd S-VAL-01~06 逐条一致（exit 0 / exit 0 / format exit 1 / usage exit 2 / format exit 1）。

### S-VAL-04 垃圾内容示例换 v0.6 文法

- **Given** `garbage.post.md` 内容非法
- **When** 执行 `paperwork validate garbage.post.md`
- **Then** exit 1；`error format:`；example 为 v0.6 形态 `paperwork post send myfile.post.md --author alice --message "hello"`。

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

### S-SHORT-02 命名政策白名单断言（冻结，SOTA C6）

- **When** 检查 `--help` 输出
- **Then** 组/动词集合精确等于 {profile,post,brief,contacts,validate}（隐藏别名不出现）；全 CLI flag 名集合与 spec §4 全表一致；短形式集合精确等于 {-a, -m, -q}（rework 裁定 F3）；spec §4「其余全部 flag」行枚举的全量清单逐一断言无短形式：`--seq`、`--stdin`、`--title`、`--to`、`--from`、`--entry`、`--entry-title`、`--profile`、`--name`、`--model`、`--description`、`--owner`、`--note`、`--regex`、`--scope-read/--scope-write/--scope-owns`（--scope-* 三 flag）、`--full`、`--limit`、`--base-dir`、`--type`、`--json`、`--plain`，另含 post send/read 两侧 `--reply-to` 与 post read `--mention`（均无短形式，共 24 项；基线勘误删除 `--participants`，其余与 spec §4 枚举逐字对齐并保留原清单 --name）（F3 收窄后本白名单断言重获完整防线意义）。
