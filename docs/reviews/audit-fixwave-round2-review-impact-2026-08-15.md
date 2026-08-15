# 审计修复波 Round2 三维评审 · 影响面（回归与破坏性变更）（2026-08-15）

- 评审视角：impact（仅「影响面：新增副作用 / 既有契约回归」；需求覆盖与内部逻辑 bug 由另两位评审员负责）
- 评审范围：`git diff 46b1f47..HEAD`（fe1899c），提交链 54beff3（F1）→ 0ffd9d2（F2）→ 0dc23f0（F3）→ 04024a8（F4）→ 8c10387（复验报告）→ 86776db（docs gate 修复）→ fe1899c（登记）
- 评审方式：只读；逐 hunk diff 核对 + 本地四门禁实测 + 全量测试复跑 + gh 线上 CI 取证 + 全仓 grep 卫生核验。未改任何源代码，未做任何 git 写操作（本报告落盘除外，未提交）。

## 发现分级总览

| 级别 | 数量 | 编号 |
|---|---|---|
| Critical（必须修） | 0 | — |
| Warning（应修） | 1 | W-1 |
| Suggestion（可考虑） | 2 | S-1、S-2 |

## Critical Issues（必须修）

无。

## Warnings（应修）

### W-1 CHANGELOG [Unreleased] 未披露 R2-01 文件通道 fix 文案变更（下游可见漂移无披露项）
[CHANGELOG.md#L6-L66](c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\CHANGELOG.md)

- **Problem**：R2-01 使 12 处文件读通道（core 10：thread_read×3 / thread_edit / thread_scan×2 / lock / contacts / profile / manifest；CLI 2：post.rs reject_foreign_thread、validate.rs）在遇到非 UTF-8 文件时，`error io:` 信封的 fix 文案由「check file permissions / check that the file is readable / check file integrity」逐字变为「the file is not valid UTF-8; ...」。这是**对外可见的错误输出文案变更**，但 [Unreleased] 段（L6–66）无任何披露条目。本仓披露纪律有同构判例：D6 stdin 通道同类文案变更在 L64–66 有专门条目；SAM-5 评估报告明示「未发布轮的披露义务不消失」。依赖 CHANGELOG 感知行为漂移的下游 agent/消费者将看不到该变更。注意：category（io）、exit code（1）、信封结构均无变化，故非功能回归，仅为披露缺口。
- **Fix**：在 [Unreleased] 追加一条（参照 D6 条目措辞），要点：文件读通道在 InvalidData（二进制/UTF-16）失败时 fix 文案改为编码指向（引用常量 FILE_NOT_UTF8_FIX 逐字值）；category 仍 io、exit 仍 1、信封结构零变更；其余 io 失败文案不变。

## Suggestions（可考虑）

### S-1 复验报告「不放行」为该档案终态文本，闭合证据在另一文档，存在误读面
[docs/dev/audit-fixwave-round2-verification-2026-08-15.md#L19](c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\docs\dev\audit-fixwave-round2-verification-2026-08-15.md)

- **Problem**：复验报告以「最终结论：不放行」收尾并归档（8c10387）；其后的 docs gate 修复（86776db）、CI 全绿（run 31884670287）与放行事实仅登记在执行日志第七节（fe1899c 追加）。只读复验报告的后续 agent 会得出「修复波仍被阻塞」的过期结论。append-only 纪律本身无错，但闭合指针缺失。
- **Fix**：在复验报告文末追加一节闭合注记（不回改正文）：阻塞项已由 86776db 修复、HEAD fe1899c 的 run 31884670287 七 job 全绿、详执行日志第七节 FR-2。

### S-2 已删分支 cli-grammar-v0.6 在冻结 spec 文档中留有基线性提及（可加一行删除标注）
[docs/ssot/specs/cli-grammar-v0.6/impl_plan.md#L107](c:\Users\15480\Desktop\AIWorkshop\repos\agent-paperwork\docs\ssot\specs\cli-grammar-v0.6\impl_plan.md)

- **Problem**：impl_plan L107 与 tdd L206 的基线句提及「cli-grammar-v0.6 分支（worktree agent-paperwork-wt-v06grammar）」；该分支本地+远端均已删（执行日志 INV-05，`git branch -a` 实测无残留）。经 grep 核验：**无任何现行 workflow/脚本/教学面引用该分支名**，残留均为历史档案叙述（reviews/ledger/assessment），不构成运行期断链，无功能影响。仅当未来 agent 按基线句尝试 checkout 该分支时才会困惑。
- **Fix**：可在 spec README 治理清单或台账追加一行「cli-grammar-v0.6 分支已于 2026-08-15 删除（本地+origin），实施成果已全量合入 master」；不做亦可（现状已有 INV-05 登记承载）。

## 重点审查项逐条核验证据

### 1. 对外契约：R2-01 仅改 fix 文案 —— PASS

- error.rs diff 仅含三类新增：`pub const FILE_NOT_UTF8_FIX`（纯 ASCII）、`pub(crate) fn io_ctx_file_read`、单测 `test_io_ctx_file_read_encoding_hint`。**七类 category 映射行（error.rs L135–143 `category()` match 臂）零改动**；`IoContext => "io"` 在位。
- 退出码契约未动：`emit_err` JSON 档 `exit_code:1`（output.rs L123）、usage 档 `exit 2`（L159）、`process::exit(1)`（main.rs L156）均不在 diff 内。
- 信封结构零变化：R2-01 只替换 `IoContext.fix` 字段字面量，且仅在 `e.kind() == InvalidData` 分支；其余 io 失败沿用原文案（post.rs L518–519、validate.rs 新 else 分支逐字保留旧文案）。
- 公有 API 面仅 additive：新增 1 个 pub 常量；`io_ctx_file_read` 为 pub(crate) 不扩对外面。spec §7「输出协议只增不改」在输出 key 面未被破坏；fix 文案变更属 R2-01 裁定范围内的一次受控漂移（已登记执行日志 F1 节与审计依据）——但披露缺口见 W-1。
- 实测佐证：新增测试 `utf16_file_read_fast_fails_with_encoding_pointing_fix` 同时断言 stderr `error io:` + 新文案逐字 + `--json` 档 `"category":"io"` + 夹具零写入，本地通过。

### 2. 冻结面 —— PASS

- ops_tests：`git diff 46b1f47..HEAD -- repos/paperwork-core/tests/` 为空；ops_tests.rs 恰 71 个 `#[test]`，本地全绿（0.09s）。
- 黄金快照：cli/core 两份 char_tests.rs、guard_tests、t6_cli、ivy_gap、ops_contacts_crud 全部零 diff（未重冻）；cli char_tests 33 测试 / 内嵌 gold 表本地全绿。
- core unit 102→103、cli_integration 148→154 为 F1/F2 纯新增，与执行日志口径一致。
- B-8 裁定注记三处一致：spec.md §3.3 注记、§3.7 注记、design.md §「本轮新增登记」第三条、bdd.md S-READ-10——四处（含 bdd）口径互洽：均声明 H1 在读侧/validate 侧非强制、钉住现行行为不改行为、指向同一测试名 `h1_leniency_missing_and_duplicate_h1_read_cleanly` 与探针 T-05/T-06。
- 全量复跑：`cargo test --workspace --locked` = **451 全绿**（7+33+154+16+4+103+12+33+18+71+0），分布与执行日志第五节逐位一致。

### 3. 分支与卫生动作影响 —— PASS（残留面均为历史档案叙述）

- cli-grammar-v0.6：`git branch -a` 仅存 master 与 wip/v0.5-perfection-snapshot-2026-08-15（本地+origin 双侧），本地与远端分支均已删；`.github/workflows`、publish.ps1、_e2e 脚本 grep 零引用；残留提及均在历史档案（impl_plan/tdd 基线句、reviews、ledger 裁定基线说明），见 S-2。删除动作本身有 INV-05 证据链（--merged + 51/0 零独有）。
- _wip_stage / _verify_tmp40：`Test-Path` 均为 False；全仓 grep 命中 10 处全部位于登记其删除/裁定的 dev 文档（执行日志、复验报告、inventory、ci-full-revalidation 历史句），无任何脚本引用，无运行期断链。
- 附带核验：`_e2e/` 未跟踪（`git ls-files _e2e` 为空；check-ignore 命中 `_*/`），S2-01 的 smoke.ps1 修复确为纯本地资产修正、未入仓——与执行日志声明一致，无 CI 面影响（CI smoke 为 ci.yml 内嵌双档，已实测全绿）。

### 4. 测试面 CI 时长影响 —— PASS（增量可忽略）

- 6 个新测试（B-1/B-2×2/B-5/B-6/B-8）本地合计 **1.28s**（Windows debug，含进程拉起）；cli_integration 全套 154 测试 4.59s。
- B-6（2500 消息）：文件约数百 KB，read/validate/send 共 5 次调用，秒级内；无内存/时长放大风险。
- B-5 的 `.timeout(30s)` 是防设备挂起的安全上限而非期望耗时；最坏情形有界（3 命令 × 30s × 3 OS），且本地/CI 实测均瞬时返回（后缀归一化确保不触碰 CON/NUL 设备）。
- 线上佐证：HEAD run（31884670287）总时长 3m3s，与 docs gate 修复前的绿色基线 run（31879040813，2m59s；31881210000，3m29s）同量级——451 基线在三平台（ubuntu/macos/windows）可预期通过且已通过（见第 6 条）。

### 5. 版本纪律与推送状态 —— PASS

- 两 crate Cargo.toml 均 `version = "0.5.0"`（diff 未触碰任何 Cargo.toml）；最新 tag 仍 `v0.5.0`；CHANGELOG 顶部仍为 [Unreleased] + [0.5.0]，无新发布段。未 bump、未 tag、未 publish——与「本轮不发布」裁定一致。
- origin 同步：`git rev-list --left-right --count master...origin/master` = **0/0**，完全同步。

### 6. 线上 CI —— PASS（docs gate 修复后全绿）

- `gh run list`：HEAD fe1899c 的 run **31884670287 = success**，7 个 job 逐一分解全 success（fmt、test×ubuntu/macos/windows、smoke×ubuntu/macos/windows），含独立 Docs gate（RUSTDOCFLAGS=-D warnings）。
- 其前两次 failure（04024a8 run 31883911085、8c10387 run 31884472866）根因为 R2-01 常量文档注释的 private intra-doc link，已由 86776db 修复并全绿实证；本地 `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` 亦 exit 0 复验。
- 本地四门禁（test 451 / clippy -D warnings / fmt --check / docs gate）于 HEAD 全部复跑通过。

### 7. 禁区核验 —— PASS（零触碰）

- wip/v0.5-perfection-snapshot-2026-08-15：本地与 origin 双侧均指向 **9d63d3b**（与执行日志登记的保全时点逐字一致）；分支 reflog 末条为 2026-08-15 03:48（本修复波开始之前），期间无任何移动。
- worktree：`git worktree list` 仅主工作区；agent-paperwork-wt-v06grammar 目录不存在（Test-Path=False）；wt-v05perfection 无触碰痕迹。

## 维度结论

**影响面维度：放行（1 Warning + 2 Suggestion，无 Critical、无功能回归）。**

本修复波对外契约面控制良好：R2-01 的漂移被严格限制在 io 信封 fix 文案的 InvalidData 分支，category/exit code/信封结构/JSON key 面零变化并经 451 全绿与线上三平台 CI 双重实证；冻结面（ops_tests 71、黄金快照、spec 冻结声明）经 git diff 与实测双重核验零越界；分支/目录卫生动作无运行期断链；版本纪律与推送状态完整；禁区零触碰。唯一应修项为 R2-01 文案变更在 CHANGELOG [Unreleased] 的披露缺口（W-1，非功能回归，可单文档提交闭合）。

（评审完。影响面评审员，2026-08-15。只读评审，未改源代码，未 git 提交。）
