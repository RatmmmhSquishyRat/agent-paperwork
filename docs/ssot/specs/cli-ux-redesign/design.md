# CLI UX 重设计 v0.5.0 — Design（设计方案）

- 日期：2026-08-09
- 版本：v0.5.0
- 文档性质：设计方案（每个 tool 一节独立完整设计 + 文法论证 + 遗留项裁决）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.5_feedbacks.md`（owner 指令，最高优先级）
  - `docs/ssot/adr/feedbacks/v0_feedbacks.md`、`docs/dev/adr-v1.md`（ADR-011）
  - `docs/researches/cli-ux-agent-visible-output-research-2026-08-08.md`
  - `docs/researches/ux-open-items-backlog-2026-08-08.md`
  - `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md`
  - `docs/reviews/v0.4-ux-review-2026-08-01.md`（旧提议，被 v0.5_feedbacks 覆盖处见 §6）

---

## 1. 设计总纲

### 1.1 对用户字面示例的采纳说明

owner 指令给出字面示例：`paperwork Post文件路径 使用者的名字 send {有关这个评论的所有参数详情}`。

**意图采纳**（100% 落实）：

- PATH 与 NAME 前置为必填位置参数，出现在命令行最靠前的位置；
- 「有关这个评论的所有参数详情」（reply-to / mention / body 等）全部后置；
- name 在发送消息时是必填项，不再允许以任何 flag 或环境变量形式缺省。

**字面文法（path-first：`paperwork <path> <name> send ...`）被否决**，动词保持在第 2 层（clap 分发器层级），即 `paperwork post send <PATH> <NAME> ...`。否决理由：

1. **后缀探测路由**：path-first 文法下，CLI 必须先读文件后缀才能知道该用哪套动词与参数 schema（`<path>` 可能是 profile/post/brief/contacts 任意一种），引入路由前 I/O——在无状态、任意路径操作的设计（ADR-011）下，路由失败还要再造一套错误类型；
2. **歧义面随文件类型平方增长**：裸 `foo.md`（无类型后缀、文件不存在时）无法判定归属哪类 tool，send/create/read 的参数 schema 互不相同，歧义组合随「文件类型 × 动词」增长；
3. **绕过 clap**：clap 的 derive 分发、help 生成、usage 错误信封全部失效，需手写分发器与 help 渲染——与本次新增的 usage 信封（错误即指导）直接冲突；
4. **破坏槽位一致性**：`profile list <DIR>`、`validate <PATH>` 等无文件实例的命令在 path-first 下无处安放。

path-first 字面文法保留为 v0.6 可选快捷前缀层提案（见 Rejected Alternatives）。

### 1.2 三条文法规则的论证

**规则 1（槽位）：PATH 恒第 1 必填；send/edit 的 NAME 第 2 必填；content 恒末位。**

- PATH 第 1：文件即接口（files as API），目标文件是全 CLI 的第一名词；与 ADR-011 路径显式一致；agent 读到第 1 个参数即知操作对象。
- NAME 第 2（仅 send/edit）：owner 指令 ② 直接落实；NAME 是这两条命令唯一的 actor 参数，紧跟 PATH 形成「对哪个文件、以谁的名义」的完整主语，其余细节全部后置。
- content 末位：v0_feedbacks #3.1（多行大片内容置末便于书写）；shell 中以 `-` 开头的正文可用 `--` 边界保护（位置参数天然支持，flag 值做不到同等自然）。

**规则 2（唯一语义）：全 CLI 任何 flag 只有一种含义。**

- v0.4 的 `--from` 在 send/edit 中=身份、在 read 中=seq 起点，是全 CLI 最严重的 UX 缺陷（U-01，ux-review §1，三次 review 反复提及）。改名（`--as`/`--seq-from`）只是缓解；身份位置参数化后 `--from` 的身份语义**结构性消亡**，`--from/--to` 仅存于 post read 且仅表 seq 范围——冲突不可能复发。
- 该规则同时约束未来：任何新 flag 若与既有 flag 同名即视为设计错误。

**规则 3（判据）：必填即位置参数，可选才做 flag。**

- 判据化后，`--title`/`--name`/`--entry`/`--entry-title`/`--profile`/`--seq` 等必填 flag 全部转位置参数不再需要逐个讨论；`contacts create --title` 因有默认值（可选）保留 flag——正是同一判据的另一面。
- 业界基准（agent-cli-ux-industry-sota-2026-08-08.md）中 git/docker/cargo 等成熟 CLI 均遵循同一判据：必填核心参数位置化，修饰参数 flag 化。

---

## 2. post — 独立完整设计（参数最复杂场景，设计重心）

### 2.1 动线与参数布局理由

post 的典型动线：**create（可选，send 可自动创建）→ send（高频）→ read/summary（高频）→ edit（低频纠错）**。

```
paperwork post create  <PATH> <TITLE> [--participants a,b]
paperwork post send    <PATH> <NAME> [BODY] [--stdin] [--reply-to N] [--mention a,b]
paperwork post read    <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]
paperwork post summary <PATH>
paperwork post edit    <PATH> <NAME> <SEQ> [NEW_BODY] [--stdin]
```

布局理由：

- **send 的参数序 `<PATH> <NAME> [BODY]`** 完全对应 owner 字面示例「Post文件路径 使用者的名字 … {评论参数详情}」的意图：前两位是每发必给的硬信息，reply-to/mention 是偶发修饰（flag），正文放最后（多行书写、`--` 保护 `-` 开头）。
- **edit 的 `<PATH> <NAME> <SEQ> [NEW_BODY]`**：SEQ 从 flag 转位置参数（必填，规则 3）；置于 NAME 之后、NEW_BODY 之前——SEQ 是寻址参数（选哪条），不是 content，content 必须让出末位给 NEW_BODY；u64 解析错误信号明确（非数字即 usage 错误），不会与 body 混淆。
- **read 完全不变**：`--from/--to` 在身份参数消失后成为全 CLI 唯一语义（规则 2 的收益兑现点）；仅做 additive 输出补全（§2.3）。
- **create 的 TITLE 位置化**：title 是 create 的必填主载荷；participants 可选保留 flag。
- 别名：新增隐藏别名 `po`（`paperwork po send ...`），缓解最高频组的输入成本；`p` 已属 profile，不挪用（N-01 裁决）。

### 2.2 错误指导样貌（example 全部刷新为新文法）

- 空正文（validation）：`example: paperwork post send standup.post.md alice "Hello"`
- 无正文且无 --stdin（validation）：`example: paperwork post send thread.post.md alice "Hello"`；该形态同时是 NAME/BODY 混淆面的恢复出口（§2.5）：message 提示「若你已给出正文，请检查是否遗漏 NAME 槽位（NAME 紧跟 PATH）」，fix 含 `--` 边界用法。
- 位置 body 与 --stdin 互斥（validation）：`example: paperwork post send thread.post.md alice --stdin`
- 线程不存在（not-found，read/summary/edit）：`fix: send a message first to create the thread` / `example: paperwork post send standup.post.md alice "Hello"`（example 一律具体可执行，禁用尖括号占位符，spec §4.2）。
- edit 三重护栏（not-allowed）：message 精确文本不变，example 换新文法。
- 旧文法调用（`--from alice`）：clap 报未知 flag → usage 信封（exit 2）+ 该命令的规范可执行示例（不携带用户原参数值，见 §2.6），一次重试即完成迁移。

### 2.3 输出增补（additive）

- **send**：发生 reply-to 隐式 mention 时，字段区增补**单数字段** `implicit-mention: <name>`（U-10）——reply-to 隐式 mention 至多触发一人（原消息发送者），故用单数；仅触发时输出，不触发则不出现该字段；JSON 同步增补同名 key。agent 由此可见副作用全貌，避免重复 `--mention`。
- **read**：字段区**恒显** `showing: <n>/<total>`（现状仅超限显示），并增补 `window: #<first>-#<last>`（按实际展示的第一条与最后一条 seq；空线程不显示 window）（U-11）；两字段均放字段区而非 conclusion 行（裁定，见 §7.4 第 3 条）；agent 首屏即知「这是窗口还是全量」，配合 seq 无间隙性质安全增量读取（`--from N+1`）。

### 2.4 help / after_help 示例文案

**语言裁定：全部英文**（裁定 8，见 §7.4）——与现状 help、纯 ASCII 输出层一致；下述全部 help/after_help 文案即为英文形态。

顶层 help 增加一行文法模板说明（与 spec §1.1 总纲逐字一致）：

```
Grammar: paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]
```

各子命令 after_help（可复制示例）：

```
# post send
Examples:
  paperwork post send standup.post.md alice "Parser module is 80% done."
  paperwork post send standup alice --reply-to 2 --mention bob "Tests merged."
  echo "multi-line body" | paperwork post send standup.post.md alice --stdin
  paperwork post send standup.post.md alice -- "-fix flag text"

# post edit
Examples:
  paperwork post edit standup.post.md alice 3 "corrected body"
  paperwork post edit standup.post.md alice 3 -- "-starts with dash"

# post read
Examples:
  paperwork post read standup.post.md --from 5 --to 20
  paperwork post read standup.post.md --mention alice --limit 20

# post create
Examples:
  paperwork post create standup "Daily Standup" --participants alice,bob
```

（send/edit 各含一条 `--` 边界示例；post read 的 seq 范围示例为 --from/--to 新唯一语义的正面教学。）

### 2.5 NAME/BODY 混淆面裁定（rework 轮，2026-08-09）

`post send <PATH> "text"`（PATH + 单字符串）中该字符串必然被 clap 绑定到必填 NAME 槽、BODY 缺省，落 **validation exit 1**；CLI 无法区分「漏 NAME」与「给了 NAME 缺 body」——这是位置文法的**固有代价与已知边界**，不可实现为 usage exit 2（三份对抗评审共识，F1 裁定）。

该混淆面的恢复成本靠**三重教学补偿**压到最低：

1. **validation 错误 example**：无正文错误的 example 展示含 NAME 槽的完整命令形态，message 显式提示「若你已给出正文，请检查是否遗漏 NAME 槽位」；
2. **after_help 示例**：post send 的可复制示例全部呈现 `PATH NAME BODY` 三槽完整形态；
3. **SKILL.md**：新文法速查把「send 的前两位必填为 PATH 与 NAME」作为首要提示条目。

### 2.6 usage 信封 example 生成策略裁定（rework 轮，2026-08-09）

否决「argv 值迁移重建逐字修正命令」方案（clap try_parse 错误对象不携带重建所需信息，自建旧文法迁移层工作量与回归面失控，两评审 Major 共识，F2 裁定）：降级为**静态规范示例**——usage 信封输出该命令的**规范 usage 行 + 一条预置可执行示例**，不携带用户原参数值。旧文法迁移教学改由「规范示例 + SKILL.md + after_help」三件套承担；BDD 中旧文法场景（S-SEND-09/S-EDIT-04/S-PROF-03/S-BRIEF-03/S-CONTACTS-03）断言相应降级为「含该命令规范形态示例」。

---

## 3. profile — 独立完整设计

### 3.1 动线与参数布局理由

动线：**create（一次）→ show/edit（低频）→ list（发现队友）**。

```
paperwork profile create <PATH> <NAME> [--model] [--description] [--scope-read/write/owns]
paperwork profile show   <PATH>
paperwork profile edit   <PATH> [--model] [--description] [--scope-*]
paperwork profile list   <DIR>
```

- **NAME 位置化（U-06）**：`--name` 本就是必填，按规则 3 转位置参数；紧跟 PATH 消除「名字出现在路径中间」的冗余感。NAME 与路径名不强制一致（保留 `agents/alice-v2 alice` 这类覆盖场景的自由），CLI 不做派生猜测——显式优于隐式。
- **show/edit/list 不变**：无 actor 命令一律单位置参数（规则 1 后半），零特例；edit 的字段全是可选修饰，全部保留 flag；list 容错式输出（`(unreadable)` 不中断）保留。

### 3.2 错误指导样貌

- 重复 create（already-exists）：example 换新文法 `paperwork profile create agents/alice.profile.md alice --model gpt-4o`。
- show 文件不存在（not-found）：`example: paperwork profile create agents/alice alice`。
- 缺 NAME（usage，exit 2）：`example: paperwork profile create agents/alice alice --model gpt-4o`。

### 3.3 help / after_help 示例文案

```
# profile create
Examples:
  paperwork profile create agents/alice alice --model gpt-4o --description "Parser implementer"
  paperwork profile create agents/alice alice --scope-read "src/**" --scope-write "src/parser/**"
```

---

## 4. brief — 独立完整设计

### 4.1 动线与参数布局理由

动线：**create（一次）→ add（累积条目）→ read（取清单）→ verify（保鲜检查）→ remove（清理）**。

```
paperwork brief create <PATH> <TITLE> [--owner] [--description]
paperwork brief add    <PATH> <ENTRY> [--regex] [--note]
paperwork brief remove <PATH> <ENTRY-TITLE>
paperwork brief read   <PATH> [--full]
paperwork brief verify <PATH> [--base-dir <DIR>]
```

- **TITLE / ENTRY / ENTRY-TITLE 位置化**：三者均为各自命令的必填主载荷（规则 3，U-07 对 add 的直接落实）；owner/description/regex/note 是可选修饰，保留 flag。
- **read/verify 不变**：只读命令单位置参数；verify 的 `--base-dir` 可选保留；三态判定（fresh/shifted/stale）与 conclusion `N/M fresh` 不动——这是 agent 知识保鲜的核心契约。

### 4.2 错误指导样貌

- brief 不存在（not-found）：`example: paperwork brief create notes "Reading List" --owner alice`。
- 条目不存在（remove，not-found）：`example: paperwork brief remove notes.brief.md "main.rs"`。
- 缺 ENTRY（usage，exit 2）：`example: paperwork brief add notes.brief.md src/main.rs --regex "fn main"`。

### 4.3 help / after_help 示例文案

```
Examples:
  paperwork brief create onboarding "Codebase Onboarding" --owner alice
  paperwork brief add onboarding.brief.md src/main.rs --regex "fn main" --note "Entry point"
  paperwork brief remove onboarding.brief.md main.rs
  paperwork brief verify onboarding.brief.md
```

---

## 5. contacts — 独立完整设计

### 5.1 动线与参数布局理由

动线：**create（一次）→ add（登记队友）→ read（发现队友 + 身份摘要）**。

```
paperwork contacts create <PATH> [--title]
paperwork contacts add    <PATH> <PROFILE-PATH>
paperwork contacts read   <PATH>
```

- **`--profile` → 位置参数 PROFILE-PATH（U-07）**：被添加的 profile 路径是 add 的唯一主载荷，必填即位置参数。
- **create 的 `--title` 保留 flag**：有默认值 `Contacts`，属可选——规则 3 的标准应用，也是全表唯一的「title 不位置化」命令，判据一致性本身就是最好的文档。
- **read 富化输出不变**：「路径 + name + description」一次调用得到「团队有谁 + 各自是谁」（F-04）。

### 5.2 错误指导样貌

- contacts 文件不存在（not-found）：`example: paperwork contacts create team --title "Core Team"`。
- profile 路径不可读（add 失败）：example 换新文法 `paperwork contacts add team.contacts.md agents/alice.profile.md`。
- 缺 PROFILE-PATH（usage，exit 2）：`example: paperwork contacts add team.contacts.md agents/alice.profile.md`。

### 5.3 help / after_help 示例文案

```
Examples:
  paperwork contacts create team --title "Core Team"
  paperwork contacts add team.contacts.md agents/alice.profile.md
  paperwork contacts read team.contacts.md
Note: title is an OPTIONAL flag here (default "Contacts"), unlike post/brief create where it is a positional argument.
```

---

## 6. validate — 独立完整设计

### 6.1 动线与参数布局理由

动线：**怀疑文件损坏 → validate → exit 0 通过 / 错误信封自修复**。

```
paperwork validate <PATH> [--type post|profile|brief|contacts]
```

- PATH 单位置参数不变（格式防火墙入口）。
- **新增 `--type`（U-15，additive）**：无类型后缀或后缀与内容不符时，agent 可显式指定解析器而非被后缀卡死；默认仍按后缀推断，行为完全后向兼容。
- validate 不参与 ensure_suffix 补后缀（它是诊断工具，应作用于 agent 给出的确切路径）；「原路径优先」解析（§8 附带变更）对其无副作用。

### 6.2 错误指导样貌

- 未知后缀且未给 `--type`（format）：`fix: file must end with .post.md/.profile.md/.brief.md/.contacts.md, or pass --type` / `example: paperwork validate myfile.md --type post`。
- 垃圾内容（format）：fix/example 保留教学价值，example 换新文法：`paperwork post send myfile alice "hello"`。

### 6.3 help / after_help 示例文案

```
Examples:
  paperwork validate standup.post.md
  paperwork validate mystery.md --type post
```

---

## 7. 遗留项裁决总表（一次性全部结案）

### 7.1 本次解决

| 编号 | 裁决 | 一句理由 |
|---|---|---|
| U-01（--from 双语义） | 身份改前置位置参数 NAME | 位置参数化使身份语义从 flag 层消失，冲突结构性消解（规则 2） |
| U-06（profile --name 冗余 flag） | 并入名字位置参数化 | 必填即位置参数（规则 3），owner 指令直接要求 |
| U-07（主载荷非位置参数） | brief add ENTRY / contacts add PROFILE-PATH 位置化 | 必填主载荷不应与可选修饰混居 flag 层（规则 3） |
| U-08 + N-01（别名） | 补 post 隐藏别名 `po`；`p/b/c/v` 不变；不引入子命令别名 | `p` 已属 profile 不可挪用；子命令别名收益低、help 表面积成本高 |
| U-10（隐式 mention 不可见） | send 输出增补 implicit-mention 单数字段 | 隐式 mention 至多一人，单数字段仅触发时出现，显式化副作用避免重复 mention（additive）；三种不触发边界（自回复/已显式 mention/reply-to 不存在）见 spec §3.1 |
| U-11（read 无窗口指示） | 恒显 showing n/total + window 字段（字段区形态） | agent 必须首屏分辨「窗口 vs 全量」才能安全增量读取（additive） |
| U-14 + N-02（后缀改写） | ensure_suffix 改三级解析（原路径→补后缀路径→以补后缀路径为落点） | 消除「恰好存在的 x.md 被改写为 x.post.md 后报 not-found」的陷阱；第①级判据 is_file()，第③级为路径决策语义（物理创建仅限写命令） |
| U-15（validate --type） | 新增可选 --type flag | 后缀推断卡死时的逃生门，additive 无兼容成本 |

### 7.2 裁决拒绝

| 编号 | 裁决 | 一句理由 |
|---|---|---|
| U-02（PAPERWORK_AGENT env 回退） | 拒绝 | 违背显式原则，且与 owner「name 必填」指令直接冲突 |
| U-05（内容优先/路径可省略） | 拒绝 | 与 owner「PATH 前置必填」指令直接冲突（ux-review §5 被 v0.5_feedbacks 覆盖） |
| R-08（--no-color / NO_COLOR） | 拒绝 | v0.4 输出已纯 ASCII 无 ANSI 码，实质影响已消失 |
| F-09（正文 markdown 校验） | 拒绝（接受现状） | 正文按四反引号围栏透传是刻意设计，validate 只保证托管文件结构 |

### 7.3 裁决延后（单独立项，写入未来工作）

| 编号 | 裁决 | 一句理由 |
|---|---|---|
| U-03 + R-01 + N-03（线程创建双轨、系统消息占 #1） | 延后 | 属文件格式层变更、波及存量文件，本次纯 CLI 文法重设计不触碰。**对 backlog「本次必须解决」口径的正面回应**：backlog 属研究文档建议而非 owner 指令（v0.5_feedbacks 未要求解决 U-03）；格式层变更需独立 spec（波及存量文件迁移），本次文法重设计范围不含，列为 **v0.6 候选**单独立项（见未来工作） |
| U-04（正文 @mention 自动提取） | 延后 | 新增隐式解析行为，违背显式原则，需单独设计正文解析边界 |
| U-09（summary 并入 read） | 延后（保留独立） | summary 语义独立、可发现性好，ux-review 自评亦可接受现状 |
| U-13（shell completions） | 延后 | 仅人类用户受益，与 agent-first 目标错位 |

### 7.4 规格模糊点裁定（2026-08-09）

文档初稿向编排层上报的 8 个规格模糊点，由编排层统一裁定如下（已逐字落实到 spec/bdd/tdd/impl_plan）：

| # | 模糊点 | 裁定 |
|---|---|---|
| 1 | usage 错误 category 归属 | 定为第七类 `usage`（category 词表为冻结枚举、仅可经评审流程扩展，本次扩展已经本次对抗评审确认）。退出码 2；usage 专指 clap 层用法错误，与运行时六类（exit 1）分层 |
| 2 | implicit-mention 字段形态 | reply-to 隐式 mention 至多触发一人（原消息发送者），故采用**单数字段** `implicit-mention: <name>`，仅在触发时出现（additive，不触发则不输出），默认档与 --json 一致 |
| 3 | read 窗口字段形态 | 采用字段区形态（不放 conclusion 行）：**恒显** `showing: n/total`（即使未超限），并增补 `window: #<first>-#<last>`（按实际展示的第一条与最后一条 seq；空线程不显示 window） |
| 4 | --json usage 错误的 exit_code 值 | JSON 内 `exit_code` 如实反映进程退出码：usage 错误填 **2**（运行时错误仍为 1） |
| 5 | 顶层解析失败的 command 标识 | 组/动词层失败导致无法确定命令时，信封与 JSON 的 command 标识统一填 **`usage`**（如 `error usage: missing subcommand`，JSON `"command":"usage"`） |
| 6 | core example 处数矛盾 | 消除「13 处」与行号清单的数字矛盾。**rework 轮已实测盘净：全仓 14 处**（thread 6 + manifest 5 + contacts 1 + profile 2，含三份评审补出的 manifest.rs L80/L151/L194），完整清单与检索命令见 impl_plan 步骤④，不再采用「实施时检索为准」延迟兜底 |
| 7 | ensure_suffix 完整语义 | 三级解析：① 传入路径原样存在（is_file()）→ 用原路径；② 否则，补类型后缀后的路径存在 → 用补后缀路径；③ 都不存在 → 以补后缀路径为操作落点（物理创建仅限写命令，只读命令报 not-found；post send 自动建线程落点即此）。spec/bdd 已补对应场景（含「x.md 与 x.post.md 同时存在时用 x.md」用例） |
| 8 | help 语言 | 全部英文（与现状 help、纯 ASCII 输出层一致）；design.md 各 after_help 示例文案确认为英文 |

### 7.5 对抗评审 rework 轮裁定表（2026-08-09）

三份对抗评审（SSOT/pillars、agent-ux、feasibility）后，编排层对设计类争议的统一裁定（已逐条落实到 spec/bdd/tdd/impl_plan）：

| # | 争议点 | 裁定 |
|---|---|---|
| F1 | NAME/BODY 混淆面（三评审共识 Critical） | `post send <PATH> "text"` 落 validation exit 1，不可实现 usage exit 2（位置文法固有代价）；S-SEND-08 改仅 PATH 形态，另立场景 S-SEND-12；混淆面靠三重教学补偿（§2.5） |
| F2 | usage 信封 example 生成（两评审 Major） | 否决值迁移重建算法，降级为静态规范示例（§2.6）；相关 BDD 断言降级为「含规范形态示例」 |
| F3 | showing total 口径 | total = 过滤后、limit 截断前的条数（与 v0.4 现状一致）；window 取实际展示首末 seq（spec §3.1） |
| F4 | ensure_suffix 完整语义修订 | 第①级判据 is_file()（目录不命中）；第③级为「路径决策」语义（物理创建仅限写命令，只读命令报 not-found）；补异型文件场景（第①级命中 → format 错误） |
| F5 | try_parse 穿透条款 | --help/-V（DisplayHelp/DisplayVersion）按 clap 原样输出、exit 0，不进 usage 信封 |
| F6 | 门禁分阶段化 | 步骤①-④门禁 = cargo build + paperwork-core 测试 + clippy 全绿（cli_integration 允许红）；步骤⑤后 workspace 全绿才是硬门禁 |
| F7 | example 占位符策略统一 | 所有 example 一律具体可复制执行，禁用尖括号占位符（遵守 SOTA 结论 10），BDD 两处矛盾按此统一 |

**SOTA 报告未竟项的采纳/拒绝记录**：C5 后半（--help --json 机器可读内省）拒绝——与 agent-first 定位错位且增加 help 表面积，补偿由 SKILL.md 承担；C6（命名政策测试强制：动词/flag 白名单断言）采纳——已纳入 tdd §3 新增用例（成本极低，复用既有精确断言模式）。

---

## 8. Rejected Alternatives

1. **path-first 字面文法**（`paperwork <path> <name> send`）：需后缀探测才能路由动词，引入路由前 I/O；裸 `foo.md` 歧义；profile list/validate 等破坏槽位一致性；绕过 clap 需手写分发与 help。保留为 v0.6 可选快捷前缀层提案。
2. **隐藏弃用别名窗口**（Eric 案：旧 `--from` 等保留一版过渡）：双文法使 help/测试/错误示例表面积翻倍，位置参数无法别名导致兼容矩阵残缺；usage 信封一次重试即完成迁移，别名收益不抵负债。
3. **`--as` flag 方案**（ux-review §1 原提议）：owner 指令明确要求名字为前置必填位置参数，flag 形态不满足。
4. **SEQ 保留 flag**（Eric 案：`edit --seq N`）：与规则 3 冲突；u64 解析错误信号明确，`--` 可防 `-` 开头 body 歧义。
5. **`--seq-from/--seq-to` 改名**（Sam 案）：身份语义消失后 --from 已无冲突，改名徒增迁移成本。
6. **usage 信封 argv 值迁移重建逐字修正命令**（rework 轮否决，F2）：clap try_parse 错误对象不携带重建信息；自建旧文法迁移层（识别 --from/--seq/--name 等并映射回位置槽）需处理多旧 flag 并存、flag 值缺失、与 `--` 边界混用等边界，工作量与回归面失控；降级为静态规范示例（§2.6）。

---

## 9. 兼容策略

- **v0.5.0 干净切断，不留隐藏弃用别名**。依据：0.x 惯例（项目 0.2/0.3/0.4 连续破坏性迭代先例）；双文法是维护噩梦；owner 为唯一真实消费方。
- **迁移教学由 usage 信封承担**：任何旧文法调用（`--from alice`、`--title X`、`--entry P` 等）因 flag 已不存在而落入 usage 信封（exit 2），信封内的规范可执行示例（§2.6 静态规范示例裁定）即为新文法形态——「错误即指导」机制从格式纠错扩展为文法迁移。
- **SKILL.md 与 usage 信封、help 示例共同构成迁移教学三件套**：v0.5.0 随仓库新增英文 SKILL.md（新文法速查 + 每个 tool 典型调用示例 + 错误自愈提示，见 impl_plan.md 步骤⑦）；依据 `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` **结论 C5 与 §8 风险 1（对冲三件套）**，随仓库发布 SKILL.md 对前置位置文法的首次误调率有显著补偿作用。
- 输出协议冻结（spec §6）保证：**信封结构与既有 key 零变更，但本版含四项消费者可感知变化**：① `showing` 由「仅超限出现」改为恒显（出现语义变化，非纯 additive）；② 新增退出码 2；③ category 词表扩为七类（usage）；④ 新增 implicit-mention / window / 错误 JSON command 三个字段。以上四项须在 CHANGELOG `Changed (Breaking)` 逐项列出并附消费者迁移说明（spec §4.6、impl_plan 步骤⑦）。
- CHANGELOG `## [0.5.0]` 以 `Changed (Breaking)` + 新旧文法迁移对照表形式先于 tag 落盘（release.yml awk 硬依赖），根 README 与 cli README 示例同步刷新（见 impl_plan.md 步骤⑦）。
