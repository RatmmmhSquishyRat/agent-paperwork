# CLI 文法 v0.6 文档集 — agent 消费者视角对抗性评审

- 日期：2026-08-09
- 任务：cli-grammar-v0.6 文档集对抗评审，agent 消费者视角（唯一真实用户是 AI agent）
- 评审对象：docs/ssot/specs/cli-grammar-v0.6/{spec,design,bdd,tdd,impl_plan,README}.md
- 评审基准：docs/researches/agent-cli-ux-industry-sota-2026-08-08.md（SOTA 结论逐条核对）；docs/researches/cli-grammar-v06-reassessment-2026-08-09.md（错误注入矩阵与混淆面分析）；docs/ssot/adr/feedbacks/v0.6_feedbacks.md；docs/ssot/adr/agent-ux-qol.md（Q-01~Q-04）；v0.5 文档集 docs/ssot/specs/cli-ux-redesign/（继承基线核验）
- 评审方法：模拟一个 LLM agent 凭 --help / SKILL.md / usage 信封学习并使用 v0.6 文法的全过程，逐命令推演典型调用与错误重试路径；关键结论均以文档原文为证据
- 评审范围声明：只看 agent 消费体验维度（一次生成正确率 / 错误恢复回路 / 迁移成本 / token 效率 / Q-01~Q-04 兑现），不审 SSOT 程序合规与代码实现细节
- 立场声明：对抗性评审，非背书

## 总体结论

v0.6 文法方向正确且对 agent 显著友好：NAME/BODY 双双具名化使混淆面结构性归零（Vera 矩阵第 3 行静默写入路径不可达，S-SEND-15 已钉住），错误恢复回路在缺必填/互斥冲突/旧文法三类失败上均一次重试可修复，迁移教学三件套闭环且 SKILL.md 刷新已进 impl_plan。但存在一处 spec 内部自相矛盾并伴随新增静默错误面（C1：规则 3 表述与 post send `--to` 签名冲突）、三处 Major 级契约/政策缺陷（M1 退出码契约两份权威文档分歧、M2 短形式政策自相矛盾、M3 建线程载荷静默忽略无声明）。判定：**不闭合，需 rework**；rework 范围限于文档修正（spec/design/bdd 文本 + 裁定记录），文法总体设计不需推翻。

---

## Issue 清单

### C1（Critical）spec §1.4 规则 3 表述与 §2 签名自相矛盾：post send 的 `--to` 与 post read 的 `--to` 语义、类型均不同；send 方向构成静默接受

证据：spec §1.4 原文「`--from/--to` 仅存于 post read，仅表 seq 范围（v0.5 已确立，不变）」；但同文 §2 全表与 §3.1 的 post send 签名含 `[--to a,b]`（format-v2 建线程收件人载荷，逗号分隔名字列表）。该表述逐字承袭 v0.5 spec §1.3（核验：v0.5 时 post 组确无 send `--to`），format-v2 并入签名表后未同步修订。design §2.1「read/summary 零改动」一节「`--from/--to` 在 v0.5 已成为 seq 范围唯一语义，本版无冲突复发可能」的论证随之失效。v0.6_feedbacks §2.1(3) 已作出「`--to` 在 post send（收件人）与 post read（seq 终点）两命令中语义各自唯一，不构成跨命令双义」的裁定，但该裁定未载入 spec §1.4，spec 内部矛盾原样保留。

agent 复现路径（静默方向）：agent 从 read 习得 `--to 20`（seq 终点）后，建线程时输入 `paperwork post send newtopic.post.md --author alice --message "hi" --to 5`。send 的 `--to` 接受名字列表，"5" 被静默接受为收件人并写入线程 participants——exit 0，错误数据落盘，无任何信号。这与本版以结构性手段消除的 Vera 错误注入矩阵第 3 行（静默写入错误 sender）是同一失败类别：v0.6 一边消除旧静默面，一边在并入 format-v2 时引入新静默面，且 spec 规则 3 的字面声明恰好掩盖了它。
agent 复现路径（显式方向）：`post read f.post.md --to bob` -> u64 解析失败，usage exit 2，一次重试可修复。双向不对称：错一半报得响，错另一半静悄悄。

附带混淆：send 的 `--to a,b`（收件人）/`--participants a,b`（参与者）/`--mention a,b`（提及）三个名字列表 flag 语义邻近，全套文档未给出区分教学。

建议：① spec §1.4 修订表述，把 `--to` 按命令唯一语义的裁定载入规则 3 边界裁定（与 `--mention` 设置/过滤裁定同格）；② design §2.1 补 `--to` send/read 混淆面论证，明确 send 方向类型差异不构成防线（"5" 是合法名字字符串），并裁定是否对 send `--to` 值域做最低校验或至少在 ok 信封回显收件人使错误可检测；③ BDD 补两场景：`send --to 5` 行为断言（静默接受现状则显式声明为契约）、`read --to bob` usage exit 2；④ after_help 与 SKILL.md 的 send 示例补一条 `--to` 收件人语义演示，并与 `--participants/--mention` 的区分写一行注记。

### M1（Major）「`--message` 与 `--stdin` 两者皆缺」的退出码契约在最高优先级文档与 spec 之间分歧，无裁定记录

证据：v0.6_feedbacks §2.3 原文「两者皆缺：落 validation 错误（exit 1），example 展示 --author + --message 完整形态」；spec §3.1「两者皆缺 -> usage exit 2（缺必填）」、spec §5 第 1 条、design §6、bdd S-SEND-06/S-EDIT-04 均为 usage exit 2。spec 文首声明 feedbacks「最高优先级，冲突处以该文件为准」，却未经任何裁定记录而偏离其字面条款——恰好复刻了本项目治理中最危险的「权威文档互相矛盾，实现者自行发挥」模式。

agent 影响：按 exit_code 分支的 harness（如 `exit_code == 2 -> 修参数重试`、`== 1 -> 修数据重试`）将面对两份文档给出的两个契约；v0.5 评审 M2 已警告过这类解析器写死场景。行为本身两个方向都一次重试可修复（example 均给完整形态），故不升 Critical，但契约歧义必须消除。

建议：spec §5 增补裁定条款——flag 层「正文通道缺失」与「缺 --author」同构，归 usage exit 2，feedbacks §2.3 原条款系 clap 层判定前的初步解读，以 spec 裁定为准；并同步在 v0.6_feedbacks §2.3 追加一行指向该裁定的记录（或按治理规则由编排层确认修订）。

### M2（Major）spec §4 短形式政策自相矛盾：原则声明「全 CLI 短形式语义无冲突」，全表却有四处跨命令多义

证据：spec §4 原则原文「高频必填 flag 给短形式；全 CLI 短形式语义无冲突」。同表实际分配：`-m` = post send/edit `--message` 与 profile `--model`；`-p` = post send `--participants` 与 contacts add `--profile`；`-t` = post send/brief create `--title` 与 validate `--type`；`-d` = profile/brief `--description` 与 brief verify `--base-dir`。四处跨命令多义，原则与全表直接冲突。

agent 影响：SOTA 结论 6（词汇一致性，Trevin-6：agent 从全部 CLI 经验建立泛化模型，命名不一致不会让 agent 失败、只会让它「缓慢成功」）在本项目内部被自己违反。`-m` 尤其刺眼：它以「git commit -m 行业惯例、迁移直觉成本最低」为由分配给 `--message`，同一理由却被 profile 的 `--model` 稀释——agent 带着 git 直觉在 profile create 上按「message」预期使用 `-m`，得到的语义是 model。误期待的实际后果是「恰好正确」（各命令内短形式唯一，调用仍能成功）或显式 usage 错误，无静默失败，故评 Major 而非 Critical；但规范自相矛盾使 bdd S-SHORT-02 的白名单断言失去防线意义（断言只核对表格，表格本身违反原则），且直接侵蚀 Q-03（语义一致）。

建议：二选一——① 修订原则为「单命令内唯一；跨命令多义允许但逐处记录在表」（诚实化现状）；或 ② 修改冲突短形式（`-m` 因 git 惯例绑定最强，建议保留 message、放弃 profile `--model` 短形式，其余三处至少记录取舍）。无论哪个方向，design §3 短形式论证须补「跨命令多义对 agent 泛化的影响」一段。

### M3（Major）post send 的 `--title/--participants/--to`「仅建线程时生效」意味着对既有线程静默忽略：无声明、无教学、无场景

证据：spec §3.1 行为保留一句「`--title/--participants` 仅在建线程时生效（format-v2 语义）」，未声明其对既有线程的行为。按 format-v2 语义，agent 对已存在线程附 `--title "New Title"` 发送 -> exit 0，标题未变，无任何信号。这是「agent 给了参数却被静默忽略」，直接违反 Q-02（操作失败是否能够快速得知，而不是靠自己辛苦探查）。

agent 复现路径：agent 想改线程标题，凭「send 签名里有 --title」推断 send 可改标题，调用成功返回却一无所获；下一步只能靠 summary 回读才发现标题未变，浪费至少一轮调用与一次回读。输出协议冻结（spec §7）使「加警告字段」不可行，但文档声明、help/SKILL.md 教学与 BDD 场景三件事都可行且都缺失。

建议：① spec §3.1 显式声明「对既有线程附 `--title/--participants/--to` 时静默忽略（format-v2 冻结语义），改标题不在本版能力范围」；② design §2.1 补一段论证；③ BDD 补场景（既有线程 + --title -> exit 0 且标题不变，作为冻结契约钉住）；④ after_help 的 send 示例注记「--title/--participants 仅在自动建线程时生效」。

---

## Minor 问题

N1（Minor）token/调用长度增长无量化论证。实测估算：`post send f.md alice "hi"`（v0.5，7 token 级）-> 全称形态增 `--author` 与 `--message` 约 19 字符（约 5 token）；短形态 `-a/-m` 约 6 字符（约 2 token）。增量是每次调用的一次性常数，不随正文长度增长（多行正文由 --stdin 管道承接，反而省引号转义成本），结论可接受；但 SOTA 报告以 token 经济学为核心证据（Infracost 结论），design 对此只字未提。建议 design 补一段量化，正面闭合该维度。

N2（Minor）BDD 缺口清单：① `--author "   "`（trim 后为空 -> validation exit 1）spec §3.1 有条款而无场景；② post edit 仅 `--stdin` 成功路径无场景（S-EDIT-01 只走 --message，send 侧有 S-SEND-08 而 edit 侧缺）；③ v0.6 文法集内缺 PATH 场景（如 `post send --author alice --message hi`）未列；④ `post read f.md --author alice`（写命令习惯迁移：想按发件人过滤）无场景——该混淆的恢复路径依赖 read after_help 已有 `--mention` 示例（design §2.1，好），建议 usage 信封对 read 内未知 flag 的 fix 文案顺带点一句 sender 过滤用 `--mention`；⑤ `--json` x conflicts 组合无场景（S-OUT-03 只覆盖多余位置参数形态）。

N3（Minor）`--reply-to` 指向不存在 seq「静默跳过」（spec §3.1 错误映射，沿用 v0.5）与 Q-02 存在张力：agent 回复一个不存在的 seq，消息照常落盘但 reply 关系丢失且无信号。属冻结行为，本轮不强制改；建议记入 docs/researches/ux-open-items-backlog，供发布轮或后续 UX 线裁决。

N4（Minor）SOTA 结论采纳/拒绝状态记录不全。已兑现：C1（被 owner 裁决越过，design §7 记录在案）、C3（--from 冲突经具名化消解——但 C1 指出 --to 使该成果部分回退）、C4（输出四档冻结继承）、C6（S-SHORT-02 命名政策白名单断言，明确标注 SOTA C6，好）、C10（example 全具体值、无尖括号占位符，S-SEND-05/S-SEND-12 明确断言「具体可执行值」）。缺记录：C5 后半（机器可读内省 agent-context/--help --json）与 C7（退出码分级）继续静默放弃（v0.5 评审 N6 的残留未闭合）；C2「枚举合法取值」在「send 不与 profile/contacts 做 author 存在性校验」（spec §3.1）的语义下实际不适用，建议一句话结案。建议 design §8 或 README 补一张 SOTA 结论状态表，逐条标注采纳/拒绝/不适用及去向。

N5（Minor）迁移信封可再降一层成本。v0.5 位置文法调用（S-SEND-12 等五场景 + S-SEND-13 的 v0.4 --from）落 usage exit 2 + 静态规范示例，一次重试闭环成立；但 clap 原生错误本已携带多余参数文本（如 unexpected argument 'alice'），信封 message 若能点名该值（不违反「不携带用户原参数值进 example」的静态示例裁定——message 与 example 是两个字段），agent 从「alice -> --author alice」的映射成本可再降。建议 impl_plan 步骤(3) 补此口径。

N6（Minor）Grammar 模板行表述瑕疵。顶层 help 的 Grammar 行 `paperwork [global flags] <group> <verb> <PATH> [--required flags] [--optional flags]`（spec §1.1、design §2.1、impl_plan 步骤(3) 三处）把必填 flags 放在可选方括号内，agent 直读易判为可选。建议改为 `[--required named flags] ...` 之外的显式形态，如 `<PATH> --required-flags [--optional-flags]` 或对必填段去括号加注 (required)。属文案级修正，但恰是 agent 学习文法的第一行。

N7（Minor）SKILL.md 在场性未纳入基线盘点。当前工作树（master + format-v2 脏变更）无 SKILL.md（实测检索为空），迁移三件套与 impl_plan 步骤(6) 都假设它随 cli-ux-v0.5 分支合并进入基线；但 impl_plan 步骤(0) 的两项盘点（core example 点位、cli 集成测试调用点）未把「确认 SKILL.md 在场、否则随合并引入」列为验收项。建议步骤(0) 补一条，避免步骤(6) 刷新对象缺失而静默跳过——那会让迁移教学三件套断一条腿。

---

## 评审焦点逐条回答

### 1. 一次生成正确率

结论：**显著提高，混淆面主体结构性归零；残留两处新引入的混淆，一处含静默方向（C1）**。

逐命令推演（LLM agent 视角）：
- post send：`--author/--message` 自带语义标签，agent 首次调用即可从 flag 名推断含义（符合 Anthropic 参数命名无歧义原则）；漏 NAME 被 body 吞掉的三类形态（Vera 矩阵 1/2/3 行）全部变为显式 usage exit 2 或结构性不可达（S-SEND-15 钉住静默路径）。`--message` 值以 `-` 开头无需 `--` 边界（S-SEND-10），v0.5 的 `--` 摔跤点消失。
- `--message`/`--stdin` 互斥报错一次重试可修复：同给 -> S-SEND-07（conflicts，example 给二选一形态）；皆缺 -> S-SEND-06（example 给二选一完整形态）。两条路径 example 均为具体值，成立。
- post edit：`--seq` 类型差防线保留（S-EDIT-06 非数字 -> usage）；三重护栏违规的 example 直接给出正确 author（S-EDIT-07），一次修复。
- profile/brief/contacts：主载荷具名化后无连续位置参数语义不可区分问题；`profile create` 用 `--name` 而非 `--author` 的区分有 v0.6_feedbacks §2.4 补记支撑，agent 从 post 习惯误用 `--author` 时落显式 usage + `--name` 示例，可恢复。
- 残留混淆一：`--to` 跨命令双语义含静默方向，见 C1。残留混淆二：`--author`（写命令身份）与 read 侧 `--mention`（过滤）语义邻近，agent 可能在 read 上试 `--author` -> 未知 flag 显式 usage，read after_help 有 `--mention` 示例兜底，两次调用内恢复（满足 SOTA 原则 5），评 Minor（N2-④）。残留混淆三：短形式跨命令多义的期待错位，见 M2。

### 2. 错误恢复回路

结论：**主回路闭合，全部三类失败一次重试可修复；三处洞见 M1/M3/N2**。

- 漏必填：每个缺必填 flag 场景（S-SEND-05/06、S-EDIT-02/03/04、S-PROF-02、S-BRIEF-03、S-CONTACTS-03）均断言 usage exit 2 + 该命令完整必填形态的具体值 example，符合 SOTA 结论 10「缺参错误给一条完整正确命令、永远可复制执行」，且尖括号占位符已被明确禁止（对比 v0.5 评审 M6 的问题，本版已修复）。
- 互斥冲突：S-SEND-07/S-EDIT-05 覆盖，message 说明两者不可同时给出 + 二选一 example，成立。
- 旧文法调用：v0.5 位置文法五命令迁移场景（S-SEND-12/S-EDIT-08/S-PROF-03/S-BRIEF-04/S-CONTACTS-04）+ v0.4 `--from` 迁移链延伸（S-SEND-13）+ --json 形态（S-OUT-03）全覆盖；「多余位置参数 + 静态规范示例」机制下 agent 需手动完成「值 -> flag」映射，成本存在但一次重试足够（维持否决 argv 值重建的裁定自洽）。
- 洞：M1（皆缺通道的退出码契约歧义）、M3（--title 类载荷静默忽略根本不进入错误回路）、N2 的场景缺口。

### 3. 迁移成本

结论：**教学闭环成立，SKILL.md 刷新已进 impl_plan；两处补强点见 N7/N5**。

- spec §8 明确三件套（usage 信封 + SKILL.md + after_help）承担迁移教学，依据 SOTA 结论 C5；design §2.1 的 after_help 文案全部换 v0.6 形态且含 `--stdin` 管道与 `-` 开头直传示例。
- impl_plan 步骤(6) 明确「SKILL.md（英文）速查表与典型调用示例全部换 v0.6 文法，错误自愈提示更新（旧文法迁移教学示例换为 v0.5->v0.6 形态）」，并覆盖根 README 与 cli README；步骤(5) CI smoke 保留「旧文法触发 usage 信封 exit 2」的断言型 smoke 且触发样例改为 v0.5 位置文法——迁移防线进了 CI。闭环成立。
- 缺口：SKILL.md 当前不在工作树，其在场性依赖基线合并，步骤(0) 盘点未覆盖（N7）；信封 message 可点名多余位置参数值以再降映射成本（N5）。
- CHANGELOG 披露延后至发布轮（spec §8 末条）与「本轮不发布」约束自洽，无缺口。

### 4. token/调用效率

结论：**可接受，但文档缺量化（N1）**。全称形态每次调用增加约 19 字符（约 5 token），短形态约 6 字符（约 2 token）；增量为一次性常数，正文越长占比越小，且多行正文经 --stdin 管道承接免去引号与转义开销。短形式覆盖评估：`-a/-m` 覆盖两个最高频必填 flag 且绑定 git 惯例（迁移直觉成本最低，SOTA §2.2/§5.1 支撑）；`--seq/--stdin` 不给短形式的理由（低频纠错、开关自明）成立。唯一代价是 M2 指出的短形式跨命令多义——效率设计反噬了语义一致性，应在修复 M2 时一并权衡。

### 5. Q-01~Q-04 承诺兑现核验

- Q-01 快速知结论：**兑现**。conclusion 首行、ok 信封、showing/window 恒显全部冻结继承（spec §5/§7），usage 错误也有信封 + category 首行。
- Q-02 失败自愈：**大部分兑现**。新文法的全部显式失败类别（缺必填/冲突/多余位置/类型错）均一次重试可修复；两个洞都是「静默」类：M3（建线程载荷对既有线程静默忽略）与 N3（--reply-to 静默跳过，冻结沿用）——静默恰恰是 Q-02 的对立面，建议至少文档声明 + backlog 记录。
- Q-03 语义一致：**部分兑现，本版最大扣分项**。规则 3 字面与 --to 现实矛盾（C1）、短形式政策自相矛盾（M2）；正面分：`--author/--message` 全 CLI 唯一语义、`--from` 唯一语义保持、`--mention` 设置/过滤裁定载入 spec §1.4、S-SHORT-02 白名单断言。
- Q-04 操作轻松：**兑现**。短形式、--stdin 管道、二选一正文通道、`--` 边界需求消亡（spec §5 第 3 条），操作面净简化。

---

## 是否闭合

**不闭合，需 rework。** 计数：Critical 1，Major 3，Minor 7。

### 必须修复清单（阻塞 impl_plan 前置门槛）

1. C1 — spec §1.4 与 §2 的 `--to` 矛盾：载入 per-command 唯一语义裁定、修订 design §2.1 论证、补 BDD 双场景、裁定 send `--to` 静默接受是否可检测化
2. M1 — 消除 feedbacks §2.3 与 spec §3.1/§5 的退出码契约分歧（spec 补裁定条款 + feedbacks 追加指向记录）
3. M2 — spec §4 短形式原则与全表二选一对齐（修订原则或修改冲突短形式），design §3 补跨命令多义论证
4. M3 — spec §3.1 声明建线程载荷对既有线程静默忽略，补 BDD 场景与 after_help/SKILL.md 教学

### 建议修复（不阻塞，建议随必须修复一并完成）

N1~N7；其中 N2（BDD 缺口）与 N7（SKILL.md 在场性盘点）直接影响步骤(0)/(4) 的实施质量，建议提高优先级。

### 附注

rework 范围限于文档层修正：新文法三规则、具名化方向、usage 信封迁移教学、输出协议冻结策略本身均通过 agent 消费视角检验，无需推翻。v0.5 评审遗留的 example 占位符问题（该文 M6 / SOTA 结论 10）在本版已确认修复。

---

## Rework 回应（2026-08-09 追加，rework 轮销账记录）

| 问题 | 处置 | 落点 |
|---|---|---|
| C1（--to 规则矛盾与静默面） | 按编排层裁定 F1 处理 | send `--to`（收件人名单，format-v2 已随 0.5.0 发布）保留不改名；规则 3 改写为「同一命令内任何 flag 只有一种含义；跨命令 --to 为显式登记的类型判别例外」（v0.6_feedbacks §2.1(3)、spec §1.4/§3.3、design §1.2/§2.1）；双向场景：bdd S-SEND-16（静默接受登记为已知行为）与 S-READ-08（显式方向 usage exit 2）；附带混淆（三名字列表 flag 区分）落 design §2.1 与 impl_plan 步骤(2) after_help 教学；建议(2)值域校验/回显收件人：不采纳为本轮实现（输出协议冻结），回显候选与 F6 的 ignored 字段同批挂 design §8 未来工作项 |
| M1（退出码分歧） | 按编排层裁定 F2 处理 | 两者皆缺 -> usage exit 2，clap `required_unless_present` 组合实现（命令层无管道）；v0.6_feedbacks §2.3 修正 exit 1 表述并附裁定补记；spec §3.1/§5、design §6、bdd S-SEND-06/S-EDIT-04、tdd §4 统一为 usage exit 2 |
| M2（短形式政策自相矛盾） | 按编排层裁定 F3 处理 | 短形式收窄为仅 `-a/--author`、`-m/--message` 加既有全局 `-q`，其余全部 flag（含 `-m`/`-p`/`-t`/`-d` 四处冲突点）收回仅长形式；spec §4 全表重写、design §3 补跨命令多义对 agent 泛化影响论证与收窄理由、bdd S-SHORT-01/02 更新（短形式集合 {-a,-m,-q} 与全量无短形式负向断言）、v0.6_feedbacks §2.2 同步；「全 CLI 短形式语义无冲突」在新表下自然成立 |
| M3（元数据 flag 静默忽略） | 按编排层裁定 F6 处理 | 本轮不改运行时行为；spec §3.1 补三 flag 参数契约（含「仅首次写入生效、既有线程静默忽略」行为登记）；design §2.1 补论证（三件套补偿）；bdd S-SEND-17 钉住；after_help 教学要求入 impl_plan 步骤(2)；可检测化（ignored 字段）列 design §8 未来工作项 |
| N1（token 量化） | 已修复 | design §3 补典型 send 调用前后字符对比（v0.5 55 字符 -> 全称 74/+19 约 5 token；短形式 61/+6 约 2 token），正面闭合 token 经济学维度 |
| N2（BDD 五处缺口） | 已修复 | (1) S-SEND-18（--author 空值 validation）；(2) S-EDIT-09（仅 --stdin 成功）；(3) S-SEND-19（缺 PATH usage）；(4) S-READ-09（read --author 习惯迁移，fix 点名 --mention）；(5) S-OUT-06（--json x conflicts）；tdd §4 同步补用例行 |
| N3（--reply-to 静默跳过） | 已修复 | 登记 docs/researches/ux-open-items-backlog-2026-08-08.md §八 B-01（冻结行为、本轮不改、候选方向与 F6 ignored 字段同批评估）；design §8 同步登记 |
| N4（SOTA 状态表） | 已修复 | design 新增 §10 SOTA 结论采纳状态表，逐条结案（含 C2 不适用一句话结案、C5 后半/C7 拒绝去向） |
| N5（message 点名多余参数值） | 已修复（不额外实现） | design §6 注明 usage 信封 message 字段天然携带 clap 报错原文中的多余参数值（message 与 example 是两个字段，不违反静态示例裁定）；impl_plan 步骤(3) 同步注明，无需额外实现 |
| N6（Grammar 模板行必填段） | 已修复 | spec §1.1、design §2.1、impl_plan 步骤(3) 三处必填 flags 移出方括号 |
| N7（SKILL.md 在场性盘点） | 已修复 | impl_plan 步骤(0) 补第三项盘点（SKILL.md 在场性确认与刷新清单输出，缺失时报告并以实际在场文件为准） |

遗留未决项：0。全部修正均为文档层变更，未触碰代码/CI/CHANGELOG。

---
（报告完）
