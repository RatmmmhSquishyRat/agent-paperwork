# CLI UX 重设计 v0.5.0 文档对抗评审 —— 实现可行性与技术健壮性（视角三）

- 日期：2026-08-09
- 评审立场：批判性技术评审（挑错而非背书）
- 评审对象：docs/ssot/specs/cli-ux-redesign/ 下 spec.md / design.md / bdd.md / tdd.md / impl_plan.md；docs/roles/cli-ux-redesign-implementer.role.md
- 对照基准：repos/paperwork-cli 与 repos/paperwork-core 现状代码（v0.4.0，clap 4.6.5）、tests/cli_integration.rs、.github/workflows/ci.yml 与 release.yml、CHANGELOG.md、publish.ps1、两份 Cargo.toml、两份 README
- 方法：全部文档精读 + 代码逐行核实行号 + 全仓 grep 核实 example 字符串与旧文法调用点

---

## 一、行号与位置引用核实（事实层）
| 文档引用 | 核实结果 |
|---|---|
| impl_plan ①：ensure_suffix 现 L24-34（cmd/mod.rs） | 准确 |
| impl_plan ②：post.rs L162/L347/L352、validate.rs L54、profile.rs L212 | 全部命中，准确 |
| impl_plan ④：ops/thread.rs L138/L228/L275/L305/L326/L341（6 处） | 准确 |
| impl_plan ④：ops/manifest.rs L32/L105（2 处） | 不准确：实测 5 处，遗漏 L80/L151/L194（均为 `brief create {} --title`） |
| impl_plan ④：ops/contacts.rs L22、ops/profile.rs L61/L91 | 准确 |
| core 旧文法 example 总数 | 实测 14 处（thread 6 + manifest 5 + contacts 1 + profile 2），文档已知清单仅列 11 处 |
| tdd §1 行号（L19/39/52/57/69/93-94/112/118/137/142/161/166/177/178/182/188/199/200/226/247/250/271/297/306/325/358） | 抽查 26 处全部与 cli_integration.rs 实际内容一致 |
| tdd §2 保留断言清单 | 抽查 12 处（L22-23/L42-43/L59-60/L84/L115-128/L168-169/L206/L259/L278/L286/L333/L346/L361-362/L374-381）均吻合 |
| tdd §1 完整性 | 遗漏：L220/L246/L270 三处 `brief create --title`（新文法须位置化，见 spec §2）；L300 `contacts create --title` 保留 flag 正确，无需改写 |
| ci.yml L56-106 / L120-161 | 基本准确：unix smoke 旧文法实际分布于 L56-106，windows L120-161；但 unix 段尾部「echo PASSED」在 L108、windows 在 L163-165，范围标注偏窄不影响改写 |
## 二、Issue 清单

### Critical

**C-1 缺 NAME（usage exit 2）场景无法按现有 clap 设计实现，BDD/TDD 自相矛盾**
- 证据：bdd.md S-SEND-08（`post send standup.post.md "body text"` 期望 exit 2 + `error usage:`）；tdd.md §3 第一条（`.code(2)`）；对照 post.rs 现结构 Send{path, name, body: Option<String>}。
- 分析：新签名 `post send <PATH> <NAME> [BODY]` 下，clap 按位置顺序填充：`post send standup.post.md "body text"` 中 `"body text"` 必然被填入 NAME 槽（第 2 必填位置参数），BODY 为空——解析**成功**，随后走 resolve_body(None,false)（post.rs L349-353 对应逻辑），产出 `error validation:`、exit **1**，而非 S-SEND-08 期望的 exit 2 usage。
  clap 层面无法区分「用户想给 body 但漏了 NAME」与「用户给了 NAME 但没给 body」——两者都是 PATH+一个位置参数的同一形态。唯一能触发 usage exit 2 的形态是 `post send <path>`（只给 1 个位置参数，缺 NAME 必填位）。
- 冲突点：bdd.md S-SEND-08 的 When 子句（带 "body text"）在任何标准 clap 实现下都必然得到 exit 1 validation；tdd.md §3 第一条却按该场景要求 `.code(2)`。二者不可同时满足，实施者照单实现必然测试红或被迫自行裁决（违反 role 文档矛盾上报原则）。
- 建议：bdd.md S-SEND-08 的 When 改为 `paperwork post send standup.post.md`（单位置参数形态），tdd.md §3 同步；或显式裁决「NAME 位被 body 占用」属 validation（exit 1）并在 BDD 中另立场景。此项不修复则步骤⑤的新增用例无法通过。

### Major

**M-1 tdd.md §1 遗漏 `brief create --title` 三处调用点（L220/L246/L270）**
- 证据：cli_integration.rs L220 `["brief","create",brief_path,"--title","My Brief"]`、L246 `--title "B"`、L270 `--title "V"`。spec §2 全表规定 `brief create <PATH> <TITLE>`（--title 位置化），但 tdd §1.2 仅列 post create 六处，完全未列 brief create 三处。
- 影响：步骤⑤按清单改写后，这三处仍以 `--title` flag 调用 brief create，编译可过但运行落入 usage 信封（flag 已移除），brief 全部集成测试红，且不在任何清单内，属「遗漏的旧文法调用点」。
- 建议：tdd §1 补 `brief create --title → TITLE` 小节（L220/L246/L270），并把 §1 合计「23+3=26 处」更正为「26+3=29 处」（L188 保留不计）；impl_plan 步骤⑤同步。注意 L300 `contacts create --title` 因 spec §2 保留 flag，确属不改，勿误伤。

**M-2 impl_plan 步骤④ core example 清单遗漏 manifest.rs 三处（实为 14 处非 11 处）**
- 证据：全仓 grep 实测 ops/manifest.rs 含旧文法 example 5 处：L32（`--entry`）、L80（`brief create {} --title`）、L105（`--entry-title`）、L151（`brief create {} --title`）、L194（`brief create {} --title`）。impl_plan 步骤④清单仅列 L32/L105 两处，漏 L80/L151/L194。
- 影响：core 层真实需刷新 example 总数为 14 处（thread 6 + manifest 5 + contacts 1 + profile 2），清单写 11 处。步骤④若照清单执行，三处 `brief create --title` example 残留旧文法，违反 spec §4.2「全部 example 字符串刷新为新文法」。
- 缓解与建议：impl_plan 已带「完整清单以实施时全仓检索为准」兜底，故降为 Major 而非 Critical；但「已知位置」清单本身错误仍会误导实施，应补 L80/L151/L194 并把 11 处改 14 处。另注意 thread.rs L288、manifest.rs L172、contacts.rs L56/L98 等 example 属**不变文法**（read/create 单位置参数），勿误刷。

**M-3 try_parse 改造未处理 --help / -V 穿透，威胁 spec §6.3 冻结条款**
- 证据：main.rs L58 现为 `Cli::parse()`；impl_plan 步骤③与 spec §4.3 要求改 `Cli::try_parse()` 并把 clap 用法错误渲染为 usage 信封 + exit 2。
- 分析：clap 4.6.5 中 `--help`/`-h`（各层级）与 `-V` 同样以 `Err` 返回（ErrorKind::DisplayHelp / DisplayVersion），其默认语义是打印帮助/版本并 exit **0**。若 try_parse 后对所有 Err 一律渲染 usage 信封并 exit 2，则 help/version 变为报错，直接违反 spec §6.3「全局 flag `--json/--plain/-q/-V` 语义不变」。
- 建议：步骤③明确「DisplayHelp/DisplayVersion 两种 kind 调 `error.print()` 后按 clap 原语义退出（exit 0），仅其余用法错误 kind 进 usage 信封 exit 2」，并在 tdd §3 补 `-V`/`--help` exit 0 的冻结用例。文档对此零着墨，属实施必然踩坑点。

**M-4 usage 信封「逐字修正命令」example 无实现机制，BDD 期望过度特化**
- 证据：spec §4.3「旧式调用自动落入 usage 信封并获得逐字修正命令」；design §9「信封内的 example 即为逐字修正后的新命令」；bdd S-SEND-09 要求 example 恰为 `paperwork post send x.post.md alice "hi"`、S-EDIT-04 要求恰为 `paperwork post edit standup.post.md bob 2 "edited"`（含用户真实参数值）。
- 分析：clap 对未知 flag 只报「unexpected argument」，不携带重建原命令所需信息。要产出含真实参数值的逐字修正命令，须自建「旧文法识别 + argv 重排」迁移层（识别 --from/--seq/--name/--title/--entry/--entry-title/--profile 并映射回位置槽），impl_plan/spec/design 均未给出该机制设计、工作量与边界。
- 影响：实施者若按常规做法输出静态模板 example（如 `paperwork post send <path> <name> <body>`），S-SEND-09/S-EDIT-04 的逐字断言即红；若要满足逐字，则需新增未规划、未设计的迁移层，工作量与回归面失控。
- 建议：二选一——(a) 在 design 增「旧文法迁移层」小节，定义识别表/重排算法/边界（多旧 flag 并存、flag 值缺失、与 -- 边界混用），并把其列入 impl_plan 步骤③；(b) 将 S-SEND-09/S-EDIT-04 的 example 断言降级为「含命令名的模板示例」。不得保持现状交付。

**M-5 ensure_suffix 三级解析对「目录路径」边界未定义**
- 证据：spec §5、design §7.4 裁定 7、bdd S-PATH-01~06 定义三级解析「① 传入路径原样存在 → 用原路径」。评审要点明确要求核对「相对路径/目录路径/已带正确后缀路径」边界，但文档仅覆盖文件存在与否，未定义「传入路径是一个已存在目录」时的行为。
- 分析：第①级「原样存在」若用 `path.exists()` 判断，则目录也命中，路径被原样透传，随后 `thread_read`/`fs::read_to_string` 读目录报 io 错误；而 v0.4 会补后缀后报 not-found。这是行为回归且 BDD 无场景约束。相对路径与已带正确后缀路径两级经核对无问题（S-PATH-01/05 覆盖）。
- 建议：spec/bdd 明确第①级判据为 `path.is_file()`（目录不命中，继续走②③级），并补一条 S-PATH 目录场景用例，避免实施者用 `exists()` 引入回归。

**M-6 步骤②后「集成测试预期红」与全局门禁「每步 cargo test 全绿」自相矛盾**
- 证据：impl_plan「全局门禁」要求每步 `cargo test` 全绿才可进入下一步；步骤②验证却写明「集成测试此步必然红，属预期，由步骤⑤恢复」。role 文档工作原则 3 也承认步骤②后暂红。
- 分析：两条规则在步骤②③④期间不可同时成立。实施者要么违反门禁（带红推进），要么违反步骤说明（强行改测试求绿，恰是原则 3 禁止的「删测试制造绿灯」）。门禁语义含糊会导致步骤流转判定失据。
- 建议：把全局门禁改为「每步 `cargo build` + `clippy` 全绿；`cargo test` 中 core/ops_tests 恒绿，cli_integration 在步骤②~④允许红、步骤⑤起必须全绿」，用可判定措辞消解矛盾。

### Minor
**m-1 usage 错误在 --json 模式下无法感知模式开关**：spec §4.3/S-OUT-03 要求 usage 错误在 `--json` 下输出单行 JSON。但 try_parse 失败时 main.rs 尚未得到 `cli.json`（L60-66 的 mode 判定依赖解析成功），需回退扫描 `std::env::args()` 是否含 `--json`。文档未给出该机制，实施时易漏。建议步骤③注明 argv 扫描兜底。
**m-2 resolve_body 的 example 为 send/edit 共用**：post.rs L342-361 的 resolve_body 被 send（L155）与 edit（L328）共用，其两处 example（L347/L352）均为 `post send` 形态。刷新文法后，edit 的「无正文/互斥」错误仍将给出 send 示例，误导纠错。建议 resolve_body 增加调用方参数以区分示例，或文档显式接受现状。
**m-3 validate.rs 未知后缀分支的 fix 文案未列入步骤②清单**：design §6.2 要求未知后缀的 fix 改为「file must end with ... or pass --type」、example 含 --type，但 impl_plan 步骤②已知点位只列 validate.rs L54；承载该 fix/example 的 L31-35 未被点名。属「其余各 cmd 文件一并核对」兜底范围，但建议点名以免漏改。
**m-4 role 文档文件清单与 impl_plan 步骤⑦不一致**：impl_plan 步骤⑦要求随仓库新增 `SKILL.md`，但 role 文档职责 2 的可改文件清单未列 SKILL.md；清单又列入 `docs/reviews/v0.5-review-*.md`，而职责 2 与 impl_plan 步骤⑨同时声明 QA 评审由独立 agent 产出、实施者不自评。清单与职责表述互斥，建议对齐（补 SKILL.md、删或标注 review book 归属）。
**m-5 post read JSON `showing` 恒现并非纯 additive**：spec §4.6 承认 `showing` 由「超限才出现」改为恒出现。这改变了既有 key 的出现语义——以「showing 缺席 == 未截断」做判断的 JSON 消费者将受影响（post.rs L217-219 现状即条件插入）。措辞上「只增不改不删」与「既有 key 出现条件改变」存在张力，建议 CHANGELOG `Changed (Breaking)` 明示该点，避免被 §6.4 条款掩盖。
**m-6 ci.yml 行号标注偏窄且 exit 2 断言写法未示范**：步骤⑥标注 L56-106/L120-161，实测 unix 段旧文法收尾于 L108（PASSED echo）、windows 段收尾于 L163-165，语法行均在标注范围内，偏窄无害。但「新增一条旧文法触发 usage 信封（exit 2）的断言型 smoke」需要 bash `$?` / pwsh `$LASTEXITCODE` 捕获退出码，文档无示范；直接沿用现有 grep 管道写法会把 exit 2 误判为失败，建议给出两平台样例。
**m-7 发布链条现状风险（非本次新引入）**：release.yml L94 的 awk 依赖 CHANGELOG 小节标题严格为 `## [0.5.0]` 前缀（与现有 `## [0.4.0] - 日期` 格式兼容，抽查通过）；publish.ps1 固定等 30 秒后即 publish cli，若 crates.io 稀疏索引传播更慢会因找不到 paperwork-core 0.5 而失败且脚本无重试。建议步骤⑧注明失败时手工重跑 cli publish。
## 三、已确认可行的设计点（防误杀清单）

1. **post edit 三必填位置参数 + 可选 body 无解析歧义**：clap 按序填充 PATH/NAME/SEQ/NEW_BODY；SEQ 为 u64，非数字输入触发 ValueValidation（usage，exit 2），S-EDIT-03 可精确断言。
2. **隐藏别名 po 无冲突**：main.rs L38/45/49/53 现有别名 p/b/c/v；clap 别名精确匹配、默认无前缀推断，`po` 与 `p` 不冲突，S-ALIAS-01 可行。
3. **`--` 边界**：clap 原生支持，S-SEND-07 可行。
4. **tdd §2 保留断言清单与现状高度吻合**：抽查 12 处行号与断言文本均与 cli_integration.rs 一致，「输出协议冻结防线」可落地。
5. **ops_tests.rs 零改动门禁可靠**：grep 证实 ops_tests.rs 不引用任何 CLI example 字符串，步骤④改文案不会破坏该回归防线。
6. **validate --type 为纯 additive**：validate.rs 未走 ensure_suffix（与 spec §6.1「validate 不参与补后缀」一致），新增可选 `--type` 不破坏既有后缀推断路径，S-VAL-01~04 可测。
7. **implicit-mention 单数字段可行**：post.rs L166-176 已在 reply-to 时隐式并入原发送者（至多一人），步骤②只需在触发分支记录该名字并增补字段，与 S-SEND-03 一致。
8. **window 字段可行**：post.rs L206-209 的「取末 N 条」窗口逻辑与 S-READ-02 `window: #31-#50` 语义一致；空线程不显 window 可在过滤后 messages 为空时跳过，S-READ-06 可测。
9. **ensure_suffix 现逻辑（L24-34）即第②级**：三级解析只需在其前置一层「原路径存在性探测」，改造面小、风险可控（目录边界见 M-5）。
## 四、发布链条完整性核查（版本号/依赖/CHANGELOG/tag/publish）

| 环节 | 现状 | 结论 |
|---|---|---|
| 版本号 | 两 crate 均 0.4.0（Cargo.toml L3） | 升 0.5.0 可行 |
| 依赖声明 | cli 依赖 core `version = "0.4"`（cli/Cargo.toml L19） | 改 `"0.5"` 可行 |
| CHANGELOG | 现有 `## [0.4.0] - 日期` 格式；release.yml awk 按 `^## \[VERSION\]` 提取 | 新增 `## [0.5.0]` 小节须先于 tag（impl_plan 已声明），格式兼容 |
| tag | release.yml on push tags `v*` | 打 `v0.5.0` 即触发，无断点 |
| publish 顺序 | publish.ps1 core→30s→cli，含 `$LASTEXITCODE` 失败即 throw | 顺序正确，无断点（30s 窗口风险见 m-7） |
## 五、是否闭合：不闭合

数量统计：**Critical 1、Major 6、Minor 7**。

必须修复清单（闭合前必改）：
1. **C-1** 修正 bdd S-SEND-08 的 When 形态为 `post send <path>` 单位置参数（或显式裁决「NAME 位被 body 占用」属 validation exit 1），tdd §3 同步——否则该新增用例在任何 clap 实现下必红。
2. **M-4** 为 usage 信封「逐字修正命令」补迁移层设计，或把 S-SEND-09/S-EDIT-04 的 example 断言降级为模板示例——二选一，不得悬空。
3. **M-1** tdd §1 补 brief create --title 三处（L220/L246/L270），合计更正为 29 处改写 + 1 处保留；impl_plan 步骤⑤同步。
4. **M-2** impl_plan 步骤④清单补 manifest.rs L80/L151/L194，总数更正为 14 处。
5. **M-3** 步骤③补 DisplayHelp/DisplayVersion 穿透条款（exit 0），tdd 补 --help/-V 冻结用例。
6. **M-5** spec/bdd 明确三级解析第①级判据为 is_file()，补目录路径场景。
7. **M-6** 全局门禁措辞改为可判定的分阶段绿/红规则，消解与步骤②「预期红」的矛盾。
Minor（m-1~m-7）建议随必须修复一并处理，不阻断闭合判定，但 m-1（--json 感知）与 m-3（validate fix 文案）若不处理会在步骤③/②留下隐性返工。

**结论**：文档集在「输出协议冻结、core API 零变更、发布链条」三方面扎实可落地，行号引用总体准确；但在**最高频命令 post send 的 usage 错误边界（C-1）**与**迁移教学核心承诺的实现机制（M-4）**上存在契约级缺陷，另有 6 处清单遗漏/门禁矛盾。以上 Critical 与 Major 未修复前，不应进入 impl_plan 步骤①。

（评审基准：v0.4.0 源码，clap 4.6.5；本评审只读，未改动任何被评审文件。）
