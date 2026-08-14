# Managed File Format v2 实施计划（impl_plan）

> **文档性质**：实施计划（Normative 流程基线）。按 format 层 → ops 层 → CLI → 测试 → 语料 → 文档 的顺序，逐文件列出改动点（具体到函数名），附依赖关系、验收门槛、风险与回退策略。
>
> **前置条件（条件式表述，F-12）**：阶段 1 对抗性 review-rework loop 闭合是进入实现的先决条件——本计划连同 spec/design/bdd/tdd 与 `docs/roles/format-v2-implementer.md` 正按 leader 裁决（R1–R15、C1–C10）执行 rework；实现者开工前必须确认评审通过结论已成立，不得以本文档存在为由自我豁免该门槛。实现中不得偏离 spec.md；发现规格问题按 OPEN-QUESTION 流程上报（spec §11），不得擅自改规格。
>
> **2026-08-09 owner 追裁（D1–D3）联动**：本文 S1.2/S1.3/S2.1/S3/S5 已按追裁改写（废 participants、废消息属性行改正文派生、围栏 info 改 `md`；`post send` 删 `--to`/`--participants` 保 `--title`）；新增 S3.0 CLI 适配基线现状（2026-08-09 检查）。**版本并入 0.5.0，不 bump**。裁决全文见 design.md §8.5 与 synthesis 末章。

---

## §0 总览与依赖关系

```text
S1 format 层 → S2 ops 层 → S3 CLI 层 → S4 测试重写
                                         ↘ S5 语料与文档（可与 S4 并行）
                                             → S6 版本与发布
```

- core（S1→S2）与 CLI（S3）严格串行（同仓耦合，签名变更跨层传导）。
- S4 紧随各层：format 内联单测与 S1 同批提交，ops_tests 与 S2 同批，cli_integration 与 S3 同批（TDD 先行的最低要求是"每层改动与其测试同一提交内全绿"）。
- S5 的语料/README/CHANGELOG/ci.yml 与 S4 无编译耦合，可并行。
- 每阶段结束的门槛：`cargo test --workspace` 全绿 + `cargo clippy --all-targets -- -D warnings` 零告警。

## §1 阶段 S1：core format 层重写（`repos/paperwork-core/src/format/`）

### S1.1 `format/mod.rs`

删除：

- `MESSAGE_HEADER_RE`、`parse_message_header`（旧 H3 `·` 头）
- `is_boundary_line`、`is_four_backtick_fence`、`find_message_boundaries`（`---` + 前瞻边界算法整体废除）
- `parse_scope_globs`、`serialize_scope_globs`（反引号 glob 构造废除；Scope 新形态为属性行列表，见 S1.4）

改写：

- `BULLET_KEY_RE` → 属性行正则 `^- ([a-z][a-z0-9-]*):\s*(.*)$`（spec §3.2）；`extract_bullet_key` 更名 `extract_attribute`。
- `validate_markdown` → 动态围栏感知：以栈/状态记录开启长度 N，关闭判定"纯反引号行且长度 ≥ N"；围栏行立场按 §3.3（≤3 空格前导；tilde 不识别）；未闭合报告开启行号（tdd T-FM-05）。

新增：

- `backtick_run(line: &str) -> usize`：行首**前导空白 ≤3 空格**之后的连续反引号计数（≥4 空格缩进返回"非围栏"语义，R13）。
- `compute_fence_length(body: &str) -> usize`：max(3, body 最长连续反引号串 + 1)（spec §3.4）。
- `fence_close_matches(line: &str, open_len: usize) -> bool` 等共享围栏谓词（供 thread.rs 与 validate_markdown 复用）。
- 重写对应内联单测（tdd T-FM-01..05；删除清单见 tdd §1）。

保留：`normalize_line_endings`（I11）不动。

### S1.2 `format/thread.rs`

- 新增静态正则 `MESSAGE_HEADER_RE = ^##\s+#(\d+)\s+([^\s()]+)\s+\((.+)\)\s*$`（spec §5.3，R1/R9：sender 排除括号、字段间空白宽容、行尾容忍尾随空白；消息头必须顶格，前导空白退化为 preamble，C4）。
- 重写 `parse_messages`：CRLF 归一 → fence 感知逐行扫描 → 首个匹配消息头的 H2 之前为 preamble、其后按匹配头切分消息；**消息无属性区**（追裁 D2：头与首个围栏之间的任何内容忽略）；动态围栏正文（**info 宽容：`md`/`markdown`/其余任意/无 info 均接受，多围栏取首个、其余忽略**，C2/D3）。
- 正文提取规范化（R12）：body = 开启行与关闭行之间的行序列，去除首尾空白行后 `\n` 连接。
- 新增 `parse_preamble(content: &str) -> ThreadMeta`：**preamble 仅剩 H1 标题**（D1）：H1 = title；标题行后散文、属性行（含历史 `- participants:`）与其余内容一律忽略；`ThreadMeta { title }` 仅作解析读取视图。
- 新增派生工具函数（spec §5.4 派生算法，读取/统计路径使用，不落盘，D2）：`derive_mentions(body: &str, sender: &str) -> Vec<String>`（`@([^\s@()]+)`，顺序去重、排除自提及、`@#N` 归 reply 不计入）与 `derive_reply_to(body: &str) -> Option<u64>`（`@#(\d+)` 取首个，不校验目标存在性）。
- 重写 `parse_message_content`：删除 `parse_to_field`/`parse_reply_to` 的属性行解析路径（D2，函数可随字段一并删除）。
- 重写 `serialize_message`：头 `## #<seq> <sender> (<ts>)`（规范单空格形态）+ 动态围栏正文（info 严格 `md`，§5.9 精确格式，D3）；**不输出任何属性行**（D2）；删除 `---\n\n` 前缀与 `- To: all`。
- 新增 `serialize_preamble(meta: &ThreadMeta) -> String`（**仅 H1 title**，§5.9，D1）；新增 `serialize_messages(&[Message]) -> String`（供 post read --plain，子集输出无 preamble）；`serialize_thread` 签名改为 `serialize_thread(meta: &ThreadMeta, messages: &[Message])`。
- 保留 `parse_timestamp`、`validate_seq_monotonicity`（错误文案按 spec §9.2 核对）。
- 新增 `validate_sender(sender: &str) -> Result<()>`（spec §5.6，无空格/无括号/非空）。
- 全部内联单测按 tdd §2.2 重写。

### S1.3 `lib.rs`

- `pub struct ThreadMeta { pub title: String }`（derive Debug/Clone/PartialEq/Serialize/Deserialize；participants 字段随追裁 D1 删除）。`Message` 删除 `to` 字段；`reply_to`/`mentions` 保留为解析期派生字段，序列化不再输出（D2：引用状态为正文文本派生，不落盘）；其余领域类型与 `VerifyResult` 不动。

### S1.4 `format/profile.rs`

- 重写 `parse_profile`：H1 = name；H1 后首个 H2 前散文段 = description；`- model:` 必需；`## Scope` 节体解析为**属性行列表**（R3：一行一个 (permission, glob) 对 `- <perm>: <glob>`，键 read/write/owns 可重复、按行序保序聚合，未知 permission 行忽略；glob 取冒号后 trim 值）。
- 重写 `serialize_profile`：按 spec §4.3 精确格式（description 空则省略；scope 空则省略整节；scope 属性行序 read → write → owns）。
- 新增内部 `parse_scope_lines` / `serialize_scope_lines`（取代原表格方案的 `parse_scope_table`/`serialize_scope_table` 命名）。
- 错误文案改小写键（spec §4.4/§9.2）；内联单测按 tdd §2.3 重写。

### S1.5 `format/manifest.rs`（brief）

- 重写 `parse_manifest`：preamble（H1 + 散文 description + `- owner:`/`- created:` 必需）；条目 = 后续各 H2；**条目属性区 = 条目 H2 之后至首个非属性非空行**（空行不终止；其后同形行归 note，R4/BRIEF-12）；条目属性 `- path:`/`- hash:`/`- regex:`（裸文本，删除 `strip_backticks` 调用路径）；` ```regex ` 围栏保留（fence 感知下收集至闭合行）；note 为首个非属性非空行起至下一 H2 的散文段；删除 `## Entries`、`---` 分隔、blockquote note、`Manifest: ` 前缀兼容分支。
- 重写 `serialize_manifest` / `serialize_entry`：小写键；无 regex 省略该行；复杂 regex（含 `\n` 或反引号）用 ` ```regex ` 围栏；note 输出为裸散文段（无 `>` 前缀）。
- 保留 `extract_regex_groups`、`parse_timestamp`、`EntryBuilder`（字段不变）。
- **核对项（C7）**：`brief verify` 路径解析（base_dir 缺省 = brief 父目录）对新格式 `- path:` 裸文本取值经 trim 的行为与现状一致，核对确认即可，不改。
- 错误文案改小写键（spec §6.5/§9.2）；内联单测按 tdd §2.4 重写。

### S1.6 `format/contacts.rs`

- 重写 `parse_contacts`：条目正则识别两种链接形式 `[label](path)` 与 `[label](<path>)`（尖括号剥离；destination 反转义 `\<`/`\>`；label 反转义 `\]`；`[label](path "title")` 忽略 title 提取 destination，C3）；非链接 bullet 忽略（hard breaking）。
- 保留 `parse_contacts_title`（文案不变）。
- 重写 `serialize_contacts(title, contacts)`：输出 `- [<label>](<destination>)`；**label 由调用方（ops 层）按 R11 派生后传入**，不在 format 层读文件；destination 含空格/tab/`(`/`)`/`<`/`>` 时用尖括号形式（`<`/`>` 转义 `\<`/`\>`，spec §7.3）。
- 内联单测按 tdd §2.5 重写。

### S1.7 `error.rs`

- 结构零改动。全部调用点 fix/example 文案更新为纯 ASCII 新格式文案（spec §9.2 表逐字采用）。

**S1 完成标志**：`cargo test -p paperwork-core --lib` 全绿，tdd §2 全部条目落地。

## §2 阶段 S2：core ops 层适配（`repos/paperwork-core/src/ops/`）

### S2.1 `ops/thread.rs`

- `SEQ_RE` 改为 `(?m)^##[ \t]+#(\d+)`（spec §5.5；多行模式使 `^` 匹配行首；`[ \t]+` 为头正则字段间 `\s+` 的行内等价）。
- `read_last_seq_locked` 两项修订：
  - **尾扫 fence 感知（缓冲区内，R6）**：对缓冲区内围栏行按 CommonMark 长度规则做开合追踪，处于开启围栏内部的候选头跳过；残留限制（缓冲起点切断围栏奇偶）以 validate seq 连续性兜底（spec §5.5 明文声明）。
  - **丢弃首行规则（R7）**：仅当 `read_start > 0` 时检查前一字节，非 `\n` 才截到第一个 `\n` 之后；`read_start == 0` 不丢弃。保持 O(1) 机制与 64KB+256B 缓冲不变。
- `thread_send` 签名简化（D1/D2）：删除 to/mentions/reply_to 参数，新增 `preamble: Option<&ThreadMeta>`（新签名形如 `thread_send(path, sender, body, preamble)`）；锁内流程：`validate_sender` → 尾扫 seq → **若文件 size == 0 且 preamble 非 None，先写 `serialize_preamble`（仅 H1，D1）**（与消息体合并为单次 `write_all`，避免交错写，不变量 I9）→ 追加消息。size > 0 时 preamble 参数忽略（OQ-1 默认）。
- 新增 `thread_meta(path: &Path) -> Result<ThreadMeta>`：文件不存在返回缺省 meta（不报错）；存在则 `parse_preamble`（仅 title，D1）。
- `thread_edit`：锁内解析 preamble 与消息；**重写时对首个消息头之前的字节区间原样搬运（R5），消息序列重新序列化**；**对新 body 序列化后执行 ≤64KB 校验，超限 `MessageTooLarge` 且文件不变（R8）**；三重约束逻辑不动。崩溃窗口声明见 spec §5.7（接受，锁排除并发写者，仅断电/杀进程暴露）。
- `thread_read`：适配新 `parse_messages`；`--mention`/`--reply-to` 过滤改为基于 `derive_mentions`/`derive_reply_to` 派生值匹配（D2）。
- `thread_summary`：适配新解析；**participants 改由消息 sender 集合派生**（按首次出现顺序去重，D1），不再读 preamble 名单。

### S2.2 `ops/profile.rs`

- 签名零改动（`create_profile`/`show_profile`/`edit_profile`）；随 S1 序列化变更自然适配；核对 `create_profile` 空 description 时输出符合 spec §4.3（省略 description 段）。

### S2.3 `ops/manifest.rs`

- 签名零改动（`brief_create`/`brief_add_entry`/`brief_remove_entry`/`brief_read`/`brief_verify`/`verify_entry`）；随 S1 适配。`brief_add_entry` 落盘 hash 保持全量（I7），核对无截断路径。

### S2.4 `ops/contacts.rs`

- 签名零改动；`contacts_add` 幂等判定按 profile_path 不变；**label 派生（R11）：写入时读取目标 profile 的 H1 作为 label，读取失败回退文件名主干（先剥 `.profile.md` 再剥 `.md`，否则原名）**；序列化走新 `serialize_contacts`（转义规则见 S1.6）。

**S2 完成标志**：`tests/ops_tests.rs` 按 tdd §3 全部改写并通过（含新增 T-OPS-16..30）。

## §3 阶段 S3：CLI 适配（`repos/paperwork-cli/src/`）

### S3.0 CLI 适配基线现状（2026-08-09 检查，只读确认，供后续实现任务依据）

检查对象：`repos/paperwork-cli/src/cmd/post.rs` 与 `src/main.rs`。结论：**cli-ux-redesign（`docs/ssot/specs/cli-ux-redesign/`）尚未落地到代码**——post 命令仍是 format-v2 v0.5 文法（`--from` 仍为 flag，未改造为 NAME 前置位置参数）。当前文法要点：

- `post` 子命令：`send` / `read` / `summary` / `edit` 四个；`create` 已不存在（system 消息废除已落地）。
- `post send <PATH> --from <NAME> [BODY] [--stdin] [--reply-to N] [--mention a,b] [--title T] [--participants a,b] [--to a,b]`：PATH 为第 1 位置参数，BODY 为可选第 2 位置参数（与 --stdin 互斥）；`ensure_suffix` 自动补 `.post.md`；`default_title`（剥 `.post.md`→剥 `.md`→原名）已实现；空 body（trim 后空）拒绝（Validation）已实现；reply 隐式 @ 原发送者逻辑已实现（写入 mentions 字段）；CLI 恒传 `Some(ThreadMeta)`、ops 锁内 size 守门已实现。
- `post read <PATH> [--from N] [--to M] [--mention X] [--reply-to N] [--limit 20]`：`--from/--to` 为 seq 区间；`--mention`/`--reply-to` 为过滤（现状基于 Message 字段，追裁后需改为派生值匹配）；`--plain` 走 `serialize_messages`（无 preamble）已落地。
- `post summary <PATH>`：已改为 `thread_meta` 直读 preamble（title/participants），无字符串切分。
- `post edit <PATH> --seq N --from NAME [NEW_BODY] [--stdin]`。
- 全局 flag：`--json` / `--plain` / `--quiet`（main.rs）；错误信封经 `PaperworkError::category()/fix()/example()` 直出。

**本轮追裁的 CLI 适配面**（在基线之上）：删 `--to`/`--participants` 两个 flag 及其处理代码（含 `clean_list(participants)`/`clean_list(to)` 调用点）；保 `--title`；`--reply-to`/`--mention` 按 spec §11 OQ-4 默认处置（保留，语义改为正文 token 注入）；`thread_send` 调用点改新签名；summary 的 participants 改取派生值；read 过滤改派生值匹配。cli-ux-redesign 的 NAME 位置参数化不在本轮范围（由其自身实施任务负责），本轮不得顺手改造。

### S3.1 `cmd/post.rs`

- `PostCommand::Create` 已不存在（基线确认，无需再动）。
- `PostCommand::Send`：删除 `--participants` 与 `--to` 两个 flag 定义及处理代码（追裁 D1/D2）；保留 `--title`（缺省算法不变，F-8）；`--reply-to`/`--mention` 保留，语义改为**正文 token 注入**（spec §11 OQ-4 默认）：send 序列化前在 body 首部注入 `@#N` 与各 `@name` token（空格分隔，其后换行接原 body）；reply 隐式 @ 原发送者逻辑同步改为注入（自回复/已在名单/seq 不存在三种不触发边界不变）。
- Send 分支：组装 `ThreadMeta { title }`（无 participants 字段），CLI 恒传 `Some(meta)`，由 ops 层锁内 size 判定守门（F-9）；调用新签名 `thread_send(path, sender, body, Some(&meta))`。
- Summary 分支：title 仍 `thread_meta` 直读；**participants 改由消息 sender 集合派生**（首次出现顺序去重，D1）。
- Read 分支：`--mention`/`--reply-to` 过滤改为基于 `derive_mentions`/`derive_reply_to` 派生值匹配（D2）；--plain 分支不动（基线已落地）。
- 默认档输出组装中 `reply:`/`mentions:`/`to:` 头行字段改为展示派生值（或删除 to 字段，随 Message 字段删除自然适配）。

### S3.2 `cmd/validate.rs`

- Post 分支：`parse_messages` → 非空校验（fix 文案按 spec §9.2）→ `validate_seq_monotonicity` → `validate_markdown`，任一失败即 error 信封。**错误信封直接透出底层变体**：seq 失败 → Validation（category `validation`），不再统一重包为 Parse（R10）；现状 validate.rs 的统一 Parse 重包逻辑删除。**空文件拒绝为行为变更**：删除现状 `!content.trim().is_empty()` 豁免，空内容按零消息报 Parse（spec §8，F-3）。
- 新增疑似消息头启发式（R9）：形似 `## #<数字>` 开头但不严格匹配头文法、不在围栏内的行 → warning + fix（不改变 ok/error 结论）。
- Profile/Brief/Contacts 分支：parser 成功后追加 `validate_markdown` 围栏闭合校验。
- 未知后缀分支不动；`FileType` 不动。

### S3.3 `cmd/profile.rs` / `cmd/brief.rs` / `cmd/contacts.rs`

- 无参数结构变更；仅输出组装适配新字段形态（brief read 的 regex 输出、contacts read 的 label 展示等随序列化变更核对）。
- `cmd/contacts.rs::enrich_profile` 即时增强行为不变。

### S3.4 `main.rs` / `output.rs` / `cmd/mod.rs`

- **零改动**（错误信封协议不变，design.md §7）。`ensure_suffix` 不动。

**S3 完成标志**：`tests/cli_integration.rs` 按 tdd §4 全部改写并通过。

## §4 阶段 S4：测试收尾

- 对照 tdd.md §6 覆盖核对表逐行销项；确认 bdd.md 79 个场景无遗漏。
- 补充等价类抽查：动态围栏 3/4/5/6（POST-06）、fence 内伪造头（POST-05）、sender 空格/括号（POST-07/17）、Windows 带空格路径（CONT-03）、CRLF、Unicode、空文件（VAL-07）、0 消息、seq gap（category validation）、断 fence、疑似头 warning（VAL-08）、并发四例（CONC-01..04）、尾扫边界（POST-32）。
- 门槛命令：`cargo test --workspace`、`cargo clippy --all-targets -- -D warnings`。

## §5 阶段 S5：语料与文档

### S5.1 `test-v05/` 冒烟语料（新建，不改动 `test-v03/`、`test-v04/`、`_fix/`）

- 正例：`alice.profile.md`、`bob.profile.md`（含 Scope 属性行列表）、`standup.post.md`（preamble 仅 H1 + 多消息，正文含 `@mention`/`@#N` 引用，围栏 info 为 `md`，D1/D2/D3）、`onboarding.brief.md`（含 ` ```regex ` 条目）、`team.contacts.md`（链接条目，含一带空格路径示例）。
- 坏例：`garbage.post.md`（零消息）、`garbage.profile.md`（缺 model）、`gap.post.md`（seq gap）、`broken-fence.post.md`（断围栏）——供 `validate` 负例冒烟。
- 语料内全部文件必须通过新 `validate`（坏例除外，坏例必须被拒绝）。

### S5.2 README（三个文件，F-4）

- `README.md`（根）：重写格式章节——四格式新 schema 示例、命令表（删 `post create` 与 `--to`/`--participants`，保 `--title`）、输出协议段落保留；一并消灭 `·`/`.` 文档偏差（synthesis §2.5 阶段 2.5 要求）。
- `repos/paperwork-cli/README.md`（crates.io 发布门面，cargo publish 的 readme 字段所指）：删除其中 `paperwork post create` 示例与旧格式片段，按新 schema 同步。
- `repos/paperwork-core/README.md`：格式描述段落按新 schema 同步核对。

### S5.3 `CHANGELOG.md`

- 新增 `0.5.0` Breaking 段（含 owner 追裁 D1–D3 变更，并入 0.5.0 不 bump）：逐格式 before/after 对照 + 手工迁移步骤（旧头改 `## #N sender (ts)`、删 `—`/`all`、4 反引号改动态、system #1 内容手工提取为 preamble H1、contacts 裸路径改链接、**participants 名单与 reply-to/mentions/to 属性行全部删除——引用改为正文 `@somebody`/`@#N` 文本、participants 无需迁移（派生）、围栏 info `markdown` 可保留（解析宽容）或改 `md`、profile Scope 表格改属性行列表**）；声明无 migrate 命令（design.md §6）。
- **迁移指南必须明示 hard breaking 后的已知症状（C7）**：旧格式 profile（大写键 `- Model:`）在 `profile list` 中显示 `(unreadable)`、`contacts read` 即时增强同理——属宽容解析的预期降级而非故障。

### S5.4 `.github/workflows/ci.yml`

- 按 tdd T-CI-01/T-CI-02 改写 unix 与 windows smoke：删 `post create` 行；首写改 `post send … --title "Test" --from alice "Hello world"`（无 --participants，D1）；`--reply-to 2` → `--reply-to 1`（OQ-4 默认：正文 token 注入）；删除 `--to` 断言（flag 已删，D2），改断言落盘围栏 info 为 `md`（D3）与 seq gap 负例；`test` job（build/test/clippy）不动。

## §6 阶段 S6：版本与发布准备

- 版本决策（owner 追裁）：**0.5.0 未发布，本轮 D1–D3 变更并入 0.5.0，不 bump 版本**；`repos/paperwork-core/Cargo.toml` 与 `repos/paperwork-cli/Cargo.toml` 维持 0.5.0（若尚未 bump 则按原计划 `0.4.0` → `0.5.0`，cli 依赖声明改 `paperwork-core = { version = "0.5", path = "../paperwork-core" }`）。
- 发布顺序不变：core → 30 秒延迟 → cli（`publish.ps1` 零改动）。发布动作不在本实现任务内，仅做版本准备。

## §7 验收门槛（全部满足才算完成）

1. `cargo test --workspace` 三平台语义全绿（本地至少 Windows 全绿；CI 三平台通过）。
2. `cargo clippy --all-targets -- -D warnings` 零告警。
3. tdd.md §6 覆盖核对表全部销项；bdd.md 79 场景无遗漏。
4. `test-v05/` 正例全部通过 `paperwork validate`，坏例全部被拒绝。
5. README（根、paperwork-cli、paperwork-core）/CHANGELOG/ci.yml smoke 与新格式一致。全仓精确检索（排除范围：历史语料目录 `test-v03/`、`test-v04/`、`_fix/`；`docs/` 下评审与历史记录；两个 Cargo.toml 的 description 英文标点 `—`），全部命中为零（F-5，可机器判定）：
   - 无 `- to: all` 行，无 `- To: all` 行；
   - 无 `·`（U+00B7）字符；
   - 无连续恰 4 个反引号的围栏行（正则 `` ^\s{0,3}`{4}(?!`) ``）；
   - 无 `---` 消息边界行（格式构造意义上的；Markdown 水平线不在此列，以文档上下文核对）；
   - 无 `post create` 命令残留（命令表、示例、smoke 脚本）；
   - post 语料与代码无 `- participants:`/`- reply-to:`/`- mentions:`/`- to:` 属性行残留（追裁 D1/D2；历史语料目录与 docs 评审记录除外）；
   - post 正文围栏开启行 info 无 `markdown` 全称写侧残留（新序列化统一 `md`，D3；解析侧宽容不受此限）。
6. 后续 Ultra Review 与 `docs/reviews/v0.5-review-<date>.md` 按 synthesis §6.4 执行（不在本计划执行范围）。

## §8 风险与回退策略

| 风险 | 缓解 / 回退 |
| --- | --- |
| `serialize_thread` 签名变更波及面大（ops/CLI/测试） | 严格按 S1→S2→S3 串行；每层提交独立可编译（编译器强制销项） |
| `thread_send` 锁内首写引入竞态 | 首写判定在锁内以 `metadata().len() == 0` 为准；T-OPS-21/30 并发用例守门 |
| 动态围栏解析与 CommonMark 细节偏差 | 解析只实现本项目需要的子集（§3.3 状态机，≤3 空格缩进、tilde 不识别），以 tdd T-FM-03/04/05 为行为基线，不引入 markdown 解析库 |
| 尾扫 `(?m)^` 锚定在缓冲区中段失配 | `read_last_seq_locked` 按 R7 条件式丢弃首个不完整行；T-OPS-28 边界三例守门 |
| 尾扫 fence 盲残留（缓冲起点切断围栏） | 缓冲区内开合追踪（R6）消除常见污染；残留限制以 validate seq 连续性兜底（spec §5.5 声明）；T-OPS-29 固化行为 |
| `thread_edit` 崩溃窗口（截断+重写中断电/杀进程丢全文件） | 接受并声明（spec §5.7）：锁已排除并发写者；未来加固方向为锁内 temp+rename（本次不做） |
| CI smoke 脚本跨平台差异 | unix/windows 两份脚本同步改写（T-CI-01/02），语义逐行对照 |
| 实现中发现 spec 矛盾/遗漏 | 停止相关改动，按 OPEN-QUESTION 流程上报 leader，不得擅自改规格（spec §11） |
| 正文派生正则边界（`@` 与邮件地址/代码片段撞形、`@#N` 与 mentions 互相污染） | 以 spec §5.4 默认规则为准（顺序去重、自提及排除、`@#N` 归 reply）；tdd T-FT-03/23/24 边界用例守门；不引入更复杂的词法分析 |
| 回退策略 | 每阶段独立提交；任一阶段门槛失败可 `git revert` 到上一阶段提交；格式层（S1）与 CLI 层（S3）提交互不嵌套，保证单阶段可整体回滚 |
