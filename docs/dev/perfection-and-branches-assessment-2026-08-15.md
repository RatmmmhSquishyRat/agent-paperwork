# 悬置工作线核查报告：v0.5-perfection 计划 × cli-ux-v0.5 分支

- 日期：2026-08-15
- 任务：#31 只读核查两条悬置工作线并给出裁定建议（全程只读 + 文档落盘）
- 取证方式：仅读取 git 对象（git show/log/grep/diff/merge-base）与磁盘文件；主工作区正处于 cli-grammar-v0.6 三方合并进行中（9 个 UU 冲突文件），本报告不依赖工作区暂态，全部结论以提交对象为准
- 取证基线：master @ 896cc62；cli-grammar-v0.6 @ a7bc3e2；cli-ux-v0.5 @ 70f7e43（= origin/cli-ux-v0.5）；wip/v0.5-perfection-snapshot-2026-08-15 @ d679b9a；origin/master @ 55c916a

---

## 一、核查线 1：v0.5-perfection 计划

### 1.1 计划全文盘点（docs/reviews/v0.5-perfection-plan-2026-08-15.md，896cc62 入库，251 行）

性质：owner「零悬置」指令驱动的 v0.5 遗留债务清零计划；权威债务来源为 v0.5-full-review-2026-08-14.md §3.4；骨架为 Eric「行为锁定先行 + 逐点等价替换 + 每步全量门禁」绞杀式方案，并入 Tina NEW-1~NEW-13 与 Sam 五项。

铁约束（计划原文）：格式规格 / CLI 文法 / ASCII 信封输出契约零变更（JSON 键名键序逐字节冻结）；公开 Rust API 不变；不 bump 版本（保持 0.5.0）；v0.6 与 cli-ux-redesign 未提交变更零触碰。
注意：其中「CLI 文法零变更」铁约束的基线是 v0.5 文法；cli-grammar-v0.6 合并入 master 后该基线整体失效，续做轮须以 v0.6 spec 为新冻结基线（见 §1.5 第 1 条）。

#### T0–T11 逐批清单（目标 / 范围 / 状态标记）

状态标记口径：已落盘 = wip/v0.5-perfection-snapshot-2026-08-15 分支实证在场；未动 = 无任何执行痕迹；部分 = 有痕迹但未闭合。

| 批次 | 目标 | 范围 | 状态标记（2026-08-15 实测） |
|---|---|---|---|
| T0 台账与计划落盘 | 文档先行，未落盘不开工 | 本计划文档本身 | 已完成（896cc62 入库） |
| T1 行为锁定 | 黄金信封快照（全命令面 stdout/stderr 逐字节）、错误文案冻结、四格式 roundtrip 恒等语料库、JSON 形状快照（9 处 payload）；既有 176 测试不动 | cli tests/char_tests.rs + core 侧特征化测试 | 已落盘（wip：cli char_tests.rs 1071 行 + core char_tests.rs 234 行）；但冻结值为 v0.5 文法口径（文件头自述 frozen v0.5.0 output contract），与 v0.6 文法/措辞冲突，合并后须重冻（见 §1.4） |
| T2 正确性护栏批 | NEW-1/2/3/4/5/6 + SAM-1（brief 部分迁移守卫）+ SAM-2（profile create 单次写入）+ SAM-4（verify_entry 错误区分），各配回归测试 | core 为主 | 已落盘（wip：guard_tests.rs 668 行逐条覆盖 NEW-1/2/4/5/6 与 SAM-1 负例；ops/mod.rs create_new_file 原子创建已接 profile/manifest/contacts 三 create；format/manifest.rs L63-90 brief 残留守卫在场） |
| T3 共享基础设施（纯新增） | 共享 fence 扫描器族、单趟 normalize、单行字段/prose 校验 helper；io_ctx helper + LockedFile RAII；头族正则集中 | format/mod.rs、error.rs、ops/lock.rs、format/thread.rs | 已落盘（wip：format/mod.rs +532 行，含 for_each_outside_fence / first/collect_outside_fence / dedup_preserve_order / strip_known_suffix / check_single_line / prose_representation_issue 等；error.rs io_ctx 构造器 + 字节一致回归测试；ops/lock.rs 251 行 LockedFile RAII） |
| T4 core 逐点迁移（严格串行） | 头族正则归族/删 SEQ_RE → IoContext 31 处 + RMW 5 处锁序列 → fence 低风险面 → 高风险面 → dedup/suffix helper 收口；每处先 differential 对拍后删旧 | core 全部 ops/format 文件 | 已落盘至终态（wip d679b9a 提交信息明示 T4 final state；ops/thread.rs 780（计划基线口径）→ master 现状 708 → wip 647 行；hash.rs 等已切 io_ctx） |
| T5 ops/thread.rs 拆分 + 性能批 | 拆 thread.rs/thread_read.rs/thread_scan.rs（pub use 保 API）；NEW-8 增量重写、NEW-7 hash 流式、NEW-10 去重、NEW-11 hex 单趟、NEW-12 reply-to 尾扫 | core ops + hash.rs + cmd/post.rs | 未动（wip 树无 thread_read.rs/thread_scan.rs；hash.rs 仍全量读、hex 仍逐字节；NEW-10 仅 helper 与 thread.rs 一处就位） |
| T6 CLI JSON 收口 | output.rs builder 替换 9 处命令侧手工 JSON（协议层 2 处保留）；validate.rs 消除重复归一化；ensure_suffix OsStr 修复（NEW-3） | cli output.rs + cmd/* | 部分落盘（wip：output.rs 已加 JsonBuilder insert/insert_opt/build + print_json；t6_cli_tests.rs 134 行含 NEW-4 回归；cmd/mod.rs NEW-3 OsStr 无损版就位——但分支 ensure_suffix 为三级解析 to_string_lossy 版，合并后须融合，见 §1.4 冲突 2） |
| T7 Ivy 测试缺口闭合 | G1–G5 超集新增测试 + 79 BDD 场景差分表落盘 + README 计数同步 | 纯新增测试 + 文档 | 部分落盘（guard_tests/char_tests 属超集增量；BDD 差分表与 README 计数未见） |
| T8 CI 与文档 | ci.yml 加 cargo doc --no-deps 与 --locked；CHANGELOG Unreleased 各披露小节；SAM-5 移除 PaperworkError::Io 死变体；Lucas/Ethan 闭合台账落盘 | ci.yml、CHANGELOG、error.rs | 未动（wip 无 ci.yml/CHANGELOG 变更；但 SAM-5 实质已在 wip error.rs 完成——Io 变体已移除、仅存 IoContext，披露动作仍欠） |
| T9 终验门禁 | 十三项 QA + 黄金快照总比对 + cargo clean 后 clippy + fmt --check + release CLI 实证 + B1 SHA256 复验 | 全量 | 未动 |
| T10 Ultra Review | 三路 CodeReview（completeness/correctness/impact）至零 Critical/Major | 评审 | 未动 |
| T11 提交推送 | 按归属分批提交（修复/测试/文档），排除 v0.6 工作流文件，push origin master | git | 未动（T1-T4 产物已被并行任务转入 wip/v0.5-perfection-snapshot-2026-08-15 分支保全；主工作区对应未跟踪测试文件已不在场） |

### 1.2 67eb049 与 perfection 计划的关系

- 67eb049「fix: address v0.5 full-review findings (1 blocking, 8 major, 15 minor)」，2026-08-15 00:12，26 文件 +1868/-498，位于 master 历史（896cc62 → 55c916a → 67eb049 → e71f4ca → 61e1e89 ...）。
- 判定：独立前置批，不是 T0、也不属 T1–T11。它闭合的是 full-review 的原始 findings（B1 contacts legacy 写守卫、RMW 上锁、parse_timestamp 收紧、CI fmt 门禁等），验证口径 176 测试全绿；perfection 计划的债务来源是同一次评审的 §3.4 台账中被「裁决留档/延期」的部分（Noah DRY 五项、Lucas m6–m10、Ethan S1、Ivy G1–G5），由 owner 指令升级为闭合项。
- 关系链：67eb049（修原始 findings，立 176 测试基线）→ 55c916a（review book 落盘）→ 896cc62（perfection 计划 T0 落盘）→ T1+ 执行。open-items-ledger 第四节已备查此关系（「commit 67eb049 修复 + 55c916a review book 落盘；其 §3.4 延期项已被 perfection-plan 升级为本轮闭合」）。

### 1.3 已知状态盘点核实（任务书三点逐项核对）

1. T1 行为锁定测试：任务书称工作区未跟踪 char_tests.rs×2、guard_tests.rs、t6_cli_tests.rs。实测：当前主工作区 git status 未跟踪项仅剩 docs/dev/ 两个文件，四个测试文件已不在工作区；全部已作为 d29fb75「wip snapshot, DO NOT DELETE, restore from this branch」提交进 wip/v0.5-perfection-snapshot-2026-08-15 分支（cli char_tests.rs 1071 行、core char_tests.rs 234 行、guard_tests.rs 668 行、t6_cli_tests.rs 134 行），与任务书「正被并行任务转入快照分支」吻合，且转入已完成。
2. T4 thread.rs 迁移：wip 分支 d679b9a 提交信息「T4 final state - exempt LockedFile::rewrite with rationale」，ops/thread.rs 降至 647 行（master 现状 708 行）——T4 在快照分支上已达终态，而非仅「进行中」；「进行中」是主工作区被冻结前的旧口径。
3. T9/T10/T11 未动：实证无痕迹（无终验记录、无评审报告、wip 分支仅 2 个快照提交未推 origin），确认未动。

### 1.4 逐批对照 cli-grammar-v0.6：覆盖 / 重做 / 冲突裁定

背景实测：cli-grammar-v0.6（a7bc3e2）对 perfection 目标文件的改动 = v0.6 具名文法重写（d6e9ff3 五个 cmd 文件全改）+ contacts CRUD（cbb3790/fdbcbab）+ 锁统一（77ab558 抽出 locked_read_modify_write、595f9b2/cbb3790 接入写路径）+ 274 测试基线（c31c4cf 集成套件重写）。实测分支不含 perfection 的 NEW-1/NEW-2/SAM-1 等护栏（git grep 无 create_new、无注入守卫标记），两线工作重叠面集中在锁、RMW 序列、cmd 文件与测试基线，而非护栏本身。

| 批次 | 与分支工作的关系 | 裁定 |
|---|---|---|
| T0 | 无冲突 | 保留；须补一次「基线变更」回流修订（文法基线 v0.5→v0.6、测试基线 176→274、ops/thread.rs 行数基线重测） |
| T1 | 冲突：黄金快照冻结的是 v0.5 文法输出；分支 5456eed 已把全部 ops example/fix 文案刷新为 v0.6 具名文法，d6e9ff3 改了 flag 面，信封新增 implicit-mention/showing/window 等字段 | 机制保留、字面值重冻：合并后在 v0.6 口径上重新生成 char_tests 全部字面量（测试骨架、确定性策略、(TS) 掩码方案可原样沿用） |
| T2 | 未被分支覆盖（分支无 NEW-1/2/4/5/6、无 SAM-1/2/4 等价物；分支自己的新增是空键守卫、reject_foreign_thread 等 v0.6 特有项） | 必须在新结构上重做：把 wip 的护栏移植到合并后的 ops/format 文件（分支重写过的 contacts/profile/manifest/thread 四文件是移植主战场） |
| T3 | 部分冲突：fence 扫描器族 / 单趟 normalize / 单行校验 helper / dedup / suffix helper 与分支无冲突可直接移植；但锁层存在两套设计——wip 的 LockedFile RAII + io_ctx（ops/lock.rs 251 行）vs 分支的 locked_read_modify_write 单一 helper（ops/lock.rs，已被 6 条写路径接入并有锁场景测试） | 非锁部分直接移植；锁层二选一融合，建议以分支实现为骨架（已随 274 测试回归验证、且 brief add/remove 与 contacts CRUD 写路径已依赖之），将 io_ctx 收口与 RAII 化的诉求（NEW-13 早退路径消除）作为其内部重构并入，配 differential 对拍；不保留两套并存 |
| T4 | 同批文件被分支重写（v0.6 文法 + CRUD + locked RMW），wip 的 IoContext→io_ctx 迁移与 fence 谓词共享无法直接叠加（合并冲突文件中 ops/contacts.rs、ops/manifest.rs、ops/profile.rs、ops/thread.rs 四个 UU 正是这批） | 必须在新结构上重做：合并定稿后按 T4 原序列（正则归族 → io_ctx 迁移 → fence 谓词共享 → helper 收口）在分支版文件上重新逐点迁移，继续「先对拍后删旧」纪律 |
| T5 | 未被分支触及（分支无拆分、无 NEW-7/8/11/12；NEW-10 仅 wip 有 helper） | 原样续做，但行号位点全部失效，须按合并后代码重新定位 |
| T6 | 部分冲突：output.rs builder 为纯新增可移植；但分支 cmd 文件已全面重写且信封字段有增改（emit_usage_error 等已演进），9 处命令侧手工 JSON 位点全部漂移；ensure_suffix 两代并存（分支三级解析但仍 to_string_lossy @ L49，wip 单级 OsStr 无损） | 重做接线：builder 接到分支版调用点；ensure_suffix 融合 = 分支三级解析语义 + wip OsStr 无损实现（NEW-3 在分支上实测仍未闭合，保留该债） |
| T7 | 测试基线从 176 变 274，G1–G5 超集断言面（validate 信封、post edit 三拒绝等）的文法字面量随 v0.6 变化 | 续做：guard/char 类测试随 T1/T2 移植；BDD 差分表按 v0.6 bdd.md 重对齐；README 计数按合并后新基线同步 |
| T8 | ci.yml 两侧都改过（合并暂存区已含分支版），CHANGELOG 分支加了 0.5.0 迁移表与 v0.6 段落 | 续做：cargo doc/--locked 门禁加到合并版 ci.yml；CHANGELOG Unreleased 披露（守卫行为新增 + SAM-5 的 Rust API 变更——Io 变体移除在 wip 已完成，披露义务不消失） |
| T9 | 无对应 | 续做；黄金快照总比对改为 v0.6 重冻版 |
| T10 | 无对应 | 续做 |
| T11 | 与正在进行的三方合并存在提交边界交织（open-items-ledger LED-02 已登记） | 续做且顺位最后：必须在 cli-grammar-v0.6 合并完成并推送之后，按归属分批提交 perfection 成果；wip 快照分支作为恢复源保留至全部成果回流 master |

#### 冲突裁定明细（两处硬冲突）

冲突 1（锁层，T3/NEW-13）：
- wip 侧：ops/lock.rs 251 行，LockedFile RAII + io_ctx helper，d679b9a 附「exempt LockedFile::rewrite」裁决注记；目标是 §2.3 的 5 处 RMW 锁序列统一与手工 unlock 早退路径消除。
- 分支侧：ops/lock.rs 的 locked_read_modify_write（闭包式 RMW，含 no-op 跳过重写语义），77ab558 抽出、595f9b2/cbb3790/da42fa7 接入 brief/profile/contacts/thread 全部写路径，7db14a3 与 dfce66c 有锁场景测试。
- 裁定建议：同一债务（RMW 锁统一）的两代方案，取分支版为 SSOT（已随 288 测试回归验证、且 brief add/remove 与 contacts CRUD 写路径已依赖之），wip 版降级为「设计输入」——其 io_ctx 错误收口与 RAII 早退消除若仍可实现为分支 helper 的内部重构，则作为续做项并入 T4 重做轮，否则以「核实无需改 + 实测证据」闭合 NEW-13。两套 lock.rs 不允许并存入 master。

冲突 2（ensure_suffix，T6/NEW-3）：
- 分支版（合并后 master 现状）：三级解析（原路径优先 → 后缀变体 → 落点），但 cmd/mod.rs L49 仍 to_string_lossy，NEW-3（非法 UTF-8 静默替换）未闭合。
- wip 版：OsStr 原生无损拼接（含非 Unicode 分量回归测试），但是单级语义。
- 裁定建议：融合——三级解析语义（U-14 已解决项）+ OsStr 无损实现（NEW-3），列为续做项。

补充实测（合并完成后的 master @ 3829fd9 取证）：ops/lock.rs = 127 行（分支版）；ops/thread.rs = 708 行（未拆分）；error.rs L40 Io(#[from]) 死变体仍在（SAM-5 未随合并落地）；cli 侧 to_string_lossy 共 5 处（cmd/mod.rs L49、post.rs L592-593、profile.rs L279、validate.rs L53），NEW-3 债务面比计划原文（1 处）更宽。

### 1.5 合并后 master 上的 perfection 续做清单

顺位即建议执行序；每项沿用计划闭合通则（修复+回归 或 核实无需改+实测证据，不留新悬置）。可派工的工单要素见第五节。

1. P-0 基线重定（文档批）：回流修订 perfection-plan——冻结基线改为 v0.6 spec / 合并后 288 测试基线 / 合并后行数；登记本报告的逐批裁定为 T0 增补；重测 §9.2 计数基线。
2. P-1 锁层融合裁定落地（冲突 1）：以合并后 master 的 locked_read_modify_write（ops/lock.rs 127 行）为 SSOT；评估 io_ctx/RAII 作为内部重构的可行性；NEW-13 按二选一闭合。
3. P-2 T2 护栏移植：NEW-1（单行字段/prose 守卫，wip format/mod.rs helper 族 + 各 ops 写侧接线）、NEW-2（create_new_file 原子创建）、NEW-4/NEW-5/NEW-6、SAM-1/2/4，连同 guard_tests.rs 668 行回归一并移植到合并后文件。
4. P-3 T3 非锁基础设施移植：fence 扫描器族、单趟 normalize、dedup_preserve_order、strip_known_suffix、check_single_line 等（format/mod.rs +532 行为纯新增，冲突面小）。
5. P-4 T4 逐点迁移重做：在合并版 ops/format 文件上按原序列迁移 io_ctx 与 fence 谓词共享，每处 differential 对拍；SAM-5（Io 死变体移除，wip 已有实现）随本批落地并披露。
6. P-5 T1 黄金快照重冻：char_tests 字面量在 v0.6 口径重新生成（骨架沿用，冻结对象明细见第五节 5.2）。
7. P-6 T6 收口：JsonBuilder 接到合并版 cmd 调用点；ensure_suffix 融合（冲突 2），并覆盖合并后实测的 5 处 to_string_lossy。
8. P-7 T5 拆分与性能批：thread.rs 三分 + NEW-7/8/10/11/12，位点按合并后代码重新定位。
9. P-8 T7/T8：BDD 差分表（对齐 v0.6 bdd.md）、README 计数、ci.yml cargo doc/--locked、CHANGELOG Unreleased 全部披露（含 SAM-5 Rust API 变更）。
10. P-9 T9/T10/T11：终验门禁 → 三路评审 → 按归属分批提交推送；wip/v0.5-perfection-snapshot-2026-08-15 分支在全部成果回流前不得删除（其提交信息已自带 DO NOT DELETE 标记）。

---

## 二、核查线 2：cli-ux-v0.5 分支

### 2.1 十个领先提交（master..cli-ux-v0.5 取证时点口径，全部 2026-08-09 02:05–02:42 窗口内）

| # | 提交 | 内容 | 规模 |
|---|---|---|---|
| 1 | 3921ee9 | docs(governance)：v0.5.0 CLI UX 重设计规格集落盘——researches×3、doc-review×4、role、v0.5_feedbacks、docs/ssot/specs/cli-ux-redesign/ 全套（README/bdd/design/impl_plan/spec/tdd） | 15 文件 +3221 |
| 2 | 74e3cc0 | docs(core)：14 条 ops 错误 example 串刷新为 v0.5 文法（纯文本） | 4 文件 +14/-14 |
| 3 | baa0cb5 | feat(cli)：v0.5.0 文法重设计——PATH/NAME 位置参数优先 + usage 信封 exit 2（main.rs +227 为信封主体） | 8 文件 +481/-66 |
| 4 | 166c4d1 | test(cli)：29 处调用点重写为 v0.5 文法 + usage 信封覆盖 | 1 文件 +782/-29 |
| 5 | cb10a8a | ci：smoke 采纳 v0.5 文法并断言 usage 信封 exit 2 | 1 文件 +36/-18 |
| 6 | d35983c | docs：CHANGELOG 0.5.0 迁移表、README 刷新、SKILL.md 新增、adr-v1 superseded 注记 | 5 文件 +210/-16 |
| 7 | 5948058 | chore：两 crate bump 0.5.0 | 3 文件 +5/-5 |
| 8 | 17c2e57 | qa：v0.5.0 CLI UX 评审书（v0.5-review-2026-08-09.md）+ test-v05 语料 19 文件 | 20 文件 +2041 |
| 9 | 39156db | fix(cli)：评审轮修复——ASCII 输出守卫、usage 信封完整性、CHANGELOG 准确性 | 13 文件 +484/-49 |
| 10 | 70f7e43 | qa：二轮复验，BUG-1/BUG-2/W-1..W-3 标记解决 | 1 文件 +17/-9 |

### 2.2 merge-base 关系（git 只读实测）

- merge-base(master, cli-ux-v0.5) = a7ea07c（README 重构）：取证时点 master 不含该分支任何提交；master 的 v0.5.0 是 format-v2 纯 Markdown 格式轮（61e1e89），与该分支的 CLI UX 文法轮是两条并行线。
- merge-base(cli-grammar-v0.6, cli-ux-v0.5) = 70f7e43 = cli-ux-v0.5 分支顶点：cli-grammar-v0.6 完全以 cli-ux-v0.5 为祖先（branch --contains 70f7e43 命中 cli-grammar-v0.6；cli-ux-v0.5..cli-grammar-v0.6 共 31 提交，含 a07ad4c「merge: integrate master (format-v2) into cli-grammar-v0.6 baseline」、42c6d34 落 v0.6 规格集、d6e9ff3 具名文法重写）。
- 计数：master..cli-ux-v0.5 = 10；cli-ux-v0.5..master = 5。远端：origin/cli-ux-v0.5 = 70f7e43（与本地一致）。
- 合并完成后复验（2026-08-15 02:1x，master @ 3829fd9）：cli-grammar-v0.6 已并入 master，故 cli-ux-v0.5 全部 10 提交现已在 master 祖先链内——该分支对 master 已无任何独有内容。

### 2.3 内容归属判定

- 该分支即 docs/ssot/specs/cli-ux-redesign 规格集的实现分支（3921ee9 落规格、baa0cb5 落实现、17c2e57/70f7e43 落评审闭环），判定为 CLI UX 重设计正式轮次而非废弃实验——但其产出的「PATH/NAME 位置参数优先」v0.5 文法已被 v0.6 具名文法整体取代：
  1. cli-grammar-v0.6 在其顶点之上用 d6e9ff3 把五个 cmd 文件全部改写为具名 flag 文法（--author/-a、--message/-m 等必选具名参数，owner 裁决），166c4d1 重写的 29 处调用点又被 c31c4cf 重写为 v0.6 文法；
  2. v0.6 分支 smoke 明确「断言旧文法落入 usage 信封 exit 2」（15d0c30），即 v0.5 位置参数文法在 v0.6 语义下是拒绝路径；
  3. adr-v1 已被 d35983c（本分支）与 902fe31（v0.6 分支）两次标注 superseded。
- 不被取代、随 cli-grammar-v0.6 血脉存续的遗产：usage 信封机制（exit 2，v0.6 演进为具名 flag 典范示例版）、ASCII 输出守卫（39156db，v0.6 的 Terry BUG-1 修复延续该线）、SKILL.md、test-v05 语料、v0.5-review 评审书、CHANGELOG 迁移表。
- 结论：内容归属 = cli-ux-redesign 实现轮；文法本体被 cli-grammar-v0.6 完全取代；分支本体被 cli-grammar-v0.6 完全包含（祖先关系），无独有提交。

### 2.4 裁定建议：废弃（合并已完成，进入分支清理，不做独立合并）

三选一中选「废弃」，理由：
1. 独立合并无意义：其全部 10 个提交已随 cli-grammar-v0.6 合入 master（3829fd9，祖先链），单独 merge 为空操作。
2. 保留挂起无收益：无独有内容、无未销账评审项（70f7e43 已标记 BUG-1/BUG-2/W-1..W-3 全部解决）、远端副本与本地一致，历史可随时从 origin 找回。
3. 文档面已闭环：cli-ux-redesign 规格集、评审书、feedbacks、adr superseded 注记均已随合并进入 master 成为历史治理档案，符合 ledger 第四节 NF-1/NF-2/NF-3 已备查的闭合口径。
执行动作（归合并轮清理）：git branch --merged master 确认含 70f7e43 → 移除本地分支 → 移除 origin/cli-ux-v0.5 远端引用（或 owner 决定保留远端存档则仅清本地，二选一由 owner 定，默认建议双清）。

---

## 三、台账联动

- open-items-ledger-2026-08-15.md 追加 LED-13（cli-ux-v0.5 处置）与 LED-14（perfection 闭合批续做），见该文件新增第七节；既有第一至六节未做任何改动。
- LED-01（分支合并）已随 3829fd9 实质闭合，剩余动作为分支清理与 push origin（含 cli-ux-v0.5 处置，见 LED-13）。
- LED-05 状态需在下轮刷新：实测 T1–T4 已落 wip 快照分支、T4 达终态，口径细化为「待按本报告续做清单在合并后 master 上回流」。

---

## 四、风险与边界声明

1. 本报告取证跨两个时点：三方合并进行中（9 个 UU 冲突未决）与合并完成后（master @ 3829fd9，工作区干净）；正文凡涉合并暂态的表述均已用合并后实测复核。
2. wip 快照分支未推 origin，属单机保全；P 批次回流前若发生本机故障将丢失 T1–T4 成果，建议合并轮清理时尽早推送该分支至 origin（本核查未代为执行）。
3. T1 黄金快照重冻工作量取决于 v0.6 信封字面量变化面（新增字段 + 文案刷新），机制零改动、字面量全量再生成。
4. SAM-5（Io 变体移除）属 Rust API 变更，wip 已有实现但未入 master；回流时 CHANGELOG 披露义务同步生效，不得静默并入。
5. 本报告为只读核查产物：未修改任何源代码，未执行任何改变 git 状态的命令；唯一的写入是本报告与 ledger 追加段。

---

## 五、续做清单工单化（修复波派工依据，基线 = 合并后 master @ 3829fd9 / 288 测试）

测试基线构成实测：cli_integration.rs 135 + ops_contacts_crud_tests.rs 18 + ops_tests.rs 59 + core 内联 76（format 五文件 + hash.rs）= 288。

执行顺序注意：§1.5 的 P 编号是清点顺位；实际执行序应为 P-0 → P-1 → P-5（行为锁定先行，作为后续一切重构的门禁）→ P-2 → P-3 → P-4 → P-6 → P-7 → P-8 → P-9。

### P-0 基线重定（文档批）
- 目标文件：docs/reviews/v0.5-perfection-plan-2026-08-15.md（append 增补段，不改原文）、本评估报告
- 改什么：登记基线变更——文法冻结基线 v0.5→v0.6 spec、测试基线 176→288、ops/thread.rs 708 行、ops/lock.rs 127 行、fence/IoContext 位点重扫
- 验收标准：计划文档与 ledger LED-05/LED-14 口径一致；重扫位点清单落盘
- 与 288 测试交互：无

### P-1 锁层融合（冲突 1 落地）
- 目标文件：repos/paperwork-core/src/ops/lock.rs（127 行 SSOT）；参考资料 wip d679b9a 的 ops/lock.rs（251 行 LockedFile RAII 与豁免注记）
- 改什么：评估 io_ctx 收口与 RAII 早退消除能否作为 locked_read_modify_write 的内部重构；可则重构 + differential 对拍，否则 NEW-13 以「核实无需改 + 实测证据」闭合（闭包错误路径已解锁、no-op 跳过已实现）
- 验收标准：master 上只存在一套 lock 实现；六条写路径（post send/edit、profile create/edit、brief add/remove、contacts 写路径）行为不变
- 与 288 测试交互：ops_tests 59、ops_contacts_crud_tests 18 与 dfce66c 锁场景测试全绿；io 信封文案逐字节不变（cli_integration 错误断言敏感）

### P-5 T1 黄金快照重冻（执行序提前至 P-2 之前，重构门禁）
- 目标文件：repos/paperwork-cli/tests/char_tests.rs（wip 版 1071 行，骨架沿用、字面量全量再生成）、repos/paperwork-core/tests/char_tests.rs（wip 版 234 行，format 层 roundtrip 恒等，重冻面小）
- 改什么：把冻结口径从 v0.5 文法换成合并后 master 的 v0.6 具名文法输出
- 冻结对象（命令 × 输出面枚举）：
  1. 五动词全命令面：profile（create/edit/show/list）、post（send/edit/read/summary/list）、brief（add/remove/read）、contacts（add/remove/update/read/list）、validate——每个命令的 default/plain/json/quiet 四种输出模式
  2. 信封路径全覆盖：成功信封、usage 错误信封（exit 2，v0.6 具名 flag 典范示例）、format/validation/io/not-found/already-exists 各类错误信封
  3. v0.6 新增输出面纳入冻结：implicit-mention 字段（仅触发时出现）、showing n/total 与 window 恒显、--entry-title 过滤、contacts remove/update 信封
- 断言粒度：stdout 与 stderr 逐字节全文字面量（LF-only，附「不含 \r」断言）+ exit code；产出文件时间戳一律 (TS) 掩码；fixture 哈希常量（H_MAIN/H_LIB）按 fixture 字节重算；错误信封的 message/fix/example 三段全文冻结（v0.6 具名文法口径）
- 验收标准：重冻后 char_tests 全绿；除 (TS) 外零字节漂移；ASCII 纯度断言保留
- 与 288 测试交互：纯新增不改既有；但一经落盘即成为 P-1/P-4/P-6 重构批的字节级门禁，故必须先于它们

### P-2 T2 护栏移植（重做）
- 目标文件：repos/paperwork-core/src/format/mod.rs（helper 族落点）、ops/{profile,manifest,contacts,thread}.rs（写侧接线）、format/manifest.rs（SAM-1 残留检测）、format/contacts.rs（NEW-5）、新增 repos/paperwork-core/tests/guard_tests.rs
- 改什么：NEW-1 单行字段写侧拒 \n/\r + prose 危险同形行防护（check_single_line / prose_representation_issue / contains_dangerous_attribute_line）；NEW-2 create_new_file 原子创建接 profile/manifest/contacts 三 create；NEW-4 resolve_contact_path 两级解析共享 helper；NEW-5 parse_contacts_title fence 感知；NEW-6 尾扫 fence 奇偶立场测试钉住；SAM-1 brief 残留守卫（## Entries / H3 结构位置特征）；SAM-2 profile create 单次写入；SAM-4 verify_entry 区分 IO 失败与 hash 不匹配
- 验收标准：wip guard_tests.rs（668 行）移植后全绿；守卫双向用例（拒注入不误伤合法多行 prose）；CHANGELOG Unreleased 披露守卫行为新增
- 与 288 测试交互：属行为新增，合法语料路径不受影响；需实测 cli_integration 的坏例语料是否触发新守卫信封（触发则同步冻结进 P-5 快照）；ops_tests 中 create 竞争与 verify 路径直接相关

### P-3 T3 非锁基础设施移植（重做）
- 目标文件：repos/paperwork-core/src/format/mod.rs（wip 版 +532 行）、format/thread.rs（头族正则归族）、format/{manifest,contacts}.rs 与 ops/thread.rs、cmd/validate.rs（扫描器调用点）
- 改什么：for_each_outside_fence / first/collect_outside_fence 扫描器族替换 11 处 fence 状态机中的 8 处行级位点（2 处字节级位点仅共享谓词）；单趟 normalize（Cow 下传，validate 路径消除 3–4 次重复归一化）；dedup_preserve_order、strip_known_suffix 共享 helper
- 验收标准：wip format/mod.rs 内联测试族（扫描器 CRLF / 缩进 / 波浪号围栏 / 断 fence / 孤立 CR 语料）移植后全绿；每处迁移先 differential 对拍后替换旧码
- 与 288 测试交互：format 层内联 76 项最直接相关；validate.rs 归一化次数变化不影响输出但须防警告面漂移（P-5 快照兜底）

### P-4 T4 逐点迁移重做 + SAM-5
- 目标文件：ops/{thread,profile,manifest,contacts}.rs、format/thread.rs、repos/paperwork-core/src/error.rs（L40 Io 死变体）
- 改什么：IoContext 样板逐点切 io_ctx helper（文案逐字保留）；fence 谓词共享（字节级两处保留循环）；移除 PaperworkError::Io 死变体（wip 已有实现；动手前先 grep 全量问号隐式转换依赖并显式化）
- 验收标准：迁移前后 diff 对拍字节一致；SAM-5 落地并在 CHANGELOG 披露 Rust API 变更；288 全绿 + P-5 快照零漂移
- 与 288 测试交互：io 错误信封（message/fix/example）被 cli_integration 与 P-5 快照双重字节级断言，io_ctx 必须逐字保留原措辞；ops_tests 59 项覆盖 RMW 与锁路径

### P-6 T6 CLI JSON 收口 + ensure_suffix 融合（重做）
- 目标文件：repos/paperwork-cli/src/output.rs（JsonBuilder 纯新增）、五个 cmd 文件（合并版调用点）、cmd/mod.rs ensure_suffix
- 改什么：9 处命令侧手工 JSON 改 JsonBuilder（serde_json Map 插入序，键名键序冻结，不用 derive struct）；ensure_suffix 融合 = 分支三级解析语义 + wip OsStr 无损实现（NEW-3）
- 其余 lossy 面：post.rs 第 592-593 行、profile.rs 第 279 行、validate.rs 第 53 行的 to_string_lossy 逐一裁决——路径改写面必修（NEW-3 扩展口径），纯展示面登记留痕
- 验收标准：json 模式输出键名键序零字节漂移（P-5 快照 json 面比对）；wip t6_cli_tests.rs（134 行）移植后全绿；NEW-3 以非 Unicode 路径分量回归闭合
- 与 288 测试交互：cli_integration 的 --json 信封断言（G3 面）字节敏感；validate.rs 重复归一化消除随 P-3 完成

### P-7 T5 拆分与性能批（原样续做，位点重定位）
- 目标文件：ops/thread.rs（合并后 708 行）拆为 thread.rs + thread_read.rs + thread_scan.rs（pub use 保公开 API）；hash.rs（NEW-7 流式 SHA-256、NEW-11 hex 单趟编码）；cmd/post.rs（NEW-12 reply-to 目标尾扫式查询）；ops/thread.rs edit 路径（NEW-8 末条增量重写）
- 改什么：按 §2.5 拆分方案执行（计划原文）；NEW-8 配全重写字节一致对照测试（CRLF / preamble 伪头 / fence 内假头语料）；NEW-10 去重剩余两处接 dedup_preserve_order
- 验收标准：公开 API 签名与 re-export 不变；拆分后 288 全绿 + P-5 快照零漂移；性能项只断言正确性并记录耗时（不设时间阈值，计划 §8.4 被拒方案 5）
- 与 288 测试交互：ops_tests 59 项中 thread send/edit/read/summary 全覆盖；hash 变更影响 brief verify 与 entry hash 断言（cli_integration 中 H 常量面）

### P-8 T7/T8 文档与 CI 批
- 目标文件：新增 BDD 差分表文档（对齐 docs/ssot/specs/cli-grammar-v0.6/bdd.md 与 format-v2 79 场景）、README 测试计数、.github/workflows/ci.yml、CHANGELOG.md
- 改什么：G1–G5 超集测试缺口闭合核对（多数已随 v0.6 轮的 tdd 1b/4 用例与 contacts CRUD 测试覆盖，逐项核实后二选一闭合）；ci.yml 并入 cargo doc --no-deps 与 cargo test --locked；CHANGELOG Unreleased 汇总披露（守卫行为新增、SAM-5 Rust API 变更、Known downgrade 复核）
- 验收标准：差分表落盘且无未映射场景；README 计数 = 实测；两门禁在 CI 三平台矩阵通过
- 与 288 测试交互：仅计数与文档同步，不改行为；--locked 门禁以合并后 Cargo.lock 为准

### P-9 T9/T10/T11 门禁与收口批
- 目标文件：全仓（终验）；git（提交推送）
- 改什么：独立 Verify 复刻评审书十三项 QA + 黄金快照（P-5 重冻版）总比对 + cargo clean 后 clippy + fmt --check + release CLI 实证 + B1 SHA256 零字节复验；三路 CodeReview 至零 Critical/Major；按归属分批提交（修复/测试/文档）并推送
- 验收标准：T9 总门禁全绿；T10 三份评审报告落盘销账；T11 提交不含 v0.6 工作流文件混入
- 与 288 测试交互：终验口径 = 288 + 各 P 批累计新增，计数须与 README/P-8 一致

（报告完。核查人：任务 #31 只读核查 agent；取证时间 2026-08-15；全部结论基于 git 对象与磁盘文件实测，未修改任何源代码，未执行任何改变 git 状态的命令；唯一写入为本报告与 ledger 追加段。）
