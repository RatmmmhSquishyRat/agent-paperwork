# CLI 文法 v0.6 文档集对抗性评审 - SSOT/pillars 合规视角

- 日期：2026-08-09
- 评审视角：SSOT/pillars 合规（第三方批判性评审，立场为挑错而非背书）
- 评审对象（全部精读）：docs/ssot/specs/cli-grammar-v0.6/ 下 spec.md / design.md / bdd.md / tdd.md / impl_plan.md / README.md；docs/ssot/adr/feedbacks/v0.6_feedbacks.md 与 v0.5_feedbacks.md（翻转落盘）；docs/researches/cli-grammar-v06-reassessment-2026-08-09.md
- 评审依据：docs/ssot/adr/feedbacks/v0_feedbacks.md、v0.5_feedbacks.md、v0.6_feedbacks.md（指令链与优先级）；docs/dev/adr-v1.md（ADR-011）；docs/ssot/dev-principles/实现流程原则.md 与 MainAgent工作编排.md；docs/ssot/specs/cli-ux-redesign/ 全套六份（v0.5 文档集，继承声明核对基准）；docs/ssot/pillars/paperwork-init-conversation/ 两份 session-log
- 评审范围限定：仅 SSOT/pillars 合规维度（指令链一致性、五文档结构、单一事实源、ADR-011、不发布约束贯穿性）；不做正确性细节与实现可行性评审

---

## 一、总体结论

| 严重度 | 数量 |
|---|---|
| Critical | 0 |
| Major | 1 |
| Minor | 6 |

**是否闭合：未闭合。** 1 项 Major（规则 3 与 owner 裁决文档的内部矛盾）须修复并经复核后方可进入实现阶段（实现流程原则.md 实现门槛）。6 项 Minor 不阻塞但应在 rework 轮一并完成。

## 二、核验通过项（对抗立场下的无问题确认，均有逐条实证）

1. **owner 裁决忠实落盘**：v0.6_feedbacks §一 (1)(2)(3) 三条裁决均以引文形式记录（接受 action-first、NAME/BODY 改具名必填 flag `--author`/`--message` 且短形式授权自行设计、本轮不发布），引文与研究文档 §6 逐字一致；编排层对短形式的裁定（`-a`/`-m`、post read `--mention` 无短形式）落盘于 §2.2 并有理由；profile create `--name` 的误录更正（初稿曾误录为 `--author`）以 task #12 补记形式显式落盘于 §2.4，裁决记录无遗漏。
2. **翻转程序双向合规**：v0.5_feedbacks §三 末尾已按该文自身约定（「翻转须由 owner 显式给出新指令并追加落盘」）追加 2026-08-09 翻转记录，指向 v0.6_feedbacks 与研究文档，且声明「其余条款不改写」；v0.6_feedbacks §三 以表格形式列出被翻转条款三条（v0.5 §二.1、v0.5 §二.1 配套解读、v0_feedbacks #3.1）与未受影响条款五条，翻转面界定清晰。
3. **优先级链无循环**：v0.6_feedbacks > v0.5_feedbacks 被翻转条款、v0.6_feedbacks 与 v0_feedbacks 其余条款叠加生效、ADR-011 为不得违背的硬约束，三层关系在 v0.6_feedbacks §四、v0.5_feedbacks §三、README §二 冲突裁定规则三处表述一致，未发现循环引用或互相否认。
4. **ADR-011 五约束合规**：新文法全部命令保留显式 PATH 第一位（spec §1.2 规则 1），无任何 env 身份回退、无登录语义、无状态目录、无路径发现；`--author` 仅为每条消息的署名元数据，与 v0_feedbacks「GDM 给名字就够了、无登录」一致；`--stdin` 为正文通道而非配置来源，不构成状态依赖。
5. **继承声明与 v0.5 原文逐条吻合**：输出协议冻结（信封结构、七类 category、退出码 0/1/2、JSON 只增不改不删、command 标识，对照 v0.5 spec §4/§6）；usage 信封机制五要素（try_parse、静态规范示例、`--help/-V` 穿透、argv 扫描感知 `--json`、顶层失败 command 填 `usage`，对照 v0.5 spec §4.3）；ensure_suffix 三级解析（对照 v0.5 spec §5 与 design §7.4 裁定 7）；隐藏别名 `p/b/c/v/po`（对照 v0.5 spec §3.6）；validate `--type`（对照 v0.5 spec §3.5）；implicit-mention/showing/window 输出增补（对照 v0.5 spec §3.1）。逐项核对未发现漂移。
6. **v0.5 条款引用编号全部属实**：抽查 v0.6 文档集对 v0.5 的全部引用，包括 spec §3.1 混淆面引 v0.5 spec §3.1、spec §5 第 3 条引 v0.5 spec §4.2 末条（`--` 教学）、design §4 引 v0.5 design §1.1 四点否决论证、design §7 状态表引 v0.5 design §8 六条、design §9 引 v0.5 design §9 与 v0.5 impl_plan 步骤⑦ m6 先例，均与 v0.5 原文吻合。
7. **Rejected Alternatives 状态更新表与研究文档一致**：design.md §7 八行状态表与研究文档 §7 逐条一致（#1/#2/#5/#6 维持否决，#3/#4 被翻转，两项新增采纳），且 #1 行明确撤销 v0.5 design §8「保留为 v0.6 可选快捷前缀层提案」的尾巴，与 owner 接受 action-first 的裁决自洽。
8. **逐命令签名表三方一致**：v0.6_feedbacks §2.4、spec §2、design §2 三份签名表逐行比对，十一条变化命令与六条不变命令签名完全一致，无版本间漂移。
9. **错误层级提升三处自洽**：`--message` 与 `--stdin` 同给由 v0.5 validation exit 1 提升为 usage exit 2，在 spec §3.1/§5 第 1 条、design §6、bdd S-SEND-07/S-EDIT-05 四处表述一致，且 design §6 给出层级提升论证与「不得静默择一」的规范含义声明。
10. **不发布约束贯穿全套文档**：README 文首与 §四、spec 文首/§7 第 4 条/§8 末、design 文首/§1.1/§9、bdd 文首、tdd 文首、impl_plan 文首交付边界/步骤(6)/文末依赖图、v0.6_feedbacks §一 (3)/§四，均声明不 bump、不打 tag、不 publish、不写 CHANGELOG 发布段；impl_plan 步骤(0)至(7)经逐步核验无任何版本或发布动作；CHANGELOG 在 spec §8 仅作为「发布时的披露载体」被提及且明示本轮不写，属合规引用而非残留步骤。
11. **tdd/impl_plan 盘点机制改进属实**：tdd §0 声明行号以合并后基线实测为准、v0.5 的 29 处清单已消耗完毕不作依据；impl_plan 步骤(0) 将 core example 点位与测试调用点两项盘点前置为编排层门禁步骤，并给出检索命令、以盘点输出为准，修正了 v0.5 时期「实施时检索为准」的延迟兜底做法（v0.5 SSOT 评审 ISSUE-M3 的教训）。
12. **pillars 继承**：session-log user msg5.1「每个原语无层级、独立成工具」落实为 design §2 逐 tool 独立论证（仅变化命令，未列命令显式声明沿用 v0.5 对应章节）；msg4.8「stub-first 流畅体验」由 post send 自动建线程（ensure_suffix 第(3)级落点，bdd S-SEND-03）继续承载；msg5.2「owner 只写 ssot/pillars、至多 ADR」的干预形式与 v0.6_feedbacks 的 owner 指令落盘形态一致。

---

## 三、问题清单

### Major

#### ISSUE-M1 规则 3「flag 全 CLI 唯一语义」的表述与 v0.6_feedbacks §2.1 规则 3 直接矛盾（三处漂移）

- 定位：spec.md §1.4（规则 3 正文）；spec.md §3.3（post read/summary 节）；design.md §1.2 规则 3 论证。对照基准：docs/ssot/adr/feedbacks/v0.6_feedbacks.md §2.1 规则 3。
- 问题描述：v0.6_feedbacks §2.1 规则 3 的准确表述是「同名 flag 跨命令只允许一种含义」，并显式记载例外裁定：「`--to` 在 post send（收件人）与 post read（seq 终点）两命令中语义各自唯一，不构成跨命令双义」。但被评审文档集三处未承接该裁定：
  1. spec §1.4 写「全 CLI 任何 flag 只有一种含义」，是比 feedbacks 更强的全称断言，且未附 `--to` 例外；
  2. spec §1.4 与 §3.3 均写「`--from/--to` 仅存于 post read，仅表 seq 范围（v0.5 已确立，不变）」，而 spec §2 自己的 post send 签名行明确含 `[--to a,b]`（format-v2 加入的收件人 flag），v0.6_feedbacks §2.4 post send 签名行同样含 `--to a,b`；
  3. design §1.2 规则 3 称「继承 v0.5 规则 2 不变」，同样未承接例外。
  其后果：按 spec §1.4/§3.3 字面，post send 的 `--to` 属「同名 flag 两种含义」与「存在于 post read 之外」，构成对规则 3 的违反；spec 自我定位为「实现与测试的唯一验收基准」，内部两条互相否定将直接导致 BDD 白名单断言（S-SHORT-02）与实现取舍无所适从。这不是实现可行性问题，而是 SSOT 文档间的规则矛盾。
- Expected：规则 3 的规范表述以 v0.6_feedbacks §2.1 规则 3 为准（跨命令单一含义 + `--to` 例外裁定），且 spec 对 `--from/--to` 的分布描述须与 spec §2 签名全表自洽。
- 修复建议：① spec §1.4 改为「同名 flag 跨命令只允许一种含义」，并追加一句「例外：`--to` 在 post send 表收件人、在 post read 表 seq 终点，两命令内语义各自唯一，不构成跨命令双义（v0.6_feedbacks §2.1 规则 3 裁定）」；② spec §1.4 与 §3.3 的「`--from/--to` 仅存于 post read」改为「`--from` 仅存于 post read 且仅表 seq 起点；`--to` 在 post read 表 seq 终点、在 post send 表收件人」；③ design §1.2 规则 3 同步补例外说明。

### Minor

#### ISSUE-m1 v0_feedbacks.md 本体缺少 #3.1 被翻转的指针落盘

- 定位：docs/ssot/adr/feedbacks/v0_feedbacks.md（v0.2 feedback 第 3 条之 3.1，全文无翻转注记）；对照 docs/ssot/adr/feedbacks/v0.5_feedbacks.md §三 末尾翻转记录（已成做法）。
- 问题描述：v0_feedbacks #3.1（content 类型参数放在最后）的字面条款已被 owner 翻转（记录于 v0.6_feedbacks §3.1），且 v0.5_feedbacks 被翻转时已在自身文档内追加指针记录；但 v0_feedbacks.md 本体没有任何注记。v0_feedbacks 未建立翻转程序条款，故不构成程序违背，但读者仅读 v0_feedbacks 时无法得知 #3.1 字面条款已失效，SSOT 追溯链存在单向断点。
- 修复建议：在 v0_feedbacks.md 的 v0.2 feedback 节末（或 #3.1 条目处）追加一行不改写原文的注记：「#3.1 字面条款（content 位置参数末位）已于 2026-08-09 被 owner v0.6 指令翻转，正文改经 `--message`/`--stdin` 具名传递，书写便利精神保留；见 docs/ssot/adr/feedbacks/v0.6_feedbacks.md §3.1」。

#### ISSUE-m2 五文档结构缺 role 文档配套，治理清单亦未列该项

- 定位：docs/ssot/specs/cli-grammar-v0.6/impl_plan.md 文首（「实施者 role 文档由编排层另行派发」）；README.md §四 治理状态清单（无 role 文档项）。对照基准：docs/ssot/dev-principles/实现流程原则.md「实现前文档」节（五份文档之外，同时为实施者单独产出 role 文档）；v0.5 先例（cli-ux-redesign-implementer.role.md 在评审阶段已存在并被评审）。
- 问题描述：实现流程原则将 role 文档列为实现前必备文档之一。本目录五份文档齐备，但 role 文档未落盘，impl_plan 仅声明「另行派发」，README 治理清单也未将其列为待办项，闭合判据不完整。
- 修复建议：评审闭合前由编排层派发 role 文档并纳入本目录 README §四 清单（增加一行「role 文档派发」待勾项），或在 impl_plan 前置门槛中显式写明 role 文档的交付时点与责任方。

#### ISSUE-m3 BDD/TDD 映射闭环缺口：v0.5 S-READ-06/S-READ-07（total 口径）无对等场景与用例

- 定位：bdd.md §3（仅 S-READ-01 至 S-READ-05，S-READ-01 Then 以括号注「行为沿用 v0.5 bdd S-READ-01/02/06/07，含过滤+limit 的 total 口径」）；tdd.md §4「冻结回归抽查」行（对应 BDD 列为「S-READ-01~03 / S-SUM-01 / S-PATH-* / S-ALIAS-* / S-OUT-01~04」）。对照基准：v0.5 bdd S-READ-06（过滤后零命中：showing 0/0 且不显示 window）与 S-READ-07（过滤+limit 组合：showing 20/25 而非 20/50，F3 裁定）、v0.5 tdd §3 两条显式用例。
- 问题描述：total 口径（过滤后、limit 截断前）是 v0.5 rework 轮的显式裁定（design §7.4 F3），v0.5 bdd/tdd 各有独立场景与用例承载；v0.6 bdd 将其降级为一句括号注，tdd §4 的抽查映射只列到 S-READ-01~03，未覆盖 S-READ-06/S-READ-07 对等断言。冻结回归的闭环因此出现缺口：实施者若仅按 v0.6 两份文档执行，total 口径与空 window 行为没有任何显式断言防线。
- 修复建议：① bdd §3 补两条场景（S-READ-06 过滤后零命中 showing 0/0 不显 window；S-READ-07 过滤+limit 的 total 口径），命令示例已是 v0.6 文法，成本极低；② tdd §4 冻结回归抽查行的对应 BDD 列同步补入。

#### ISSUE-m4 spec §3.1 post send 参数表未覆盖 `--title`/`--participants`/`--to`，逐命令契约不完整

- 定位：spec.md §3.1 post send 参数表（仅 PATH、`--author`、`--message`、`--stdin` 四行）与节末行为保留句（「`--title/--participants` 仅在建线程时生效」）。对照：spec.md §2 post send 签名行与 v0.6_feedbacks §2.4 post send 签名行（均含 `[--title T] [--participants a,b] [--to a,b]`）。
- 问题描述：spec §3 自我定位为「逐命令契约与错误 category 映射」，但 post send 的参数表只覆盖四参，签名中的 `--title`/`--participants` 仅以一句行为保留带过，`--to`（收件人）完全未提及。三者属 format-v2 引入的载荷，但 spec §2 与 feedbacks §2.4 既然将其纳入 v0.6 签名全表，逐命令契约就应给出对应的形态与必填性说明（至少一行表项），否则与 ISSUE-M1 叠加后 `--to` 在 spec 内彻底无契约描述。
- 修复建议：spec §3.1 参数表补三行（`--title`/`--participants`/`--to`，均为可选 flag，format-v2 语义，建线程时生效的适用条件写明）；错误映射句无需改动。

#### ISSUE-m5 BDD 场景编号跨版本复用，同名异义引用易致混淆

- 定位：bdd.md 全文场景编号（S-SEND-09、S-SEND-10、S-SEND-12、S-EDIT-02、S-EDIT-03、S-PROF-03、S-BRIEF-03、S-BRIEF-04、S-CONTACTS-03、S-CONTACTS-04、S-VAL-04 等在 v0.5 与 v0.6 两套 bdd 中含义不同）；典型如 v0.6 S-SEND-15 Then 引「v0.5 S-SEND-12 的静默写入路径」、S-SEND-04 引「v0.5 bdd S-SEND-10b/S-SEND-11」。
- 问题描述：v0.6 bdd 重新起编号（本身合理），但大量场景号与 v0.5 bdd 撞号且含义不同，而文档内又频繁以「v0.5 bdd S-xxx」形式交叉引用旧编号。凡引用处漏写「v0.5」前缀（如 tdd §4 抽查行的「S-READ-01~03」未注明版本归属），读者即可能在两套语义间串线。该风险目前靠上下文兜底，未发现已发生的错误引用，故判 Minor。
- 修复建议：① 在 bdd.md 文首约定节加一句「本文场景编号为 v0.6 独立编号；凡引用 v0.5 场景一律带 v0.5 前缀」；② tdd §4 表中跨版本引用处统一补版本前缀。

#### ISSUE-m6 v0.6_feedbacks §五 关联文件表未收录本套治理文档，且 README 0b 行对研究文档的内容描述与实际章节不符

- 定位：docs/ssot/adr/feedbacks/v0.6_feedbacks.md §五（关联文件表，含 v0.5 feedbacks、v0 feedbacks、研究文档、v0.5 design、ADR-011、v0.5 文档集，未含 docs/ssot/specs/cli-grammar-v0.6/）；README.md 一、清单 0b 行（描述研究文档为「基线核实 / 错误注入矩阵 / path-first 复评 / Rejected Alternatives 再评估」）。对照：v0.5_feedbacks §四 曾自指其治理文档集（做法先例）；研究文档实际章节为 §2 基线核实、§3 三视角结论（错误注入矩阵内嵌于 §3.2）、§4 path-first 复评、§5 混淆面枚举、§6 owner 裁决、§7 Rejected Alternatives 再评估。
- 问题描述：① 指令落盘文档与依据其产出的治理文档集之间缺一条互指，追溯链弱于 v0.5 先例；② README 0b 行把「错误注入矩阵」单列为研究文档的顶级内容，而实际无此独立章节，且漏列 §5/§6 两个顶级章节，与 v0.6_feedbacks §五 对同一文档的描述不一致。
- 修复建议：① v0.6_feedbacks §五 补一行「docs/ssot/specs/cli-grammar-v0.6/（spec/design/bdd/tdd/impl_plan/README）：依据本指令产出的治理文档集」；② README 0b 行描述改为与 v0.6_feedbacks §五 一致（基线核实 / 三视角方案取舍与错误注入矩阵 / path-first 复评 / 混淆面枚举 / owner 裁决记录 / Rejected Alternatives 再评估）。

## 四、其他观察（不计入 Issue 数）

1. design §3 对「`--mention` 不给短形式」的论证表述为「直接违反规则 3」，严格说规则 3 约束的是 flag 全称的同名双义，短形式冲突属规则 3 的延伸适用；建议表述改为「违反规则 3 的短形式延伸约束（v0.6_feedbacks §2.2 裁定）」，与 spec §4 表尾口径统一。
2. bdd S-OUT-04 Then 写「五场景行为与 v0.5 bdd S-OUT-04~07 逐条一致」，而 S-OUT-04~07 为四个编号（其中 S-OUT-07 含 --help 两层与 -V 共三形态），数字口径靠展开才可自洽；建议改为「与 v0.5 bdd S-OUT-04~07 逐条一致（含 --help 各层级与 -V）」。
3. impl_plan 步骤(5) 依赖「推送后 CI 实际绿」，属实现阶段事项；本次评审不触碰代码与 CI，仅记录该步骤与不发布约束无涉（CI smoke 换文法不等于发布）。

## 五、是否闭合与必须修复清单

**结论：未闭合。**

必须修复（闭合前完成，均为文档层修改，成本低）：

1. ISSUE-M1：spec §1.4 与 §3.3、design §1.2 按 v0.6_feedbacks §2.1 规则 3 原文改写，补 `--to` 例外裁定与分布描述修正，消除规则 3 与签名全表的内部矛盾。

建议一并修复（不阻塞但应在 rework 轮完成）：ISSUE-m1 至 ISSUE-m6。

对抗立场下的总体评价：文档集在指令链忠实性（三条 owner 裁决引文落盘、task #12 更正记录）、翻转程序双向合规、继承声明与 v0.5 原文逐条吻合、ADR-011 五约束、不发布约束贯穿性（七份文档全部声明、impl_plan 无任何发布残留步骤）上表现扎实，且相对 v0.5 文档集改进了盘点前置与行号基线声明。唯一 Major 属文档内部规则表述与 owner 裁决文档的矛盾，修复为纯文字改写；6 项 Minor 均为追溯链完整性与映射闭环层面的补漏。全部问题均不构成对 owner 指令或 pillars 的实质违背，修复后可闭合。

---
（评审报告完 - SSOT/pillars 合规视角，评审者：第三方批判性评审 subagent）
