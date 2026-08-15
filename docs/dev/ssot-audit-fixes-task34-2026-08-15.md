# 任务 #34：SSOT 走查文档侧发现修复 — 销账报告

- 日期：2026-08-15
- 任务：#34（纯文档修复：零 .rs / Cargo.toml / 测试文件变更，零 git 提交）
- 权威输入：docs/dev/audit-ssot-agentux-2026-08-15.md（深审 C，13 项发现：3 重要 + 9 低 + 1 事实记录）
- 口径说明：任务书预告「1 重要 + 9 低」与报告实盘（3 重要）不符，已按 leader 裁定以报告为准
- 纪律执行：未编辑审计文件本体（Tina 仍在补写期间规避写冲突）；未 add/commit/stash；改动留待编排层统一提交

---

## 一、逐条 S-xx 销账表

| 编号 | 严重度 | 处置 | 销账说明 |
|---|---|---|---|
| S-01 | 重要 | 修复 | 根 README Quick start 与 Install 节各加版本警示块：crates.io 0.5.0 = v0.5 位置文法 + 旧格式，本文档对应未发布 master，引导 from source 安装；Install 节 crates.io 小标题去掉「recommended」并标注所发布版本形态。发布轮统一消除 |
| S-02 | 重要 | 修复 | SKILL.md 新增「Output encoding」节、根 README Output protocol 节新增「Encoding contract」段：输出恒合法 UTF-8、消费端须按 UTF-8 解码、信封结构面纯 ASCII、io 类 message 可内嵌 OS 本地化文本、PowerShell `[Console]::OutputEncoding` 规避提示（io 根因报告 §6 建议全文落地）；README「All output is pure ASCII」与「ASCII-only envelope」两处绝对表述按「结构面 ASCII + 字节恒合法 UTF-8」口径收窄；spec.md §5 第 4 条同步收窄（纯文档口径，`ascii_output_contract_guard` 测试防线措辞保留，行为面零变更） |
| S-03 | 重要 | 修复 | SKILL.md 新增「Write commands and file locks」节：六写路径 + post send/edit 经排他文件锁串行化、临界区毫秒级、等待无内建超时、编排层可施加进程级超时后杀进程重试（add 幂等、其余先读后写，重试安全）、持锁进程消亡后 OS 自动释放锁（spec §3.9 契约的 agent 可见摘要） |
| S-04 | 低 | 修复 | 根 README Commands 节补 `post edit`（--author/--seq/--message 三必填）与 `brief remove`（--entry-title 键语义）各一行示例 |
| S-05 | 低 | 修复 | 根 README usage 信封示例删尾部省略号；实测逐字复核：`error usage: the following required arguments were not provided: --author <AUTHOR>`（本报告 §三 PROBE-U） |
| S-06 | 低 | 修复 | 根 README contacts read 注释改为 `# shows stored path + name (+ description)`；实测输出形态 `<存储路径>: <name> (<description>)` 与注释吻合（PROBE-C） |
| S-07 | 低 | 修复 | SKILL.md 规则 4 补半句「以裸 .md 结尾的路径被替换为类型后缀（`notes.md` -> `notes.post.md`）」；repos/paperwork-cli/README.md Quick Example 三行示例 `./alice.md`/`./thread.md` 改裸名 `alice`/`thread`；裸 .md 替换行为实测复现（PROBE-M） |
| S-08 | 低 | 修复 | cli-ux-redesign/README.md 顶部与 spec.md 头部各加 Superseded-by 注记（体例对齐 adr-v1.md 双语双指针：文法层以 cli-grammar-v0.6/spec.md 为准、本套件为历史档案、正文不可改写）；README 治理清单追加一条 2026-08-15「实现后事追加」勾选项结案原未勾项，历史行未改写 |
| S-09 | 低 | 修复 | cli-grammar-v0.6/README.md 状态行刷新为「实现已随 master @ 3829fd9 三方合并生效，发布待 owner 裁定」；治理清单按既成事实勾选对抗评审闭合确认 / 基线合并 / 步骤(1)~(7) 实现三项（各附证据：3829fd9、288 测试基线、任务 #19/#20）；发布项保留未勾并注记「owner 裁定延后（v0.6_feedbacks §一(3)）」 |
| S-10 | 低 | 修复 | cli-grammar-v0.6/tdd.md 本轮增量映射表补两行：S-CONTACTS-15（add/update 空键护栏）与 S-BRIEF-10（brief read --entry-title 空值守栏），均指向既有测试覆盖并 grep 实证（core 单测 ops_contacts_crud_tests.rs L398/L421、集成测试 cli_integration.rs L4445/L4490，实现侧 cmd/brief.rs L191 与 ops/contacts.rs L78/L179 在场）——四件套互引链两断点闭合，未新增测试文件（纯文档） |
| S-11 | 低 | 修复 | bdd.md S-SHORT-02「共 26 项」改为「总数不写死，以下枚举为准」并注分项口径与勘误来由；tdd.md §8.3 第 1 条联动同步（去掉「25 项 + 1 = 26 项」硬编码），后续短形式增删以枚举为唯一对账口径 |
| S-12 | 低 | 修复 | open-items-ledger-2026-08-15.md 追加第八节（append-only，未动第一至七节）：「待合并」一律改读为「已随 master @ 3829fd9 合入」，列出波及条目 U-01/U-08/U-10/U-11/U-14/U-15/Q-03/N-01/N-02，交叉验证结论与审计一致；backlog 本体经 grep 实证无「待合并」字样（唯一「合并」命中为 U-09 议题名），无需追加 |
| S-13 | 事实 | 钉住 | 提交时机归编排层/统一提交流程；本任务依任务书不执行任何 git 提交。注：本轮新增本报告后 docs/dev/ 未提交文档为四份（原三份 + 本销账报告），一并在统一提交清单内 |

销账统计：修复 12 / 移交修复波 0 / 钉住 1（S-13，事实记录非缺陷）。

---

## 二、移交修复波清单（代码侧，本轮未动）

1. **ASCII 契约与测试防线的口径对齐评估**（源自 S-02 第 3 点的代码面）：spec §5 第 4 条已按「结构面 ASCII」口径收窄，但 `ascii_output_contract_guard` 集成测试防线若对 message 字段做全字节 ASCII 断言，则与「io message 可内嵌 OS 本地化文本」的实测存在潜在张力（io 根因报告 §6 评估为产品无缺陷、不建议代码硬化去本地化文本）。是否调整测试断言面或做 locale 无关化，由修复波（任务 #27）评估；文档面本轮已按审计建议口径收窄完毕。
2. **crates.io 与文档的版本对齐**（S-01 根治项）：警示块为过渡措施，根治依赖发布轮（owner 裁定时机）把 v0.6 具名文法 + v2 格式发布至 crates.io。

以上两项均非文档可独立闭合面，文档侧已尽披露义务。

---

## 三、示例实测验证结果（TEMP 夹具，零仓库残留）

方法：target/debug/paperwork.exe（master @ 3829fd9 合并后工作区构建在场），夹具目录 `$env:TEMP\pwtask34-*`（脚本结束即删除），验证脚本置于 gitignored target/ 并于验证后删除；会话先设 `[Console]::OutputEncoding = UTF8`（S-02 声明的规避动作本身亦被验证）。

| 探针 | 内容 | 结果 |
|---|---|---|
| PROBE-FLOW | SKILL.md 全流程：profile create/show/edit/list -> contacts create/add/read/update/remove -> post send(建线程)/send(回复)/read/summary/edit/read --json -> brief create/add/read(TOC/--full/--entry-title)/verify/remove -> validate(后缀推断 + --type) | 全部 exit 0，信封字段与 SKILL/README 描述逐条吻合（implicit-mention、showing 2/2、window #1-#2、updated 箭头串、removed 字段均在场） |
| PROBE-C | contacts read 输出形态（S-06） | `agents/alice.profile.md: alice (Parser owner)` —— 主列 = 存储路径，注释修正成立 |
| PROBE-U | usage 信封逐字（S-05） | `error usage: the following required arguments were not provided: --author <AUTHOR>`（无尾省略号）+ fix + canonical example，exit 2；`--message`/`--stdin` 同传亦 usage exit 2 |
| PROBE-NF | not-found 信封 | `error not-found: Thread 'ghost.post.md' not found` + fix + example，exit 1 |
| PROBE-Q | --quiet 写命令 | status 行抑制、字段保留、exit 码语义不变 |
| PROBE-M | 裸 .md 替换（S-07） | `profile create ./probe1.md` 落盘 probe1.profile.md；`post send ./probe2.md` 落盘 probe2.post.md —— SKILL 规则 4 补句与实测一致 |

自检结论：新 agent 仅凭修复后的 SKILL.md 可一次走通 profile -> contacts -> post -> brief 全流程（含错误自愈面与机器可读面），与审计第四节盲测结论互为印证；仓库内无验证残留（git status 无新增非文档产物）。

---

## 四、新建与修改文件清单

新建（1）：

- docs/dev/ssot-audit-fixes-task34-2026-08-15.md（本报告）

修改（10）：

| 文件 | 对应发现 |
|---|---|
| README.md（根） | S-01、S-02、S-04、S-05、S-06 |
| SKILL.md（根） | S-02、S-03、S-07 |
| repos/paperwork-cli/README.md | S-07 |
| docs/ssot/specs/cli-ux-redesign/README.md | S-08 |
| docs/ssot/specs/cli-ux-redesign/spec.md | S-08 |
| docs/ssot/specs/cli-grammar-v0.6/README.md | S-09 |
| docs/ssot/specs/cli-grammar-v0.6/tdd.md | S-10、S-11 |
| docs/ssot/specs/cli-grammar-v0.6/bdd.md | S-11 |
| docs/ssot/specs/cli-grammar-v0.6/spec.md | S-02（§5 第 4 条口径收窄） |
| docs/dev/open-items-ledger-2026-08-15.md | S-12（append-only 追加第八节） |

未修改：任何 .rs 源码、Cargo.toml、测试文件、审计文件本体、CHANGELOG、backlog 本体（无过期字样）、cli-grammar-v0.6 其余文档（design/impl_plan/bdd 其余段落）。

---

（报告完。任务 #34 文档侧 13 项发现：12 修复 + 1 事实钉住；移交修复波 2 项均为代码/发布面，文档披露义务已尽。）
