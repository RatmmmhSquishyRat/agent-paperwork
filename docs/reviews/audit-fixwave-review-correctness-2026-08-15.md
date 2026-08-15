# 修复波三维评审——正确性视角报告

- 评审对象: `git diff 3829fd9..da954c2`（origin/master → HEAD），重点 repos/paperwork-core 与 repos/paperwork-cli 的 .rs 改动（约 6276 行新增，28 文件）
- 评审维度: 仅正确性（logic/security bugs）；需求覆盖完整性与回归影响面另有评审员负责
- 方法: 通读全部 .rs 改动；cargo test --workspace --locked 全绿；独立探针项目 target/_review_probe（TEMP 夹具，未入库）
- 日期: 2026-08-15

## 一、阻塞（MUST FIX）

### C-1 SAM-1 legacy residue 护栏与 brief 写路径构成写→读闭环断裂（文件永久不可读）

**位置**:
- 读侧护栏: repos/paperwork-core/src/format/manifest.rs L72-L79 contains_legacy_brief_residue，L89-L95 由 parse_manifest 调用
- 写侧护栏缺口: repos/paperwork-core/src/ops/manifest.rs L109-L118 note 护栏仅用 note_representation_issue（只查首个非空行，format/manifest.rs L42-L55）；entry title 派生 L138-L141 无护栏
- 序列化: format/manifest.rs L325-L327 serialize_entry 将 note 裸写入（无 fence 包裹）

**问题**: 读侧 SAM-1 护栏拒绝全文任意 fence 外的 `### ` 行与任意 `## Entries` 行，而写侧不保证写入内容不含此类行。两条触发路径：

- 路径 A: note 任一行（含首行）包含 `### `。写侧护栏只检查首个非空行，故校验通过、写入成功；note 裸序列化无 fence，下一次 parse_manifest 将该行判为 legacy residue，返回 Parse error。
- 路径 B: entry 路径文件名为 `Entries`（如 --entry notes/Entries）→ title 派生为 Entries → 序列化为 `## Entries` 标题行 → 同样命中护栏。

**后果**: 写入返回 Ok，但该 brief 之后所有依赖 parse_manifest 的命令（brief read/add/remove/verify）全部失败——文件级永久 lockout，且无 CLI 自救手段（remove 本身也要先解析该文件）。

**探针复现**（target/_review_probe 实际执行输出）:

    P1  add(note 非首行含 "### x"): Ok -> re-read FAIL category=format
    P1b add(note 首行为 "### x"):   Ok -> re-read FAIL
    P2  add(entry 文件名 Entries):  Ok -> re-read FAIL
    msg: Parse error: brief contains legacy v0.4 residue
         (## Entries wrapper heading or ### entry headers)

**本 diff 引入判定**: contains_legacy_brief_residue 及其在 parse_manifest 的调用为本 diff 新增；写侧仅首行护栏与 title 派生为既有逻辑，但写→读闭环由读侧护栏的引入而断裂，故记为本 diff 引入的缺陷。

**修复方向**（二选一）:
1. 补齐写侧: note 护栏扩展为全文拒绝 fence 外 `### ` 行与 `## Entries` 行（复用 first_outside_fence，与读侧语义对齐）；同时拒绝序列化后命中 residue 模式的 entry title。
2. 或放宽序列化: 对 note 做 fence 包裹，使 residue 护栏对工具自写内容恒不误报。
3. 任一方案均须补 roundtrip 回归测试: add(note 含 ### ) → read 与 add(entry Entries) → read。

## 二、重要（SHOULD FIX）

无。

## 三、低（CONSIDER）

### L-1 D3 标题行护栏对 "#hashtag" 类无害行误杀

**位置**: repos/paperwork-core/src/format/mod.rs L438-L441 contains_heading_line。

**问题**: 护栏采用 trim_start().starts_with(#) 且故意非 fence-aware。探针 P4 确认 "#hashtag in prose"、首行后接 "### ..."、"#define FOO 1" 等正文行均被 validation 拒绝；而 profile 解析中这些行实际无害地落入 description（见 format/profile.rs 解析路径），构成误杀。doc 注释声明此为故意保守策略，故记为低。

**修复方向**: 若收窄，可仅拒绝 fence 外且符合 CommonMark 标题结构（#~###### 后随空白）的行；若维持保守则保留现状。

### L-2 R7 尾扫丢首行规则在 CRLF 文件上理论可误丢一整行

**位置**: repos/paperwork-core/src/ops/thread_scan.rs L223-L240 read_tail_scan_buffer。

**问题**: R7 以 prev[0] != \n 判定丢弃 buffer 首行。若手工维护的 CRLF thread 文件的 buffer 起点恰好落在 \r\n 两字节之间（prev 字节为 \r），buffer 开头一条完整行会被误丢。触发条件苛刻：工具自身只写 LF，需手工 CRLF 且文件长度恰对齐 64KB+256B 窗口边界，故记为理论低危。

**修复方向**: 将 R7 判定放宽为 prev 字节为 \n 或 \r 均视为行边界。

### 备注（既有问题，非本 diff 引入）

- 探针 P3: note 含 "## forged entry" 行会被解析静默分裂出第二个 path 为空的 entry。note 护栏非本 diff 改动，属既有问题；可与 C-1 修复合并解决（全文 fence 外标题行护栏）。

## 四、已验证安全点清单（评审要点中可疑点逐一排除）

1. **NEW-12 expect 不可达**: ops/thread_scan.rs L329-L331 expect("header_seq matched MESSAGE_HEADER_RE")——header_seq 内部即以 MESSAGE_HEADER_RE.captures(line)? 门控，同一正则二次匹配恒成功，expect 永不触发。
2. **normalize_line_endings expect 安全**: format/mod.rs 单遍重写仅替换 ASCII \r 行尾字节，其余字节原样搬运，UTF-8 不变，String::from_utf8 expect 不可能失败。
3. **NEW-8 增量重写探针正确**: ops/thread.rs thread_edit 以 serialize_messages(prefix) 与文件前缀做字节等价探针；探针失败（CRLF/非规范头/preamble 不符）即 fallback 全量重写，两路径产出字节一致；单消息时 prefix 为空串，空串恒等于任何前缀起点，探针正确退化为 offset=preamble_end。
4. **NEW-7 流式 hash 与 NEW-11 hex_encode 字节等价**: hash.rs 测试 test_hex_encode_full_byte_range（0-255 全字节域 LUT）与 test_hash_file_matches_hash_bytes（chunk 边界/+1/>1MB）已固定等价性。

5. **锁内读改写六步模板保持**: ops/lock.rs locked_read_modify_write 以同一持锁句柄完成读与改写；全部错误路径（含闭包内错误）先 unlock 再返回；no-op 跳过重写。测试 closure_error_path_releases_lock 以 try_lock_exclusive 证明闭包出错后锁已释放。锁内无外部 I/O：brief_add_entry 的 hash_file 已移到锁外（ops/manifest.rs L120-L132），derive_label 亦在锁外。
6. **create_new 原子性**: ops/mod.rs create_new_file 以 OpenOptions create_new(true) 单内核操作完成存在检查+创建（NEW-2 TOCTOU 闭合）；AlreadyExists 与 Windows ERROR_FILE_EXISTS 均映射至调用方闭包信封。
7. **ensure_suffix OsStr 无损**: cli/cmd/mod.rs suffixed_variant/os_strip_suffix 全程 OsStr 原生操作（Unix as_bytes/from_vec、Windows encode_wide/from_wide），无 lossy roundtrip；测试覆盖 0xFF 非法 UTF-8 字节与 0xD800 孤立代理项。路径处理无新增穿越面。
8. **D2 fence 平衡预检边界**: ops/thread.rs thread_send 预检对嵌套/多段 fence 采用 CommonMark 长度规则（关闭 fence 长度 ≥ 开启），行内反引号因非行首 backtick run 不被判为 fence，无误判；空文件以 !file_empty 豁免，正确。

9. **D6 stdin 编码信封**: cli/cmd/post.rs resolve_body 将 InvalidData 映射为 Validation 信封（stdin is not valid UTF-8），category/fix/example 字段完整。
10. **exit code 分层**: cli/output.rs emit_err exit_code=1、emit_usage_error exit_code=2；JSON 模式下 exit_code 字段与进程实际退出码一致。
11. **注入净化无新旁路**: check_single_line 同时拒绝 \n 与 \r（覆盖 CRLF/CR 拆行）；D4 scope glob 单行校验生效。探针 P5: CRLF body 的 thread send 正常落为 seq 2。
12. **全量测试绿**: cargo test --workspace --locked 全部 crate 0 failed。

## 五、正确性维度结论

本次改动区间（3829fd9..da954c2）发现 **1 个阻塞**：SAM-1 读侧 residue 护栏的引入断裂了 brief 的写→读闭环（note 含 `### ` 行、entry 文件名 `Entries` 两条触发路径均已探针复现），触发后文件永久不可读且无 CLI 自救手段，建议发布前修复并补 roundtrip 回归测试。**重要问题无**。低级 2 项（D3 "#hashtag" 误杀、R7 CRLF 理论边界）与 1 项既有备注（note 含 `## ` 静默分裂，可与 C-1 合并修复）。

并发/锁、原子创建、hash 等价性、NEW-8 增量重写、OsStr 路径处理、错误信封字段完整性与 exit code 分层，均已逐项验证正确，未见本 diff 引入的正确性缺陷。

---

## 六、销账段（修复轮二，2026-08-15 追加）

本报告全部发现逐项销账（明细与证据链见 docs/dev/fix-ledger-2026-08-15.md 第六节）：

| 发现 | 终态 | 处置与提交哈希 |
|---|---|---|
| C-1 阻塞：SAM-1 护栏与 brief 写路径写→读闭环断裂（文件永久 lockout） | 修复 | 采纳评审建议方向 1：写侧补齐——`note_representation_issue` 扩展为全文 fence-aware 标题行扫描（复用 `for_each_outside_fence`）+ 未闭合 fence 检查；`brief_add_entry` 锁前拒绝 title `Entries`；roundtrip 回归 + 负向零写入测试齐备（guard +3 / e2e +2，419 全绿）；修复前后复现证据链见 _fix/repro-c1.ps1（P1/P1b/P2 由 exit 0→lockout 变为 exit 1 validation 零写入，brief 全程保持可读）；roundtrip 测试另暴露并修复读侧围栏 note 闭合行静默丢弃缺陷 — 0b4da90 |
| L-1 `#hashtag` 误杀 | 裁定：维持保守策略 | preamble prose 护栏刻意镜像解析器对裸序列化 prose 的非 fence-aware 行为；收窄将为形近变体重新打开结构伪造面，误杀代价仅为带 fix 指引的 validation 拒绝；登记于 fix-ledger 第六节，无代码动作 — 本节所属 docs 提交 |
| L-2 R7 尾扫 CRLF 边界 | 修复 | prev 判定放宽为 `\n` 或 `\r` 均为行边界；新增三回归测试（lone-CR 保留完整首行 / CRLF 分裂保持 / 真·行中切割仍丢弃）— ec59c01 |
| 备注：note 含 `## x` 静默分裂空 path entry（既有问题） | 修复（合并） | 由 C-1 的全文 fence 外标题行护栏一并覆盖（P3 探针实测：修复后 add exit 1 零写入）— 0b4da90 |

销账统计：1 阻塞修复、低级 1 修复 1 裁定登记、备注合并闭合，悬置 0。兼容面盘点（既往合法形态不受影响，收紧面已入 CHANGELOG 与 fix-ledger）见 db3d023。
