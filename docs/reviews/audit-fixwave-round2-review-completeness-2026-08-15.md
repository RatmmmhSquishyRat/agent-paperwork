# 审计修复波 Round2 三维评审 —— 完整性（需求覆盖）维度报告

## [Perspective] completeness

- 日期：2026-08-15
- 评审对象：git diff 46b1f47..HEAD（修复波 F1~F4 + docs gate 修复轮：54beff3 / 0ffd9d2 / 0dc23f0 / 04024a8 / 8c10387 / 86776db / fe1899c）
- 需求基线：docs/dev/audit-fixwave-round2-execution-log-2026-08-15.md（F1~F4 + 第七节修复轮二全部承诺项）与四份审计报告处置承诺（repo-state-inventory INV-01~18、audit-grammar-matrix-round2 G2-O1/O2、audit-robustness-round2 R2-01/B-x、audit-ssot-round2 S2-01~04）
- 评审纪律：只读取证 + 独立复跑门禁；未改任何源代码；本评审仅覆盖「完整性」维度（正确性与回归影响面由另评审员负责）

### 独立复核手段（本评审实测，非转述）

| 复核项 | 手段 | 结果 |
|---|---|---|
| 451 全绿声称 | 现场 `cargo test --workspace --locked` | 451 通过 / 0 失败，分布 7+33+154+16+4+103+12+33+18+71+0 与执行日志逐位一致 |
| docs gate 声称 | `cargo clean -p paperwork-core` 后 `RUSTDOCFLAGS="-D warnings" cargo doc -p paperwork-core --no-deps`（规避复验报告 L41 登记的增量缓存伪绿） | exit 0，零警告，`FILE_NOT_UTF8_FIX.html` 生成 |
| S2-01 本地修复 | `_e2e/smoke.ps1` 现场读取 | L36-40 裁决注释 + 正文直书 `-m "@#1 Tests merged. cc @alice"` 在场 |
| FR-1 零残留 | 独立重扫 .github / SKILL.md / README / _e2e / docs/ssot 两 spec 套件 | ci.yml 仅裁决注释（L82/L184）；SKILL/README 命中均为读侧过滤器与撤销声明；_e2e 仅 smoke.ps1 注释 1 处；cli-grammar-v0.6 命中均为负向场景（bdd S-SEND-22/23、S-EDIT-10）与撤销登记；cli-ux-redesign 23 处命中均位于 README 已标 Superseded 的历史档案。写侧糖标志教学面零残留成立 |
| INV-05 分支清理 | `git branch` + `git ls-remote origin` | 本地与远端均无 cli-grammar-v0.6；远端仅 master（fe1899c）与 wip 分支 |
| INV-06/07 删除 | `Test-Path _verify_tmp40` / `Test-Path _wip_stage` | 均 False |
| 禁区保全 | `git ls-remote origin` | wip/v0.5-perfection-snapshot-2026-08-15 本地与 origin 均 9d63d3b，零触碰 |

## Critical Issues (MUST FIX)

无。全部已承诺项均落地（逐项核对见文末「承诺项逐项核对表」），未发现阻塞级漏项或虚假销账。

## Warnings (SHOULD FIX)

### 盲区 B-3/B-4/B-7 既未处置也未登记，与台账第十六节的登记前提和零开放终态声明相抵触
[docs/dev/open-items-ledger-2026-08-15.md#L482-L516](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/open-items-ledger-2026-08-15.md)

**Problem**：audit-robustness-round2 §10 将 B-1~B-8 一并列为「无测试覆盖且产品行为未钉住」盲区，§12 处置建议明示「**B-1~B-8** 为测试/文档增补项……建议纳入下一测试批」。F2 批仅钉住 B-1/B-2/B-5/B-6/B-8（6 测试在场，无争议）；B-3（emoji/组合字符不归一化 roundtrip）、B-4（junction/hardlink 透明读写）、B-7（advisory 大文件探测成本）既无测试钉住、也无任何登记——全仓 grep 三者仅命中审计报告自身正文（[docs/dev/audit-robustness-round2-2026-08-15.md#L168-L172](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/audit-robustness-round2-2026-08-15.md)）。而台账第十六节追加依据明确列出「audit-robustness-round2（B-x 盲区）」，却只登记了 G2/A 族（LED-18~23），并在 L512 声明「仍开放项为零」。同批同源观察项处置不一致（G2-O1/O2、A-01~A-04 均获 LED 编号登记），B-3/B-4/B-7 三条审计派生项失去可追溯去向，「零开放终态」在 B-x 面上不成立。

**Fix**：在台账第十六节（或新追加节）将 B-3/B-4/B-7 登记为观察/延续项（与 LED-20~23 同形态，注明「本修复波范围外，下一测试批承接」）；或在执行日志补显式范围声明并在台账交叉引用。登记后「零开放」口径方可自洽。

### S2-02 勾选销账的提交链证据引用错误：f94b65f 属任务 #52 回填批，非任务 #36 裁决批链端
[docs/ssot/specs/cli-grammar-v0.6/README.md#L65-L65](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/ssot/specs/cli-grammar-v0.6/README.md)

**Problem**：S2-02 处置在 README 勾选项写入「已完成：2026-08-15，提交链 9821933（O1 实施）→f94b65f（ci.yml 内嵌 smoke 回填修正，run 31879040813 全绿实证）」，执行日志第六节表（[audit-fixwave-round2-execution-log#L62](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/audit-fixwave-round2-execution-log-2026-08-15.md)）与 workflow §5.1（[workflow-and-todo#L144](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/workflow-and-todo-2026-08-15.md)）同口径。经 git 实测核验：任务 #36 裁决批实施链为六提交 9821933/14f3b57/77f19e2/6a36639/72c85ac/**b9b059c**（台账第十四节 L436 SSOT 记录在案；审计 S2-02 出处 B 原文亦为「9821933..b9b059c」；`git log --oneline -1 b9b059c` = "O5 (supplement)"）。f94b65f 实为任务 #52 Plan-C 回填提交（commit message "fix: backport residual Ultra Review increments (Plan-C, wip …)"），其 ci.yml smoke 修正归属在 fix-ledger CI-F1 与 ci-failure-diagnosis 中均明确记为「修复者 = 任务 #52 回填批，与裁决批无因果关联」。F3 文档债批新写的销账证据将两个批次混接，与台账 SSOT 及审计报告自相矛盾——属「声称与在案证据不符」，正是本修复波所要清除的文档债同型问题。

**Fix**：勾选项提交链改为 9821933→b9b059c（任务 #36 六提交）；smoke 绿态证据如保留，单独表述为「ci.yml smoke 修正属任务 #52 回填批 f94b65f（run 31879040813 全绿实证）」。执行日志与 workflow §5.1 以追加注记方式同步更正。

## Suggestions (CONSIDER)

### 修复轮二闭环链游离于台账与复验体系之外
[docs/dev/audit-fixwave-round2-execution-log-2026-08-15.md#L102-L132](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/audit-fixwave-round2-execution-log-2026-08-15.md)

**Problem**：两处低危缺口。① 最新复验报告 [audit-fixwave-round2-verification-2026-08-15.md#L19](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/audit-fixwave-round2-verification-2026-08-15.md) 以「最终结论：不放行」终稿，86776db 修复后未产出复验放行报告，闭环唯一证据为执行日志第七节的自报四门禁表（本评审独立复跑已证实质成立：451 全绿 + 强制重 rustdoc 后 docs gate 绿，但证据链形态上缺独立复验落点）。② 台账第十六节 L514 自设纪律「本节起 finding 销账即台账联动即时执行」，但轮二阻塞项销账（86776db）与 FR-2 规则均未联动台账——对照 FR-1 先例为三处落点（台账第十五节 + fix-ledger 第九节 + workflow-and-todo），FR-2 仅执行日志第七节单处登记。

**Fix**：追加轮二复验放行结论（独立报告或在复验报告以追加注记方式补放行终态）；台账追加一条 FR-2 与 86776db 销账的联动登记（指向执行日志第七节即可）。

### FR-2 未并入 workflow-and-todo 门禁清单，根因整改只落了规则未落清单
[docs/dev/workflow-and-todo-2026-08-15.md#L136-L163](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/workflow-and-todo-2026-08-15.md)

**Problem**：轮二根因为「终局门禁清单缺 docs gate」；FR-2 四件套已固化于执行日志第七节，但 workflow-and-todo 的验证/终局门禁表述（含第五节修订节）仍未列入 docs gate，复验报告过程建议 2 明示「建议并入 workflow-and-todo 验证门禁条目」。后续批次若只依 workflow-and-todo 门禁清单执行，仍有复现漏 docs gate的路径。

**Fix**：workflow-and-todo 以追加方式将终局门禁刷新为 FR-2 四件套（指向执行日志第七节 FR-2）。

## 承诺项逐项核对表

| 承诺项 | 核对结果 | 证据 |
|---|---|---|
| S2-01 smoke.ps1 修复（正文直书 + 裁决注释，本地资产登记） | ✅ 在场 | smoke.ps1 L36-40 现场读取；`_e2e/` 属 gitignore `_*/` 覆盖，不入仓口径与登记一致 |
| FR-1 全仓扫查零残留 | ✅ 成立 | 本评审独立重扫五面（见复核手段表），与执行日志声称逐面一致 |
| R2-01 文件通道编码文案（常量 + 构造函数 + 10 core 调用点 + 2 CLI 内联 + 单测，category/exit 不变） | ✅ 在场 | error.rs `FILE_NOT_UTF8_FIX`/`io_ctx_file_read`/`test_io_ctx_file_read_encoding_hint`；thread_read ×3 + thread edit + thread_scan ×2 + lock + contacts + profile + manifest = 10 处迁移；post.rs/validate.rs 内联 InvalidData 判断引用共享常量 |
| B-1/B-2/B-5/B-6/B-8 测试钉住（6 测试） | ✅ 在场 | cli_integration.rs L5322-5589 六测试函数名与断言内容逐条符合执行日志描述（B-2 含新文案逐字断言 + JSON category=io + 字节级零写入；B-5 含 30s timeout；B-6 含 20/2500 与 seq 2501 头尾读回） |
| B-8 语义裁定三文档注记 | ✅ 在场 | spec.md §3.3（L131）/§3.7（L195）、design.md §8（L221）、bdd.md S-READ-10（L255-259），口径一致互引闭合 |
| INV-01（LED-07 勘误） | ✅ 在场 | closure 报告文末追加勘误节（31→28 行，O-2 口径，不回改原文） |
| INV-02（LED-08 口径注明） | ✅ 在场 | research 4.1 节行内注明 + §6 后块注记（五→六写路径，六路径逐一点名） |
| INV-03（LED-06 台账刷新） | ✅ 在场 | 台账第十六节 LED-06 确认闭合销账行（tdd L107 证据指向） |
| INV-04（workflow 过期修订） | ✅ 在场 | workflow 第五节修订节（§三统计刷新 + §四 LED-09~12 逐项 + 基线 451 分布） |
| INV-05（分支清理） | ✅ 完成 | 本地与 origin 双侧 cli-grammar-v0.6 已删（ls-remote 实测） |
| INV-06（_verify_tmp40） | ✅ 如实登记 | 现场 Test-Path=False，与「声明与现场已相符」登记一致 |
| INV-07（_wip_stage 删除 + wip 保全前置核验） | ✅ 完成 | 目录不存在；wip 分支 9d63d3b 本地/远端双侧保全 |
| INV-08/09（裁定保留） | ✅ 在场 | 台账第十六节两条裁定登记；现场 `_fix/`、`_master_lock.rs`/`_wip_lock.rs` 仍在 |
| 四报告 + 执行日志入库 | ✅ 在场 | 04024a8 含四份审计报告与日志一至六节；8c10387/fe1899c 依次入库 |
| 台账第十六节零开放终态 | ⚠️ 主体在场，B-x 面有缺口 | LED-06/07/08 销账表 + LED-18~23 + INV-08/09 均在位；但 B-3/B-4/B-7 未登记（见 Warnings 第 1 条） |
| S2-02/S2-03/S2-04 文档债 | ⚠️ 处置在场，S2-02 证据引用有误 | S2-03 双处裁决指针、S2-04 双 role 归档声明逐字在场；S2-02 勾选在场但提交链引用错误（见 Warnings 第 2 条） |
| docs gate 修复（86776db） | ✅ 在场 | error.rs 文档注释已改纯文本表述；强制重 rustdoc 实测 exit 0 |
| 全仓 intra-doc link 扫查声称 | ✅ 与现场相符 | thread_scan.rs 模块头恰 5 处 `[`…`]` 链接（L5/L6/L9×2/L13）指向 pub(super) 项，私有模块内文档；docs gate -D warnings 实测零告警 |
| FR-2 四件套登记 | ✅ 在场（单处） | 执行日志第七节 FR-2 全文明确；台账/ workflow 联动缺失见 Suggestions |
| 451 全绿 / 门禁声称 | ✅ 独立复现 | 本评审现场复跑 451 全绿，分布逐位一致 |

## 维度结论

**修复波 F1~F4 与修复轮二的全部已承诺项均已落地，且经独立复跑实证**：R2-01 修复与迁移点完整、六盲区测试与 B-8 裁定注记齐备、INV-01~09 处置全部兑现（含分支/目录清理实测与保留裁定登记）、S2-01~04 处置在场、docs gate 修复经强制重文档化复验为绿、451 全绿独立复现。未发现虚假销账，未发现声称与 diff 主体不符。

存在两项重要级完整性缺口：**B-3/B-4/B-7 三条审计盲区既未处置也未登记，削弱台账「零开放终态」声明的成立范围**；**S2-02 销账证据将任务 #52 回填提交 f94b65f 误引为任务 #36 裁决批链端，与台账第十四节 SSOT 及审计出处自相矛盾**。另有两项低危/建议级：轮二闭环缺独立复验落点与台账联动、FR-2 未并入 workflow-and-todo 门禁清单。

**判定：有条件通过**——补齐上述两项 Warnings 后，完整性维度即可完全放行；两项 Suggestions 不阻塞放行。

（评审完。撰写：三维评审完整性维度评审员；2026-08-15。只读取证，除本报告外零落盘，未触碰源代码与 git 状态。）
