# CLI 文法 v0.6: 文档索引

- 日期：2026-08-09
- 版本：v0.6（本轮不发布：不 bump 版本、不打 tag、不 publish、不写 CHANGELOG 发布段，owner 显式约束）
- **状态：文档初稿落盘（2026-08-09），待对抗评审**（实现流程原则.md：评审闭合后方可开工）

---

## 一、本目录文档清单与阅读顺序

| 顺序 | 文件 | 内容 |
|---|---|---|
| 0 | `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（目录外） | owner 指令落盘，全套文档的 architectural basis，最高优先级 |
| 0b | `docs/researches/cli-grammar-v06-reassessment-2026-08-09.md`（目录外） | 三视角重评估报告（基线核实 / 错误注入矩阵 / path-first 复评 / Rejected Alternatives 再评估） |
| 1 | `spec.md` | 行为规范：新文法三规则、逐命令签名契约全表、短形式全表、错误 category 映射、输出协议冻结条款、兼容策略 |
| 2 | `design.md` | 设计方案：逐 tool 动线与参数布局论证、三方案对比与 owner 裁决依据、path-first 复评否决记录、短形式论证、互斥语义设计、Rejected Alternatives 状态更新 |
| 3 | `bdd.md` | 行为场景：Given/When/Then 覆盖全部命令的正常与错误路径（重点：缺必填 flag / 互斥冲突 / 短形式等价 / 旧文法迁移 / 混淆面消亡） |
| 4 | `tdd.md` | 测试计划：v0.5 文法调用点改写清单（按类）、输出协议保留断言、新增用例、ops_tests 零改动声明 |
| 5 | `impl_plan.md` | 实施计划：步骤 (0) 至 (7) 带依赖与门禁；**无发布步骤** |

**建议阅读顺序**：先读 v0.6_feedbacks.md（为什么改、改什么的硬指令）与研究文档（论证全貌）-> spec.md（改成什么样）-> design.md（为什么这么改）-> bdd.md（行为验收）-> tdd.md（测试验收）-> impl_plan.md（怎么改）。

## 二、与 v0.5 文档集（docs/ssot/specs/cli-ux-redesign/）的关系

**继承范围（冻结，逐条有效）**：

- 输出协议：ok/error 信封结构、七类 error category、退出码 0/1/2、`--json/--plain/-q` 三档、JSON key 只增不改不删、command 标识、纯 ASCII 输出契约（v0.5 spec §4/§6，本集 spec §5/§7 引用声明）；
- usage 信封机制：try_parse、静态规范示例、`--help/-V` 穿透、argv 扫描感知 `--json`、顶层失败 command 填 `usage`（仅示例文案换 v0.6）；
- ensure_suffix 三级解析、隐藏别名 `p/b/c/v/po`、validate `--type`、implicit-mention / showing / window 输出增补、分阶段门禁先例；
- 遗留项裁决（U-03/U-04/U-09/U-13 延后项、U-02/U-05/R-08/F-09 拒绝项）沿用（design.md §8）。

**取代范围（本集覆盖 v0.5 对应章节）**：

- 核心文法三规则（v0.5 spec §1 由本集 spec §1 替换：位置参数仅剩 PATH、必填一律具名 flag、flag 唯一语义）；
- 命令签名全表（v0.5 spec §2/§3 由本集 spec §2/§3 替换）；
- NAME/BODY 混淆面裁定与三重教学补偿（v0.5 design §2.5/§7.5 F1）废止：混淆面结构性消亡；
- `--` 边界教学条款（v0.5 spec §4.2/§4.3 相关）废止：正文经 `--message` flag 值直传；
- Rejected Alternatives（v0.5 design §8）状态更新：#3（--as）与 #4（SEQ 保留 flag）被翻转，其余维持（本集 design §7）。

**冲突裁定规则**：与 v0.5 文档冲突处，以 `docs/ssot/adr/feedbacks/v0.6_feedbacks.md` 为准，其次以本目录文档为准；v0.5_feedbacks §二.1 与 v0_feedbacks #3.1 的字面条款已被 owner 显式翻转（翻转记录见 v0.6_feedbacks §三 与 v0.5_feedbacks §三 末尾追加段）。

## 三、architectural basis 引用

| 文件 | 作用 |
|---|---|
| `docs/ssot/adr/feedbacks/v0.6_feedbacks.md` | 本次 owner 指令（接受 action-first；NAME/BODY 改具名必填 flag `--author/--message`；本轮不发布），最高优先级 |
| `docs/researches/cli-grammar-v06-reassessment-2026-08-09.md` | 三视角重评估（Sena/Vera/Milo）与 path-first 复评、错误注入矩阵、Rejected Alternatives 再评估 |
| `docs/ssot/specs/cli-ux-redesign/`（spec/design/bdd/tdd/impl_plan/README） | v0.5 文档集：体例模板与继承基线 |
| `docs/ssot/adr/feedbacks/v0_feedbacks.md` | 前序 owner ADR（#3.1 字面条款被翻转，其余叠加生效） |
| `docs/dev/adr-v1.md`（ADR-011） | 硬约束：stateless / path-explicit / 无登录 |
| `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` | 业界基准（git `-m` 惯例、SKILL.md 迁移补偿实证） |

## 四、治理状态

- [x] owner 裁决落盘（v0.6_feedbacks.md）与 v0.5_feedbacks 翻转记录追加
- [x] 三视角重评估研究落盘（cli-grammar-v06-reassessment-2026-08-09.md）
- [x] spec / design / BDD / TDD / impl_plan 五份文档齐备（本目录）
- [ ] 对抗评审 loop 至闭合（任务 #13，实现流程原则.md 门槛）
- [ ] 基线合并（cli-ux-v0.5 + format-v2，编排层执行，impl_plan 步骤 (0)）
- [ ] 按 impl_plan 步骤 (1) 至 (7) 实现（评审闭合后）
- [ ] 发布（不在本轮：owner 于功能稳定后另行裁定）
