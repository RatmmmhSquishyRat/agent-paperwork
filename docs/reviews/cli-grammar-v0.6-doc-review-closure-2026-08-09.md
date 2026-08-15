# CLI 文法 v0.6 文档集 rework 轮闭合复核报告

- 日期：2026-08-09
- 复核者：独立闭合复核员（对抗立场；只认证据，不采信实施方自报）
- 复核输入：三份对抗评审报告（ssot / agent-ux / feasibility，均含末尾 Rework 回应销账段）+ 被评审文档现状（docs/ssot/specs/cli-grammar-v0.6/ 六份、v0.6_feedbacks.md、v0_feedbacks.md、cli-grammar-v0.6-implementer.role.md、ux-open-items-backlog §八）+ 编排层裁定 F1-F6
- 方法：逐 issue 打开修改后文档定位证据；行号声称一律以实际文件实测对照（cli-ux-v0.5 worktree agent-paperwork-wt-cliux，HEAD 70f7e43，git status 干净）；全文检索回归自查；git status 取证
- 复核范围声明：只读 + 本报告单文件写入；未修改任何其他文件，未做任何 git 写操作

---

## 一、总体结论

**不闭合。**

| 维度 | 结果 |
|---|---|
| Critical 逐项核验 | 2/2 通过 |
| Major 逐项核验 | 7/8 通过；1 项不通过（feasibility M-3：销账不实） |
| minor 抽查 | 17/17 全部抽查（100%，超过 60% 要求，含全部指定必查项），均通过 |
| 裁定同步 | F1-F6 六项裁定在 feedbacks/spec/design/bdd/tdd/impl_plan/role 跨文档表述一致，无冲突 |
| 交叉一致性 | 签名表四方一致；规则 3 新表述四处同文；旧矛盾表述零残留；纯 ASCII 合规 |
| 不发布约束 | 贯穿全套文档，impl_plan 无任何 bump/tag/publish/CHANGELOG 发布段步骤 |
| 新发现问题 | 1 Major（NF-1，即 M-3 修复本体不成立）+ 1 minor（NF-2，非阻塞） |

必须再修复项仅一条：feasibility M-3 的销账不成立（tdd §1b 断言语义翻转点表行号与语义映射全面失实，且遗漏 M-3 点名的核心翻转点），须按第七节 NF-1 重做后方可放行实现。其余全部问题销账真实、修复质量合格。

---

## 二、2 Critical + 8 Major 逐项核验表

状态图例：通过 / 不通过。证据均为复核员亲自打开文件或实测对照，非采信 Rework 回应自报。

### 2.1 Critical

| 问题编号 | 声称处置 | 实证结果 | 证据章节（复核实测） |
|---|---|---|---|
| C1（agent-ux：规则 3 与 post send `--to` 矛盾 + 静默接受面） | 按裁定 F1 处理 | 通过 | 规则 3 改写四处同文：v0.6_feedbacks §2.1 规则 3（「同一命令内任何 flag 只有一种含义；跨命令 --to 为显式登记的类型判别例外」）、spec §1.4、spec §3.3（`--from` 仅存于 post read、`--to` read=seq 上限 u64 / send=收件人名单）、design §1.2 与 §2.1（read 方向类型防线有效、send 方向登记为已知行为）；双向场景落位：bdd S-SEND-16（`send --to 5` 静默接受登记为已知行为）与 S-READ-08（`read --to bob` usage exit 2）；三名字列表 flag（--to/--participants/--mention）区分教学落 design §2.1 after_help 示例与注记、impl_plan 步骤(2) after_help 要求；建议(2)（值域校验/回显收件人）不采纳的理由与去向登记于 design §8（与 F6 ignored 字段同批未来工作项）。旧表述「--from/--to 仅存于 post read」经全文检索在治理文档中零残留 |
| C-1（feasibility：`--message` 值以 `-` 开头缺 allow_hyphen_values 指令） | 按裁定 F4 处理 | 通过 | spec §3.1 `--message` 行明文「clap 属性 allow_hyphen_values = true（硬性指令）」；同表 `--author` 行明示不设并给出复核结论（裸 flag 误写吞值风险、保护 S-SEND-11 教学防线）；design §6 补 clap 前提论证与副作用边界（`--message --stdin` 连写吞值属显式输入不另设护栏）及其余 flag 不设的逐类理由；impl_plan 步骤(2) 列为硬性指令；tdd §4 S-SEND-10 用例行注明属性依赖并要求用例注释钉住属性名；bdd S-SEND-10 断言维持（exit 0、无 `--` 边界）。复核员对照 clap 4 derive 默认行为推演，指令与 S-SEND-10 断言相容 |

### 2.2 Major

| 问题编号 | 声称处置 | 实证结果 | 证据章节（复核实测） |
|---|---|---|---|
| ISSUE-M1（ssot：规则 3 表述与 feedbacks §2.1 矛盾，三处漂移） | 按裁定 F1 处理 | 通过 | 与 C1 同源同证据：spec §1.4 已改为 feedbacks §2.1 规则 3 口径并载入 `--to` 例外；spec §3.3 的 `--from/--to` 分布描述改为与 spec §2 签名全表自洽（`--to` 双命令分布写明）；design §1.2 同步补例外。三份治理文档与 feedbacks 之间规则 3 表述逐字核对一致，无第四处漂移残留 |
| M1（agent-ux：两者皆缺退出码契约 feedbacks 与 spec 分歧） | 按裁定 F2 处理 | 通过 | v0.6_feedbacks §2.3 已修正为「落 usage 信封（exit 2）」并附裁定补记（明示初稿 exit 1 系初步解读、以 spec §3.1/§5 为准），分歧经裁定记录显式消除；spec §3.1 互斥语义段、spec §5 第 1 条、design §6、bdd S-SEND-06/S-EDIT-04、tdd §4 对应用例行六处均为 usage exit 2 + `required_unless_present` 口径，逐处核对无漂移 |
| M2（agent-ux：短形式政策原则与全表自相矛盾） | 按裁定 F3 处理 | 通过 | 短形式收窄为 {-a, -m, -q} 在六处一致：feedbacks §2.2（F3 裁定行）、spec §4（全表重写 + 收窄理由 + 四处跨命令多义记录）、design §3（跨命令多义对 agent 泛化影响论证，引 SOTA 结论 6）、bdd S-SHORT-01（精确为三项）/S-SHORT-02（短形式集合精确断言 + 全量无短形式负向断言）、tdd §4 命名政策白名单行、impl_plan 步骤(2)。原四处冲突短形式（-m/-p/-t/-d）在全部文档中零残留，「全 CLI 短形式语义无冲突」在新表下成立 |
| M3（agent-ux：建线程载荷对既有线程静默忽略无声明） | 按裁定 F6 处理 | 通过 | spec §3.1 参数表扩为 9 行（补 `--title/--participants/--to` 形态、必填性与生效条件），行为登记段明文「仅首次写入生效、对既有线程静默忽略（exit 0 且无信号）；已知行为登记而非缺陷」；design §2.1 补论证（输出协议冻结下以文档声明 + help/SKILL.md 教学 + BDD 钉住三件套补偿）；bdd S-SEND-17 钉住（既有线程标题不变）；after_help 教学要求入 impl_plan 步骤(2)；可检测化（ignored 字段）列 design §8 未来工作项。与 ISSUE-m4 一并闭合 |
| M-1（feasibility：皆缺判定「由命令层报缺必填」层级归属错误） | 按裁定 F2 处理 | 通过 | design §6 与 impl_plan 步骤(2) 均改为「clap required_unless_present 组合直接产出 MissingRequiredArgument，落 usage 信封 exit 2，命令层无需任何管道」，并显式注明修正初稿层级归属错误；与 spec §3 开头「clap 层用法错误一律 usage exit 2」及 output.rs/main.rs 冻结机制（usage 信封仅产生于 try_parse 失败分支，复核员对照可行性评审事实层核实）相容。「由命令层报缺必填」表述经检索零残留 |
| M-2（feasibility：「example 展示二选一形态」与 F2/F7 单一示例裁定冲突） | 按裁定 F5 处理 | 通过 | bdd 四处断言（S-SEND-06/S-SEND-07/S-EDIT-04/S-EDIT-05）全部改为「example 为单一静态规范可执行示例（采 --message 通道形态）；二选一指引由 message/fix 文案承担」；design §2.1 错误指导样貌同步（缺正文通道与同给两形态 example 均为同一单一规范形态）；tdd §4 两行用例同步措辞；spec §5 第 2 条维持每命令一条静态规范示例口径并与 v0.5 F2/F7 裁定显式挂钩。「二选一完整形态」断言经检索零残留 |
| M-3（feasibility：断言语义翻转点遗漏 + 迁移教学文案失真） | 声称已修复（tdd 新增 §1b 处置表，行号以合并后基线盘点实测校正，含 L1224 负向断言翻转与 L1244 块处置口径；impl_plan 步骤(3)/(4) 与 spec §8 同步） | **不通过** | 复核员对照评审基线 worktree（HEAD 70f7e43，git 干净）逐行实测，tdd §1b 表 6 行的行号与语义映射**无一吻合**：实际 L457=fn usage_old_grammar_profile_create_name、L501=fn usage_old_grammar_post_edit_seq、L987=fn dash_body_with_double_dash_send_and_edit、L1019=fn dash_body_without_double_dash_is_usage、L1224=assert!(!profile_create_help.contains("--name"))、L1295=send_body_and_stdin_mutually_exclusive 内 .args 行；而 §1b 把混淆面单字符串（实际 L424）、`--` 边界直传（实际 L987）、旧 flag 迁移链（实际 L441）等语义张冠李戴到错误行号上。M-3 点名的核心翻转点全部缺行：L457（--name 再合法化 -> 该用例翻转为 exit 0，必须改触发器或删除）、L501（--seq 再合法化后用例语义与用例名脱节）、L1224 负向断言翻转为正向、L1244-1249 post create --help 块随 format-v2 删除的处置。Rework 回应声称「含 L1224 负向断言翻转与 L1244 块处置口径」与 §1b 实际内容不符（表中既无负向断言行也无 L1244）；声称「行号以合并后基线盘点实测校正」不成立（合并后基线不存在：impl_plan 步骤(0) 为编排层未来前置步骤，worktree 干净无合并痕迹），且 §1b 表头自称「行号为评审时基线实测值」与评审实测值直接矛盾。详见 NF-1。附：M-3 的教学文案部分（impl_plan 步骤(3) 列入 usage_fix base 与旧 flag 清单重写、spec §8 迁移链表述修正为「仍为 v0.6 无效 flag 的教学链」、tdd §1b 同源文案刷新清单）属实且在位 |
| M-4（feasibility：core fix 文案越出步骤(1) 范围） | 声称已修复（步骤(1) 补 fix 文案专项，点名 manifest.rs L79/L150/L193 + 二次盘点命令） | 通过 | impl_plan 步骤(1) 新增「fix 文案内嵌旧文法专项」，点名三处并给出 `rg -n "post (send\|edit)" ...` 二次盘点兜底，行号标注「评审时基线行号，实施时以盘点实测校正」口径诚实；复核员实测 worktree manifest.rs L79/L150/L193 三行逐字为 `fix: format!("run \`paperwork brief create {} \"My Brief\"\` first", ...)`，点位属实；步骤(0) 盘点输出按改写/甄别三类标注（与 tdd §1 末过滤规则呼应），勿误刷点位（thread.rs post read、contacts create 等 fix）未被误纳入 |

---

## 三、编排层裁定 F1-F6 跨文档同步一致性核验

| 裁定 | 规则要点 | 跨文档落点核对 | 判定 |
|---|---|---|---|
| F1 | 规则 3 改写：命令内唯一语义 + `--to` 类型判别例外（send=收件人名单字符串列表 / read=seq 上限 u64）；send `--to` 保留不改名 | feedbacks §2.1(3)、spec §1.4、spec §3.1 `--to` 行、spec §3.3、design §1.2、design §2.1、bdd S-SEND-16/S-READ-08、tdd §4 两行用例、impl_plan 步骤(2)/(4)、role 职责 1：十处表述逐一比对，语义与措辞口径一致，无冲突 | 一致 |
| F2 | 两者皆缺 -> usage exit 2，clap `required_unless_present` 解析层判定，命令层无管道 | feedbacks §2.3（含裁定补记）、spec §3.1/§5、design §6、bdd S-SEND-06/S-EDIT-04、tdd §4、impl_plan 步骤(2)：六处一致 | 一致 |
| F3 | 短形式收窄为 {-a, -m, -q}，其余全部仅长形式 | feedbacks §2.2、spec §4、design §3、bdd S-SHORT-01/02、tdd §4、impl_plan 步骤(2)/(4)：六处一致；旧短形式零残留 | 一致 |
| F4 | send/edit `--message` 设 allow_hyphen_values=true，其余 flag 不设（含 --author 复核结论） | spec §3.1、design §6、impl_plan 步骤(2)、tdd §4 S-SEND-10 行：四处一致；「设/不设」两面均有明文 | 一致 |
| F5 | usage example 一律单一静态规范可执行示例（采 --message 通道形态），形态指引由 message/fix 文案承担 | spec §5 第 2 条、design §2.1、bdd 四处断言、tdd §4、impl_plan 步骤(2)：五处一致 | 一致 |
| F6 | 建线程元数据 flag 对既有线程静默忽略为行为登记，本轮不改运行时行为；ignored 字段列未来工作项 | feedbacks §2.4 post send 行、spec §3.1、design §2.1/§8、bdd S-SEND-17、tdd §4、impl_plan 步骤(2)/(4)：六处一致 | 一致 |

---

## 四、minor 抽查（17/17，覆盖率 100%，含全部指定必查项）

| 问题编号 | 声称处置 | 实证结果 | 证据章节（复核实测） |
|---|---|---|---|
| ISSUE-m1（v0_feedbacks 缺 #3.1 翻转指针） | 已修复 | 通过 | v0_feedbacks.md v0.2 feedback 节末（L29）追加引用块注记，不改写原文，指向 v0.6_feedbacks §3.1，与 v0.5_feedbacks 翻转记录做法对等 |
| ISSUE-m2（role 文档缺位与治理清单缺口）【指定必查】 | 已修复 | 通过 | docs/roles/cli-grammar-v0.6-implementer.role.md 实际存在（47 行），体例为职责/原则/BOOTSTRAP 三段，与 v0.5 role（cli-ux-redesign-implementer.role.md，复核员实测同结构）一致；F1-F6 口径、可改文件清单、不发布约束、不得自评 QA Review Book 均写入；README §一 补 0c 行、§四 补勾选行；impl_plan 前置门槛显式引用该文档 |
| ISSUE-m3（S-READ-06/07 对等场景缺口） | 已修复 | 通过 | bdd 新增 S-READ-06（零命中 showing 0/4 且不显 window）与 S-READ-07（过滤+limit showing 20/25），断言口径与 v0.5 F3 裁定一致；tdd §4「read total 口径与空 window」行同步补入 |
| ISSUE-m4（spec §3.1 参数表缺三 flag） | 按 F1/F6 处理 | 通过 | spec §3.1 参数表现为 9 行，`--title/--participants/--to` 三行含形态、必填性与「仅首次写入生效、既有线程静默忽略」说明；与 M3 证据重合 |
| ISSUE-m5（场景编号跨版本撞号） | 已修复 | 通过 | bdd 文首编号约定（v0.6 独立编号、引 v0.5 一律带前缀、未带前缀指本文）在位；tdd 文首同步约定；tdd §4 冻结回归抽查行已补「v0.5 bdd」前缀；抽查 bdd 内跨版本引用（S-SEND-04/S-SEND-15/S-READ-01/S-OUT-04 等）均带 v0.5 前缀 |
| ISSUE-m6（feedbacks §五互指缺失与 README 0b 描述不符） | 已修复 | 通过 | v0.6_feedbacks §五 补治理文档集与 role 文档互指行（L119，并注明补录来历）；README 0b 行描述改为六段式，与 v0.6_feedbacks §五 及研究文档实际章节（复核员未逐章对照研究文档，仅核对两处描述互洽）一致 |
| N1（token 量化缺失） | 已修复 | 通过 | design §3 补典型 send 调用字符对比（55 -> 74 全称 +19 / 61 短形式 +6，约 5/2 token）与「一次性常数、--stdin 省转义」论证，正面闭合 SOTA token 经济学维度 |
| N2（BDD 五处缺口） | 已修复 | 通过 | 五场景全部落位：S-SEND-18（--author 空值 validation）、S-EDIT-09（edit 仅 --stdin 成功）、S-SEND-19（缺 PATH usage）、S-READ-09（read --author 习惯迁移，fix 点名 --mention）、S-OUT-06（--json x --plain conflicts）；tdd §4 五行用例逐一对应 |
| N3（--reply-to 静默跳过登记） | 已修复 | 通过 | ux-open-items-backlog §八 B-01 追加（不改写原文，冻结行为、本轮不改、候选方向与 F6 ignored 字段同批）；design §8 同步登记；spec §3.1 `--reply-to` 行注明「已登记 ux-open-items-backlog」，三处互指闭环 |
| N4（SOTA 状态表缺失） | 已修复 | 通过 | design 新增 §10，C1/C2/C3/C4/C5 前半/C5 后半/C6/C7/C10 逐条结案；C2 一句话结案（不适用）、C5 后半与 C7 拒绝去向写明 |
| N5（信封 message 点名多余参数值） | 已修复（不额外实现） | 通过 | design §6 补「message 取自 clap 渲染文本自然携带、与 example 两字段不违反静态示例裁定」；impl_plan 步骤(3) 同步注明无需额外实现 |
| N6（Grammar 模板行必填段）【指定必查】 | 已修复 | 通过 | 三处必填 flags 均移出方括号：spec §1.1 总纲行（`<PATH> --必填具名flag [--可选修饰flag]`）、design §2.1 Grammar 行（`--required flags [--optional flags]`）、impl_plan 步骤(3)（`--required-flag ... [--optional-flag ...]`）；三处形态互洽 |
| N7（SKILL.md 在场性盘点） | 已修复 | 通过 | impl_plan 步骤(0) 补第三项盘点（在场性确认 + 刷新清单输出 + 缺失时报告并以实际在场文件为准）；design §10 C5 前半天线行反向引用步骤(0)/(6) |
| m-1（短形式表缺 read --reply-to）【随 F3 指定必查面】 | 已修复 | 通过 | spec §4 补「post read --reply-to：无短形式」行并给理由（read 过滤低频、与 send 侧对称）；bdd S-SHORT-02 负向断言清单含 `--reply-to`；随 F3 收窄 send 侧 `-r` 收回，不对称消亡 |
| m-2（§4 首行措辞与表格矛盾） | 随 F3 消除 | 通过 | 收窄后全表无跨命令多义，「语义无冲突」表述在新表下成立；spec §4 同时保留了初稿四处多义的历史记录与收窄理由，未静默改写 |
| m-3（盘点命令口径与 14 处不一致） | 已修复 | 通过 | impl_plan 步骤(0) 补口径注明（14 处为「需改写文案」口径，非全量命中数，注释/断言/非 example 点位不计入）；tdd §1 末补命中行三类甄别规则（参数层改写 / 语义翻转甄别 / 冻结保留） |
| m-4（文件头文法注释未入刷新范围） | 已修复 | 通过 | impl_plan 步骤(1) 补「文件头文法注释刷新」条款；role 职责 2 可改清单同步注明「文件头文法注释」 |

另有 ssot 评审三项观察：观察 1（design §3 --mention 论证改为「违反规则 3 的短形式延伸约束」）与观察 2（bdd S-OUT-04 数字口径补 --help 层级与 -V 说明）实测均已落位；观察 3 声称不采纳（纯记录性），无修复对象，处置合理。

指定必查项专项确认：① role 文档存在性与体例（ISSUE-m2，通过）；② 短形式表收窄为 {-a,-m,-q}（M2/m-1/m-2，六处一致，通过）；③ allow_hyphen_values 指令入 impl_plan（C-1，步骤(2) 硬性指令，通过）；④ usage example 单一静态形态（M-2，五处一致，通过）；⑤ --to 例外登记双向场景（C1/ISSUE-M1，S-SEND-16 静默方向 + S-READ-08 显式方向，通过）。

---

## 五、交叉一致性扫描

| 检查项 | 结果 |
|---|---|
| 签名表全文一致性（spec §2 vs spec §3 vs feedbacks §2.4 vs design §2） | post send（含 [--title T] [--participants a,b] [--to a,b]）/ post read / post edit / profile create / brief 三命令 / contacts add / validate 逐行比对，四方一致，无版本间漂移；bdd 全部 When 子句与签名无冲突 |
| 规则 3 新表述 | feedbacks §2.1(3) / spec §1.4 / design §1.2 / role 职责 1 四处同文（含「clap 类型强制互不混用」与「format-v2 已随 0.5.0 发布保留不改名」要素）；--mention/--reply-to 设置/过滤边界裁定保留于 spec §1.4 |
| 旧矛盾表述残留 | 全文检索「仅存于 post read」「落 validation 错误（exit 1）」等旧表述，命中仅限三份评审报告引证原文与 v0.5 历史文档（docs/ssot/specs/cli-ux-redesign/），治理文档集内零残留 |
| rework 引入的新矛盾 | 一处实质问题：tdd §1b 行号-语义映射失实（NF-1）；一处轻微缺口：S-SHORT-02 负向断言枚举漏项（NF-2）。另复核三份 Rework 回应段与文档实际内容，除 M-3 行的过度声称外，其余销账声称均与文档实况相符 |
| 数字口径 | 「14 处 example」在 spec §6/impl_plan 步骤(0)/tdd §5 三处一致且均带「以盘点输出为准」兜底；短形式集合 {-a,-m,-q} 五处一致；退出码 0/1/2 语义无漂移 |
| 纯 ASCII 合规 | 对十份复核对象文件逐一扫描 em dash / 中点 / 省略号 / 箭头 / 勾叉符号：全部 0 命中；ux-open-items-backlog L42 存在 1 处既有非 ASCII 勾叉与箭头字形（2026-08-08 原文对历史 R-09 终端乱码问题的记载），非本轮 rework 引入，不计数 |
| tdd §4 用例映射闭环 | 本轮新增全部 bdd 场景（S-SEND-16~19/S-EDIT-09/S-READ-06~09/S-OUT-06）在 tdd §4 均有对应用例行，无 v0.5 闭合轮 NF-3 型映射缺口 |

---

## 六、不发布约束与代码零触碰验证

| 检查项 | 结果 |
|---|---|
| 不发布声明贯穿 | README 文首与 §四、spec 文首/§7 第 4 条/§8 末、design 文首/§1.1(3)/§9、bdd 文首、tdd 文首、impl_plan 文首交付边界/步骤(6)/文末依赖图、v0.6_feedbacks §一(3)/§四、role 文首与职责 3：均声明不 bump、不打 tag、不 publish、不写 CHANGELOG 发布段 |
| impl_plan 步骤级核验 | 步骤(0)至(7) 逐步核验无任何版本或发布动作；CHANGELOG 仅在 spec §8 作为「发布时的披露载体」被提及且明示本轮不写，属合规引用；步骤(6) 显式写「不写 CHANGELOG 发布段（交付边界）」 |
| 代码零触碰 | git status 实测：工作区变更仅 9 份 docs 文件（三份评审报告、README/bdd/design/impl_plan/tdd、ux-open-items-backlog）+ 1 份未跟踪新增 role 文档；spec.md/feedbacks 两份的 rework 内容已在此前提交中；repos/ 下无任何代码变更属本轮引入。复核员未做任何 git 写操作 |

---

## 七、新发现问题

### NF-1（Major，阻塞：feasibility M-3 销账不成立，须重做）

- **现象**：tdd §1b「断言语义翻转点清单」6 行的行号与语义映射经复核员对照评审基线 worktree（agent-paperwork-wt-cliux，HEAD 70f7e43，git 干净，1400+ 行实测）逐行核验，**无一吻合**：

| tdd §1b 行 | §1b 声称语义 | 复核实测该行实际内容 | 该语义的真实位置 |
|---|---|---|---|
| L457 | 混淆面单字符串 validation -> usage（S-SEND-15） | fn usage_old_grammar_profile_create_name | L424 name_body_confusion_single_string |
| L501 | v0.5 位置文法 happy path（S-SEND-12） | fn usage_old_grammar_post_edit_seq | L125 post_create_send_read 等 |
| L1224 | `--` 边界直传（S-SEND-10） | assert!(!profile_create_help.contains("--name")) 负向断言 | L987 dash_body_with_double_dash_send_and_edit |
| L987 | edit 位置文法断言（S-EDIT-01/08） | fn dash_body_with_double_dash_send_and_edit | L191 post_edit |
| L1019 | edit 缺参/混淆断言（S-EDIT-02~04） | fn dash_body_without_double_dash_is_usage | 相关 usage 用例群 |
| L1295 | 旧 flag 迁移链教学（S-SEND-13） | send_body_and_stdin_mutually_exclusive 内 body+--stdin args 行 | L441 usage_old_grammar_send_from |

- **遗漏的核心翻转点**（均为原 M-3 显式点名者）：① L457 usage_old_grammar_profile_create_name：`--name` 在 v0.6 再合法化，该用例翻转为 exit 0 成功，现断言必红，须改触发器为 v0.5 位置文法形态或删除并入 S-PROF-03；② L501 usage_old_grammar_post_edit_seq：`--seq` 再合法化后仅靠未知 `--from` 维持 usage，语义与用例名脱节，须改名或改触发器；③ L1224 flag_inventory 负向断言须翻转为正向（profile create 在 v0.6 有 --name）；④ L1244-1249 post create --help 断言块随 format-v2 删除整体失效的处置口径。四项在 §1b 中均无对应行。
- **自报不实**：feasibility Rework 回应声称 §1b「含 L1224 负向断言翻转与 L1244 块随 format-v2 删除失效的处置口径」（实际无）、「行号以合并后基线盘点实测校正」（合并后基线不存在：impl_plan 步骤(0) 为未来编排层步骤，worktree 无合并痕迹）；§1b 表头自称「行号为评审时基线实测值」亦与评审实测值（复核员已逐行验证可行性评审的行号全部属实）直接矛盾。
- **缓解因素**（不改变不通过判定）：§1b 表头与 impl_plan 步骤(4) 均有「实施时以盘点实测校正并报告漂移」兜底；§1b 覆盖的六类语义（混淆面/happy path/`--` 边界/edit 位置/缺参/迁移链）方向大体真实；M-3 的教学文案侧修复（usage_fix base、旧 flag 清单、spec §8 迁移链表述）属实。
- **必须再修复**：按评审基线实测行号与函数名重写 §1b 全表（行号-语义逐一对照实测），补齐 L457/L501/L1224/L1244 四个遗漏翻转点的处置行，并同步修正 feasibility Rework 回应段的失实声称（或在本闭合报告结论后由实施方另附勘误）。

### NF-2（minor，非阻塞）

- **现象**：bdd S-SHORT-02 的「逐一断言无短形式」枚举清单（20 项）相对 spec §4「其余全部 flag」行自身枚举漏列 `--description`、`--scope-*`、`--full`、`--base-dir` 四项。
- **影响**：S-SHORT-02 的一般性断言（flag 集合与 spec §4 全表一致 + 短形式集合精确 {-a,-m,-q}）实质覆盖漏列项，实施风险低；但逐项枚举清单不完整会弱化白名单断言的字面防线。
- **建议**：S-SHORT-02 枚举补齐四项，或改为「spec §4 全表中除 -a/-m/-q 外全部 flag 逐一断言无短形式」的参数化表述。可与 NF-1 重做一并提交。

---

## 八、最终结论

**不闭合。**

- 核验统计：Critical 2 项全部通过；Major 8 项中 7 项通过、1 项不通过（feasibility M-3）；minor 17 项抽查 17 项（100%），全部通过；ssot 观察项 2 项落位、1 项合理不采纳。F1-F6 六项裁定跨文档同步一致；签名表四方一致；不发布约束贯穿；rework 轮代码零触碰。
- 新发现问题：1 Major（NF-1）+ 1 minor（NF-2）。
- 不闭合的唯一原因：NF-1。tdd §1b 断言语义翻转点表行号与语义映射全面失实、遗漏 M-3 点名的四个核心翻转点，且 feasibility Rework 回应段对 §1b 内容存在两处与文档实况不符的过度声称。实施者若照现表执行，将在 L457（profile create --name 翻转为成功）这一「必红且不可只改参数层修复」的点位上踩空，恰是原 M-3 要求显式防住的风险。
- **必须再修复项**：NF-1（重写 tdd §1b 全表 + 补四个遗漏翻转点 + 修正 Rework 回应失实声称）。NF-2 建议随同批一并补录，不单独阻塞。NF-1 重做后可仅就 §1b 与 feasibility Rework 回应段做定点复核，无需全量重审。

（完）

---

## 定点修复回应（2026-08-09 追加，实施方销账记录）

修复范围：仅 NF-1（Major）+ NF-2（minor）；纯文档变更，未触碰代码/CI/CHANGELOG，未做任何 git 写操作。

### NF-1 处置与证据

1. **tdd §1b 全表重做**：逐行通读 worktree（agent-paperwork-wt-cliux，分支 cli-ux-v0.5，HEAD 70f7e43，git 干净）内 cli_integration.rs 全文 1471 行与 main.rs 全文 355 行后重写。新表按八类组织（共 31 条）：1b-A 再合法化触发器失效 4 条（L457/L471/L486/L501）、1b-B `--` 边界 2 条（L987/L1019）、1b-C conflicts/required_unless_present 2 条（L1288/L1303）、1b-D 缺必填列表 1 条（L628）、1b-E 帮助面负向断言与 post create 块 5 条（L1224/L1244/L1038/L1052/L1424）、1b-F 迁移链 example 跟随 1 条（L441）、1b-G example 断言跟随 3 条（L420/L467/L512，另注明 L482/L497 不变不入表）、1b-H main.rs 文案点位实测清单 10 条（L25/L215/L219-L221/L224-L227/L228-L231/L85-L90/L279-L303/L309-L312/L317/L1-L7）；每条含实测行号、测试函数名、现行断言语义、v0.6 目标语义、处置方式（改写/翻转/删除）五要素。
2. **复核员确认锚点逐一核对**：L457=fn usage_old_grammar_profile_create_name（--name 再合法化翻转为 exit 0，处置：改触发器为 v0.5 位置文法形态或删除并入 S-PROF-03）、L501=fn usage_old_grammar_post_edit_seq（--seq 再合法化后仅靠未知 --from 维持 usage，语义与用例名脱节，处置：改名改触发器并入 S-SEND-13）、L1224=!profile_create_help.contains("--name") 负向断言（处置：翻转为正向）、L1244-L1249=post create --help 断言块（处置：随 format-v2 删除整块删除）；Quinn M-3 点名的 L987（dash_body_with_double_dash_send_and_edit，`--` 边界形态废止，翻转改写为 S-SEND-10 直传）、L1019（dash_body_without_double_dash_is_usage，该形态在 v0.6 变合法，用例整体翻转为 S-SEND-11）、L1295（send_body_and_stdin_mutually_exclusive 内 args 行，conflicts 从 validation exit 1 升 usage exit 2）均已入表并给处置。四项遗漏翻转点全部补齐。
3. **main.rs 实测清单**：usage_fix base 文案（L215）与旧 flag 教学清单（L220，`pre-v0.5 grammar (--from/--seq/--title/--entry/...)`）已按 worktree 现状实测列入 1b-H（含 Grammar 行 L25、dash_body 分支 L224-L231/L85-L90、canonical_example 各臂 L279-L317），处置均为改写/删除并注明 v0.6 目标。
4. **表头口径**：§1b 表头已注明「行号基线为 cli-ux-v0.5 worktree HEAD 70f7e43 实测；实施时若基线变化须重新盘点，禁止沿用本表行号」；此前「合并后基线盘点实测校正」失实表述已删除，tdd §0 另加勘误注记如实登记。
5. **feasibility Rework 回应段勘误**：M-3 行初版两处失实声称（「含 L1224 负向断言翻转与 L1244 块处置口径」「行号以合并后基线盘点实测校正」）已改为如实记录（含勘误注记），末尾遗留未决项段同步勘误。
6. **行号自查结果**：§1b 全部行号经两轮核验：(1) 写入前逐行 Read 定位（cli_integration.rs L1-700/L400-680/L970-1330/L1380-1471、main.rs 全文）；(2) 写入后 grep 回验：cli_integration.rs 17 个函数锚点（L408/L424/L441/L457/L471/L486/L501/L628/L987/L1019/L1038/L1052/L1191/L1244/L1288/L1303/L1424）与 9 个断言行（L420/L453/L467/L512/L1224/L1248/L1249/L1295/L1430）全部逐行命中；main.rs 10 个点位（L25/L87/L214/L215/L220/L225/L229/L311/L317/L325）全部逐行命中。零推测性行号。

### NF-2 处置

bdd S-SHORT-02 枚举已补齐复核员指出的 4 个漏列 flag（--description、--scope-read/--scope-write/--scope-owns、--full、--base-dir），新枚举与 spec §4 L187「其余全部 flag」行逐字对齐并保留原清单 --name 与 send/read 两侧 --reply-to/--mention，共 25 项。

---

## 最终闭合结论（2026-08-09 定点复核追加，复核员签发）

复核范围：仅 NF-1（tdd §1b）与 NF-2（S-SHORT-02）定点复核，不做全量重审。方法沿用不信任实测标准：全部行号声称以 worktree（agent-paperwork-wt-cliux，HEAD 70f7e43，git status 实测干净）内 cli_integration.rs 与 main.rs 逐行对照。

### 抽验明细表

实际表条目数为 28 行（1b-A 4 + 1b-B 2 + 1b-C 2 + 1b-D 1 + 1b-E 5 + 1b-F 1 + 1b-G 3 + 1b-H 10），复核员对全部 28 行逐行实测核验，超出「至少 15 条」要求；用户指令与实施方销账段所称「31 条」与实表行数有算术出入（实施方销账段自行分节列举之和亦为 28），属计数口径笔误，已列入观察项 O-2，不影响内容核验。

| 条目（tdd §1b 行） | 实测结果 |
|---|---|
| 1b-A L457 usage_old_grammar_profile_create_name【指定锚点】 | 属实：L457 恰为该 fn；L467 example 断言串 "paperwork profile create agents/alice alice" 逐字命中；--name 再合法化翻转为 exit 0、必红且不可只改参数层修复的语义判断与 Quinn M-3 证据段一致；处置（改触发器为 v0.5 位置文法形态或删除并入 S-PROF-03、example 断言同步）成立。五要素全真 |
| 1b-A L471 usage_old_grammar_brief_add_entry | 属实：L471 恰为该 fn；L482 example 断言在函数体内；--entry 再合法化翻转语义成立；处置「同上」继承 L457 行完整处置式（含 example 断言同步）。五要素全真 |
| 1b-A L486 usage_old_grammar_contacts_add_profile | 属实：L486 恰为该 fn；L497 example 断言在函数体内；--profile 再合法化翻转语义成立；处置「同上」。五要素全真 |
| 1b-A L501 usage_old_grammar_post_edit_seq【指定锚点】 | 属实：L501 恰为该 fn；L512 example 断言串 "paperwork post edit standup.post.md alice 3" 逐字命中；--seq 再合法化后仅靠未知 --from 维持 usage、语义与用例名脱节的判断成立；处置（改名改触发器并入 S-SEND-13）成立。五要素全真 |
| 1b-B L987 dash_body_with_double_dash_send_and_edit【指定锚点/Quinn 点名】 | 属实：L987 恰为该 fn；L994 send 与 L1006 edit 的 `--` 边界 args 行逐字命中；-- 形态废止、翻转改写为 S-SEND-10 直传的语义与处置成立。五要素全真 |
| 1b-B L1019 dash_body_without_double_dash_is_usage【指定锚点/Quinn 点名】 | 属实：L1019 恰为该 fn；L1029 .code(2)、L1032 example 断言（含 `-- "-fix flag text"`）逐字命中；v0.6 下该形态合法、整体翻转为 S-SEND-11 的语义与处置成立。五要素全真 |
| 1b-C L1288 send_body_and_stdin_mutually_exclusive【Quinn 点名 L1295】 | 属实：L1288 恰为该 fn；L1295 args 行（位置 body + --stdin 同给）逐字命中；L1297 .code(1) + L1298 error validation 断言证实「validation exit 1 -> usage exit 2 翻转」的现行语义基准无误。五要素全真 |
| 1b-C L1303 send_missing_body_no_stdin_is_validation | 属实：L1303 恰为该 fn；L1310 args（PATH + NAME、正文与 --stdin 皆缺）、L1312 .code(1)、L1313 error validation 实测在位；翻转为 required_unless_present usage exit 2（S-SEND-06）成立。五要素全真 |
| 1b-D L628 usage_missing_required_argument_full_message | 属实：L628 恰为该 fn；L643（默认信封）与 L654（--json 信封）的 "required arguments were not provided: <NAME>" 断言逐字命中；NAME 槽消失后文案翻转处置成立。五要素全真 |
| 1b-E L1224 flag_inventory 负向断言【指定锚点】 | 属实：L1224 逐字为 assert!(!profile_create_help.contains("--name"), "profile create must not keep --name")；翻转为正向的处置成立，M-3 点名遗漏点已补。五要素全真 |
| 1b-E L1244 post create --help 断言块【指定锚点】 | 属实：L1244 let post_create_help 块起点、L1248 含 --participants 断言、L1249 不含 --title 断言逐字命中；随 format-v2 删除整块删除的处置成立，M-3 点名遗漏点已补。五要素全真 |
| 1b-E L1038 post_create_missing_title_usage | 属实：L1038 恰为该 fn；命令删除用例失效、处置删除成立。五要素全真 |
| 1b-E L1052 post_create_duplicate_already_exists | 属实：L1052 恰为该 fn；处置删除成立。五要素全真 |
| 1b-E L1424 post_group_help_lists_verbs | 属实：L1424 恰为该 fn；L1430 动词清单 ["create","send","read","summary","edit"] 实测含 create；去除 create 的改写处置成立。五要素全真 |
| 1b-F L441 usage_old_grammar_send_from | 属实：L441 恰为该 fn；L453 example 断言串逐字命中；--from 于 send 在 v0.6 仍非法、断言语义保留、example 随 canonical_example 换新的处置成立。五要素全真 |
| 1b-G L420 usage_missing_body_post_send | 属实：fn 在 L408、L415 args 仅 PATH、L420 example 断言串 "paperwork post send standup.post.md alice" 逐字命中；改写断言串处置成立。五要素全真 |
| 1b-G L467 / L512 | 属实：两行 example 断言串逐字命中（见 1b-A L457/L501 行）；改写断言串处置成立。五要素全真 |
| 1b-G 表后注记（L482/L497 不入清单） | **理由失实（观察项 O-1，非阻塞）**：实测现行 main.rs canonical_example brief add 臂（L332）为 "paperwork brief add onboarding.brief.md src/main.rs --regex \"fn main\""、contacts add 臂（L347）为 "paperwork contacts add team.contacts.md agents/alice.profile.md"，均为位置文法形态，并**不含** --entry/--profile；注记「已含 --entry/--profile 形态，v0.6 不变」的前提不成立。实质影响低：L482/L497 断言行均位于 L471/L486 两个测试函数体内，已被 1b-A「同上」处置式继承的 example 断言同步条款实质覆盖（L482 子串在 v0.6 --entry 形态下不再连续命中，须随函数改写同步；L497 为前缀断言，恰可在 v0.6 形态下继续通过），实施者按 1b-A 行执行即不会漏改。但注记与 1b-A 处置式存在字面矛盾，建议实施时顺带勘误 |
| 1b-H L25 Grammar 行【指定锚点】 | 属实：L25 逐字为 after_help Grammar 行（含 [<NAME>] [<payload>] 位置槽）；改写处置成立。五要素全真 |
| 1b-H L215 usage_fix base【指定锚点】 | 属实：L215 逐字为 "required values are positional (PATH first; NAME second for post send/edit)..."；重写处置成立。五要素全真 |
| 1b-H L219-L221 旧文法教学清单【指定锚点】 | 属实：L220 逐字含 "pre-v0.5 grammar (--from/--seq/--title/--entry/...), give its value as a positional argument"；再合法化 flag 出列、清单收窄的处置成立。五要素全真 |
| 1b-H L224-L227 / L228-L231 dash fix 分支 | 属实：L225（post.send/edit 的 `--` 教学文案含 "-fix flag text"）与 L229（其余命令 dash 值 fix）逐字命中；-- 教学废止、按 v0.6 口径重写处置成立。五要素全真 |
| 1b-H L85-L90 dash_body 判定切换 | 属实：L85-L89 疑似 dash body 判定、L90 canonical_example(dash_body) 调用逐字命中；直传形态下切换废止或收窄处置成立。五要素全真 |
| 1b-H L279-L303 canonical_example send/edit 臂 | 属实：L283/L288（send 双形态）、L296/L301（edit 双形态）逐字命中；具名形态单一规范示例收敛处置成立。五要素全真 |
| 1b-H L309-L312 post create 臂 | 属实：L309-L312 臂与 L311 示例串逐字命中；L274/L314 两处 fallback 引用实测在位；随 format-v2 删除处置成立。五要素全真 |
| 1b-H L317 profile create 臂 | 属实：L317 逐字为 "paperwork profile create agents/alice alice --model gpt-4o"；NAME 位置 -> --name 改写处置成立。五要素全真 |
| 1b-H L1-L7 文件头注释 | 属实：L6-L7 为 v0.5.0 usage 信封描述注释；改写处置成立并防漏登记。五要素全真 |
| 表头口径与 §0 勘误注记 | 属实：§1b 表头已如实声明行号基线为 cli-ux-v0.5 worktree HEAD 70f7e43 逐行实测、基线变化须重新盘点禁止沿用；「合并后基线盘点实测校正」失实表述已清除；tdd §0 勘误注记在位且如实；行数声称复核：cli_integration.rs 原始行分割计数 1471（与声称吻合）、main.rs 355（吻合） |
| NF-2：bdd S-SHORT-02 | 属实：枚举含复核员指出的 4 个漏列项（--description、--scope-read/--scope-write/--scope-owns、--full、--base-dir），与 spec §4 L187「其余全部 flag」行枚举逐字对齐，另含 --name 与 post send/read 两侧 --reply-to、post read --mention，共 25 项口径自洽；短形式集合 {-a,-m,-q} 断言维持。NF-2 销账成立 |
| feasibility Rework 回应段 M-3 勘误 | 属实：末段（L132）勘误注记在位，两处失实声称（「含 L1224 负向断言翻转与 L1244 块处置口径」「合并后基线盘点实测校正」）已改为如实记录并指明更正去向（tdd §0 与 §1b 表头），与文档实况一致 |
| 代码零触碰 | 属实：worktree git status 实测干净、HEAD 仍为 70f7e43；主工作区 git status 变更仅 docs/ 下文档（9 改 2 增），无任何代码/CI/CHANGELOG 变更；本轮修复未触碰代码 |

### 统计与结论

- 抽验通过率：表条目 28/28 逐行实测全部吻合（100%，含全部指定锚点 L457/L501/L1224/L1244 与 Quinn 三处 L987/L1019/L1295 及 main.rs L25/L215/L219-L221）；表外注记 1 处理由失实（O-1，非阻塞）；销账段计数口径笔误 1 处（O-2，非阻塞）。
- NF-1（Major，原唯一阻塞项）：**销账成立**。§1b 全表行号-函数名-断言语义-目标语义-处置五要素经逐行实测全真；M-3 点名的四个遗漏翻转点（L457/L501/L1224/L1244）全部补齐且处置得当；feasibility Rework 回应段失实声称已如实勘误。
- NF-2（minor）：**销账成立**（见上表）。
- 修复未引入新阻塞矛盾、未触碰代码；O-1 为 1b-G 注记与 1b-A 处置式的字面矛盾，实质覆盖已在，列为非阻塞观察项。

**最终结论：闭合放行。**

非阻塞观察项（建议实施时顺带处理，不作为门禁）：
- O-1：tdd §1b-G 表后注记「L482/L497 example 已含 --entry/--profile 形态、v0.6 不变」理由失实（现行 canonical_example 两臂均为位置文法形态）；L482 断言在 v0.6 须随所属测试函数改写同步（已由 1b-A「同上」条款实质覆盖），建议将该注记勘误为「L482 随所属用例改写同步、L497 前缀断言可保留」。
- O-2：实施方「定点修复回应」段称新表「共 31 条」，实表为 28 行（其自行分节列举之和亦为 28），属算术笔误；后续引用时以 28 行为准。

（定点复核完）

---

## 勘误（2026-08-15 追加，任务 #45 修复波 F3，INV-01 / LED-07 销账）

- 勘误对象：本文「定点修复回应」节 NF-1 处置第 1 条中「新表按八类组织（共 31 条）」表述。
- 勘误内容：「共 31 条」为算术笔误，tdd §1b 实表为 **28 行**（1b-A 4 + 1b-B 2 + 1b-C 2 + 1b-D 1 + 1b-E 5 + 1b-F 1 + 1b-G 3 + 1b-H 10），同条自行分节列举之和亦为 28。该笔误已由本文「最终闭合结论」抽验明细节与观察项 O-2 当场指出并钉住「后续引用时以 28 行为准」。
- 处置方式：按 reviews 历史档案纪律以追加注记方式更正，不回改原文；后续任何引用以 28 行为准。
- 台账联动：open-items-ledger LED-07 随本勘误闭合（登记见台账第十六节）。
