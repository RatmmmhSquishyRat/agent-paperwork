# v0.5-perfection 续做清单执行日志（P-0 ~ P-9）

- 日期：2026-08-15
- 执行人：任务 #32 实施 agent（Felix）
- 权威依据：`docs/dev/perfection-and-branches-assessment-2026-08-15.md` 第五节工单
- 起点：master `3829fd9`（合并定稿后）
- 设计素材：本地分支 `wip/v0.5-perfection-snapshot-2026-08-15`（HEAD d679b9a，只读）+ 悬空提交链 `0842522 → 47958aa → a9b68a0 → ecc2c23`（P-7 素材）
- 纪律：每批原子提交带批次号；每批后 `cargo test --workspace` + clippy `-D warnings` + `fmt --check`；**不推送**（任务书覆盖工单的推送要求）
- 终态：**397 测试全绿**（6 + 31 + 136 + 4 + 96 + 12 + 23 + 18 + 71 + 0）；clippy 零警告；fmt 零漂移；cargo doc 零警告

---

## 提交链（3829fd9..HEAD，按时间序）

| 提交 | 批次 | 摘要 |
|---|---|---|
| a941b3b | P-0 | 计划文档增补段：在合并后 master 上重定基线（v0.6 文法、288 测试、位点重扫） |
| c62df0e | P-1 | 锁层融合裁定落盘：`locked_read_modify_write` 保持 SSOT，NEW-13 以证据闭合 |
| 7e2dc16 | P-5 | T1 黄金快照按 v0.6 文法重冻（cli 30 测试 / 140 冻结条目 + core 12 roundtrip） |
| 243207e | P-5 | .gitattributes 钉 eol=lf（保护字节精确黄金测试） |
| 669befa | P-2 | T2 写侧护栏移植（NEW-1/NEW-2/NEW-4/NEW-5/NEW-6、SAM-1/2/4）+ guard_tests.rs 23 测试 |
| 6d5caeb | P-1 | 锁层回归测试 rustfmt reflow（提交后 fmt 清扫） |
| bed9db0 | P-3 | T3 非锁基建移植：scanner 族迁移、单趟 normalize、dedup_preserve_order / strip_known_suffix 接线、头正则归族、T4 差分语料 |
| ce71fbc | P-4 | SAM-5：38 处 IoContext 样板迁至 io_ctx helper（措辞逐字）、移除死变体 `PaperworkError::Io`、信封对等测试、CHANGELOG Rust API 披露 |
| 613734e | P-6 | T6 CLI JSON 收口：JsonBuilder 单构造路径（9 处调用点，输出字节冻结）、ensure_suffix OsStr 无损三级融合（NEW-3）、default_title OsStr 化、contacts read 富化 resolve_contact_path（NEW-4）、t6_cli_tests.rs 4 测试、CHANGELOG 披露 |
| 7132e19 | P-7 | T5 拆分：ops/thread.rs → thread/thread_read/thread_scan（re-export 面不变；master 锁立场 + P-3 scanner 语义保留） |
| 91061e6 | P-7 | NEW-8：thread_edit 末条增量重写（规范前缀等价探针 + 全量回退；差分语料钉住） |
| 1576e3a | P-7 | NEW-7 流式 SHA-256（BufReader 64KB）+ NEW-11 单趟 hex（nibble LUT；io 信封措辞保留 master 逐字版） |
| db71803 | P-7 | NEW-12：post send --reply-to 尾扫 sender 查询（missing seq 静默不变；gold 快照新增 3 条） |
| b05c3ca | P-7 | NEW-10 收口：最后一处内联 mention 去重接入 dedup_preserve_order |
| e9c0a41 | P-8 | BDD 差分表落盘、ci.yml `--locked` + `cargo doc --no-deps` 门禁、rustdoc 9 警告清零、CHANGELOG P-7/P-8 披露 |

## 各批次明细

### P-0 基线重定（文档批）
- 做了什么：在 `docs/reviews/v0.5-perfection-plan-2026-08-15.md` append 增补段，将计划基线从 176 测试/旧文法重定为合并后 288 测试/v0.6 文法，位点全部重扫。
- 测试：288 全绿（基线）。

### P-1 锁层融合裁定
- 做了什么：以证据闭合 NEW-13（closure-error unlock + no-op skip 钉住），裁定 master `locked_read_modify_write` / 手动 fs2 锁为 SSOT；wip 的 `LockedFile` RAII **不采纳**（后续所有移植均改写为手动 lock/unlock 形态）。
- 测试：288 全绿。

### P-5 行为锁定先行（T1 黄金快照重冻）
- 做了什么：按 v0.6 文法重冻 char_tests 黄金快照（cli 30 测试 / 140 冻结条目）与 core 12 roundtrip 测试；`.gitattributes` 钉 `eol=lf`。此批作为后续一切重构的字节级门禁。
- 测试：重冻后全绿（288 → 基线口径不变，char 覆盖扩面）。

### P-2 写侧护栏（T2）
- 做了什么：NEW-1 单行字段换行注入拒绝、NEW-2 create_new 原子创建、NEW-4/5/6、SAM-1/2/4；新增 guard_tests.rs（23 测试）。
- 测试：+23 → 311 全绿。

### P-3 非锁基建（T3）
- 做了什么：fence scanner 族 8 处行级迁移、单趟 normalize（Cow 传递）、`dedup_preserve_order` 接 participants/derive_mentions、`strip_known_suffix` 接线、头正则归族（LEGACY_HEADER_RE_FMT）、T4 差分语料 + normalize 等价测试。
- 测试：全绿（约 330 口径）。

### P-4 SAM-5 Rust API 收口
- 做了什么：38 处 IoContext 样板迁至 `io_ctx` helper（措辞逐字保留）；移除死变体 `PaperworkError::Io`；移植 io_ctx 信封对等测试；CHANGELOG Unreleased 披露 Breaking（仅 Rust 直调面）。
- 测试：全绿。

### P-6 T6 CLI JSON 收口
- 做了什么：output.rs JsonBuilder（insert/insert_opt/build）接 9 处命令侧调用点（协议层 2 处保留），输出字节由 char gold 冻结；ensure_suffix 三级语义 + OsStr 无损融合（NEW-3，双平台非 Unicode 回归测试）；default_title 改 OsStr 原生剥后缀；contacts read 富化走 resolve_contact_path（NEW-4）；wip t6_cli_tests.rs 按 v0.6 文法移植 4 测试；CHANGELOG 披露。
- 测试：377 全绿。

### P-7 T5 拆分与性能批（素材 = 悬空提交链）
- 做了什么（四个子提交 + NEW-10 收口）：
  1. T5 拆分（0842522 cherry-pick 适配）：thread.rs 三分，re-export 面不变；编译修复（去 LockedFile、import 合并、模块文档改 P-1 立场）。
  2. NEW-8（47958aa 带入）：edit 末条增量重写；等价探针只在磁盘区与规范序列化字节相等时 truncate+append，否则全量回退；差分语料 8 测试（CRLF / preamble 伪头 / fence 内假头）钉住两路字节一致。
  3. NEW-7/NEW-11（a9b68a0 cherry-pick）：流式 SHA-256 + 单趟 hex；新增 4 测试（全字节域 / chunk 边界 / 空文件 / 缺失文件）；io 信封措辞保留 master 完整版（含 example）。
  4. NEW-12（ecc2c23 cherry-pick，三冲突手工解决）：reply-to sender 查询改有界尾扫 `find_message_sender`（手动锁形态）；post.rs 保留 implicit_mention 冻结字段；char 测试按 v0.6 文法重写并录 3 条新 gold；`contains_legacy_headers` 改用 master `for_each_outside_fence`。
  5. NEW-10 收口：post.rs mention 去重循环接入 `dedup_preserve_order`（最后一处内联 O(n²) 位点；participants/derive_mentions 已在 P-3 完成）。
- 测试：377 → 396 全绿（core lib 96、ops_tests 71、char 31）。
- **Sam-m-γ 不采纳**：default_title 保持 P-6 OsStr 原生形态（ecc2c23 附带的 strip_known_suffix 接线与 P-6 裁定冲突）。

### P-8 T7/T8 文档与 CI 批
- 做了什么：
  - BDD 差分表 `docs/dev/bdd-scenario-test-map-2026-08-15.md`：v0.6 bdd.md 全部 S-* 场景 + format-v2 全部 79 场景逐条映射，无未映射场景。
  - 差分核对发现的唯一缺口：S-EDIT-08（v0.5 edit 位置文法）补 `edit_v05_grammar_positional_is_usage`。
  - ci.yml：test 改 `cargo test --locked --workspace`，新增 `cargo doc --no-deps --workspace` 门禁；顺带清零 9 处 rustdoc 警告（私有 intra-doc 链接 5 处 + `<verb>` 未闭合 HTML 标签 4 处）。
  - CHANGELOG Unreleased：P-7 core-internals 段 + P-8 CI 段。
  - README 测试计数：实测 README/子仓 README 均无硬编码计数，无需同步（实证关闭）。
- 测试：397 全绿（cli_integration 135 → 136）。

### P-9 终验门禁与收口
- cargo clean（1.8GiB）后冷重建：`fmt --check` 零漂移、clippy `--locked -D warnings` 零警告、`cargo test --locked --workspace` 397 全绿。
- release 实证（target/release/paperwork.exe，临时目录全流程）：post send/read/summary（含 NEW-12 implicit-mention 面）、brief create/add/read --full/verify、contacts create/add/read、profile create、validate 双文件、--json summary 单行信封 —— 全部符合冻结口径。
- **B1 SHA256 零字节复验**：空文件 brief add 落盘 hash = `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`（零字节 SHA-256 标准值），逐字一致。
- 三路评审自查：本批全部代码逐批过 test/clippy/fmt 三门禁 + CR 字节扫描（零 CR 字节、零冲突标记残留）；无 Critical/Major 残留。
- 提交纪律：全部按归属分批（15 个提交），不含他人未提交改动混入（README.md/SKILL.md/docs/ssot/* 等 M 项与审计 agent 的 ?? 文档全程未触碰）。

---

## 采纳 / 改写 / 放弃清单

### 采纳（wip 素材按原意移植）
- T2 写侧护栏全家（NEW-1/2/4/5/6、SAM-1/2/4）
- T3 非锁基建（scanner 族、单趟 normalize、dedup helper、strip_known_suffix、头正则归族）
- SAM-5（IoContext 迁移 + Io 变体移除）
- T6 JsonBuilder + t6_cli_tests（v0.6 文法适配）
- T5 三分拆分（re-export 保 API）
- NEW-7 / NEW-8 / NEW-10 / NEW-11 / NEW-12（性能五件）

### 改写（按 master 实况适配后移植）
- 所有锁相关代码：wip `LockedFile` RAII → master 手动 `lock_exclusive`/`unlock`（P-1 裁定）
- thread_read.rs / thread_scan.rs 新文件的 io_ctx 文案：保留 wip 简写（io_ctx 签名 `impl Into<PathBuf>`/`impl Into<String>`，行为等价）；hash.rs 保留 master 完整 wording（含 example）
- NEW-12 的 char 测试：按 v0.6 文法（`--author alice --message "Hello"`）重写；`contains_legacy_headers` 改 master `for_each_outside_fence`（master 无 `first_outside_fence`）
- implicit_mention：wip NEW-12 删除了该字段，移植时保留（P-5 快照冻结的 v0.6 特性）
- ensure_suffix：分支三级解析语义 + wip OsStr 无损实现融合（NEW-3 闭合）

### 放弃（不采纳及理由）
- `LockedFile` RAII：与 master 锁 SSOT 裁定冲突（P-1）
- Sam-m-γ（default_title 接 strip_known_suffix）：P-6 已将 default_title 改为 OsStr 原生剥后缀，wip 方案是回退
- wip `--from` 旧文法示例：v0.6 文法已冻结
- wip io_ctx 简写覆盖 hash.rs：master 完整 wording（含 example）信息量更高且已被 P-4 逐字钉住

## 偏差登记
1. default_title：P-3 曾按 wip 接 `strip_known_suffix`，P-6 被 OsStr 原生版替换（Sam-m-γ 因此整体不采纳）。
2. thread_read/thread_scan 新文件保留 wip 简写 io_ctx 文案（行为等价，信封字节一致，由 io_ctx 对等测试钉住）。
3. implicit_mention 保留 = 对 wip NEW-12 的行为适配（非偏差于本仓契约，偏差于 wip 素材）。
4. P-7 计划基线行号全部失效，按合并后代码重新定位（工单预期内）。

## 未闭合残留
- 代码面：无。P-0~P-9 全部闭合，397 全绿，三门禁零警告。
- 文档面：本日志与 BDD 差分表落盘后，本任务文档义务完成；`docs/dev/open-items-ledger-2026-08-15.md` 由任务 #22 负责，本批未改（禁区）。
- git 面：**未推送**（任务书纪律）；wip/v0.5-perfection-snapshot-2026-08-15 分支成果已全部回流，处置由 leader 决定。
- 工作区残留 M/?? 项均为他人/审计 agent 的未提交改动，按禁区纪律未触碰。

（日志完。执行人：Felix；全部结论基于 git 对象与磁盘实测。）
