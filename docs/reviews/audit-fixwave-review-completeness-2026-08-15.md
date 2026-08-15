# 修复波三维评审 · 完整性（requirement coverage）报告

- 日期：2026-08-15
- 评审维度：**仅完整性**（正确性 bug 与回归影响由另两位评审员负责，本报告不越界判定）
- 评审范围：`git diff 3829fd9..da954c2`（origin/master → HEAD，23 提交：perfection 续做 P-0~P-9 + 审计修复波 D2/D3/D4/D5/D6/D7/A-01/A-02 + ledger 销账）
- 需求基线：perfection-and-branches-assessment §五 工单、perfection-execution-log、audit-robustness D1~D7、audit-grammar-matrix A-01/A-02、fix-ledger、ssot-audit-fixes-task34 移交 2 项、bdd-scenario-test-map
- 评审方法：逐条把台账声称与 diff/源码/测试实况对拍；全量 `cargo test --workspace --locked` 实测；BDD 差分表程序化核对（89 个 S-* ID 全量 + 20 个引用测试名抽查）
- 纪律：只读评审，未改任何源代码，未做 git 提交（本报告写入除外）

---

## 一、P-0~P-9 逐批核对结果

| 批次 | 工单验收标准 | diff 实况核验 | 判定 |
|---|---|---|---|
| P-0 基线重定 | 计划文档 append 增补、位点重扫落盘、与 LED-05/14 口径一致 | `docs/reviews/v0.5-perfection-plan-2026-08-15.md` §10 增补段在场（a941b3b，+41 行）：10.1 基线变更表（176→288、708 行、执行序）、10.2 位点重扫清单（IoContext 44 处、RMW 6 处、lossy 7 处等实测值）、10.4 逐批裁定登记；§10.4(5) 声明 ledger 本体为豁免区 | 满足 |
| P-1 锁层融合 | master 仅一套锁实现；NEW-13 二选一闭合；六写路径行为不变 | `ops/lock.rs` 头部 L18-27 落盘裁定（locked_read_modify_write 为 SSOT，LockedFile RAII 不采纳）；全仓 grep 确认无 LockedFile 类型代码（仅注释提及）；NEW-13 证据测试在场（lib 实测跑到 closure_error_path_releases_lock、no-op 跳过测试）；6d5caeb fmt 清扫 | 满足 |
| P-5 黄金快照重冻 | v0.6 口径重冻、全绿、ASCII 断言保留 | cli char_tests.rs 1749 行 31 测试 + core char_tests.rs 12 roundtrip 实测全绿；`.gitattributes` 钉 `* text=auto eol=lf`（243207e）保护字节门禁 | 满足 |
| P-2 护栏移植 | NEW-1/2/4/5/6 + SAM-1/2/4 全接线、guard_tests 全绿、CHANGELOG 披露 | 写侧守卫逐点在场：thread.rs L100-104（title）、profile.rs L44-45/L83-84、manifest.rs L43-47/L107、contacts.rs L33/L76；create_new_file 原子创建；guard_tests.rs 30 测试实测全绿；CHANGELOG Unreleased「Added — write-side injection guardrails (P-2 batch)」段在场 | 满足 |
| P-3 非锁基建 | scanner 族迁移、单趟 normalize、dedup/suffix helper、差分语料 | format/mod.rs helper 族在场（for_each_outside_fence / check_single_line / prose_representation_issue 等）；test_normalize_single_pass_equivalence 等差分测试实测在场全绿 | 满足 |
| P-4 IoContext 迁移 + SAM-5 | io_ctx 切换、Io 死变体移除、对拍测试、CHANGELOG Breaking 披露 | error.rs `Io(#[from])` 变体已不再在场、io_ctx 构造器在场；test_io_ctx_envelope 对拍测试实测跑到；CHANGELOG Unreleased 的 SAM-5 Rust API 变更（Breaking，仅 Rust 直调面）披露段在场 | 满足 |
| P-6 JSON 收口 + NEW-3 融合 | JsonBuilder 单构造路径、ensure_suffix 三级+OsStr 融合、非 Unicode 回归、CHANGELOG 披露 | output.rs JsonBuilder（insert/insert_opt/build/print_json）在场；cmd/mod.rs L126-205 双平台非 Unicode 回归（unix bytes + windows 0xD800）在场且实测全绿（cli unit 6）；default_title OsStr 原生剥后缀（post.rs L584-602）；t6_cli_tests 4 测试全绿；CHANGELOG P-6 两段在场 | 满足 |
| P-7 拆分 + 性能五件 | re-export 面不变、NEW-7/8/10/11/12 落地 | thread_read.rs/thread_scan.rs 新文件在场、thread.rs 保留 re-export 注释面；NEW-7/11（hash.rs 流式 + 单趟 hex，含 e3b0c442 空哈希钉住 L89/L182）、NEW-8（edit 增量重写差分语料 8 测试）、NEW-12（find_message_sender 尾扫）、NEW-10（mention 去重接 helper）均有对应提交与测试；CHANGELOG P-7 段在场 | 满足 |
| P-8 文档与 CI | BDD 差分表无未映射、README 计数、ci 双门禁、CHANGELOG | bdd-scenario-test-map 落盘：程序化核对 bdd.md 全部 89 个 S-* ID 均在表中（missing=0），抽查 20 个引用测试名 20/20 存在；README 实测无硬编码测试计数（「无需同步」实证关闭成立）；ci.yml `cargo test --locked --workspace` + `cargo doc --no-deps --workspace` 双门禁在场；CHANGELOG P-8 CI 段在场 | 满足 |
| P-9 终验收口 | T9 门禁全绿、release 实证、B1 SHA256、按归属分批提交 | `docs/dev/e2e-verification-2026-08-15.md` 落盘：cargo clean 冷重建、clippy 零警告、fmt 通过、release 构建、34 探针 smoke、16 并发压测、CHANGELOG 纪律核验；B1 零字节 SHA256（e3b0c442 开头标准值）在执行日志记录且由 hash.rs 测试常量钉住；23 提交逐批归属清晰；未推送 = 任务书覆盖工单推送要求（执行日志明示），合规 | 满足（见低-5） |

执行日志「采纳/改写/放弃」清单与 diff 实况抽查一致：LockedFile RAII 确未入 master（放弃成立）；implicit_mention 字段保留（改写声明成立，implicit_mention_triggered_on_reply 实测全绿）；Sam-m-γ 不采纳（default_title 保持 OsStr 原生形态，post.rs 实况吻合）。

## 二、D1~D7 / A-01 / A-02 / S-01 / 移交项核对结果

| 条目 | 台账声称终态 | diff 实况核验 | 判定 |
|---|---|---|---|
| D1（阻塞） | 登记：已由 669befa NEW-1 批闭环 | 三格式 title 注入向量守卫在场（thread.rs L103 / manifest.rs L43 / contacts.rs L33）；guard_tests 对应测试全绿；复现脚本 _fix 目录 repro-audit.ps1 的 D1 三探针即原始 R-17/R-18 攻击向量 | 闭环成立（见低-1） |
| D2（阻塞） | 修复：锁内 fence 平衡预检 | thread_scan.rs unclosed_fence_issues_locked 在场；thread_send L180-185 与 thread_edit L370-374 双路 fast-fail；guard_tests 2 测试 + cli_integration post_send_and_edit_refuse_unclosed_fence_thread 实测全绿；8abdec6 | 满足 |
| D3（高） | 修复：prose 拒绝标题形态行 | format/mod.rs contains_heading_line 接入 prose_representation_issue（L389-395）；guard_tests 3 测试（create/edit/brief 标题注入拒绝）+ cli_integration profile_create_heading_description_injection_refused_zero_write 全绿；2c7a180 | 满足 |
| D4（中） | 修复：scope glob 单行校验 + create 原子化 |ops/profile.rs check_scope_globs（L26-32）create_full 与 edit 双接入（L85-89、L164-167）；2c7a180；cli 侧 profile.rs L116-130 改走 create_profile_full 单次原子写；guard_tests 与 cli_integration 零写入回归全绿；guard_tests 2 测试 + cli_integration profile_create_scope_glob_injection_refused_zero_write 全绿 | 满足（见低-2） |
| D5（中） | 钉住 + 登记 |cli_integration ascii_contract_is_structural_surface_only（3be44dc）实测全绿；io 豁免面已登记台账 LED-16；task34 移交项 1（ascii 契约守护测试口径对齐评估）的评估结论在 fix-ledger D5 与 LED-16 落盘 | 满足 |
| D6（轻微） | 修复：fix 文案指向编码 | b107771：cmd/post.rs +32（InvalidData 走 Validation 信封分支）；cli_integration post_send_stdin_non_utf8_fix_points_at_encoding 实测全绿 | 满足 |
| D7（轻微） | 修复（文档） | 9884d89：spec.md §3.1 --author 行改写为单 token 校验并交叉引用 fix-ledger，diff 逐字核实 | 满足 |
| A-01（低） | 修复（文档） | bdd.md S-READ-06 空线程 showing 口径由 0/4 更正为 0/0，tdd.md 同数值同步，diff 逐字核实 | 满足 |
| A-02（低） | 修复（文档） | bdd.md S-VAL-04 example 文件名 myfile.post.md 更正为 myfile，diff 逐字核实 | 满足 |
| S-01 | 登记不改，转发布轮 | open-items-ledger 第九节 LED-15 在场（9884d89 追加，含发布轮一次性闭合建议）；README crates.io 警示块（task34 过渡措施）8 处命中在场 | 满足 |
| LED-16 登记 | D5-io 豁免登记 | open-items-ledger 第九节 LED-16 在场，状态「已裁定（登记不改）」 | 满足 |

## 三、测试覆盖与口径核验（实测）

- `cargo test --workspace --locked` 实跑：**410 通过 / 0 失败**，分文件计数与 fix-ledger §四 逐项吻合：cli unit 6、cli char 31、cli_integration 141、t6 4、core lib 97、core char 12、guard 30、contacts 18、ops 71。台账无虚报。
- 每个代码修复均有回归测试：D2（guard×2 + e2e×1）、D3（guard×3 + e2e×1）、D4（guard×2 + e2e×1）、D5（边界钉住×1，含正向回显 + 结构面 ASCII 双向断言）、D6（e2e×1，正向 stdin 由既有 post_send_stdin 承载）。
- BDD 差分表「无未映射场景」属实：v0.6 bdd.md 全部 89 个 S-* ID 程序化比对零缺失；format-v2 侧 79 场景计数（PROF 11 + POST 36 + BRIEF 12 + CONT 8 + CONC 4 + VAL 8）与表内枚举一致；抽查 20 个被引用测试名（含 S-EDIT-08 新增的 edit_v05_grammar_positional_is_usage）20/20 在场。

---

## 四、发现清单

### 阻塞（MUST FIX）

无。

### 重要（SHOULD FIX）

无。

### 低（CONSIDER）

**低-1 fix-ledger D1 条目的实测证据链描述与复现脚本实际探针形态不符**
[docs/dev/fix-ledger-2026-08-15.md#L17](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/fix-ledger-2026-08-15.md)
- 证据：台账称复现脚本对「author 带换行 / scope 带换行 / body 首行伪装属性行」三种形态实测，但 `_fix/repro-audit.ps1` L16-25 实际探针是 D1 的原始攻击向量「title 带换行」三形态（post/brief/contacts）。实际覆盖比台账描述更贴 D1 原义，不构成虚报，但证据链文字失实。
- 建议：把 D1 条目实测核验句更正为「title 带换行三形态（post/brief/contacts）」，与 repro 脚本对齐。

**低-2 D4 原报告「空格形态拒收无绕过引导」面未显式说明处置**
[docs/dev/fix-ledger-2026-08-15.md#L39-L44](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/fix-ledger-2026-08-15.md)
- 证据：audit-robustness D4 描述含两面——「等号粘连放通注入」（已由 2c7a180 闭合）与「带值 flag 空格形态拒收且无绕过引导」（R-12 面）。台账 D4 条目仅覆盖注入面，第二面既未修复也未登记；注入面闭合后等号形态已成为安全 bypass，该面实质降为 UX 观察项，但销账文字未交代。
- 建议：在 D4 条目补一句对 R-12 面的裁决说明。

**低-3 P-6「纯展示面 lossy 登记留痕」未见集中登记点**
[docs/dev/perfection-execution-log-2026-08-15.md#L59-L61](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/perfection-execution-log-2026-08-15.md)
- 证据：工单要求 5+2 处 to_string_lossy「路径改写面必修、纯展示面登记留痕」。必修面已完成（ensure_suffix OsStr 融合 + default_title OsStr 化，代码注释有裁决）；但剩余展示/推断面（cmd/profile.rs L263 列表展示、cmd/validate.rs L53 后缀推断、core ops/contacts.rs L304、ops/manifest.rs L140）的「保留不改」裁决仅散见计划 §10.2 的位点清单，执行日志 P-6 明细与 ledger 均无登记点。
- 建议：在执行日志 P-6 段或 ledger 补一行裁决登记（保留位点清单 + 理由「纯展示/ASCII 后缀推断，无路径改写」）。

**低-4 fix-ledger 头部权威输入标注把 A-01/A-02 归于深审 C**
[docs/dev/fix-ledger-2026-08-15.md#L5](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/fix-ledger-2026-08-15.md)
- 证据：台账头部写「audit-ssot-agentux（深审 C：A-01/A-02/S-01）」，但深审 C 全文为 S-01~S-13 编号、无 A-xx；A-01/A-02 实际出自 audit-grammar-matrix（深审 A），台账正文销账内容与深审 A 完全吻合。
- 建议：头部出处更正为「深审 A：A-01/A-02；深审 C：S-01」。

**低-5 T10 三路评审正式报告尚不在基线内（本轮正在产出）**
[docs/dev/perfection-execution-log-2026-08-15.md#L82-L87](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/perfection-execution-log-2026-08-15.md)
- 证据：工单要求「T10 三份评审报告落盘销账」；da954c2 基线内仅有执行日志的自查记录，正式 completeness/correctness/impact 三份报告由当前评审轮产出（本报告为其一）。属预期内进行中动作，列出以免销账遗漏。
- 建议：三份评审报告落盘并销账后再宣告 P-9/T10 最终闭合。

**低-6 台账联动卫生：LED-04/05/14 状态未随闭合刷新；thread_scan.rs 模块注释引用已放弃的 LockedFile**
[docs/dev/open-items-ledger-2026-08-15.md#L42-L49](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/open-items-ledger-2026-08-15.md)、[repos/paperwork-core/src/ops/thread_scan.rs#L15](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/repos/paperwork-core/src/ops/thread_scan.rs)
- 证据：perfection 全批闭合后 LED-05 仍「进行中」、LED-14 仍「开放」、LED-04 未随 LED-16 裁定联动刷新（执行日志声明 ledger 本体为禁区、仅 §九 append，属既定分工，非违规）；thread_scan.rs L15 注释「inside the caller's LockedFile window」引用了 P-1 裁定不采纳的类型名。
- 建议：下一轮 ledger 维护时刷新 LED-04/05/14 状态；注释改为「caller's lock window」。

---

## 五、完整性维度结论

**通过。**

P-0~P-9 十个批次全部落地且各自验收标准经 diff 与实测核验满足；D1~D7 + A-01/A-02 逐项有对应代码或文档改动与回归测试，无遗漏、无虚报（410 测试实测口径与台账逐项吻合）；task34 移交 2 项均已落实（ascii 契约口径评估结论落盘 + crates.io 对齐登记 LED-15）；LED-15/16 登记在场；BDD 差分表「无未映射场景」经程序化全量比对属实。6 条发现均为低级别的台账措辞/登记卫生问题，不影响任何工单的实质交付。T10 三路评审报告为当前进行中收尾动作（低-5），建议在报告落盘销账后正式宣告 perfection 全链闭合。

（报告完。评审人：完整性维度评审 agent；全部结论基于 git 对象、磁盘文件与 cargo test 实测。）

---

## 六、销账段（修复轮二，2026-08-15 追加）

本报告 6 项发现逐项销账（明细见 docs/dev/fix-ledger-2026-08-15.md 第六、七节）：

| 发现 | 终态 | 处置与提交哈希 |
|---|---|---|
| 低-1 D1 证据链措辞失实 | 修复（文档） | fix-ledger 第一节 D1 更正为「title 带换行三形态（post/brief/contacts）」— 本节所属 docs 提交 |
| 低-2 D4 R-12 第二面未裁决 | 修复（文档） | fix-ledger D4 条目补显式裁决句（等号形态成安全 bypass，降为 UX 观察项，登记不处置）— 本节所属 docs 提交 |
| 低-3 P-6 lossy 无集中登记点 | 登记 | fix-ledger 新增第七节：四位点清单 + 保留理由（纯展示/ASCII 后缀推断，无路径改写）— 本节所属 docs 提交 |
| 低-4 头部 A-01/A-02 归属错误 | 修复（文档） | fix-ledger 头部权威输入行更正（A-01/A-02 归深审 A audit-grammar-matrix；深审 C 仅 S-01）— 本节所属 docs 提交 |
| 低-5 T10 三份评审报告未在基线 | 修复（文档） | 三份评审报告落盘并各追加销账段（本段即其一）；P-9/T10 至此正式闭合 — 本节所属 docs 提交 |
| 低-6 LED-04/05/14 未刷新 + LockedFile 注释残留 | 修复 | 台账侧：open-items-ledger 第十节状态刷新（LED-04/05/14 均已闭合）；代码侧：thread_scan.rs L15 注释清理随 L-2 修复提交 — ec59c01 |

销账统计：6/6 全部落入终态（修复 5 + 登记 1），悬置 0。修复轮二全量验证：cargo test --workspace --locked 419 全绿 + clippy -D warnings 零警告 + fmt --check 通过；代码修复提交 0b4da90（C-1）/ ec59c01（L-2），CHANGELOG 补录 db3d023（I-2）。
