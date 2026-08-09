# CLI 文法 v0.6: Design（设计方案）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布）
- 文档性质：设计方案（动线与参数布局论证 + 方案对比 + owner 裁决依据 + 否决记录）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令，最高优先级）
  - `docs/researches/cli-grammar-v06-reassessment-2026-08-09.md`（三视角重评估、path-first 复评、Rejected Alternatives 再评估）
  - `docs/ssot/specs/cli-ux-redesign/design.md`（v0.5 设计论证，继承基线）
  - `docs/ssot/adr/feedbacks/v0_feedbacks.md`、`docs/dev/adr-v1.md`（ADR-011）

---

## 1. 设计总纲

### 1.1 owner 裁决的采纳说明

owner 裁决三点（原文落盘见 v0.6_feedbacks.md §一）：

1. **接受 action-first**：「看来action first是cli中的基本设计, 那么我接受你们的这个设计」。`paperwork <组> <动词> <PATH> ...` 槽位顺序保留；v0.5 design §1.1 对 path-first 的四点否决论证经复评全部成立（§4），path-first 结案。
2. **NAME/BODY 具名化**：「name这里确实会和content歧义, 因此我们改为不定位置的必选参数, 也就是用户名使用--author这个全称, 内容使用--message这个全称, 然后简称自己设计」。全称由 owner 指定，短形式授权实现方设计（编排层裁定 `-a/-m`，§3）。
3. **本轮不发布**：不 bump/tag/publish/CHANGELOG 发布段。

意图落实核对：owner 的核心诉求是「NAME 必填、显式、每调用必给，且不与 content 混淆」，具名必填 flag 100% 满足该意图（必填性由 clap required 强制，显式性由 flag 名自带标签，混淆面结构性归零），且不再需要 v0.5 的三重教学补偿。

### 1.2 三条新文法规则的论证（替换 v0.5 三规则）

**规则 1（位置参数仅剩 PATH）**：PATH 是全 CLI 第一名词（文件即接口，ADR-011 路径显式），保持唯一位置参数；其余必填参数全部具名化。收益：任何命令的参数解析不再有「连续位置参数语义不可区分」问题（v0.5 混淆面的根源）。

**规则 2（必填与可选一律具名 flag）**：v0.5 规则 3（必填即位置参数）判据被 owner 裁决翻转：位置化的收益（少打 flag 名）不抵其混淆成本（错误注入矩阵第 3 行静默写入，研究文档 §3.2）。判据统一为「位置槽只留给无类型歧义的 PATH」，未来新命令无需再逐参数讨论。

**规则 3（flag 唯一语义，含显式登记例外）**：同一命令内任何 flag 只有一种含义；跨命令 `--to` 为显式登记的类型判别例外（rework 裁定 F1）：post send `--to` = 收件人名单（字符串列表，format-v2 已随 0.5.0 发布，保留不改名），post read `--to` = seq 上限（u64），clap 类型强制互不混用。具名化后该规则反而强化：`--author` 与 `--message` 以 flag 名自带语义标签，agent 首次调用即可从名称推断含义（SOTA 报告参数无歧义原则）。`--to` 例外的残留混淆面（send 方向数字串静默接受为收件人名）为已知行为登记而非缺陷，见 §2.1 与 bdd S-SEND-16。

### 1.3 与 v0.5_feedbacks 的冲突显式标注

v0.5_feedbacks §二.1（NAME 前置位置参数）与 v0_feedbacks #3.1（content 末位位置参数）被本设计直接翻转，依据为 owner v0.6 显式指令（v0.6_feedbacks §三翻转记录；v0.5_feedbacks §三 末尾已追加翻转记录）。v0.5 design.md 中凡依赖这两条的论证（规则 1 的 NAME 第 2 槽、规则 3 判据、§2.5 混淆面三重教学补偿、§2.1 参数序论证）一律以本文为准。

---

## 2. 逐 tool 动线与参数布局论证（仅变化命令；未列命令沿用 v0.5 design 对应章节）

### 2.1 post: send / edit 重排（设计重心）

动线不变：**send（高频）-> read/summary（高频）-> edit（低频纠错）**（post create 已由 format-v2 删除，建线程并入 send 自动创建）。

```
paperwork post send    <PATH> --author <NAME> (--message <BODY> | --stdin) [--reply-to N] [--mention a,b] [--title T] [--participants a,b] [--to a,b]
paperwork post read    <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]
paperwork post summary <PATH>
paperwork post edit    <PATH> --author <NAME> --seq <N> (--message <NEW_BODY> | --stdin)
```

布局理由：

- **PATH 保持唯一位置参数**：操作对象仍是命令行第一名词，agent 读到第 1 个参数即知目标文件（ADR-011）。
- **`--author` 具名必填**：署名语义由 flag 名自明；缺省落 usage exit 2 且 example 展示完整必填形态，「每发必给」的 owner 意图由 clap required 强制兑现。
- **`--message` 与 `--stdin` 二选一**：单行正文走 `--message`（flag 值直传，`-` 开头无需 `--` 边界）；多行大片内容走 `--stdin`（承接 v0_feedbacks #3.1 便于书写的精神，载体由位置槽改为管道）。互斥冲突在 clap conflicts 层判定，错误信号先于任何文件 I/O，落 usage exit 2。
- **edit 的 `--seq` 保留具名必填 flag**：v0.5 曾将 SEQ 位置化（理由：必填即位置），本版随规则 2 回到 flag 层；SEQ 是寻址参数（选哪条），flag 名自带「序列号」语义，u64 类型错误信号明确（非数字即 usage exit 2）。
- **read/summary 零改动**：`--from` 在 read 中为 seq 起点唯一语义；`--to` 在 read（seq 上限 u64）与 send（收件人名单）间为规则 3 登记例外（§1.2）：read 方向类型防线有效（`read --to bob` -> u64 解析失败 usage exit 2，显式信号）；send 方向无类型防线（任何字符串含数字串均为合法名字），登记为已知行为（spec §3.1，bdd S-SEND-16），本轮不改运行时行为。
- **建线程元数据载荷 `--title/--participants/--to`（rework 裁定 F6）**：三者仅在 send 自动建线程（首次写入）时生效；对既有线程附这三个 flag 时静默忽略（format-v2 冻结语义，exit 0 且无信号）。论证：输出协议冻结（spec §7）使「加警告字段」在本版不可行，故本轮以「文档声明 + help/SKILL.md 教学 + BDD 场景钉住」三件套补偿（spec §3.1 登记、impl_plan 步骤(2) after_help 教学、bdd S-SEND-17）；可检测化（ok 信封 ignored 字段增补）需解冻输出协议，列入 §8 未来工作项。附带区分教学：`--to`（收件人名单，仅建线程）/`--participants`（参与者名单，仅建线程）/`--mention`（提及名单，每条消息生效）三个名字列表 flag 语义邻近，after_help 与 SKILL.md 须给一行区分注记（Pete C1 附带混淆）。

错误指导样貌（example 全部为 v0.6 形态，每命令一条静态规范可执行示例，rework 裁定 F5）：

- 缺 `--author`（usage）：`example: paperwork post send standup.post.md --author alice --message "Hello"`
- 缺正文通道（usage）：同上形态（采 `--message` 通道为规范形态）；`--message` 与 `--stdin` 同给（usage）：example 同为该单一规范形态；「二选一」指引由 message/fix 文案承担，不在 example 中表达（v0.5 F2/F7 裁定延续）。
- 空正文（validation）：`example: paperwork post send standup.post.md --author alice --message "Hello"`（不再需要 NAME 槽位遗漏提示与 `--` 教学）。
- 线程不存在（not-found，read/summary/edit）：`fix: send a message first to create the thread` / `example: paperwork post send standup.post.md --author alice --message "Hello"`。
- edit 三重护栏（not-allowed）：message 精确文本不变，example 换 `paperwork post edit standup.post.md --author bob --seq 3 --message "corrected"`。
- v0.5 位置文法调用（`send <PATH> alice "Hi"`）：多余位置参数 -> usage 信封 exit 2 + 静态规范示例（机制沿用 v0.5 design §2.6）。

help / after_help 示例文案（语言沿用 v0.5 裁定：全部英文）：

```
Grammar: paperwork [global flags] <group> <verb> <PATH> --required flags [--optional flags]

# post send
Examples:
  paperwork post send standup.post.md --author alice --message "Parser module is 80% done."
  paperwork post send standup.post.md -a alice -m "Tests merged." --reply-to 2 --mention bob
  echo "multi-line body" | paperwork post send standup.post.md --author alice --stdin
  paperwork post send standup.post.md --author alice --message "-starts with dash is fine"
  paperwork post send new-topic.post.md --author alice --message "kickoff" --to bob,carol
  # --to (recipients, thread creation only) / --participants (participants, thread creation only) /
  # --mention (mentions, per message): three name-list flags; the first two are ignored on existing threads.

# post edit
Examples:
  paperwork post edit standup.post.md --author alice --seq 3 --message "corrected body"

# post read
Examples:
  paperwork post read standup.post.md --from 5 --to 20
  paperwork post read standup.post.md --mention alice --limit 20
```

Grammar 模板行必填段移出方括号（rework 修正，Pete N6）；send 示例补 `--to` 收件人语义演示与三 flag 区分注记（rework 裁定 F1/F6 教学要求，Pete C1）。

### 2.2 profile: create 回收具名

```
paperwork profile create <PATH> --name <NAME> [--model] [--description] [--scope-*]
```

- `--name` 回到具名必填 flag（v0.5 位置化的逆向）。理由：profile 名字不与 content 同槽，不存在 NAME/BODY 型混淆；owner 的 `--author` 裁决针对 post 署名歧义，不覆盖 profile（v0.6_feedbacks §2.4 补记）；规则 2 下必填参数一律具名 flag，判据自洽。
- show/edit/list 不变（无 actor 命令单位置参数，零特例，沿用 v0.5 design §3）。
- 错误示例：缺 `--name`（usage）`example: paperwork profile create agents/alice --name alice --model gpt-4o`。

### 2.3 brief: 主载荷回收具名

```
paperwork brief create <PATH> --title <T> [--owner] [--description]
paperwork brief add    <PATH> --entry <E> [--regex] [--note]
paperwork brief remove <PATH> --entry-title <T>
paperwork brief read   <PATH> [--full]
paperwork brief verify <PATH> [--base-dir <DIR>]
```

- `--title/--entry/--entry-title` 回到具名必填 flag（规则 2 直接推论，无需逐参数讨论）。
- read/verify 不变；三态判定契约冻结。
- add/remove 的 basename 推导规则（add 传相对路径、remove 传 basename）沿用 v0.5 spec §3.3，不受参数形态影响。

### 2.4 contacts: add 回收具名

```
paperwork contacts create <PATH> [--title]
paperwork contacts add    <PATH> --profile <P>
paperwork contacts read   <PATH>
```

- `--profile` 回到具名必填 flag；create 的 `--title` 因有默认值保持可选 flag（判据：可选才做 flag 的另一面在规则 2 下依然成立）。
- read 富化输出不变。

### 2.5 validate: 不变

`validate <PATH> [--type post|profile|brief|contacts]` 沿用 v0.5 design §6（格式防火墙入口 + --type 逃生门）。

---

## 3. 短形式设计论证

**收窄裁定（rework 裁定 F3，v0.6_feedbacks §2.2）**：全 CLI 仅保留 `--author/-a`、`--message/-m` 与既有全局 `-q`；初稿中实现方设计的一切其他短形式（`-r/-p/-t/-l/-d/-o/-n` 等）全部收回为仅长形式。

- `--author/-a`：全称首字母，全 CLI 无其他 `-a` 竞争。
- `--message/-m`：git `commit -m` 是 agent 训练语料中最强的「短 flag 传正文」惯例，迁移直觉成本最低。
- post read `--mention` **刻意不给短形式**：若给 `-m`，将与 send/edit 的 `--message` 短形式在 post 组内形成双义，违反规则 3 的短形式延伸约束（v0.6_feedbacks §2.2 裁定）；宁可牺牲低频过滤参数的输入便利。
- `--seq` 不给短形式：edit 是低频纠错路径，短形式反而扩大误触发面。
- **收窄理由（跨命令多义对 agent 泛化的影响，Pete M2）**：初稿全表存在四处跨命令同短形式异语义（`-m/-p/-t/-d`），agent 从全部 CLI 经验建立泛化模型（SOTA 结论 6，Trevin-6），命名不一致不会让 agent 失败、只会让它「缓慢成功」；尤其 `-m` 若同时绑 `--message` 与 profile `--model`，agent 带着 git 直觉在 profile create 上按 message 预期使用 `-m` 会得到 model 语义，期待错位。收窄后短形式集合极小且全 CLI 语义唯一，白名单断言（S-SHORT-02）重获防线意义；输入效率损失有限（其余 flag 均低频可选，全称可复制粘贴）。

**token/调用长度量化（Pete N1）**：典型 send 调用前后对比：v0.5 `paperwork post send standup.post.md alice "Parser done"`（55 字符）-> v0.6 全称形态 `paperwork post send standup.post.md --author alice --message "Parser done"`（74 字符，+19 字符约 5 token）；短形式形态 `paperwork post send standup.post.md -a alice -m "Parser done"`（61 字符，+6 字符约 2 token）。增量为每次调用的一次性常数，不随正文长度增长；多行正文经 `--stdin` 管道承接，反而省去引号与转义开销。结论：token 经济学维度可接受，正面闭合 SOTA 报告的核心证据维度。

---

## 4. path-first 复评否决记录

完整复评论证见研究文档 §4，此处仅记结论以供追溯：

| 形态 | 结论 | 关键理由 |
|---|---|---|
| 完全 path-first（`paperwork <PATH> [<NAME>] <verb> ...`） | 维持否决 | 后缀路由需路由前 I/O 且裸 `.md` 判型引入新静默错误；clap help/usage/error 生成全部丧失需手写；`profile list`/`validate` 无文件实例无处路由 |
| 组内路径先行折中（`paperwork post <PATH> send ...`） | 维持否决 | 虽免后缀路由，仍需绕过 clap subcommand 层级手写分发与 help；槽位破坏依旧；help/error 表面积大于 action-first |

v0.5 design §1.1 四点否决论证经本轮复评全部成立；owner 裁决接受 action-first，该分歧结案。v0.5 design §8 #1 中「保留为 v0.6 可选快捷前缀层提案」的尾巴一并撤销：owner 已明确接受 action-first 为基本设计，不再预留 path-first 提案位。

---

## 5. 三方案对比与 owner 裁决依据

三个研究员的重评估方案（研究文档 §3）收敛过程：

| 方案 | 提出者 | 核心主张 | 与最终裁决的关系 |
|---|---|---|---|
| (d) NAME 回退 `--from` flag、BODY 留位置槽、read 改 `--after/--before` | Sena | 结构性消除混淆面、clap 全量利用 | BODY 具名化方向被采纳；`--from` 复用被 owner 的 `--author` 全称指定取代；read 改名未被要求（`--from` 唯一语义已成立） |
| (D) NAME 留位置槽、BODY 改 `--body` flag | Vera | 消除错误矩阵第 3 行静默写入，同时满足当时 owner 位置化指令 | BODY flag 化被采纳，全称由 owner 指定为 `--message`；NAME 留位置槽的过渡形态被 owner 直接越过 |
| (d) BODY 转 `-m/--message` flag，最小 diff（约 60 行） | Milo | 改动量最小、风险最低 | flag 名与短形式 `-m` 与最终裁决逐字一致 |

**裁决逻辑**：三份报告对「混淆面必须结构性消除」一致，分歧仅在 NAME 是否暂留位置槽；owner 在「name 确实会和 content 歧义」的认知上直接裁定 NAME/BODY 双双具名化，一步到位越过过渡形态。三方案的合流即最终方案（spec §2 全表）。

---

## 6. `--message` 与 `--stdin` 互斥语义设计

- **二选一必填，两形态均在 clap 解析层判定（rework 裁定 F2）**：clap 侧 `--message` 与 `--stdin` 互斥（conflicts_with），且 `--message` 设 `required_unless_present = "stdin"`。四种形态：同给 -> ArgumentConflict，usage exit 2；皆缺 -> MissingRequiredArgument，usage exit 2；仅 message / 仅 stdin -> 通过。两形态均自然走 try_parse 失败分支落 usage 信封，**命令层无需任何管道**（与 usage 信封机制冻结、spec §7 破坏面限定自洽；修正初稿「由命令层报缺必填」的层级归属错误，Quinn M-1）。
- **错误层级提升的论证**：v0.5 该冲突在运行时判定（validation exit 1），因为位置 BODY 是否出现需解析完成后才可知；v0.6 两者皆为 flag，冲突在 clap 解析层即可判定，归入 usage 层（clap 层用法错误）更精确，且与「缺必填 flag = usage」的判据自洽。
- **语义优先级声明**：编排层裁定「stdin 优先，同时给报 usage 错误」；实现上不存在真正的优先级分支（同时给直接拒绝），该表述的规范含义是「不得静默择一」，显式优于隐式。
- 仅 `--stdin`：正文从 stdin 读取（行为沿用 v0.5）；仅 `--message`：flag 值即正文，trim 后为空落 validation exit 1（空正文拒绝沿用）。
- **`-` 开头正文直传的 clap 前提（rework 裁定 F4，Quinn C-1）**：clap 4 默认 `allow_hyphen_values = false`，`--message` 后跟 `-` 开头 token 会被拒收为解析错误；故 send/edit 两处的 `--message` Arg 必须设 `allow_hyphen_values = true`，否则 bdd S-SEND-10 不可兑现（v0.5 时代该问题被 `--` 边界机制掩盖，本版废止 `--` 教学后此属性为硬前提）。副作用边界登记：`--message --stdin` 连写时 `--stdin` 会被吞为正文值，属显式输入，不另设护栏（两者同给的正确写法顺序不受影响，conflicts 判定仍对独立 token 生效）。其余 flag（`--author/--seq/--to/--participants/--mention` 等）不设该属性：值域无 `-` 开头合法形态，设置反扩大误解析面。
- **usage 信封 message 字段附带说明（Pete N5）**：clap 报错原文本已携带多余位置参数值（如 unexpected argument 'alice'），信封 message 取自 clap 渲染文本即自然携带，agent 可完成「alice -> --author alice」映射；不额外实现任何值重建（与 v0.5 F2 静态示例裁定不冲突：message 与 example 是两个字段）。

---

## 7. 对 v0.5 Rejected Alternatives 的状态更新表

沿用研究文档 §7 的再评估结论（该表为权威版本）：

| v0.5 design §8 条目 | v0.6 状态 | 说明 |
|---|---|---|
| #1 path-first 字面文法 | 维持否决 | 复评四点理由成立；owner 接受 action-first；v0.6 快捷前缀层提案位撤销 |
| #2 隐藏弃用别名窗口 | 维持否决 | 双文法表面积论证不变 |
| #3 `--as` flag 方案 | 被翻转 | owner 显式裁决名字具名化（全称改 `--author`） |
| #4 SEQ 保留 flag | 被翻转（改判采纳） | 规则 2 下 SEQ 回归必填 flag `--seq` |
| #5 `--seq-from/--seq-to` 改名 | 维持否决 | `--from` 唯一语义前提不变 |
| #6 usage 信封 argv 值迁移重建 | 维持否决 | 静态规范示例裁定继续有效 |
| 新增：BODY/MESSAGE 具名化 | 采纳 | Vera/Milo 方案合流，owner 指定全称 `--message` |
| 新增：NAME/AUTHOR 具名化 | 采纳 | owner 显式裁决 `--author` |

---

## 8. 遗留项裁决（沿用 v0.5）

v0.5 design §7 遗留项裁决总表**整体沿用**，本轮不受影响：

- 延后项继续延后：U-03/R-01/N-03（线程创建双轨）、U-04（正文 @mention 自动提取）、U-09（summary 并入 read）、U-13（shell completions）。format-v2 已删除 post create，U-03 的「双轨」只剩 send 自动创建单轨，该议题随 format-v2 治理线另行结案，不在本轮。
- 拒绝项维持拒绝：U-02（env 回退）、U-05（content-first/路径可省略）、R-08（--no-color）、F-09（正文 markdown 校验）。其中 U-05 与本轮 PATH 唯一位置参数、PATH 恒必填的规则一致。
- v0.5 §7.4 规格模糊点裁定（usage category、implicit-mention 形态、showing/window 形态、exit_code 如实、command=usage、ensure_suffix 三级、help 英文）全部继续有效。
- **本轮新增登记（rework 裁定 F6 与 Pete N3）**：
  - send 元数据 flag（`--title/--participants/--to`）对既有线程静默忽略的可检测化：未来工作项为 ok 信封增补 `ignored` 字段（如 `ignored: title,participants`），需解冻输出协议（JSON 只增不改不删约束下可 additive 实现），由发布轮或后续 UX 线另行裁决；本轮仅行为登记与教学（spec §3.1、bdd S-SEND-17）。同批评估候选（Pete C1 建议(2)）：ok 信封回显收件人名单使 send `--to` 数字串误用可检测（与 S-SEND-16 已知行为登记配套），同样需解冻输出协议，本轮不实现。
  - `--reply-to` 指向不存在 seq 静默跳过（冻结沿用）与 Q-02 的张力：已登记 `docs/researches/ux-open-items-backlog-2026-08-08.md`，供发布轮或后续 UX 线裁决。

---

## 9. 兼容策略

- 干净切断、usage 信封迁移教学、SKILL.md 三件套：见 spec §8（立场与 v0.5 design §9 一致）。
- **本轮不发布**（owner 显式约束）：版本 bump、tag、publish、CHANGELOG 发布段全部延后；版本号与发布时机由 owner 在功能稳定后另行裁定（v0.6_feedbacks §一 (3)）。impl_plan 因此不含任何发布步骤。
- adr-v1.md 示例层 Superseded-by 注记（v0.5 impl_plan 步骤(7) m6 确立的做法）在本轮实现完成后同样追加一行指向本目录 spec.md，不改写历史内容。

---

## 10. SOTA 结论采纳状态表（rework 补录，Pete N4）

对照 `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` 结论编号逐条结案：

| SOTA 结论 | 状态 | 去向 |
|---|---|---|
| C1（混淆面结构性消除方向） | 采纳 | 被 owner 裁决直接越过过渡形态（NAME/BODY 双具名化）；design §7 记录在案 |
| C2（枚举合法取值） | 不适用 | send 不与 profile/contacts 做 author 存在性校验（spec §3.1），无值域枚举需求，一句话结案 |
| C3（同名 flag 冲突消解） | 采纳 | `--from` 冲突经具名化消解；`--to` 跨命令双语义经 rework 裁定 F1 登记为类型判别例外（spec §1.4），非完全消解但已钉住（bdd S-SEND-16/S-READ-08） |
| C4（输出四档） | 采纳 | `--json/--plain/-q` 与默认档冻结继承（spec §5/§7） |
| C5 前半（SKILL.md 迁移补偿） | 采纳 | 三件套迁移教学（spec §8），SKILL.md 刷新入 impl_plan 步骤(6)，在场性盘点入步骤(0)（Pete N7） |
| C5 后半（机器可读内省 agent-context/--help --json） | 拒绝 | 维持 v0.5 静默放弃：help 表面积与维护成本权衡，本轮不引入新内省通道 |
| C6（词汇一致性/命名政策） | 采纳 | S-SHORT-02 命名政策白名单断言；rework 裁定 F3 收窄短形式后跨命令多义消除 |
| C7（退出码分级细化） | 拒绝 | 退出码维持 0/1/2 三档（输出协议冻结，spec §7）；错误信息区分度由七类 category 承担 |
| C10（example 永远可复制执行） | 采纳 | example 全具体值、禁占位符（spec §5 第 2 条、bdd S-SEND-05/S-SEND-12 断言）；rework 裁定 F5 重申每命令一条静态规范示例 |
