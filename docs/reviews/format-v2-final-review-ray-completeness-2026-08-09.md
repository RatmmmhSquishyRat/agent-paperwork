# format-v2 终审评审 — 视角 A：完整性（Ray）

- 日期：2026-08-09
- 评审对象：master(a7ea07c) → 工作树全量 diff（`_review_tmp_full.diff`，282KB，2026-08-09 11:45 生成）
- 范围界定：format-v2（Managed 格式全面重设计 v0.5）+ owner 追裁 D1-D3 的落地交付。v0.6 CLI 文法（--author/--message 具名化）未实现，不在本轮范围。
- 基线：cargo test --workspace 全绿（core format 32 + core ops 75 + CLI 集成 47 = 154 测试）；clippy -D warnings 零告警。

## 1. 完整性核查清单

| 维度 | 核查项 | 结论 |
|---|---|---|
| 规格文档 | format-v2 五文档（spec/bdd/tdd/impl_plan/…）是否联动 D1-D3 | ✅ 11:11-11:17 已更新；spec §5.4 派生算法、§5.7 preamble 首写与 --to/--participants 删除、§5.9 序列化、§9.2 文案表、§11 OQ-1/OQ-4 齐全 |
| BDD | bdd.md 75 个场景是否覆盖 D 系列 | ✅ 场景与实现一致；但缺"文件末尾无换行时追加"的场景（见 R1） |
| TDD | tdd.md T-FT/T-OPS/T-CLI/T-CI 条目 | ✅ 均联动 D1-D3 |
| core format | 四格式解析/序列化（thread/profile/brief/contacts） | ✅ fence 辅助函数、header_seq 溢出归 preamble、derive_reply_to/derive_mentions（排除 reply 形态 token 与 sender 自提及）按 §5.4 实现 |
| core ops | thread_send/thread_edit/read_last_seq_locked | ✅ 首写门（锁内 file size）、legacy 写守卫、64KB 前置校验、fence 感知尾扫、R7 丢弃首行均在位；唯追加路径缺尾换行守卫（见 R1 关联 K1） |
| CLI 层 | post/validate/brief/contacts 命令面 | ✅ --to/--participants 已删；--reply-to/--mention 糖衣按 OQ-4 注入正文 token；validate 接入 seq 连续性 + 围栏闭合 |
| 测试 | 154 测试是否覆盖 D 系列与回归 | ✅ `post_send_to_and_participants_flags_removed`、`post_send_mention_injects_body_tokens`、`post_send_reply_to_injects_body_tokens`、`post_send_reply_token_dedup` 等齐备；盲区见 R1 |
| 语料 | test-v05/ 新格式语料 | ✅ 存在且被 CI smoke 引用 |
| CI | ci.yml smoke 段 | ✅ 新文法，无 --to/--participants 残留 |
| README | 三个 README | ✅ 无旧构造残留 |
| CHANGELOG | 0.5.0 段落 | ✅ D 系列行为、迁移指南、Rust API breaking、行为契约变化完整披露 |
| 版本 | Cargo.toml | ✅ 0.5.0，符合"追裁并入 0.5.0 不 bump"裁决 |

## 2. Findings

### R1（BLOCKING，与正确性视角 K1 同源）：追加路径完整性缺口 — 无尾换行文件的场景与守卫双缺

- 位置：`paperwork-core/src/ops/thread.rs` `thread_send`（非空文件追加分支）；`bdd.md` / `ops_tests.rs` / `cli_integration.rs` 均无对应场景。
- 事实：`serialize_message` 输出以 `## #N` 开头且不含前置换行；追加分支直接 `write_all(serialized)`，不检查文件最后一字节是否为 `\n`。手工编辑（或任何外部工具截断尾换行）后的文件再 `post send`，新消息头会黏在前一行（如闭合围栏行）上，新消息被静默吞入前一条 body。
- 完整性定性：BDD/TDD/测试三层对"尾换行缺失"这一外部可触发输入态零覆盖，属场景完整性缺口；功能缺陷本体由 Kim 视角 K1 记录，合并报告按单一 finding 去重。

### R2（PASS）：其余文档-实现-测试三角闭合

规格、实现、测试、语料、CI、README 六面一致，未发现其它遗漏面。

## 3. 结论

交付完整性整体达标；唯一阻断项为 R1/K1 的追加路径尾换行守卫及其配套场景缺失。要求：修复 K1 的同时在 BDD（新增场景）、core ops 测试、CLI 集成测试三层补齐"无尾换行追加"用例，方可闭合。

> 闭合状态（2026-08-09）：R1 已按上述要求闭合（BDD POST-36 + T-OPS-31/T-CLI-25 + 3 个回归测试 + 手工冒烟），详见合并报告闭合记录表（`format-v2-final-review-merged-2026-08-09.md`）。
