# contacts CRUD 轮治理文档集评审报告（维度：SSOT 与治理合规）

- 日期：2026-08-09
- 评审维度：SSOT 与治理合规（owner 指令一致性 / SSOT 引用与条款关系 / 引用真实性 / bdd 结构完整性 / 治理流程合规）
- 被评审文档：
  1. `docs/ssot/adr/feedbacks/v0.7_feedbacks.md`（108 行，全文通读）
  2. `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md`（153 行，全文通读）
  3. `docs/ssot/specs/cli-grammar-v0.6/spec.md`（275 行）、`bdd.md`（464 行）、`tdd.md`（266 行）、`impl_plan.md`（154 行）本轮增量修订部分
- 评审方法：全部被评审文档逐行通读；外部引用抽查 **30+ 处**（远超 ≥10 要求），全部以 Read/Grep 实测核对路径与行号，无凭文档自述采信项。
- 编号约定：C-n（Critical）/ M-n（Major）/ m-n（minor）；位置行号均为本次评审实测值。

---

## 一、owner 指令原文一致性

### 1.1 逐字收录核查

- v0.7_feedbacks §一 (1)(2)(3) 共 4 段引文（三条指令，(3) 含两段回复）。**核查限制声明**：owner 原始消息发生于编排层对话中，未以独立 SSOT 载体先于本文落盘，评审方无法逐字比对原始消息文本，仅能做文档间交叉一致性核验。
- 交叉核验：§一 (3) 两段回复与 spec.md §3.9 L189 所引「遇到冲突了就锁定文件阻塞, 另一侧fast fail, 这是paperwork 中并行场景以外的基本做法」逐字一致（与 §一 (1) 同句）；指令 (2) 的盘点要求在 spec §9、研究文档 §2 的落地映射完整（profile/contact/brief read 面 + brief/post summary 先行）。未发现引文内部自相矛盾或转述漂移。
- 体例合规：与 v0.6_feedbacks §一 体例一致（原文引用块 + 即解）；「architectural basis / 最高优先级输入」定位声明在 spec.md L9、bdd.md L8、tdd.md L8、impl_plan.md L8 四处一致登记。

### 1.2 解读越权核查（owner 已授权 spec 治理自决，边界 = 不超出指令语义）

- §2.1 contacts 完整 CRUD（键语义、不支持改 label、R11 重派生）：属指令 (1)「contact当然需要支持完整CRUD」语义内的形态自决，且各点均锚定既有 SSOT（format-v2 spec §7.2/§7.3），无越权。
- §2.2 锁 + fast fail 延伸至 brief add/remove、profile edit 补锁：依据指令 (3) owner 自设准则「是否使用同样的路径锁, 取决于我给出的功能要求是否也适用于同样的机制」+「非并发编辑情况下使用锁定+fast fail方式处理」的**通用表述**（「这是paperwork 中并行场景以外的基本做法」），延伸面与准则语义同构，未超出指令语义。**已核查、无发现**（唯计数错误见 M-2）。
- §2.4 brief 选择性详情（`--entry-title`）：指令 (2) 要求「brief和post都应该支持先看summary再选择性或者全部查看详情的」，「选择性查看详情」在 brief 现状（TOC/`--full` 两档）下不可达，补齐第三档属指令语义内的缺口闭合，非无中生有。post 面以 summary + read 窗口过滤登记结案并有 U-09 先例裁决支撑（backlog L22 实测命中），成立。
- 信封字段命名、错误 category 选择、usage exit 2 层级等细节：属 §2.3 授权自决范围并均引用既有体例，无越权。
- **发现 M-1**（见下）：命名政策核验引用 SOTA C6 失实，属解读中的事实性错误。

## 二、SSOT 一致性（与前序文档集的引用与条款关系）

### 2.1 已核查、无发现的项

| 核查点 | 实测结论 |
|---|---|
| v0.7 §四「不发布约束延续 v0.6_feedbacks §一 (3)」 | v0.6_feedbacks.md L25-29 §一 (3)「我没让你发布0.6…」+ 四不约束，实测命中，成立 |
| v0.7 §2.3(2)「v0.5_feedbacks §一.3 授权涌现先例」 | v0.5_feedbacks.md L19-23 ③「授权文法自行涌现」，实测命中，成立 |
| append-only 适用域论证引证链 | `docs/dev/adr-v1.md` L36-37（DM thread / Post/GDM append-only 两行表格）✓、format-v2 spec L232（§5.8 thread 域保留约束）✓、L399（I1 thread append-only）✓、v0.3-review L78（brief remove 反证）✓，四处实测全部逐字命中；「适用域仅限通信线程」的解释与四源一致，且 v0.7 §四 明示为「适用域解释而非文本翻转」，治理定性正确 |
| 与 v0.6 spec §7 冻结条款的 additive 自洽性 | spec.md L250 第 5 条 additive 登记与 v0.7 §四 第三条双向引用一致；core 仅新增函数、格式零触碰（format-v2 spec L12 实测命中「profile/brief/contacts 三格式语义不变」）、输出协议只增、不发布延续，四要件与冻结条款逐项对得上，自洽成立 |
| v0.7 §五 依据表 9 处文件引用 | 逐处实测存在且内容匹配（backlog L22 / v0.3-review L78、L152 / SOTA L63、L194 等），唯 L107 C6 行失实（M-1） |
| 优先级声明 | 仅声称高于「实施方此前规划立场与 append-only 延伸论证」（无成文条款，§3.1 表已如实标注「无成文条款」），不声称高于任何 owner 成文条款；对 v0.6_feedbacks 声明「叠加生效」；边界声明（翻转须 owner 新指令）与前序体例一致，成立 |
| 新信封字段 `contacts`/`removed`/`updated` 与 command id | 与既有字段（seq/path/sender/removed 无前例冲突）无重名；command id 沿用 `<组>.<动词>` 体例；JSON additive 口径与 v0.5 spec §4 冻结条款（只增不改不删）相容 |

### 2.2 发现

- **M-1｜SOTA C6 动词白名单引用失实**（v0.7_feedbacks.md L107；research L123）
  - 问题：v0.7 §五 L107 称「动词白名单命名政策（L196 C6，update 在既定白名单内）」；研究文档 L123 称「update/remove/read 均在 SOTA C6 动词白名单内（…L196）」。实测 `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` L196 白名单逐字为「create/send/read/list/edit/add/remove/verify/validate/summary」——**不含 `update`**。`remove`/`read` 确在白名单内，唯 `update` 不在。文档把「白名单扩容」表述成「白名单既有」，属对被引 SSOT 的事实性误述；而该白名单同时是 tdd §8.3 机械测试防线的依据，证据链在此断裂。
  - 建议修复：`update` 的纳入由 owner 指令 (1)（contacts 完整 CRUD）天然授权，治理上可成立，但须如实表述——在 spec.md（§7 第 5 条或 §2 本轮增量说明处）补记「SOTA C6 白名单随本轮 additive 扩容 `update`，依据 v0.7_feedbacks §一 (1)」，并把 v0.7 L107 / 研究文档 L123 的「在既定白名单内」改为「经本轮扩容后纳入」；tdd §8.3 白名单测试项同步明示扩容来源。
- **M-2｜锁缺口计数「五处」与实际六处不符**（v0.7_feedbacks.md L47）
  - 问题：§2.2(3) 末句「上述五处写路径均为无锁 read-modify-write（调研文档 §4 缺口表）」，但其上 L44-46 自列 **6 条**写路径（contacts add/remove/update、brief add/remove、profile edit）；研究文档 §4 缺口表（L78-82）的「无锁现状」行实为 4 条既有路径（contacts_add/brief_add/brief_remove/edit_profile），「本轮处置」合计 6 条。spec §3.9 L190、§6 L240、bdd S-LOCK-03 L461、tdd §8.6(4) 均正确写「六处/六写路径」，唯 v0.7 指令记录档为「五处」，指令记录档与下游全部文档计数不一致。
  - 建议修复：v0.7 L47「五处」改「六处」（若本意指「补锁动作五处 = 3 条 contacts + brief 2 条 + profile 1 条」亦应写明口径；按现行列举即为 6）。
- **m-2｜研究文档对被修订文档的行号引用整体失效**（research L12-20、L36、L46、L148-149）
  - 问题：研究文档 §1/§2.1/§2.3/§7 大量引用 spec.md（如「§2 全表（L45-60）」）与 bdd.md（「S-SHORT-02 白名单冻结断言（L391）」）的行号，均为本轮 spec/bdd 增量修订**之前**的行号：修订后 spec §2 全表实为 L48-66、bdd S-SHORT-02 实为 L440-443（实测）。按落盘时序（研究先行、修订在后）这不属虚假行号，但当前读者按文索骥全部落空，且研究文档文首「行号纪律」声明未标注所引 spec/bdd 为修订前基线。
  - 建议修复：研究文档「核实基线」行（L5-6）补注「本文对 spec.md/bdd.md 的行号引用以本轮增量修订前基线为准，修订后位置见被引章节号」；或以章节号替代行号。
- **m-3｜spec §2 表头语义漂移**（spec.md L48）
  - 问题：第三列表头为「相对 v0.5 变更」，本轮起该列承载 v0.7 轮「**本轮新增**」标注（L59/L63/L64/L62），表头语义未随之扩展。
  - 建议修复：表头改「相对 v0.5 变更 / 本轮增量标注」，或在 L46 约定行补一句。

## 三、文档内引用真实性（≥10 处实测抽查，重点防线）

实测核对 30+ 处，清单与结论如下（全部 Read/Grep 实测，✓ = 逐字/逐行命中）：

| # | 引用（出处） | 实测结论 |
|---|---|---|
| 1 | `ops/contacts.rs` L1 模块注释「create, add, read」（v0.7 L33、研究 L24） | ✓ |
| 2 | `ops/contacts.rs` L17/L53/L95 三公开函数、L74 幂等判据 `c.profile_path == profile_path`、L121-149 `derive_label`（研究 L24、L99-100、v0.7 L35） | ✓ 全部命中 |
| 3 | `ops/manifest.rs` L21/L69/L145/L188、L84 读 L134 写、L155 读 L177 写（研究 L25、L80-81） | ✓ |
| 4 | `ops/profile.rs` L78 `edit_profile`（研究 L26、L82） | ✓ |
| 5 | `ops/thread.rs` L61/L94/L97、L345/L366，锁区 L355-364 开句柄、L366-371 取锁、L373-388 锁内读、L491-511 truncate+rewrite、L513-518 解锁、L393 `unlock().ok()`（研究 L72、L107，v0.7 L42/L47） | ✓ 逐段命中 |
| 6 | 全 core `lock_exclusive` 仅两处（研究 L72、v0.7 L47） | ✓（Grep 实测 2 处，均在 thread.rs） |
| 7 | worktree `post.rs` L459-469 QA BUG-2 注释、L485-487 跨句柄读注释（研究 L86，v0.7 L48） | ✓（`agent-paperwork-wt-v06grammar` 实测） |
| 8 | format-v2 spec L12「三格式语义不变」（v0.7 L90、spec L250） | ✓ |
| 9 | format-v2 spec L37（§2 第 5 条 contacts 链接 bullet）、L228（§5.7 崩溃窗口）、L232（§5.8）、L330（§7.2）、L344（§7.3 R11）、L399（I1）（研究 L92/L98/L100，v0.7 L41/L49/L103） | ✓ 六处全部逐字命中 |
| 10 | `docs/dev/adr-v1.md` L36-37 append-only（v0.7 L41/L88） | ✓ |
| 11 | v0.3-review L72-73 TOC/--full 实测记录、L78 brief remove、L152「Nice progressive disclosure」（研究 L41，v0.7 L61/L106） | ✓ |
| 12 | backlog L22 U-09「保留独立命令结案」（v0.7 L60、spec L269） | ✓ |
| 13 | pillars session-log user-only L27（渐进阅读原话）、L45（summary 原话）；完整版 L157、L300（研究 L50-51，spec L274） | ✓ 四处逐字命中 |
| 14 | SOTA L22（Trevin 原则 7）、L62（谓词下推）、L63（summary 先于细读）、L194（C4）（研究 L52/L132，v0.7 L64/L107） | ✓ |
| 15 | SOTA L196（C6 动词白名单）（v0.7 L107、研究 L123） | 位置 ✓，**内容误述**（update 不在白名单，见 M-1） |
| 16 | v0.6_feedbacks §一 (3) 不发布（v0.7 L82/L89、spec L249、impl_plan L12） | ✓ |
| 17 | v0.5_feedbacks §一 ③ 授权涌现（v0.7 L54） | ✓ |
| 18 | 双树一致性抽查：worktree `ops/contacts.rs` L53/L74 与主工作区同值（研究 L5） | ✓ |

**抽查结论**：未发现虚假行号事故迹象；行号纪律总体兑现。唯一内容性失实为 M-1（引用位置真实、但断言内容与被引原文矛盾）。

## 四、bdd.md 章节结构完整性（独立复核）

- **§ 编号连续性**：§1 post send → §2 post edit → §3 post read/summary → §4 profile → §5 brief → §6 contacts → §7 validate → §8 路径解析 → §9 输出模式 → §10 别名 → §11 短形式 → §12 写路径锁。实测骨架连续无跳号，§12 位于文末、收 S-LOCK-01~03，此前「误插 §12 破坏结构」的修复**确认到位**。
- **S-* 场景编号**：S-SEND-01~15/17~21（16 号缺口由 L112 基线勘误注记解释，属有意删除非事故）；S-EDIT-01~09、S-READ-01~09、S-SUM-01、S-PROF-01~05、S-BRIEF-01~09、S-CONTACTS-01~11、S-VAL-01~06、S-OUT-01~06、S-ALIAS-01、S-SHORT-01/02、S-LOCK-01~03。本轮新增 13 个场景（S-BRIEF-07~09、S-CONTACTS-06~11、S-LOCK-01~03）编号均为各区段连续顺延，**无撞号**；v0.5/v0.6 同名异义撞号面有 L13 编号约定防线，引用均带前缀或明示本文编号，抽查 tdd §8.2 对 bdd 场景的映射 13/13 全对。
- **白名单断言更新位置**：S-SHORT-02（§11，L440-443）为唯一白名单断言位，本轮就地更新（contacts 动词集合 {create,add,remove,update,read}、追加 `--new-profile`），位置正确；但**计数错**，见 M-3。
- **M-3｜S-SHORT-02 白名单计数「共 25 项」与枚举清单不符**（bdd.md L443；连带 tdd.md L246）
  - 问题：逐项点数 L443 枚举清单：--seq/--stdin/--title/--to/--from（5）+ --entry/--entry-title/--profile/--new-profile（4）+ --name/--model/--description/--owner/--note/--regex（6）+ --scope-* 三 flag（3）+ --full/--limit/--base-dir/--type/--json/--plain（6）= 24，再加「另含」--reply-to 与 --mention（2），**合计 26 项**。文中括号注「其余与 spec §4 枚举逐字对齐…（本轮 additive 新增 --new-profile 一项）」说明修订前清单为 25 项（含 --participants 26 − 删 1 = 25），新增 `--new-profile` 后应为 **26**，但「共 25 项」未随之更新。tdd §8.3 第 1 条「共 25 项」同样沿误。
  - 建议修复：bdd L443 与 tdd L246 两处「共 25 项」改「共 26 项」；实施时白名单测试断言以逐项枚举为准（spec §4 L208 全表与 bdd 枚举一致，实测无遗漏）。
- **已核查、无发现**：§12 三场景与 spec §3.9 四款（原则/适用范围/Windows 判例/崩溃窗口）一一对应；S-LOCK-03 明示「集成测试不强制模拟 OS 级锁失败、以 code review + 点位盘点断言」与 tdd §8.6(4) `rg lock_exclusive` 门禁咬合。

## 五、治理流程合规

- **先文档后实现**：impl_plan 文首前置门槛（L11）与本轮增量前置门槛（L106，点名 spec §2/§3.5/§3.6/§3.9/§7(5)/§9 + bdd S-BRIEF-07~09、S-CONTACTS-06~11、§12、S-SHORT-02 + tdd §8）均要求对抗评审闭合后方可动码，引用对象与实际修订面逐一吻合；研究文档先于设计落盘（v0.7 §五、研究文首互引），符合「调研先行、持久化后方可实现」的项目纪律。**已核查、无发现**。
- **不发布约束**：impl_plan L12 交付边界、L100、L108、L143（不写 CHANGELOG 发布段）、tdd §8.6(5)、spec §7 第 4/5 条、v0.7 §3.2/§四 全程一致声明「不 bump、不 tag、不 publish、不写 CHANGELOG 发布段」；R1~R6 步骤无任何发布动作。**已核查、无发现**。
- **工作区边界**：impl_plan L22/L107 双处声明禁止触碰主工作区 repos/、R1~R6 在 worktree 分支执行，口径一致。**已核查、无发现**。
- **QA 独立性**：本轮 QA Review Book 沿用步骤(7) 口径「不得由 impl agent 自评」（impl_plan L153），合规。
- 备注：owner 指令 (1)(2)(3) 原文仅存在于 v0.7_feedbacks 单一载体（编排层对话无独立落盘），符合前序各轮 feedbacks 体例，不构成违规，但意味着 M-1/M-2 类错误只能靠文档间交叉核验发现，建议编排层知悉该单点属性。

## 六、发现汇总

| 编号 | 级别 | 位置（实测） | 摘要 |
|---|---|---|---|
| M-1 | Major | v0.7_feedbacks.md L107；研究文档 L123 | 「update 在 SOTA C6 既定白名单内」失实：L196 白名单逐字不含 update；须改为「本轮经 owner 指令授权扩容」并在 spec/tdd 登记扩容来源 |
| M-2 | Major | v0.7_feedbacks.md L47 | 锁缺口「五处」与自列 6 条写路径及下游 spec/bdd/tdd 一致的「六写路径」计数矛盾 |
| M-3 | Major | bdd.md L443；tdd.md L246 | S-SHORT-02 白名单「共 25 项」实际枚举为 26 项，新增 --new-profile 后计数未更新 |
| m-2 | minor | 研究文档 L5-6/L12/L36/L46/L148-149 | 对 spec/bdd 的行号引用为修订前基线，修订后整体失效，未标注基线口径 |
| m-3 | minor | spec.md L48 | §2 第三列表头「相对 v0.5 变更」未覆盖本轮增量标注语义 |

统计：**C = 0，M = 3，m = 2**。

（编号说明：m-1 号未启用，避免与「已核查、无发现」维度混淆；各级别内按发现顺序编号。）

## 七、总判定

**有条件放行。**

必须修复项（放行前置条件）：

1. **M-1**：修正 v0.7 L107 与研究文档 L123 对 SOTA C6 白名单的失实表述，并在 spec.md（additive 登记处）与 tdd §8.3 补记 `update` 白名单扩容的授权来源（owner 指令 (1)）。此为证据链断裂点，且直接关联机械测试防线（flag_inventory/动词白名单测试）的口径，不得带错进入实施。
2. **M-2**：v0.7 L47「五处」改「六处」（或写明计数口径）。指令记录档是本轮最高优先级输入，计数错误对实施方有直接误导面（漏锁 profile edit 的风险形态）。
3. **M-3**：bdd L443 与 tdd L246「共 25 项」改「共 26 项」，两处同改。

建议修复项（不阻塞放行）：m-2（研究文档补基线口径注记）、m-3（spec §2 表头扩义）。

除上述 5 项外，本轮文档集在 SSOT 一致性、引用真实性（30+ 处实测无虚假行号）、bdd 结构骨架、治理流程四个面上均达标：append-only 适用域论证四源引证链完整、additive 扩展与 §7 冻结条款自洽、不发布约束全程贯彻、先文档后实现门槛齐备。

---
（评审报告完）

---

## Rework 回应销账段（2026-08-09，实施方 Robin 补录；修复位置行号均为销账时点 Grep/Read 实测）

| 编号 | 修复位置（实测） | 状态 |
|---|---|---|
| M-1（SOTA C6 白名单失实） | `docs/ssot/adr/feedbacks/v0.7_feedbacks.md` §2.5 L66-71（扩容裁定记录）+ §五 L113（依据表行更正）；`docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L123（改为「经 owner CRUD 指令授权的白名单扩容」）；`docs/ssot/specs/cli-grammar-v0.6/spec.md` L5（header）/L70（§2 扩容登记）/L254（§7 第 5 条扩容登记）；`tdd.md` L257（扩容来源登记）；update/edit 语义分工落 spec.md L174 与 impl_plan.md R6 L143 | 已销账 |
| M-2（锁路径计数五处） | `docs/ssot/adr/feedbacks/v0.7_feedbacks.md` L47（改「六处」并写明计数口径与下游六写路径口径对齐） | 已销账 |
| M-3（白名单计数 25） | `docs/ssot/specs/cli-grammar-v0.6/bdd.md` S-SHORT-02 L464（「共 26 项」，修订前 25 + --new-profile）；`tdd.md` §8.3 L256（同步 26 项 + 新建/扩展措辞） | 已销账 |
| m-2（研究文档行号基线失效） | `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L6（时效声明：对 spec/bdd 引用为修订前基线，以章节号/场景号为准）+ L156（S-SHORT-02 改场景号引用） | 已销账 |
| m-3（spec §2 表头语义漂移） | `docs/ssot/specs/cli-grammar-v0.6/spec.md` L48（表头改「相对 v0.5 变更 / 本轮增量标注」） | 已销账 |
| 「已核查、无发现」各维度（§1.1/§1.2/§2.1/§三 30+ 处抽查/§四 骨架/§五 治理） | 无需修复，rework 未触碰其认定面 | 维持 |

销账统计：本报告 5 条发现（3M+2m）全部销账，无挂起项。
