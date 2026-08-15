# 文档与 SSOT 一致性深审 —— 第二轮增量复审报告（S2 系列）

- 日期：2026-08-15
- 审计基线：master @ 46b1f47（任务 #44，纯只读取证；本报告为唯一落盘产物）
- 前序审计：docs/dev/audit-ssot-agentux-2026-08-15.md（S-01~S-13，已闭环）
- 背景：2026-08-15 owner 四项裁决实施（写侧糖参数撤销 / contacts advisory / 读侧过滤器保留 / completions 钉住 + agent-first 方向）与任务 #52 回填批合并入 master 后的增量复审
- 方法：只读取证 + 隔离区实测盲测（target/audit-round2/，target/release/paperwork.exe，cargo build --release 确认二进制与 HEAD 源码一致）

## 一、结论概览

- 发现数：4 项。分级：重要 1（S2-01）、低 2（S2-02、S2-04）、事实 1（S2-03）。无阻塞项。
- 前序 S-01~S-13 修复逐项复核：全部在场（S-09 同类型在另一勾选项位置复发，见 S2-02）。
- 盲测：模拟新 agent 仅凭 SKILL.md 走核心流程，约 30 项探针，无迷路点。

## 二、发现详情（S2-xx，双方出处 + 证据）

### S2-01（重要）_e2e/smoke.ps1 残留已撤销写侧糖标志 —— FR-1 规则自身所列扫查面漏改

- 出处 A（代码面）：_e2e/smoke.ps1 L38 仍调用 `post send ... --reply-to 1 --mention alice`（写侧糖衣 flag，2026-08-15 裁决已撤销）。
- 出处 B（规则面）：docs/dev/open-items-ledger-2026-08-15.md 第十五节 FR-1（及提交 8571186 信息）明确要求：CLI flag 增删必须全仓扫查 .github/workflows/*.yml、SKILL.md、README.md、_e2e/*。本项正是 FR-1 所列 _e2e/* 面的漏改。
- 实测证据：按该行参数逐字执行 → exit 2，`error usage: unexpected argument '--reply-to' found`，fix 含撤销教学文案（--reply-to was removed from write commands (owner ruling 2026-08-15)...）。脚本 ErrorActionPreference=Continue 不会中止，但该步 reply/mention 语义落空，后续 post read/edit 冒烟偏离脚本意图。
- 影响与定级：ci.yml 不引用该脚本（docs/dev/ci-full-revalidation-2026-08-15.md 已确认），不影响 CI，故不定阻塞；但属 FR-1 明列扫查面的确定性失败，定重要。
- 修复建议：L38 改正文直书形态（如 -m "@#1 Tests merged. cc @alice"），并顺带重跑 FR-1 扫查清单确认无其他漏网。

### S2-02（低）cli-grammar-v0.6/README.md 治理清单「任务 #36 实施」勾选项过期

- 出处 A：docs/ssot/specs/cli-grammar-v0.6/README.md L65 仍为未勾选：「[ ] owner 裁决批实施（任务 #36：impl_plan O1~O5...）」。
- 出处 B：任务 #36 实施已完成入库（9821933..b9b059c）。docs/dev/open-items-ledger-2026-08-15.md 第十四节实施链终态、docs/dev/rulings-execution-log-2026-08-15.md O1~O5 执行记录、impl_plan L192 O5 偏离裁定登记均确认完成并经任务 #37 验证与三维评审。
- 性质：与前序 S-09（治理状态过期）同型，复发于另一勾选项位置。
- 修复建议：勾选 [x] 并注明完成日期与验证承载（rulings-execution-log / rulings-verification 报告）。

### S2-03（事实）format-v2/spec.md OQ-4/§5.7 无撤销裁决指针注记

- 出处 A：docs/dev/format-v2/spec.md §5.7 L225 仍写「--reply-to/--mention 糖衣 flag 的去留见 §11 OQ-4」；OQ-4 L417 默认「保留这两个 flag」。全文 grep「2026-08-15 / 撤销」零命中，无裁决指针。
- 出处 B：cli-grammar-v0.6 spec §5 与 docs/dev/owner-rulings-2026-08-15.md：写侧糖 flag 已撤销（SSOT 以 cli-grammar-v0.6 为准）。
- 评估：口径不算错——OQ-4 原文自带「如 leader 另有裁决以裁决为准」逃逸条款，format-v2 属历史 spec，冲突处以 cli-grammar-v0.6 为准。但回填批 F2 已回改本文档 §5.7/§7.3 R11 suffix 链口径，建议顺带在 OQ-4 加一行裁决指针注记闭环。定事实记录。

### S2-04（低）两份 role 文档缺「历史归档 / 以 spec 为准」标注

- 出处 A：docs/roles/cli-grammar-v0.6-implementer.role.md 与 docs/roles/cli-ux-redesign-implementer.role.md 头部均无归档或 superseded 注记（grep「归档 / Superseded」仅命中职责正文，无声明）。
- 出处 B（同目录对照模板）：docs/roles/format-v2-implementer.md L3 已有归档声明（任务 #37 Paul S-1 销账），明示历史角色剧本、冲突处以 cli-grammar-v0.6 spec 为准。
- 评估：cli-ux-redesign role 含 v0.5 时代已撤销文法口径（位置参数 NAME/BODY、--from/--seq 移除前的基线描述），新 agent 误当现行教学的风险较高；v0.6 role 的文法主体仍现行，仅裁决批后局部过期，风险较低。两者模式不一致，建议统一补一行归档声明。定低。

## 三、前序 S-01~S-13 修复复核（全部在场）

| 编号 | 原发现 | 复核结果与证据 |
|------|--------|----------------|
| S-01 | README 安装引导与版本错配 | 在场：根 README L24-27 与 L72-77 两处版本警示（crates.io 0.5.0 为旧文法、仓库源码为新文法） |
| S-02 | 输出协议声明缺失 / ASCII 承诺矛盾 | 在场：根 README L162-171 UTF-8 与信封契约；SKILL.md L36-45 编码契约；实测信封结构面纯 ASCII 成立 |
| S-03 | 锁行为未进 agent 可见文档 | 在场：SKILL.md L47-56 锁行为小节 |
| S-04 | Commands 节缺两动词示例 | 在场：根 README 已含 contacts remove/update 与 brief read --entry-title 示例 |
| S-05 | usage 信封示例与实测不符 | 在场：实测 usage 信封与 README/SKILL 示例逐字一致 |
| S-06 | contacts read 注释与输出不符 | 在场：实测输出形态与文档一致 |
| S-07 | 裸 .md 替换未文档化 | 在场：SKILL.md 已文档化，实测裸 .md 解析为 .profile.md 复现 |
| S-08 | cli-ux-redesign 缺反向 superseded-by | 在场：docs/ssot/specs/cli-ux-redesign/README.md 头部注记 |
| S-09 | 治理清单过期 | 当时修复在场；同型复发于另一勾选项 → 本轮 S2-02 |
| S-10 | bdd 两守卫场景 tdd 无映射 | 在场：tdd.md L245（S-CONTACTS-15）/L249（S-BRIEF-10）任务 #34 补映射，含既有测试在场实证 |
| S-11 | S-SHORT-02 计数口径含糊 | 在场：bdd.md L510 改枚举口径并明示「总数不写死」 |
| S-12 | ledger/backlog 待合并口径过期 | 在场：台账已推进至第十五节；backlog 第八/九节与文末更正注记齐备 |
| S-13 | 三份新文档未提交 | 闭环：三份文档均已入库 HEAD |

## 四、七审计面核验通过项

1. **SKILL.md agent 入口（逐条实测，全绿）**：L103-106 撤销声明在场，全文无写侧 --reply-to/--mention 教学残留；advisory 三形态（default/--json/不触发）披露于 L140-145 且实测复现（exit 0，同名 key，合法 destination 无该字段）；UTF-8 契约 L36-45、锁行为 L47-56 在场；读侧过滤器、-q/--json/--plain、--title 对既有线程静默忽略、等号形态 -m="--stdin" 逐字写入均与教学一致。
2. **README 面**：根 README、repos/paperwork-cli/README.md、CHANGELOG 与 0.5.0 未发布现状一致；CHANGELOG Unreleased 段含 Removed（糖参数）与 Added（advisory），历史 NEW-12/NEW-10 条目带更正标注（L46-51），已发布 0.5.0 文法段带 superseded 声明（L266-274）；无 bump/tag/publish。
3. **spec 套件内一致**：S-SEND-20/22/23、S-EDIT-10、S-CONTACTS-16/17 编号链在 spec/bdd/tdd/impl_plan 互引无漂移；短形式集合 {-a,-m,-q} 口径五份文档一致；回填批 suffix 链三闭环在场——format-v2 spec §5.7 L225（title 链）与 §7.3 R11 L344（label 链）文档、format/mod.rs L323-343 实现（strip_title_suffix / strip_label_suffix）、L506-526 退化场景单测。
4. **台账账目链**：open-items-ledger（第十二~十五节）、fix-ledger 第九节 CI-F1、rulings-execution-log、ci-failure-diagnosis、ci-full-revalidation、perfection-execution-log、v05-wip-backport 交叉引用一致（LED-17/CI-F1/FR-1/O5 偏离登记互指），无自相矛盾与过期口径；append-only 纪律保持（以追加节刷新口径、不改写历史正文）。
5. **冻结面声明准确性**：flag_inventory_matches_spec（tests/cli_integration.rs L2041+）钉住 send 侧撤销 flag 负向断言与 read 侧过滤器保留断言；VALUE_TAKING_FLAGS（main.rs L294 附近）与 spec §5 第 5 条口径一致（--reply-to/--mention 作为 read 侧带值 flag 保留在列）；输出协议「只增不改」声明与 advisory 实现一致（新字段不触碰既有 key）；黄金快照重冻清单登记于 impl_plan O3/O5 与 rulings-execution-log。
6. **历史文档归档状态**：cli-ux-redesign README Superseded-by 在场；format-v2-implementer 归档声明在场；backlog 更正注记在场；缺口仅两份 role 文档（→ S2-04）。docs/reviews/v0.5-debt-closure-ledger 来源注记在场。
7. **盲测复核（模拟新 agent 仅凭 SKILL.md）**：隔离区 target/audit-round2/ 依序走 profile create/edit → contacts 全 CRUD（含 advisory 触发与不触发）→ post send/read/summary/edit（含裁决后正文直书 @#N/@name 写法、读侧 --mention/--reply-to 过滤、implicit-mention 触发边界：正文无显式 @name 时触发、显式时不触发）→ brief add/read --entry-title → validate，约 30 项探针全部与 SKILL.md 教学一致，无迷路点。

## 五、外围观察（事实注记，不计 S2 发现）

- git status 见三个未跟踪文件：CON.post.md 与 NUL.post.md（疑为深审任务 #42/#43 并发/边界探针产物，非文档面）、docs/dev/repo-state-inventory-2026-08-15.md（任务 #41 进行中产物，尚未提交，类比前序 S-13 形态，随其所属任务闭环即可）。
- design.md L47/L266 旧口径残留已由 impl_plan L202「自查遗留差异点名」登记（冲突处以 spec 为准，按冻结纪律不回改），无需另行处理。

## 六、修复优先级建议

1. S2-01（重要）：修复波同步 _e2e/smoke.ps1 L38，并按 FR-1 清单重跑全仓扫查（.github/workflows/*.yml、SKILL.md、README.md、_e2e/*）确认无其他漏网。
2. S2-02 / S2-04（低）：治理清单勾选回补 + 两份 role 文档头部补一行归档声明（仿 format-v2-implementer L3 模板）。
3. S2-03（事实）：可与 S2-02 同批顺带在 format-v2 spec OQ-4 加裁决指针，亦可延后（不影响现行口径正确性）。

—— 报告终（审计执行：任务 #44，只读取证，未改任何仓库文件与 git 状态，唯一落盘为本报告）
