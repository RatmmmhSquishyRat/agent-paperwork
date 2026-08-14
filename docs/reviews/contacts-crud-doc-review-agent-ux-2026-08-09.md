# contacts 完整 CRUD 轮治理文档集 — agent 消费者视角对抗性评审

- 日期：2026-08-09
- 任务：contacts CRUD + 写路径锁统一 + 渐进阅读补齐轮（v0.7 feedback 轮）治理文档集对抗评审，agent 消费者视角（唯一真实用户是 AI agent）
- 评审对象：
  1. `docs/ssot/adr/feedbacks/v0.7_feedbacks.md`
  2. `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md`
  3. `docs/ssot/specs/cli-grammar-v0.6/{spec,bdd,tdd,impl_plan}.md` 本轮增量修订部分
- 评审基准：v0.6 文法三规则（位置参数仅剩 PATH、必填/可选一律具名 flag、flag 唯一语义）；输出协议冻结（ok/error 信封、七类 category、退出码 0/1/2、JSON key 只增不改、usage 信封附逐字修正示例、纯 ASCII）；短形式集合冻结 {-a,-m,-q}；SOTA 命名政策（`docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` L196 C6）
- 评审方法：全文通读四份治理文档 + feedbacks + 调研报告；模拟 LLM agent 学习并使用 contacts remove/update、brief read --entry-title 的调用与错误重试路径；全部关键行号以 Read/Grep 实测——含 worktree `agent-paperwork-wt-v06grammar`（分支 cli-grammar-v0.6）代码只读核对（core 层 contacts.rs/thread.rs/manifest.rs/profile.rs 与 CLI 层 cmd/contacts.rs、cmd/brief.rs、cmd/post.rs）
- 评审范围声明：只看 agent 消费体验维度（文法一致性 / 错误路径可教学性 / 渐进阅读动线 / 输出契约 / 锁语义可见性 / 文档-代码一致性），不审 SSOT 程序合规
- 立场声明：对抗性评审，非背书

## 总体结论

本轮增量设计质量整体高：contacts remove/update 与 brief read --entry-title 与 v0.6 文法三规则同构，键语义（profile 路径 / 条目存储标题）单一且与既有判据复用，错误恢复回路（not-found/already-exists/usage）主体一次重试可闭合，渐进阅读三档动线顺畅且组合语义已钉住，输出契约严格 additive。调研报告的代码行号声称经逐项实测**全部准确**（详见「已核查、无发现」节）。但存在三处 Major：一处最高优先级文档的**事实性错误声称**（update 不在 SOTA C6 动词白名单内，两份文档均声称为在）、锁阻塞的 **agent 可见行为未定义**（无限期挂起无声明、timeout 否决仅落调研未落契约）、update 对不存在 NEW 的**静默成功面**未声明且无 CLI 级场景钉住（恰是 agent 最易犯的路径笔误形态，并牵出 spec 对 add 现状的一句失实描述）。判定：**有条件放行**；修复面全部为文档层，文法设计本身不需推翻。

---

## Issue 清单

### M-1（Major）SOTA C6 动词白名单声称失实：`update` 不在白名单内，v0.7_feedbacks 与调研报告均声称为在；新动词构成无裁定记录的白名单扩张

位置（实测）：
- `docs/researches/agent-cli-ux-industry-sota-2026-08-08.md` L196：C6 动词白名单原文为「create/send/read/list/edit/add/remove/verify/validate/summary」——**不含 update**（共 10 动词，Grep 实测）
- `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L123：「命名政策核验：update/remove/read 均在 SOTA C6 动词白名单内」——update 一项失实（remove/read 确在）
- `docs/ssot/adr/feedbacks/v0.7_feedbacks.md` L107：「动词白名单命名政策（L196 C6，update 在既定白名单内）」——最高优先级输入中的同一失实声称
- `docs/ssot/specs/cli-grammar-v0.6/bdd.md` L443（S-SHORT-02）：contacts 组动词集合断言 {create,add,remove,update,read} 将以测试强制该白名单外动词

问题：SOTA C6 的本意正是「成文命名政策 + 测试强制」以保护 agent 的词汇泛化预期。本轮新增动词 `update` 落在白名单之外，而两份上游文档以「在既定白名单内」的失实声称代替了应有的白名单扩展裁定。对 agent 消费者：白名单内已有 `edit`（post edit / profile edit 在用），`update` 与 `edit` 的语义分工（update = 键控条目的身份替换，edit = 内容原地修改）对凭词汇直觉生成命令的 agent 并非自明，而文档集没有任何一处向消费者侧教学该分工（spec §3.6、impl_plan R6 的 README/SKILL.md 刷新清单均未点名）。

建议修复：① 更正两份文档的失实声称；② 二选一并落盘裁定记录——（a）正式扩展白名单并论证 update 与 edit 的语义分工（建议方向：edit 恒为「同键内容修改」、update 恒为「换键」，载入 spec §1.4 规则 3 同级的命名裁定段），或（b）重新论证动词选型；③ impl_plan R6 补一条：SKILL.md/README 的 contacts update 示例附一句与 edit 的区分注记。

### M-2（Major）锁阻塞的 agent 可见行为未定义：`lock_exclusive` 无限期阻塞，spec §3.9 无挂起声明、无时长预期、无编排侧 timeout 指引；timeout 否决记录只落在调研而非契约

位置（实测）：
- `docs/ssot/specs/cli-grammar-v0.6/spec.md` L190（§3.9）：「冲突阻塞等待持锁；`lock_exclusive` 获取失败即 fast fail」——阻塞时长无任何界定
- 同文 L193（对外契约面）：「六路径的可观察输出……与补锁前同构，仅并发安全性增强」——**阻塞等待本身就是一个新增的可观察行为**（命令可能长时间乃至无限期不返回），该句遗漏了它
- `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L142（被否方案表）：「锁失败重试/超时机制 | 无既有先例……owner 指令即『锁定阻塞 + fast fail』两态」——否决有据，但只存在调研文档，spec/bdd 均无承接
- 代码实测：worktree `repos/paperwork-core/src/ops/thread.rs` L94/L366 使用 fs2 `lock_exclusive`（Windows 为 LockFileEx 阻塞语义，无 fail-immediately 标志），证实「阻塞即无限期」

问题：回答评审基准的直问——**会不会无限挂起？会**（持锁者长时间不释放时）；**是否需要 timeout 语义？** owner 已否决引入 timeout flag（研究 §6.1，理由成立，本评审不推翻），但「否决 timeout」不等于「可以不声明阻塞行为」。agent 在编排中执行 paperwork 命令时，一条被阻塞的 `contacts update` 会表现为进程无响应：既无退出码、也无信封，agent 侧唯一可用的自救手段（进程级超时 + 重试）没有任何文档指引。S-LOCK-01/02（bdd L447-457）只断言了「最终全部 exit 0」的乐观形态。

建议修复：spec §3.9 增补「agent 可见阻塞行为」一段，要素：① 阻塞等待无内建超时（fs2 语义，Windows/Linux 一致）；② 写临界区为锁内读改写、毫秒级，正常场景阻塞短暂；③ 持锁进程崩溃/退出后 OS 自动释放锁，不会永久死锁（Windows 句柄锁随进程消亡释放）；④ 对编排层的指引：可对 paperwork 进程施加自有时限，超时后杀进程重试（幂等的 add 与先读后写的 remove/update 重试安全，exit code 语义不变）；⑤ timeout flag 的否决记录从研究 §6.1 引用一行入 spec。bdd §12 可补一句 S-LOCK 注释钉住「阻塞无内建超时」契约。

### M-3（Major）contacts update 对不存在/不可读 NEW 的静默成功面未声明、无 CLI 级场景钉住——这是 agent 最可能犯的错（路径笔误/忘后缀/传名字当路径）；并牵出 spec 对 add 现状的一句失实描述

位置（实测）：
- `docs/ssot/specs/cli-grammar-v0.6/tdd.md` L217（§8.1）：「contacts_update label 回退 | NEW profile 不可读时 label = 文件名主干」——静默成功被钉为预期行为
- `docs/ssot/specs/cli-grammar-v0.6/spec.md` L171（§3.6 update）：「label 依 R11 对 NEW 重派生（读 NEW 目标 profile H1，失败回退文件名主干）」——只写派生规则，未声明「NEW 不存在时 update 仍 exit 0 且写入不可用 destination」这一静默面
- `docs/ssot/specs/cli-grammar-v0.6/spec.md` L169（§3.6 add）：「profile 路径不可读 -> validation/not-found 沿用现状」——**失实**：现状 add 对 profile 路径无任何可读性校验（worktree `ops/contacts.rs` L53-92 `contacts_add` 实测：不校验，直接 `derive_label` 静默回退，L121-149；CLI 层 `cmd/contacts.rs` L74-84 亦无校验），「沿用现状」的真实行为是**静默回退而非任何 validation/not-found**
- `docs/ssot/specs/cli-grammar-v0.6/bdd.md` L362-366（S-CONTACTS-09）：仅覆盖 OLD 未命中与 NEW 已存在两条错误路径，**无「NEW 不存在 -> exit 0 + 回退 label」形态的场景**（core 级仅 tdd §8.1 覆盖，CLI 信封级无钉住）

问题：agent 复现路径——`paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol`（忘写 `.profile.md` 后缀）或 `--new-profile Carol`（把显示名当键）：exit 0，条目落为 `[carol](carol)`，destination 不可用，直到下一轮 `contacts read` 才见 `(unreadable)`——恰好命中本任务评审基准点名的「传字面量文本当路径」错误形态，且是静默方向。该行为继承自 add 的冻结语义（本轮不改运行时是对的），但本轮文档的处置与本项目自有先例不一致：同为静默面的 post send `--title` 忽略，走的是「spec 显式声明 + bdd S-SEND-17 钉住 + backlog 登记」三件套（v0.6 轮 M3 的修复范式），本轮的静默面三件套全缺，且 spec 对 add 现状的描述本身是错的——后者会直接误导实施者（若实施者信了 L169，可能给 remove/update 加上 add 并不存在的校验，制造三动词行为分叉）。

建议修复：① spec §3.6 L169 更正为「profile 路径不做可读性校验（现状冻结）；不可读时 label 依 R11 回退文件名主干」；② spec §3.6 update 段按 S-SEND-17 体例补「行为登记」：update 到不存在/不可读的 NEW 仍 exit 0，destination 按原值落盘、label 回退（已知静默面，非缺陷）；③ bdd 补 CLI 级场景（update NEW 不存在 -> exit 0 + 回退 label + `updated` 字段回显原值）钉住契约；④ 「写前 destination 存在性校验/回显」作为候选增强登记 `docs/researches/ux-open-items-backlog-2026-08-08.md`，供发布轮裁决（本轮不实现，与冻结约束自洽）。

---

### m-1（Minor）S-SHORT-02 白名单项数自相矛盾：枚举实测 26 项，两处文本均称「共 25 项」

位置（实测）：
- `docs/ssot/specs/cli-grammar-v0.6/bdd.md` L443：逐项点数枚举（--seq、--stdin、--title、--to、--from、--entry、--entry-title、--profile、--new-profile、--name、--model、--description、--owner、--note、--regex、--scope-read/write/owns 三项、--full、--limit、--base-dir、--type、--json、--plain、--reply-to、--mention）= **26 项**，文本却写「共 25 项」
- `docs/ssot/specs/cli-grammar-v0.6/tdd.md` L246（§8.3 第 1 条）：「追加 `--new-profile`（……共 25 项，bdd S-SHORT-02）」——同样 stale（25 是追加前数量，追加后应为 26）

问题：这是测试断言契约，off-by-one 会直接误导 flag_inventory 用例的实现与核对。

建议修复：两处「25 项」改「26 项」，或重新清点后枚举与计数对齐。

### m-2（Minor）新命令的 usage 信封规范示例未逐字钉住，留有实施自由度

位置（实测）：
- `docs/ssot/specs/cli-grammar-v0.6/spec.md` L221（§5 第 2 条）：先例是逐字钉住（post send 规范示例给了完整字符串）
- `docs/ssot/specs/cli-grammar-v0.6/bdd.md` L368-371（S-CONTACTS-10）：只描述形态「example 分别为含 --profile / 含 --profile + --new-profile 的规范形态示例（具体值）」，无逐字串
- `docs/ssot/specs/cli-grammar-v0.6/impl_plan.md` L131（R4）：「usage 信封静态规范示例与 main.rs canonical_example 补 remove/update 两条」——同样未逐字

问题：「错误即指导」依赖 example 逐字可复制执行；本项目既有体例（spec §5 F5、v0.5 评审 M6 修复）要求具体值无占位符，但具体值是什么目前由实施者自决，QA 断言（tdd §8.2）又将逐字断言 example——钉住点与断言点错位，存在返工面。not-found 信封的 example 同理未钉（remove/update 各应给 `paperwork contacts read <PATH>` 形态）。

建议修复：在 spec §3.6 或 bdd S-CONTACTS-10 逐字钉住两条 usage 规范示例（如 `paperwork contacts remove team.contacts.md --profile alice.profile.md`、`paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md`）与两条 not-found example 形态。

### m-3（Minor）「把 label 当键」这一最高频 agent 错误无专门场景，not-found fix 未教学键口径

位置（实测）：
- `docs/ssot/specs/cli-grammar-v0.6/bdd.md` L350-354（S-CONTACTS-07）：触发值 `ghost.profile.md` 是路径形态，未覆盖 label-as-key 形态（如 `--profile alice`）
- `docs/ssot/specs/cli-grammar-v0.6/spec.md` L170：remove not-found 的 fix 仅「引导 `paperwork contacts read <PATH>` 核对条目清单」，未点名「键是 profile 路径而非 label」

问题：agent 从 `contacts read` 的富化输出（`<路径>: <name> (...)`）最容易误把 name/label 当键。当前回路可闭合（not-found -> read -> 自纠，两步），但 fix 文案不教学键口径，恢复成本多一轮推理；键语义（destination 恒为键、label 是 R11 派生不可作键）在 v0.7_feedbacks §2.1 与 spec §3.6 有充分声明，唯独没到达错误信封这个 agent 实际消费的界面。

建议修复：① S-CONTACTS-07 增加一条 label-as-key 触发形态（Given 含名为 alice 的条目，When `--profile alice`，Then not-found）；② not-found message/fix 补一句「the key is the profile path as stored in the contacts file, not the label」（纯 ASCII）。

### m-4（Minor）`updated` 信封字段为箭头拼接复合字符串，偏离既有字段的分键结构化先例

位置（实测）：
- `docs/ssot/specs/cli-grammar-v0.6/bdd.md` L360（S-CONTACTS-08）：「字段区含……`updated: alice.profile.md -> carol.profile.md`」
- 既有先例（worktree 实测）：`cmd/contacts.rs` L79-81 contacts.add 字段为分键 `contacts` + `profile`；`cmd/brief.rs` L155-157 brief.remove 字段为分键 `brief` + `removed`——既有体例是两个值两个键

问题：命名风格（`removed`/`updated` 过去分词）与既有 key 一致、additive 成立、`contacts` 键复用 contacts.add 既有 key（甚至不算新 key），这些均通过核查；唯独 `updated` 的值形态 `"OLD -> NEW"` 是箭头拼接字符串，agent 机器解析需自行拆分，与同类信封的分键先例不一致。严重度低：conclusion 首行已携带同一信息，JSON 侧 agent 亦可退回解析 conclusion。

建议修复：改为分键（如 `profile`（OLD）+ `new-profile`（NEW），或 `updated-from`/`updated-to`）；若编排层裁定维持箭头串，则在 spec §3.6 逐字钉住值格式契约（目前仅 bdd 出现）。

### m-5（Minor）调研报告对本轮修订对象的行号引用已漂移，无时效声明

位置（实测）：
- `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L12：「spec.md §2 全表（L45-60）」——修订后 spec.md 全表实测为 L48-66
- 同文 L149：「bdd.md S-SHORT-02 白名单冻结断言（L391）」——修订后实测为 L440

问题：调研成文先于 spec/bdd 本轮增量修订，行号在落盘时点真实（行号纪律成立），漂移属预期；但文档无「引用以落盘时点为准、修订后可能漂移」的声明，后续追溯者可能误判为失实引用。调研对**非修订对象**的引用（格式、评审、backlog、SOTA、代码）经抽查均准确。

建议修复：调研文首行号纪律句补半句「对 cli-grammar-v0.6 文档集的引用为本轮修订前形态，修订后行号可能漂移」。

### m-6（Minor）`--new-profile` 命名被否替代未落盘（命名本身予以认可）

位置（实测）：
- `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L123：仅一句「`--new-profile` 与既有 `--profile` 语义互补（旧键/新键），同命令内唯一语义（规则 3）成立」，无替代对比
- `docs/ssot/specs/cli-grammar-v0.6/spec.md` L212：同样只有一句登记

问题：评审基准点名「有无更优替代及其论证」。本评审独立推演的替代与否决理由：`--to <NEW>`（更短，但与 post read 独占的 `--from/--to` seq 语义冲突，且复活刚随 format-v2 删除的 send `--to` 语义阴影，违反 v0.6 轮 C1 教训）；`--old-profile/--new-profile` 对称形态（更显式，但破坏 `--profile` 在 add/remove/update 三动词间的同一对象复用——`--profile` 恒指「操作所锚定的既有键」，是规则 3 的正面资产）；`--replace-with`（语义准但与白名单动词体系无关联、冗长）。结论：`--new-profile` 是可接受乃至最优选择，**认可不改名**；缺的只是被否替代记录，使后续评审可核验取舍穷尽性。

建议修复：research §5.4 或 spec §3.6 补一张 2-3 行被否替代表（上列内容可直接采用）。

---

## 已核查、无发现的维度

1. **新文法同构性（键 flag 语义唯一性）**：`--profile` 在 add/remove/update 三处均指「条目键 = profile 路径字符串」，同一对象、精确匹配判据与 add 幂等口径复用（worktree `ops/contacts.rs` L74 实测一致），无歧义；`--entry-title` 在 brief remove（必填）/ read（可选）均指条目存储标题，spec §4 L212 已按 §1.4「同一语义对象的同构延伸」裁定体例显式登记；brief 条目标题唯一性有代码防线（worktree `ops/manifest.rs` L100-107 重复标题 AlreadyExists 拒绝），`--entry-title` 作为 read 过滤键无多命中歧义。位置参数仅剩 PATH、新 flag 一律具名、短形式集 {-a,-m,-q} 不变——三规则全部同构。
2. **渐进阅读动线**：brief read 默认 TOC -> `--entry-title` 选择性详情 -> `--full` 全量三档顺畅；组合语义（`--entry-title` + `--full` 合法、等价单条目详情）bdd S-BRIEF-09 钉住；无匹配 not-found 的 fix 引导全量 TOC 列表（S-BRIEF-08），恢复一步闭合；TOC/全量两档冻结不变有 S-BRIEF-06 回归。spec §9 登记结案表与调研 §2 证据链一致。
3. **输出契约**：command id `contacts.remove`/`contacts.update` 命名风格与既有点分形态一致；信封字段 `contacts`（contacts.add 已有）、`removed`（brief.remove 已有同名先例，worktree `cmd/brief.rs` L157 实测）为复用，`updated` 为唯一新 key，additive 成立（除 m-4 的值形态瑕疵）；ok 首行 `ok <id> <A> -> <B>` 与 contacts.add 既有 conclusion 形态一致；退出码映射（缺必填 usage 2 / not-found 1 / already-exists 1）在 spec §3.6 与 bdd 逐场景一致；ASCII 契约延伸至本轮新信封（tdd §8.6 第 3 条）。
4. **锁语义设计面（除 M-2 的可见性声明缺口外）**：六写路径清单与 owner 指令逐字对应；thread_edit 六步模板描述与代码逐行同构（worktree `ops/thread.rs` L355-521 实测：开句柄 L355-364、取锁 L366-371、锁内经持锁句柄读 L373-388、truncate+rewrite L491-511、解锁 L513-518，错误路径先 `unlock().ok()` 再返回 L393/L406/L420 均核实）；Windows os error 33 判例引用准确（worktree `cmd/post.rs` L459-469 注释、L485-487 实测）；崩溃窗口判例引用准确（format-v2 spec L228 实测）；fast-fail fix 文案沿用 thread 既有文案（thread.rs L97 实测）；S-LOCK-01~03 场景集覆盖并发不丢失/串行化/无降级不变量。
5. **文档-代码行号一致性（调研声称）**：逐条实测全部命中——`ops/contacts.rs` L1 模块注释、L17 `contacts_create`、L53 `contacts_add`、L74 幂等判据、L95 `contacts_read`、L121-149 `derive_label`；`ops/thread.rs` L61/L94/L97、L345/L366；`ops/manifest.rs` L21/L69/L145/L188；`ops/profile.rs` L78；外部引用 format-v2 spec L12/L228/L232/L330/L344、v0.3-review L78/L152、backlog L22、adr-v1 L36-37、SOTA L194 均核实一致（SOTA L196 的行号命中但声称内容失实，见 M-1）。worktree CLI 层现状（contacts 仅 create/add/read、brief read 无 --entry-title）与「本轮为实施前设计落盘」的定位一致。
6. **治理完备性**：冻结边界核对（spec §7 第 5 条 additive 登记）、不发布约束延续、ops_tests 字节级零改动防线延续（tdd §8.4）、core 新测试独立文件不并入既有防线（tdd §8.1/§8.4）、门禁链完整（impl_plan R1-R6 + QA 独立验证条款）——均无缺口。

---

## 总判定

**有条件放行。** 计数：Critical 0，Major 3，Minor 6。

### 放行条件（文档层修复，阻塞 impl_plan R1 前置门槛闭合）

1. M-1 — 更正 update 在 SOTA C6 白名单内的失实声称（两处），落盘白名单扩展裁定与 update/edit 语义分工
2. M-2 — spec §3.9 增补 agent 可见阻塞行为契约（无限期阻塞声明 + 时长预期 + 编排侧 timeout 指引 + timeout 否决记录引用）
3. M-3 — 更正 spec §3.6 对 add 现状的失实描述；update 静默成功面按 S-SEND-17 体例声明 + bdd 补 CLI 级场景钉住 + backlog 登记候选增强

### 建议修复（不阻塞，随上述一并完成）

m-1（白名单项数 25->26）、m-2（usage 规范示例逐字钉住）、m-3（label-as-key 场景与 fix 教学）、m-4（updated 字段分键或逐字钉住值格式）、m-5（调研行号时效声明）、m-6（--new-profile 被否替代落盘）。

### 附注

本轮增量文法设计（两新动词 + 一键 flag + 六路径锁统一）经 agent 消费视角检验无需推翻；三处 Major 均为「声称/声明/钉住」层面的文档完备性问题，其中最需警惕的是 M-3——静默成功恰是本项目历轮评审反复设防的失败类别（v0.6 轮 C1/M3 同源），不应在新动词上留下未声明的静默面。

---
（报告完）

---

## Rework 回应销账段（2026-08-09，实施方 Robin 补录；修复位置行号均为销账时点 Grep/Read 实测）

| 编号 | 修复位置（实测） | 状态 |
|---|---|---|
| M-1（SOTA 白名单失实 + 无裁定记录 + 无 update/edit 分工教学） | 失实声称更正：`docs/ssot/adr/feedbacks/v0.7_feedbacks.md` L113 + `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L123；扩容裁定记录：v0.7_feedbacks.md 新 §2.5 L66-71（选建议方向 (a)：正式扩容 + 语义分工落盘）；update/edit 分工教学：`spec.md` L174（§3.6 新条）+ spec.md L70/L254 登记；建议 ③（impl_plan R6 补区分注记）：`impl_plan.md` L143 | 已销账 |
| M-2（锁阻塞 agent 可见行为未定义） | `spec.md` §3.9 L196（对外契约面例外声明：阻塞为新增可观察行为）+ L197（新条「agent 可见阻塞行为契约」五要素：无内建超时/毫秒级临界区/OS 自动释放/编排侧进程级超时+重试指引/timeout 否决记录引用研究 §6.1）；bdd 承接：`bdd.md` §12 备注 L486 | 已销账 |
| M-3（update 静默成功面 + add 现状失实描述） | 建议① add 现状更正：`spec.md` L169（改为「不做任何可读性校验…纯静默回退」）；建议② update 行为登记：spec.md L172（「NEW 不存在/不可读时的行为契约」条，S-SEND-17 体例）；建议③ bdd CLI 级场景：`bdd.md` S-CONTACTS-14 L392-398（声明面+正常例+agent 自救指引三件套）；建议④ backlog 登记：`docs/researches/ux-open-items-backlog-2026-08-08.md` B-02 L113；core 侧同步钉住：`tdd.md` L222/L243 | 已销账 |
| m-1（白名单项数 25/26） | `bdd.md` S-SHORT-02 L464 与 `tdd.md` §8.3 L256 两处同改「共 26 项」 | 已销账 |
| m-2（usage/not-found 示例未逐字钉住） | usage 规范示例逐字：`spec.md` L225（§5 第 2 条）+ L170/L171（§3.6 双出处）；bdd 钉住：`bdd.md` S-CONTACTS-10 L371；not-found example 形态（`paperwork contacts read <PATH>`）：spec.md L225；实施面：`impl_plan.md` R4 L131 | 已销账 |
| m-3（label-as-key 无专门场景 + fix 不教学键口径） | 建议①：`bdd.md` S-CONTACTS-07 L350-355 新增 And 段 label-as-key 触发形态；建议②：`spec.md` L170/L171 not-found fix 补教学句 `the key is the profile path as stored in the contacts file, not the label`；bdd L353 同步；测试面：`tdd.md` L244 | 已销账 |
| m-4（updated 箭头串偏离分键先例） | 编排层裁定维持箭头串（与 conclusion 同构、不新增分键），逐字钉住值格式：`spec.md` L173（新条「updated 字段值格式契约」）；bdd 同步：`bdd.md` S-CONTACTS-08 L361；spec §7 第 5 条 L254 交叉引用 | 已销账 |
| m-5（调研行号时效声明） | `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L6 行号纪律句补时效半句 | 已销账 |
| m-6（--new-profile 被否替代未落盘） | `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` §5.4 L124-129 新增被否替代表（--to / --old-profile 对称形态 / --replace-with 三条，采评审独立推演内容）；spec §3.6 L174 交叉引用；结论认可不改名 | 已销账 |
| 「已核查、无发现」六维度 | 无需修复，rework 未触碰其认定面 | 维持 |

销账统计：本报告 9 条发现（3M+6m）全部销账，无挂起项。
