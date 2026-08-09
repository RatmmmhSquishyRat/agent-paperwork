# format-v2 终审评审 — 三视角合并报告（按严重度分组）

- 日期：2026-08-09
- 评审对象：master(a7ea07c) → 工作树全量 diff（format-v2 v0.5 + owner 追裁 D1-D3 落地；v0.6 文法未实现、不在范围）
- 输入：Ray（完整性）/ Kim（正确性）/ Paul（影响面）三份独立报告，同目录
  - `format-v2-final-review-ray-completeness-2026-08-09.md`
  - `format-v2-final-review-kim-correctness-2026-08-09.md`
  - `format-v2-final-review-paul-impact-2026-08-09.md`
- 去重规则：R1 与 K1 同源（场景缺口与功能缺陷本体），合并为单一 finding F1，双视角归因；其余 findings 无重叠。
- 评审基线：154 测试全绿、clippy 零告警（评审开始时）。

## 处置要求（owner 指令）

BLOCKING/MAJOR 回流修复并回归；MINOR 记录处置。

## BLOCKING

### F1（Kim K1 = Ray R1）：thread_send 追加不检查文件尾换行 → 消息黏连、静默数据丢失

- 位置：`paperwork-core/src/ops/thread.rs` `thread_send` 非空文件追加分支。
- 根因：`serialize_message` 输出以 `## #N` 开头、不含前置换行；追加分支假设文件必以 `\n` 结尾且不校验。外部编辑截断尾换行后即触发。
- 实测复现：无尾换行的合法线程（末行为闭合围栏）执行 `post send` 报 `ok seq: 2`，文件出现黏连行 ` ```## #2 bob (…) `，`post read` 仅剩 1 条消息——新消息被静默吞入前一条 body，无任何错误信封。
- 双视角归因：正确性——违反写后读一致不变量；完整性——BDD/TDD/测试三层对"无尾换行追加"输入态零覆盖（既有 154 测试均以本工具自产文件为输入，天然满足尾换行）。
- 修复要求：锁内追加前读文件最后一字节，非 `\n` 时 payload 前补 `\n`；同步补 BDD 场景、core ops 测试、CLI 集成测试三层回归。

## MAJOR

### F2（Paul P1）：v0.6_feedbacks.md §2.4 签名表残留已删除的 --participants/--to

- 位置：`docs/ssot/adr/feedbacks/v0.6_feedbacks.md` §2.4（post send 行）与 §2.3 规则 3（`--to` 在 post send 的语义描述）。
- 冲突：D1/D2 追裁已废除 participants 与 --to/--participants flag（CLI usage 错误 + 集成测试固化）；该文档晚于 D 系列文档更新，属漏改。
- 影响：v0.6 实现的 SSOT 输入，照表实现将逆转 owner 追裁。
- 修复要求：勘误 §2.4 post send 行与备注；修正 §2.3 规则 3 中 post send --to 的表述。

## MINOR（记录处置，本轮一并修复）

### F3（Kim K2）：post edit 的 body 错误示例为 send 形态

- 位置：`paperwork-cli/src/cmd/post.rs` `resolve_body`（send/edit 共用），example 硬编码 send 形态。
- 处置：`resolve_body` 增加命令形态参数，edit 传 edit 形态；补断言或测试确认文案。

### F4（Kim K3）：--mention 含 sender 本人时注入惰性 token —— 复核后作废（已被 MJ-2 闭合）

- 位置：`paperwork-cli/src/cmd/post.rs`。
- 复核结论：评审草稿基于旧认知；实际代码中 `validate_mention_value`（上轮 MJ-2 修复）已在 flag 层拒绝 `value == from`（"it mentions the sender itself"，Validation 错误信封且拒绝时不落盘），集成测试 `post_send_mention_rejects_malformed_values` 已固化。自提及根本无法写入，惰性 token 不成立，finding 作废，无需修复。
- 处置：记录作废；Kim 报告 K3 以本段为准。

### F5（Paul P2，记账）：Review Book 测试计数滞后（144 → 154+）

- 处置：本轮闭合时更新 `docs/reviews/v0.5-review-2026-08-09.md` 结论段、测试计数与本轮 MINOR 处置记录。

## 通过项汇总

- 完整性：format-v2 五文档联动 D 系列、75 BDD 场景、TDD 条目、四格式实现、CLI 命令面、语料、CI smoke、README 均闭合（除 F1 场景缺口）。
- 正确性：fence 感知尾扫、header_seq 溢出处理、derive_reply_to、thread_edit 三重护栏、legacy 写守卫、validate 深化均核查通过。
- 影响面：CHANGELOG 0.5.0（含迁移指南、Rust API breaking 六条、行为契约变化）、版本策略（并入 0.5.0 不 bump）、--json 形状披露、CI/README/语料均达标。

## 放行结论

**不放行**，直至 F1（BLOCKING）与 F2（MAJOR）修复并全量回归（cargo test --workspace + clippy -D warnings + F1 场景手工冒烟）通过；F3-F5 随本轮处置并记录。

## 闭合记录（2026-08-09，修复后回填）

| Finding | 处置 | 验证 |
|---|---|---|
| F1 BLOCKING | `ops/thread.rs` thread_send 锁内尾字节探测，非 `\n` 时 payload 前补 `\n`；BDD 新增 POST-36，tdd.md 新增 T-OPS-31/T-CLI-25，ops_tests +2、cli_integration +1 | 全量测试绿（161 测试）；原复现场景手工冒烟：新头独立成行、2 条消息读回完整 |
| F2 MAJOR | v0.6_feedbacks.md §2.3 规则 3 与 §2.4 post send 行勘误（删 --participants/--to，标注追裁废除来源与终审编号） | 文档 grep 无残留已删 flag 签名 |
| F3 MINOR | `resolve_body` 增 `BodyCommand` 判别，edit 错误 example 为 edit 形态；cli_integration +1 断言测试 | 测试 `post_edit_missing_body_example_shows_edit_form` 通过 |
| F4 MINOR | 复核作废：`validate_mention_value`（MJ-2）已在 flag 层拒绝自提及，无惰性 token 可写入 | 既有测试 `post_send_mention_rejects_malformed_values` 固化 |
| F5 MINOR | Review Book（v0.5-review-2026-08-09.md）补记本轮终审处置与测试计数 | 同文件追加段落 |

回归基线：cargo test --workspace 全绿（cli_integration 35 + core 单测 75 + ops_tests 51 = 161）；cargo clippy --workspace --all-targets -- -D warnings 零告警；BDD 79 场景。
