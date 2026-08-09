# format-v2 终审评审 — 视角 B：正确性（Kim）

- 日期：2026-08-09
- 评审对象：master(a7ea07c) → 工作树全量 diff（format-v2 v0.5 + owner 追裁 D1-D3 落地）
- 方法：逐路径代码审读（ops/thread.rs、format/thread.rs、format/mod.rs、cmd/post.rs、cmd/validate.rs）+ 编译产物实测冒烟（`$env:TEMP\pw-review\` 三组用例）
- 基线：154 测试全绿、clippy 零告警（评审开始时）

## 1. Findings

### K1（BLOCKING）：thread_send 追加不检查文件尾换行 → 消息黏连、静默数据丢失

- 位置：`paperwork-core/src/ops/thread.rs` `thread_send`，非空文件追加分支（payload = serialized 直写）。
- 根因：`serialize_message` 输出以 `## #N sender (ts)` 开头，不含前置换行。追加路径假设"文件必然以 `\n` 结尾"，但该不变量并非系统强制——本工具的序列化确实总是输出尾换行，可文件可被外部编辑器/管道截断尾换行。一旦不成立，新消息头与上一行（通常是闭合围栏 ` ``` `）拼成一行。
- 复现（实测）：
  1. 构造 `t3.post.md`：`# T3\n\n## #1 alice (2026-08-09T03:50:00Z)\n\n` + 围栏 ```` ```md\nfirst\n``` ````（无尾换行）。
  2. 执行 `paperwork post send t3.post.md --from bob "second"`，命令报 `ok post.send` / `seq: 2`（无任何错误信号）。
  3. 文件出现黏连行 ` ```## #2 bob (2026-08-09T03:53:25Z) `；`post read` 只返回 1 条消息，bob 的整条消息（头+围栏+body）被吞进 alice #1 的 body。
- 定性：静默数据丢失 + 静默格式损坏，无任何错误信封；违反"写后读一致"核心不变量。154 个既有测试全部以本工具自身产出文件为输入，天然满足尾换行，故零检出。
- 修复建议：锁内追加前读文件最后一字节，非 `\n` 时在 payload 前补 `\n`（O(1) seek+read 1 字节）。同时补 core ops 与 CLI 两层回归测试。

### K2（MINOR）：post edit 的 body 缺失错误展示 send 形态示例

- 位置：`paperwork-cli/src/cmd/post.rs` `resolve_body`（send/edit 共用），错误信封 `example` 硬编码 `paperwork post send thread.post.md --from alice ...`。
- 影响：edit 路径 body 缺失/读取失败时，agent 收到的 example 是 send 命令形态，按示例重试不会成功（一次重试自纠错契约被削弱）。行为本身正确，仅文案错配。
- 修复建议：`resolve_body` 增加命令名/示例形态参数，edit 分支传 edit 形态。

### K3（MINOR）：--mention 含 sender 本人时注入惰性 token

- 位置：`paperwork-cli/src/cmd/post.rs` send 分支 mention 注入（`clean_list(mention)` 不过滤 sender 本人）。
- 现象：`post send --from alice --mention alice,bob "hi"` 会在正文写入 `@alice @bob`，但读侧 `derive_mentions` 按 spec §5.4 排除 sender 自提及，`@alice` 落盘后永远不被派生为 mention —— 惰性 token，读出的 mentions 与写入 token 不一致，对 agent 反直觉。
- 定性：行为符合规格（写侧注入、读侧派生各司其职），规格未要求写侧过滤；属体验问题而非规格违规。隐式 reply @ 已有自回复跳过先例（`original.sender != from`）。
- 修复建议：CLI 注入时跳过 sender 本人（与隐式 reply 行为对齐），补测试；或提请 owner 裁决保留现状。

## 2. 正确性核查通过项

- fence 感知尾扫（`read_last_seq_locked`）：64KB+256B 尾缓冲、R6 长度规则、R7 丢弃首行、孤立 `\r` 处理均正确。
- `header_seq`：溢出/seq 0 归 preamble（前轮 review M1 修复有效）。
- `derive_reply_to`：取首个 `@#(\d+)`、不校验目标存在，符合 §5.4。
- `thread_edit`：preamble 经 `first_message_header_offset` 原样搬运，三重护栏、64KB 前置校验、崩溃窗口声明在位。
- `contains_legacy_headers` 写守卫：v0.4 线程追加被拒，错误信封含迁移指引。
- validate：空 post 文件拒绝、seq 连续性、围栏闭合均实测通过。

## 3. 结论

1 BLOCKING（K1，已实测复现）+ 2 MINOR（K2/K3）。K1 必须修复并补回归测试后方可放行；K2/K3 建议同轮顺手修复，处置记录进 Review Book。

> 闭合状态（2026-08-09）：K1 已修复并三层回归闭合；K2 已修复（`resolve_body` 按命令判别示例形态）；K3 复核作废——`validate_mention_value`（上轮 MJ-2）已在 flag 层拒绝自提及，惰性 token 前提不成立。详见合并报告闭合记录表（`format-v2-final-review-merged-2026-08-09.md`）。
