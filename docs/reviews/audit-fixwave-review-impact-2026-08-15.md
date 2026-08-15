# 修复波 + perfection 续做 · 影响面（regressions / breaking changes）评审 — 2026-08-15

- 评审维度：仅影响面（对外契约破坏 / 行为回归 / 文档漂移 / 发布纪律 / 依赖构建）；需求覆盖完整性与新代码内部逻辑 bug 由另两位评审员负责，本报告不越界。
- 评审范围：origin/master（3829fd9）→ HEAD（da954c2），23 个本地未推送提交，51 文件 +8336/−1009。
- 方法：diff 通读 + 只读实测探针（target/debug/paperwork.exe，TEMP 夹具）+ `cargo test --workspace --locked` 复跑。
- 实测基线：410 测试全绿（core 228 / cli 182），与 fix-ledger 四节声称一致。

---

## 一、对外契约核验结论

### 1.1 CLI 命令面 / flag / exit code / 信封字段 — 未见破坏

- diff 全量扫描无任何 clap 属性（`#[arg(...)]` / long / short / value_name）增删；命令面、flag 集合、exit code 语义（0/1/2）不变。
- 黄金快照 char_tests（cli 1749 行 / core 234 行）在 origin/master 上**不存在**，为本轮新建冻结（7e2dc0e），不存在"重冻旧快照掩盖输出变更"的风险；JsonBuilder 重构（613734e）由该快照锁字节序。
- 实测探针：`post send` 正常路径 exit 0、fence 预检 exit 1 `error format`、非 UTF-8 stdin exit 1 `error validation`，信封三行结构（message/fix/example）未变。
- 唯一的信封 **category** 语义变化：非 UTF-8 stdin 由旧 generic fallback 的 `io` 改为 `validation`（b107771，实测确认）。exit code 不变。fix-ledger 三节第 4 条已披露；**但 CHANGELOG 未披露**（见 2.1）。

### 1.2 Rust 公共 API（paperwork-core pub 面）

- **唯一 breaking**：删除 `PaperworkError::Io(std::io::Error)` 变体（含 `#[from]`，即直接消费方 `?` 传播 `io::Error` 的代码不再编译）。CHANGELOG Unreleased「Removed — Rust API」节已如实披露，且给出迁移指引；`category()` 对 io 仍返回 `"io"`，CLI 输出不变。评级：已披露的预期 breaking，可接受。
- 其余均为 additive：`create_profile_full`、`find_message_sender`、`resolve_contact_path`、format 层护栏函数族（`check_single_line` / `prose_representation_issue` / `contains_heading_line` / `dedup_preserve_order` / `strip_known_suffix` / `unclosed_fence` / `for_each_outside_fence`）。无签名变更的既有 pub fn（`thread_send` / `thread_edit` / `thread_read` / `thread_summary` / `edit_profile` 等签名逐一核对未动）。
- T5 拆模块后 `paperwork_core::ops::thread::*` re-export 面与历史一致（thread_meta/thread_read/thread_summary/thread_send/thread_edit 等回导），路径兼容无破坏。

### 1.3 托管文件格式兼容性 — **发现一处写/读自相矛盾（阻塞级，见 2.1 前置的 I-1）**

- 既有合法 v0.5 文件读写路径未见破坏：roundtrip 测试 12 项全绿；NEW-8 增量改写与全量重写双路径字节等价（差分语料钉住）；hash_file 流式化摘要位等价。
- 新护栏使既往合法输入变非法的清单及披露核对见下表：

| 护栏 | 既往合法 → 现拒绝 | 披露位置 | 评级 |
|---|---|---|---|
| brief 遗留残留解析护栏（P-2） | 围栏外任何 `### ` 行 / `## Entries` 行的 brief 文件解析拒绝 | CHANGELOG Unreleased（表述为 "legacy v0.4 residue"）+ fix-ledger | 表述窄于实际触发面，见 I-1 |
| fence 平衡预检（D2） | 含未闭合 fence 的 thread 的 send/edit（旧行为 exit 0 静默吞/抹） | fix-ledger 三节第 3 条；**CHANGELOG 未披露** | 保护性收紧，影响可接受 |
| prose 标题形态行拒绝（D3） | description 含 `#`/`##`/`###` 起首行的 profile/brief 写入 | fix-ledger 三节第 1 条；CHANGELOG 只披露了属性行拒绝，**未披露标题行拒绝** | 同上 |
| scope glob 单行校验（D4） | 含换行的 scope 值写入 | fix-ledger 三节第 2 条；CHANGELOG 未披露 | 同上 |
| stdin 非 UTF-8（D6） | category io → validation | fix-ledger 三节第 4 条；CHANGELOG 未披露 | 同上 |
| contacts 标题围栏感知（P-2） | 围栏内 H1 不再被误读为标题（读侧结果可能变化） | CHANGELOG 已披露 | 可接受 |
| brief verify io/Stale 分流（P-2） | 真读失败由 exit 0 Stale 变 exit 1 io 信封 | CHANGELOG 已披露 | 可接受 |

---

## 二、发现清单（按严重度）

### 阻塞（MUST FIX）

#### I-1 brief 条目 note 可含 `### ` 行：写侧放行、读侧永久拒绝，工具自产不可读文件

- **证据（实测探针，HEAD 二进制）**：
  1. `paperwork brief add probe2.brief.md --entry entry.rs --note "First note line\n### sub heading\nlast line"` → **exit 0**，写入成功；
  2. 随后 `paperwork brief read probe2.brief.md` → **exit 1**：`error format: Parse error: brief contains legacy v0.4 residue ('## Entries' wrapper heading or '### ' entry headers)`；
  3. 手写等价文件探针同样 exit 1 同信封。
- **因果链**：P-2 批（669befa）在 `parse_manifest` 加了围栏外 `### ` / `## Entries` 解析拒绝（format/manifest.rs `contains_legacy_brief_residue`）；但写侧 note 护栏只有 `note_representation_issue`（仅查首行，ops/manifest.rs L110-118），note 正文中的 `### ` 行不被拦截。note 为裸散文序列化，`### ` 原样落盘 → 下次任何 brief 操作（read/add/remove/verify）全部 Parse 拒绝。
- **影响面**：agent 消费方一条含 Markdown 子标题的无辜 note 即可使整个 brief 对工具永久不可用（文件本身仍是合法 Markdown，人工可读）。这与修复波 D2 要消灭的"静默数据损坏"同型：写侧 exit 0、损坏延迟暴露。D3 提交信息自己也承认 "`### x` in a brief description trips the legacy-residue parse guard permanently" 并给 description 补了写侧护栏——**note 路径漏补**。
- **披露核对**：CHANGELOG 将该护栏框定为 "legacy v0.4 residue"，实际触发面是"任何围栏外 `### ` 行"，包含工具自己新写的 v0.5 note；框定失真。
- **修复方向**（二选一，交实现方裁量）：(a) `note_representation_issue` 增补拒绝围栏外标题形态行（与 D3 prose 护栏同源基建，写侧零写入 fast-fail）；(b) 收窄解析护栏只在结构位（preamble/条目标题位）触发。推荐 (a)，与"写侧镜像解析器所见"的既定设计一致。

### 重要（SHOULD FIX）

#### I-2 CHANGELOG Unreleased 未披露修复波行为变更，披露面出现"半份清单"

- **证据**：CHANGELOG.md 的 Unreleased 节由 P-批提交逐批增补（P-2/P-4/P-6/P-7/P-8 俱全），但修复波四提交（2c7a180 / 8abdec6 / b107771 及 D7 文档组）**未触碰 CHANGELOG**（`git log origin/master..HEAD -- CHANGELOG.md` 证实最后一笔为 e9c0a41）。
- **缺口**（均已在 fix-ledger 三节披露，但 CHANGELOG 未收录）：
  1. thread send/edit fence 平衡预检 fast-fail（既往 exit 0 的调用现在 exit 1 format，零写入）；
  2. preamble prose 拒绝标题形态行（D3；CHANGELOG 只写了属性行拒绝）；
  3. scope glob 单行校验（D4）；
  4. 非 UTF-8 stdin 信封 category 由 io 语义升级为 validation（D6，exit 1 不变）。
- **影响面**：Unreleased 节已有 "Added — write-side injection guardrails" 完整小节，读者会认为护栏清单是完备的；agent 消费方依赖 CHANGELOG 做回归对账时，会漏掉 4 类新的拒绝触发面与 1 处 category 变化。fix-ledger 是过程性台账，发布披露的权威口径仍是 CHANGELOG。
- **修复方向**：向 Unreleased 追加 fix-wave 条目（可并既有 guardrails 节）；若团队决定统一推迟到发布轮（S-01/LED-15 的 bump 0.6.0 一次性闭合），则应在 Unreleased 节显式注记"修复波变更见 fix-ledger，发布轮合入"，消除"半份清单"歧义。

### 低（CONSIDER）

#### I-3 thread_send 每次发送在独占锁内新增整文件读取，大线程锁持有时间回涨

- **证据**：origin/master 的 `thread_send` 仅 64KB 尾部扫描（`read_last_seq_locked`）；D2（8abdec6）在锁内加 `unclosed_fence_issues_locked`（seek(0) + read_to_string 全文 + normalize + validate_markdown）。P-7 NEW-12 刚省下的发送路径读放大，D2 又为正确性加回一次全读。
- **影响面**：正确性换性能的有意取舍，fix-ledger 已披露实现形态；但大线程（>数 MB）高并发发送场景下锁持有时间从 O(64KB) 变 O(file)，与 SKILL.md 新写的"锁临界区为毫秒级 read-modify-write"的表述在极端体量下不再严格成立。
- **建议**：登记 open-items（可优化为仅扫末段 fence 状态的增量预检）；或在 SKILL.md 锁节补一句"send 预检含一次整文件扫描"。

#### I-4 新增 `.gitattributes`（`* text=auto eol=lf`）对既有克隆的 working tree 影响

- **证据**：243207e 新建 `.gitattributes`（origin/master 上不存在）。目的正确（保护字节级黄金测试在 autocrlf=true 机器上不被 CRLF 化）。
- **影响面**：既有协作者克隆在拉取该提交后，本地若已有 CRLF 检出的文本文件，后续 `git status` / `git add` 会出现 renormalize 噪音；需要一次性 `git add --renormalize .` 或重新检出。对单人仓库当前无实际影响。
- **建议**：发布轮 release notes 提一句即可。

#### I-5 未跟踪文件 docs/dev/e2e-verification-2026-08-15.md 悬置于工作区

- **证据**：`git status --porcelain` 显示该文件未跟踪、未提交（与本评审报告同目录惯例不一致；docs 轮提交 a81d9ad 收拢了 16 份 .md 但未含此件，因其后产生）。
- **影响面**：无代码影响；仅台账完整性。建议随下一 docs 提交纳入或显式 .gitignore。

---

## 三、核验通过项（无发现）

1. **版本与发布纪律**：三处 Cargo.toml 零改动（0.5.0 未动）；tag v0.5.0（1c539fe）为 origin/master 祖先、未被触碰；CHANGELOG `[0.5.0]` 发布段零改动（仅 Unreleased 追加）；未打新 tag、未推送——与 fix-ledger 纪律声明逐条一致。23 个未推送提交结构合理（P-0…P-9 → 修复波 → 台账），提交粒度与编号自洽。
2. **依赖与构建**：无依赖增删；CI 仅两处 additive 门禁（`cargo test --locked`、`cargo doc --no-deps`），只收紧不放松；`--locked` 与已提交 lockfile 实测兼容（本评审 `cargo test --workspace --locked` 全绿）。
3. **锁层融合**：lock.rs 211 行变更为 io_ctx 同构重构，fix/example 文案逐字不变（抽样比对），信封契约无漂移；`locked_read_modify_write` 仍为 SSOT，闭包错误不写盘语义保持。
4. **ensure_suffix OsStr 融合（NEW-3）**：合法 Unicode 路径行为三段语义不变；非 Unicode 路径从 U+FFFD 静默改径修正为字节保真——纯修复，双平台回归测试在场。
5. **文档抽查**：README/SKILL.md 新增示例（`post edit`、`brief remove --entry-title`）与二进制 `--help` 实测一致；ASCII 契约改口径（结构面 ASCII + UTF-8 信封）与 D5 钉住一致；spec §3.1 --author 按实现收口（D7），grep 无"可含空格"残留；bdd S-READ-06 `showing: 0/0` 与 tdd 同步（A-01），S-VAL-04 example 文件名收口（A-02）。
6. **护栏收紧对既往文件的读侧影响**：fence 预检只拦写不拦读——含未闭合 fence 的历史 thread 仍可 `post read`/`summary`（parse 侧本就容忍），仅 send/edit 被 fast-fail 且零写入，agent 可据 fix 行自愈；profile/brief description 护栏只校验新传入值，不回扫既有内容，存量文件的 model 改名等其他编辑不受影响（ops/profile.rs edit 路径逐行核对）。

---

## 四、影响面维度结论

**总体判定：有条件通过（1 阻塞 + 1 重要待闭环）。**

本轮 23 提交对外契约纪律总体良好：CLI 命令面/flag/exit code 零变更，信封结构零变更，黄金快照为新建而非重冻，Rust API 唯一 breaking（`PaperworkError::Io` 删除）已规范披露，版本/tag/发布段零触碰，CI 门禁只收紧。护栏收紧的方向（把静默损坏改为 fast-fail 零写入）对 agent 消费方是净收益，且全部行为变更在 fix-ledger 三节有完整清单。

唯二的实质问题：**I-1** 是护栏体系自身的自洽性漏洞——brief note 写侧放行 `### ` 而读侧拒绝，工具可自产永久不可读文件，属"新护栏制造的新回归"，应阻塞合入/发布直至补齐写侧护栏或收窄读侧触发面；**I-2** 是披露口径问题——修复波 4 类行为变更只有 fix-ledger 记录、CHANGELOG Unreleased 缺载，应在发布轮前补齐或显式注记。其余 3 项为低风险登记项。

（评审完。撰写：影响面评审 agent；2026-08-15；全部结论基于 origin/master..HEAD diff、HEAD 二进制实测探针与 410 测试复跑。）

---

## 五、销账段（修复轮二，2026-08-15 追加）

本报告 5 项发现逐项销账（明细与证据链见 docs/dev/fix-ledger-2026-08-15.md 第六节）：

| 发现 | 终态 | 处置与提交哈希 |
|---|---|---|
| I-1 阻塞：brief note 写侧放行 `### ` 行，工具自产永久不可读文件 | 修复 | 采纳建议方向 (a)：写侧镜像解析器所见——note 护栏扩展为全文 fence 外标题行拒绝 + 未闭合 fence 拒绝，entry title `Entries` 锁前拒绝；修复后实测探针：P1/P1b/P3 类形态 add exit 1 validation 零写入，brief 全程保持可读（lockout 消除）；roundtrip 回归与负向测试齐备；兼容面披露入 CHANGELOG 与 fix-ledger — 0b4da90 |
| I-2 重要：CHANGELOG Unreleased 缺载修复波 4 类行为变更（半份清单） | 修复 | Unreleased 补录 fix-wave 小节：D2 fence fast-fail、D3 prose 标题行拒绝、D4 scope glob 单行校验、D6 category io→validation，并一并披露 C-1 修复与兼容面盘点；P-2 residue 触发面表述失真同步澄清 — db3d023 |
| I-3 低：thread_send 锁内新增整文件读，大线程持锁回涨 | 登记：已知权衡 | 正确性换性能的有意取舍；本产品场景 thread 文件量级小，持锁 O(file) 可接受；未来大线程场景再立专项增量预检；登记于 fix-ledger 第六节，无代码动作 — 本节所属 docs 提交 |
| I-4 低：.gitattributes eol=lf renormalize 噪音 | 裁定：保留 | 保护字节级黄金测试不被 CRLF 化，目的必要；单人仓库无实际影响，未来协作一次性 renormalize 即可，发布轮 release notes 提一句；登记于 fix-ledger 第六节，无代码动作 — 本节所属 docs 提交 |
| I-5 低：e2e-verification 文档悬置 | 修复 | docs/dev/e2e-verification-2026-08-15.md 纳入本次 docs 提交（与三份评审报告、台账追加同批）— 本节所属 docs 提交 |

销账统计：5/5 全部落入终态（修复 3 + 登记 2），悬置 0；「有条件通过」的两个前置条件（I-1 阻塞、I-2 重要）均已闭环。修复轮二全量验证：cargo test --workspace --locked 419 全绿 + clippy -D warnings 零警告 + fmt --check 通过；未 bump/tag/推送，输出协议只增不改，黄金快照未重冻。
