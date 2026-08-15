# Audit Fixwave Round-2 Review — Correctness (逻辑与安全 bug)

- 日期: 2026-08-15
- 评审范围: `git diff 46b1f47..HEAD`（commits 54beff3 / 0ffd9d2 / 86776db 及文档轮）
- 评审维度: 正确性（correctness）— 编译/语法、逻辑与安全、错误处理与边界、可实证性能。需求覆盖与回归影响面由另两位评审员负责，本报告不涉及。
- 评审方式: 读 diff + 源码 + 本机构建探针实测（Windows 真实靶机）+ 全量测试 + docs gate 复演。
- 维度结论: **CLEARED — 0 Critical / 0 Warning / 3 条可选 Suggestion（均为加固项，非缺陷）**

---

## 1. R2-01 修复（文件读通道 InvalidData 识别）— 通过

### 1.1 识别谓词位置正确，无误伤

识别谓词统一为 `e.kind() == std::io::ErrorKind::InvalidData`，出现在三处：

- `repos/paperwork-core/src/error.rs` L121（`io_ctx_file_read` 构造器）
- `repos/paperwork-cli/src/cmd/post.rs` L516（`reject_foreign_thread`）
- `repos/paperwork-cli/src/cmd/validate.rs` L87

`read_to_string`（`std::fs::read_to_string` 与 `File::read_to_string`）在 std 实现中仅在字节序列未通过 UTF-8 校验时产生 `InvalidData`（"stream did not contain valid UTF-8"），其他失败（不存在/权限/共享冲突）分别映射为 NotFound / PermissionDenied 等。因此：

- 合法 UTF-8 文件不会命中该分支；
- 带 UTF-8 BOM（EF BB BF）的文件可正常解码为 U+FEFF，**不会**命中 InvalidData 分支 —— B-1 测试实测通过（read 解析出 1 message、validate exit 0），证明无误伤。

### 1.2 文案无注入风险

`FILE_NOT_UTF8_FIX` 是纯 ASCII 静态常量（error.rs L15），fix 字段不做任何用户输入插值；`example` 中的路径插值（validate.rs L96）为本 diff 之前的既有形态，未改动。无注入面。

### 1.3 category / exit code 保持 io / 1 不变

- 构造器仍返回 `PaperworkError::IoContext`，`category()` 匹配臂未动（error.rs L139 → "io"）；
- error.rs 新单测 `test_io_ctx_file_read_encoding_hint` 双分支断言（InvalidData→编码文案；PermissionDenied→透传调用方文案）；
- 实测探针（UTF-16 LE contacts 文件走 core `contacts_read` 通道）：

```
OUT: error io: IO error at '...\team.contacts.md': stream did not contain valid UTF-8
OUT: fix: the file is not valid UTF-8; check that the file is UTF-8 encoded (binary and UTF-16 files are not supported)
EXIT: 1
```

### 1.4 stdin 通道与文件通道口径一致、无重复识别冲突

- 两通道共用同一检测谓词（InvalidData→编码指向），是谓"口径一致"；
- category 差异为**有意裁决而非 bug**：stdin D6 裁决落 `Validation`（post.rs L607），文件通道 R2-01 明确"category stays io, exit code stays 1"（diff 注释与 B-2 测试双重钉死）。两处注释互相引用（D6↔R2-01），语义闭环；
- 无重复识别冲突：stdin 与文件是互斥的读取源；`post send` 流程中 `reject_foreign_thread`（文件通道）与 `resolve_body`（stdin 通道）顺序执行且各自只映射自己的错误，同一 io 错误不可能被两个分支双重映射。

### 1.5 迁移完整性（正确性视角）

全仓枚举生产路径 `read_to_string` 调用点，全部完成迁移或语义豁免：

| 调用点 | 状态 |
|---|---|
| core: thread_read(meta/read/summary)、thread.rs(edit)、lock.rs(RMW 读)、thread_scan.rs×2、contacts/manifest/profile | 已迁移 `io_ctx_file_read` |
| cli: post.rs reject_foreign_thread、validate.rs | 已迁移（内联谓词，同常量） |
| cli: contacts/profile/brief 的 `--plain` 输出分支（contacts.rs L194 等 3 处） | 豁免成立：到达该行前内容已经 core 解析器以 UTF-8 成功解码，InvalidData 不可达 |
| cli: `destination_advisory`（contacts.rs L248） | 豁免成立：`Err(_)→advisory` 为文档化的探测语义（Ray S-2），且 advisory 非阻塞 |

---

## 2. 新测试质量 — 通过（本机 Windows 真实靶机 6/6 绿，合计 1.05s）

```
test binary_file_read_fast_fails_with_encoding_pointing_fix ... ok
test bom_prefixed_thread_is_tolerated_on_read_and_validate ... ok
test utf16_file_read_fast_fails_with_encoding_pointing_fix ... ok
test h1_leniency_missing_and_duplicate_h1_read_cleanly ... ok
test reserved_device_names_are_sealed_by_suffix_normalization ... ok
test large_thread_2500_messages_send_read_roundtrip ... ok
```

### 2.1 B-1（BOM）/ B-2（UTF-16/二进制）

- B-1 先写合法文件再前置 BOM，断言 read/validate 与无 BOM 等价 —— 强断言，无假阳性面；
- B-2 四层断言（stderr 文案、exit 1、--json category=io + fix、**零写字节级比对** `assert_eq!(fs::read, bytes)`）—— 断言强度高；
- 二进制伴随测试覆盖 0xFF/NUL/overlong（0xC0 0x80）字节。
- 小提示（非缺陷）：B-2 的 UTF-16 fixture 内容未含 ```md 围栏，但该路径在解码阶段即 fast-fail，永不到达解析，fixture 完整性不影响断言语义。

### 2.2 B-5（CON/NUL 保留名）— 30s timeout 不会造成 CI 挂起

三重保险核验：

1. **机制层**：`assert_cmd::Command::timeout` 到期即 kill 子进程并使断言失败 —— 最坏情形是 30s 后测试 FAIL，而非无限挂起；
2. **代码层**：`post send/read` 在任何 open 之前先经 `ensure_suffix` 补后缀（post.rs L168/L251），落点恒为 `CON.post.md` 普通文件；`validate` 裸 CON 在后缀推断阶段（validate.rs L63-80）即以 `Parse`/format 类"unknown file type" fast-fail，**位于 `read_to_string` 之前**，从不开设备句柄；
3. **平台层**：Linux CI runner 上 CON/NUL 无设备语义，退化为普通文件名，风险仅存在于 Windows —— 而本评审恰在 Windows 靶机实测通过（含 `is_file()` 对裸 CON 返回 false 的关键分支）。

### 2.3 B-6（2500 消息）— 时长与断言强度

- 实测含 B-6 在内 6 项合计 1.05s，全量 154 项 cli_integration 3.6s，无时长风险；
- 断言链：总数 2500 → 窗口 20/2500 → validate 全文件 → 追加 seq 2501 → 头（#1 alice）尾（#2501 bob + body）双端回读 —— 覆盖读窗口、写追加、seq 连续性，无弱断言。

### 2.4 B-8（H1 宽容）— 缺 H1 与双 H1 双场景 × read/validate 四象限断言，行为冻结语义清晰。

---

## 3. smoke.ps1（S2-01，gitignored 本地资产）— 通过

- **PowerShell 解析**：用官方 Parser AST 实测 `PARSE_OK`，零解析错误；
- **正文直书转义**：L41 `"-m","@#1 Tests merged. cc @alice"` —— AST 探针提取字符串常量实测值为 `[@#1 Tests merged. cc @alice]`，逐字符无损伤。原理核验：双引号串内 `@` 与 `#` 均非元字符；`@"..."@` here-string 语法要求 `@` 位于 token 首位，此处引号先于 `@`，不触发；L48 既有 `` `" `` 反引号转义亦为正确的 pwsh 写法；
- **派生语义匹配**：body-token 形式（`@#N` 回复 / `@name` 提及，读侧派生）与 post.rs clap after_help 及 SSOT 文法一致；脚本后续 `post read`/`post summary` 步骤正好用于目验派生输出，裁决注释（L38-40）写明依据。

---

## 4. docs gate 修复 — 通过

- `FILE_NOT_UTF8_FIX` 文档注释（error.rs L9-14）改为反引号提及 `` `io_ctx_file_read` `` + "crate-internal" 文字说明，**未使用 intra-doc link**，语义无失真（该项确为 `pub(crate)`）；`io_ctx_file_read` 注释内 `[`Self::io_ctx`]` 链接指向同可见度项，合法；
- 全仓 grep 无其他指向私项的 `[`...`]` intra-doc link 残留；
- **门禁复演**：`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` exit 0，与 ci.yml L46-52 的门禁形态逐字对齐。

---

## 5. 并发/锁面核验 — 未触碰，符合预期

- `lock.rs` 本轮唯一改动是读取失败分支的构造器 `io_ctx`→`io_ctx_file_read`（L86）；错误路径先 `file.unlock().ok()` 再返回的顺序原样保留，锁获取/释放序列、fs2 语义、os error 33 守卫、零写 no-op 跳过逻辑全部未动；
- `thread_scan.rs` 两处为持锁句柄读取的构造器对换，解锁责任仍在调用方，行为不变；
- `post.rs reject_foreign_thread` 读失败路径 `file.unlock().ok()` 保留（L531）。
- 结论：本轮无任何锁路径语义变更；`closure_error_path_releases_lock` 等既有锁测试全绿。

---

## 6. 全量验证记录

| 检查项 | 命令 | 结果 |
|---|---|---|
| 新增 6 项集成测试 | `cargo test -p paperwork-cli --test cli_integration -- <6 names>` | 6/6 ok（1.05s，Windows 靶机） |
| 全量测试 | `cargo test --workspace --locked` | 全绿（0 failed） |
| docs gate 复演 | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` | exit 0 |
| UTF-16 端到端探针 | 手工构造 UTF-16 LE contacts 文件 → `contacts read` | io 包络 + 编码 fix + exit 1 |
| smoke.ps1 AST 解析 | PowerShell Parser::ParseFile + StringConstant AST 提取 | PARSE_OK；body token 逐字符保真 |

---

## 7. 发现清单（分级）

### Critical（必须修复）

无。

### Warning（应当修复）

无。

### Suggestion（可考虑，均为加固项，不构成阻塞）

- **S-1** stdin 与文件通道的编码失败 category 不对称（validation vs io）虽是双重裁决的有意结果，但建议在 SSOT 侧（如 cli-grammar spec 的错误模型一节）补一行"stdin=validation / file=io"的对照记录，防止未来"一致性重构"误平该差异。
- **S-2** B-5 的正确性隐含依赖 Windows 上 `Path::is_file()` 对裸 CON/NUL 返回 false 的平台行为；当前 30s timeout 已将其钉为"失败而非挂起"，可考虑在 ensure_suffix 附近补一行注释点明该依赖（测试 B-5 已是行为锚，注释仅为可读性）。
- **S-3** B-6 验证了头/尾两端与总数，未抽样中段消息正文；若未来线程解析引入窗口/偏移类缺陷，中段破坏可能被该测试漏检。可加一条 `--from 1250 --to 1250` 的中点抽查（一条断言的成本）。

## 8. 维度结论

**正确性维度：CLEARED。** R2-01 识别谓词位置精确（仅 UTF-8 解码失败可达 InvalidData，合法 UTF-8/BOM 无误伤）、文案零注入、io/1 契约保持、stdin/文件双通道无冲突；6 项新测试在 Windows 真实靶机全绿且断言强度充分，B-5 的 CI 挂起风险经机制/代码/平台三层排除；smoke.ps1 转义逐字符保真；docs gate 以门禁同参复演通过；锁路径零语义触碰。全部 154+ 测试与 docs gate 实测通过，无 Critical/Warning 级发现。
