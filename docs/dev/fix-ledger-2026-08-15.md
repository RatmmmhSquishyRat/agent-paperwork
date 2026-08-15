# 修复波销账台账（fix ledger）— 2026-08-15

- 日期：2026-08-15
- 任务：#27 修复波——闭环本轮全部审计发现
- 权威输入：docs/dev/audit-robustness-2026-08-15.md（深审 A/B：D1–D7 缺陷清单）、docs/dev/audit-ssot-agentux-2026-08-15.md（深审 C：A-01/A-02/S-01）、docs/dev/io-encoding-rootcause-2026-08-15.md（任务 #25 根因裁定）
- 取证基线：master @ a81d9ad（修复波起点）→ master @ 9884d89（修复与文档终点）；本台账自身落盘于随后的 docs 提交（本地未推送）
- 纪律核验：每组缺陷原子提交（fix 前缀+编号）；每缺陷含正向+负向回归；全程 `cargo test --workspace --locked` 全绿 + `cargo clippy --workspace --all-targets -- -D warnings` 零警告 + `cargo fmt --all --check` 通过；未 bump 版本、未打 tag、未动 CHANGELOG 发布段、未推送

---

## 一、逐项销账表

终态三分法：修复（代码或文档实质变更）/ 登记（不改，落台账）/ 钉住（行为锁定测试，含理由）。

### D1 带换行单行语义字段注入（阻塞级）— 已闭环（登记）
- 终态：登记——缺陷已由 v0.5-perfection-plan NEW-1 写侧护栏批（669befa，Felix 移植）先行闭环，本轮无新增代码动作
- 实测核验（证据链）：修复波复现脚本对三种形态（author 带换行 / scope 带换行 / body 首行伪装属性行）实测均 exit 1 validation 拒绝，修复前基线即已拒绝，前后一致
- 测试证据：guard_tests.rs 既有 NEW-1 护栏套件（check_single_line / contains_dangerous_attribute_line / first_line_representation_issue 对应测试）持续全绿
- 提交哈希：无新提交（历史闭环：669befa）

### D2 未闭合 fence 导致 thread send 吞消息 / edit 抹除（阻塞级）— 修复
- 终态：修复——锁内 fence 平衡预检，fast-fail 零写入
- 修复内容：
  - core：thread_scan.rs 新增 `unclosed_fence_issues_locked`（锁内 seek(0)+read_to_string+normalize+validate_markdown）
  - thread_send：legacy guard 之后、写入之前做锁内 fence 预检；thread_edit：parse_messages 之前对读到的 content 做 `validate_markdown` 检查
  - 错误信封：Parse category，message = `unclosed code fence (N backticks) opened at line X`，fix 指引闭合 fence 并声明「文件未被改动」
- 证据链（修复前→修复后）：修复前 send 含未闭合 fence 的 body → exit 0，消息被吸入 fence（read 可见内容消失）；修复后 → exit 1 `error format: Parse error: unclosed code fence (3 backticks) opened at line 5`，目标文件零写入
- 测试证据：guard_tests.rs 新增 D2 回归（send/edit 两侧 fast-fail）；cli_integration.rs 新增端到端 fence 预检测试（正向：闭合 fence 正常收发；负向：未闭合 fence exit 1）
- 提交哈希：8abdec6

### D3 `=` 粘连旁路注入——preamble prose 吞模型行（`## Scope` 前粘连）— 修复
- 终态：修复——净化基建统一：prose 表征检查新增标题行拒绝
- 修复内容：format/mod.rs 新增 `contains_heading_line`，并接入 `prose_representation_issue`（位于 first-line 检查之后、dangerous-attribute 检查之前）；理由文案声明 preamble prose 以裸文本序列化于结构标题之前，嵌入标题会在下次解析时截断或伪造结构。检查刻意非 fence-aware，镜像解析器对 preamble 的行为
- 证据链：修复前 create 时 description 尾部粘连 `## Scope` → exit 0 写入，随后 show 解析失败（missing - model:）；修复后 → exit 1 validation 拒绝，文件不存在（零写入）
- 负向回归：`- model: evil` 注入形态确认已由 NEW-1 dangerous-attribute 检查覆盖（D3b，历史闭环）
- 测试证据：guard_tests.rs 新增 D3 回归（标题行 prose 拒绝 + 普通 prose 放行）；cli_integration.rs 新增端到端零写入测试
- 提交哈希：2c7a180（D3+D4 合并提交，同源净化基建）

### D4 scope glob 换行伪造条目（`=` 粘连旁路第二分支）— 修复
- 终态：修复——scope glob 三列表（read/write/owns）逐项单行校验
- 修复内容：ops/profile.rs 新增 `check_scope_globs`（对每个 glob 调 `check_single_line`），create_profile_full 与 edit_profile（锁前）均接入；CLI 侧 profile create 改走 `create_profile_full` 单次原子写，消除 create-then-edit 两步法在校验失败时留下半成品文件的问题
- 证据链：修复前 scope 值带换行注入 `/etc/**` → exit 0 且 show 显示注入生效（scope.write: /etc/**）；修复后 → exit 1 validation 拒绝 + 零写入（文件不存在）
- 测试证据：guard_tests.rs 新增 D4 回归；cli_integration.rs 新增 profile create 零写入端到端测试
- 提交哈希：2c7a180（与 D3 同源合并）

### D5 ASCII 契约收窄（中）— 钉住 + 登记
- 终态：钉住（代码面）+ 登记（io 豁免面）
- 修复内容（评估结论）：文档侧已将「全部输出纯 ASCII」收窄为信封结构面（status token / command id / 字段名 / code / exit_code）纯 ASCII，用户数据值可含合法 UTF-8。代码面核查：output.rs 字面量、全部固定信封文案、category 名称均为 ASCII，既有 ascii_output_contract_guard / help / contacts 信封守护与新口径无冲突——无需代码变更
- 钉住：cli_integration.rs 新增 `ascii_contract_is_structural_surface_only` 边界测试——非 ASCII author（e-acute）经 post send/read 在值位置正常回显，结论面与字段名面全部结构 token 保持 ASCII
- 登记：io 信封嵌入 OS 本地化消息文本按任务 #25 根因报告裁定保留（环境面非产品面），登记点见 open-items-ledger LED-16
- 证据链：复现脚本非 ASCII 作者探针 exit 0 正常回显，结构面无非 ASCII 字节
- 提交哈希：3be44dc

### D6 非 UTF-8 stdin 的 fix 文案指向编码（轻微）— 修复
- 终态：修复——fix 文案从文件权限改指编码问题，并升级为 validation 信封
- 修复内容：cmd/post.rs resolve_body 区分错误源：InvalidData → Validation 信封（message `stdin is not valid UTF-8`，fix 指引重新编码或改用 --message，example 按 send/edit 场景分叉）；其他 io 错误 → IoContext（path `<stdin>`）
- 证据链：修复前非 UTF-8 stdin → fix 为「check the file path and permissions」（误导）；修复后 → validation 信封，fix 直指编码
- 测试证据：cli_integration.rs 新增 write_stdin 非 UTF-8 字节序列测试（exit 1 + 文案断言）
- 提交哈希：b107771

### D7 spec §3.1 --author「可含空格」与实现不一致（轻微）— 修复（文档）
- 终态：修复——以实现为准收口文档
- 修复内容：spec.md §3.1 --author 行改写为单 token 校验描述（拒绝空格/制表符/换行与括号，违规 validation exit 1），并交叉引用本台账；实现依据 = format/thread.rs `validate_sender`
- 证据链：grep 确认 spec 集内无「可含空格」残留（其余命中均为 contacts 路径场景，语义不同，不属于本项）
- 提交哈希：9884d89

### A-01 bdd S-READ-06 空线程 showing 计数错误 — 修复（文档）
- 终态：修复——`showing: 0/4` 更正为 `showing: 0/0`；tdd.md 同数值同步修正（SSOT 单口径）
- 提交哈希：9884d89

### A-02 bdd S-VAL-04 validate 示例文件名形态错误 — 修复（文档）
- 终态：修复——`myfile.post.md` 更正为 `myfile`，与文档化用法形态一致
- 提交哈希：9884d89

### S-01 crates.io 0.5.0 与仓库版本语义错配 — 登记（不改）
- 终态：登记——发布轮事项，修复波纪律不在本轮处置；登记点见 open-items-ledger 第九节 LED-15（含发布轮一次性闭合建议：bump 0.6.0 + CHANGELOG + crates.io + tag）
- 提交哈希：9884d89（随文档组提交，台账追加部分）

---

## 二、提交清单（修复波范围，master 本地）

| 提交 | 主题 | 覆盖缺陷 |
|---|---|---|
| a81d9ad | docs: audits, ledger, SSOT audit fixes and perfection logs | 第 0 步文档收拢（16 份 .md） |
| 2c7a180 | fix(D3+D4): reject heading-shaped lines in preamble prose and single-line-guard scope globs | D3、D4 |
| 8abdec6 | fix(D2): fence-balance precheck on thread send/edit — fast-fail instead of silent data loss | D2 |
| b107771 | fix(D6): non-UTF-8 stdin fix wording points at the encoding, not file permissions | D6 |
| 3be44dc | fix(D5): pin the narrowed ASCII contract — structural surface only | D5（钉住） |
| 9884d89 | fix(D7+A-01+A-02): doc-side closures — spec/bdd/tdd alignment + ledger registration | D7、A-01、A-02、S-01 登记、LED-16 登记 |

D1 无新提交（历史闭环 669befa，NEW-1 护栏批）。

---

## 三、行为变更清单（面向评审与发布披露）

1. **preamble prose 拒绝标题形态行**：description 等 prose 字段若含 `#`/`##`/`###` 起首行，写入前 validation 拒绝（防止裸序列化 prose 在下次解析时截断/伪造结构）
2. **scope glob 单行校验**：profile create/edit 的 scope read/write/owns 每个 glob 拒绝换行与多行粘连
3. **thread send/edit fence 平衡预检**：目标线程文件存在未闭合 code fence 时 fast-fail（Parse 信封，零写入），替代旧行为的静默吞消息/抹除
4. **stdin 非 UTF-8 信封**：fix 文案从文件权限改指编码，category 由 io 语义升级为 validation（exit 1 不变）
5. **profile create 原子化**：create 改走单次原子写（create_profile_full），校验失败不再留下半成品 profile 文件
6. **输出协议口径**：无信封结构变更——全部为拒绝路径收紧与文案修正，符合「只增不改」纪律；ASCII 契约正式口径 = 信封结构面纯 ASCII（用户数据值可含合法 UTF-8，io OS 消息按根因裁定豁免）

---

## 四、最终验证结果

- `cargo test --workspace --locked`：**410 通过 / 0 失败**（core 228：lib 97 + char 12 + guard 30 + contacts 18 + ops 71；cli 182：unit 6 + char 31 + cli_integration 141 + t6 4；doc-tests 0）
- `cargo clippy --workspace --all-targets --locked -- -D warnings`：零警告
- `cargo fmt --all --check`：通过
- 复现证据链脚本（_fix/repro-audit.ps1）：D1×3 / D2 / D3 / D3b / D4 / D6 / D5 全部探针呈修复后预期形态（拒绝或结构面 ASCII 保持）

## 五、销账统计

- 清单总数：11 项（D1–D7 + A-01/A-02 + S-01）
- 修复：8（D2、D3、D4、D6 代码；D7、A-01、A-02 文档；D1 经核验确认已被历史批次修复）
- 钉住：1（D5 代码面，含边界测试）
- 登记：2（S-01 → LED-15；D5-io → LED-16）
- 悬置：0——全部条目落入「修复/钉住/登记」三种终态之一

（台账完。撰写：任务 #27 修复波执行 agent；取证时间 2026-08-15；全部结论基于复现脚本实测与测试输出。）
