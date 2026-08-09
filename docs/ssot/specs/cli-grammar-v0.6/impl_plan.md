# CLI 文法 v0.6: Impl Plan（实施计划）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布）
- 文档性质：实施计划（分层步骤 + 依赖 + 门禁）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令，最高优先级）
  - `docs/ssot/dev-principles/实现流程原则.md`（文档闭合后方可实现）
  - `docs/ssot/specs/cli-ux-redesign/impl_plan.md`（v0.5 实施体例与门禁先例）
- 前置门槛：本目录 spec/design/bdd/tdd 四份文档经对抗评审闭合后方可开始步骤(1)（实现流程原则.md）；实施者 role 文档：`docs/roles/cli-grammar-v0.6-implementer.role.md`（rework 补录 Nora ISSUE-m2，体例仿 v0.5 implementer role）。
- **交付边界（owner 显式约束，v0.6_feedbacks §一 (3)）**：本计划**不含**版本 bump、CHANGELOG 发布段、tag、publish 任何步骤；版本与发布时机由 owner 在功能稳定后另行裁定。

---

## 全局门禁（分阶段化，沿用 v0.5 F6 裁定）

- **步骤(1)(2)(3)期间**：每步完成后 `cargo build`（workspace）+ `cargo test -p paperwork-core`（ops_tests 恒绿）+ `cargo clippy --all-targets -- -D warnings` 全绿即可进入下一步；**cli_integration 允许红**（步骤(2)起 v0.5 位置文法调用必然红，属预期，由步骤(4)恢复）。
- **步骤(4)完成后**：`cargo test`（workspace 全量）+ clippy 全绿为硬门禁，后续步骤不得带红推进。
- core 层锁/seq/格式/hash 逻辑零改动；`ops_tests.rs` 一行不改且必须全绿（越界即回滚）。
- 输出协议冻结条款（spec §7）全程有效：ok/error 信封结构、七类 category、command 标识、JSON 既有 key 只增不改不删、纯 ASCII 输出契约。
- **禁止触碰主工作区 repos/ 下任何文件**（format-v2 已随合并进入本 worktree 分支，主工作区仅 docs/ 允许勘误同步）；步骤(0) 的合并由编排层授权执行（已于本轮落盘，见步骤(0) 记录），impl agent 仅在合并完成后的基线上工作。

---

## 步骤(0) 基线合并与全量盘点（编排层前置，非 impl 步骤）

- **内容**：将 cli-ux-v0.5 分支（v0.5.0 发布形态：位置文法 + v0.5 UX）与 master format-v2 分支（post create 删除、send 增 `--title` 建线程载荷、`--to/--participants` 已按 owner 追裁 D1/D2 删除、core v2 格式）合并为实现基线；合并由编排层授权在 worktree 内执行（合并提交 a07ad4c；文件格式与 D1/D2 行为以 format-v2 为准，CLI 参数文法以 v0.6 spec 为准叠加；基线事实链见 design §11 基线勘误记录）。合并后执行三项盘点：
  1. core example 点位：`rg -n "paperwork (post|brief|contacts|profile)" repos/paperwork-core/src`，确认 14 处清单（v0.5 基线实测值；format-v2 合并可能引入点位增减，以盘点输出为准）。**口径注明（rework 补录 Quinn minor）**：14 处为「需改写文案」口径（v0.5 rework 实测盘净），非全量命中数；命中行中属注释、断言、非 example 字符串的点位不计入改写清单；
  2. cli 集成测试调用点：`rg -n "\"(send|edit|create|add|remove)\"" repos/paperwork-cli/tests/cli_integration.rs`，输出 tdd §1 改写清单（命中行甄别规则见 tdd §1 末）；
  3. **SKILL.md 在场性盘点（rework 补录 Pete N7）**：确认仓库根 `SKILL.md` 存在且包含旧文法示例；盘点其全部命令行示例与错误自愈提示点位，输出步骤(6) 刷新清单；若盘点发现 SKILL.md 缺失或已移位，在 review 中报告并以实际在场文件为准。
- **门禁**：合并后 workspace `cargo build` + 全量测试绿（确认基线健康）方可放行步骤(1)。

## 步骤(1) core 层 example 字符串换 v0.6 文法（纯文案，API 不动）

- **文件**：`repos/paperwork-core/src/ops/thread.rs`、`ops/manifest.rs`、`ops/contacts.rs`、`ops/profile.rs`
- **内容**：步骤(0) 盘点出的全部 example 字符串换 v0.6 文法（`--author alice --message "Hello"` 形态；profile create 示例用 `--name`；brief 示例用 `--title/--entry/--entry-title`；contacts 示例用 `--profile`）。format-v2 若已改动部分点位，以盘点输出为准逐处刷新；不变文法形态的点位（如 validate 引导）勿误刷。
- **fix 文案内嵌旧文法专项（rework 补录 Quinn M-4）**：`manifest.rs` L79 / L150 / L193 三处（评审时基线行号，实施时以盘点实测校正）为 fix 文案而非 example 字段，同样内嵌 v0.5 位置文法调用示例，需一并换 v0.6 具名形态；盘点命令若未命中（fix 文案不含 paperwork 前缀），补充 `rg -n "post (send|edit)" repos/paperwork-core/src/ops/manifest.rs` 二次盘点确保不遗漏。
- **文件头文法注释刷新（rework 补录 Quinn minor）**：core 与 cli 各源文件头部若含文法描述性注释（如模块 doc comment 中的调用示例），随本步骤一并刷新，属改写范围。
- **约束**：仅改字符串文案；函数签名、错误类型、锁/seq/格式逻辑零改动；`ops_tests.rs` 原样全绿（tdd §5 防线）。
- **依赖**：步骤(0)。**验证**：cargo build + `cargo test -p paperwork-core` + clippy 全绿。

## 步骤(2) CLI 五文件 clap 签名重排与逻辑改造

- **文件**：`repos/paperwork-cli/src/cmd/post.rs`、`profile.rs`、`brief.rs`、`contacts.rs`、`validate.rs`
- **内容**：
  - 按 spec §2 全表重排 clap 签名：位置参数仅留 PATH；post send/edit 增 `--author/-a`（required）与 `--message/-m`（与 `--stdin` conflicts_with，二者共同构成正文必填通道）；`--seq` 保持必填 flag（u64）；profile create `--name` required；brief `--title/--entry/--entry-title` required；contacts add `--profile` required；
  - 短形式按 spec §4 全表落实（rework 裁定 F3 收窄：全 CLI 短形式仅 `-a/--author`、`-m/--message` 加既有全局 `-q`；`--mention`/`--seq`/`--stdin`/`--model`/`--title`/`--reply-to` 等其余全部 flag 仅长形式）；
  - 正文通道缺省判定（rework 裁定 F2）：`--message` 与 `--stdin` 皆缺时由 clap `required_unless_present` 组合直接产出 MissingRequiredArgument（落 usage 信封 exit 2，命令层无需任何管道）；仅 `--stdin` 时 stdin 读取逻辑沿用；`--message` 空值 trim 判定沿用（validation exit 1）；各分支 example 为单一静态规范可执行示例（采 `--message` 形态，rework 裁定 F5，不展示二选一占位形态）；
  - **`allow_hyphen_values` 属性（rework 裁定 F4，硬性指令）**：send 与 edit 两处 `--message` 均设 `allow_hyphen_values = true`（否则 `-` 开头正文值被 clap 拒收，S-SEND-10 不成立）；**其余 flag 不设**：`--author` 复核结论为不需要（署名值为纯名字字符串，`-` 开头名字不在语料预期内，且设后会把误写的裸 flag 静默吞为署名值，反而削弱 S-SEND-11 教学防线）；`--seq`（u64 类型自防）、其余 flag 同理不设；
  - **example 字符串刷新为 v0.6 文法**：各 cmd 文件内全部 example（空正文、无正文、not-found、format 教学、validate 未知后缀分支等）逐处核对刷新；resolve_body 的 send/edit 示例区分机制沿用 v0.5（edit 错误给 edit 示例）；
  - **v0.5 混淆面教学条款拆除**：「若已给出正文请检查是否遗漏 NAME 槽位」提示与 `--` 边界教学文案删除（混淆面结构性消亡，spec §5 第 3 条）；裸 `-xxx` 残留的 usage fix 改为引导 `--message` 形态；
  - 各子命令 after_help 示例换 v0.6 文法（design.md §2.1 文案为准；send/edit 不再需要 `--` 边界示例，改示范 `-` 开头 flag 值直传）；**after_help 教学要求（rework 裁定 F6，基线勘误后缩减，见 design §11）**：send after_help 需明示 `--title` 为建线程元数据载荷，仅首次写入生效，对既有线程静默忽略（行为登记，本轮不改运行时行为）；并注明 `--reply-to/--mention` 为糖衣 flag（值以 `@#N`/`@name` token 注入正文，D2/OQ-4）。原「`--to alice,bob` 收件人名单示范与三 flag 区分教学」随 `--to/--participants` 删除而废止。
- **依赖**：步骤(1)（core example 与 cli example 文法约定须一致，以 spec §2 全表为准）。**验证**：cargo build + clippy 全绿（cli_integration 此步必然红，属门禁允许范围）。

## 步骤(3) main.rs usage 信封静态规范示例换 v0.6

- **文件**：`repos/paperwork-cli/src/main.rs`
- **内容**：usage 信封机制（try_parse、exit 2、`--help/-V` 穿透、argv 扫描感知 `--json`、顶层失败 command 填 `usage`）**全部不变**；仅将各命令的静态规范示例（规范 usage 行 + 预置可执行示例）换 v0.6 文法；顶层 help 的 Grammar 模板行换为 `paperwork [global flags] <group> <verb> <PATH> --required-flag ... [--optional-flag ...]`（必填 flags 移出方括号，rework 补录 Pete N6）；**usage_fix base 文案与旧 flag 教学清单刷新（rework 补录 Quinn M-3）**：main.rs 中 usage_fix 的 base 示例文案（v0.5 位置文法形态）换 v0.6 具名形态，旧 flag 教学清单（`--from` 等 v0.4 残留指引、`--` 边界指引）同步清除或换向；疑似 flag 残留的 fix 文案按步骤(2) 新教学口径同步；usage 信封 message 字段天然携带 clap 报错原文中的多余位置参数值，无需额外实现（rework 补录 Pete N5，design §6）。
- **依赖**：步骤(2)。**验证**：cargo build + clippy 全绿。

## 步骤(4) cli_integration.rs 改写 + 新增

- **文件**：`repos/paperwork-cli/tests/cli_integration.rs`
- **内容**：按 tdd §1 改写全部 v0.5 位置文法调用点（只改参数层）；按 tdd §1b 处理断言语义翻转点（行号基线随合并提交 a07ad4c 失效，已按合并后基线重新盘点，以 tdd §1b 现行表为准）；tdd §3 输出协议断言一字不改；按 tdd §4 新增用例（缺必填 flag 各形态 usage exit 2 且 example 含完整必填形态、`--message`/`--stdin` conflicts usage exit 2、短形式等价（短形式集合 {-a, -m, -q}，F3）、v0.5 位置文法迁移、混淆面消亡确认、`--message` 值 `-` 开头直传无 `--`（注释钉住 allow_hyphen_values 属性依赖，F4）、`--mention` 无短形式负向断言、ASCII 契约扩展至新 usage 形态、命名政策白名单、S-SEND-17 `--title` 元数据 flag 对既有线程静默忽略（F6，基线勘误后缩减为仅 --title）、S-SEND-18/19、S-EDIT-09、S-READ-06~09、S-OUT-06）；冻结回归用例（showing/window、implicit-mention、三级解析、别名、三档输出）改参数层后原样通过。
- **依赖**：步骤(1)(2)(3)全部完成。**验证**：cargo test 全量全绿 + clippy 全绿，这是文法落地主验证点（硬门禁生效）。

## 步骤(5) CI smoke 换 v0.6 文法

- **文件**：`.github/workflows/ci.yml`（跨平台 smoke 段）
- **内容**：smoke 命令逐条换 v0.6 文法（send/edit/add/remove 的参数形态）；既有「旧文法触发 usage 信封 exit 2」的断言型 smoke 保留写法（`set +e`/`$LASTEXITCODE` 两平台范式沿用 v0.5 impl_plan 步骤(6)），触发样例改为 v0.5 位置文法（如 `post send x.post.md alice "hi"`）；smoke 样例全部纯 ASCII。
- **依赖**：步骤(4)（本地全绿后才动 CI）。**验证**：语法检查 + 推送后 CI 实际绿。

## 步骤(6) 文档示例：README 与 SKILL.md

- **文件**：根 `README.md`、`repos/paperwork-cli/README.md`、仓库根 `SKILL.md`、`docs/dev/adr-v1.md`（仅加一行注记）
- **内容**：
  - 根 README 与 cli README 的全部命令示例刷新为 v0.6 文法；
  - SKILL.md（英文）速查表与典型调用示例全部换 v0.6 文法，错误自愈提示更新（旧文法迁移教学示例换为 v0.5->v0.6 形态）；
  - adr-v1.md 顶部 Superseded-by 注记追加一行指向本目录 spec.md（不改写历史内容，ADR 不可变原则）；
  - **不写 CHANGELOG 发布段**（交付边界，见文首）。
- **依赖**：步骤(4)（行为定稿后写文档，避免返工）。

## 步骤(7) QA Review Book（独立验证，非 impl agent 职责）

- **文件**：`docs/reviews/v0.6-review-{实施完成日期}.md`
- **内容**：由独立验证 agent 实测：v0.6 文法全命令冒烟、缺必填 flag/conflicts usage 信封、v0.5 位置文法迁移教学、短形式等价性、混淆面消亡确认、输出协议冻结核验（对照 spec §7）。
- **依赖**：步骤(6)后执行；**不得由 impl agent 自评**（MainAgent工作编排.md：执行 agent 自报完成默认不可信）。

---

## 依赖图与并行度

```
(0)(基线合并, 编排层) -> (1)(core 文案) -> (2)(CLI 签名+逻辑) -> (3)(main.rs 示例) -> (4)(集成测试) -> (5)(CI) -> (6)(文档) -> (7)(QA)
```

- **串行链**：(1) -> (2) -> (3) -> (4) -> (5) -> (6)；(7) 由独立 agent 承接。
- (1) 与 (2)(3) 分属不同 crate，理论上可并行，但 example 文法约定以 spec §2 全表为唯一基准；单 impl agent 顺序执行按编号推进即可。
- **发布轮（版本 bump / CHANGELOG 发布段 / tag / publish）不在本计划内**：由 owner 在功能稳定后另行裁定并单独立项（v0.6_feedbacks §一 (3)）。
