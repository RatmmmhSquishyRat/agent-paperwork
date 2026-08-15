# 修复波销账台账（fix ledger）— 2026-08-15

- 日期：2026-08-15
- 任务：#27 修复波——闭环本轮全部审计发现
- 权威输入：docs/dev/audit-robustness-2026-08-15.md（深审 B：D1–D7 缺陷清单）、docs/dev/audit-grammar-matrix-2026-08-15.md（深审 A：A-01/A-02）、docs/dev/audit-ssot-agentux-2026-08-15.md（深审 C：S-01）、docs/dev/io-encoding-rootcause-2026-08-15.md（任务 #25 根因裁定）（头部归属更正见第六节低-4 销账）
- 取证基线：master @ a81d9ad（修复波起点）→ master @ 9884d89（修复与文档终点）；本台账自身落盘于随后的 docs 提交（本地未推送）
- 纪律核验：每组缺陷原子提交（fix 前缀+编号）；每缺陷含正向+负向回归；全程 `cargo test --workspace --locked` 全绿 + `cargo clippy --workspace --all-targets -- -D warnings` 零警告 + `cargo fmt --all --check` 通过；未 bump 版本、未打 tag、未动 CHANGELOG 发布段、未推送

---

## 一、逐项销账表

终态三分法：修复（代码或文档实质变更）/ 登记（不改，落台账）/ 钉住（行为锁定测试，含理由）。

### D1 带换行单行语义字段注入（阻塞级）— 已闭环（登记）
- 终态：登记——缺陷已由 v0.5-perfection-plan NEW-1 写侧护栏批（669befa，Felix 移植）先行闭环，本轮无新增代码动作
- 实测核验（证据链）：修复波复现脚本对 title 带换行三形态（post/brief/contacts，即 D1 原始攻击向量 R-17/R-18）实测均 exit 1 validation 拒绝，修复前基线即已拒绝，前后一致（措辞更正见第六节低-1 销账：原句误写为 author/scope/body 形态）
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
- R-12 第二面显式裁决（评审闭环补记）：原报告第二面「带值 flag 空格形态拒收且无绕过引导」——注入面由本项单行校验闭合后，等号形态（`--scope-read=glob`）已成为安全 bypass，该面实质降为 UX 观察项；裁决：登记不处置，不列入修复项
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

---

## 六、评审闭环节（修复轮二，2026-08-15 追加，append-only，未改动第一至五节）

追加依据：三维独立评审三份报告（docs/reviews/audit-fixwave-review-{completeness,correctness,impact}-2026-08-15.md，评审基线 master @ da954c2）。取证基线补充：master @ da954c2（评审基线）→ master @ db3d023（修复轮二代码/CHANGELOG 终点）；本节与三份报告销账段落盘于随后的 docs 提交（本地未推送）。

### C-1/I-1 brief 写→读闭环断裂（阻塞，正确性 C-1 与影响面 I-1 两名评审员独立命中同一根因）— 修复
- 终态：修复——写侧补全 fence-aware 标题行护栏 + Entries title 拒绝；正确性报告备注面（note 含 `## x` 行静默分裂空 path entry）由同一护栏合并闭合
- 修复内容：
  - format/manifest.rs：`note_representation_issue` 在 M1 首行检查之上扩展两项——全文 fence-aware 标题行扫描（`note_contains_heading_outside_fence`，复用 `for_each_outside_fence`，拒绝 fence 外任何 `#` 起首行）与未闭合 fence 检查；同时修复读侧潜在缺陷：`parse_entry_body` 此前对 note 内非 regex 围栏的闭合行静默丢弃，导致围栏 note 无法 roundtrip——现闭合行保留（roundtrip 回归测试暴露）
  - ops/manifest.rs：`brief_add_entry` 锁前拒绝 title 为 `Entries` 的 entry（序列化为 `## Entries` 命中 SAM-1 读侧护栏）；title 派生移出锁外
  - 错误信封：validation，fix 指引声明标题形态行属于 code fence 内；拒绝即零写入
- 证据链（_fix/repro-c1.ps1 实测，修复前→修复后）：
  - 修复前：P1/P1b（note 含 fence 外 `### ` 行，非首行/首行）add exit 0 → 随后 brief read exit 1 `error format: Parse error: brief contains legacy v0.4 residue`，永久 lockout；P2（entry 文件名 Entries）同路径 lockout；P3（note 含 `## forged`）add exit 0 → read exit 0 但静默分裂出两个 entry（其一为空 path 的 forged entry）
  - 修复后：P1/P1b/P3 add exit 1 validation `note is not representable in brief format: note embeds a heading-shaped line ('#', '##' or '###') outside a code fence...` 零写入；P2 add exit 1 `entry title 'Entries' serializes to the legacy '## Entries' wrapper heading`；全部探针后 brief read 保持 exit 0 可读；P4（fence 内标题行）仍 exit 0 且 roundtrip 通过（既往合法内容不受影响）
- 测试证据：guard_tests.rs +3（note 标题行拒绝、Entries 拒绝、fenced-heading roundtrip）；cli_integration.rs +2（端到端零写入 + roundtrip）；workspace 410 → 419 全绿
- 兼容面盘点（已入提交信息与 CHANGELOG）：既往「合法」但现拒绝者仅为必然导致损坏的形态（fence 外标题行 note、未闭合 fence note、Entries 文件名）；合法内容不受影响（普通 prose、属性形态非首行、fence 内标题示例）；读侧对存量文件行为不变
- 提交哈希：0b4da90

### I-2 CHANGELOG Unreleased 补录修复波行为变更（重要）— 修复（文档）
- 终态：修复——Unreleased 追加 fix-wave guardrails 小节（D2 fence fast-fail、D3 prose 标题行拒绝、D4 scope glob 单行校验 + profile create 原子化、C-1 brief 护栏与读侧闭合行修复、兼容面盘点与 P-2 residue 触发面澄清）及 D6 category io→validation 变更小节，风格对齐既有条目；「半份清单」歧义消除
- 提交哈希：db3d023

### L-2 R7 尾扫 CRLF 边界（低）— 修复
- 终态：修复——thread_scan.rs R7 prev 判定放宽为 `\n` 或 `\r` 均为行边界；新增三回归测试（lone-CR 后保留完整首行、CRLF 分裂保持、真·行中切割仍丢弃作回归控制）
- 提交哈希：ec59c01

### I-5 e2e-verification 文档悬置（低）— 修复
- 终态：修复——docs/dev/e2e-verification-2026-08-15.md 纳入本次 docs 提交
- 提交哈希：本节所属 docs 提交（见 git log；三份评审报告与台账追加同批）

### 台账卫生 6 项（完整性报告低-1/2/3/4/6）— 修复/登记
1. 低-1 D1 证据链措辞更正：第一节 D1 条目已更正为「title 带换行三形态（post/brief/contacts）」，与 _fix/repro-audit.ps1 实际探针对齐 — 本节所属 docs 提交
2. 低-2 D4 R-12 第二面显式裁决：第一节 D4 条目已补裁决句（等号形态成安全 bypass，降为 UX 观察项，登记不处置）— 本节所属 docs 提交
3. 低-3 P-6 lossy 集中登记：见本台账第七节 — 本节所属 docs 提交
4. 低-4 头部 A-01/A-02 归属更正：台账头部权威输入行已更正（A-01/A-02 归深审 A audit-grammar-matrix；深审 C 仅 S-01）— 本节所属 docs 提交
5. 低-6(1) LED-04/05/14 状态刷新：见 open-items-ledger 第十节 — 本节所属 docs 提交
6. 低-6(2) thread_scan.rs L15 注释清理 LockedFile 残留：已随 L-2 修复提交（注释改为 caller's lock window）— ec59c01

### L-1 contains_heading_line 对 `#hashtag` 误杀（低）— 裁定：维持保守策略
- 裁定理由：preamble prose 护栏刻意非 fence-aware，镜像解析器对裸序列化 prose 的行为（写侧镜像解析器所见）；若收窄为 CommonMark 标题结构（# 后随空白），将为形近变体（`#define`、`#hashtag` 起首的粘连注入）重新打开结构伪造面，而误杀代价仅为带 fix 指引的 validation 拒绝，agent 可自愈；正确性优先于宽松度。登记不改，无代码动作。

### I-3 D2 预检锁内读整文件（低）— 登记：已知权衡
- 登记口径：正确性换性能的有意取舍（防静默数据丢失优先）；本产品场景 thread 文件量级小（笔记/消息体），持锁 O(file) 可接受；若未来出现大线程高并发场景，再立专项改仅扫末段 fence 状态的增量预检。登记不改，无代码动作。

### I-4 .gitattributes eol=lf renormalize 噪音（低）— 裁定：保留
- 裁定理由：该文件保护字节级黄金测试（char_tests）在 autocrlf=true 机器上不被 CRLF 化，目的必要；单人仓库当前无实际影响，未来协作时一次性 `git add --renormalize .` 或重新检出即可，发布轮 release notes 提一句。登记不改，无代码动作。

### 评审闭环销账统计
- 三份报告发现总数：11 项（完整性 6 低；正确性 1 阻塞 + 2 低 + 1 备注；影响面 1 阻塞 + 1 重要 + 3 低）
- 修复：6（C-1/I-1+备注面、L-2 代码；I-2、I-5、卫生 1/2/3/4/5 文档；卫生 6 随 ec59c01）
- 裁定登记：3（L-1 维持保守、I-3 已知权衡、I-4 保留）
- 悬置：0
- 纪律核验：原子提交（0b4da90 fix / ec59c01 fix / db3d023 docs）；`cargo test --workspace --locked` 419 全绿；clippy -D warnings 零警告；fmt --check 通过；未 bump/tag/推送；输出协议只增不改；黄金快照未重冻

---

## 七、P-6 纯展示/推断面 lossy 集中登记（评审闭环追加，2026-08-15）

裁定：保留不改。路径改写面已必修闭合（ensure_suffix OsStr 融合 NEW-3 + default_title OsStr 化，代码注释有裁决）；剩余四处为纯展示/ASCII 后缀推断面，lossy 不影响任何写路径：

| 位点 | 用途 | 保留理由 |
|---|---|---|
| cli cmd/profile.rs L263 | profile list 文件名展示 | 纯展示面，lossy 不改写磁盘路径 |
| cli cmd/validate.rs L53 | 后缀推断验证类型 | ASCII 后缀推断（.post/.brief/.contacts 均为 ASCII），lossy 不改变推断结果 |
| core ops/contacts.rs L304 | 联系人名 fallback（文件名 stem） | 仅用于展示用的名字推断，非路径改写 |
| core ops/manifest.rs brief_add_entry title 派生（file_name lossy，约 L124） | entry title 派生 | title 仅序列化为标题文本，不回写路径；非 Unicode 文件名经 C-1 Entries 护栏与常规 title 护栏后行为可预期 |

（第六、七节完。撰写：修复轮二执行 agent；取证时间 2026-08-15。）

---

## 八、边界更正注记（2026-08-15，owner 边界更正，append-only，未改动第一至七节）

- 更正：owner 从未指示发布 0.6。本台账第一节 S-01 条目中「含发布轮一次性闭合建议：bump 0.6.0 + CHANGELOG + crates.io + tag」与第六节 I-4 裁定理由中「发布轮 release notes 提一句」改读为：发布时机待 owner 指示，本工作流无发布计划；相关事项仅作事实登记，不构成 bump 建议或发布轮计划。
- S-01 的事实登记面（crates.io 0.5.0 与仓库版本语义错配）本身成立，不受本更正影响。
- 权威口径见 docs/dev/open-items-ledger-2026-08-15.md 第十二节。

---

## 九、CI smoke 失败事件全记录（任务 #39，2026-08-15，append-only，未改动第一至八节）

追加依据：任务 #38 诊断报告 docs/dev/ci-failure-diagnosis-2026-08-15.md（Lucas，状态已结案：根因确认 + 已被后续提交修复）。本节为事件账目登记，纯文档，无代码动作。

### CI-F1 ci.yml 内嵌 smoke 调用已撤销糖标志，smoke×3 平台失败 — 已闭合（由回填批 f94b65f 修复）

- **失败 run**：31877484785（HEAD `46c637c`，三维评审修复轮）与 31877562381（HEAD `669342e`，台账第十四节提交）；两 run 结构完全一致：fmt ✔ / test×3 ✔（含 clippy、docs）/ **smoke ubuntu/macos/windows ×3 ✘**。
- **根因（Verified，因果闭环）**：裁决批 O1（`9821933`）撤销写侧糖标志 `--reply-to`/`--mention` 时，漏改 `.github/workflows/ci.yml` 内嵌 smoke 脚本（unix 块第 77 行 + windows 块对应行仍调 `post send standup --author bob --reply-to 1 --mention alice --message "Reply"`）；CLI 以 usage 错误（exit 2）拒绝，`grep "^ok post.send"` / `Assert-Contains` 断言失败，step exit 1。`9821933..669342e` 逐提交核验 8/8 版 ci.yml 均含该调用；O2–O5 与三维评审轮均未补（本地 426 全绿未拦住的原因：smoke 内嵌于 ci.yml，非 cargo 测试目标，`cargo test` 永不执行）。
- **批次归属**：引入者 = 裁决批自身（O1 漏改）；修复者 = 任务 #52 回填批 `f94b65f`（commit message 明示 "Incidental: ci.yml smoke ... corrected to the v0.6 body-token form"，smoke 两处改 `--message "@#1 Reply @alice"` 形态，并将 docs gate 加固为 RUSTDOCFLAGS -D warnings）；回填批与失败无因果关联（其 Ivy 16 项测试在失败 run 的 test job 中亦全绿）。
- **闭合证据**：run 31879040813（HEAD `3ef5dc5`）**全绿**；本地按 ci.yml 实际命令逐项复跑（444 测试 + fmt/clippy/docs gate + windows pwsh 与 Git Bash 双档 smoke）全部 PASS，与线上一致；历史失败点静态（`git show 669342e:.github/workflows/ci.yml`）+ 动态（现行二进制复现逐字同一 usage 错误）双闭环。
- **终态**：已闭合，无新代码改动需求；release.yml 未触发（仅 tag 触发），无改动需求。

### 防复发措施（流程规则，登记为后续批次强制项）

- 凡 CLI 标志增删（尤其裁决类 breaking 变更），验证清单必须包含对以下面的全仓 grep 扫查，逐个命中点处置：`.github/workflows/*.yml`（内嵌 smoke 是本次漏点）、`SKILL.md`、`README.md`（含 repos/ 下子 README）、`_e2e/*`。
- 该规则同步登记于 open-items-ledger 第十五节（供后续批次引用）与 workflow-and-todo 验证阶段检查项。
- ci.yml unix/windows smoke 双份内嵌的结构性重复面登记备查：可考虑抽脚本由 workflow 调用，但当前内嵌形态已被 owner 既往接受，**稳定期不主动改**（诊断报告 §7.2 第 2 条口径）。

### 销账统计（本节）

- 缺陷条目：1（CI-F1）；终态：已闭合（由 f94b65f 修复，run 31879040813 全绿实证）；防复发规则：已登记（3 处落点）；悬置：0。
