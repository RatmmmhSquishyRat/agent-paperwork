# CLI 文法 v0.6: Spec（行为规范）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布，见 §7）
- 本轮增量（2026-08-09，v0.7 feedback 轮）：contacts remove/update 新增动词、brief read 新增 `--entry-title`、写路径锁统一（additive 扩展，见 §7 第 5 条）；rework 补录：SOTA C6 动词白名单扩容登记（`update` 经 owner CRUD 指令授权纳入，v0.7_feedbacks §2.5）、update/edit 语义分工（§3.6）、锁阻塞 agent 可见行为契约（§3.9）
- 文档性质：行为规范（命令契约 + 输出协议），实现与测试的唯一验收基准
- 增量修订（2026-08-15，owner 四项裁决轮，任务 #35 文档落盘，实施归任务 #36）：撤销 post send 写侧糖衣 flag `--reply-to`/`--mention`（写命令传入落 usage exit 2，reply/mention 语义由 agent 正文直书 `@#N`/`@name` 表达）；post read 读侧过滤器 `--mention`/`--reply-to` 保留声明；contacts add/update 新增非阻塞 destination advisory 校验契约（ok 信封 `advisory` 字段，只增不改协议）。裁决原文逐字与解释口径（编排层拟定，供 owner 复核推翻）见 docs/dev/owner-rulings-2026-08-15.md
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令，最高优先级；与 v0.5 文档冲突处以该文件为准）
  - `docs/ssot/adr/feedbacks/v0.7_feedbacks.md`（本轮 owner 指令：contacts 完整 CRUD + 写路径锁统一 + 渐进阅读补齐；本轮增量的最高优先级输入）
  - `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md`（本轮调研报告：动词盘点、锁缺口、CRUD 形态分析）
  - `docs/researches/cli-grammar-v06-reassessment-2026-08-09.md`（三视角重评估与 path-first 复评结论）
  - `docs/ssot/specs/cli-ux-redesign/`（v0.5 文档集：输出协议、ensure_suffix、usage 信封、别名、validate --type 等继承基线）
  - `docs/ssot/adr/feedbacks/v0_feedbacks.md`、`docs/dev/adr-v1.md`（ADR-011）
- 实现基线（基线勘误后，见 design.md 基线勘误记录）：cli-ux-v0.5 分支（v0.5.0 发布形态：旧文件格式 + v0.5 位置文法）与 master 的 format-v2 分支（post create 删除、send 增 `--title`、core v2 格式；`--to`/`--participants` flag 已按 owner 追裁 D1/D2 删除，未随 0.5.0 发布）的合并结果。

---

## 1. 核心文法（总纲与三条规则，替换 v0.5 三规则）

### 1.1 总纲

```
paperwork [全局flag] <组> <动词> <PATH> --必填具名flag [--可选修饰flag]
```

action-first 槽位顺序（组、动词先于路径）经 owner 显式裁决保留（v0.6_feedbacks §一 (1)）；path-first 两形态维持否决（design.md §4）。必填 flag 段移出方括号（rework 裁定：方括号仅留给可选修饰，避免 agent 误判必填为可选，Pete N6）。

### 1.2 规则 1（位置参数仅剩 PATH）

全 CLI 任何命令只保留**一个**位置参数 `<PATH>`（目标文件路径，恒必填、恒第一位）。v0.5 规则 1 中「NAME 第 2 必填位置参数」「content 恒末位」废止；v0_feedbacks #3.1 的字面条款（content 位置参数末位）被 owner 显式翻转（v0.6_feedbacks §三），多行大片内容改由 `--stdin` 通道承接。

### 1.3 规则 2（必填与可选一律具名 flag）

NAME、BODY、SEQ、TITLE、ENTRY、ENTRY-TITLE、PROFILE-PATH 等全部必填参数均为具名必填 flag，不占位置槽；可选修饰保持 flag。v0.5 规则 3（「必填即位置参数」判据）废止。必填 flag 缺省落入 usage 信封（exit 2），example 展示该命令完整必填形态。

### 1.4 规则 3（flag 唯一语义）

同一命令内任何 flag 只有一种含义。基线勘误后（format-v2 owner 追裁 D1/D2 删除 send 的 `--to`/`--participants`），全 CLI flag 恢复唯一语义的干净表述：`--from`/`--to` 仅存于 post read，仅表 seq 起点/上限（u64）；`--author` 恒指署名身份（仅 post send / post edit）；`--message` 恒指写入正文（仅 post send / post edit）。

裁定（规则 3 边界；2026-08-15 owner 裁决更新，原 v0.5 裁定废止 send 侧）：`--mention/--reply-to` 的**写侧语义（send「设置」）已撤销**——post send 传入该两 flag 落 usage exit 2（owner 裁决：「reply, mention 等等语义都在 markdown 消息本身中负责表达, cli 部分不应该给出这种参数用法」）；**read 侧「过滤」语义保留**——查询过滤属同一语义对象的只读延伸，不构成双语义。读侧保留口径显式声明于 §2/§3.3/§10（读侧过滤器是查询而非语义表达，不在撤销范围）。

---

## 2. 新文法全表（命令签名契约）

约定：`<PATH>` = 唯一位置参数（必填）；`--x` = flag；`(--a | --b)` = 二选一必填；`[--x]` = 可选。相对 v0.5 的变更在第三列标注。

| 命令 | v0.6 签名 | 相对 v0.5 变更 / 本轮增量标注 |
|---|---|---|
| post send | `post send <PATH> --author <NAME> (--message <BODY> \| --stdin) [--title T]` | NAME/BODY 位置参数 -> `--author/-a`、`--message/-m` 具名必填（owner 裁决）；`--title` 为 format-v2 建线程载荷；**2026-08-15 owner 裁决撤销：写侧糖衣 flag `--reply-to`/`--mention` 删除，传入落 usage exit 2；reply/mention 语义由 agent 正文直书 `@#N`/`@name` token 表达（读取时 derive 机制不变）** |
| post edit | `post edit <PATH> --author <NAME> --seq <N> (--message <NEW_BODY> \| --stdin)` | NAME、SEQ、NEW_BODY 位置参数 -> `--author/-a`、`--seq`、`--message/-m` 具名必填；edit 本就无 `--reply-to`/`--mention`（v0.6 既成事实），2026-08-15 撤销口径按「写命令」外延一并声明 |
| post read | `post read <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]` | 不变；`--mention` 无短形式（避免 -m 双义）；**2026-08-15 owner 裁决保留声明：读侧 `--mention`/`--reply-to` 是查询过滤器而非语义表达，不在写侧撤销范围，冻结保留（§3.3/§10）** |
| post summary | `post summary <PATH>` | 不变 |
| profile create | `profile create <PATH> --name <NAME> [--model] [--description] [--scope-read/write/owns]` | NAME 位置参数 -> `--name` 必填 flag（回到具名形态；v0.6_feedbacks §2.4 补记） |
| profile show/edit/list | `<PATH>` / `<PATH> [--field..]` / `<DIR>` | 不变 |
| brief create | `brief create <PATH> --title <T> [--owner] [--description]` | TITLE 位置参数 -> `--title` 必填 flag |
| brief add | `brief add <PATH> --entry <E> [--regex <PATTERN>] [--note <TEXT>]` | ENTRY 位置参数 -> `--entry` 必填 flag（勘误：--regex/--note 为带值 flag，QA BUG-4） |
| brief remove | `brief remove <PATH> --entry-title <T>` | ENTRY-TITLE 位置参数 -> `--entry-title` 必填 flag |
| brief read | `brief read <PATH> [--full] [--entry-title <T>]` | **本轮新增**：`--entry-title` 选择性详情（复用 brief remove 的 `--entry-title` 键语义，§3.5） |
| brief verify | `brief verify <PATH> [--base-dir]` | 不变 |
| contacts create | `contacts create <PATH> [--title]` | 不变（title 有默认值，属可选） |
| contacts add | `contacts add <PATH> --profile <P>` | PROFILE-PATH 位置参数 -> `--profile` 必填 flag；本轮补锁（写路径锁统一，§3.9） |
| contacts remove | `contacts remove <PATH> --profile <P>` | **本轮新增**（CRUD 补齐）；键 = profile 路径；未命中 not-found exit 1 |
| contacts update | `contacts update <PATH> --profile <OLD> --new-profile <NEW>` | **本轮新增**（CRUD 补齐）；OLD 未命中 not-found、NEW 已存在 already-exists、label 依 R11 重派生、条目顺序保留、不支持改 label |
| contacts read | `contacts read <PATH>` | 不变 |
| validate | `validate <PATH> [--type post\|profile\|brief\|contacts]` | 不变 |

注：v0.5 的 `post create` 已被 format-v2 删除（建线程职责由 send 自动创建承担，`--title` 为建线程时的线程元数据载荷），v0.6 不恢复；format-v2 同批按 owner 追裁 D1/D2 删除了 send 的 `--to`/`--participants` flag，v0.6 同样不恢复。

注（2026-08-15 owner 裁决）：post send 的写侧糖衣 flag `--reply-to`/`--mention` 撤销（format-v2 D2/OQ-4 的「糖衣注入」参数面废止）；reply/mention 语义改由 agent 在正文直接书写 `@#N`/`@name` token 表达，读取时 derive 机制不变（§3.1/§3.3）。撤销与读侧保留的边界口径见 §10。

本轮增量（v0.7 feedback 轮）：contacts remove/update 两动词与 brief read `--entry-title` 为 additive 新增；不涉及任何既有动词/flag 的删除或改名（v0.7_feedbacks §四）。**SOTA C6 白名单扩容登记（rework 补录）**：`update` 不在 SOTA C6 既定动词白名单内（`docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` L196，既定 10 动词），本轮经 owner CRUD 指令（v0.7_feedbacks §一 (1)）授权扩容纳入（裁定记录 v0.7_feedbacks §2.5）；`update` 与既有 `edit` 的语义分工见 §3.6（edit = 文件自身内容就地修改；contacts update = 条目 destination 换绑，contacts 组无 edit 动词）。

---

## 3. 逐命令契约与错误 category 映射

约定：默认值语义沿用 v0.5 现状，本版本不改默认值。错误 category 与退出码映射：clap 层用法错误（缺必填 flag、未知 flag、互斥冲突、类型错误、多余位置参数）一律 usage exit 2；运行时六类（format/validation/io/not-found/already-exists/not-allowed）exit 1（v0.5 spec §4，继承不变）。

### 3.1 post send

```
paperwork post send <PATH> --author <NAME> (--message <BODY> | --stdin) [--title T]
```

| 参数 | 形态 | 必填 | 说明 |
|---|---|---|---|
| `PATH` | 位置参数 | 是 | 线程文件路径；经 ensure_suffix 三级解析（v0.5 spec §5，继承不变） |
| `--author/-a` | 具名 flag | 是 | 发送者名字（署名）；语义与校验沿用 v0.5 NAME（trim 后为空拒绝 validation；**单 token 校验：拒绝空格/制表符/换行与括号**，违规 validation exit 1——以实现 `validate_sender` 为准，spec format §5.6；初稿「可含空格」为文档与实现不一致，按实现收口，fix-ledger D7）；不与 profile/contacts 做存在性校验；不设 `allow_hyphen_values`（名字值无 `-` 开头合法形态，设置反扩大误解析面，F4 复核结论） |
| `--message/-m` | 具名 flag | 与 `--stdin` 二选一 | 消息正文；flag 值直传，以 `-` 开头的正文无需 `--` 边界；clap 属性 `allow_hyphen_values = true`（rework 裁定 F4，impl_plan 步骤(2) 硬性指令） |
| `--stdin` | 开关 flag | 与 `--message` 二选一 | 从 stdin 读正文（多行大片内容首选通道） |
| `--title` | 具名 flag | 否 | 建线程载荷：线程标题（preamble 仅 H1 标题，D1）；仅首次写入（自动建线程、锁内 size==0）时生效，对既有线程静默忽略（行为登记见下，OQ-1） |

- **写侧糖衣 flag 撤销（2026-08-15 owner 裁决）**：`--reply-to`/`--mention` 从本命令签名撤销，send 传入该两 flag 一律落 **usage exit 2**（未知 flag 路径，usage 信封机制承担迁移教学）；原「值以 `@#N`/`@name` token 注入正文」的糖衣注入逻辑删除（format-v2 D2/OQ-4 参数面废止）。reply/mention 语义由 agent 在正文直接书写 `@#N`/`@name` token 表达；读取时 derive 机制不变（post read 从正文派生 reply/mention 关系，implicit-mention 输出字段派生逻辑冻结，§3.3）。原「`--reply-to` 指向不存在 seq 静默跳过」条款随撤销废止（backlog B-01 问题面消解，台账 LED-09 闭合）；原「`--reply-to 0` 拒绝（validation）」分支随 flag 撤销不可达。

- **`--message` 与 `--stdin` 互斥语义**：二选一必填。同时给出 -> clap conflicts 层拒绝，**usage exit 2**（v0.5 时该冲突为 validation exit 1，本版提升为 usage 层，见 §5）；两者皆缺 -> usage exit 2（clap `required_unless_present` 组合在解析层判定 MissingRequiredArgument，命令层无需管道，rework 裁定 F2）；仅 `--stdin` 时正文从 stdin 读取；`--message` 值 trim 后为空 -> validation exit 1（空正文拒绝，行为沿用）。
- **NAME/BODY 混淆面结构性归零**：两参数均不占位置槽，v0.5 spec §3.1 记载的混淆面（PATH+单字符串无法区分漏 NAME 与缺 body）不复存在；v0.5 的三重教学补偿条款随混淆面消亡而废止。
- 行为保留：线程不存在时自动创建（ensure_suffix 第(3)级落点）。
- **建线程元数据载荷行为登记（rework 裁定 F6，本轮不改运行时行为）**：`--title` 仅在线程首次写入（自动建线程）时生效；对既有线程附该 flag 时**静默忽略**（format-v2 冻结语义，exit 0 且无信号；改标题不在本版能力范围，OQ-1）。该静默面为已知行为登记而非缺陷（bdd S-SEND-17 钉住）；可检测化的未来工作项（ok 信封 ignored 字段增补）见 design.md §8。
- 输出增补 `implicit-mention`（U-10）行为沿用 v0.5 spec §3.1，不变。

错误映射：缺 `--author` / 缺 `--message` 且无 `--stdin` / 两者同给 / 未知 flag / 多余位置参数 -> usage exit 2（**含写命令传入已撤销的 `--reply-to`/`--mention`，按未知 flag 落 usage**）；空正文 -> validation exit 1；异型文件 -> format exit 1。

### 3.2 post edit

```
paperwork post edit <PATH> --author <NAME> --seq <N> (--message <NEW_BODY> | --stdin)
```

| 参数 | 形态 | 必填 | 说明 |
|---|---|---|---|
| `PATH` | 位置参数 | 是 | 线程文件路径 |
| `--author/-a` | 具名 flag | 是 | 编辑者名字，须与原消息发送者一致（三重护栏之一） |
| `--seq` | 具名 flag | 是 | 目标消息序列号 u64；非数字 -> usage exit 2 |
| `--message/-m` | 具名 flag | 与 `--stdin` 二选一 | 新正文；互斥语义同 send |

- 三重编辑护栏（自己的、自己最新的、线程最后一条）行为不变，违规 not-allowed exit 1。
- 错误映射：缺 `--author` / 缺 `--seq` / 缺正文通道 -> usage exit 2；护栏违规 -> not-allowed exit 1；异型文件 -> format exit 1（v0.5 已确立，不变）。

### 3.3 post read / summary（不变；读侧过滤器保留声明）

```
paperwork post read <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]
paperwork post summary <PATH>
```

- **读侧过滤器保留声明（2026-08-15 owner 裁决，显式写明）**：post read 的 `--mention <name>` / `--reply-to <seq>` 是**查询过滤器而非语义表达**，不在写侧撤销范围，冻结保留；其过滤判定基于读取时从正文 derive 的 `@name`/`@#N` token（derive 机制不变）。撤销与保留的边界口径见 §1.4/§10；解释口径由编排层拟定并显式声明，供 owner 复核推翻（docs/dev/owner-rulings-2026-08-15.md）。
- `--from` 仅表 seq 起点；`--to` 仅表 seq 上限（u64 类型，非数字即 usage exit 2，显式信号）。基线勘误后 `--from/--to` 仅存于 post read，规则 3 唯一语义无例外（§1.4）。`--limit` 默认 20；`--mention` 与 read 的 `--reply-to` 均**无短形式**（短形式全表见 §4）。
- 输出增补（恒显 `showing: n/total` + `window: #first-#last`，U-11）沿用 v0.5 spec §3.1，不变。
- 缺 PATH -> usage exit 2；文件不存在 -> not-found exit 1。

### 3.4 profile 组

```
paperwork profile create <PATH> --name <NAME> [--model] [--description] [--scope-read ...] [--scope-write ...] [--scope-owns ...]
paperwork profile show <PATH>
paperwork profile edit <PATH> [--model] [--description] [--scope-*]
paperwork profile list <DIR>
```

- `--name` 回到具名必填 flag（v0.5 曾将其位置化，本版按规则 2 收编回 flag 层；profile 名字不与 content 同槽，不适用 `--author`，v0.6_feedbacks §2.4 补记）。
- show/edit/list 行为不变；create 缺 `--name` -> usage exit 2；重复 create -> already-exists exit 1。

### 3.5 brief 组

```
paperwork brief create <PATH> --title <T> [--owner] [--description]
paperwork brief add <PATH> --entry <E> [--regex <PATTERN>] [--note <TEXT>]
paperwork brief remove <PATH> --entry-title <T>
paperwork brief read <PATH> [--full] [--entry-title <T>]
paperwork brief verify <PATH> [--base-dir <DIR>]
```

- `--title/--entry/--entry-title` 回到具名必填 flag（v0.5 位置化的逆向收编）。
- add 时自动计算目标文件 SHA-256 快照（不变）；remove 传条目存储标题（basename 推导规则沿用 v0.5 spec §3.3）。
- read 默认 TOC / `--full` 全量两档行为不变（三态判定 fresh/shifted/stale 与 conclusion `N/M fresh` 为 agent 知识保鲜契约，冻结，属 verify）。
- **read `--entry-title`（本轮新增，渐进阅读第三档）**：可选过滤，复用 brief remove 的 `--entry-title` 键语义（存储标题，basename 推导规则沿用 v0.5 spec §3.3）；命中时输出该条目详情（path/hash/regex/note，即 `--full` 档字段，仅该条目）；**字段面口径（rework 补录）**：`--entry-title` 命中即按 `--full` 档字段输出，Default 与 `--json` 两档同口径（即 JSON entries 命中条目含 path/hash/regex/note，不再受 `--full` 门控，Daniel m-4 定案）；无匹配 -> not-found exit 1，fix 引导 `brief read <PATH>` 列出条目；与 `--full` 组合合法（等价于单条目详情，无冲突）；未给 `--entry-title` 时 TOC / `--full` 行为冻结不变（含 conclusion `N entries` 形态与 JSON 非 full 档字段面）。**空值守栏（评审轮补录，F1）**：`--entry-title` 值为空或 trim 后全空白 -> validation exit 1（message `entry title (--entry-title) is empty`，fix `provide a non-empty --entry-title value`，example 为 `paperwork brief read <PATH> --entry-title main.rs`），镜像 post send `--message` 空值判定先例；属行为变更（此前空值落入 not-found）——空键是「无键」而非未命中。
- 缺必填 flag -> usage exit 2；条目不存在（remove / read `--entry-title`）-> not-found exit 1。
- brief add/remove 写路径本轮补锁（§3.9）。

### 3.6 contacts 组

```
paperwork contacts create <PATH> [--title]
paperwork contacts add <PATH> --profile <P>
paperwork contacts remove <PATH> --profile <P>
paperwork contacts update <PATH> --profile <OLD> --new-profile <NEW>
paperwork contacts read <PATH>
```

- create 不变（`--title` 有默认值 `Contacts`，属可选，保持 flag）；add 的 `--profile` 回到具名必填 flag；read 富化输出不变。
- add 缺 `--profile` -> usage exit 2。**add 对 profile 路径的可读性现状（rework 更正）**：add 对目标 profile 路径**不做任何可读性校验**（现状冻结，worktree `ops/contacts.rs` `contacts_add` L53-92 实测：直接 `derive_label` 静默回退）；不可读时 label 依 R11 回退文件名主干，仍 exit 0——属纯静默回退而非任何 validation/not-found（初版曾失实表述为「validation/not-found 沿用现状」，经 Ryan M-3 指出后更正）。
- **空键护栏（评审轮补录，F1；Kim M-1 + QA BUG-1）**：add 的 `--profile`、update 的 `--profile`/`--new-profile`，值为空或 trim 后全空白时一律 validation 错误 exit 1——message 逐字为 `profile path (--profile) is empty` / `new profile path (--new-profile) is empty`，fix 逐字为 `provide a non-empty --profile value` / `provide a non-empty --new-profile value`，example 为各动词规范示例（同 §5 第 2 条钉住形态）；镜像 post send `--message`/`--author` 空值判定先例（category validation，fix 教学 + canonical example，纯 ASCII）。护栏置于 core 函数入口（`contacts_add`/`contacts_update`），库直调同样覆盖；校验先于文件存在性的 not-found 判定（空键 + 文件不存在亦落 validation）。**行为变更登记**：护栏前 add 空键会把不可解析 bullet `- []()` 写入落盘（下次解析该 bullet 静默消失，validate 判结构损坏，属静默数据损坏）；update 空 `--new-profile` 更会把既有条目替换成该退化 bullet。空键是「无键」，不是「不可读的路径」——与不可读路径静默回退判例（上一条）不冲突。
- **contacts remove（本轮新增）**：`--profile` 具名必填 flag；键 = profile 路径（字符串精确匹配，与 add 幂等判据同一口径）；命中删除该条目，exit 0，ok 信封首行 `ok contacts.remove <profile> -> <contacts路径>`，字段区含 `contacts`（contacts 路径）、`removed`（被删 profile 路径）；未命中 -> not-found exit 1（resource Contacts entry），fix 引导 `paperwork contacts read <PATH>` 核对条目清单，并补键口径教学句：`the key is the profile path as stored in the contacts file, not the label`（纯 ASCII，rework 补录 Ryan m-3）；缺 `--profile` -> usage exit 2，规范示例逐字为 `paperwork contacts remove team.contacts.md --profile alice.profile.md`（rework 补录 Ryan m-2）。
- **contacts update（本轮新增）**：`--profile <OLD>` 与 `--new-profile <NEW>` 均具名必填；判定顺序：OLD 命中检查先于 NEW 已存在检查（OLD 未命中即 not-found，OLD==NEW 且 OLD 命中时落入 already-exists）；OLD 未命中 -> not-found exit 1（fix 同 remove 的键口径教学句 + `paperwork contacts read <PATH>` 引导）；NEW 已存在于条目清单 -> already-exists exit 1（fix 引导先 remove 或改用既有条目）；命中则原地替换 destination，label 依 R11 对 NEW 重派生（读 NEW 目标 profile H1，失败回退文件名主干，format-v2 spec §7.3），条目顺序保留；**不支持改 label**（无 label 类 flag；label 为 R11 派生数据，不可作键、不可手工覆盖）；ok 信封首行 `ok contacts.update <OLD> -> <NEW>`，字段区含 `contacts`、`updated`（OLD -> NEW）；缺任一必填 flag -> usage exit 2，规范示例逐字为 `paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md`（rework 补录 Ryan m-2）。
- **NEW 不存在/不可读时的行为契约（rework 补录，Ryan M-3；裁决维持现状行为，显式声明并钉住，bdd S-CONTACTS-14）**：update 到不存在/不可读的 NEW 仍 **exit 0 静默写入**：destination 按用户所给原值落盘，label 依 R11 回退文件名主干（与 format-v2 R11 及 add 现状一致，不引入 add/remove/update 三动词行为分叉；本轮不改运行时）。该面为**已知静默面，非缺陷**（处置体例同 post send `--title` 静默忽略的 S-SEND-17 三件套：本条声明 + bdd 场景钉住 + backlog 登记）；agent 自救指引：写入前先 `paperwork contacts read <PATH>` 核对既有条目，或 `paperwork validate <NEW>` 确认目标 profile 合法。候选增强「写前 destination 存在性校验/回显」已登记 `docs/researches/ux-open-items-backlog-2026-08-08.md` B-02，~~供发布轮裁决~~，2026-08-15 owner 裁决落地为下条 advisory 校验契约（backlog B-02 已追加裁决注记，台账 LED-10 闭合）。
- **destination advisory 校验契约（2026-08-15 owner 裁决新增，bdd S-CONTACTS-16/17；任务 #36 实施）**：contacts add/update 对 destination（add 的 `--profile`；update 的 `--new-profile`）执行**非阻塞 advisory 校验**（owner 裁决原文：「contact 可以加入路径和格式上面的 validation, 但是不阻塞 agent 编辑即可, 也就是如果文件不存在或者错误, 也不需要阻塞编辑的 agents 添加这个 profile 文件」）：
  - **触发条件**（任一命中即触发）：① destination 路径不存在；② 存在但不可读（非文件/打开失败）；③ 可读但非合法 profile 格式（parse 失败）。校验在写入动作完成之后、ok 信封渲染之前执行，仅只读探测，不改变写入结果。
  - **非阻塞保证**：触发时仍**照常写入、exit 0**；永不因 destination 问题 exit≠0；不引入任何新 flag、不阻塞 agent 编辑（label 依 R11 回退行为不变，上一条静默面契约维持，其上叠加 advisory 提示）。
  - **advisory 字段契约**：ok 信封增补字段，字段名钉住为 `advisory`（值 = 单行提示文本，文案**模板**恒为纯 ASCII，建议文案形态：`destination '<P>' does not exist` / `destination '<P>' is not readable` / `destination '<P>' is not a valid profile file`，2026-08-15 任务 #36 实施定稿冻结：三形态文案逐字采用建议形态；2026-08-15 三维评审 Ray S-1 口径收窄：destination 按用户所给原文插值回显（与 conclusion/profile 字段同权），整行 ASCII 仅当 destination 路径本身为 ASCII，纯 ASCII 声明的口径限于文案模板）；**仅触发时出现**（destination 合法时字段不存在，避免噪音）；Default 与 `--json` 档同名出现（JSON 只增不改不删纪律之下的 additive 字段）。被否替代字段名登记：`destination_note`（编排层候选，裁定取 `advisory`，理由：语义更准确且为后续同类提示预留通用面）。
  - **适用范围**：仅 contacts add/update 两写路径；contacts remove（键不命中为 not-found exit 1，无 advisory 面）、brief/profile 组不适用；与空键护栏（上一条 F1）不冲突：空键仍落 validation exit 1，advisory 仅覆盖「非空但异常」的 destination。
- **`updated` 字段值格式契约（rework 补录，Ryan m-4 定案）**：编排层裁定维持箭头串形态（与 ok 首行 conclusion 同构，不新增分键），值格式逐字钉住为 `<OLD> -> <NEW>`（单空格分隔的三段拼接，如 `alice.profile.md -> carol.profile.md`）；机器解析侧可退回解析 conclusion，两载体携带同一信息。
- **update 与 edit 的语义分工（rework 补录，Ryan M-1 ②）**：`edit` 恒为「对文件自身内容的就地修改」（post 消息正文、profile 字段）；`contacts update` 恒为「条目 destination 路径的换绑」（键控条目的身份替换），contacts 组无 `edit` 动词；两动词作用对象不同，不构成语义重叠。`--new-profile` 命名被否替代（`--to`/`--old-profile` 对称形态/`--replace-with`）见研究文档 §5.4 被否替代表（Ryan m-6）。
- add/remove/update 三写路径一律 fs2 `lock_exclusive` 锁内读改写（§3.9 写路径锁统一）。

### 3.7 validate（不变）

```
paperwork validate <PATH> [--type post|profile|brief|contacts]
```

- `--type` 语义沿用 v0.5 spec §3.5（给出时覆盖后缀推断；未知后缀且未给 `--type` -> format exit 1；`--type` 非法值 -> usage exit 2）。

### 3.8 别名与隐藏别名（不变）

- 隐藏别名 `p`(profile)/`b`(brief)/`c`(contacts)/`v`(validate)/`po`(post) 全部保留（v0.5 确立）。
- 不引入子命令级别名。

### 3.9 写路径锁语义（本轮统一）

- **原则**（owner 指令，v0.7_feedbacks §一 (1)/(3)）：非并行编辑场景的写路径，一律锁定 + fast fail——「遇到冲突了就锁定文件阻塞, 另一侧fast fail, 这是paperwork 中并行场景以外的基本做法」。
- **适用范围**：contacts add / remove / update、brief add / brief remove、profile edit 六处写路径，一律 fs2 `lock_exclusive` 锁内读改写（复刻 `thread_edit` 模式：开 read+write 句柄 -> 取锁 -> 经持锁句柄读 -> 变更序列化 -> 锁内 truncate+rewrite -> 解锁）；冲突阻塞等待持锁；`lock_exclusive` 获取失败即 fast fail，落 io 错误信封 exit 1（fix：`another process may hold the lock; retry shortly`，沿用 thread 路径既有文案），**禁止无锁降级写入**。
- **Windows 判例**：锁内必须经同一持锁句柄读取文件内容；跨句柄读被锁字节区间即时失败 os error 33（ERROR_LOCK_VIOLATION，QA BUG-2 教训）。
- **崩溃窗口**：锁内 truncate+rewrite 的断电/杀进程丢文件窗口沿用 format-v2 spec §5.7 已接受判例；本轮不引入 temp+rename 加固。
- **对外契约面**：六路径的可观察输出（信封、退出码、文件产物）与补锁前同构，仅并发安全性增强；contacts add 幂等语义不变。**例外声明（rework 补录）**：阻塞等待本身是本轮新增的可观察行为（命令可能长时间乃至无限期不返回，见下条），不属「同构」范围。
- **内容未变零写入（评审轮补录，F4；Kim m-1 = Oscar M-2）**：锁 helper 在闭包返回内容与原内容字节相同时跳过 truncate+rewrite（仅解锁返回）；contacts add 幂等路径由此恢复补锁前基线的「零写入」语义（mtime 稳定、无 no-op 崩溃窗口）。**io 失败 fix 文案冻结（评审轮补录，F5；Oscar M-1）**：六路径写失败（open/set_len/write_all）的 io 信封 fix 文案与补锁前基线逐字一致：`check that the target path is writable`。
- **agent 可见阻塞行为契约（rework 补录，Ryan M-2）**：① `lock_exclusive` 阻塞等待**无内建超时**（fs2 语义，Windows LockFileEx / Linux flock 一致），持锁者长时间不释放时等待侧表现为进程无响应（无退出码、无信封）；② 写临界区为锁内读改写，毫秒级，正常场景阻塞短暂；③ 持锁进程崩溃/退出后 OS 自动释放锁（Windows 句柄锁随进程消亡释放），不会永久死锁；④ 对 agent 编排层的指引：可对 paperwork 进程施加自有时限（进程级超时），超时后杀进程重试——幂等的 add 与先读后写的 remove/update/brief add/remove/profile edit 重试安全，exit code 语义不变；⑤ timeout flag 否决记录：无既有先例、扩大 flag 面违背最小变更、owner 指令即「锁定阻塞 + fast fail」两态（研究文档 §6.1 被否方案表），否决 timeout flag 但**不豁免本条阻塞行为声明**。

---

## 4. 短形式全表

**短形式收窄为编排层裁定（rework 裁定 F3）**：全 CLI 仅 `--author/-a`、`--message/-m` 两个命令级短形式，加既有全局 `-q`；此前初稿中实现方设计的一切其他短形式（`-r/-p/-t/-l/-d/-o/-n` 等）全部收回为仅长形式。理由：初稿全表存在四处跨命令多义（`-m/-p/-t/-d`），与「语义无冲突」原则自相矛盾且侵蚀 agent 泛化预期（Pete M2）；收窄后「全 CLI 短形式语义无冲突」在新表下自然成立；`--mention` 对 `-m` 的避让随之保留。

| flag | 短形式 | 依据 |
|---|---|---|
| `--author` | `-a` | 编排层裁定（全称首字母，全 CLI 唯一） |
| `--message` | `-m` | 编排层裁定（git `commit -m` 行业惯例，短 flag 传正文的最强迁移直觉） |
| 全局 `-q` | `-q` | 既有全局 flag（v0.5 已发布，冻结） |
| post read `--mention` | 无 | 编排层裁定：避免 `-m` 在 post 组内双义；2026-08-15 写侧 `--mention` 撤销后，read 过滤器为该 flag 唯一在场面 |
| post read `--reply-to` | 无 | rework 补录（Quinn m-1）：read 过滤低频；2026-08-15 owner 裁决撤销写侧 `--reply-to` 后，read 过滤器为该 flag 唯一在场面（原「与 send 的 `--reply-to` 对称」依据废止），仍仅长形式 |
| 其余全部 flag（`--seq/--stdin/--title/--from/--to/--entry/--entry-title/--profile/--new-profile/--model/--description/--owner/--note/--regex/--scope-*/--full/--limit/--base-dir/--type/--json/--plain` 等） | 无 | F3 收窄裁定：除 `-a/-m/-q` 外一律仅长形式，从根上消除跨命令短形式多义 |

短形式与全称严格等价（clap 同一 Arg 的 short/long 两面），BDD 断言见 bdd.md S-SHORT-01。

本轮增量（v0.7 feedback 轮）：新 flag `--new-profile`（contacts update）与 brief read 的可选 `--entry-title` 均仅长形式，短形式集合 {-a, -m, -q} 不变；`--entry-title` 同时出现于 brief remove（必填）与 brief read（可选），均指条目存储标题，属同一语义对象的同构延伸，不构成双语义（规则 3，§1.4 裁定先例）。

2026-08-15 owner 裁决：写侧 `--reply-to`/`--mention` 撤销后，短形式集合 {-a, -m, -q} 不变（该两 flag 本就无短形式）；无短形式负向清单口径相应收窄为「post read `--reply-to` / `--mention` 两项」（send 侧 flag 不复存在，探针移除，bdd S-SHORT-02 枚举同步）。**VALUE_TAKING_FLAGS 对应表口径（main.rs usage 路径 `--json` 探针依赖）**：`--reply-to` / `--mention` 两项**保留在列**——post read 侧仍是带值 flag；撤销仅作用于写侧 clap 签名，若从该表移除会破坏 `post read x --mention "--json"` 类探针的值跳过逻辑（audit-grammar-matrix-2026-08-15 §6 一致性表的计数口径随实施批次同步重盘，任务 #36）。

---

## 5. 输出 envelope 契约（对 v0.5 的引用声明）

v0.5 spec §4（成功信封、错误信封七类 category、usage 信封机制、退出码 0/1/2 语义、`--json/--plain/-q` 三档、JSON 只增不改不删）**整体继承，逐条冻结**，本文不重复其全文；仅声明以下本版差异：

1. **`--message` 与 `--stdin` 互斥两形态的错误层级**：两者同给 -> clap conflicts 判定，**usage exit 2**（v0.5 为 validation exit 1，本版层级提升）；两者皆缺 -> clap `required_unless_present` 组合在解析层判定（MissingRequiredArgument），同样 **usage exit 2**，命令层无需管道（rework 裁定 F2，与 v0.6_feedbacks §2.3 裁定补记一致）。信封结构不变。
2. **usage 信封静态规范示例全部换 v0.6 文法，每命令一条**：机制（静态规范示例、不携带用户原参数值、`--help/-V` 穿透、argv 扫描感知 `--json`、顶层失败 command 填 `usage`）沿用 v0.5 spec §4.3，仅示例文案更新；每命令一条静态规范可执行示例（具体值、无占位符，v0.5 F2/F7 裁定延续，rework 裁定 F5），post send 规范示例为 `paperwork post send standup.post.md --author alice --message "Hello"`（采 `--message` 通道形态）；「二选一」等形态指引由 message/fix 文案承担，不在 example 中表达。**本轮新命令逐字钉住（rework 补录，Ryan m-2）**：contacts remove 规范示例 `paperwork contacts remove team.contacts.md --profile alice.profile.md`；contacts update 规范示例 `paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md`（与 §3.6 同一出处，实施与测试逐字断言以此为准）；not-found example 形态：remove/update 未命中均为 `paperwork contacts read <PATH>`（PATH 取用户所给实际值）。
3. **validation fix 文案的 `--` 教学废止**：正文经 `--message` flag 值直传，以 `-` 开头的正文不再需要 `--` 边界（v0.5 spec §4.2 末条废止）；usage 信封涉及裸 `-xxx` 残留时的 `--` 教学改为引导 `--message` 形态。
4. **纯 ASCII 输出契约（口径收窄，任务 #34 文档轮修订；行为面零变更）**：信封结构面（status 行 / category / `fix:` / `example:` 字段）全部 stdout/stderr 字节保持 ASCII，本版不变，`ascii_output_contract_guard` 级别集成测试防线保留；全量字节流恒为合法 UTF-8，消费端须按 UTF-8 解码；已知 locale 依赖面：io 类信封的 `message` 字段可内嵌 OS 本地化文本（字节为合法 UTF-8，依据 docs/dev/io-encoding-rootcause-2026-08-15.md §6 钉住结论；是否进一步代码硬化去本地化文本由修复波评估，审计建议不做）。
5. **VALUE_TAKING_FLAGS 对应表口径（2026-08-15 owner 裁决配套声明）**：写侧 `--reply-to`/`--mention` 撤销后，main.rs `VALUE_TAKING_FLAGS` 常量中该两项**保留**（post read 侧仍是带值 flag，usage 路径 `--json` 探针的值跳过逻辑依赖其在列）；常量其余项不变，计数口径（audit-grammar-matrix §6）随任务 #36 实施重盘。
6. **advisory 信封字段形态（2026-08-15 owner 裁决新增）**：contacts add/update 成功（exit 0）且 destination advisory 校验触发时，ok 信封字段区增补 `advisory: <单行提示>`（Default 档与 `--json` 档同名 key；仅触发时出现；文案模板纯 ASCII，destination 原文插值回显，整行 ASCII 仅当路径为 ASCII——Ray S-1 口径收窄；字段契约全文见 §3.6「destination advisory 校验契约」）。

---

## 6. 附带行为变更清单（相对 v0.5）

| 变更 | 内容 | 性质 |
|---|---|---|
| NAME/BODY/主载荷具名化 | post send/edit 的 NAME/BODY/SEQ 与 profile NAME、brief TITLE/ENTRY/ENTRY-TITLE、contacts PROFILE-PATH 一律改具名必填 flag | 文法 breaking（参数层） |
| `--message`/`--stdin` 互斥层级 | validation exit 1 -> usage exit 2（clap conflicts 判定） | 错误层级变更 |
| NAME/BODY 混淆面 | v0.5 固有边界（S-SEND-12 形态）结构性消亡；三重教学补偿废止 | 边界消亡 |
| `--` 边界需求 | `--message "-dash body"` flag 值直传无需 `--`；v0.5 的 `--` 教学条款废止 | 教学面收缩 |
| v0.5 位置文法迁移 | `send <PATH> alice "Hi"` 等 v0.5 调用中多余位置参数落 usage exit 2，信封示例即 v0.6 形态 | 迁移教学（usage 信封机制承担） |
| core example 文案 | ops/*.rs 14 处 example 字符串换 v0.6 文法（纯文案，API 零变更） | 文案 |
| ensure_suffix / 别名 / validate --type / 输出增补字段 | 全部继承 v0.5，零变更 | 冻结 |
| contacts remove/update 新增（本轮增量） | 完整 CRUD 补齐；键 = profile 路径；label 依 R11 重派生；锁内读改写 | 动词新增（additive） |
| brief read `--entry-title`（本轮增量） | 选择性详情（渐进阅读第三档）；复用 remove 键语义 | flag 新增（additive） |
| 写路径锁统一（本轮增量） | contacts add/remove/update、brief add/remove、profile edit 六写路径补锁（锁内读改写 + fast fail，§3.9） | 内部机制增强，对外契约不变 |
| 写侧 `--reply-to`/`--mention` 撤销（2026-08-15 owner 裁决） | post send 传入该两 flag 落 usage exit 2；reply/mention 语义由正文直书 `@#N`/`@name`（读侧 derive 不变）；「指向不存在 seq 静默跳过」面消解 | 文法 breaking（flag 面撤销，任务 #36 实施） |
| contacts destination advisory 校验（2026-08-15 owner 裁决） | add/update 非阻塞 advisory 校验：destination 异常仍 exit 0，ok 信封增补 `advisory` 字段（只增不改协议） | 输出协议 additive（任务 #36 实施） |

---

## 7. 输出协议冻结条款

1. v0.5 spec §6 冻结条款**逐条继续有效**：ok/error 信封结构、七类 error category、command 标识（`post.send` 等全部不变，且随 format-v2 删除 `post.create`）、全局 flag `--json/--plain/-q/-V`、JSON 既有 key 只增不改不删、纯 ASCII 输出契约。
2. **core API 与文件格式零变更**：`paperwork-core` 公开函数签名、返回类型、错误类型不变；四类托管文件 Markdown 结构不变（format-v2 已确立的 v2 格式为基线）。
3. 本次破坏面**仅限命令参数文法**（位置 NAME/BODY/主载荷 -> 具名 flag、互斥错误层级），其余一切对外接口冻结。
4. **版本与发布**：本轮不 bump 版本、不打 tag、不 publish、不写 CHANGELOG 发布段（owner 显式约束，v0.6_feedbacks §一 (3)）；发布时机与版本号由 owner 在功能稳定后另行裁定。
5. **本轮 additive 扩展登记（v0.7 feedback 轮，2026-08-09）**：本轮为上述冻结条款之下的 additive 扩展——组集合 {profile, post, brief, contacts, validate} 不变；动词集合仅新增 `contacts remove` / `contacts update`（command id `contacts.remove` / `contacts.update`）；flag 表仅新增 `--new-profile` 与 brief read 的可选 `--entry-title`；短形式集合 {-a, -m, -q} 不变；JSON key 与信封字段只增不改不删（新增字段 `contacts` / `removed` / `updated`，`updated` 值格式逐字钉住见 §3.6）；core API 仅新增函数（`contacts_remove` / `contacts_update`），既有函数签名不变、行为面仅增加并发安全性（§3.9）；文件格式零触碰（format-v2 spec L12「profile/brief/contacts 三格式语义不变」继续成立）；本条第 4 款不发布约束对本轮延续有效。**SOTA C6 白名单扩容登记（rework 补录）**：`update` 不在 SOTA C6 既定动词白名单内（`docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` L196，既定 10 动词），随本轮 additive 扩容纳入，授权依据 owner 指令 (1)（v0.7_feedbacks §一 (1)，裁定记录 v0.7_feedbacks §2.5）；白名单测试口径见 tdd §8.3 与 bdd S-SHORT-02。
6. **2026-08-15 owner 裁决修订面登记（任务 #35 落盘，任务 #36 实施）**：在上述冻结条款之下——撤销面：post send 的 `--reply-to`/`--mention` 两个可选糖衣 flag（写命令传入落 usage exit 2）；新增面：ok 信封 `advisory` 字段（仅 contacts add/update 触发时，JSON 只增不改不删）；读侧豁免：post read `--reply-to`/`--mention` 过滤器、implicit-mention 派生、showing/window 均不受影响；短形式集合 {-a, -m, -q} 不变；core API 与文件格式零变更；不发布约束（本条第 4 款）对本轮延续有效。裁决原文与解释口径见 docs/dev/owner-rulings-2026-08-15.md。

---

## 8. 兼容策略

- **干净切断**（沿用 v0.5 design §9 立场）：0.x 惯例、双文法是维护噩梦、owner 为唯一真实消费方，不留隐藏弃用别名。
- **迁移教学由 usage 信封承担**：v0.5 位置文法调用（`send <PATH> alice "Hi"`、`profile create <PATH> alice` 等）因多余位置参数落入 usage 信封 exit 2，信封内静态规范示例即 v0.6 形态，一次重试完成迁移；仍为 v0.6 无效 flag（`--from` 作身份等）的教学链自然延伸覆盖（rework 修正，Quinn M-3：`--name/--seq/--title/--entry/--profile` 在 v0.6 重新合法，不再落入未知 flag 教学，原表述过度声称废止）。
- **SKILL.md + usage 信封 + after_help 三件套**继续承担迁移教学（依据 `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` 结论 C5）；SKILL.md 示例随实现步刷新为 v0.6 文法。
- 消费者可感知变化（参数文法 breaking、互斥错误层级变化）的披露载体为发布时的 CHANGELOG（**本轮不写**，由 owner 裁定的发布轮承担（§7 第 4 条）。

---

## 9. 渐进阅读能力登记结案（本轮增量）

owner 指令（v0.7_feedbacks §一 (2)）要求盘点全部 read 面的「先看 summary 再选择性或全部查看详情」能力，登记如下（证据全文见研究报告 §2）：

| 面 | summary/TOC 档 | 选择性详情档 | 全量详情档 | 结论 |
|---|---|---|---|---|
| post | `post summary` | `post read` 窗口过滤（`--from/--to/--mention/--reply-to/--limit`） | `post read` 全窗口 | 已满足；summary 独立动词经 U-09 裁决保留结案（`docs/researches/ux-open-items-backlog-2026-08-08.md` L22） |
| brief | `brief read` 默认 TOC | `brief read --entry-title <T>`（**本轮补齐**） | `brief read --full` | 本轮补齐第三档后满足；两档先例见 `docs/reviews/v0.3-review-2026-08-01.md` L152 |
| profile | —（单文档粒度） | `profile show` 直读 | `profile show` 直读 | 已满足（show 即 read 能力，§3.4） |
| contacts | —（单文档粒度） | `contacts read` 直读 | `contacts read` 直读 | 已满足（富化输出，§3.6） |

SSOT 依据：pillars session-log 原话「agent读取时候会先读到目录, 然后可以选择直接全量阅读, 或者根据路径自己手动选择性阅读」（`docs/ssot/pillars/paperwork-init-conversation/session-log-2026-07-29-agent-paperwork-user-only.md` L27）；SOTA summary-before-detail（`docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` L63、L194 C4）。本登记后渐进阅读面结案；新缺口须 owner 新指令再开。

---

## 10. owner 裁决增量修订登记结案（2026-08-15，任务 #35 文档落盘，实施归任务 #36）

owner 于 2026-08-15 对台账 LED-09/10/11/12（backlog B-01/B-02/U-04/U-13）下达四项裁决（逐字原文、解释口径与影响面全文见 docs/dev/owner-rulings-2026-08-15.md；台账闭合见 open-items-ledger 第十三节；backlog 联动见其第九节）。本 spec 的修订面登记如下：

| 裁决 | 对象 | 修订去向 |
|---|---|---|
| 裁决 1 | B-01：撤销写侧语义糖参数 | §1.4 裁定更新；§2 签名表 post send 行；§2 注；§3.1 参数表/撤销声明/错误映射；§6 变更清单；§7 第 6 条；bdd S-SEND-20 改写与 S-SEND-22/23、S-EDIT-10 新增 |
| 裁决 2 | B-02：contacts 非阻塞 advisory 校验 | §3.6「destination advisory 校验契约」（触发条件/非阻塞保证/字段契约/适用范围）；§5 第 6 条；§6 变更清单；§7 第 6 条；bdd S-CONTACTS-14 追加与 S-CONTACTS-16/17 新增 |
| 裁决 3 | U-04：同 B-01 处理 | 随裁决 1 同批落地（写侧 `--mention` 撤销，方向消解销账） |
| 裁决 4 | U-13：completions 钉住结案 | 不改 spec（completions 本就未在 flag/命令面）；确立长期方向「UX/QoL 以 agent 使用习惯为准」，登记于裁决记录 §二 口径 D |

**读侧过滤器保留声明（本节的边界线，显式写明）**：post read 的 `--mention <name>` / `--reply-to <seq>` 是查询过滤而非语义表达，不在撤销范围；其无短形式钉住（§4）、过滤行为（§3.3、bdd S-READ-04/06/07）与 derive 机制全部冻结保留。解释口径由编排层拟定并显式声明，供 owner 后续复核推翻。

**canonical_example 影响登记**：post send 规范示例 `paperwork post send standup.post.md --author alice --message "Hello"`（§5 第 2 条）本不含该两 flag，无需替换；但实现面 after_help 中含糖衣 flag 的示例行（post.rs send after_help 第二条示例）与 core/cli 内嵌旧形态 example/fix 文案，属任务 #36 实施批次的文案同步面（impl_plan O1/O4 点名）。

**实施与重冻**：行为变更由任务 #36 按 impl_plan「2026-08-15 owner 裁决实施批次」（O1~O5）执行；测试映射与黄金快照重冻预告见 tdd §9；SKILL.md/README 示例差异留给实施批次同步（impl_plan O4 点名，本任务不回改）。
