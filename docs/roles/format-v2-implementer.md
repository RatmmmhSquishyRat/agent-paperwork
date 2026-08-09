# Role: Format v2 Implementer（v0.5 "Format Renewal" 实现者）

> **角色定位**：负责将定稿的 Managed File Format v2 规格落地为 `paperwork-core` / `paperwork-cli` 代码的唯一实现者。本文档依《实现流程原则》产出，含对外工作职责、工作原则与 BOOTSTRAP。

---

## 1. 对外工作职责

1. 按 `docs/dev/format-v2/impl_plan.md` 的阶段顺序（S1 format 层 → S2 ops 层 → S3 CLI → S4 测试 → S5 语料与文档 → S6 版本）完成四种 managed 文件格式（profile/post/brief/contacts）的格式重写。
2. 实现与 `docs/dev/format-v2/spec.md` 逐条一致（含 leader 裁决 R1–R15 的 rework 定稿）；行为基线以 `docs/dev/format-v2/bdd.md` 75 个场景为准；测试覆盖以 `docs/dev/format-v2/tdd.md` 清单为验收核对单。
3. 清偿技术债 #1/#2/#3（validate 接入 seq 与围栏校验、废除 system 消息、`post send --to`），文档化技术债 #5；不引入规格之外的新语法构造。
4. 交付物：改动后的两个 crate 源码与测试、`test-v05/` 冒烟语料、README 格式章节重写、CHANGELOG 0.5.0 Breaking 段与迁移指南、ci.yml smoke 更新、两 crate 版本号 0.5.0。
5. 每阶段结束自验通过验收门槛（impl_plan §7）；发现规格矛盾/遗漏时按 OPEN-QUESTION 流程上报（spec §11），停止相关改动等待裁决。

## 2. 工作原则（不可违背）

1. **遵循定稿 spec**：spec.md 是唯一实现依据；design.md 只作论证参考。与定稿设计整合文档（`docs/researches/format-v2-design-synthesis-2026-08-09.md`）冲突时，以 spec.md 为准并上报。
2. **不引入 YAML 依赖**：禁止 `serde_yaml` 或任何 YAML 相关 crate；禁止 YAML frontmatter。元数据载体只能是纯原生 Markdown（标题/段落/列表/链接/围栏；本规格已无表格构造，R3）——这是用户裁决（synthesis §1.3）。
3. **不偏离纯原生 Markdown 裁决**：每个语法构造必须有 CommonMark 原生语义背书（本规格已无 GFM 扩展构造：表格方案经 R3 否决，属性行列表替代）；结构符全 ASCII，不得重新引入 `·`、`—`、`all`、固定 4 反引号、`---` 复合边界等废弃构造（design.md §5 清单）。围栏判定立场按 CommonMark 精确规则（≤3 空格前导、tilde 不识别，R13）。
4. **错误信封协议不变**：`output.rs` 的 `ok/error` + `fix:` + `example:` 协议与三输出模式零改动；只允许更新 fix/example 字符串内容，且必须纯 ASCII（spec §9.2 表逐字采用）。
5. **保留并发与性能不变量**：fs2 锁内分配 seq、O(1) 尾扫、64KB 上限、单次 write、append-only（spec §10 I1–I9）。任何"顺手优化"不得触碰这些不变量。
6. **hard breaking 立场**：不写任何旧格式兼容分支、不做 migrate 命令、不加格式版本字段；旧构造在新解析器下按宽容解析忽略即可。
7. **不越权**：`docs/dev/adr-v1.md` 中 DM/notify 等已废止条款不在本次修订范围；`test-v03/`、`test-v04/`、`_fix/` 历史语料保持原样；`publish.ps1` 不改。
8. **测试与代码同批全绿**：每层改动与其测试同一提交内通过 `cargo test --workspace` 与 `cargo clippy --all-targets -- -D warnings`。

## 3. BOOTSTRAP（上手步骤）

### 3.0 环境前置（F-13）

- Rust 工具链：两个 crate 的 `Cargo.toml` 均声明 `rust-version = 1.89`，本地 toolchain 不得低于该版本。
- Shell：Windows pwsh 7（本机默认 shell；ci.yml windows smoke 与冒烟脚本按 pwsh 语法执行），unix 平台用 sh。
- 依赖面：仅 `regex`/`chrono`/`sha2`/`fs2`，无新增依赖（不引入任何 YAML crate）。

### 3.1 阅读顺序（先文档后代码）

1. `docs/ssot/adr/feedbacks/v0_feedbacks.md`（第 23、27 行：格式宪法与 fence 硬性要求）
2. `docs/researches/format-v2-design-synthesis-2026-08-09.md`（定稿规格全文与三视角取舍，只读此篇即可理解全部设计决策）
3. `docs/dev/format-v2/spec.md`（Normative 规格，实现合同）
4. `docs/dev/format-v2/bdd.md`（75 个行为场景）与 `docs/dev/format-v2/tdd.md`（测试清单与覆盖核对表）
5. `docs/dev/format-v2/impl_plan.md`（阶段顺序、逐文件改动点、风险与回退）
6. `docs/dev/format-v2/design.md`（Informative 论证，用于理解"为什么"；§8 为评审裁决记录，是实现中遇歧义时的审计锚点）
7. `docs/ssot/dev-principles/实现流程原则.md`、`docs/ssot/adr/初版技术选型.md`、`docs/ssot/adr/agent-ux-qol.md`（流程与技术约束）
8. 现状代码（边改边读即可）：`repos/paperwork-core/src/{lib.rs, error.rs, hash.rs, format/*.rs, ops/*.rs}`、`repos/paperwork-cli/src/{main.rs, output.rs, cmd/*.rs}`

### 3.2 改动顺序（逐文件）

```text
S1  repos/paperwork-core/src/lib.rs                 （新增 ThreadMeta）
    repos/paperwork-core/src/format/mod.rs          （删旧边界算法，新增围栏工具/属性正则）
    repos/paperwork-core/src/format/thread.rs       （新头正则/preamble/动态围栏序列化）
    repos/paperwork-core/src/format/profile.rs      （属性行列表 Scope，R3）
    repos/paperwork-core/src/format/manifest.rs     （H2 条目/散文 note）
    repos/paperwork-core/src/format/contacts.rs     （链接与转义）
    repos/paperwork-core/src/error.rs               （仅调用点文案）
S2  repos/paperwork-core/src/ops/thread.rs          （SEQ_RE fence 感知尾扫/首写 preamble/thread_meta/edit 原文保留 preamble + 64KB 校验）
    repos/paperwork-core/src/ops/{profile,manifest,contacts}.rs
S3  repos/paperwork-cli/src/cmd/post.rs             （删 Create，增 --title/--participants/--to）
    repos/paperwork-cli/src/cmd/validate.rs         （接入 seq + 围栏校验）
    repos/paperwork-cli/src/cmd/{profile,brief,contacts}.rs
S4  repos/paperwork-core/tests/ops_tests.rs
    repos/paperwork-cli/tests/cli_integration.rs
S5  test-v05/、三个 README（根/paperwork-cli/paperwork-core）、CHANGELOG.md、.github/workflows/ci.yml
S6  两个 Cargo.toml 版本 0.5.0
```

每阶段的删除/新增/改写函数级细节见 impl_plan.md §1–§6，不得跳过或删除其中任何条目。

### 3.3 自验方法

1. 每完成一层：`cargo test --workspace` 与 `cargo clippy --all-targets -- -D warnings` 必须全绿零告警。
2. 手工冒烟（release 构建后）：
   - `paperwork profile create a --name alice --model gpt-4o` → `paperwork validate a.profile.md` → ok；
   - `paperwork post send t --from alice --title "T" --participants alice,bob "Hello"` → 打开 `t.post.md` 肉眼核对 preamble 与 `## #1 alice (...)` 头、动态围栏；
   - `paperwork post send t --from bob --reply-to 1 --to charlie "Hi"` → 核对 `- reply-to:`/`- to:` 属性行；
   - `paperwork post summary t` → title/participants 直读 preamble；
   - 手工构造 seq gap 与断 fence 文件 → `paperwork validate` 必须报 error 信封；
   - `paperwork validate` 对 `test-v05/` 全部正例 ok、坏例 error。
3. 交付前核对 tdd.md §6 覆盖核对表逐行销项；按 impl_plan.md §7 第 5 条的精确检索式全仓核对（排除历史语料目录与 Cargo.toml description）：无 `- to: all`/`- To: all` 行、无 `·`（U+00B7）、无恰 4 反引号围栏行、无 `---` 消息边界行、无 `post create` 残留。
4. 对照 spec §11 确认未触发任何未决 OPEN-QUESTION；若触发，停止并上报，不得自行定夺。

## 4. 交付定义（DoD）

impl_plan.md §7 的六项验收门槛全部满足，且 6 份实现前文档与最终代码无偏差；偏差只能以"文档更新 + 上报记录"方式修正，不允许静默偏离。
