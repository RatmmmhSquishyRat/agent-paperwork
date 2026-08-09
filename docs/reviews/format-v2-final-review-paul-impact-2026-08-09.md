# format-v2 终审评审 — 视角 C：影响面（Paul）

- 日期：2026-08-09
- 评审对象：master(a7ea07c) → 工作树全量 diff（format-v2 v0.5 + owner 追裁 D1-D3 落地）
- 关注面：下游消费者影响（agent 调用方、IDE 插件、Rust 库使用者）、文档治理一致性、发布面（CHANGELOG/版本/CI）
- 基线：154 测试全绿、clippy 零告警（评审开始时）

## 1. Findings

### P1（MAJOR）：v0.6_feedbacks.md §2.4 签名表残留已删除的 --participants/--to（治理漂移）

- 位置：`docs/ssot/adr/feedbacks/v0.6_feedbacks.md` §2.4 post send 行：`<PATH> --author <NAME> (--message <BODY> | --stdin) [--reply-to N] [--mention a,b] [--title T] [--participants a,b] [--to a,b]`，备注称"`--title`/`--participants` 为 format-v2 加入的建线程载荷"。
- 冲突：D1 裁决 participants 废除（由 sender 集合派生）、D2/命令面删除 `--to`/`--participants`（CLI 实测传参即 usage 错误；集成测试 `post_send_to_and_participants_flags_removed` 固化）。该文档 mtime 11:43 晚于 D 系列文档更新（11:11-11:17），属追裁落地时的漏改。
- 影响面：该文是 v0.6 CLI 文法实现的 SSOT 输入。未来 v0.6 实现者照表实现会复活两个已裁决删除的 flag，直接逆转 owner 追裁，属高成本返工风险。
- 修复建议：§2.4 post send 行删除 `[--participants a,b] [--to a,b]`，备注改为"`--title` 为建线程载荷（send 自动建线程时生效）；participants/to 已于 D1/D2 裁决废除"；同步核查 §2.3 规则 3 中"`--to` 在 post send（收件人）"一句——post send 的 --to 已不存在，该句应改为仅描述 post read 的 seq 范围语义。

### P2（MINOR，记账类）：Review Book 测试计数与本轮处置未更新

- 位置：`docs/reviews/v0.5-review-2026-08-09.md`（144 测试时代的 Review Book）。
- 影响：治理账目滞后（144 → 154 + 本轮新增回归测试），后续评审轮次基线引用失真。
- 处置：本轮闭合时更新结论段与 MINOR 处置记录。

## 2. 影响面核查通过项

| 面 | 结论 |
|---|---|
| CHANGELOG 0.5.0 | ✅ D 系列行为、手工迁移指南四步、行为契约变化（空文件 validate、--to/--participants 删除、未迁移文件静默空读/写拒绝）、Rust API breaking 六条全部披露，无遗漏面 |
| 版本策略 | ✅ 未 bump（0.5.0），符合"D 系列并入 0.5.0"追裁；0.5.0 本身为 hard breaking，追加行为变化并入无额外冲击 |
| Rust API breaking | ✅ thread_send 签名、ThreadMeta.participants、Message.to、ContactEntry.label、serialize_thread 签名、六个删除的 pub fn 均已逐条列出，库消费者可编译期感知 |
| `post read --json` 形状变化 | ✅ reply_to/mentions 为读时派生、to 字段删除已在 CHANGELOG 披露 |
| CLI 命令面 | ✅ --to/--participants 删除为 usage 错误（exit 2），agent 一次重试可自纠；--reply-to/--mention 保留但语义变更已披露 |
| CI smoke | ✅ ci.yml smoke 段已用新文法，无已删 flag 残留 |
| 语料 | ✅ test-v05/ 替换旧语料，未迁移旧语料未遗留 |
| README ×3 | ✅ 无 v0.4 构造残留 |

## 3. 结论

发布面与下游披露整体达标；唯一 MAJOR 为 P1（SSOT 文档治理漂移，需勘误以防 v0.6 逆转裁决），另有 P2 记账项随本轮闭合处理。

> 闭合状态（2026-08-09）：P1 已勘误（v0.6_feedbacks.md §2.3 规则 3 与 §2.4 post send 行，标注追裁废除来源与终审编号）；P2 已在 Review Book §8 补记。详见合并报告闭合记录表（`format-v2-final-review-merged-2026-08-09.md`）。
