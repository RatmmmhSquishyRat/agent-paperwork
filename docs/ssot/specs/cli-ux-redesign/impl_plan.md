# CLI UX 重设计 v0.5.0 — Impl Plan（实施计划）

- 日期：2026-08-09
- 版本：v0.5.0
- 文档性质：实施计划（按总方案「实施」章节逐条展开为带依赖的顺序步骤）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.5_feedbacks.md`（owner 指令）
  - `docs/dev/adr-v1.md`（ADR-011）、`docs/ssot/adr/feedbacks/v0_feedbacks.md`
  - `docs/researches/cli-ux-agent-visible-output-research-2026-08-08.md`
  - `docs/researches/ux-open-items-backlog-2026-08-08.md`
- 前置门槛：本目录 spec/design/BDD/TDD 四份文档与 role 文档经对抗评审闭合后方可开始步骤①（实现流程原则.md）；实施者为 `docs/roles/cli-ux-redesign-implementer.role.md` 定义的角色。

---

## 全局门禁（分阶段化，F6 裁定）

- **步骤①~④期间**：每步完成后 `cargo build`（workspace）+ `cargo test -p paperwork-core`（ops_tests 恒绿）+ `cargo clippy --all-targets -- -D warnings` 全绿即可进入下一步；**cli_integration 允许红**（步骤②起旧文法调用必然红，属预期，由步骤⑤恢复）。
- **步骤⑤完成后**：`cargo test`（workspace 全量）+ clippy 全绿为硬门禁，后续步骤不得带红推进。
- core 层锁/seq/格式/hash 逻辑零改动；`ops_tests.rs` 一行不改且必须全绿（越界即回滚）。
- 输出协议冻结条款（spec §6）全程有效：ok/error 信封结构、command 标识、JSON 既有 key 只增不改不删；**注意**本版含四项消费者可感知变化（showing 恒显、exit 2、第七类 usage、三个新字段），须在 CHANGELOG 逐项披露（步骤⑦，design §9）。

---

## 步骤① ensure_suffix 原路径优先（U-14/N-02）

- **文件**：`repos/paperwork-cli/src/cmd/mod.rs`（`ensure_suffix`，现 L24-34）
- **内容**：改为**三级解析**——① 传入路径原样存在且为**文件**（判据 `is_file()`，**目录不命中**，落入后续级别）→ 用原路径；② 否则，补类型后缀后的路径存在 → 用补后缀路径（补后缀即现有逻辑：已带类型后缀原样 / 裸 `.md` 替换 / 无 `.md` 追加）；③ 都不存在 → **以补后缀路径为操作落点**。
- **第③级为「路径决策」语义**：三级解析只决定操作落点路径；物理创建仅发生在写命令（send/create/add），**只读命令（read/summary/validate）三级均无文件时报 not-found，不创建文件**（消除 spec 与 S-PATH-04 的冲突，符合 ADR-011 stateless）。
- **注意**：第①级命中异型文件（非 paperwork 格式）时按对应类型解析器报 format 错误，不再自动改道（与 v0.4 无条件改写的行为差异，spec §5，S-PATH-07）；传目录且补后缀形态亦不存在时报 not-found（S-PATH-08）；场景覆盖见 bdd.md S-PATH-01/02/04/05/06/07/08。
- **依赖**：无。**验证**：cargo build + core 测试 + clippy 全绿（此步不新增测试，测试随步骤⑤落）。

## 步骤② CLI 层签名重排与 example 刷新

- **文件**：`cmd/post.rs`、`cmd/profile.rs`、`cmd/brief.rs`、`cmd/contacts.rs`、`cmd/validate.rs`
- **内容**：
  - 按 spec §2 全表重排 clap 签名：必填 flag 转位置参数（NAME/SEQ/TITLE/ENTRY/ENTRY-TITLE/PROFILE-PATH），`post send/edit` 移除 `--from`，`post edit` 移除 `--seq`；
  - `post send` 输出增补**单数字段** `implicit-mention: <name>`（U-10，additive，仅在发生隐式 mention 时出现，不触发则不输出该字段）；
  - `post read` 输出增补恒显 `showing: n/total` 与 `window: #<first>-#<last>`（字段区形态，按实际展示的第一条与最后一条 seq；空线程不显示 window）（U-11）；
  - `validate` 增补 `--type post|profile|brief|contacts`（U-15）；
  - **example 字符串刷新为新文法**：CLI 层已知点位 `post.rs` L162（空正文）、L347（--stdin 互斥）、L352（无正文）、`validate.rs` L54（format 教学示例）与 **L31-35（未知后缀分支的 fix/example 承载处，点名免漏改）**、`profile.rs` L212（list not-found）；其余各 cmd 文件内 example 一并核对刷新。
  - **resolve_body 示例按命令区分（可行性评审 m-2）**：`post.rs` 的 resolve_body 被 send/edit 共用，其错误 example 现为 send 形态；增补调用方参数（或等价手段）使 edit 的「无正文/互斥」错误给出 edit 示例，不得用 send 示例误导纠错；
  - **NAME/BODY 混淆面教学（F1）**：无正文错误的 message 补「若你已给出正文，请检查是否遗漏 NAME 槽位」提示，example 为含 NAME 槽完整形态；validation fix 含 `--` 边界用法（spec §4.2，X4）；
  - 各子命令 after_help 补可复制示例（design.md §2.4/§3.3/§4.3/§5.3/§6.3；send/edit 各含一条 `--` 边界示例，post read 含 seq 范围示例，contacts create 含 title 可选 flag 注记）。
- **依赖**：步骤①（send/read 路径解析行为变化需先就位）。**验证**：cargo build + clippy 全绿（cli_integration 此步必然红，属全局门禁允许范围，由步骤⑤恢复）。

## 步骤③ main.rs usage 信封与别名；output.rs 错误 JSON

- **文件**：`main.rs`、`output.rs`
- **内容**：
  - `main.rs`：`Cli::parse()` 改 `Cli::try_parse()`；clap 用法错误渲染为标准 usage 信封（category `usage` + **静态规范示例**：该命令规范 usage 行 + 一条预置可执行示例，不携带用户原参数值，F2 裁定）并 **exit 2**；运行时错误保持 exit 1；
  - **穿透条款（F5，可行性评审 M-3）**：DisplayHelp/DisplayVersion 两种 kind（--help/-h 各层级与 -V）调 `error.print()` 后按 clap 原语义 **exit 0**，不进 usage 信封（守住 spec §6.3）；仅其余用法错误 kind 进 usage 信封；
  - **--json 感知（可行性评审 m-1）**：try_parse 失败时尚无 `cli.json`，须回退扫描 `std::env::args()` 判定 `--json` 是否出现，是则输出 JSON 形态 usage 错误；
  - `Post` 变体增加隐藏别名 `po`（`#[command(alias = "po")]`），`p/b/c/v` 不动；
  - 顶层 help 增加文法模板一行（design.md §2.4）；
  - `output.rs`：`emit_err` 的 JSON 形态增补 `command` 字段（additive）；usage 错误的 JSON 形态同样携带 `command` 与 `exit_code`，且 `exit_code` **如实反映进程退出码**（usage 错误填 **2**，运行时错误仍为 1）；**顶层解析失败**（组/动词层无法确定命令）时信封与 JSON 的 command 标识统一填 **`usage`**。
- **依赖**：步骤②（usage 信封的规范示例需要新文法 example 就位）。**验证**：cargo build + clippy 全绿。

## 步骤④ core 层 example 字符串换新文法（纯文案，API 不动）

- **文件**：`repos/paperwork-core/src/ops/*.rs`（CLI 文法 example 字符串 **14 处**，rework 轮实测盘净）
- **清单**：`ops/thread.rs` L138 / L228 / L275 / L305 / L326 / L341（6 处）；`ops/manifest.rs` L32 / L80 / L105 / L151 / L194（5 处，后三处为 `brief create {} --title` 形态，rework 轮补漏）；`ops/contacts.rs` L22（1 处）；`ops/profile.rs` L61 / L91（2 处）。**实施前先执行全仓检索核对**：`rg "paperwork (post|brief|contacts|profile)" repos/paperwork-core/src`，预期输出即上述 14 处（其中 thread.rs L288、manifest.rs L172、contacts.rs L56/L98、profile.rs L20 等属不变文法形态，不在预期内、勿误刷）。
- **约束**：仅改字符串文案；函数签名、错误类型、锁/seq/格式逻辑零改动；`ops_tests.rs` 必须原样全绿。
- **依赖**：与步骤②③ 无代码依赖，**可并行**（不同 crate）；但合入验证以整 workspace 全绿为准。

## 步骤⑤ cli_integration.rs 改写 + 新增

- **文件**：`repos/paperwork-cli/tests/cli_integration.rs`
- **内容**：按 tdd.md §1 改写全部旧文法调用点（**29 处**：leader 清单 23 + 补充 L39/L297/L358 + rework 补漏 L220/L246/L270；L300 contacts create --title 保留 flag 不改）；L188 `read --from/--to` 原样保留；输出协议断言按 tdd.md §2 一字不改；按 tdd.md §3 新增用例（usage 信封 exit 2 含仅 PATH 形态、NAME/BODY 混淆面 validation 形态（F1）、旧文法 usage（example 断言为规范形态，F2）、多余位置参数、三级解析含异型文件与目录场景（F4）、implicit-mention 单数字段与三种不触发边界、窗口字段与过滤+limit 的 total 口径（F3）、--help/-V 穿透冻结（F5）、validate --type 含非法值与交叉形态、`--` 边界 send/edit、po 别名、--json usage 错误（exit_code=2）、顶层解析失败 command=usage、命名政策白名单断言（C6））。
- **依赖**：步骤①②③④全部完成。**验证**：cargo test 全量全绿 + clippy 全绿——这是文法落地的主验证点。

## 步骤⑥ CI smoke 换新文法

- **文件**：`.github/workflows/ci.yml` L56-106 与 L120-161（两处跨平台 smoke 段）
- **内容**：smoke 命令逐条换新文法（send/edit/create/add/remove 的参数序），新增一条旧文法触发 usage 信封（exit 2）的断言型 smoke。**exit 2 断言写法示范（m-6：不得沿用 grep 管道写法，会把 exit 2 误判为失败）**：

```yaml
# unix
- name: usage envelope smoke (exit 2)
  run: |
    set +e
    ./target/release/paperwork post send x.post.md --from alice "hi"
    code=$?
    set -e
    if [ "$code" -ne 2 ]; then echo "expected exit 2, got $code"; exit 1; fi
```

```yaml
# windows
- name: usage envelope smoke (exit 2)
  shell: pwsh
  run: |
    ./target/release/paperwork.exe post send x.post.md --from alice "hi"
    if ($LASTEXITCODE -ne 2) { throw "expected exit 2, got $LASTEXITCODE" }
```
- **依赖**：步骤⑤（本地全绿后才动 CI）。**验证**：语法检查 + 推送后 CI 实际绿。

## 步骤⑦ 文档：CHANGELOG 与 README

- **文件**：`CHANGELOG.md`、根 `README.md`、`repos/paperwork-cli/README.md`
- **内容**：
  - CHANGELOG 新增 `## [0.5.0]` 小节，`Changed (Breaking)` 列出全部文法变更 + **新旧文法迁移对照表**（`--from`→NAME 位置参数等逐条对照）；**必须先于 tag 落盘**（release.yml awk 对 CHANGELOG 有硬依赖）；**逐项列出四项消费者可感知变化并附迁移说明（X3）**：① `showing` 由仅超限出现改为恒显（出现语义变化）；② 新增退出码 2；③ category 词表扩为七类（usage）；④ 新增三个字段（implicit-mention / window / 错误 JSON command）；
  - 根 README 与 cli README 的全部命令示例刷新为新文法；
  - **随仓库新增 `SKILL.md`**（英文）：新文法速查 + 每个 tool 的典型调用示例 + 错误自愈提示。依据：`docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` **结论 C5 与 §8 风险 1（对冲三件套）**——随仓库发布 SKILL.md 对前置位置文法的首次误调率有显著补偿作用（业界 SOTA 实证）；
  - **adr-v1.md 示例层注记（SSOT 评审 m6）**：实施完成后在 `docs/dev/adr-v1.md` 顶部加一行 Superseded-by 注记：「本文 CLI Command Model 示例为 v0.4 及更早文法；v0.5.0 文法以 docs/ssot/specs/cli-ux-redesign/spec.md 为准」——**不改写历史内容**（ADR 不可变原则）。
- **依赖**：步骤⑤（行为定稿后写文档，避免返工）。

## 步骤⑧ 版本、tag、发布

- **内容**：
  - 双 crate 升 `0.5.0`；paperwork-cli 对 paperwork-core 的依赖版本改 `"0.5"`；
  - 打 tag `v0.5.0`（CHANGELOG 已落盘为前提）；
  - `publish.ps1` 按 **core → 等待 30 秒 → cli** 顺序发布（既有发布约定，crates.io 索引延迟）；**30 秒窗口风险（m-7）**：若 crates.io 稀疏索引传播更慢，cli publish 会因找不到 paperwork-core 0.5 而失败且脚本无重试——失败时手工重跑 cli publish 即可（core 已发布，无需重发）。
- **依赖**：步骤⑥⑦完成且 CI 绿。

## 步骤⑨ QA Review Book

- **文件**：`docs/reviews/v0.5-review-{实施完成日期}.md`
- **内容**：由独立验证 agent 实测（仿七段结构）：新文法全命令冒烟、旧文法 usage 信封迁移教学、`--` 边界、原路径优先、并发 send seq 无间隙、输出协议冻结核验（对照 spec §6）。
- **依赖**：步骤⑧前或后均可，但**不得由 impl agent 自评**（MainAgent工作编排.md：执行 agent 自报完成默认不可信）。

---

## 依赖图与并行度

```
①(ensure_suffix) → ②(签名+example) → ③(usage信封/output)
                                          ↘
                        ④(core 文案, 可与②③并行) → ⑤(集成测试) → ⑥(CI) → ⑦(文档) → ⑧(版本/发布) → ⑨(QA)
```

- **串行链**：① → ② → ③ → ⑤ → ⑥ → ⑦ → ⑧；⑨ 由独立 agent 承接。
- **可并行**：④ 与 ②③（不同 crate、无编译依赖交叉，仅共享 example 文法约定——以 spec §2 全表为准）。
- 单一 impl agent 顺序执行时按 ①→⑨ 编号推进即可；并行编排时遵守上述依赖边。
