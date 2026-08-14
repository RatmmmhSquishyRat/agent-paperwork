# contacts CRUD 轮治理文档评审：实现可行性与格式健壮性

- 日期：2026-08-09
- 评审维度：实现可行性与格式健壮性（批判性评审）
- 被评审文档（全部通读）：
  1. `docs/ssot/adr/feedbacks/v0.7_feedbacks.md`（108 行）
  2. `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md`（153 行）
  3. `docs/ssot/specs/cli-grammar-v0.6/spec.md`（275 行）、`bdd.md`（464 行）、`tdd.md`（266 行）、`impl_plan.md`（154 行）的本轮增量修订部分
- 对照代码基线（只读实测）：worktree `agent-paperwork-wt-v06grammar`（分支 cli-grammar-v0.6，v0.6 实施完成态）下 `repos/paperwork-core/src/ops/{contacts,thread,manifest,profile}.rs`、`format/contacts.rs`、`error.rs`、`hash.rs`；`repos/paperwork-cli/src/{main.rs, output.rs, cmd/contacts.rs, cmd/brief.rs, cmd/post.rs}`、`tests/cli_integration.rs`（2503 行，113 项 `#[test]` 实测）；`paperwork-core/tests/ops_tests.rs`（1146 行，51 项 `#[test]` 实测）
- 行号纪律：本文全部行号为落盘前 Read/Grep/Select-String 实测值，无凭记忆填写项
- 编号约定：C-n（致命/阻断）、M-n（重大，实现启动前应修复）、m-n（轻微，可随实现顺带修复）

---

## 一、锁模板可行性

**结论：已核查、无发现（引用行号全部实测吻合）。**

1. `thread_edit` 六步锁模板实读确认（worktree `ops/thread.rs`）：
   - 存在性预检 L346-353；开 read+write 句柄（无 create）L355-364；`lock_exclusive` L366-371；锁内经持锁句柄 seek(0)+read_to_string L373-388；锁内 set_len(0)+seek(0)+write_all L491-511；末尾 unlock 带错误映射 L513-518；错误路径先 `unlock().ok()` 再返回 Err 的点位实测有 L393、L406、L420、L441、L456、L475 等，与调研文档 §5.2（L107）所述「L393/L406/L420 等点位同构」一致。
   - 锁获取失败 fix 文案 `another process may hold the lock; retry shortly` 实测于 `thread_send` L97 与 `thread_edit` L369，spec §3.9（L190）与 impl_plan R1（L113）的「沿用既有文案」成立。
2. 调研文档 §4 锁缺口表的行号逐条实测吻合：`thread_send` L61/L94、`thread_edit` L345/L366、`contacts_add` L53-92（读 L63、写 L84、幂等判据 L74）、`brief_add_entry` L69-142（读 L84、写 L134）、`brief_remove_entry` L145-185（读 L155、写 L177）、`edit_profile` L78（读 L95、写 L121）、`derive_label` L121-149、`ops/contacts.rs` L1 模块注释「create, add, read」均与代码一致。
3. Windows os error 33 判例吸收核查：判例原文实测于 worktree `cmd/post.rs` L459-469（注释：Windows 强制字节区间锁下跨句柄读被锁区间即时失败 ERROR_LOCK_VIOLATION）与 L485-487（经同一持锁句柄读取）；v0.7_feedbacks §2.2.4（L48）、调研 §4.1（L86-88）、spec §3.9（L191）、impl_plan R1（L113）四处均正确承继「锁内仅经持锁句柄读取」约束，无遗漏。
4. 锁内跨文件读取安全性核查：新路径在持锁期间的外部读取只有 `derive_label` 读 profile 文件（ops/contacts.rs L121-149）与 `brief_add_entry` 经 `hash::hash_file` 读 entry 文件（ops/manifest.rs L118、hash.rs L22-30），均为**另一文件**，不构成对被锁文件的跨句柄读，os error 33 面不暴露。
5. 锁内产物等价性核查：无锁路径与锁内路径均走同一 `serialize_contacts`/`serialize_manifest`/`serialize_profile`（parse→serialize 往返一致），tdd §8.4「锁内产物与无锁产物逐字节一致」与 §8.1「锁内序列化等价」用例可实现。
6. 遗漏步骤排查：impl_plan R1（L113）六步描述（开句柄→取锁→seek(0)+持锁句柄读→变更序列化→set_len(0)+seek(0)+write_all→unlock）与 thread_edit 实测逐步对应，错误路径先解锁、fast fail 落 IoContext、崩溃窗口沿用 format-v2 §5.7（实测 `docs/dev/format-v2/spec.md` L228）均已写明，无关键步骤遗漏。

## 二、brief/profile 补锁影响面（既有测试防线）

**结论：已核查、无发现。「ops_tests.rs 字节级零改动」防线守得住。**

1. 测试规模实测核对：`ops_tests.rs` 51 项、`cli_integration.rs` 113 项 `#[test]`，与任务口径一致。
2. 涉及补锁函数的既有用例逐一排查（ops_tests.rs）：
   - `edit_profile_updates_fields`（L106-136）：顺序单进程，断言为 `serialize_profile` 确定性产物（`## Scope`、`- read:` 等 contains 断言），锁内产物字节一致，不受影响；
   - brief 面：`brief_add_entry_hash_full`（L858-880）、`brief_remove_entry`（L883-896）、`brief_remove_entry_not_found`（L899-907）、`brief_verify_three_states`（L910-942）、`brief_verify_newline_sensitive`（L945-960），全部顺序调用、断言面向解析结果/hash/三态，不断言错误文案与 IO 细节；
   - contacts 面：`contacts_create_writes_file`（L967）、`contacts_create_rejects_overwrite`（L979）、`contacts_add_and_read`（L989 起，用不存在路径触发 label 回退），同样不受锁影响。
   - 全文件 Grep 实测：ops_tests.rs 无任何 `File::open`/`OpenOptions`/持句柄模式，无并发场景，无锁相关错误文案断言。
3. cli_integration 涉补锁函数用例排查：`brief_create_add_read`（L255-303）、`brief_remove`（L282-303）、`brief_add_remove_basename_mapping`（L1146-1175）、`brief_missing_required_flags_are_usage`（L2313-2342，usage 层不进 core）、profile edit 断言 `changed: model`（L106-111，CLI 信封层，core 补锁不改信封）、`contacts_create_add_read`（L335-359）、`contacts_add_missing_profile_is_usage`（L2345 起）——全部顺序单进程，成功路径信封/退出码/文件产物在补锁后不变，失败路径（NotFound/AlreadyExists/IoContext）的 category 与文案不受锁改造影响（锁获取失败仅新增于「另一进程持锁」场景，测试环境不可达）。
4. 结论：补锁仅增强并发安全性，不改变任一既有用例的可观察断言面；51+113 项预期全绿，tdd §8.4 防线声明成立。

## 三、contacts update 语义可实现性

**结论：已核查，主体无发现；一致性口径见 m-6。**

1. `derive_label`（ops/contacts.rs L121-149）实读确认：该函数**永不失败**——目标 profile 按「原样路径 → contacts 文件父目录相对路径」解析，读取或解析失败即回退文件名主干（先剥 `.profile.md` 再剥 `.md`，否则原名）。spec §3.6（L171）「读 NEW 目标 profile H1，失败回退文件名主干」与 tdd §8.1「contacts_update label 回退」用例（tdd.md L217）对新路径 profile 不存在/不可读的失败路径**已有定义**，且与代码行为一致；label 取 `profile.name` 即 profile 文件 H1（format/profile.rs L30-34 实测），R11 引证（format-v2 spec L344）实测吻合。
2. already-exists 判定与 add 幂等口径核查：两者判据同为 destination 字符串精确匹配（add 幂等判据实测 ops/contacts.rs L74 `c.profile_path == profile_path`，无规范化，调研 §5.1 L99 的「不做规范化」与代码一致）。外显行为存在刻意不对称：add 已存在条目 → Ok no-op（S-CONTACTS-11 L373-377 钉住）；update 的 NEW 已在清单 → AlreadyExists exit 1（S-CONTACTS-09 L362-366、tdd §8.1 OLD==NEW 行 L220 钉住）。该不对称在「绝不产生重复 destination」目标下自洽（update 原地替换若放行已存在 NEW 将造成重复条目），文档两处均有钉住用例，可实现、不矛盾。
3. 键语义与顺序保留：`parse_contacts` 保持行序（format/contacts.rs L34-49），原地替换 + `serialize_contacts` 保序输出，「条目顺序保留」（spec §3.6、S-CONTACTS-08 L356-360）可实现。

## 四、格式健壮性

**发现：M-4（见下）。**

- update/remove 复用 `serialize_contacts`（format/contacts.rs L173-194），转义（`]`→`\]`、反斜杠自反、空格/制表/括号/尖括号走 `<...>` 形态）与既有 format 层往返测试（T-FC-01~08、B2、N1，L207-371）完整存在，序列化面本身健壮。
- 缺口：本轮 bdd/tdd 未覆盖 update/remove 经锁内 parse→serialize 往返后的边界形态（详见 M-4）。

## 五、测试可执行性（tdd §8 / bdd §12）

**发现：M-1、M-2、m-2、m-4、m-6（见下）。**

- tdd §8.2 用例表逐条可落地性核查：contacts remove/update 成功与错误路径的断言对象（`Envelope::new(command, conclusion)` 首行 `ok <command> <conclusion>`、field 区、`--json` key）与现有 cmd/contacts.rs add 臂（L74-84）同构，`canonical_example`（main.rs L323-401）补 remove/update 两臂可行（R4 已列，impl_plan L131）；brief read `--entry-title` 过滤在 cmd/brief.rs Read 臂（L162-219）上加可选过滤可行。
- BUG-5 flaky 教训吸收核查：既有 `multiprocess_concurrent_send_no_lost_messages`（cli_integration.rs L2022-2089）已于 L2068-2070 钉住「并发提交顺序不确定，断言用集合比较（BTreeSet）而非逐位配对」；tdd §8.2 S-LOCK-01 行（L240）「条目集合 = 并集」与 tdd §8.1「多线程并发 add/remove：条目无丢失」均为集合/总量口径，**已正确吸收**。
- Windows CI 可执行性核查：CI 三平台矩阵（ubuntu/macos/windows，ci.yml L16）下已有 10 进程并发实测先例（L2022 起），S-LOCK-01/02 同形态（spawn N 个 `paperwork` 子进程）可实现；S-LOCK-03 自述为代码级不变量（code review + 锁点位盘点，tdd §8.6 第 4 条 rg 门禁 L264），不要求 OS 级锁失败模拟，可执行。
- S-LOCK-02 断言与机制矛盾（M-1）、S-BRIEF-07 首行与冻结矛盾（M-2）、brief 并发语料缺口（m-2）、brief read JSON 字段面歧义（m-4）、update 文件不存在用例缺失（m-6）详见发现清单。

## 六、bdd 白名单断言更新的实现一致性

**发现：M-3、m-1、m-3（见下）。**

现状实读（cli_integration.rs）：
- `naming_policy_whitelist`（L1224-1238）：仅断言五组在 root help 出现 + 隐藏别名不外泄；**无任何组的动词集合精确断言**；
- `short_form_whitelist_is_exact`（L2427-2475）：四屏 help 短形式集合精确断言（send/edit 为 {a,h,m,q}、read 为 {h,q}、root 为 {V,h,q}）+ **仅 6 个负向探针**（L2460-2467：`-s/-l/-n/-t/-e/-p`），不存在 25 项逐 flag 负向清单；
- 组级动词清单断言仅 post 组存在（`post_group_help_lists_verbs` L1480-1491），contacts/brief 组无对应用例（Grep `"contacts", "--help"` 零命中实测）；
- `all_help_output_is_pure_ascii` 动词清单（L2485-2490）枚举至 `contacts create/add/read`，未含本轮新动词。

与 bdd S-SHORT-02（L440-443）要求的「contacts 组动词集合精确等于 {create,add,remove,update,read}」「全 CLI flag 名集合与 spec §4 全表一致」「25 项无短形式逐一断言」对照，tdd §8.3（L246-247）以「追加」措辞描述的工作量与现状不符（详见 M-3）。

---

## 七、发现清单

### C 级（致命/阻断）

无。

### M 级（重大）

#### M-1 bdd S-LOCK-02 的结果断言与其自身规定的锁内读改写机制矛盾（按文实现必红）

- **位置**：`docs/ssot/specs/cli-grammar-v0.6/bdd.md` L453-457（S-LOCK-02）
- **问题**：场景规定两进程并发执行 `profile edit --model X` 与 `profile edit --description "..."`（两字段**不重叠**），Then 断言「最终文件……内容为两次编辑之一的完整结果」。但按 spec §3.9 锁内读改写（复刻 thread_edit）：两写者串行，后一编辑**读取前一编辑的落盘结果**再施加自身字段变更，终态必为两次编辑的**字段并集**（model=X 且 description=新值），既不等于「仅编辑一」的结果（model=X、description 旧值），也不等于「仅编辑二」的结果（model 旧值、description=新值）。按原文写断言，正确实现必然测试失败；「禁止出现部分字段混合的中间态」措辞又恰好把唯一正确终态（两字段均变）描述成了违例形态。
- **建议修复**：将 Then 改为「两者均 exit 0；最终文件 validate 通过，且 model 与 description 两字段**各自取自两次编辑的写入值**（非重叠字段串行合并），无交错损坏字节」；或改用例设计为两进程编辑**同一字段**（此时终态为两者之一才成立，但仍需以集合/枚举口径断言而非逐位配对）。

#### M-2 bdd S-BRIEF-07 的 ok 首行契约与既有冻结结论形态冲突，实现方被迫二选一

- **位置**：`docs/ssot/specs/cli-grammar-v0.6/bdd.md` L301-305（S-BRIEF-07）；冲突对照：bdd.md L297-299（S-BRIEF-06 冻结）、worktree `repos/paperwork-cli/src/cmd/brief.rs` L171（JSON conclusion）与 L197（Default conclusion）
- **问题**：S-BRIEF-07 Then 断言「stdout 首行 `ok brief.read <path>`」。实测现状：brief read 信封 conclusion 为 `"{N} entries"`（`Envelope::new("brief.read", format!("{} entries", ...))`，output.rs L81 首行即 `ok brief.read N entries`，不含路径）。而 S-BRIEF-06 明确「read……输出样貌……与 v0.5 完全一致」属冻结面。两者矛盾：保持冻结则 S-BRIEF-07 首行断言不可满足；改 conclusion 为路径则违反 S-BRIEF-06 冻结与 tdd §3 保留清单（`ok brief.read` 首行口径）。
- **建议修复**：将 S-BRIEF-07 首行改为 `ok brief.read <N> entries`（与冻结一致）；若确有意变更 conclusion 形态，须先由编排层显式裁决并同步修订 S-BRIEF-06 与 tdd §3，不得在 BDD 内部静默冲突。

#### M-3 tdd §8.3 白名单更新以「追加」措辞描述，但现状不存在可追加的断言面（工作量低估、落点缺失）

- **位置**：`docs/ssot/specs/cli-grammar-v0.6/tdd.md` L246-247（§8.3 第 1/2 条）；现状对照：worktree `cli_integration.rs` L2427-2475（short_form_whitelist_is_exact，负向探针仅 6 个，L2460-2467）、L1224-1238（naming_policy_whitelist，无动词集合断言）、L1480-1491（组级动词断言仅 post 组存在）
- **问题**：(1) 第 1 条称「无短形式负向断言清单**追加** `--new-profile`（……共 25 项，bdd S-SHORT-02）」——实测现状不存在 25 项清单，仅有 `-s/-l/-n/-t/-e/-p` 六个一次性负向探针；(2) 第 2 条称「contacts 组 help 动词列表断言**追加** remove/update」——实测现状不存在任何 contacts 组动词列表断言（`paperwork contacts --help` 无用例覆盖）。按 tdd 原文执行，实施者找不到追加点位；bdd S-SHORT-02 要求的「动词集合精确等于」「逐一断言无短形式」两面实际均需**新建**断言面。
- **建议修复**：tdd §8.3 改为明示「新建」：(1) 仿 `post_group_help_lists_verbs`（L1480）体例新建 contacts 组动词精确集合断言（含反向断言：不出现清单外动词）；(2) 将 25 项（计数问题另见 m-1）无短形式负向断言作为新建清单落 `short_form_whitelist_is_exact` 或独立用例，`--new-profile` 探针建议形态 `contacts update <PATH> --profile a.profile.md --new-profile b.profile.md` 加 `-N`/`-w` 类短形式误写触发 usage exit 2。

#### M-4 格式健壮性边界未被定义与钉住：remove 最后一条目后的文件形态、特殊字符路径的 remove/update 往返

- **位置**：`docs/ssot/specs/cli-grammar-v0.6/bdd.md` L344-377（S-CONTACTS-06~11 全集）、`tdd.md` L211-223（§8.1 用例表）；行为基准对照：worktree `format/contacts.rs` L173-194（serialize_contacts 空条目输出 `# <title>\n\n`）
- **问题**：(1) 「remove 最后一条目后文件形态」在 spec/bdd/tdd 三处均无定义——按 `serialize_contacts(title, &[])` 实测产物为仅 H1 + 空行（`# <title>\n\n`，与 contacts create 初态同形），该形态合法且可被 validate/parse 接受，但无用例钉住，实现与测试均可能各凭理解；(2) remove/update 走锁内 parse→serialize 往返，但无用例覆盖需转义路径（空格/制表/括号/尖括号/尾随反斜杠路径经 `(<...>)` 形态序列化，label 含 `]`）在 update/remove 键匹配与重写后的往返一致性——既有 T-FC 系列只钉 serialize/parse 层，不钉 ops 层 remove/update 经键匹配命中转义条目的路径（键为未转义原串，序列化后形态变化，二次 remove/update 仍须命中）。
- **建议修复**：bdd §6 增「remove 最后一条目 -> 文件仅剩 title（与 create 初态同形）、validate 合法、再 remove 同键 not-found」与「update/remove 命中含空格/括号/反斜杠路径条目（键 = 未转义原串）成功且往返后其余条目字节不变」两场景；tdd §8.1 增对应 core 用例（含 update 后新路径含空格走 angle-bracket 形态的断言）。

### m 级（轻微）

#### m-1 bdd S-SHORT-02 枚举 26 项却声称「共 25 项」，tdd §8.3 沿用同一计数

- **位置**：`bdd.md` L443；`tdd.md` L246；对照 `spec.md` L208（§4「其余全部 flag」行）
- **问题**：S-SHORT-02 行内枚举实际为 26 项（含 `--name`，而 spec §4 该行枚举不含 `--name`）：seq/stdin/title/to/from/entry/entry-title/profile/new-profile/name/model/description/owner/note/regex/scope-read/scope-write/scope-owns/full/limit/base-dir/type/json/plain（24）+ reply-to + mention = 26，与「共 25 项」不符；spec §4 口径（无 --name）恰为 23+2=25。两文档计数基准不一致。
- **建议修复**：二选一并统一两文：若 `--name` 纳入负向清单则改「共 26 项」，否则从枚举中删除 `--name` 并保持 25 项。

#### m-2 S-LOCK-01 brief 侧并发语料前置条件缺失（entry 目标文件必须存在）

- **位置**：`bdd.md` L447-451（S-LOCK-01）；`tdd.md` L240（§8.2 对应行）；行为依据：worktree `ops/manifest.rs` L118 + `hash.rs` L22-30（entry 文件缺失 → IoContext exit 1）
- **问题**：场景称「另一组 N 个进程并发执行 brief add（互不相同的条目）」且 Then「全部 exit 0」，但 `brief_add_entry` 须对 entry 目标文件做 SHA-256 快照，文件不存在即 io 错误，N 个进程中全部 exit 0 要求 N 个 entry 文件预先创建——语料前置条件未写明，照字面实现会造出必红用例。
- **建议修复**：Given 补「预创建 N 个互不相同的 entry 目标文件（与 N 个条目一一对应）」。

#### m-3 新动词的 verb 级 help 未纳入 ASCII 全量防线清单

- **位置**：`tdd.md` L242（§8.2「ASCII 契约扩展」行）；现状对照：worktree `cli_integration.rs` L2485-2490（`all_help_output_is_pure_ascii` 动词清单止于 `contacts create/add/read`）
- **问题**：tdd §8.2 ASCII 扩展仅提「remove/update 的 usage/not-found/already-exists 信封 stderr」，未点名 `all_help_output_is_pure_ascii` 的动词清单需同步追加 `contacts remove` / `contacts update` 两行（该清单是逐 verb help 的 ASCII 逐字节防线，遗漏即新 help 面失去覆盖）。
- **建议修复**：tdd §8.2（或 §8.3）补一句：`all_help_output_is_pure_ascii` 动词清单追加 `contacts remove`、`contacts update`。

#### m-4 brief read `--entry-title` 命中时的 JSON 字段面存在歧义（是否自动采用 --full 档字段）

- **位置**：`spec.md` L154（§3.5）；`bdd.md` L301-305（S-BRIEF-07）；现状对照：worktree `cmd/brief.rs` L179-186（JSON 仅 `full` 时输出 regex/note）
- **问题**：spec §3.5 称命中时输出「path/hash/regex/note，即 --full 档字段」，但 S-BRIEF-07 对 `--json` 只钉「entries 数组仅含该条目」，未钉字段面；现状 JSON 在非 --full 时不含 regex/note。未给 `--full` 而给 `--entry-title` 时，JSON 是否输出 regex/note 未定义，两种实现均能满足现文档文本。
- **建议修复**：在 spec §3.5 与 S-BRIEF-07 明示：`--entry-title` 命中即按 --full 档字段输出（Default 与 JSON 两档同口径），或明示 JSON 字段面仍受 `--full` 门控。

#### m-5 调研文档对 bdd 的引用行号未随同轮修订同步（L391 vs 实测 L440）

- **位置**：`docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L149（§7「bdd.md S-SHORT-02 白名单冻结断言（L391）」）；实测现状：`bdd.md` L440-443
- **问题**：研究文档先行落盘、bdd 同轮修订后行号漂移，与其自述「行号纪律：全部外部引用行号均为落盘前实测值」产生表面冲突，易误导后续引用者。
- **建议修复**：将该处行号校正为 L440（或改为节号引用 S-SHORT-02，免行号漂移）。

#### m-6 tdd §8.1 缺 contacts_update「contacts 文件不存在」用例；OLD==NEW 判定顺序未写明

- **位置**：`tdd.md` L213-220（§8.1 用例表）
- **问题**：表中 remove 有「文件不存在 → NotFound（resource Contacts）」行，update 无对等行（实现上两者共享 exists 预检，但测试面不对称）；OLD==NEW 行仅给结论（AlreadyExists），未写明判定顺序（须 OLD 命中在先、NEW 存在性检查在后，否则 OLD==NEW 且 OLD 未命中时的分支行为未定——该分支实为 NotFound，与「OLD 未命中」行重合，但文档未点破）。
- **建议修复**：补 update 文件不存在用例一行；OLD==NEW 行加注「判定顺序：OLD 命中检查先于 NEW 已存在检查」。

---

## 八、各维度核查结论汇总

| 维度 | 结论 |
|---|---|
| 锁模板可行性（thread_edit 复刻、os error 33 吸收） | 已核查、无发现（六步模板与判例引用行号全部实测吻合） |
| brief/profile 补锁影响面（51+113 项既有测试） | 已核查、无发现（字节级零改动防线成立） |
| contacts update 语义可实现性（derive_label 失败路径、already-exists 口径） | 已核查、无发现（m-6 为用例表完备性小项） |
| 格式健壮性（往返/空条目/仅剩 title） | M-4（最后条目移除形态与转义路径往返未定义未钉住） |
| 测试可执行性（tdd §8 逐条、S-LOCK-* 于 Windows CI、BUG-5 教训） | M-1、M-2、m-2、m-4（BUG-5 集合比较教训已正确吸收；多进程形态有 10 进程先例可落地） |
| bdd 白名单断言更新的实现一致性 | M-3、m-1、m-3（「追加」前提与现状断言面不符） |

---

## 九、总判定

**有条件放行。**

- 发现数量统计：C-0 / M-4 / m-6，合计 10 项。
- 放行条件：M-1（S-LOCK-02 断言矛盾）、M-2（S-BRIEF-07 首行与冻结冲突）、M-3（白名单更新落点缺失）、M-4（格式健壮性边界未钉住）四项须在 impl_plan 步骤 R1 启动前修订闭合（口径同 impl_plan 文首前置门槛「对抗评审闭合后方可开始步骤 R1」）；m-1~m-6 可随 R2~R5 实施顺带修复，但 m-1 计数基准须在 M-3 修订时一并定案。
- 判定理由：锁模板、补锁影响面、update 语义三大可行性主轴实测全部成立，文档集引用纪律整体优秀（research 全部行号实测吻合）；剩余问题集中于测试契约文本的内部矛盾与断言面落点失实，均为文档面可修复项，不动设计主轴。

---

## Rework 回应销账段（2026-08-09，实施方 Robin 补录；修复位置行号均为销账时点 Grep/Read 实测）

| 编号 | 修复位置（实测） | 状态 |
|---|---|---|
| M-1（S-LOCK-02 断言与锁机制矛盾） | `docs/ssot/specs/cli-grammar-v0.6/bdd.md` S-LOCK-02 L474-478：主形态改为「非重叠字段串行合并，终态 = 两次编辑的字段并集（model=X 且 description=D，无丢失写）」；保留同字段变体口径（最后写入者胜，集合口径断言）并写明二选一由实施方选定；tdd 同步：`tdd.md` §8.2 L249 | 已销账 |
| M-2（S-BRIEF-07 首行与冻结冲突） | `bdd.md` S-BRIEF-07 L301-305：首行断言改为现状形态 `ok brief.read 2 entries`（conclusion = 全量条目数 `N entries`，worktree cmd/brief.rs L171/L197 实测口径，S-BRIEF-06 冻结不违反）；tdd 同步：`tdd.md` L245；spec 侧冻结声明：`spec.md` §3.5 L154（补 conclusion N entries 形态冻结描述） | 已销账 |
| M-3（白名单「追加」措辞与现状断言面不符） | `tdd.md` §8.3 L252-260：措辞改为「新建/扩展断言面」，新增现状基线段（6 探针/无 contacts 动词断言/ASCII 清单止于 read），落点：仿 `post_group_help_lists_verbs` 新建 contacts 组动词集合断言（含反向断言）、26 项负向清单新建/扩展、`--new-profile` 探针建议形态；bdd 同步改写：`bdd.md` S-SHORT-02 L464；impl_plan 同步：`impl_plan.md` R5 L137 | 已销账 |
| M-4（格式健壮性边界未钉住） | `bdd.md` 新增 S-CONTACTS-12 L380-384（remove 最后一条目 -> 仅剩 title H1 + 空行、validate 合法、再 remove 同键 not-found）与 S-CONTACTS-13 L386-390（含空格/括号路径的 update/remove 往返，键 = 未转义原串，二次操作仍命中）；tdd 同步：`tdd.md` §8.1 L223-224（core 用例含 angle-bracket 形态断言）+ §8.2 L241-242 | 已销账 |
| m-1（计数基准二选一） | 定案为「共 26 项」（--name 保留在负向清单内）：`bdd.md` L464 + `tdd.md` L256 两处统一，并写明修订前 25 + --new-profile = 26 | 已销账 |
| m-2（S-LOCK-01 brief 侧语料前置缺失） | `bdd.md` S-LOCK-01 L468-472 Given 补「预创建 N 个互不相同的 entry 目标文件」（并写明 brief add 快照依赖与 contacts 侧不校验的差异）；tdd 同步：`tdd.md` L248 | 已销账 |
| m-3（新动词未入 ASCII help 防线清单） | `tdd.md` §8.2 ASCII 契约扩展行 L250 + §8.3 第 5 条 L260：`all_help_output_is_pure_ascii` 动词清单追加 `contacts remove`、`contacts update` 两行 | 已销账 |
| m-4（brief read --entry-title JSON 字段面歧义） | 定案为命中即按 --full 档字段输出（Default/JSON 同口径，不受 --full 门控）：`spec.md` §3.5 L154；`bdd.md` S-BRIEF-07 L305；`tdd.md` L245；impl_plan 同步：`impl_plan.md` R4 L131 | 已销账 |
| m-5（research 对 bdd 的行号漂移） | `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md` L156：改为「落盘时点实测 L391；修订后以场景号 S-SHORT-02 为准」（采建议的节号引用免漂移）；另 L6 补全局时效声明 | 已销账 |
| m-6（update 文件不存在用例缺失 + OLD==NEW 判定顺序） | `tdd.md` §8.1 L219（补 contacts_update 文件不存在 -> NotFound 行）+ L221（OLD==NEW 行补判定顺序注：OLD 命中检查先于 NEW 已存在检查，OLD==NEW 且 OLD 未命中落入 NotFound）；spec 同步：`spec.md` L171 | 已销账 |
| 已核查无发现维度（锁模板可行性/brief-profile 补锁影响面/update 语义可实现性主体） | 无需修复，rework 未触碰其认定面 | 维持 |

销账统计：本报告 10 条发现（4M+6m）全部销账，无挂起项。
