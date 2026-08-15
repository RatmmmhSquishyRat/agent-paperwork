# 深审 C：SSOT 一致性与 agent 可用性走查报告

- 日期：2026-08-15
- 任务：#26（调研+实测，零源码变更、零 git 提交，本报告为唯一落盘产物）
- 取证基线：master @ 3829fd9（刚合入 cli-grammar-v0.6 三方合并；v0.6 具名文法 + contacts CRUD + 六写路径锁统一已在 master 生效）；paperwork 0.5.0 未发布（crates.io 现行 0.5.0 = tag v0.5.0 = 70f7e43，v0.5 位置文法 + 旧格式）
- 方法：release 构建实测（target/release/paperwork.exe）全部 help 面逐命令采集；SKILL.md / 根 README / repos/paperwork-cli/README.md 逐行核对；cli-grammar-v0.6 四件套互引核对；backlog 与 open-items-ledger 交叉验证（不重复裁定）；agent 视角盲测全流程 24 步实测；io 信封字节级取证
- 严重度口径：文档缺陷最高「重要」；仅当误导 agent 造成数据风险才「阻塞」（本轮无阻塞项）

---

## 一、发现清单（摘要表）

| 编号 | 严重度 | 一句话 |
|---|---|---|
| S-01 | 重要 | README 引导 cargo install 的 crates.io 0.5.0 与全文档命令面（v0.6 具名文法 + v2 格式）不一致，无警示 |
| S-02 | 重要 | 无「输出恒 UTF-8、消费端须 UTF-8 解码」声明；README「All output is pure ASCII」与实测矛盾（io message 嵌本地化文本） |
| S-03 | 重要 | 锁阻塞行为（无超时、编排层应加进程级超时重试）未在 SKILL/README 披露 |
| S-04 | 低 | 根 README Commands 缺 post edit 与 brief 条目删除动词示例 |
| S-05 | 低 | README usage 信封示例 --author <AUTHOR> 后多省略号，实测无 |
| S-06 | 低 | README contacts read 注释 "shows name + description" 漏掉实际输出的主列 path |
| S-07 | 低 | 裸 .md 路径被替换为类型后缀的解析分支未文档化；cli README 快速示例使用该形态 |
| S-08 | 低 | cli-ux-redesign 套件无反向 superseded-by 指针，治理状态陈旧（仍称「待开工」） |
| S-09 | 低 | cli-grammar-v0.6 README 治理清单未勾选项已过期（实现已合入 master） |
| S-10 | 低 | bdd 两个空键/空值守卫场景在 tdd 无用例行映射（实现与测试均在场） |
| S-11 | 低 | bdd S-SHORT-02「共 26 项」计数口径含糊（枚举实为 24+2+1） |
| S-12 | 低 | ledger/backlog 的「待合并」口径随 3829fd9 合入而过期，裁定本身全部仍成立 |
| S-13 | 事实 | docs/dev/ 三份新文档未提交（含任务书未点名的第三份） |

---

## 二、发现明细

### S-01（重要）README 安装引导与文档命令面版本错配

- 位置：根 README.md L20-22（Quick start `cargo install paperwork-cli`）、L64-68（Install）；其后全部示例为 v0.6 具名文法
- 问题：crates.io 现行 paperwork-cli 0.5.0 对应 tag v0.5.0（70f7e43）。实测 `git show v0.5.0:repos/paperwork-cli/src/cmd/post.rs` 头部注释为 v0.5 位置文法（NAME 第 2 位置参数、content 末位、post create 仍在），且该 tag 不含 format-v2（61e1e89 非其祖先，merge-base 实测）。agent 按 README 安装后立即遭遇：全部具名 flag 示例落 usage exit 2；新旧文件格式互不兼容（旧二进制写旧格式，新 validate 拒绝）。README/SKILL 均无「文档对应未发布 master」的警示。
- 佐证：static.crates.io 存在 paperwork-cli-0.5.0.crate（实测可下载）；spec §7 第 4 条与 v0.6_feedbacks §一(3) 的不发布约束在文档面未向消费者披露。
- 建议修法：README Install 节头部加醒目声明——crates.io 0.5.0 为 v0.5 位置文法+旧格式，本文档对应未发布的 master 形态，现阶段请走 from source 段落（`cargo install --path repos/paperwork-cli`）；发布轮统一消除。
- 不定「阻塞」理由：旧二进制的错误均显式暴露（usage/format 信封），无静默数据损坏路径。

### S-02（重要）输出协议声明缺失且 ASCII 承诺与实测矛盾

- 位置：根 README.md L58（"ASCII-only envelope"）、L143（"All output is pure ASCII — no color, no Unicode symbols"）；SKILL.md 全文无编码声明；spec.md §5 第 4 条（纯 ASCII 契约）
- 问题：io 根因报告（docs/dev/io-encoding-rootcause-2026-08-15.md §6）建议的文档声明「输出恒为 UTF-8，消费端须以 UTF-8 解码（PowerShell 先设 [Console]::OutputEncoding=UTF8）」在 SKILL/README 均未落地。且实测推翻绝对 ASCII 承诺：本机 zh-CN Windows 下 `validate C:\nonexistent-dir-zz\no.md --type post` 的 stderr 信封 204 字节中含 33 个非 ASCII 字节（message 字段内嵌中文 OS 错误文本；字节本身为合法 UTF-8，严格 UTF-8 解码通过）。
- 建议修法：(1) SKILL.md 与 README Output protocol 节各加一条：paperwork 全部输出（stdout/stderr，全部模式）恒为合法 UTF-8；信封结构面（status 行/category/fix/example）纯 ASCII，io 类 message 字段可能内嵌 OS 本地化文本；消费端必须按 UTF-8 解码，Windows PowerShell 会话捕获前先设 `[Console]::OutputEncoding = [Text.Encoding]::UTF8`。(2) README L143 的绝对表述按上述口径收窄。

### S-03（重要）锁行为未进入 agent 可见文档

- 位置：SKILL.md 全文、根 README.md 全文、repos/paperwork-cli/README.md 全文——均无锁相关描述
- 问题：spec §3.9「agent 可见阻塞行为契约」（六写路径 fs2 lock_exclusive 阻塞等待、无内建超时、持锁进程崩溃后 OS 自动释放、编排层应对 paperwork 进程施加进程级超时并杀进程重试、幂等/先读后写路径重试安全）是本轮合并的新可观察行为，但三份 agent 面向文档零披露。新 agent 编排层无从预期写命令可能长时间不返回。
- 建议修法：SKILL.md 增一小节（3-5 行）：写命令（send/edit/add/update/条目删除/profile edit）经文件锁串行化；正常毫秒级；等待无超时，编排侧可对进程加时限，超时杀进程重试安全（add 幂等，其余先读后写）。

### S-04（低）根 README Commands 节缺两个动词示例

- 位置：根 README.md L95-104（post 节仅 send/read/summary）、L108-116（brief 节仅 create/add/verify/read）
- 问题：`post edit`（含 --author/--seq/--message 三必填）与 `brief remove`（--entry-title 键语义）无示例；SKILL.md 均覆盖。agent 仅读 README 会漏掉两个动词（可由 --help 自发现，故列低）。
- 建议修法：各补一行示例（文例照抄 SKILL.md L69 / L87 即可）。

### S-05（低）README usage 信封示例与实测逐字不符

- 位置：根 README.md L167
- 问题：示例 `error usage: the following required arguments were not provided: --author <AUTHOR>...` 尾部多省略号；实测逐字为 `... --author <AUTHOR>`（PROBE4）。该节以逐字信封示人，应保持精确。
- 建议修法：删省略号。

### S-06（低）README contacts read 注释与输出形态不符

- 位置：根 README.md L125（`# shows name + description`）
- 问题：实测输出主列为 `<存储路径>: <name>`（description 存在时以括号追加），bdd S-CONTACTS-05 钉住 `<路径>: <name> (<description>)`；注释漏 path 主列，而 path 恰是 remove/update 的键来源（README L128 正文描述正确，仅注释失准）。
- 建议修法：注释改 `# shows stored path + name (+ description)`。

### S-07（低）裸 .md 路径替换解析未文档化

- 位置：SKILL.md 规则 4（L21-23）；repos/paperwork-cli/README.md L25-27 Quick Example（`./alice.md`、`./thread.md`）
- 问题：实测 `profile create ./alice2.md` 落盘 `alice2.profile.md`、`post send ./thread.md` 落盘 `thread.post.md`——ensure_suffix 的 suffixed_variant 对裸 `.md` 是「剥离再替换」而非追加（repos/paperwork-cli/src/cmd/mod.rs L48-57）。SKILL 规则 4 只讲裸名补后缀与既存文件优先，未讲 `.md` 替换分支；cli README 快速示例恰用该形态，行为虽正确但产物文件名与入参不同，依赖信封回显才不迷路（信封确实回显，无数据风险）。
- 建议修法：cli README 示例改裸名（`alice`/`thread`）；SKILL 规则 4 补半句「以裸 .md 结尾的路径会被替换为类型后缀」。

### S-08（低）cli-ux-redesign 套件缺反向 superseded-by 指针

- 位置：docs/ssot/specs/cli-ux-redesign/README.md 与 spec.md 头部
- 问题：v0.6 spec README 已正向声明取代关系（cli-grammar-v0.6 取代 cli-ux-redesign 的文法层），但旧套件自身无反向 Superseded-by 注记，README 状态栏仍停留在「治理文档定稿、实现已合入」之前的旧表述，新 agent 从旧目录进入会误判其为现行文法。对照 adr-v1.md 的双行 Superseded-by 注记做法（v0.5/v0.6 双指针，已实测在场），旧套件应同样补一行。
- 建议修法：cli-ux-redesign/README.md 顶部加一行 Superseded-by：文法层以 docs/ssot/specs/cli-grammar-v0.6/spec.md 为准，本套件保留为历史记录（不改写历史内容，依 adr-v1 不可变原则同款做法）。

### S-09（低）cli-grammar-v0.6 README 治理清单过期

- 位置：docs/ssot/specs/cli-grammar-v0.6/README.md 治理核对清单
- 问题：清单中实现/验证相关勾选项仍为未完成态，而实现已随 3829fd9 合入 master、验证与三维评审（任务 #19/#20）已完成。治理文档与实际进度脱节。
- 建议修法：按既成事实勾选；发布相关项保留未勾并注记「owner 裁定延后（v0.6_feedbacks §一(3)）」。

### S-10（低）bdd 两个守卫场景在 tdd 无映射

- 位置：cli-grammar-v0.6/bdd.md 的 S-BRIEF-10 与 S-CONTACTS-15；tdd.md 映射表
- 问题：两个空键/空值守卫场景（brief 空 entry-title、contacts 空键）在 bdd 定义并有实测行为支撑（实现与集成测试均在场，grep 实证），但 tdd 映射表无对应单测条目，四件套互引链出现两个断点。
- 建议修法：tdd 映射表补两行（指向现有集成测试或补最小单测）；或 bdd 场景注明「由集成测试覆盖」并双向引用。

### S-11（低）S-SHORT-02「共 26 项」计数口径含糊

- 位置：cli-grammar-v0.6/bdd.md S-SHORT-02
- 问题：断言文案称短形式/命名政策枚举「共 26 项」，实际枚举为 24 项跨命令断言 + 2 项补充 + 1 项总括，口径不清，未来增删短形式时容易对不上账。
- 建议修法：改为不写死总数（「以下枚举为准」）或注明分项口径。

### S-12（低）ledger/backlog「待合并」口径过期

- 位置：docs/dev/open-items-ledger-2026-08-15.md（基线 55c916a，合并前）；docs/researches/ux-open-items-backlog-2026-08-08.md 相关条目
- 问题：ledger 以 55c916a 为取证基线，多处「待合并到 master」表述随 3829fd9 合入而已成事实；po 别名、implicit-mention 等项 master 已在场（实测验证），「待合并」口径过期。交叉验证结论：两文档无重复裁定冲突，ledger 全部裁定本身仍然成立，仅时态/状态描述需刷新。
- 建议修法：任务 #27 修复波或文档闭合轮（任务 #30）统一把「待合并」改为「已随 3829fd9 合入」，不改裁定内容。

### S-13（事实记录）docs/dev/ 三份新文档未提交

- 位置：docs/dev/io-encoding-rootcause-2026-08-15.md、docs/dev/open-items-ledger-2026-08-15.md、docs/dev/audit-ssot-agentux-2026-08-15.md（本报告）
- 说明：git status 实测三份均为 untracked。任务书仅点名前两份且明确「不要提交」；第三份（本报告）为任务书点名落盘产物。本审计不提交任何文件，提交时机由编排层裁定。
---

## 三、一致性核验通过项（无发现）

| 核验项 | 方法 | 结论 |
|---|---|---|
| help 输出 vs spec §2 签名表 vs README | release 构建逐命令采集（顶层/5 组/全动词级）三角核对 | 逐条吻合，无陈旧文法残留 |
| 短形式集合 | 全部 -x 短形式枚举实测 | 精确 {-a,-m,-q}，与 spec/bdd S-SHORT-02 一致 |
| --message 与 --stdin 互斥 | 同传实测 | usage 信封 exit 2，与 spec §4 一致 |
| 退出码三档 | 正常/错误/用法错误三类路径实测 | 0/1/2 与冻结协议一致 |
| ok/error 信封结构面 | 逐字段比对 spec §5 | status 行/category/fix/example 结构一致 |
| 空键守卫 | grep 实现与集成测试 | 实现与测试均在场（仅 tdd 映射缺，见 S-10） |
| adr-v1.md Superseded-by 注记 | 读文件 | v0.5/v0.6 双行指针在场，历史正文未改写 |
| feedbacks 治理链 | 三份 feedbacks 通读 | v0.5→v0.6 翻转记录在场、依据表互链完整，无待回写结论 |
| backlog vs ledger 交叉 | 逐条比对裁定 | 无重复/冲突裁定（仅「待合并」时态过期，见 S-12） |
| contacts remove/update 键语义 | 以存储路径为键实测 | 与 bdd 断言一致；label 依 R11 派生；updated 箭头串格式一致 |
| SKILL.md 命令面 | 与 help/spec 逐条核对 | 全量如实（含 contacts remove/update、--entry-title、--json/--quiet、短形式约定），无陈旧残留 |

---

## 四、agent 视角盲测记录（仅凭 SKILL.md，24 步全绿）

环境：target/blindtest/（gitignored）隔离语料区，target/release/paperwork.exe，全程零失败、零回退，每步一次通过。

| # | 步骤 | 命令要点 | 结果 |
|---|---|---|---|
| 1 | 建 profile（alice） | profile create agents/alice.profile.md | ok，落盘 agents/alice.profile.md |
| 2 | 建 profile（carol） | profile create agents/carol.profile.md | ok |
| 3 | 建 contacts | contacts create team.contacts.md | ok |
| 4 | contacts add ×2 | contacts add team.contacts.md <路径> --name | ok，两名条目在账 |
| 5 | contacts read | contacts read team.contacts.md | ok，<路径>: <name> 主列形态 |
| 6 | 首发建线程 | post send standup.post.md --title --author alice -m | ok，首 send 即建线程（无 post create） |
| 7 | 回复 | post send standup.post.md --author carol --reply-to 1 -m | ok，@#1 注入 |
| 8 | 隐式 mention | post send standup.post.md --author alice -m 正文含 @carol | ok，正文 token 直接生效 |
| 9 | post read 全量 | post read standup.post.md | ok，参与者读时重推导 |
| 10 | post read --json | 同上加 --json | ok，JSON 信封只增不改约束成立 |
| 11 | post edit | post edit standup.post.md --author carol --seq 2 -m | ok |
| 12 | 裸 .md 替换探针 | profile create ./alice2.md / post send ./thread.md | ok，落盘 alice2.profile.md / thread.post.md（S-07 证据） |
| 13 | 建 brief | brief create onboarding.brief.md | ok |
| 14 | brief add 条目 | brief add onboarding.brief.md --entry-title -m | ok |
| 15 | brief 选择性读 | brief read onboarding.brief.md --entry-title | ok，仅回目标条目 |
| 16 | brief verify | brief verify onboarding.brief.md | ok |
| 17 | contacts update | contacts update team.contacts.md <存储路径> --name | ok，updated 箭头串回显 |
| 18 | contacts remove | contacts remove team.contacts.md <存储路径> | ok |
| 19 | validate 全绿 | validate 各产物 --type | ok，exit 0 |
| 20 | validate 缺路径 | validate C:\\nonexistent-dir-zz\\no.md --type post | io 信封 exit 1（字节取证入第五节） |
| 21 | usage 探针 | post send 缺 --author | usage 信封 exit 2，文案逐字采集 |
| 22 | 互斥探针 | send 同传 --message 与 --stdin | usage 信封 exit 2 |
| 23 | --quiet 探针 | 写命令加 -q | ok 信封静默、退出码语义不变 |
| 24 | 锁行为观察 | 串行连发写命令 | 毫秒级返回，无异常（阻塞面见 S-03 披露缺口） |

盲测结论：新 agent 仅凭 SKILL.md 可一次走通全部核心流程（建 profile→建 contacts→发 post→读 post→建 brief→加条目→选择性读条目→contacts update/remove），含错误面与机器可读面；SKILL.md 作为唯一入口可用性成立。缺口不在命令面，而在 S-01/S-02/S-03 三项环境级披露。
---

## 五、UTF-8 输出协议声明专项检查

- 背景：io 信封中文乱码问题已由任务 #25 根因结案（docs/dev/io-encoding-rootcause-2026-08-15.md），其 §6 给出文档建议：声明输出恒 UTF-8、消费端按 UTF-8 解码。
- 字节级取证（PROBE12）：zh-CN Windows 下 paperwork validate C:\nonexistent-dir-zz\no.md --type post，stderr 信封落盘 io_err.bin（204 字节），其中非 ASCII 字节 33 个，全部位于 message 字段（内嵌中文 OS 错误文本）；整体按严格 UTF-8 解码通过——产品字节恒合法 UTF-8 的结论复核成立，乱码成因是 cp936 会话捕获侧的双重编码，非产品缺陷。
- 检查结论：
  1. 该文档建议在 SKILL.md 与根 README 均未落地（S-02）。
  2. README L143「All output is pure ASCII」与实测矛盾：信封结构面确为纯 ASCII，但 io 类 message 字段可内嵌 OS 本地化文本，绝对 ASCII 承诺属 locale 依赖的过度承诺，应按「结构面 ASCII + 字节恒合法 UTF-8」口径收窄。
  3. spec.md §5 第 4 条的纯 ASCII 契约同样需按上述口径修订（是否解冻措辞由任务 #27 裁定；行为面零变更，纯文档口径）。

---

## 六、修复建议优先级排序（供任务 #27 销账）

| 批次 | 编号 | 内容 | 理由 |
|---|---|---|---|
| P1（环境级披露） | S-01 | README 安装节加 crates.io 0.5.0 版本错配警示，引导 from source | 唯一可致新 agent 起步即系统性失败的缺口 |
| P1 | S-02 | SKILL/README 加 UTF-8 契约声明，收窄 ASCII 绝对表述 | io 根因报告既定建议未落地 |
| P1 | S-03 | SKILL 增锁行为小节（阻塞无超时、编排层进程级超时重试） | 新可观察行为零披露 |
| P2（文档精确性） | S-04/S-05/S-06/S-07 | README 示例补齐/勘误；SKILL 规则 4 补半句；cli README 示例改裸名 | 逐字精确性，改动小 |
| P3（治理刷新） | S-08/S-09/S-12 | Superseded-by 反向指针；v0.6 README 清单勾选；ledger「待合并」改「已合入」 | 状态口径，不改裁定 |
| P3 | S-10/S-11 | tdd 映射补两行；S-SHORT-02 计数口径 | 四件套互引闭合 |
| 仅记录 | S-13 | 三份未提交文档由编排层裁定提交时机 | 本审计不提交 |

无阻塞项；P1 三项完成后，新 agent 冷启动路径即无环境级误导。

---

报告完。深审 C（任务 #26）取证与结论至此闭合；13 项发现（3 重要 + 9 低 + 1 事实记录）移交任务 #27 修复波逐条销账。

---

更正注记（2026-08-15，owner 边界更正，仅追加不重写正文）：owner 从未指示发布 0.6；本报告 S-01 建议修法末句「发布轮统一消除」改读为「发布时机待 owner 指示，本工作流无发布计划，相关事项仅作事实登记」。S-01 的事实发现（README 安装引导与文档命令面版本错配）与其文档面修法（加警示、引导 from source）不受本更正影响。权威口径见 docs/dev/open-items-ledger-2026-08-15.md 第十二节。
