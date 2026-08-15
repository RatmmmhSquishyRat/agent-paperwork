# 审计修复波 Round2 执行日志（任务 #45，2026-08-15）

- 执行者：任务 #45 执行 agent（Jason）
- 起始基线：master @ 46b1f47，444 测试全绿
- 依据四份审计报告（随本提交入库）：
  - docs/dev/repo-state-inventory-2026-08-15.md（INV-01~18）
  - docs/dev/audit-grammar-matrix-round2-2026-08-15.md（G2-O1/O2）
  - docs/dev/audit-robustness-round2-2026-08-15.md（R2-01 + B-x 盲区）
  - docs/dev/audit-ssot-round2-2026-08-15.md（S2-01~04）
- 纪律遵守：每批原子提交；输出协议只增不改（R2-01 仅改 fix 文案，exit code 与 category 不变）；0.5.0 不 bump/tag/发布；禁区（wt-v05perfection worktree 与 wip/v0.5-perfection-snapshot-2026-08-15 分支）全程零触碰。

## 一、F1：代码与脚本缺陷修复（提交 54beff3）

### S2-01（重要）：smoke.ps1 残留已撤销糖标志
- `_e2e/smoke.ps1` L38 的 `--reply-to 1 --mention alice` 改为正文直书形态 `-m "@#1 Tests merged. cc @alice"`（含裁决注释）。
- `_e2e/` 被 gitignore 覆盖（`_*/` 规则），该修复不入仓，属本地资产修正；此处登记为证据。

### FR-1 全仓扫查结果（零残留结论）
扫查面：`.github/workflows/*.yml`、`SKILL.md`、`README.md`、`_e2e/*`、docs 教学面，关键字 `--reply-to`/`--mention`。
- `.github/workflows/ci.yml`：仅含裁决说明注释与正文直书形态 smoke 命令，无写侧糖标志调用。
- `SKILL.md` / 根与 cli `README.md`：命中均为读侧过滤器（`post read --reply-to/--mention` 过滤语义，现行保留面）与撤销声明文本，人工甄别后保留。
- `docs/ssot/specs/cli-ux-redesign/`：命中均位于已标记 superseded 的历史档案，不回改。
- `docs/ssot/specs/cli-grammar-v0.6/`：命中为读侧过滤器场景与撤销登记/负向场景。
- 结论：写侧糖标志教学面零残留；read 侧过滤器与撤销教学引用按任务书豁免条款人工甄别保留。

### R2-01（低危缺陷）：文件通道编码指向文案
- `paperwork-core/src/error.rs` 新增 `pub const FILE_NOT_UTF8_FIX`（逐字："the file is not valid UTF-8; check that the file is UTF-8 encoded (binary and UTF-16 files are not supported)"）与 `pub(crate) fn io_ctx_file_read(...)`：`source.kind() == InvalidData` 时用编码指向文案，否则沿用调用方默认文案；category 维持 io、exit 1 不变。
- core 侧 10 处文件读通道调用点迁移至 `io_ctx_file_read`（thread_read ×3、thread edit、thread_scan ×2、lock、contacts、profile、manifest）。
- CLI 侧 2 处（post.rs reject_foreign_thread、validate.rs）因 `io_ctx_file_read` 为 `pub(crate)`，内联 InvalidData 判断并引用共享常量。
- 与 stdin 通道 D6 口径对齐（D6 落 validation 信封；文件通道按审计建议维持 io category 仅改 fix 文案）。
- 新增单测 `test_io_ctx_file_read_encoding_hint`（error.rs）。

### F1 门禁
- cargo test --workspace --locked：445 全绿（444 + error.rs 新单测 1）。
- clippy -D warnings 零警告；fmt --check 通过。

## 二、F2：盲区测试钉住（提交 0ffd9d2）

cli_integration.rs 追加 6 个测试（均带探针编号注释）：
- B-1：`bom_prefixed_thread_is_tolerated_on_read_and_validate`——BOM 前缀文件读/validate 宽容放行。
- B-2 ×2：`utf16_file_read_fast_fails_with_encoding_pointing_fix`（UTF-16 fast-fail + R2-01 新文案逐字断言 + JSON category=io + 零写入）、`binary_file_read_fast_fails_with_encoding_pointing_fix`（二进制伴侣用例）。
- B-5：`reserved_device_names_are_sealed_by_suffix_normalization`——CON/NUL 保留名经 ensure_suffix 归一为 CON.post.md/NUL.post.md（30s timeout 防设备挂起；validate 裸名走 unknown file type）。
- B-6：`large_thread_2500_messages_send_read_roundtrip`——2500 消息线程 send/read 往返完整性（showing 20/2500、seq 2501、head+tail 读回），实测秒级内。
- B-8：`h1_leniency_missing_and_duplicate_h1_read_cleanly`——H1 宽容面（缺 H1/双 H1 读侧与 validate 宽容放行）。

B-8 语义裁定注记（不改行为，仅钉住）：
- spec.md §3.3/§3.7、design.md §8、bdd.md 新增 S-READ-10 场景登记。
- 裁定口径：H1 preamble 读侧/validate 侧非强制；写侧首写仍写 H1；读写不对称为刻意保留。

### F2 门禁
- cargo test --workspace --locked：451 全绿（445 + cli_integration 新测试 6）。
- clippy -D warnings 零警告；fmt --check 通过。

## 三、F3：文档债清零（提交 0dc23f0）

| 编号 | 事项 | 处置 |
|---|---|---|
| INV-01 / LED-07 | closure 报告「共 31 条」笔误 | 文末追加勘误节（实表 28 行，以 O-2 口径为准；按 reviews 档案纪律不回改原文） |
| INV-02 / LED-08 | research 写路径计数口径 | 4.1 节与 §6 后追加口径注明（调研时点五写路径 → 实施落地六写路径） |
| INV-03 / LED-06 | 台账状态未刷新 | 台账第十六节确认闭合销账（tdd L107 勘误证据在案） |
| INV-04 | workflow-and-todo §三/§四 过期 | 追加第五节修订节（append-only，刷新「仍开放 8 项」与 LED-09~12 口径、基线 451） |
| S2-02 | spec README 治理清单 | 勾选任务 #36 裁决批（提交链 9821933→f94b65f，444 全绿承载） |
| S2-03 | format-v2 spec 裁决指针 | §5.7 与 OQ-4 补 2026-08-15 裁决指针注记（撤销写侧糖标志，正文直书 + 读侧派生） |
| S2-04 | role 文档归档标注 | cli-grammar-v0.6 与 cli-ux-redesign 两份 implementer role 头部加历史归档声明 |
| G2-O1/O2、A-01~A-04 | 观察/延续项登记 | 台账第十六节 LED-18~23（不改行为） |

台账第十六节另含 INV-08/09 裁定保留登记与零开放终态声明（登记总数 23，仍开放项为零；非闭合保留项仅 LED-15/16/19 及 LED-18/20~23 登记项）。

## 四、F4：卫生与归档（本提交）

| 编号 | 事项 | 处置与证据 |
|---|---|---|
| INV-05 | cli-grammar-v0.6 分支清理 | 核验：`git branch --merged master` 含该分支、`rev-list --left-right --count master...origin/cli-grammar-v0.6` = 51/0（零独有）；删除本地（was a7bc3e2）与远端（push origin --delete 成功） |
| INV-06 | _verify_tmp40 残留 | 执行时实测已不存在（Test-Path=False），如实登记：声明与现场已相符，无需动作 |
| INV-07 | _wip_stage 目录 | 先核验 wip 保全：wip/v0.5-perfection-snapshot-2026-08-15 本地与 origin 双侧在案（9d63d3b），且 C 类在制品提交 d29fb75 经 `git branch --contains` 确认可达于该分支；确认后删除 _wip_stage（含二进制） |
| INV-08 | _fix/ 目录 | 裁定保留为历史证据链资产（fix-ledger 证据链直接引用），不入库；登记于台账第十六节 |
| INV-09 | _master_lock.rs/_wip_lock.rs | 裁定保留为机器本地态实验残留，不删除、不入库；登记于台账第十六节 |
| 报告入库 | 四份审计报告 + 本日志 | 随本提交入库（此前为未跟踪文件） |

禁区核验：wip/v0.5-perfection-snapshot-2026-08-15（9d63d3b）本地与 origin 全程未触碰；wt-v05perfection worktree 零触碰。

## 五、终局门禁（F4 前实测）

- `cargo test --workspace --locked`：**451 全绿**（分布：cli doc 7 + cli char_tests 33 + cli_integration 154 + ivy_gap 16 + t6_cli 4 + core unit 103 + core char_tests 12 + guard_tests 33 + ops_contacts_crud 18 + ops_tests 71 + 0）。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`：零警告。
- `cargo fmt --all --check`：通过。
- 版本纪律：paperwork 0.5.0，未 bump、未打 tag、未新增 CHANGELOG 发布段。

## 六、提交链

| 批次 | 提交 | 内容 |
|---|---|---|
| F1 | 54beff3 | S2-01 smoke 修复登记 + R2-01 编码文案修复（445 基线） |
| F2 | 0ffd9d2 | 六盲区测试钉住 + B-8 三文档注记（451 基线） |
| F3 | 0dc23f0 | 文档债清零八项 + 台账第十六节 |
| F4 | 本提交 | 四报告 + 本日志入库 |

（执行日志完。任务 #45 执行 agent，2026-08-15。）

---

## 七、修复轮二登记（2026-08-15 追加，任务 #46 复验阻塞闭环，append-only，未改动第一至六节）

### 阻塞事实

- 任务 #46 复验不放行，唯一阻塞项：F1 批在 `repos/paperwork-core/src/error.rs` 新增的公有常量 `FILE_NOT_UTF8_FIX` 文档注释中，intra-doc link `[`PaperworkError::io_ctx_file_read`]` 指向 `pub(crate)` 私项。
- 后果：docs gate（`RUSTDOCFLAGS="-D warnings" cargo doc`）报「public documentation links to private item」硬错误；线上 CI run 31883911085 test×3 全红。

### 根因

- 修复波终局门禁清单缺 docs gate：本日志第五节门禁仅含 test + clippy + fmt 三项，未含 `RUSTDOCFLAGS="-D warnings" cargo doc`；而 CI 的 docs 面是独立 gate，本地三项全绿拦不住 rustdoc 硬错误。

### 修复内容（提交 86776db）

- error.rs `FILE_NOT_UTF8_FIX` 文档注释改写：去掉对私项的 intra-doc link，改为纯文本表述（`io_ctx_file_read`，crate-internal），文案语义不变，常量本身与行为零变更。
- 全仓扫查：公有项文档中无其他指向私项的 intra-doc link；thread_scan.rs 模块头 `//!` 中 5 处 `[`...`]` 链接指向 `pub(super)` 项，属私有模块内部文档，rustdoc 实测零告警，无需处置。

### 四门禁实测（修复后全绿）

| 门禁 | 命令 | 结果 |
|---|---|---|
| docs gate | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` | exit 0，零警告 |
| test | `cargo test --workspace --locked` | **451 全绿**，0 failed |
| clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 零警告 exit 0 |
| fmt | `cargo fmt --all --check` | 通过 exit 0 |

### 防复发规则（固化，规则号 FR-2）

- **FR-2（修复波终局门禁四件套）**：凡修复波/紧急修复轮的终局门禁必须同时包含四项：`cargo test --workspace --locked` + `cargo clippy --workspace --all-targets --locked -- -D warnings` + `cargo fmt --all --check` + `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`；四项全绿方可放行推送。缺任一项视为门禁不成立。
- 教训登记：修复波终局门禁遗漏 docs gate 导致本地全绿但线上 CI 红，复验轮拦截后紧急修复；后续批次引用本节 FR-2 即可。

（修复轮二登记完。追加：任务 #46 阻塞闭环执行 agent，2026-08-15。修复提交 86776db，本节随 docs 登记提交入库。）

---

## 八、收口轮更正与补记（2026-08-15 追加，任务 #47 三维评审发现闭环；append-only，未改动第一至七节）

### S2-02 提交链引用更正（Adam 重要-2）

- 本日志第三节 F3 表 S2-02 行「勾选任务 #36 裁决批（提交链 9821933→f94b65f，444 全绿承载）」表述有误，更正为：任务 #36 裁决批实施链为**六提交 9821933/14f3b57/77f19e2/6a36639/72c85ac/b9b059c**（链端 b9b059c，台账第十四节 SSOT 记录在案，审计 S2-02 出处 B 原文亦为「9821933..b9b059c」）。
- f94b65f 为任务 #52 Plan-C 回填批提交（commit message "fix: backport residual Ultra Review increments (Plan-C, wip …)"），其 ci.yml smoke 修正归属在 fix-ledger CI-F1 与 ci-failure-diagnosis 中均已明确记为「修复者 = 任务 #52 回填批，与裁决批无因果关联」。
- 同步更正落点：spec README 勾选项（就地更正）、workflow-and-todo §5.4（追加注记）、台账第十七节勘误登记。

### 三维评审 9 项发现处置指针

- 9 项发现（Adam 重要-1/重要-2/低-3/低-4、Kevin W-1/S-1/S-2、Evan S-1/S-2/S-3）处置与销账详见台账第十七节与本轮提交；B-3 以新测试钉住闭合（cli_integration `emoji_and_combining_chars_roundtrip_without_normalization`，基线 451→452），B-4/B-7 登记为开放盲区项（LED-24/25）；复验报告文末已追加闭合注记（放行改判）。

（第八节完。追加：任务 #47 执行 agent，2026-08-15。）
