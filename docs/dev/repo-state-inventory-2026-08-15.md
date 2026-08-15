# agent-paperwork 仓库现状全量盘点报告（repo state inventory）

- 日期：2026-08-15
- 任务：#41 全量盘点（纯调研：只读取证，不改文件、不提交、不推送）
- 取证基线：master @ 46b1f47（= origin/master，领先/落后 0/0；工作区干净）；`cargo test --workspace --locked` 本盘点现场复跑 **444 全绿**（7+33+148+16+4+102+12+33+18+71+0）；CI 双绿（run 31879040813 @3ef5dc5、run 31880791223 @8571186）
- 背景已闭合面：owner 四项裁决实施（O1~O5 + 修复轮 46c637c）、任务 #52 v0.5-perfection Plan-C 选择性回填（0b648d7/f94b65f/3ef5dc5）、CI smoke 失败事件闭环（台账第十五节 LED-17 + fix-ledger 第九节）
- 纪律：全部条目基于文件/命令实证；编号 INV-xx 只增不复用；分级 = 缺陷 / 实现不完全 / 文档债 / 卫生债 / 流程债 / 仅登记 / 禁区登记

---

## 一、台账终态核对（open-items-ledger 十五节，LED-01~17）

抽查方法：逐条比对当前 git/文件现场与台账登记口径。

| LED | 台账口径 | 现场核实 | 结论 |
|---|---|---|---|
| LED-01 分支未合/未推 | 已闭合（第十一节） | 合并面属实（cli-grammar-v0.6 三方合并 3829fd9）；推送面属实（origin/master 同步）；**但收尾动作「合并后本地分支清理」未执行**：`git branch --merged master` 仍含 cli-grammar-v0.6（本地 + origin 双侧在案，merge-base=其顶点 a7bc3e2，零独有内容） | 闭合属实，清理尾巴转卫生债（INV-05） |
| LED-02 工作区未提交 | 已闭合 | `git status -uall` 空，实测干净 | 属实 |
| LED-03 qa-tmp/ | 已闭合 | 无该目录、无未跟踪残留 | 属实 |
| LED-04 io 乱码 | 已闭合（裁定保留，LED-16） | io-encoding-rootcause 报告在案；ASCII 契约收窄已钉测试（fix-ledger D5） | 属实 |
| LED-05 perfection 闭合批 | 已闭合 | perfection-execution-log P-0~P-9 全批终态记录在案 | 属实 |
| LED-06 tdd §1b-G 注记勘误 | 开放（文档轮） | **勘误实质已完成**：tdd.md L107 现有注记「前一版『L482/L497 example 断言不变』注记随重盘点勘误……已入表」，O-1 点名的失实注记已被重写为正确口径 | 可升级闭合；仅台账状态未刷新（INV-03） |
| LED-07 销账计数勘误 | 开放（文档轮） | **仍未勘误**：docs/reviews/cli-grammar-v0.6-doc-review-closure-2026-08-09.md L167 原文「新表按八类组织（共 31 条）」仍在场（实为 28 行，O-2 已钉「后续引用以 28 行为准」） | 仍开放（INV-01） |
| LED-08 写路径计数口径 | 开放（文档轮顺带） | **仍未注明**：contacts-crud research L87「五个写路径」、L151「五处写路径」原文仍在场（下游六写路径口径未回注） | 仍开放（INV-02） |
| LED-09/10/11/12 四项裁决 | 已闭合并已实施推送（第十三/十四节） | 现场全链核实：代码面（O1/O2 日志在案）、advisory 面（CHANGELOG Added 段在案）、测试面（426→444 含 S-SEND-22/23、S-CONTACTS-16/17 用例）、spec/bdd/tdd 修订段在场、backlog 第九节注记在场 | 属实，无尾巴 |
| LED-13 cli-ux-v0.5 | 已闭合 | `git branch -a` 无该分支（本地/远端均无） | 属实 |
| LED-14 perfection 续做 | 已闭合（第十二节任务 #52 追加登记：成果回流 + 分支存档保留） | Plan-C 回填三提交在案（v05-wip-backport 执行结果节） | 属实 |
| LED-15 crates.io 错配 | 登记不改（事实登记） | 事实仍成立：Cargo.toml 双仓 0.5.0、tag 止于 v0.5.0、CHANGELOG 无发布段；owner 从未指示发布 | 维持仅登记（INV-10） |
| LED-16 io 消息豁免 | 已裁定备查 | fix-ledger D5/第七节口径在案 | 属实 |
| LED-17 CI smoke 事件 | 已闭合（第十五节） | 现行 ci.yml L85/L187 已是正文 token 形态（`--message "@#1 Reply @alice"`）+ 裁决注释块；防复发规则 FR-1 三处落点在场 | 属实 |

台账汇总刷新口径（按本盘点实测改读，不回改原文）：
- 登记总数 17（LED-01~17）；已闭合/已裁定 13；仍开放实质 **3 项**：LED-06（实质已勘误、仅差台账闭合刷新）、LED-07、LED-08；事实登记 1 项：LED-15。
- 台账卫生发现：LED-06 状态未随 tdd 勘误刷新（违反 workflow-and-todo §2.3「台账未刷新=未销账」纪律）——本身即一条新文档债（INV-03）。

---

## 二、backlog 与 researches 未闭合条目

docs/researches/ 共 7 份。逐份核查：

| 文件 | 未销账条目 | 现状 |
|---|---|---|
| ux-open-items-backlog-2026-08-08.md | 原开放 4 项（U-04/U-13/B-01/B-02） | **全部闭合**：第九节 owner 四项裁决注记在场（B-01 方向消解、B-02 advisory 落地、U-04 同 B-01、U-13 钉住结案）；文末更正注记（发布口径）在场；U-01~U-15/R/F/Q/N 系列裁定早在台账第五节全部销账 |
| contacts-crud-progressive-read-research | 无开放项 | 仅余 LED-08 所指「五个/五处」历史口径文本（不回改属既定裁决） |
| cli-grammar-v06-reassessment / agent-cli-ux-industry-sota / cli-ux-agent-visible-output-research / research-repo-formats / format-v2-design-synthesis | 无开放项 | 均为历史调研档案，结论已被 v0.6 轮与 format-v2 轮消化（台账第四/五节销账覆盖） |

结论：researches 无未销账条目；唯一可动项为 LED-08（INV-02，成本极低的口径注明）。

---

## 三、spec 套件 vs 实现差异（docs/ssot/specs/cli-grammar-v0.6/）

六份文档（spec/design/bdd/tdd/impl_plan/README）全文扫描「待实施/遗留差异/点名待办」类标记，结果：

1. impl_plan 步骤(0)~(7)、R1~R6、O1~O5：**全部完成**。证据 = 台账第十三/十四节实施链哈希（d920271→O1~O5→46c637c）+ 本盘点 444 测试全绿 + ci-full-revalidation 放行报告。
2. impl_plan「自查遗留差异点名」三条全部核实闭合：
   - design.md 残留旧口径（L47/L266）→ 已按冻结纪律以〔2026-08-15 owner 裁决更正〕注记覆盖（design.md L48-49、L269 实测在场），冲突处以 spec 为准，无需再动；
   - SKILL.md/README 差异 → O4 已消除（本盘点 grep 实测：根 README L116-122、SKILL.md L91/L104-106 仅存读侧过滤器示例与撤销声明，无任何写侧糖 flag 教学残留）；
   - audit-grammar-matrix 计数口径重盘 → 归 O3/O5（S-SHORT-02 收窄、24 口径，任务 #37 I-2 销账）。
3. bdd 裁决场景 S-SEND-20~23、S-EDIT-10、S-CONTACTS-16/17 全部在场且带裁决修订标记；S-CONTACTS-14 静默面契约维持 + advisory 叠加口径在场。
4. README.md 状态行已刷新为「实现已随 master @ 3829fd9 合并生效 + 裁决增量已落盘 + 发布待 owner 裁定」。

结论：spec 套件**无待实施项、无遗留差异**。

---

## 四、CHANGELOG [Unreleased] 完整性（对照 669342e..HEAD 全部提交）

669342e 之后共 5 提交：0b648d7 / f94b65f / 3ef5dc5 / 8571186 / 46b1f47。

| 提交 | 变更面 | CHANGELOG 覆盖 |
|---|---|---|
| 0b648d7 | ivy_gap_tests 16 测试移植（v0.6 文法） | 已录：Internal backport 段点名「Ivy G1-G5 CLI-surface gap tests (16 tests, rewritten to the v0.6 named grammar)」 |
| f94b65f | F2 suffix 双链（strip_title_suffix/strip_label_suffix 替换 strip_known_suffix）、F3 RUSTDOCFLAGS、F4 JsonBuilder 键序文档+单测、F5③ find_message_sender 措辞、ci.yml smoke 纠偏 | 已录：Internal backport 段逐一点名（suffix chains / RUSTDOCFLAGS / JsonBuilder / doc wording / smoke corrected incidental）；「无输出字节变化」声明与实测一致 |
| 3ef5dc5 | docs-only：两份 wip 独有文档归档 + 台账登记 | 已录：「two wip-only documents archived」+ decision record 指针 |
| 8571186 / 46b1f47 | 纯 docs（CI 事件账目、复验报告） | 无行为变更，按 Keep-a-Changelog 惯例无需记录 |

交叉核实：W-1 修复轮对 NEW-12/NEW-10 条目的更正标注（〔2026-08-15 owner-ruling correction〕两段）在场。

结论：[Unreleased] **无遗漏**——行为面（糖 flag 撤销、advisory）与内部面（回填批全部采纳项）均已披露；版本纪律（0.5.0、无 tag、无发布段）保持。

---

## 五、git 资产盘点

- **分支（本地 3 / 远端 3）**：
  - master = origin/master = 46b1f47，rev-list 左右计数 0/0，完全同步；
  - cli-grammar-v0.6（本地 + origin，a7bc3e2）：**已全量合并**（merge-base = 其顶点），零独有内容 → 可清理（INV-05）；
  - wip/v0.5-perfection-snapshot-2026-08-15（本地 + origin，9d63d3b）：owner 裁决「保留作存档，不再处置」（台账第十二节任务 #52 追加登记）且属另一工作流 wip 线 → **禁区，只登记不动**（INV-18）。
- **tag**：v0.2.0 / v0.3.0 / v0.4.0 / v0.5.0，止于 v0.5.0（版本纪律在场，无 0.6 tag）。
- **stash**：无。
- **worktree**：仅主工作区；agent-paperwork-wt-v05perfection 已按任务 #52 退役移除（台账登记在案）。
- **origin 同步**：fetch 口径下 master 与 origin/master 逐字一致；无他人新提交、无分叉。

---

## 六、仓库卫生

现场：`git status -uall` 干净，无未跟踪文件。任务书提示「ci-full-revalidation 可能有未提交改动」——**实测不成立**：该文件已随 46b1f47 提交，工作区零差异（只登记，无需处置）。

被忽略的遗留资产（gitignore `_*/`、`test-*/` 与 .git/info/exclude 覆盖，0 个被跟踪）：

| 目录/文件 | 内容 | 处置建议 |
|---|---|---|
| _wip_stage/ | 历史 staging worktree 全量镜像（含 splice.exe/splice.pdb 二进制、13 份 .diff、旧版 repos 快照） | **建议删除**——任务 #52 闭环后其素材价值已被 v05-wip-backport 决策记录取代；二进制留仓外资产无益（INV-07） |
| _verify_tmp40/ | 任务 #40 现场夹具（spotcheck 脚本 + fixture） | **建议删除**——ci-full-revalidation §6 自称「验证后清理」但目录仍在场，属声明与现场不符的小尾巴（INV-06） |
| _fix/ | 修复波复现脚本与提交文案（repro-audit.ps1、repro-c1.ps1 等，fix-ledger 证据链直接引用） | 二选一：保留为证据链资产（维持现状）或删除并在 fix-ledger 追加「证据链脚本已随轮次归档清理」注记；倾向保留至发布轮（INV-08） |
| _e2e/ | 本地 e2e 资产（smoke.ps1/concurrency.ps1 + 64 份输出文件） | 维持现状；是否纳入 CI/入库为独立候选工作项（ci-full-revalidation L104 已登记），非卫生清理对象（INV-15） |
| test-v03/v04/v05/ | 各代 smoke 语料 | **维持不入仓**——既有裁定在案（CHANGELOG 0.5.0 Added 段明示 test-v05 not tracked；v03/v04 为历史文法语料）（INV-16） |
| _master_lock.rs / _wip_lock.rs | 根目录临时锁实验文件，经 .git/info/exclude 忽略 | exclude 规则不入仓、新克隆会失去该忽略 → 建议改落 .gitignore 或直接删除（INV-09，低） |
| target/ | 构建产物 | 正常，gitignore 覆盖 |

---

## 七、流程债全集（workflow-and-todo / fix-ledger / perfection-execution-log / rulings-execution-log）

1. **workflow-and-todo §四 遗留 owner 决策清单过期**（INV-04）：第 4~7 项（LED-09~12）仍列为待裁，实际已随同日晚些时候的四项裁决闭合（台账第十三/十四节）；§三快照统计「仍开放 8 项」同口径过期。建议文档轮追加修订注记（append-only）。
2. **FR-1 强制规则**（台账第十五节 + fix-ledger 第九节 + workflow-and-todo 验证阶段）：后续批次 CLI 标志增删必查 .github/workflows/*.yml、SKILL.md、README.md、_e2e/*——规则已成文，非待办，仅需后续批次执行（INV-17）。
3. fix-ledger「登记不改」裁定集（全部有明确裁定理由，属**仅登记**，INV-12）：L-1（contains_heading_line 保守策略维持）、I-3（D2 预检锁内读整文件的性能权衡，大线程场景再立项）、I-4（.gitattributes eol=lf 保留；未来协作时一次性 renormalize，发布轮 release notes 提一句——该项与发布轮绑定）、D4 R-12 第二面（等号形态安全 bypass，UX 观察项）、P-6 lossy 四处（纯展示/推断面保留）。
4. ci.yml unix/windows smoke 双份内嵌的结构性重复：「可考虑抽脚本，稳定期不主动改」（fix-ledger 第九节 + 台账第十五节双处登记）——仅登记（INV-13）。
5. LED-16 条件项：若未来 owner 裁决全信封纯 ASCII，需另立专项——仅登记（INV-14）。
6. _e2e 本地资产纳入 CI 的候选（ci-full-revalidation L104）：属新工作项候选，仅登记（INV-15）。
7. perfection-execution-log / rulings-execution-log：两日志「未闭合残留」节均为「无」，偏差登记全部有销账去向（O5 偏离裁定登记 + 承载映射表在案），无新增流程债。

---

## 八、待办全集清单（INV-xx）

| 编号 | 分级 | 事项 | 出处证据 | 处置建议 |
|---|---|---|---|---|
| INV-01 | 文档债 | LED-07 勘误未执行：closure 报告 L167「共 31 条」笔误仍在场（实为 28 行） | docs/reviews/cli-grammar-v0.6-doc-review-closure-2026-08-09.md L167 vs L186 抽验明细 | 文档轮勘误（或按 reviews 历史档案纪律以注记方式更正）；闭合后刷新台账 |
| INV-02 | 文档债 | LED-08 口径注明未执行：research「五个/五处写路径」与下游六写路径口径差异未注明 | contacts-crud-progressive-read-research-2026-08-09.md L87/L151 | 文档轮顺带注明（成本极低） |
| INV-03 | 文档债（台账联动） | LED-06 实质已闭合但台账状态未刷新：tdd §1b-G 失实注记已被 L107 勘误注记取代 | tdd.md L107 实测；台账第一节 LED-06 仍「开放」 | 追加台账刷新节将 LED-06 置为已闭合（附 tdd L107 证据） |
| INV-04 | 文档债（台账联动） | workflow-and-todo §三/§四 过期：LED-09~12 已裁决闭合但清单仍列「待 owner 裁定」；「仍开放 8 项」统计过期 | workflow-and-todo L111-114、L127-130 vs 台账第十三/十四节 | 文档轮 append-only 修订注记 |
| INV-05 | 卫生债 | cli-grammar-v0.6 分支（本地 + origin）已全量合并未清理——LED-01 收尾动作残留 | git branch --merged master 实测；merge-base=a7bc3e2=分支顶点 | 删除本地分支 + push origin 删远端引用（owner 亦可裁决保留远端存档） |
| INV-06 | 卫生债 | _verify_tmp40/ 仍在场（任务 #40 报告自称已清理） | ci-full-revalidation §6 vs 目录实测存在 | 删除 |
| INV-07 | 卫生债 | _wip_stage/ staging 镜像（含二进制 splice.exe/pdb 与 13 份 diff）任务 #52 后无承接价值 | _wip_stage 目录实测；v05-wip-backport 已归档全部 wip 价值面 | 删除（owner 确认后） |
| INV-08 | 卫生债 | _fix/ 证据链脚本目录去留未裁决 | fix-ledger 第一/六节引用 _fix/repro-*.ps1 | 保留至发布轮或删除+补注记（建议保留） |
| INV-09 | 卫生债 | _master_lock.rs/_wip_lock.rs 依赖 .git/info/exclude（机器本地规则，不入仓） | check-ignore 实测命中 .git/info/exclude L8/L9 | 改入 .gitignore 或删除文件 |
| INV-10 | 仅登记 | LED-15 crates.io 0.5.0 与仓库 v0.6 文法语义错配；发布时机待 owner 指示，无发布计划 | 台账第九节 LED-15 + 第十二节更正二 | 维持事实登记；owner 指示发布时一次性闭合 |
| INV-11 | 仅登记 | KL-1~4 钉住项（尾扫缓冲/锁内重写崩溃窗口/brief hash 换行敏感/--title 静默忽略） | 台账第三节 | 备查，不动 |
| INV-12 | 仅登记 | fix-ledger 登记不改裁定集：L-1 保守策略、I-3 性能权衡、I-4 .gitattributes renormalize（发布轮 release notes 提一句）、D4 R-12 第二面、P-6 lossy 四处 | fix-ledger 第六/七节 | 备查；I-4 一项随未来发布轮顺带 |
| INV-13 | 仅登记 | ci.yml 双份内嵌 smoke 结构性重复可抽脚本——稳定期不主动改 | fix-ledger 第九节 + 台账第十五节 | 备查 |
| INV-14 | 仅登记 | LED-16 条件项：未来若裁决全信封纯 ASCII 需另立专项 | 台账第九节 LED-16 | 备查 |
| INV-15 | 流程债（候选工作项） | _e2e/smoke.ps1 与 concurrency.ps1 为本地资产非 CI 面；纳入 CI 属新工作项 | ci-full-revalidation L104 | owner 决定是否立项 |
| INV-16 | 仅登记 | test-v03/v04/v05 本地语料不入仓（既有裁定） | CHANGELOG 0.5.0 Added 段（test-v05 指针） | 维持现状 |
| INV-17 | 仅登记 | FR-1 规则（CLI 标志增删全仓扫查）已成文，后续批次强制执行 | 台账第十五节 | 非待办；执行即可 |
| INV-18 | 禁区登记 | wip/v0.5-perfection-snapshot-2026-08-15 分支（本地 + origin，9d63d3b）——另一工作流 wip 线存档，owner 裁决保留不处置 | 台账第十二节任务 #52 追加登记 | 不动、不删、不推 |

### 统计

- **待办总数：18 项**
- 缺陷：**0**（未发现任何未修复缺陷；444 测试全绿 + CI 双绿 + 三轮评审闭环）
- 实现不完全：**0**（spec/bdd/tdd/impl_plan 全步骤与 owner 裁决实施链全部落地，第三节逐项核实）
- 文档债：**4**（INV-01~04，均为低成本勘误/台账联动刷新）
- 卫生债：**5**（INV-05~09）
- 流程债：**1**（INV-15 候选工作项）
- 仅登记：**7**（INV-10~14、16、17）
- 禁区登记：**1**（INV-18）

### 最高优先项（若 owner 授权下一轮动作）

1. **INV-03/INV-04（台账联动）**：LED-06 闭合刷新 + workflow-and-todo 过期清单修订——纯文档、append-only、恢复台账 SSOT 可信度（本轮已发现「台账未刷新=未销账」再次发生）。
2. **INV-05（cli-grammar-v0.6 分支清理）**：LED-01 明示的收尾动作，--merged 证据确凿，删除无风险。
3. **INV-01/INV-02（LED-07/LED-08 勘误）**：台账最后两个真实开放项，一次文档轮即可清零，使 LED 台账达到「零开放」终态。
4. **INV-06/INV-07（_verify_tmp40/_wip_stage 删除）**：纯本地卫生，删除后仓库根目录遗留面收窄为 _fix/_e2e/test-*（均有既定裁定）。

---

（盘点完。撰写：任务 #41 调研 agent；2026-08-15。本文档只落盘不提交；全部结论可由文内点名的文件位置与 git 命令复核。）
