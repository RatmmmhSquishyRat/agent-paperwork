# CLI UX 重设计 v0.5.0 文档集对抗性评审 — SSOT/pillars 合规视角

- 日期：2026-08-09
- 评审视角：视角一 — SSOT/pillars 合规（第三方批判性评审，立场为挑错而非背书）
- 评审对象（全部精读）：docs/ssot/specs/cli-ux-redesign/ 下 spec.md / design.md / bdd.md / tdd.md / impl_plan.md / README.md；docs/ssot/adr/feedbacks/v0.5_feedbacks.md；docs/roles/cli-ux-redesign-implementer.role.md
- 评审依据：docs/ssot/dev-principles/实现流程原则.md 与 MainAgent工作编排.md；docs/ssot/adr/初版技术选型.md、agent-ux-qol.md、feedbacks/v0_feedbacks.md；docs/dev/adr-v1.md（ADR-011）；docs/ssot/pillars/paperwork-init-conversation/ 两份 session-log；docs/reviews/v0.4-ux-review-2026-08-01.md；docs/researches/ux-open-items-backlog-2026-08-08.md
- 评审方法：逐文档精读 + 与依据文档逐条交叉核对 + 代码层实证核验（tdd/impl_plan 全部行号抽查、现状行为核验、category 词表核验、ci.yml 区段核验、SOTA 引用溯源）

---

## 一、总体结论

| 严重度 | 数量 |
|---|---|
| Critical | 0 |
| Major | 4 |
| Minor | 8 |

**是否闭合：未闭合。** 4 项 Major 须修复并经复核后方可进入实现阶段（实现流程原则.md 实现门槛）。

## 二、核验通过项（对抗立场下的无问题确认，均有实证）

1. **治理结构合规**：实现流程原则.md 要求的五份文档（spec/design/BDD/TDD/impl_plan）齐备，role 文档含对外工作职责/工作原则/BOOTSTRAP 三要素（README.md L11-19 清单；role.md 一/二/三章）。
2. **ADR-011 硬约束逐条合规**：stateless / path-explicit / 无登录 / 无状态目录 / 任意 CWD——新文法全部命令携带显式 PATH，无 env 身份回退（U-02 裁决拒绝正是守住显式与无状态），无工作区概念（spec.md 全文；design.md §7.2；bdd.md 前言 L13）。
3. **pillars 继承**：session-log user msg5.1「每个原语无层级、独立成工具」与 msg4.8「stub-first 流畅体验」落实为 design.md §2-6 每 tool 一节独立完整设计、post send 自动建线程保留；owner 干预形式（msg5.2：只写 ssot/pillars/ADR）与 v0.5_feedbacks.md 的形态一致。
4. **v0_feedbacks 继承**：#3.1 content 置末（BODY/NEW_BODY 恒末位，spec.md §1.2/§1.5）、无登录只给名字、无 .paperwork 目录、四类类型后缀——全部正确继承，无违背。
5. **v0.5_feedbacks.md 忠实性**：三条 owner 指令以引文形式落盘（§一 ①②③）；优先级声明（§三）完整覆盖 ux-review §1(--as)/§2(env)/§5(content-first) 三项拒绝，并与 backlog U-01/U-02/U-05 映射一致。
6. **遗留项裁决覆盖完整**：backlog 全部未决项（U-01~U-15、R-01、R-08、F-09、N-01~N-03）在 design.md §7.1/§7.2/§7.3 均有处置（本次解决 8 项、裁决拒绝 4 项、延后 4 项）；v0.4-ux-review 全 15 节提议均有明确裁决，无悬空项。
7. **tdd 行号高度可信**：对 cli_integration.rs（v0.4.0，382 行）逐行复核——26 处改写点（L19/L39/L52/L57/L69/L93-94/L112/L118/L137/L142/L161/L166/L177/L178/L182/L199/L200/L226/L247/L250/L271/L297/L306/L325/L358）与 L188 保留点、§2 全部保留断言行号全部吻合；合计式（leader 清单 23 + 补充 3 = 26 改写 + 1 保留）自洽。
8. **impl_plan 点位多数属实**：ensure_suffix 位于 cmd/mod.rs L24-34（实测）；CLI 层 example 点位 post.rs L162/L347/L352、validate.rs L54、profile.rs L212 全部命中；core 层 ops/thread.rs L138/L228/L275/L305/L326/L341、ops/contacts.rs L22、ops/profile.rs L61/L91 全部命中；ci.yml 两段 smoke（L56-106/L120-161，文件共 165 行）吻合。遗漏问题见 ISSUE-M3。
9. **签名内部一致性**：spec §2 全表 15 条命令签名与 spec §3 逐命令契约、design §2-6、bdd 全部场景逐一比对，未发现签名漂移；退出码 0/1/2 语义在 spec §4.4、bdd、tdd 三处一致；对照 output.rs 现状 key 集合，JSON「只增不改不删」条款成立（新增 command/implicit-mention/window 均为新 key）。
10. **category 词表核验**：spec §4.2 声明的运行时六类（format/validation/io/not-found/already-exists/not-allowed，含 MessageTooLarge→validation）与 paperwork-core/src/error.rs L81-91 category() 实现逐一吻合。

---

## 三、问题清单

### Major

#### ISSUE-M1 bdd S-SEND-08 行为契约在既定文法下不可实现（Severity: Major）

- 证据：bdd.md L61-65（S-SEND-08）；tdd.md L118（§3 新增用例首行同文）；spec.md L75-83（§3.1：NAME 必填第 2 位、BODY 可选）；repos/paperwork-cli/src/cmd/post.rs L342-353（现状 resolve_body：无 body 且无 --stdin → validation 错误）。
- Expected：「缺 NAME」唯有位置参数仅给 PATH（`post send <PATH>`）时才构成 clap 用法错误，exit 2；「PATH + 单个字符串」形态下该字符串必然绑定到必填的 NAME 槽位。
- Actual：场景书写命令为 `post send standup.post.md "body text"`（两个位置值）。按 spec 既定文法，clap 必将 "body text" 绑定 NAME、BODY 缺省，随后落入 validation 分支 exit 1（与 S-SEND-05 同路径），永不产生 usage exit 2。括号注「或位置参数不足」不能拯救已书写的命令。该场景无法实现，且与文法契约直接矛盾；tdd §3 对应用例继承同一错误。
- 建议：① 将 S-SEND-08 命令改为 `paperwork post send standup.post.md`（仅 PATH，NAME 缺失 → usage exit 2）；② 新增场景明确「PATH + 单个字符串」形态按 NAME 绑定、BODY 缺省处理（validation exit 1）；③ design 补一句裁定：CLI 无法区分第二位置参数是 NAME 还是 BODY，此为位置文法的已知边界；④ tdd §3 同步修正。

#### ISSUE-M2 ensure_suffix 第③级表述与只读命令行为漂移（Severity: Major）

- 证据：spec.md L299（§5 表：「都不存在 → 创建补后缀路径」）；design.md L298（§7.4 裁定 7 同文）；bdd.md L290-294（S-PATH-04：read 两者均不存在时报 not-found，不创建文件）；impl_plan.md L27（步骤①注：read 类「主要受益于第①②级」，未排除创建）。
- Expected：第③级应为「路径决策」语义——以补后缀路径为目标路径；物理建文件仅发生于写命令（post send 自动建线程、各 create）。只读命令（read/summary/validate）在第③级只报 not-found（S-PATH-04 已如此要求）。
- Actual：spec §5 与 design §7.4 的字面表述「创建补后缀路径」允许实现者把 read 命令也实现为物理创建空文件，与 bdd S-PATH-04 冲突；impl_plan 步骤①未明示排除。若按 spec 字面实现，read 将产生文件副作用，违背 ADR-011 stateless 原则与只读语义。
- 建议：① spec §5 与 design §7.4 裁定 7 将第③级改写为「都不存在 → 以补后缀路径为目标路径（是否物理创建由命令写语义决定：send/create 创建，read 类报 not-found）」；② impl_plan 步骤①补一句「第③级对只读命令仅为路径决策，不创建文件」；③ bdd S-PATH-04 无需改动。

#### ISSUE-M3 impl_plan 步骤④「已知位置」清单遗漏 3 处旧文法 example（Severity: Major）

- 证据：impl_plan.md L55（清单声明 ops/manifest.rs 仅 L32/L105 两处）；repos/paperwork-core/src/ops/manifest.rs 实测 L80/L151/L194 另有三处旧文法 example：`paperwork brief create {} --title <title>`（新文法 --title 已转位置参数，属必改点位）。
- Expected：「已知位置」清单应覆盖全部旧文法 example（manifest.rs 应为 L32/L80/L105/L151/L194 五处），其余文件经本次复核实测无误（thread.rs 6 处、contacts.rs L22、profile.rs L61/L91；thread.rs L288、contacts.rs L56/L98、profile.rs L20 与新文法同形无需改）。
- Actual：清单已知 11 处，实际旧文法点位 14 处，漏 3 处。虽有「完整清单以实施时全仓检索为准」兜底，但 design §7.4 裁定 6 已经暴露过一次同类盘点矛盾（「13 处」数字矛盾），更新后的清单仍未盘净，说明该兜底机制掩盖了清单不可靠的事实；且 tdd §4「ops_tests 零改动」防线依赖 core 改动范围被精确认知。
- 建议：impl_plan 步骤④补齐 manifest.rs L80/L151/L194，并将清单改为「先执行全仓检索、再附检索命令与完整输出」的形式，删除「以实施时检索为准」的延迟兜底表述。

#### ISSUE-M4 spec 缺 showing 字段的 total 口径定义（Severity: Major）

- 证据：spec.md L108（§3.1）与 L289（§4.6）仅写「恒显 showing: n/total」，未定义 total；design.md L91 同；bdd.md L112-116（S-READ-02：50 条线程 limit 20 → showing 20/50）与 L134-138（S-READ-06：过滤后零命中 → showing 0/0）；现状代码 repos/paperwork-cli/src/cmd/post.rs L194-209（total = 过滤后、limit 截断前的条数）。
- Expected：spec 自我定位为「实现与测试的唯一验收基准」（spec.md L5），新增输出字段的口径应能从 spec 本身推导：total = 满足过滤条件（--mention/--reply-to）的消息数，--limit 截断前；无过滤时为线程全部条数。
- Actual：spec/design 均未定义 total。bdd 两个场景隐式预设了上述口径（与现状代码一致），但若实现者将 total 理解为线程物理总数，S-READ-06 将输出 0/6 而验收失败；「字段区形态、不放 conclusion 行」已有裁定（design §7.4 裁定 3），total 口径却漏裁。
- 建议：spec §3.1 post read 与 §4.6 补一句 total 定义（过滤后、limit 前）；bdd 无需改动。

### Minor

#### ISSUE-m1 「可扩展封闭集合」自相矛盾措辞与自指性预声明

- 证据：spec.md L250/L257/L311、design.md L292（四处均称「category 词表为可扩展封闭集合，本次扩展已走评审流程」）；README.md L40（治理清单：对抗评审状态为待办）。
- Expected：措辞不应自相矛盾；「已走评审流程」应在评审闭合后陈述。
- Actual：「封闭」与「可扩展」同句并用；文档落盘时对抗评审尚未开始（README 清单未勾选），却已声称扩展「已走评审流程」，属自指性预声明。实质条款（六类冻结不变 + additive 第七类 usage + exit 2 分层）本身自洽，不构成对「六种 category」既有承诺的违背——spec 的表述是「六类冻结、新增第七类」，逻辑上成立；问题仅在措辞与时态。
- 建议：改为「category 词表为冻结枚举，仅可经评审流程扩展」；「已走评审流程」改为「待本次对抗评审确认」。

#### ISSUE-m2 role 文档字段名漂移（implicit-mentions 复数）

- 证据：docs/roles/cli-ux-redesign-implementer.role.md L16（职责 1 写「implicit-mentions」）；spec.md L83/L288、design.md L90/L293（§7.4 裁定 2）、bdd.md L35、tdd.md L129 均为单数 implicit-mention。
- Expected：字段名以裁定后的单数 implicit-mention 为准（design §7.4 裁定 2 明文「单数字段」）。
- Actual：role.md 用复数。role 是实施者入职首读文档，复数形态有被带入实现的风险（role §二.1 同时声明以 spec 为唯一基准，风险可控，故判 Minor）。
- 建议：role.md L16 改单数。

#### ISSUE-m3 role 文档可修改文件清单遗漏 SKILL.md

- 证据：role.md L17（职责 2 列举「仅修改 impl_plan.md 指定文件」清单，含 CHANGELOG/README/Cargo.toml 等，无 SKILL.md）；impl_plan.md L77（步骤⑦：随仓库新增 SKILL.md（英文））。
- Expected：role 文件清单与 impl_plan 文件集合一致。
- Actual：清单缺 SKILL.md；严格执行 role 清单的实施者会漏掉步骤⑦交付物，或在补交时自认越界（role §二 禁止扩大范围）。
- 建议：role.md L17 清单补 SKILL.md。

#### ISSUE-m4 U-03 延后裁决未正面回应 backlog 的「本次必须解决」

- 证据：docs/researches/ux-open-items-backlog-2026-08-08.md L16（U-03「本次必须解决：统一线程创建语义（v0.2/v0.3/v0.4 连续三版遗留）」）、L85（§六 高严重度）、L92（§七 Top 2）；design.md L281（§7.3 改为延后，一句理由）。
- Expected：翻转研究文档的「必须」级建议时应显式说明推翻理由（评审要点 4 要求每项有明确处置——处置存在，但回应不完整）。
- Actual：design §7.3 理由（文件格式层变更、波及存量文件、本版纯 CLI 文法）实质成立，但未正面交代与 backlog「本次必须解决」口径冲突的取舍；对比 U-04 同为翻转却给出原则级理由（违背显式原则），U-03 处理较弱。注意：backlog 属研究文档而非 owner 指令，v0.5_feedbacks 未要求解决 U-03，故不构成 SSOT 违背，判 Minor。
- 建议：design §7.3 U-03 行补一句对 backlog 口径的显式调和（如「backlog 建议本次解决；本版范围限定 CLI 参数层，格式层变更单独立项，见未来工作」）。

#### ISSUE-m5 SKILL.md 依据引用编号错误（SOTA 结论 6 与 10）

- 证据：design.md L317 与 impl_plan.md L77 均引「agent-cli-ux-industry-sota-2026-08-08.md 结论 6 与 10」作为 SKILL.md 依据；实测该文档 L195 C5 才是「随仓库发布 SKILL.md」（Infracost 难题正确率 0/6→6/6 的关键），L196 C6 为成文命名政策，L200 C10 为错误三层 example——两者均与 SKILL.md 补偿效应无关；「首次误调率补偿」表述实际出自该文档 §8 风险 1（对冲三件套）与 §1.5c/§1.6。
- Expected：引用指向真实支撑条款（C5 + §8 风险 1）。
- Actual：引用编号错误；实质支撑在同一文档内存在，决策本身不受影响，故判 Minor，但追溯链失真会误导后续实施与评审。
- 建议：两处引用改为「结论 5（C5）与 §8 风险 1（对冲三件套）」。

#### ISSUE-m6 ADR-011 示例层在 v0.5 实现后将与现实脱节

- 证据：docs/dev/adr-v1.md L52-73（CLI Command Model 代码块仍为旧文法：profile create --name、post send --from、post create --title、brief add --entry-path、contacts add --profile）；impl_plan.md 全文文件清单（不含 docs/dev/adr-v1.md）。
- Expected：文法重排后，依据 ADR 的旧文法示例块应有历史注记或同步安排（实现流程原则：SSOT 变更时须完整检查并更新相关文档——此处 ADR 未变更，但其示例层与实现即将脱节）。
- Actual：文档集未做任何安排；实现后 adr-v1.md 示例将与实际 CLI 不一致。ADR-011 的原则层（stateless/path-explicit/无登录/无状态目录/任意 CWD）经逐条核验未被新文法违背，仅示例层脱节，故判 Minor。
- 建议：adr-v1.md CLI Command Model 节首加一行注记「示例为 v0.4 及更早文法；v0.5.0 文法以 docs/ssot/specs/cli-ux-redesign/spec.md 为准」，或纳入 impl_plan 步骤⑦。

#### ISSUE-m7 implicit-mention 不触发条件欠明确

- 证据：spec.md L83（仅写「至多触发一人（原消息发送者）；仅当触发时」）；现状代码 repos/paperwork-cli/src/cmd/post.rs L166-176（三种不触发情形：原发送者即发送者本人、原发送者已在显式 --mention 中、reply-to seq 不存在时静默跳过）；bdd.md L31-35（S-SEND-03 仅覆盖触发场景与「未触发 reply-to 时不出现」）。
- Expected：spec 列明不触发边界，bdd 至少一条边界场景。
- Actual：自回复、已显式 mention、reply-to 指向不存在 seq 三种边界在 spec/bdd 均无依据；「行为不变」仅隐含于「additive」表述。
- 建议：spec §3.1 补一句不触发条件（维持现状行为）；bdd 可选补 S-SEND-11 边界场景。

#### ISSUE-m8 关键解读条款缺确认途径记录（证据链风险）

- 证据：docs/ssot/adr/feedbacks/v0.5_feedbacks.md L27-31（§二「指令解读（落盘时点共识）」：字面示例的意图为硬性要求、字面文法细节属授权涌现）；design.md L20-35（§1.1 对 owner 字面示例 path-first 文法的四点否决论证）；design.md L35（保留为 v0.6 提案）。
- Expected：design §1.1 否决 owner 字面示例（动词居第三位的 path-first 文法）是全设计的关键转折，其唯一书面依据是 feedbacks §一 ③（授权涌现）与 §二 解读条款；该解读应记录确认途径（落盘对话、owner 后续消息编号等），以备审计。
- Actual：§二 仅标注「落盘时点共识」，无确认途径记录。本评审未发现任何下游文档违背 feedbacks 文本之处——解读与授权条款内部自洽；但若「共识」未经 owner 实际确认，否决字面文法的正当性根基薄弱。此为证据链风险而非已证实的违背，判 Minor。
- 建议：feedbacks §二 补记确认途径；或请 owner 在本轮评审中对「意图采纳、字文法否决」做一次显式追认并追加落盘。

## 四、其他观察（不计入 Issue 数）

1. tdd.md §1 标题「约 24 处」与 §1.7 合计「26 处改写」数字表述漂移，建议统一为 26。
2. design §7.4 裁定 6 以「实施时检索为准」消解 example 处数矛盾，ISSUE-M3 证明该方式掩盖清单不完备；建议清单先刷新再附检索命令。
3. usage 信封要求对任意旧文法调用给出「逐字修正命令」（bdd S-SEND-09），实现上需从 clap 错误重构用户原始命令，工程难度较高；非 SSOT 问题，提示实施风险。

## 五、是否闭合与必须修复清单

**结论：未闭合。**

必须修复（闭合前完成，均为文档层修改，成本低）：

1. ISSUE-M1：修正 bdd S-SEND-08 场景命令（改为仅 PATH 形态）与 tdd §3 对应用例；补「PATH + 单字符串 → validation」场景与 NAME/BODY 不可区分裁定。
2. ISSUE-M2：改写 spec §5 与 design §7.4 裁定 7 的第③级表述为路径决策语义；impl_plan 步骤①补只读命令不建文件的排除条款。
3. ISSUE-M3：impl_plan 步骤④补齐 ops/manifest.rs L80/L151/L194 三处遗漏，清单改为先检索后附完整输出。
4. ISSUE-M4：spec §3.1/§4.6 补 showing 字段 total 口径定义（过滤后、limit 截断前）。

建议一并修复（不阻塞但应在 rework 轮完成）：ISSUE-m1~m8。

对抗立场下的总体评价：文档集在治理结构、ADR-011 五约束合规、pillars 继承、owner 指令落盘忠实性、遗留项裁决覆盖、tdd 行号可信度上表现扎实；4 项 Major 均属文档层契约歧义或事实性遗漏，无一构成对 owner 指令或 pillars 的实质违背，修复后可闭合。

---
（评审报告完 — 视角一：SSOT/pillars 合规，评审者：第三方批判性评审 subagent）
