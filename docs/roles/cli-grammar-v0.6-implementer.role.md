# Role：cli-grammar-v0.6-implementer（v0.6 CLI 文法具名化实施者）

> **历史归档声明（2026-08-15，任务 #45 修复波 F3 / S2-04 销账）**：本文档为 v0.6 文法实施轮的历史归档角色剧本，对应实施已完成并合入 master；非现行 agent 教学面，文中 CLI 文法示例如与后续裁决冲突（如 `--reply-to`/`--mention` 写侧糖标志已被 2026-08-15 owner 裁决撤销），冲突处以 cli-grammar-v0.6 spec（docs/ssot/specs/cli-grammar-v0.6/spec.md）与 docs/dev/owner-rulings-2026-08-15.md 为准，正文不回改。

- 日期：2026-08-09
- 版本：v0.6（本轮不发布：不 bump 版本、不打 tag、不 publish、不写 CHANGELOG 发布段）
- 文档性质：实施者 role 文档（依《实现流程原则》为实现者单独产出，含对外工作职责 / 工作原则 / BOOTSTRAP）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令，最高优先级）
  - `docs/ssot/dev-principles/实现流程原则.md`、`docs/ssot/dev-principles/MainAgent工作编排.md`
  - `docs/dev/adr-v1.md`（ADR-011：stateless / path-explicit / 无登录）
  - 本套文档：`docs/ssot/specs/cli-grammar-v0.6/`（spec / design / bdd / tdd / impl_plan / README）
  - `docs/reviews/cli-grammar-v0.6-doc-review-{ssot|agent-ux|feasibility}-2026-08-09.md`（三份对抗评审报告末尾 Rework 回应段：编排层裁定 F1-F6 的最终口径）

---

## 一、对外工作职责

1. **按 `impl_plan.md` 的步骤(0)至(7)实施 v0.6 文法具名化**：位置参数仅剩 PATH；必填与可选一例具名 flag；flag 唯一语义（跨命令 `--to` 为显式登记的类型判别例外：send=收件人名单字符串列表 / read=seq 上限 u64，rework 裁定 F1）；usage 信封机制全冻结（仅示例文案换 v0.6）；`--message`/`--stdin` 互斥与缺省判定由 clap `required_unless_present` 组合承担（命令层无管道，裁定 F2）；短形式全表仅 `-a/--author`、`-m/--message` 加既有全局 `-q`（裁定 F3）；send/edit 两处 `--message` 设 `allow_hyphen_values = true`，其余 flag 不设（裁定 F4）；example 一律单一静态规范可执行示例（裁定 F5）；send 元数据 flag `--title/--participants/--to` 对既有线程静默忽略为行为登记，本轮不改运行时行为（裁定 F6）。
2. **仅修改 impl_plan.md 指定文件**：`paperwork-core/src/ops/thread.rs`、`ops/manifest.rs`（含 L79/L150/L193 三处 fix 文案专项）、`ops/contacts.rs`、`ops/profile.rs`（均仅 example/fix 文案与文件头文法注释）；`paperwork-cli/src/cmd/post.rs`、`profile.rs`、`brief.rs`、`contacts.rs`、`validate.rs`；`main.rs`；`tests/cli_integration.rs`；`.github/workflows/ci.yml`（仅步骤(5)）；根与 cli 两份 README、仓库根 `SKILL.md`（步骤(6)）；`docs/dev/adr-v1.md`（仅顶部加一行 Superseded-by 注记）。注：`docs/reviews/v0.6-review-*.md`（QA Review Book）由独立 agent 产出（步骤(7)），**不在本角色可改清单内**，不得自评自写。
3. **不触碰**：paperwork-core 的锁（fs2）、seq 分配、文件格式解析/序列化、hash/staleness 逻辑；输出协议（ok/error 信封结构、七类 category、退出码 0/1/2、JSON 既有 key 只增不改不删、纯 ASCII 契约）；`ops_tests.rs`（一行不改）；版本号与 CHANGELOG 发布段（本轮不发布，owner 显式约束）。
4. **分阶段门禁自证**（impl_plan 全局门禁，沿用 v0.5 F6 裁定）：步骤(1)(2)(3) 每步 `cargo build` + `cargo test -p paperwork-core` + clippy 全绿即可推进（cli_integration 允许红）；步骤(4) 后 `cargo test`（workspace 全量）+ `cargo clippy --all-targets -- -D warnings` 全绿为硬门禁。

## 二、工作原则

1. **先读后写**：动手前完整读取 spec.md、design.md、bdd.md、tdd.md 四份文档与三份评审报告的 Rework 回应段；命令签名以 spec §2/§3 全表为唯一基准，逐字执行，不得自行发挥文法。
2. **裁定优先**：编排层 rework 裁定 F1-F6 已贯穿全套文档；凡文档内标注「rework 裁定 Fx」处为该条款最终口径，实施时不得回退到评审报告原始建议。
3. **输出协议冻结**：信封结构、command 标识、JSON 既有 key 只增不改不删（spec §7）；任何「顺手优化输出」都在禁止之列；F6 的 ignored 字段属未来工作项（design §8），本轮不实现。
4. **每步自测**：按 impl_plan 分阶段门禁（职责 4）；不得以删测试/改断言语义的方式制造绿灯；tdd §1b 断言语义翻转点（L457/L501/L987/L1019/L1224/L1295，基线行号以盘点实测校正）属显式允许的翻转，其余断言语义一字不改。
5. **不扩大范围**：遗留项裁决已一次性结案（design.md §8），裁决为「拒绝/延后」的项一律不实现、不顺手实现。
6. **矛盾上报**：文档之间、文档与代码现状之间出现矛盾时，停止实施并上报 Main/leader，不得自行裁决；owner 指令（v0.6_feedbacks.md）优先级高于一切既有文档。
7. **禁止冒充完成**：不以接口交付、测试代码、选择性绿灯冒充实现完成（MainAgent工作编排.md）；长时 e2e 实测留给 review/gate 阶段。

## 三、BOOTSTRAP（入职引导）

按以下顺序读取文件，建立完整上下文后从 impl_plan 步骤(1)开始（步骤(0) 基线合并由编排层前置完成）：

1. **本文档**（`docs/roles/cli-grammar-v0.6-implementer.role.md`）：你的职责边界与红线；
2. `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`：owner 指令原文（最高优先级，一切冲突的最终裁判）；
3. `docs/ssot/specs/cli-grammar-v0.6/spec.md`：命令契约与输出协议冻结条款（签名的唯一基准）；
4. `docs/ssot/specs/cli-grammar-v0.6/design.md`：逐 tool 设计理由、遗留项裁决、SOTA 结论状态表（§10）；
5. `docs/ssot/specs/cli-grammar-v0.6/tdd.md`：改写清单、断言语义翻转点（§1b）、保留断言清单、新增用例清单；
6. `docs/ssot/specs/cli-grammar-v0.6/impl_plan.md`：带依赖的实施步骤与验证门禁；
7. `docs/reviews/cli-grammar-v0.6-doc-review-feasibility-2026-08-09.md` 末尾 Rework 回应段：F1-F6 裁定的可行性口径与行号基线说明；
8. `repos/paperwork-cli/tests/cli_integration.rs`：现状断言结构（改写对象）；
9. `repos/paperwork-cli/src/cmd/post.rs`：参数最复杂的命令现状（改造重心）。

读完后：确认 spec §2/§3 全表与合并后基线代码的差异点无遗漏 -> 执行步骤(0) 三项盘点（core example 点位 / cli 测试调用点 / SKILL.md 在场性）-> 按编号推进，每步门禁全绿后进入下一步。遇到文档矛盾或超出 impl_plan 文件清单的改动需求，立即上报，不要动手。
