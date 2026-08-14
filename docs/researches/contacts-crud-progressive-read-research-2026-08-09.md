# contacts 完整 CRUD + 写路径锁统一 + 渐进阅读补齐：调研报告

- 日期：2026-08-09
- 性质：实施前调研（三视角合并落盘），`docs/ssot/adr/feedbacks/v0.7_feedbacks.md` 的论证依据
- 核实基线：主工作区 `repos/`（master，core 层与 worktree 分支 cli-grammar-v0.6 逐行一致，已抽查 `ops/contacts.rs` L53/L74 双树同值）；CLI 层 v0.6 文法实现以 worktree `agent-paperwork-wt-v06grammar`（分支 cli-grammar-v0.6）为基线；治理文档以 `docs/ssot/specs/cli-grammar-v0.6/` 为准
- 行号纪律：本文全部外部引用行号均为落盘前 Read/Grep 实测值，无凭记忆填写项。**时效声明（rework 补录）**：本文对 `docs/ssot/specs/cli-grammar-v0.6/` 文档集（spec.md/bdd.md）的行号引用为本轮增量修订**前**基线，修订后行号可能漂移，追溯时以被引章节号/场景号为准；对非修订对象（格式 spec、评审、backlog、SOTA、代码）的行号不受影响。

---

## 1. 现状动词盘点全表

依据 `docs/ssot/specs/cli-grammar-v0.6/spec.md` §2 全表（L45-60）逐组盘点，代码面与文档面交叉核实：

| 组 | 动词集合 | 写路径 | 读路径 | 文档证据 |
|---|---|---|---|---|
| post | send / read / summary / edit | send、edit | read（窗口过滤）、summary | spec.md L47-50 |
| profile | create / show / edit / list | create、edit | **show 即 read 能力**、list | spec.md L51-52 |
| brief | create / add / remove / read / verify | create、add、remove | read（默认 TOC + `--full`） | spec.md L53-56 |
| contacts | create / add / read | create、add | read（富化输出） | spec.md L57-59 |
| validate | validate | 无 | validate | spec.md L60 |

代码面核实（主工作区 `repos/paperwork-core/src/ops/`）：

- `ops/contacts.rs` L1 模块注释自述「Contacts operations: create, add, read — all path-explicit.」；全文件仅 `contacts_create`（L17）、`contacts_add`（L53）、`contacts_read`（L95）三个公开函数——**无 update/remove**。
- `ops/manifest.rs`：`brief_create`（L21）、`brief_add_entry`（L69）、`brief_remove_entry`（L145）、`brief_read`（L188）。
- `ops/profile.rs`：`edit_profile`（L78）。

结论：owner 指令 (2) 所述「profile, contact, brief 都有 read」在现状中全部成立（profile 的 read 能力由 show 承担）；contacts 的 CRUD 缺口 = update + remove 两个写动词。

## 2. read / 渐进阅读能力核实

owner 要求（v0.7_feedbacks §一 (2)）：brief 与 post 都应支持「先看 summary 再选择性或者全部查看详情」。逐面核实：

### 2.1 post：已满足（summary -> read 窗口过滤）

- `post summary <PATH>` 独立动词 + `post read` 窗口过滤（`--from/--to/--mention/--reply-to/--limit 20`），spec.md L49-50、L113-118。
- U-09 历史裁决：ux-review 曾提议「summary 并入 read --summary」，backlog 裁决「review 自评可接受现状, 建议保留独立命令结案」——`docs/researches/ux-open-items-backlog-2026-08-08.md` L22。

### 2.2 brief：已满足两档，缺第三档（选择性详情）

- `brief read` 默认输出 TOC（title、owner、entry count、names），`--full` 输出全量详情（path、hash、regex、note）：`docs/reviews/v0.3-review-2026-08-01.md` L72-73 实测记录，L152 UX 评估「`brief read --full` | Nice progressive disclosure: TOC default, `--full` for detail.」。
- 缺口：无法按单条目查看详情（要么 TOC 无详情，要么 `--full` 全量）。本轮以 `--entry-title <T>` 补第三档。

### 2.3 profile / contacts：直读能力已满足

- `profile show <PATH>` 单位置参数直读（结构化字段视图，spec.md L125、L131「show/edit/list 行为不变」）；`contacts read <PATH>` 富化输出（`<路径>: <name> (<description>)`，spec.md L156、bdd.md S-CONTACTS-05）。两文件本身即「单文档 = 一屏可读」粒度，无需 summary 分层，登记结案。

### 2.4 SSOT 与 SOTA 依据

- 渐进披露原话（pillars session-log）：「agent读取时候会先读到目录, 然后可以选择直接全量阅读, 或者根据路径自己手动选择性阅读」——`docs/ssot/pillars/paperwork-init-conversation/session-log-2026-07-29-agent-paperwork-user-only.md` L27；同文双落盘版本 `session-log-2026-07-29-agent-paperwork.md` L157。
- owner 同场对 summary 分层的原话：「可读性不好, 可以让post owner/users给summary, 尤其是过长的消息, 过长的消息序列段, 给summary就行」——user-only session-log L45（完整版 L300）。
- SOTA summary-before-detail：Infracost 定量证据「summary 先于细读（post summary）正是"摘要+标识符先于原始细节"，保持」——`docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` L63；结论 C4「首行结论与 Infracost "summary-before-detail" 一致」——同文件 L194；Trevin 原则 7 有界高信号响应——同文件 L22。

### 2.5 核实结论

| 面 | summary/TOC 档 | 选择性详情档 | 全量详情档 | 判定 |
|---|---|---|---|---|
| post | summary | read 窗口过滤 | read 全窗口 | 已满足 |
| brief | read 默认 TOC | **缺** -> 本轮 `--entry-title` | read --full | 补一档即满足 |
| profile | —（单文档粒度） | show 直读 | show 直读 | 已满足 |
| contacts | —（单文档粒度） | read 直读 | read 直读 | 已满足 |

## 3. contacts CRUD 缺口确认

- **代码面**：`ops/contacts.rs` 仅三公开函数（§1）；`repos/paperwork-cli/src/cmd/contacts.rs` 相应仅 create/add/read 接线。
- **文档面**：全 `docs/` 检索 `contacts (update|remove)|contacts_remove|contacts_update` 零命中（Grep 实测）；v0.6 spec §2 全表、bdd.md §6 场景集（S-CONTACTS-01~05）均无 update/remove。
- **backlog 面**：`ux-open-items-backlog-2026-08-08.md` §一~§五（U-xx/R-xx/F-xx/N-xx 全清单）无 contacts update/remove 条目。
- 结论：update/remove 三处均无既有设计，属真实缺口，本轮为首次设计落盘。

## 4. 锁现状缺口表

全 core 检索 `lock_exclusive|lock_shared` 仅两处命中，均在 `ops/thread.rs`：`thread_send` L94 与 `thread_edit` L366（`thread_edit` 完整锁区 L355-521：开句柄 L355-364 -> 取锁 L366-371 -> 锁内经持锁句柄读 L373-388 -> 锁内 truncate+rewrite L491-511 -> 解锁 L513-518）。

| 写路径 | 位置（主工作区 core） | 现状 | 风险 | 本轮处置 |
|---|---|---|---|---|
| thread_send | ops/thread.rs L61，锁 L94 | 已加锁 | — | 不变（基准模板之一） |
| thread_edit | ops/thread.rs L345，锁 L366 | 已加锁 | — | 不变（读改写模板） |
| contacts_add | ops/contacts.rs L53-92 | 无锁 read-modify-write（读 L63，写 L84） | 并发写丢条目 | 补锁 |
| contacts remove/update | 本轮新增 | — | — | 新增即带锁 |
| brief_add_entry | ops/manifest.rs L69-142 | 无锁 read-modify-write（读 L84，写 L134） | 并发写丢条目 | 补锁 |
| brief_remove_entry | ops/manifest.rs L145-185 | 无锁 read-modify-write（读 L155，写 L177） | 并发写丢失/复活条目 | 补锁 |
| edit_profile | ops/profile.rs L78 | 无锁 read-modify-write | 并发编辑互相覆盖 | 补锁 |

### 4.1 Windows 判例（QA BUG-2，os error 33）

- 判例出处：worktree 分支 cli-grammar-v0.6 `repos/paperwork-cli/src/cmd/post.rs` L459-469 注释：Windows 强制字节区间锁下，**跨句柄读取被另一进程锁定的字节区间会即时失败 ERROR_LOCK_VIOLATION（os error 33）**；旧无锁 pre-read 与并发 `thread_send` 竞态导致间歇性丢消息。解法：先取锁（阻塞至写者完成），且**经同一持锁句柄读取**（同文件 L485-487 补充注释）。
- 对本轮的约束：新补锁的五个写路径一律「开句柄 -> lock_exclusive -> 经持锁句柄读 -> 变更 -> 锁内重写 -> 解锁」，禁止锁外预读。
- fast fail 语义：`lock_exclusive` 获取本身失败（IO/锁错误）即落 `IoContext` 信封 exit 1，fix 沿用既有文案「another process may hold the lock; retry shortly」（thread.rs L97），不做无锁降级写入。

### 4.2 崩溃窗口判例

锁内 truncate + rewrite 存在断电/杀进程丢全文件窗口，format-v2 spec §5.7 已显式接受（`docs/dev/format-v2/spec.md` L228「崩溃窗口声明」）；contacts/brief/profile 补锁后的重写路径沿用该判例，本轮不做 temp+rename 加固（同判例声明的后续加固方向）。

## 5. CRUD 形态分析

### 5.1 键语义

- contacts 条目 = Markdown 链接 bullet `- [<label>](<destination>)`（format-v2 spec §7.2，`docs/dev/format-v2/spec.md` L330；§2 第 5 条 contacts 条目退化为链接 bullet，L37）。
- **键 = destination（profile 路径字符串）**：add 的幂等判据即 `c.profile_path == profile_path`（ops/contacts.rs L74）；remove/update 复用同一判据。路径比较为字符串精确匹配（与 add 一致，不做规范化——规范化会引入与 add 幂等口径不一致的新行为，属越权设计）。
- **label 是 R11 派生数据**：写入时读目标 profile H1，失败回退文件名主干（先剥 `.profile.md`，再剥 `.md`，否则取原名）——format-v2 spec §7.3（L344）、`ops/contacts.rs` `derive_label` L121-149。推论：
  - label 不可作键（同一 profile 改名后 label 变化而 destination 不变）；
  - update 换 profile 后 label 必须依 R11 对新 destination 重派生，**不支持手工指定 label**（无 `--label` flag）；
  - 条目顺序在 update 后保留（原地替换，不重排）。

### 5.2 锁模板

复刻 `thread_edit`（ops/thread.rs L355-521）六步：`OpenOptions read+write` 开句柄 -> `lock_exclusive` -> seek(0) + 经持锁句柄 `read_to_string` -> 解析/变更/序列化 -> `set_len(0)` + seek(0) + `write_all` -> `unlock`。错误路径在解锁后返回（先 `unlock().ok()` 再返回 Err，与 thread_edit L393/L406/L420 等点位同构）。

### 5.3 格式零触碰论证

- format-v2 spec L12 冻结条款：「profile/brief/contacts 三格式语义不变」。
- remove/update 只操作既有构造（链接 bullet 的删除与原地替换），不引入新 Markdown 构造、不改序列化规则（`serialize_contacts` 复用）、不改解析规则；brief/profile 补锁为纯并发机制变更，文件字节产物与无锁路径一致（同一 serialize 函数）。格式层零触碰成立。

### 5.4 文法增量（逐字进 spec §2）

| 命令 | 签名 | 契约要点 |
|---|---|---|
| contacts remove | `contacts remove <PATH> --profile <P>` | 键=profile 路径；命中删除、未命中 not-found exit 1；缺 `--profile` -> usage exit 2；锁内读改写 |
| contacts update | `contacts update <PATH> --profile <OLD> --new-profile <NEW>` | OLD 未命中 not-found、NEW 已存在 already-exists（exit 1）；label 依 R11 重派生；条目顺序保留；不支持改 label；缺必填 flag -> usage exit 2；锁内读改写 |
| brief read | `brief read <PATH> [--full] [--entry-title <T>]` | 新增可选过滤，复用 brief remove 的 `--entry-title` 键语义（存储标题/basename 推导）；无匹配 not-found exit 1；与 `--full` 组合合法（等价于单条目详情） |

- 新 flag 仅长形式（短形式集 {-a,-m,-q} 不变，spec §4 F3 收窄裁定延续）；新 command id `contacts.remove`/`contacts.update`（JSON additive）；信封新字段 `contacts`（目标 contacts 路径）、`removed`（被删 profile 路径）、`updated`（OLD -> NEW）。
- 命名政策核验（rework 更正，2026-08-09）：`remove`/`read` 在 SOTA C6 既定动词白名单内（`docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` L196，既定白名单逐字为 create/send/read/list/edit/add/remove/verify/validate/summary）；**`update` 不在既定白名单内**，本轮属「经 owner CRUD 指令（v0.7_feedbacks §一 (1)）授权的白名单扩容」，扩容裁定记录见 v0.7_feedbacks §2.5（本文初版曾失实表述为「均在白名单内」，经双评审指出后更正）；`--new-profile` 与既有 `--profile` 语义互补（旧键/新键），同命令内唯一语义（规则 3）成立。
- `--new-profile` 命名被否替代（rework 补录，Ryan m-6；结论：认可不改名）：

| 被否替代 | 否决理由 |
|---|---|
| `--to <NEW>` | 更短但与 post read 独占的 `--from/--to` seq 语义冲突（规则 3）；且复活刚随 format-v2 owner 追裁 D1/D2 删除的 send `--to` 语义阴影（v0.6 轮 C1 教训） |
| `--old-profile/--new-profile` 对称形态 | 更显式但破坏 `--profile` 在 add/remove/update 三动词间的同一对象复用——`--profile` 恒指「操作所锚定的既有键」，是规则 3 的正面资产 |
| `--replace-with` | 语义准但与白名单动词体系无关联、冗长 |

## 6. 三视角方案对比摘要

沿用本项目三视角评审惯例（简洁可维护 / agent 效率 / 最小变更，先例见 `docs/researches/cli-grammar-v06-reassessment-2026-08-09.md` 体例）：

| 视角 | 对采纳方案（两动词 + 锁统一 + `--entry-title`）的评价 |
|---|---|
| 简洁可维护 | 键语义单一（destination 字符串比较）；锁模板唯一（thread_edit 六步），可抽 helper 供五处复用；无新格式构造、无新错误 category |
| agent 效率 | 错误路径一次重试可自愈（not-found fix 指向 `contacts read`、already-exists fix 指向先 remove）；brief 选择性详情直接省 token（对齐 Infracost 谓词下推结论，SOTA L62） |
| 最小变更 | 文件格式零触碰；输出协议只增不改；既有命令行为不变（contacts add 仅补并发安全性，幂等语义不变）；不发布约束延续 |

### 6.1 被否方案

| 被否方案 | 否决理由 |
|---|---|
| contacts 维持 append-only，不支持 update/remove | owner 显式否决（v0.7_feedbacks §一 (1)）；append-only 条款适用域仅限通信线程（§4 引证链）；brief 早有 remove 动词，contacts 独缺无理由 |
| contacts set（覆盖式重写整个名单） | 违背最小变更；整表替换放大并发冲突面与误操作损失面；既有 add/remove/update 三原语已覆盖全部需求 |
| update 支持改 label（`--label` flag） | label 是 R11 派生数据（format-v2 §7.3），用户可写 label 会制造「label 与 profile H1 漂移」的新不一致面；改名需求由 profile edit 自然传导（下次 add/update 重派生） |
| 锁失败重试/超时机制 | 无既有先例（thread 路径即阻塞 + IO 错误 fast fail）；引入超时参数扩大 flag 面，违背最小变更；owner 指令即「锁定阻塞 + fast fail」两态 |
| brief 选择性详情用位置参数或 seq 序号 | 违背 v0.6 规则 1（位置参数仅剩 PATH）；条目无 seq 概念，键为存储标题，与 brief remove 既有 `--entry-title` 同构复用成本最低 |
| temp+rename 原子重写 | format-v2 §5.7 判例已接受崩溃窗口且明言「后续加固方向，本次不做」；本轮保持一致，避免五处写路径行为分叉 |

## 7. 冻结边界与 additive 关系核对

- v0.6 spec §7（L216-221）：core API 与文件格式零变更、输出协议只增不改、本轮不发布。本轮：core **新增**函数（`contacts_remove`/`contacts_update`）不改既有签名；既有五函数行为面仅增加并发安全性（锁内语义与无锁语义产物一致）；文件格式零触碰；command id 与 JSON key 只增；不发布约束延续。属 additive 扩展，需在 spec §7 补记（已列入 spec.md 修订清单）。
- bdd.md S-SHORT-02 白名单冻结断言（落盘时点实测 L391；本轮增量修订后位于 §11，以场景号 S-SHORT-02 为准，免行号漂移）：组/动词集合、flag 全表、短形式集 {-a,-m,-q} 三面需随本轮 additive 扩展同步更新（动词集合 {profile,post,brief,contacts,validate} 不变；flag 面新增 `--new-profile`，`--entry-title` 为既有 flag 新增可选语境；短形式面不变）。

---
(报告完)
