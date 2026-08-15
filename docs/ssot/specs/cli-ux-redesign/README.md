# CLI UX 重设计 v0.5.0 — 文档索引

> **Superseded-by note (v0.6)**: 本套件的 CLI 文法层已由 v0.6 具名文法整体取代，现行文法以 `docs/ssot/specs/cli-grammar-v0.6/spec.md` 为准；本套件保留为历史治理档案，历史正文不可改写。（The CLI grammar layer of this suite is superseded by the v0.6 named-flag grammar, authoritative in `docs/ssot/specs/cli-grammar-v0.6/spec.md`; this suite is retained as a historical governance archive. Historical content below is immutable.）

- 日期：2026-08-09
- 版本：v0.5.0
- **状态：文档层已闭合（2026-08-09），NF-2/NF-3 非阻塞补录已完成，待按 impl_plan 开工**（三份一轮评审报告与闭合复核报告见 `docs/reviews/cli-ux-redesign-doc-review-{ssot,agent-ux,feasibility,closure}-2026-08-09.md`；合并修复清单已逐条落实，design.md §7.4/§7.5 为两轮裁定记录；NF-1（并行 format-v2 线程版本边界）待编排层裁定）

---

## 一、本目录文档清单与阅读顺序

| 顺序 | 文件 | 内容 |
|---|---|---|
| 0 | `docs/ssot/adr/feedbacks/v0.5_feedbacks.md`（目录外） | owner 指令落盘——全套文档的 architectural basis，最高优先级 |
| 1 | `spec.md` | 行为规范：核心文法总纲与三条规则、逐命令签名契约、输出 envelope 契约、冻结条款 |
| 2 | `design.md` | 设计方案：每个 tool 一节独立完整设计、文法规则论证、遗留项裁决总表、Rejected Alternatives、兼容策略 |
| 3 | `bdd.md` | 行为场景：Given/When/Then 覆盖全部命令的正常与错误路径 |
| 4 | `tdd.md` | 测试计划：旧文法调用点改写清单（行号）、输出协议保留断言清单、新增用例、ops_tests 零改动声明 |
| 5 | `impl_plan.md` | 实施计划：①→⑨ 带依赖步骤、并行/串行标注、验证门禁 |
| — | `docs/roles/cli-ux-redesign-implementer.role.md`（目录外） | 实施者 role 文档：对外工作职责 / 工作原则 / BOOTSTRAP |

**建议阅读顺序**：先读 v0.5_feedbacks.md（为什么改、改什么的硬指令）→ spec.md（改成什么样）→ design.md（为什么这么改 + 遗留项结案）→ bdd.md（行为验收）→ tdd.md（测试验收）→ impl_plan.md（怎么改）。实施者按 role 文档 BOOTSTRAP 顺序入职。

## 二、architectural basis 引用

| 文件 | 作用 |
|---|---|
| `docs/ssot/adr/feedbacks/v0.5_feedbacks.md` | 本次 owner 指令（路径与名字前置必填位置参数；每 tool 独立完整设计；授权文法涌现），优先级高于 v0.4-ux-review 旧提议 |
| `docs/researches/cli-ux-agent-visible-output-research-2026-08-08.md` | 现状全貌调研（v0.4.0 命令、输出、格式、agent 理解模型） |
| `docs/researches/ux-open-items-backlog-2026-08-08.md` | 遗留项盘点与编号体系（U-xx / R-xx / F-xx / N-xx / Q-xx），裁决对象 |
| `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` | 业界 CLI UX 基准（规则 3 判据的外部佐证） |
| `docs/dev/adr-v1.md`（ADR-011） | 硬约束：stateless / path-explicit / 无登录——本次文法不得违背 |
| `docs/ssot/adr/feedbacks/v0_feedbacks.md` | 前序 owner ADR（#3.1 content 置末继续有效） |
| `docs/reviews/v0.4-ux-review-2026-08-01.md` | 旧提议清单；§1 `--as`、§2 env 回退、§5 content-first 已被 v0.5_feedbacks 覆盖并裁决拒绝 |

## 三、治理状态

- [x] owner 指令落盘（v0.5_feedbacks.md）
- [x] spec / design / BDD / TDD / impl_plan 五份文档齐备
- [x] role 文档（职责 / 原则 / BOOTSTRAP）齐备
- [x] **对抗评审第一轮（已完成）**：三份批判性评审报告落 `docs/reviews/`（SSOT/pillars、agent-ux、feasibility），均判未闭合
- [x] **rework 轮修订（已完成，2026-08-09）**：编排层裁定 F1-F7 + 事实性修正 X1-X4 + 全部 Minor 项逐条落实（对照表见当轮汇报；两轮裁定记录见 design.md §7.4/§7.5）
- [x] **闭合复核（已完成，2026-08-09）**：`cli-ux-redesign-doc-review-closure-2026-08-09.md` 判定文档层闭合（36 项独立问题：已修复 35 + 已修复带轻微残留 1）；F1-F7 7/7 逐字落实；事实抽查 29 处 8/8、14 处 5/5 吻合且全量枚举盘净
- [x] **非阻塞补录（已完成，2026-08-09）**：NF-2（bdd S-SEND-14 usage 负形态 + spec §4.3 `--` 教学条款 + tdd §3 用例）、NF-3（tdd §3 补 S-CREATE-02/S-CREATE-03/S-PROF-02/S-READ-04/S-BRIEF-07/S-CONTACTS-05 六行映射）
- [ ] 按 impl_plan 开始实现（前置：编排层对 NF-1 两线程版本边界裁定；裁定若影响 spec §6 基线须回写适配注记）
- [x] **实现后事追加（2026-08-15，不改写上文）**：v0.5.0 位置文法实现已随 0.5.0 发布（tag v0.5.0 @ 70f7e43），随后被 v0.6 具名文法整体取代；v0.6 实现已随 master @ 3829fd9 三方合并生效。上一未勾项按既成事实结案（实现与裁定均已成为历史），现行文法见文首 Superseded-by 注记。
