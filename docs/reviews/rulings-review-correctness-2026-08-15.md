# 三维评审 · 正确性维度报告（2026-08-15 owner 裁决实施 O1~O5）

- 评审视角：**正确性（逻辑与安全 bug）** —— 需求覆盖与回归影响面由另两位评审员负责，本报告不越界。
- 评审范围：`git diff d920271..HEAD`（基线 d920271 → b9b059c，提交 9821933/14f3b57/77f19e2/6a36639/72c85ac/b9b059c）。
- 方法：diff + 源码走读（post.rs / contacts.rs / main.rs / core 依赖面）+ `cargo test --release` 全量（core 71 绿、CLI 191 绿）+ 临时探针（`_review_probe`，已清理，未触碰源代码与 git 状态）。

## 维度结论

**正确性维度通过：未发现 Critical 或 Warning 级问题。** 撤销彻底、advisory 探测三级判定正确且无锁放大、正文直书路径与读侧 derive 完全自洽、测试断言有效。仅存 4 项 Suggestion 级备注（见下）。

### 逐项核查结果（对照评审任务重点）

1. **撤销彻底性 —— 通过**
   - 注入管道全链删除且无残留引用：`clean_list` / `validate_mention_value` / `inject_reference_tokens` 三个助手及 `--reply-to 0` validation 分支、mention 清洗/校验/dedup/注入调用全部移除，全仓 grep（含测试）零残留。
   - Send clap 签名只剩 path/--author/--message/--stdin/--title；`after_help` 已改为正文直书教学（post.rs L52）；Edit 本无此二 flag，外延拒绝由 `usage_fix` 覆盖（main.rs L244-251，`post.send | post.edit` 双臂，探针实测 edit 亦落教学面 exit 2）。
   - `VALUE_TAKING_FLAGS` 保留 `--reply-to`/`--mention` 是**正确**的：post read 侧仍是带值 flag，`--json` 探针的值跳过逻辑依赖其在列（spec 4/10 显式声明）。
   - 隐式行为无残留注入：探针验证 `--message "@#1 reply"` 落盘字节逐字（无 `@alice` 注入），`implicit-mention` 仅为输出字段（post.rs L206-216, L232-234），测试 `implicit_mention_persisted_to_file` 有负断言防线。
   - 探针实测 `--reply-to=2` 等号形态同样命中教学臂（clap 报错剥离值），无逃逸面。

2. **advisory 探测正确性 —— 通过**（contacts.rs L236-252）
   - 三级判定顺序与短路：exists -> read_to_string -> parse_profile，逐级短路，与冻结文案三形态一一对应。
   - **锁外探测**：`locked_read_modify_write`（core ops/lock.rs）在函数内完成加/解锁后才返回，CLI 层探测发生在写完成之后；contacts 锁未因探测延长，探测目标（profile 文件）本就与写锁对象不同。写侧 `derive_label` 亦在锁外预读（core ops/contacts.rs L87-89）。无「持锁放大 I/O 窗口」。
   - **探测失败不 panic、不改退出码**：三个失败分支全部映射为 advisory 文本，`emit_ok` + `Ok(())` 路径不变；探针实测目录、非 UTF-8 二进制、含引号/百分号路径均 exit 0 无 panic。
   - 路径解析与读侧一致：复用 `resolve_contact_path`（as-given CWD 优先 -> contacts 目录相对），探针验证「文件仅存在于 contacts 目录」时无误报；update 探测对象为 **NEW** profile（contacts.rs L156），OLD 不再探测 —— 与「re-bind 目的地」语义一致，探针确认。
   - 文案注入风险：`format!` 将 destination 作为参数而非格式串（`%`/花括号无意义）；默认档逐字回显（与既有 conclusion/profile 字段同权），`--json` 档经 serde_json 正确转义（探针：含引号路径输出合法 JSON）。`check_single_line` 在 core 写入前已禁换行，advisory 单行性不受破坏。

3. **正文直书路径（derive 边界）—— 通过**（post.rs L206-216 + core format/thread.rs）
   - implicit-mention 改由正文 `@#N` 驱动：`derive_reply_to(&body)` -> `find_message_sender` 有界尾扫 -> 与 `derive_mentions(&body, &author)` 比对；自回复/已显式 mention/seq 缺失三边界与原 v0.5 语义逐条等价（S-SEND-10b/11 测试改写后仍冻结该行为）。
   - 探针实测边界：`@#0` -> 静默（seq 0 永非合法头，尾扫返回 None），exit 0；负数不适用（正则 `@#(\d+)`）；多个 `@#N` 取第一个，与读侧 parse 时 derive 完全一致（写侧报告与读侧字段无漂移）；全角 `＠` 不匹配（ASCII `@` only），宽松跳过；重复 `@bob @bob` 写侧逐字、读侧 dedup；u64 溢出数字串 parse 失败被 filter_map 宽松跳过。

4. **并发时序 —— 通过（声明在案）**
   - advisory 探测在写锁释放之后运行：写-探测窗口内并发修改目的地文件可使探测读到中间态并报对应 advisory —— 该字段按裁决即**咨询性、非阻塞**，不影响退出码与写入结果，可接受。代码注释（contacts.rs L119-121）已声明 non-blocking 语义。
   - implicit-mention 查询（`find_message_sender`）在 `thread_send` 之前自持锁读、写时另持锁 —— 与撤销前 flag 驱动路径完全相同的时序，非本 diff 新引入面；seq 查询为锚定查询，并发 append 不改变既有 seq，缺失静默。

5. **新测试断言质量 —— 通过（一条 Suggestion）**
   - 金快照重冻与行为一一对应（`contacts_add_second_json_stdout` 增 advisory、`post_send_*_file` 去注入形态），撤销面测试带「无文件写入」负断言、flag 清单测试带 `!contains` 负断言、usage/纯 ASCII 双档覆盖。无假阳性发现；一条弱断言备注见 S-4。

## Critical Issues (MUST FIX)

无。

## Warnings (SHOULD FIX)

无。

## Suggestions (CONSIDER)

### S-1 advisory 冻结文案的 pure ASCII 声明对非 ASCII 目的地不成立
位置：repos/paperwork-cli/src/cmd/contacts.rs L233-L252（destination_advisory 文案与 doc-comment）
- 问题：doc-comment 与 CHANGELOG 均称 advisory 为 single-line pure ASCII，但插值的是用户给定的 destination 原文。探针实测中文文件名目的地时，advisory 行含非 ASCII 字节——模板 ASCII，插值非 ASCII。非功能性缺陷（同一字符串已在 conclusion/profile 字段原样出现，非 advisory 新引入），但与冻结文案字面矛盾，且 O3 的纯 ASCII 字节级测试只用 ASCII 文件名，覆盖不到该形态。
- 修复：把文案声明收窄为「模板为纯 ASCII；destination 按给定原样回显（与 conclusion/profile 字段同权）」，或补一条非 ASCII 目的地形态的冻结声明（属文案/测试口径修正，不改行为）。

### S-2 非 UTF-8 目的地归入 is not readable 而非 is not a valid profile file，建议注释显式记录
位置：repos/paperwork-cli/src/cmd/contacts.rs L241-L244
- 问题：read_to_string 对非 UTF-8 文件返回 InvalidData，落入第二级。这与三级探测顺序自洽：可读即定义为可解码为字符串。探针验证行为良性、无误报，但该分类点未被文档化，后续维护可能误改。
- 修复：在 destination_advisory doc-comment 注明 non-UTF-8 内容按探测顺序归入第二级。

### S-3 advisory 探测对目的地做第二次全量读
位置：repos/paperwork-cli/src/cmd/contacts.rs L236-L252
- 问题：写侧 derive_label 已在锁外完整读过一次目的地（core ops/contacts.rs L295），advisory 再全量读一次并整体 parse。profile 文件通常很小，无正确性问题；仅当目的地异常巨大时多一次无上限读取。
- 修复：可接受现状（advisory 独立性换来实现简单）；若要收敛，可让 core 在 derive_label 时顺带返回探测三态供 CLI 复用，收益有限，不建议本轮动。

### S-4 post_send_reply_token_no_injection_dedup 的 dedup 断言已不可证伪
位置：repos/paperwork-cli/tests/cli_integration.rs L618-L673
- 问题：改写后输入体只含单个 @alice，content.matches 计数等于 1 的断言恒真成立——原测试「重复 mention 只注入一次」的性质随注入管道撤销而消解，断言名实不符（非假阳性，属弱断言）。
- 修复：改为双 token 输入（正文写 @alice 两次）断言落盘恰为两个 @alice（逐字 passthrough 的新冻结性质），或改名并去掉该计数断言。

### 备注（非 diff 引入，登记备查）

- @#0 旧为 --reply-to 0 validation 拒绝面，现随 flag 撤销消解：正文 @#0 静默写入且读侧 derive reply_to 为 0。此为宽松 derive 的既有语义（diff 前逐字正文同样可达该形态），非本轮新引入缺陷，tdd 改写表已显式登记该分支随 flag 撤销整体下线。

## 验证证据摘要

| 项 | 手段 | 结果 |
|---|---|---|
| 全量测试 | cargo test --release（workspace） | core 71 / CLI 191 全绿 |
| 注入残留 | git grep 三助手 + --reply-to 全 cmd 面 | 零残留，仅存 read 过滤器 |
| @#0 / 多 @#N / 全角 @ / 重复 mention | 探针（release 二进制） | 逐字写入、derive 一致、无 panic |
| 特殊字符目的地（引号/百分号/中文/二进制/目录） | 探针 | exit 0，JSON 转义正确，三级归类正确 |
| 相对路径二级解析 / update 探测 NEW | 探针 | 与读侧一致，OLD 不误报 |
| usage 教学臂（含 --flag=v 形态、edit 外延） | 探针 + 金快照测试 | exit 2 + 正文直书教学 |
