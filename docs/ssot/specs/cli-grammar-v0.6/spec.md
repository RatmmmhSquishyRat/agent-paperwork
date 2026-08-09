# CLI 文法 v0.6: Spec（行为规范）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布，见 §7）
- 文档性质：行为规范（命令契约 + 输出协议），实现与测试的唯一验收基准
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令，最高优先级；与 v0.5 文档冲突处以该文件为准）
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

裁定（规则 3 边界，沿用 v0.5 spec §1.3）：`--mention/--reply-to` 在 send 中为「设置」、在 read 中为「过滤」，视为同一语义对象的同构延伸，不构成双语义。

---

## 2. 新文法全表（命令签名契约）

约定：`<PATH>` = 唯一位置参数（必填）；`--x` = flag；`(--a | --b)` = 二选一必填；`[--x]` = 可选。相对 v0.5 的变更在第三列标注。

| 命令 | v0.6 签名 | 相对 v0.5 变更 |
|---|---|---|
| post send | `post send <PATH> --author <NAME> (--message <BODY> \| --stdin) [--reply-to N] [--mention a,b] [--title T]` | NAME/BODY 位置参数 -> `--author/-a`、`--message/-m` 具名必填（owner 裁决）；`--title` 为 format-v2 建线程载荷；`--reply-to`/`--mention` 为糖衣 flag，值以 `@#N`/`@name` token 注入正文（format-v2 D2，OQ-4） |
| post edit | `post edit <PATH> --author <NAME> --seq <N> (--message <NEW_BODY> \| --stdin)` | NAME、SEQ、NEW_BODY 位置参数 -> `--author/-a`、`--seq`、`--message/-m` 具名必填 |
| post read | `post read <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]` | 不变；`--mention` 无短形式（避免 -m 双义） |
| post summary | `post summary <PATH>` | 不变 |
| profile create | `profile create <PATH> --name <NAME> [--model] [--description] [--scope-read/write/owns]` | NAME 位置参数 -> `--name` 必填 flag（回到具名形态；v0.6_feedbacks §2.4 补记） |
| profile show/edit/list | `<PATH>` / `<PATH> [--field..]` / `<DIR>` | 不变 |
| brief create | `brief create <PATH> --title <T> [--owner] [--description]` | TITLE 位置参数 -> `--title` 必填 flag |
| brief add | `brief add <PATH> --entry <E> [--regex] [--note]` | ENTRY 位置参数 -> `--entry` 必填 flag |
| brief remove | `brief remove <PATH> --entry-title <T>` | ENTRY-TITLE 位置参数 -> `--entry-title` 必填 flag |
| brief read/verify | `<PATH> [--full]` / `<PATH> [--base-dir]` | 不变 |
| contacts create | `contacts create <PATH> [--title]` | 不变（title 有默认值，属可选） |
| contacts add | `contacts add <PATH> --profile <P>` | PROFILE-PATH 位置参数 -> `--profile` 必填 flag |
| contacts read | `contacts read <PATH>` | 不变 |
| validate | `validate <PATH> [--type post\|profile\|brief\|contacts]` | 不变 |

注：v0.5 的 `post create` 已被 format-v2 删除（建线程职责由 send 自动创建承担，`--title` 为建线程时的线程元数据载荷），v0.6 不恢复；format-v2 同批按 owner 追裁 D1/D2 删除了 send 的 `--to`/`--participants` flag，v0.6 同样不恢复。

---

## 3. 逐命令契约与错误 category 映射

约定：默认值语义沿用 v0.5 现状，本版本不改默认值。错误 category 与退出码映射：clap 层用法错误（缺必填 flag、未知 flag、互斥冲突、类型错误、多余位置参数）一律 usage exit 2；运行时六类（format/validation/io/not-found/already-exists/not-allowed）exit 1（v0.5 spec §4，继承不变）。

### 3.1 post send

```
paperwork post send <PATH> --author <NAME> (--message <BODY> | --stdin) [--reply-to N] [--mention a,b] [--title T]
```

| 参数 | 形态 | 必填 | 说明 |
|---|---|---|---|
| `PATH` | 位置参数 | 是 | 线程文件路径；经 ensure_suffix 三级解析（v0.5 spec §5，继承不变） |
| `--author/-a` | 具名 flag | 是 | 发送者名字（署名）；语义与校验沿用 v0.5 NAME（trim 后为空拒绝 validation；可含空格；不与 profile/contacts 做存在性校验）；不设 `allow_hyphen_values`（名字值无 `-` 开头合法形态，设置反扩大误解析面，F4 复核结论） |
| `--message/-m` | 具名 flag | 与 `--stdin` 二选一 | 消息正文；flag 值直传，以 `-` 开头的正文无需 `--` 边界；clap 属性 `allow_hyphen_values = true`（rework 裁定 F4，impl_plan 步骤(2) 硬性指令） |
| `--stdin` | 开关 flag | 与 `--message` 二选一 | 从 stdin 读正文（多行大片内容首选通道） |
| `--reply-to` | 具名 flag | 否 | 回复锚点 seq；format-v2 D2 下为糖衣 flag：值以 `@#N` token 注入正文首行，读取时派生（OQ-4）；指向不存在 seq 静默跳过（沿用 v0.5，已登记 ux-open-items-backlog） |
| `--mention` | 具名 flag | 否 | 提及名单（逗号分隔名字）；同为糖衣 flag：值以 `@name` token 注入正文（D2，OQ-4）；无短形式（§4） |
| `--title` | 具名 flag | 否 | 建线程载荷：线程标题（preamble 仅 H1 标题，D1）；仅首次写入（自动建线程、锁内 size==0）时生效，对既有线程静默忽略（行为登记见下，OQ-1） |

- **`--message` 与 `--stdin` 互斥语义**：二选一必填。同时给出 -> clap conflicts 层拒绝，**usage exit 2**（v0.5 时该冲突为 validation exit 1，本版提升为 usage 层，见 §5）；两者皆缺 -> usage exit 2（clap `required_unless_present` 组合在解析层判定 MissingRequiredArgument，命令层无需管道，rework 裁定 F2）；仅 `--stdin` 时正文从 stdin 读取；`--message` 值 trim 后为空 -> validation exit 1（空正文拒绝，行为沿用）。
- **NAME/BODY 混淆面结构性归零**：两参数均不占位置槽，v0.5 spec §3.1 记载的混淆面（PATH+单字符串无法区分漏 NAME 与缺 body）不复存在；v0.5 的三重教学补偿条款随混淆面消亡而废止。
- 行为保留：线程不存在时自动创建（ensure_suffix 第(3)级落点）。
- **建线程元数据载荷行为登记（rework 裁定 F6，本轮不改运行时行为）**：`--title` 仅在线程首次写入（自动建线程）时生效；对既有线程附该 flag 时**静默忽略**（format-v2 冻结语义，exit 0 且无信号；改标题不在本版能力范围，OQ-1）。该静默面为已知行为登记而非缺陷（bdd S-SEND-17 钉住）；可检测化的未来工作项（ok 信封 ignored 字段增补）见 design.md §8。
- 输出增补 `implicit-mention`（U-10）行为沿用 v0.5 spec §3.1，不变。

错误映射：缺 `--author` / 缺 `--message` 且无 `--stdin` / 两者同给 / 未知 flag / 多余位置参数 -> usage exit 2；空正文 -> validation exit 1；异型文件 -> format exit 1；`--reply-to` 指向不存在 seq -> 静默跳过（沿用）。

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

### 3.3 post read / summary（不变）

```
paperwork post read <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]
paperwork post summary <PATH>
```

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
paperwork brief add <PATH> --entry <E> [--regex] [--note]
paperwork brief remove <PATH> --entry-title <T>
paperwork brief read <PATH> [--full]
paperwork brief verify <PATH> [--base-dir <DIR>]
```

- `--title/--entry/--entry-title` 回到具名必填 flag（v0.5 位置化的逆向收编）。
- add 时自动计算目标文件 SHA-256 快照（不变）；remove 传条目存储标题（basename 推导规则沿用 v0.5 spec §3.3）。
- read/verify 不变（三态判定 fresh/shifted/stale 与 conclusion `N/M fresh` 为 agent 知识保鲜契约，冻结）。
- 缺必填 flag -> usage exit 2；条目不存在 -> not-found exit 1。

### 3.6 contacts 组

```
paperwork contacts create <PATH> [--title]
paperwork contacts add <PATH> --profile <P>
paperwork contacts read <PATH>
```

- create 不变（`--title` 有默认值 `Contacts`，属可选，保持 flag）；add 的 `--profile` 回到具名必填 flag；read 富化输出不变。
- add 缺 `--profile` -> usage exit 2；profile 路径不可读 -> validation/not-found 沿用现状。

### 3.7 validate（不变）

```
paperwork validate <PATH> [--type post|profile|brief|contacts]
```

- `--type` 语义沿用 v0.5 spec §3.5（给出时覆盖后缀推断；未知后缀且未给 `--type` -> format exit 1；`--type` 非法值 -> usage exit 2）。

### 3.8 别名与隐藏别名（不变）

- 隐藏别名 `p`(profile)/`b`(brief)/`c`(contacts)/`v`(validate)/`po`(post) 全部保留（v0.5 确立）。
- 不引入子命令级别名。

---

## 4. 短形式全表

**短形式收窄为编排层裁定（rework 裁定 F3）**：全 CLI 仅 `--author/-a`、`--message/-m` 两个命令级短形式，加既有全局 `-q`；此前初稿中实现方设计的一切其他短形式（`-r/-p/-t/-l/-d/-o/-n` 等）全部收回为仅长形式。理由：初稿全表存在四处跨命令多义（`-m/-p/-t/-d`），与「语义无冲突」原则自相矛盾且侵蚀 agent 泛化预期（Pete M2）；收窄后「全 CLI 短形式语义无冲突」在新表下自然成立；`--mention` 对 `-m` 的避让随之保留。

| flag | 短形式 | 依据 |
|---|---|---|
| `--author` | `-a` | 编排层裁定（全称首字母，全 CLI 唯一） |
| `--message` | `-m` | 编排层裁定（git `commit -m` 行业惯例，短 flag 传正文的最强迁移直觉） |
| 全局 `-q` | `-q` | 既有全局 flag（v0.5 已发布，冻结） |
| post read `--mention` | 无 | 编排层裁定：避免 `-m` 在 post 组内双义 |
| post read `--reply-to` | 无 | rework 补录（Quinn m-1）：read 过滤低频；与 send 的 `--reply-to` 同无短形式，保持对称 |
| 其余全部 flag（`--seq/--stdin/--title/--from/--to/--entry/--entry-title/--profile/--model/--description/--owner/--note/--regex/--scope-*/--full/--limit/--base-dir/--type/--json/--plain` 等） | 无 | F3 收窄裁定：除 `-a/-m/-q` 外一律仅长形式，从根上消除跨命令短形式多义 |

短形式与全称严格等价（clap 同一 Arg 的 short/long 两面），BDD 断言见 bdd.md S-SHORT-01。

---

## 5. 输出 envelope 契约（对 v0.5 的引用声明）

v0.5 spec §4（成功信封、错误信封七类 category、usage 信封机制、退出码 0/1/2 语义、`--json/--plain/-q` 三档、JSON 只增不改不删）**整体继承，逐条冻结**，本文不重复其全文；仅声明以下本版差异：

1. **`--message` 与 `--stdin` 互斥两形态的错误层级**：两者同给 -> clap conflicts 判定，**usage exit 2**（v0.5 为 validation exit 1，本版层级提升）；两者皆缺 -> clap `required_unless_present` 组合在解析层判定（MissingRequiredArgument），同样 **usage exit 2**，命令层无需管道（rework 裁定 F2，与 v0.6_feedbacks §2.3 裁定补记一致）。信封结构不变。
2. **usage 信封静态规范示例全部换 v0.6 文法，每命令一条**：机制（静态规范示例、不携带用户原参数值、`--help/-V` 穿透、argv 扫描感知 `--json`、顶层失败 command 填 `usage`）沿用 v0.5 spec §4.3，仅示例文案更新；每命令一条静态规范可执行示例（具体值、无占位符，v0.5 F2/F7 裁定延续，rework 裁定 F5），post send 规范示例为 `paperwork post send standup.post.md --author alice --message "Hello"`（采 `--message` 通道形态）；「二选一」等形态指引由 message/fix 文案承担，不在 example 中表达。
3. **validation fix 文案的 `--` 教学废止**：正文经 `--message` flag 值直传，以 `-` 开头的正文不再需要 `--` 边界（v0.5 spec §4.2 末条废止）；usage 信封涉及裸 `-xxx` 残留时的 `--` 教学改为引导 `--message` 形态。
4. **纯 ASCII 输出契约**（v0.5 修复轮确立的 `ascii_output_contract_guard` 级别约束）：全部 stdout/stderr 字节保持 ASCII，本版不变，集成测试防线保留。

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

---

## 7. 输出协议冻结条款

1. v0.5 spec §6 冻结条款**逐条继续有效**：ok/error 信封结构、七类 error category、command 标识（`post.send` 等全部不变，且随 format-v2 删除 `post.create`）、全局 flag `--json/--plain/-q/-V`、JSON 既有 key 只增不改不删、纯 ASCII 输出契约。
2. **core API 与文件格式零变更**：`paperwork-core` 公开函数签名、返回类型、错误类型不变；四类托管文件 Markdown 结构不变（format-v2 已确立的 v2 格式为基线）。
3. 本次破坏面**仅限命令参数文法**（位置 NAME/BODY/主载荷 -> 具名 flag、互斥错误层级），其余一切对外接口冻结。
4. **版本与发布**：本轮不 bump 版本、不打 tag、不 publish、不写 CHANGELOG 发布段（owner 显式约束，v0.6_feedbacks §一 (3)）；发布时机与版本号由 owner 在功能稳定后另行裁定。

---

## 8. 兼容策略

- **干净切断**（沿用 v0.5 design §9 立场）：0.x 惯例、双文法是维护噩梦、owner 为唯一真实消费方，不留隐藏弃用别名。
- **迁移教学由 usage 信封承担**：v0.5 位置文法调用（`send <PATH> alice "Hi"`、`profile create <PATH> alice` 等）因多余位置参数落入 usage 信封 exit 2，信封内静态规范示例即 v0.6 形态，一次重试完成迁移；仍为 v0.6 无效 flag（`--from` 作身份等）的教学链自然延伸覆盖（rework 修正，Quinn M-3：`--name/--seq/--title/--entry/--profile` 在 v0.6 重新合法，不再落入未知 flag 教学，原表述过度声称废止）。
- **SKILL.md + usage 信封 + after_help 三件套**继续承担迁移教学（依据 `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` 结论 C5）；SKILL.md 示例随实现步刷新为 v0.6 文法。
- 消费者可感知变化（参数文法 breaking、互斥错误层级变化）的披露载体为发布时的 CHANGELOG（**本轮不写**，由 owner 裁定的发布轮承担（§7 第 4 条）。
