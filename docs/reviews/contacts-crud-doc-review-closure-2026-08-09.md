# contacts CRUD 轮治理文档 rework 闭合复核报告（独立复核，未参与本轮起草/评审/rework）

- 日期：2026-08-09
- 复核对象：三份评审报告末尾「Rework 回应销账段」声称的 24 条发现销账（Mark/SSOT 5 条 + Ryan/agent-ux 9 条 + Daniel/feasibility 10 条）
- 复核方法：全部 24 条逐条 Read 销账声称的修复位置并核验内容与修复意图真实吻合（不抽样、不只看行号）；交叉一致性抽查 10 处（≥8 要求）；回归检查（章节骨架/场景编号/冻结条款自洽）。
- 读取清单（全部全文 Read，除注明区间者）：`docs/ssot/adr/feedbacks/v0.7_feedbacks.md`（114 行）、`docs/researches/contacts-crud-progressive-read-research-2026-08-09.md`（160 行）、`docs/ssot/specs/cli-grammar-v0.6/spec.md`（279 行）、`bdd.md`（487 行）、`tdd.md`（277 行）、`impl_plan.md`（154 行）、`docs/researches/ux-open-items-backlog-2026-08-08.md`（117 行）、worktree `agent-paperwork-wt-v06grammar` `cmd/brief.rs` L150-210（只读）
- 立场：批判优先；未发现任何修改行为（本复核仅创建本报告文件）

---

## 一、24 条逐条复核结论表

### 1.1 Mark（SSOT 与治理合规）5 条

| 编号 | 销账声称位置 | 复核实测 | 结论 |
|---|---|---|---|
| M-1 SOTA C6 白名单失实 | v0.7 §2.5 L66-71 + §五 L113；research L123；spec L5/L70/L254；tdd L257；spec L174 + impl_plan L143 | v0.7 L66-71 新增 §2.5 三点齐备（如实更正「不含 update」/扩容裁定 10→11 动词/update-edit 语义分工）；L113 依据表行已改「既定白名单 10 动词不含 update；本轮经 owner 指令 (1) 授权扩容纳入」；research L123 改「update 不在既定白名单内…经 owner CRUD 指令授权的白名单扩容」并自注初版失实；spec L5（header）/L70（§2 扩容登记）/L254（§7 第 5 条）三处口径一致；tdd L257 登记扩容来源；update/edit 分工落 spec L174 与 impl_plan R6 L143（SKILL.md/README 区分注记） | 吻合 |
| M-2 锁缺口计数「五处」 | v0.7 L47 改「六处」 | L47 逐字为「上述**六处**写路径…（rework 修订：原『五处』为计数笔误，与本节自列清单及下游 spec §3.9/§6、bdd §12、tdd §8.6(4) 的『六写路径』口径统一为六处）」 | 吻合 |
| M-3 白名单「共 25 项」 | bdd L464 + tdd L256 改 26 | bdd L464「**共 26 项**…修订前 25 项 + 1 = 26 项…原『共 25 项』为 stale 计数，Mark M-3/Ryan m-1/Daniel m-1 定案为 26」；tdd L256「26 项全量清单（修订前基线 25 项 + --new-profile = 26 项）」；本复核独立点数 bdd L464 枚举 = 5+4+6+3+6+2 = 26，计数与枚举自洽 | 吻合 |
| m-2 研究文档行号基线失效 | research L6 时效声明 + L156 场景号引用 | L6 补「**时效声明（rework 补录）**：本文对 spec.md/bdd.md 的行号引用为本轮增量修订**前**基线，修订后行号可能漂移，追溯时以被引章节号/场景号为准」；L156 改「落盘时点实测 L391；本轮增量修订后位于 §11，以场景号 S-SHORT-02 为准，免行号漂移」 | 吻合 |
| m-3 spec §2 表头语义漂移 | spec L48 表头扩义 | L48 表头逐字为「相对 v0.5 变更 / 本轮增量标注」 | 吻合 |

### 1.2 Ryan（agent 消费者视角）9 条

| 编号 | 销账声称位置 | 复核实测 | 结论 |
|---|---|---|---|
| M-1 白名单失实 + 无裁定 + 无分工教学 | v0.7 L113 + research L123 + 新 §2.5 L66-71；spec L174/L70/L254；impl_plan L143 | ① 失实更正两处到位（同上 Mark M-1）；② 裁定记录选建议方向 (a)（正式扩容 + 语义分工落盘），v0.7 §2.5 L66-71；③ 分工教学 spec L174 新条「update 与 edit 的语义分工」+ L70/L254 登记；④ impl_plan R6 L143「contacts update 示例附一句与 edit 的区分注记」 | 吻合 |
| M-2 锁阻塞 agent 可见行为未定义 | spec §3.9 L196 例外声明 + L197 五要素新条；bdd L486 备注 | spec L196 补「**例外声明**：阻塞等待本身是本轮新增的可观察行为…不属『同构』范围」（正面回应原评审「L193 遗漏可观察行为」）；L197 新条五要素逐一齐备：①无内建超时（fs2/Windows LockFileEx）②毫秒级临界区③OS 自动释放不死锁④编排层进程级超时+杀进程重试指引（幂等/先读后写重试安全）⑤timeout 否决记录引用研究 §6.1 且不豁免声明；bdd §12 L486 备注承接「锁阻塞等待无内建超时…契约全文见 spec §3.9」 | 吻合 |
| M-3 update 静默成功面 + add 现状失实 | spec L169 更正 + L172 行为登记；bdd S-CONTACTS-14 L392-398；backlog B-02 L113；tdd L222/L243 | ① spec L169 改「不做任何可读性校验…属纯静默回退而非任何 validation/not-found（初版曾失实表述…经 Ryan M-3 指出后更正）」；② spec L172 新条「NEW 不存在/不可读时的行为契约」按 S-SEND-17 三件套体例（声明 + bdd 钉住 + backlog 登记）；③ bdd L392-398 新场景 S-CONTACTS-14：声明面/Given（忘后缀形态）/Then（exit 0 + `[carol](carol)` + updated 回显原值 + 下轮 read 见 unreadable）/agent 自救指引；④ backlog L113 B-02 登记「写前 destination 存在性校验/回显」候选增强；⑤ tdd L222（core 静默成功行）/L243（CLI 级 S-CONTACTS-14 行）双侧钉住 | 吻合 |
| m-1 白名单项数 25→26 | bdd L464 + tdd L256 | 两处同改「共 26 项」（同 Mark M-3） | 吻合 |
| m-2 usage/not-found 示例未逐字钉住 | spec L225 + L170/L171；bdd S-CONTACTS-10 L371；impl_plan R4 L131 | spec L225 §5 第 2 条逐字钉住 remove/update 两条规范示例 + not-found example 形态 `paperwork contacts read <PATH>`（PATH 取用户所给实际值）；spec L170/L171 双出处同串；bdd L371-372「example 逐字钉住（rework 补录 Ryan m-2，spec §5 第 2 条同源）」三形态 example 均为逐字串；impl_plan L131 实施面同串 | 吻合 |
| m-3 label-as-key 场景 + fix 键口径教学 | bdd S-CONTACTS-07 L350-355；spec L170/L171；bdd L353；tdd L244 | bdd L352 Given 补「含名为 alice 的条目（label = alice，键 = 其 profile 路径）」；L355 新增 And 段「`--profile alice`（把 label 当键）同样 exit 1 not-found…agent 经 contacts read 一步自纠」；spec L170/L171 not-found fix 补教学句 `the key is the profile path as stored in the contacts file, not the label`（纯 ASCII）；bdd L354 同句同步；tdd L244 新增 label-as-key 用例行 | 吻合 |
| m-4 updated 箭头串偏离分键先例 | spec L173 值格式契约；bdd S-CONTACTS-08 L361；spec §7 L254 交叉引用 | spec L173 新条「编排层裁定维持箭头串形态…值格式逐字钉住为 `<OLD> -> <NEW>`（单空格分隔的三段拼接）」；bdd L361 同步逐字钉住；spec L254 §7 第 5 条「`updated` 值格式逐字钉住见 §3.6」交叉引用成立 | 吻合 |
| m-5 调研行号时效声明 | research L6 | L6 行号纪律句补时效半句（详见 Mark m-2，同一落点） | 吻合 |
| m-6 --new-profile 被否替代未落盘 | research §5.4 L124-129 被否替代表；spec L174 交叉引用 | research L124-130 新增三条被否替代表（`--to` seq 语义冲突 + D1/D2 阴影 / `--old-profile` 对称形态破坏 --profile 三动词复用 / `--replace-with` 无关联冗长），结论「认可不改名」；spec L174 末句交叉引用研究 §5.4 | 吻合 |

### 1.3 Daniel（实现可行性）10 条

| 编号 | 销账声称位置 | 复核实测 | 结论 |
|---|---|---|---|
| M-1 S-LOCK-02 断言与锁机制矛盾 | bdd S-LOCK-02 L474-478；tdd §8.2 L249 | bdd L478 主形态改为「**终态为两次编辑的字段并集**：model 取 X 且 description 取 D（后一编辑经锁内读改写读到前一编辑落盘结果再施加自身变更，无丢失写）…禁止出现仅一侧生效的丢写终态」；同字段变体口径保留（最后写入者胜，集合口径，二选一由实施方在用例内写清）；tdd L249 同步「终态为两次编辑的字段并集…同字段变体则最后写入者胜」——原矛盾（按文实现必红）消除 | 吻合 |
| M-2 S-BRIEF-07 首行与冻结冲突 | bdd L301-305；tdd L245；spec §3.5 L154 | bdd L305 首行断言改 `ok brief.read 2 entries`（conclusion 为全量条目数 N entries 现状形态，并自注 worktree cmd/brief.rs L171/L197 实测）；tdd L245 同步「ok brief.read <N> entries」；spec L154 补「未给 --entry-title 时 TOC / --full 行为冻结不变（含 conclusion `N entries` 形态…）」——与 S-BRIEF-06 冻结不再冲突（worktree 实测见 §二 F7） | 吻合 |
| M-3 白名单「追加」措辞与现状不符 | tdd §8.3 L252-260；bdd S-SHORT-02 L464；impl_plan R5 L137 | tdd L252 标题改「措辞由『追加』改为如实的『新建/扩展断言面』」；L254 新增现状基线段（6 探针/contacts 组无动词断言/ASCII 清单止于 read）；L256 负向清单「**新建/扩展**」；L257 contacts 组动词断言「**新建**…仿 post_group_help_lists_verbs 体例…含反向断言」；bdd L464 同步「断言面落点…为本轮新建，非『追加』」；impl_plan L137「口径为新建/扩展而非追加」 | 吻合 |
| M-4 格式健壮性边界未钉住 | bdd S-CONTACTS-12 L380-384 + S-CONTACTS-13 L386-390；tdd L223-224 + L241-242 | bdd L380-384：remove 最后一条目 -> 仅剩 title H1 + 空行（与 create 初态同形）+ validate 合法 + 再 remove 同键 not-found；L386-390：含空格/括号路径 update/remove 往返（键 = 未转义原串、angle-bracket 形态、往返后余条字节不变、二次操作仍命中）；tdd L223-224 core 用例两行（含 angle-bracket 断言）+ L241-242 CLI 用例两行 | 吻合 |
| m-1 计数基准二选一 | bdd L464 + tdd L256 定案 26 | 定案「共 26 项」，--name 保留在清单内；两处统一并写明「修订前 25 + --new-profile = 26」；本复核点数枚举确为 26（见 Mark M-3） | 吻合 |
| m-2 S-LOCK-01 brief 语料前置缺失 | bdd S-LOCK-01 L468-472；tdd L248 | bdd L470 Given 补「**预创建 N 个互不相同的 entry 目标文件**（与 N 个 brief 条目一一对应；brief add 须对 entry 目标文件做 SHA-256 快照，文件缺失即 io 错误 exit 1）」并写明 contacts 侧不校验的差异；tdd L248 同步「Given 预创建 N 个 entry 目标文件（brief add 快照前置）」 | 吻合 |
| m-3 新动词未入 ASCII help 防线 | tdd L250 + L260 | tdd L250 ASCII 契约扩展行补「all_help_output_is_pure_ascii 动词清单追加 contacts remove、contacts update 两行（现状清单止于 contacts create/add/read）」；L260 §8.3 第 5 条同句 | 吻合 |
| m-4 brief read --entry-title JSON 字段面歧义 | spec §3.5 L154；bdd L305；tdd L245；impl_plan R4 L131 | 定案「命中即按 --full 档字段输出（Default/JSON 同口径，不受 --full 门控）」四处同口径：spec L154「字段面口径（rework 补录）…不再受 --full 门控，Daniel m-4 定案」；bdd L305 JSON 分句同口径；tdd L245 同；impl_plan L131「命中即按 --full 档字段输出，Default/JSON 同口径，spec §3.5」 | 吻合 |
| m-5 research 对 bdd 行号漂移 | research L156 + L6 | L156 改「落盘时点实测 L391；修订后以场景号 S-SHORT-02 为准，免行号漂移」（采建议节号引用）；L6 全局时效声明（同 Mark m-2） | 吻合 |
| m-6 update 文件不存在用例 + OLD==NEW 顺序 | tdd L219 + L221；spec L171 | tdd L219 新增「contacts_update 文件不存在 -> NotFound（与 remove 文件不存在行对等，共享 exists 预检）」；L221 OLD==NEW 行补「判定顺序：OLD 命中检查先于 NEW 已存在检查（OLD==NEW 且 OLD 未命中时落入 NotFound，与 OLD 未命中行重合）」；spec L171 同步「判定顺序：OLD 命中检查先于 NEW 已存在检查」 | 吻合 |

**逐条统计：24/24 吻合，吻合率 100%。无一条为「行号存在但内容不符」的虚假销账形态。**

---

## 二、交叉一致性抽查（10 处，≥8 要求）

| # | 抽查点 | 实测结论 |
|---|---|---|
| F1 | 白名单扩容表述三处口径（feedbacks/研究/spec） | v0.7 §2.5 L68-69「不含 update…扩容为 11 动词」；research L123「update 不在既定白名单内…经 owner CRUD 指令授权的白名单扩容」；spec L70/L254「不在既定白名单内（L196，既定 10 动词），经 owner 指令 (1) 授权扩容纳入（裁定记录 v0.7_feedbacks §2.5）」——三处对「既定 10 动词不含 update / 授权来源 = owner 指令 (1) / 裁定载体 = §2.5」三点逐字同口径，无漂移 |
| F3 | 26 项计数 bdd/tdd 一致 | bdd L464「共 26 项…修订前 25 + 1 = 26」；tdd L256「26 项全量清单」；impl_plan L137「26 项无短形式负向清单」；独立点数 bdd 枚举 = 26（5+4+6+3+6+2），与文本计数自洽 |
| F4 | add 现状更正与 S-CONTACTS-14 呼应 | spec L169「add 不做任何可读性校验…纯静默回退而非任何 validation/not-found」与 bdd S-CONTACTS-14 L394「与 format-v2 R11 及 add 现状一致，属已知静默面非缺陷」、backlog B-02 L113「现状（冻结）：add/update 对目标 profile 不做任何可读性校验」三处呼应一致；原失实句「validation/not-found 沿用现状」全库已无残留（更正句自注初版失实） |
| F6 | S-LOCK-02 断言形态自洽 | bdd L478 主形态（非重叠字段并集、无丢失写、禁止丢写终态）与 spec §3.9 L193 锁内读改写机制（复刻 thread_edit）逻辑吻合；tdd L249 与 bdd 措辞一致；变体口径（同字段最后写入者胜 + 集合断言）与 BUG-5 教训（集合比较）同构 |
| F7 | brief.read conclusion 形态与 worktree 实测一致 | worktree `cmd/brief.rs` L171 JSON conclusion = `format!("{} entries", manifest.entries.len())`、L197 Default `Envelope::new("brief.read", format!("{} entries", ...))`——实测逐字命中；bdd S-BRIEF-07 L305 断言 `ok brief.read 2 entries`（2 条目场景）与现状代码形态一致，S-BRIEF-06 冻结不违反；spec L154/tdd L245 同口径 |
| 8 | updated 值格式三处同串 | spec L173「`<OLD> -> <NEW>` 单空格三段拼接」= bdd L361 断言「alice.profile.md -> carol.profile.md」= tdd L237「值逐字为 `<OLD> -> <NEW>` 单空格三段拼接」= impl_plan L131「逐字 `<OLD> -> <NEW>`」 |
| 9 | 键口径教学句四处同串 | spec L170/L171、bdd L354、tdd L238/L244、impl_plan L131 均为逐字同一句 `the key is the profile path as stored in the contacts file, not the label`（纯 ASCII） |
| 10 | usage 规范示例五处同串 | spec L170/L171/L225、bdd L372、impl_plan L131 中 remove/update 两条规范示例逐字一致（`...remove team.contacts.md --profile alice.profile.md` / `...update ... --new-profile carol.profile.md`） |
| 11 | 「六处/六写路径」计数下游全对齐 | v0.7 L47「六处」= spec §3.9 L193「六处写路径」= spec §6 L244「六写路径」= bdd S-LOCK-03 L482「六写路径」= tdd §8.6(4) L275 口径；无「五处」残留（research L87「新补锁的五个写路径」指既有路径补锁面、L151「五处写路径」指 temp+rename 否决语境，与「本轮处置合计六条」口径不同维度，非本轮 rework 触碰面，原评审亦未列为发现，记录备查不构成不闭合） |
| 12 | m-4 定案四处同口径 + m-6 被否替代表三替代理由与评审独立推演一致 | spec §3.5 L154 = bdd L305 = tdd L245 = impl_plan L131（见 §一 Daniel m-4）；research L126-130 三行被否替代理由与 Ryan m-6 评审独立推演逐条对应 |

**抽查结论：10 组全部一致，未发现口径漂移或单点孤证销账。**

---

## 三、回归检查（rework 是否引入新矛盾）

1. **bdd 章节骨架连续**：§1 post send → … → §6 contacts → … → §11 短形式 → §12 写路径锁（L466 起，收 S-LOCK-01~03 + L486 阻塞备注），无跳号、无误插。
2. **场景编号无撞号**：本轮新增 S-CONTACTS-12/13/14 在 06~11 后连续顺延，与既有各区段无撞号；S-BRIEF-07~09、S-LOCK-01~03 编号区段不变；tdd §8.2 对新场景的映射（S-CONTACTS-12/13/14、S-LOCK-01/02、S-BRIEF-07~09）逐一存在且语义对齐。
3. **冻结条款与新契约自洽**：spec §7 第 5 条 additive 登记（组集合不变/动词仅增/JSON 只增/格式零触碰/不发布）与 S-BRIEF-07 conclusion 现状形态（S-BRIEF-06 冻结）、S-CONTACTS-14 静默面（本轮不改运行时）、白名单扩容（登记为 additive 裁定而非翻案 SOTA）均自洽；「共 26 项」与 spec §4 口径差异（§4 该行不含 --name）已由 bdd L464「保留原清单 --name」明示定案，两文计数基准统一，无新矛盾。
4. **rework 注记可追溯**：全部修复点均带「rework 补录/修订 + 评审编号」溯源标注（如「Ryan M-3」「Daniel m-2」「Mark M-3/Ryan m-1/Daniel m-1 定案」），符合 append-only 式更正体例，未出现静默改写原文。

---

## 四、最终判定

**闭合放行。**

- 24 条发现（Mark 5 + Ryan 9 + Daniel 10）逐条核验全部吻合，吻合率 24/24 = 100%；
- 交叉抽查 10 处（含任务点名的 F1/F3/F4/F6/F7）全部一致，F7 经 worktree 代码实测确认 conclusion 形态真实；
- 回归检查未发现 rework 引入的新矛盾（骨架连续、编号无撞号、冻结条款自洽）；
- 未发现虚假行号或「行号存在但内容不符」的虚假销账形态；
- 备查记录（不构成打回项）：research L87/L151 两处「五个/五处写路径」为补锁面与否决语境的既有表述，与「本轮处置合计六条」属不同计数维度，原三评审均未列为发现；如后续轮次追求计数口径绝对统一，可在发布轮顺带注明，不阻塞本轮放行。

---
（复核报告完）
