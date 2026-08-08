# CLI UX 重设计 v0.5.0 文档集 rework 轮闭合复核报告（Task 8 第二轮）

- 日期：2026-08-09
- 复核者：独立闭合复核者（对抗性第二轮；只认证据，不接受自报）
- 复核输入：三份一轮评审报告（ssot/agent-ux/feasibility，2026-08-09）+ 修订后文档集（specs 全 6 份 + role + v0.5_feedbacks）+ 编排层裁定基线 F1-F7
- 方法：逐 issue 定位证据 + 行号对照源码 + rg 回归自查 + git 取证 + 文件 mtime 取证

---

## 一、总体结论

**闭合（文档层）。**

| 维度 | 结果 |
|---|---|
| 逐项销账 | 清单条目 41 项（去重后 36 项独立问题）：已修复 35、已修复带轻微残留 1（NF-2）、未修复 0、修复不当 0 |
| 裁定符合性 | F1-F7 共 7/7 逐字忠实落实 |
| 事实抽查 | tdd 29 处抽 8/8 吻合且独立全量枚举盘净；core 14 处抽 5/5 吻合且全量枚举盘净 |
| 回归自查 | 数字 29/14、category 词表、退出码、字段名无新不一致（除 NF-3 映射缺口） |
| 代码零触碰 | rework 轮自身未触碰任何代码；工作区存在并行线程代码变更（NF-1，需编排层确认归属与提交边界） |

去重说明（41→36）：5 个条目为跨视角同一问题——S-SEND-08（SSOT-M1 / agent-ux-C1 / feasibility-C-1）、showing total 口径（SSOT-M4 / agent-ux-M3）、manifest 清单（SSOT-M3 / feasibility-M-2）、usage example 生成（agent-ux-M1 / feasibility-M-4）。

---

## 二、逐项销账表

状态图例：已修复 / 残留（已修复但留有非阻塞残留，见第七节）。证据位置均经复核者亲自打开文件定位，非采信文档自报。

### 2.1 视角一 SSOT/pillars（4 Major + 8 Minor）

| issue | 严重度 | 状态 | 证据位置（复核实测） |
|---|---|---|---|
| M1 S-SEND-08 不可实现 | Major | 已修复 | bdd.md S-SEND-08 改为仅 PATH 形态 exit 2；新增 S-SEND-12（PATH+单字符串→validation exit 1，example 含 NAME 槽）；spec.md L85 混淆面裁定；design.md §2.5 三重教学补偿；tdd.md §3 两行用例同步。符合 F1 |
| M2 第③级表述漂移 | Major | 已修复 | spec.md §5 表（「物理创建仅发生在写命令…只读命令三级均无文件时报 not-found」）；design.md §7.4 裁定 7；impl_plan.md 步骤① 明示只读命令不建文件；bdd S-PATH-04/06/08 一致 |
| M3 manifest 清单漏 3 处 | Major | 已修复 | impl_plan.md 步骤④ 清单含 L80/L151/L194，共 14 处，附全仓检索命令与预期输出，删除延迟兜底；复核者 rg 实测 core 全仓恰 14 处旧文法 example，与清单逐一吻合 |
| M4 showing total 口径缺失 | Major | 已修复 | spec.md §3.1 post read（total=过滤后、limit 截断前）；bdd.md 新增 S-READ-07（过滤+limit 组合，20/25）；tdd.md §3 对应用例。符合 F3 |
| m1 措辞自相矛盾+自指预声明 | Minor | 已修复 | spec.md §4.2/§4.3/§6.1 均改「冻结枚举，仅可经评审流程扩展」；rg 全仓 0 处「可扩展封闭集合」残留；时态改为「已经本次对抗评审确认」（见第七节观察项 2） |
| m2 implicit-mentions 复数 | Minor | 已修复 | role.md L16 已为单数 implicit-mention；rg 全仓 0 处复数残留 |
| m3 role 清单缺 SKILL.md | Minor | 已修复 | role.md L17 清单含「SKILL.md（新增，步骤⑦）」 |
| m4 U-03 未正面回应 backlog | Minor | 已修复 | design.md §7.3 U-03 行补「对 backlog『本次必须解决』口径的正面回应」，列为 v0.6 候选单独立项 |
| m5 SOTA 引用编号错误 | Minor | 已修复 | design.md §9 与 impl_plan.md 步骤⑦ 均改「结论 C5 与 §8 风险 1（对冲三件套）」；复核者实测 SOTA 文档 L195 C5=随仓库发布 SKILL.md、L205 §8 风险 1=对冲三件套，引用属实 |
| m6 ADR 示例层脱节无安排 | Minor | 已修复 | impl_plan.md 步骤⑦：adr-v1.md 顶部加一行 Superseded-by 注记、不改写历史内容（ADR 不可变原则）；role.md L17 清单同步收录该文件 |
| m7 implicit-mention 不触发条件 | Minor | 已修复 | spec.md L87 列明三种不触发边界（自回复/已显式 mention/reply-to 不存在）；bdd.md S-SEND-10b 与 S-SEND-11 两条边界场景；tdd.md §3 用例对应 |
| m8 解读条款缺确认途径 | Minor | 已修复 | v0.5_feedbacks.md §二 补「确认途径记录」（编排层逐轮确认 + 四点论证引用），并诚实声明「owner 未提出异议；若追认或翻转须追加落盘」 |

### 2.2 视角二 agent-ux（1 Critical + 6 Major + 8 Minor）

| issue | 严重度 | 状态 | 证据位置（复核实测） |
|---|---|---|---|
| C1 S-SEND-08 与签名互斥 | Critical | 已修复 | 与 SSOT-M1 同一问题：bdd S-SEND-08 改仅 PATH、新增 S-SEND-12（混淆面→validation exit 1，example 含 NAME 槽，fix 含 `--`）；spec L85 显式声明混淆面；design §2.5 三重教学补偿；tdd §3 两行用例。F1 逐字落实 |
| M1 usage example 生成规则未定义 | Major | 已修复 | F2 裁定落实：spec §4.3「静态规范示例…不携带用户原参数值、不做 argv 值迁移重建」；design §2.6 裁定 + Rejected Alternatives 第 6 条；S-SEND-09/S-EDIT-04/S-PROF-03/S-BRIEF-03/S-CONTACTS-03 五条断言全部降级为「规范形态示例」 |
| M2 「零破坏」声明过度 | Major | 已修复 | design §9 改写为「信封结构与既有 key 零变更，但本版含四项消费者可感知变化」并逐项列出；impl_plan 步骤⑦ 要求 CHANGELOG Changed (Breaking) 逐项披露附迁移说明；spec §4.6 对 showing 出现语义变化显式标注 |
| M3 total 语义未定义 | Major | 已修复 | 与 SSOT-M4 同一问题（F3）：spec §3.1 定义 total=过滤后、limit 前；bdd 新增 S-READ-07 过滤+limit 组合场景 |
| M4 `--` 边界零教学 | Major | 残留 | design §2.4 after_help 补 send/edit 各一条 `--` 示例；spec §4.2 规定 validation 类 fix 文案必含 `--` 教学；bdd S-EDIT-05 新增。**残留**：「`-` 开头 body 未加 `--`」的 usage 负形态仍无 BDD 场景，spec §4.3 未要求 usage 信封 fix 提及 `--`（见 NF-2） |
| M5 第①级异型文件角落 | Major | 已修复 | spec §5 声明异型文件命中第①级按对应解析器报 format、不改道；bdd 新增 S-PATH-07（example 引导 validate --type）；design §7.4 裁定 7 与 F4 一致 |
| M6 占位符策略不一致 | Major | 已修复 | F7 落实：spec §4.2 写入「example 一律具体可复制执行、禁尖括号占位符」；design §2.2 not-found 示例已改具体命令；bdd 全部 example 断言为具体值（rg 复查无 `<path>/<name>` 占位残留，S-PROF-05 等 `<PATH>` 为签名槽位非 example 字符串） |
| N1 contacts create 负迁移无补偿 | Minor | 已修复 | bdd 新增 S-CONTACTS-05（多余位置参数→usage，after_help 注记）；design §5.3 after_help 文案含英文 Note「title is an OPTIONAL flag here」 |
| N2 brief add/remove 参数映射缺失 | Minor | 已修复 | spec §3.3 brief remove 补推导规则（条目存储标题=ENTRY 的 basename）；bdd 新增 S-BRIEF-07（add src/main.rs 后 remove main.rs 成功、传原路径 not-found） |
| N3 NAME 字符集与校验未声明 | Minor | 已修复 | spec §3.1 post send 补「NAME 校验语义（沿用 v0.4）」：不与 profile/contacts 校验、空串拒绝、可含空格、含逗号不建议（无硬约束） |
| N4 BDD 缺口六场景 | Minor | 已修复 | 六条全部落位：①S-SEND-13 多余位置参数 ②S-VAL-05 --type 非法值 ③S-CREATE-03 post create already-exists ④S-EDIT-05 edit `--` 边界 ⑤S-SEND-10b 自回复不触发 ⑥S-VAL-06 --type 与后缀交叉 |
| N5 --mention/--reply-to 双角色未裁定 | Minor | 已修复 | spec §1.3 规则 2 下新增裁定段：设置/过滤为同一语义对象的同构延伸、不构成双语义 |
| N6 SOTA C5/C6 未竟项无记录 | Minor | 已修复 | design §7.5 表后补采纳/拒绝记录：C5 后半（--help --json 内省）拒绝并附理由；C6 采纳——tdd §3 末行「命名政策白名单断言（SOTA C6 采纳）」复用精确断言模式 |
| N7 数字出入+调研文档勘误 | Minor | 已修复 | tdd §1 标题与合计统一为 29 处改写 + 1 处保留；现状调研文档 L180 补 brief add flag 勘误（--entry/--entry-title）、L581 与 L596 补「send/edit 不校验 --from 与 profile 一致性」勘误——复核者已打开该调研文件实测两处勘误在位 |
| N8 post read 无 after_help | Minor | 已修复 | design §2.4 补 post read 两条示例（--from 5 --to 20 与 --mention alice --limit 20），并注明为 --from/--to 新唯一语义正面教学 |

### 2.3 视角三 feasibility（1 Critical + 6 Major + 7 Minor）

| issue | 严重度 | 状态 | 证据位置（复核实测） |
|---|---|---|---|
| C-1 缺 NAME 场景不可实现 | Critical | 已修复 | 与 C1/SSOT-M1 同一问题，证据同上（F1） |
| M-1 tdd 漏 brief create 三处 | Major | 已修复 | tdd §1.8 新增小节列 L220/L246/L270；合计式改 23+3+3=29 处改写 + 1 处保留；impl_plan 步骤⑤ 同步 29 处；复核者实测 cli_integration.rs 三行确为 brief create --title，且全文件旧文法枚举恰为 29+2（含 L188/L300 两个保留点） |
| M-2 core 清单 11→14 | Major | 已修复 | 与 SSOT-M3 同一问题：impl_plan 步骤④ 清单 14 处含 manifest L80/L151/L194，附检索命令；复核者 rg 实测吻合 |
| M-3 --help/-V 穿透缺失 | Major | 已修复 | F5 落实：impl_plan 步骤③ 明文 DisplayHelp/DisplayVersion 调 error.print() 后 exit 0 不进信封；spec §4.3 穿透条款；bdd 新增 S-OUT-07（--help/子命令 --help/-V 三者 exit 0）；tdd §3 补冻结用例 |
| M-4 逐字修正命令无机制 | Major | 已修复 | 与 agent-ux M1 同一问题，采 F2 二选一之 (b)：降级为静态规范示例并同步降级 BDD 断言；design §2.6 + Rejected 6 给出否决迁移层的完整理由 |
| M-5 目录路径边界未定义 | Major | 已修复 | F4 落实：spec §5 与 impl_plan 步骤① 判据改 is_file()（目录不命中，落入后续级别）；bdd 新增 S-PATH-08（目录场景→not-found，不创建） |
| M-6 门禁自相矛盾 | Major | 已修复 | F6 落实：impl_plan 全局门禁改为分阶段——步骤①~④ build+core 测试+clippy 全绿即可推进、cli_integration 允许红；步骤⑤后 workspace 全绿为硬门禁；role 文档职责 4 与工作原则 3 同步改写，三处口径一致 |
| m-1 --json 模式感知机制 | Minor | 已修复 | spec §4.3 补「argv 扫描兜底」机制；impl_plan 步骤③ 同步；bdd S-OUT-03 Then 子句写明 argv 扫描 |
| m-2 resolve_body 示例共用 | Minor | 已修复 | impl_plan 步骤② 新增条目：增补调用方参数（或等价手段）使 edit 错误给出 edit 示例 |
| m-3 validate fix 点位未点名 | Minor | 已修复 | impl_plan 步骤② 点名 validate.rs L31-35（未知后缀分支 fix/example 承载处，点名免漏改） |
| m-4 role 清单与步骤⑦不一致 | Minor | 已修复 | role L17 补 SKILL.md；QA Review Book 明确标注「由独立 agent 产出、不在本角色可改清单内、不得自评自写」，互斥消解 |
| m-5 showing 恒现非纯 additive | Minor | 已修复 | spec §4.6 显式写明「这是既有 key 出现语义的变化而非纯 additive…须在 CHANGELOG Changed (Breaking) 明示」 |
| m-6 exit 2 断言写法未示范 | Minor | 已修复 | impl_plan 步骤⑥ 给出 unix（set +e + $?）与 windows（$LASTEXITCODE）两段 yaml 样例，并警示不得沿用 grep 管道 |
| m-7 发布链条 30s 窗口风险 | Minor | 已修复 | impl_plan 步骤⑧ 注明失败时手工重跑 cli publish（core 已发布无需重发） |

---

## 三、F1-F7 裁定符合性逐条核验

| 裁定 | 基线要求 | 落实证据（复核实测） | 判定 |
|---|---|---|---|
| F1 | NAME/BODY 混淆面=固有代价+三重教学补偿；S-SEND-08 仅 PATH→exit 2；混淆面新场景→validation exit 1 且 example 含 NAME 槽 | bdd S-SEND-08 When 恰为 `post send standup.post.md`（仅 PATH）exit 2；S-SEND-12 为 PATH+单字符串→exit 1、example `post send standup.post.md alice "body text"` 含 NAME 槽；两场景 argv 形态互斥无重叠；spec L85「位置文法固有边界」+ design §2.5 三重教学补偿（validation example/after_help/SKILL.md）逐字在位；tdd §3 两行用例分别钉住 | 逐字落实 |
| F2 | usage example 降级为静态规范示例，禁逐字重建承诺 | spec §4.3「不携带用户原参数值、不做 argv 值迁移重建」；design §2.6 裁定 + Rejected 6 否决记录；五条旧文法 BDD 断言全部改为「规范形态示例」；design §9「信封内的规范可执行示例」不再称「逐字修正」 | 逐字落实 |
| F3 | showing total=过滤后 limit 前 | spec §3.1 post read 明文定义（无过滤时为线程全部条数，与 conclusion 行同口径）；bdd S-READ-07 用 20/25 钉死口径（而非 20/50） | 逐字落实 |
| F4 | 三级解析：is_file() 判据；第③级路径决策语义（物理创建仅写命令）；异型文件命中第①级报 format | spec §5、design §7.4 裁定 7、impl_plan 步骤① 三处一致；**只读命令不建文件条款**在 impl_plan 步骤① 明文（「只读命令（read/summary/validate）三级均无文件时报 not-found，不创建文件」）且 spec §5 同文；S-PATH-04/06/07/08 场景齐备 | 逐字落实 |
| F5 | --help/-V 穿透 exit 0 | spec §4.3 穿透条款（DisplayHelp/DisplayVersion 调 error.print() 后 exit 0、不进 usage 信封、守住 §6.3）；impl_plan 步骤③ 同文；bdd S-OUT-07 三命令断言；tdd §3 冻结用例 | 逐字落实 |
| F6 | 门禁分阶段：①-④允许 cli_integration 红，⑤后全绿 | impl_plan 全局门禁、tdd §6 验证门禁、role 职责 4/原则 3 三处口径一致，措辞可判定 | 逐字落实 |
| F7 | example 全具体值禁占位符 | spec §4.2 书写约定（引用 SOTA 结论 10）；design/bdd 全部 example 为具体值；rg 复查文档集无尖括号占位符 example 残留 | 逐字落实 |

---

## 四、事实抽查（复核者亲自对照源码，非采信文档行号）

### 4.1 tdd 所称 29 处改写点：随机抽 8 处对照 cli_integration.rs（v0.4.0，382 行）

| tdd 声称 | 复核实测 cli_integration.rs | 结果 |
|---|---|---|
| L39 profile_create_json 中 --name bob | L39 `["--json","profile","create",path,"--name","bob"]` | 吻合 |
| L118 post send --from alice "I think we should use Rust." | L118 逐字吻合 | 吻合 |
| L182 post edit --seq 2 --from bob edited | L182 逐字吻合 | 吻合 |
| L188 read --from 2 --to 2 原样保留 | L188 逐字吻合（保留点） | 吻合 |
| L220 brief create --title "My Brief" | L220 逐字吻合 | 吻合 |
| L250 brief remove --entry-title e.txt | L250 逐字吻合 | 吻合 |
| L297 contacts 测试前置 profile create --name agent | L297 逐字吻合 | 吻合 |
| L358 quiet 测试 --name q | L358 `["--quiet","profile","create",path,"--name","q"]` | 吻合 |

8/8 命中。另做独立完备性核验：以 `--name|--title|--entry|--entry-title|--profile|--seq|--from` 全量枚举 cli_integration.rs，旧文法调用点恰为 29 处改写点 + L188（read --from/--to，保留）+ L300（contacts create --title，保留 flag），无清单外遗漏、无误伤。

### 4.2 impl_plan 所称 core example 14 处：随机抽 5 处对照源码

| impl_plan 声称 | 复核实测 | 结果 |
|---|---|---|
| manifest.rs L80 brief create {} --title 形态 | L80 `format!("paperwork brief create {} --title <title>", ...)` | 吻合 |
| manifest.rs L194 同形态（rework 补漏点） | L194 同形态 | 吻合 |
| thread.rs L305 post edit --seq/--from 形态 | L305 `format!("paperwork post edit {} --seq {} --from {} <body>", ...)` | 吻合 |
| contacts.rs L22 contacts add --profile 形态 | L22 `format!("paperwork contacts add {} --profile <path>", ...)` | 吻合 |
| profile.rs L91 profile create --name 形态 | L91 `format!("paperwork profile create {} --name <agent>", ...)` | 吻合 |

5/5 命中。全量核验：rg `paperwork (post|brief|contacts|profile)` 扫 paperwork-core/src 全部输出，旧文法 example 恰 14 处（thread.rs L138/L228/L275/L305/L326/L341 共 6 + manifest.rs L32/L80/L105/L151/L194 共 5 + contacts.rs L22 共 1 + profile.rs L61/L91 共 2），与清单一致；不变文法 5 处（thread.rs L288 post read、manifest.rs L172 brief read、contacts.rs L56/L98 contacts create、profile.rs L20 profile edit --model）均为单位置参数或保留 flag 形态，确属「勿误刷」，与 impl_plan 步骤④ 声明一致。

注：core 层 example 现含 `<name>/<body>/<title>` 尖括号占位——这 14 处正是步骤④要刷新为具体值的对象（spec §4.2 覆盖 CLI 层与 core 层），与 F7 不冲突，实施后须按 F7 落具体值。

---

## 五、回归检查（修订是否引入新的内部不一致，rg 自查）

| 检查项 | 结果 |
|---|---|
| 数字 29 一致性 | tdd §1 标题、§1 合计式、impl_plan 步骤⑤ 三处均为 29 处改写；rg 无「约 24」「26 处」残留 |
| 数字 14 一致性 | design §7.4 裁定 6、impl_plan 步骤④、tdd §4、role L17 四处均为 14 处；「11 处」「13 处」仅存于裁定 6 对历史矛盾的转述（可接受）；「以实施时检索为准」延迟兜底表述已删除 |
| category 词表 | 六类运行时 + 第七类 usage 在 spec §4.2/§4.3/§4.4/§6.1、design §9、role 职责 3 口径一致；「可扩展封闭集合」矛盾措辞 0 残留 |
| 退出码 | 0/1/2 语义在 spec §4.4、bdd（各场景 exit 标注）、tdd §3（.code 断言）、impl_plan 步骤③⑥ 一致；usage=2、运行时六类=1 无漂移 |
| 字段名 | implicit-mention 全文单数（含 role），复数 0 残留；showing/window/command 三新字段名在 spec/bdd/tdd/impl_plan/design 一致 |
| 签名表 | spec §2 全表与 §3 逐命令契约、design §2-6 代码块、bdd 全部 When 子句交叉比对无签名漂移 |
| 新发现的不一致 | 一项：bdd 新增场景中 6 个（S-CREATE-02/S-CREATE-03/S-PROF-02/S-READ-04/S-BRIEF-07/S-CONTACTS-05）在 tdd §3 无对应用例行，与 tdd 文首「S-xxx 与本文用例一一对应」承诺不符——见 NF-3 |

---

## 六、代码零触碰验证（git status / git diff / mtime 取证）

**复核者本轮多次执行 `git status --porcelain` + `git diff --stat`，以最后一次完整输出为准：**

- 工作区共 9 个已跟踪代码文件被修改，其中 8 个源码文件位于 `repos/paperwork-core/src/`（`git diff --stat` 计 +1725 / -1396 行），另有 1 个测试文件：
  - `format/contacts.rs`、`format/manifest.rs`、`format/mod.rs`、`format/profile.rs`、`format/thread.rs`
  - `ops/contacts.rs`、`ops/thread.rs`
  - `lib.rs`（+16/-4）
  - `tests/ops_tests.rs`（format-v2 tdd §3 的 ops 层测试改写对象；复核收尾时点最新 git status 新出现，佐证 format-v2 线程仍在活动）
- 未跟踪项全部为 docs（含本轮三份评审报告、本复核报告、format-v2 文档目录、v0.5_feedbacks 等）。
- cli-ux-redesign 直接改写对象 `repos/paperwork-cli/tests/cli_integration.rs` 未被修改（mtime 仍为 2026/8/2 2:53:19，git 无 diff），`paperwork-cli` crate 与 `paperwork-core` 的 cmd/validate 层均无改动。

**归属取证（8 个代码文件变更属并行 format-v2 线程，非本轮 rework 引入）：**

1. 内容对照：`lib.rs` diff 为新增 `ThreadMeta { title, participants }` 与 `ContactEntry` 由 `{profile_path, summary}` 改为 `{label, profile_path}`——逐字对应 `docs/dev/format-v2/design.md` §4 第 6 条与 `impl_plan.md` S1.3/S1.6/S2.4（R11 label 派生）；与 cli-ux-redesign 文档集无任何对应关系（后者明示 core API 零变更、仅刷新 14 处 example 字符串）。
2. 时间对照：format-v2 文档 mtime 0:50:42–0:53:26，cli-ux-redesign rework 文档 mtime 0:50:36–0:54:07，`lib.rs` 代码 mtime 1:03:07 晚于两组文档，序列上与「format-v2 线程实施 S1 阶段」自洽；两组文档时间重叠，证实两线程并行进行。
3. 范围对照：被改 8 文件恰为 format-v2 impl_plan S1/S2 阶段清单（format 五件 + ops 两件 + lib.rs）；cli-ux-redesign 实施范围（步骤①~⑥涉及的 cmd 层、信封层、cli_integration.rs、core example 字符串）零改动。

**判定：rework 轮自身未触碰任何代码；但「工作区仅 docs 变更」严格不成立——存在并行线程的 core 代码变更（+1725/-1396）。** 该事实不构成本轮文档闭合判据的违反（复核任务口径为「本轮 rework 是否改代码」），但 ContactEntry 字段变更属 core 公开 API 的 breaking change，与 cli-ux-redesign spec §6「core API 零变更」基线存在潜在冲突，须由编排层裁定两线程版本顺序与提交边界，见 NF-1。

---

## 七、新发现问题（本轮修订引入或复核中暴露）

### NF-1（环境事实，需编排层裁定，非文档缺陷）

- **现象**：工作区 8 个 paperwork-core 代码文件已被修改（+1725/-1396），归属并行 format-v2 线程 S1/S2 阶段实施（第六节取证；收尾时点又新增 `tests/ops_tests.rs` 进入修改清单，进一步佐证）。
- **冲突点**：`ContactEntry` 由 `{profile_path, summary}` 改为 `{label, profile_path}`，属 core 公开 API 的字段级 breaking change；而 cli-ux-redesign spec §6 以「core API 零变更」为输出协议冻结基线，其 impl_plan 步骤④的 14 处 example 刷新也假定 core 结构不变。两线程若先后合入，后合入一方须承担适配成本；CHANGELOG/版本号语义（谁吃 0.5.0、谁升 0.6.0）也需明确。
- **建议**：编排层在 Task 9 开工前裁定：(a) 两线程的版本顺序与提交边界；(b) 若 format-v2 先合入，cli-ux-redesign spec §6 基线须追加一条「以合入时 core API 为准」的适配注记；(c) format-v2 design §4 第 6 条「lib.rs 仅新增 ThreadMeta」的表述与实际 lib.rs diff（含 ContactEntry 字段变更）不一致，建议 format-v2 线程自行勘误（超出本复核范围，仅提示）。

### NF-2（Minor 残留，非阻塞）

- **现象**：agent-ux M4 建议②未落——「`-` 开头 body 未加 `--` 直接传参被 clap 吞掉/误判」的 usage 负形态无 BDD 场景；spec §4.3 未要求 usage 信封的 fix 提及 `--`。
- **现状**：`--` 教学目前钉在 validation fix（spec §4.2）与 after_help（design §2.4）两层，S-SEND-07/S-EDIT-05 覆盖正形态。残留风险有限但不为零（clap 对未知 `--xxx` 报 usage 错误时，用户可能不知道 `--` 边界的存在）。
- **建议**：Task 9 实施前或实施中补一条 BDD 负形态场景（如 `post send f.post.md alice -fix`→exit 2 usage、fix 提示 `--`），并在 spec §4.3 usage 信封 fix 约定中加一句「涉及疑似 flag 的 argv 残留时提示 `--` 边界」。可与 NF-3 合并为一次补录提交。

### NF-3（Minor，非阻塞）

- **现象**：bdd 本轮新增场景中 6 个（S-CREATE-02/S-CREATE-03/S-PROF-02/S-READ-04/S-BRIEF-07/S-CONTACTS-05）在 tdd §3 用例表无对应行，与 tdd 文首「本文用例与 bdd.md 场景编号一一对应」的承诺不符。
- **影响**：不影响修复真实性（bdd 场景本身成立），仅映射表缺行；Task 9 实施者若严格按 tdd §3 落测试会漏 6 条断言。
- **建议**：tdd §3 补 6 行（每行注明对应 S-xxx 与断言要点：exit code + category + example 形态）。

### 观察项（不计入未通过清单）

1. bdd 新增场景编号非单调（S-SEND-12/13 插在 09/10 之间、S-CONTACTS-05 排在 04 前）——编号仅要求唯一可引用，无单调性契约，可接受，但后续新增建议追加到节末。
2. spec 第六类冻结表述「已经本次对抗评审确认」的时态超前于本次闭合复核——一轮三份评审确实未反对第七类 usage 扩展（均只质疑实现机制），表述可接受；本复核闭合后即名副其实。

---

## 八、最终结论

**结论：闭合（文档层）。**

- 三份评审报告全部 Critical/Major/Minor（清单 41 条目、去重后 36 项独立问题）中：已修复 35、已修复带轻微残留 1（agent-ux M4，即 NF-2）、未修复 0、修复不当 0。
- 编排层裁定 F1-F7 共 7/7 逐字忠实落实，重点条款（F1 两场景 argv 形态互斥、F4 只读命令不建文件条款三处同文、F5 穿透条款四处同文）经逐字比对确认。
- 事实抽查：tdd 29 处改写清单抽 8/8 吻合且独立全量枚举盘净（29 改写 + 2 保留，无清单外遗漏）；core example 14 处抽 5/5 吻合且全量枚举盘净（14 旧文法 + 5 不变文法，与「勿误刷」声明一致）。
- 回归自查：数字 29/14、category 词表、退出码、字段名（implicit-mention 单数）、签名表全文一致，未发现修订引入的 Critical/Major 级新不一致。
- 代码零触碰：rework 轮自身未触碰任何代码（cli_integration.rs 与全部 cli-ux 范围文件零改动）；工作区存在的 8 个 core 代码文件变更经取证归属并行 format-v2 线程，非本轮引入（NF-1，需编排层裁定版本顺序，不阻塞文档闭合）。

**放行条件**：可按 impl_plan 步骤①进入实施（F6 分阶段门禁生效）。NF-2/NF-3 为 Minor 补录项，建议在 Task 9 开工前一次性补录（合计约 1 条 BDD 场景 + 6 行 tdd 用例 + 2 句 spec 措辞），不构成本轮闭合的阻塞项。NF-1 提交编排层裁定，裁定结果若影响 spec §6 基线须回写一行适配注记。

（完）
