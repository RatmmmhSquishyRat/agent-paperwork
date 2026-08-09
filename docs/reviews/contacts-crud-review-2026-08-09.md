# Review Book — contacts CRUD 轮（contacts remove/update + brief read --entry-title + 写路径锁统一）· QA 全量实测

**Date**: 2026-08-09
**Scope**: contacts 完整 CRUD（新增 remove/update）、brief 选择性详情（read `--entry-title`）、六写路径锁统一的独立 QA 全量实测
**规格基线**: 主工作区 `docs/ssot/specs/cli-grammar-v0.6/spec.md` §3.5（brief read --entry-title）/§3.6（contacts 组）/§3.9（写路径锁语义）修订稿 + `docs/researches/contacts-crud-progressive-read-research-2026-08-09.md`（注：spec 修订稿为主工作区未提交工作副本，worktree 内 spec.md 仍为旧基线——见 §6 OBS-2）
**实测对象**: worktree `agent-paperwork-wt-v06grammar` 分支 `cli-grammar-v0.6`，HEAD `e7eb049`，`cargo build --release` 现构建产物；`paperwork -V` 仍为 `paperwork 0.5.0`（未 bump、无发布动作，合规）
**实测环境**: Windows 25H2 / PowerShell 7 / zh-CN locale；实测目录为系统 TEMP 下独立目录 `%TEMP%\pwqa-contacts-20260809`（与仓库零交叠），样本 profile（alice/bob/carol/dave/erin）现场 `profile create` 生成；全部输出逐字记录
**Previous review**: [v0.6-review-2026-08-09.md](./v0.6-review-2026-08-09.md)（§8 已给出「可供 owner 验收的稳定候选」判定，本轮为其后的 contacts CRUD 增量轮）
**角色约束**: 独立 QA——只做构建、运行与临时目录实测，不改任何代码/既有文档；本轮不发布（spec §7.4 延续），判定口径为「可发布候选 / 可验收稳定候选 / 需修复」三选一。

**本轮提交面清点**（基线 `0f6c384` 之后 6 提交）：

| 提交 | 主题 | 变更面 |
|---|---|---|
| `77ab558` | feat(core) | 抽取锁内读改写 helper（thread_edit 六步模板） |
| `cbb3790` | feat(core) | `contacts_remove`/`contacts_update` 新增 + contacts add 写路径补锁 |
| `595f9b2` | feat(core) | brief add/remove、profile edit 写路径补锁 |
| `fdbcbab` | feat(cli) | contacts remove/update 动词接线 + brief read `--entry-title` 过滤 |
| `dfce66c` | test(cli) | 集成套件 +625：contacts CRUD、brief --entry-title、白名单扩容、锁场景 |
| `e7eb049` | docs | README/SKILL 补 contacts remove/update 与 brief read --entry-title 示例 |

---

## 1. 概览与环境

| 项 | 值 |
|---|---|
| 实测项总数 | **104**（CRUD 22 + brief 选择性详情 9 + 锁并发 10 轮 + 边界 21 + 回归 20 + 协议 7 + ASCII 字节级 8 + `-q` 5 + 门禁/版本 2） |
| 通过 | 101 |
| 发现 | BUG ×1（minor）+ 观察 ×2（非缺陷登记）；BUG-1 由 3 条边界探针（E18/E19/E21）触发 |
| 门禁 | `cargo test --workspace` 270 全绿（CLI 集成 129 + core 75 + ops_contacts_crud 15 + ops 51） |
| 方法 | release 二进制直跑；并发用 `Start-Process` 独立进程 + 集合比较断言；ASCII 用字节级 ReadAllBytes 扫描；中文路径探针用 ProcessStartInfo.ArgumentList + UTF8 编码逐字节复核 |

实测范围与 owner 指令逐条对应：contacts CRUD 全路径（create 默认/自定义 title、add 正/幂等/缺 flag、read、remove 四形态、update 五形态）、brief `--entry-title` 五态、锁行为（contacts add ×5、brief add ×5、update×remove 交叉、profile edit ×5）、回归面冒烟、输出协议、边界与怪输入。全部真实执行，无推演项。

---

## 2. 实测项逐条

### 2.1 contacts CRUD 全路径（C01–C22）

| 用例 | 命令形态 | 期望 | 实际（逐字摘要） | exit | 判定 |
|---|---|---|---|---|---|
| C01 | `contacts create team.contacts.md` | exit 0，默认 title `Contacts` | `ok contacts.create team.contacts.md` / `title: Contacts` | 0 | PASS |
| C02 | `contacts create crew.contacts.md --title "Crew Roster"` | 自定义 title 生效 | `title: Crew Roster` | 0 | PASS |
| C03 | `contacts add team.contacts.md --profile alice.profile.md` | exit 0，箭头 conclusion | `ok contacts.add alice.profile.md -> team.contacts.md`；字段 `contacts`/`profile` | 0 | PASS |
| C04 | 同 C03 重复（幂等） | exit 0，幂等语义冻结，无重复条目 | 与 C03 同型 ok 信封；后续 read 无重复 | 0 | PASS |
| C05 | `contacts add team.contacts.md`（缺 `--profile`） | usage exit 2 + 规范示例 | `error usage: the following required arguments were not provided: --profile <PROFILE>` + `example: paperwork contacts add team.contacts.md --profile agents/alice.profile.md` | 2 | PASS |
| C06/C07 | add bob / carol | exit 0 | `ok contacts.add bob.profile.md -> ...` 等 | 0 | PASS |
| C08 | `contacts read team.contacts.md` | 富化输出 `<path>: <name> (<desc>)` | `ok contacts.read 3 contacts` + 三行富化（alice (team lead) 等） | 0 | PASS |
| C09 | `contacts remove team.contacts.md --profile bob.profile.md`（命中） | exit 0，信封含 `contacts`/`removed` | `ok contacts.remove bob.profile.md -> team.contacts.md` / `removed: bob.profile.md` | 0 | PASS |
| C10 | 同键再删（未命中） | not-found exit 1，fix 含键口径教学 | `error not-found: Contacts entry 'bob.profile.md' not found`；fix 逐字含 `the key is the profile path as stored in the contacts file, not the label` + `paperwork contacts read` 引导 | 1 | PASS |
| C11 | `contacts remove team.contacts.md`（缺 flag） | usage exit 2，规范示例逐字 | `example: paperwork contacts remove team.contacts.md --profile alice.profile.md`（与 spec §3.6 Ryan m-2 逐字一致） | 2 | PASS |
| C12 | `remove --profile alice`（把 label 当键，常见 agent 错误） | not-found exit 1，fix 点破键口径 | `error not-found: Contacts entry 'alice' not found` + 键口径教学句（点破成立） | 1 | PASS |
| C13 | remove 后 read + 打开文件 | 条目消失、文件为链接 bullet 格式 | read 2 contacts；文件 `# Contacts` + `- [alice](alice.profile.md)` 等 | 0 | PASS |
| C14 | `contacts update team.contacts.md --profile alice.profile.md --new-profile dave.profile.md`（换绑成功） | exit 0，`updated` 箭头串 | `ok contacts.update alice.profile.md -> dave.profile.md`；`updated: alice.profile.md -> dave.profile.md`（spec §3.6 Ryan m-4 定案的 `<OLD> -> <NEW>` 三段单空格形态逐字符合） | 0 | PASS |
| C15 | update OLD 未命中（bob 已删） | not-found exit 1 + 键口径教学 | `error not-found: Contacts entry 'bob.profile.md' not found` + 同 remove 教学句 | 1 | PASS |
| C16 | update NEW 已存在（carol 在册） | already-exists exit 1，fix 引导先 remove | `error already-exists: Contacts entry 'carol.profile.md' already exists` / `fix: remove the existing entry first or use a different profile path` / example 为可执行 remove | 1 | PASS |
| C17 | update 缺 `--new-profile` | usage exit 2，规范示例逐字 | `example: paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md`（逐字符合） | 2 | PASS |
| C18 | update 缺 `--profile` | usage exit 2 | 同 C17 规范示例 | 2 | PASS |
| C19 | update NEW 指向不存在 profile（`ghost.profile.md`） | **spec §3.6 契约：exit 0 静默写入，label 依 R11 回退文件名主干** | exit 0；`ok contacts.update dave.profile.md -> ghost.profile.md`；落盘 `- [ghost](ghost.profile.md)`（label=文件名主干，R11 回退成立） | 0 | PASS |
| C20 | update NEW==OLD 且在册（ghost→ghost） | 判定顺序 OLD 先命中 → already-exists | `error already-exists: Contacts entry 'ghost.profile.md' already exists`（判定顺序符合 spec：OLD 命中检查先于 NEW） | 1 | PASS |
| C21 | update NEW==OLD 且不在册 | OLD 未命中先行 → not-found | `error not-found: Contacts entry 'nobody.profile.md' not found`（顺序契约成立） | 1 | PASS |
| C22 | C19 后 read | 富化输出对不可读 profile 的形态 | `ghost.profile.md: (unreadable)`（name/desc 富化失败回退形态，exit 0 不阻断） | 0 | PASS |

另核验（落盘顺序）：C14/C19 换绑后条目保持原位（ghost 恒在第 1 位、carol 恒第 2 位）——「条目顺序保留」契约成立。

### 2.2 brief 选择性详情 `--entry-title`（B01–B09）

| 用例 | 命令形态 | 期望 | 实际（逐字摘要） | exit | 判定 |
|---|---|---|---|---|---|
| B01 | `brief create ob.brief.md --title Onboarding --owner alice --description "qa brief"` | exit 0 | `ok brief.create ob.brief.md` | 0 | PASS |
| B02/B03 | `brief add --entry alice.profile.md --note ...` / `--entry bob.profile.md --regex name --note ...` | exit 0 | `ok brief.add <entry> -> ob.brief.md` ×2 | 0 | PASS |
| B04 | `brief read ob.brief.md`（默认 TOC，冻结形态） | TOC：conclusion `2 entries` + 名单无详情 | `ok brief.read 2 entries` / title/owner + 两行纯名单（TOC 冻结未被新 flag 扰动） | 0 | PASS |
| B05 | `brief read --entry-title alice.profile.md`（命中，plain 档） | 按 `--full` 档字段输出该单条目（Daniel m-4 定案：不受 --full 门控） | conclusion 仍 `2 entries`（总数形态冻结）；正文仅 alice 一行，含 `hash: edb47a...13846`（64 位）+ `note: lead profile`——full 档字段面成立 | 0 | PASS |
| B06 | B05 + `--json` | JSON entries 命中条目含 path/hash/regex/note 同口径 | `{"command":"brief.read","conclusion":"2 entries","entries":[{"hash":"edb47a...","note":"lead profile","path":"alice.profile.md","title":"alice.profile.md"}],...}`——JSON 与 default 同口径成立 | 0 | PASS |
| B07 | `--entry-title nope.md`（未命中） | not-found exit 1，fix 引导列出条目 | `error not-found: Brief entry 'nope.md' not found` / `fix: run \`paperwork brief read ob.brief.md\` to list entries` | 1 | PASS |
| B08 | `--entry-title bob.profile.md --full`（组合） | 合法、等价单条目详情 | bob 一行含 `hash` + `regex: name` + `note: reviewer`（regex 字段面在命中条目上齐备） | 0 | PASS |
| B09 | `--entry-title ""`（空值边界） | 无匹配 → not-found | `error not-found: Brief entry '' not found` + 同 B07 fix | 1 | PASS |

### 2.3 锁行为并发（10 轮，全部独立进程）

方法：`Start-Process` 拉起 N 个 `paperwork.exe` 独立进程，等待全部退出后以集合比较断言并集/无丢失（v0.6 BUG-5 教训：不做顺序配对断言）。

| 轮次 | 配置 | 结果 | 判定 |
|---|---|---|---|
| L-r1 | contacts add：5 进程各 add 不同 profile 至新 contacts | exit 全 0；read=5 contacts；文件 5 条 bullet 恰为 {alice,bob,carol,dave,erin}.profile.md 并集，无丢失无重复 | PASS |
| L-idem | contacts add：5 进程并发 add **同一** profile（幂等竞态） | exit 全 0；read=1 contacts，恰 1 条——幂等语义在并发下保持 | PASS |
| L-brief | brief add：5 进程各 add 不同 entry 至新 brief | exit 全 0；`brief read --full` 5 条目齐，hash/note 各自正确，无丢失 | PASS |
| L-xr1/2/3 | update×remove 交叉：预置 [alice,bob,carol]，4 进程并发 = update alice→dave + remove bob + update carol→erin + remove bob（重复删） | 3 轮 exit 恒为 `0,0,0,1`（重复 remove 恰一条落 not-found，信号如实）；终态 3/3 轮恒为 {dave, erin}，bob 恒缺席，文件无损坏 | PASS |
| L-r2/r3 | contacts add 5 进程并发 ×2 复轮 | 2/2 轮全 exit 0、5 条并集无丢失 | PASS |
| L-mixed | 混合：5 写（add）+ 5 读（contacts read）+ 3 读（brief read --full）共 13 进程 | 13/13 全 exit 0；终态 5 条齐——锁等待下读者不失败（BUG-2 型 os error 33 未在补锁路径复现） | PASS |
| L-pe | profile edit：5 进程并发改同一 profile description | 5/5 全 exit 0；终态文件合法（validate 通过），description 为最后持锁者值（last-writer-wins，锁内读改写语义） | PASS |

**结论**：六写路径补锁后，并发不变量（并集无丢失、无损坏、幂等保持、重复操作如实报错）在 10 轮共 51 个真实进程下全部成立；锁等待语义（阻塞至持锁者完成）实测可观察且最终一致。

### 2.4 边界与怪输入（E01–E21）

| 用例 | 形态 | 实际（逐字摘要） | exit | 判定 |
|---|---|---|---|---|
| E01 | `profile create "dir with space\张 三.profile.md" --name zhangsan` | exit 0，文件建成 | 0 | PASS |
| E02 | `contacts create "dir with space\名 单.contacts.md" --title "Space Team"` | exit 0 | 0 | PASS |
| E03 | `contacts add` 中文+空格路径 profile 入中文+空格路径 contacts | exit 0，双向路径原值落盘 | 0 | PASS |
| E04 | `contacts read` 该中文 contacts | 功能正确（1 contacts、zhangsan 富化）；**回显路径含非 ASCII 字节**——字节级复核 6 个非 ASCII 字节（见 §6 OBS-1，定性为用户数据回显，非缺陷） | 0 | PASS（附观察） |
| E05 | `contacts add --profile ""`（空字符串） | **exit 0 静默接受**，落盘退化 bullet `- []()`——见 §3 BUG-1 | 0 | **FAIL** |
| E06 | `contacts add --profile missing.profile.md`（不可读 profile） | exit 0，label 依 R11 回退文件名主干（与 spec §3.6 rework 更正的「纯静默回退」契约一致，现状冻结面） | 0 | PASS |
| E07 | add 后 read | `(unreadable)` 富化回退形态如实 | 0 | PASS |
| E08–E10 | 依次 remove bob / 读剩 1 条 / remove missing（不可读但在册条目可删） | 键口径与在册性解耦：不可读 profile 的条目照常可删（键=存储路径） | 0 | PASS |
| E11 | 删至 1 条后 `validate --type contacts` | `ok validate team.contacts.md` | 0 | PASS |
| E12 | `brief read ghost.brief.md --entry-title x.md`（brief 文件不存在） | 文件级 not-found 先行：`error not-found: Brief 'ghost.brief.md' not found` + create 教学 example | 1 | PASS |
| E13 | `contacts update ghost-contacts ...`（contacts 文件不存在） | `error not-found: Contacts 'ghost-contacts.contacts.md' not found`（ensure_suffix 补后缀如实回显）+ create 教学 | 1 | PASS |
| E14 | `contacts create team.contacts.md`（重复 create） | already-exists exit 1，fix 引导 `contacts add` | 1 | PASS |
| E15 | `contacts remove --profile ""` | `error not-found: Contacts entry '' not found`（空键一致走 not-found，与 BUG-1 的 add 面不对称，见 §3） | 1 | PASS |
| E16 | `contacts update --profile "" --new-profile x.md` | 同 E15：not-found，空键口径一致 | 1 | PASS |
| E17–E21 | BUG-1 复现组（兼空名单输出形态探针）：E17 新建 empty.contacts.md → E18 add "" → raw 文件含 `- []()` → E19 read 报 **0 contacts**（看不见该条目，无正文行——即 remove 全部条目后的同型空名单输出形态；raw 仅剩 `# Contacts` 加退化 bullet）→ E20 再 add alice → E21 read 报 1 contacts，raw 文件中 `- []()` **被静默清除** | 三态（信封 ok / 文件有条目 / read 视图 0 条）不一致；下次写入静默丢条目 | 0 | **FAIL（BUG-1）** |

### 2.5 `-q` 与新动词组合（Q01–Q05）

| 用例 | 形态 | 实际 | exit | 判定 |
|---|---|---|---|---|
| Q01 | `contacts create q.contacts.md`（无 -q 基线对照） | ok 首行 + 字段齐 | 0 | PASS |
| Q02 | `contacts add ... --profile alice.profile.md`（无 -q 基线对照） | ok 首行 + `contacts`/`profile` 字段 | 0 | PASS |
| Q03 | `contacts update ... -q` | 仅隐 `ok` 首行，`contacts`/`updated` 字段保留 | 0 | PASS |
| Q04 | `contacts remove ... -q` | 同上，`removed` 保留 | 0 | PASS |
| Q05 | `brief read --entry-title ... -q` | 隐 ok 首行，条目详情正文保留 | 0 | PASS |

---

## 3. 发现清单

### BUG-1 · minor · `contacts add --profile ""` 静默接受：退化条目 `- []()` 写入后对 read 不可见，且被下次写入静默清除（信封/文件/读视图三态不一致）

**复现**（TEMP 独立目录，release 二进制）：

```
paperwork contacts create empty.contacts.md          # exit 0（E17）
paperwork contacts add empty.contacts.md --profile ""  # exit 0（E18，见下）
Get-Content empty.contacts.md                        # "# Contacts" + "- []()"
paperwork contacts read empty.contacts.md            # ok contacts.read 0 contacts（E19，看不见该条目）
paperwork contacts add empty.contacts.md --profile alice.profile.md  # E20
Get-Content empty.contacts.md                        # E21 后仅剩 alice 条目，"- []()" 被静默清除
```

**实际输出**（关键逐字）：

```
ok contacts.add  -> empty.contacts.md
contacts: empty.contacts.md
profile:
```

**期望**：空键拒绝（validation exit 1，与 post send 空 author/空正文的拒绝口径对齐），或至少 read 视图与文件一致。
**定性**：① 缺陷面在 `contacts_add`（冻结既有函数），**非本轮新引入**——v0.5 add 即同行为，本轮 QA 首曝；② 新增的 remove/update 对空键的处理反而一致（均 not-found exit 1，E15/E16），不对称仅在 add 入口；③ 实际触发条件苛刻（agent 显式传空串），无数据损坏（被清除的仅是退化条目本身），故定 minor。
**建议修复方向**（供 owner 裁决，QA 不改码）：`contacts_add` 入口对 trim 后为空的 profile 路径落 validation exit 1（一行级护栏），与既有「不可读路径静默回退」判例不冲突——空串不是「不可读的路径」，是「无路径」。

### OBS-1 · 观察（非缺陷）· 用户路径含非 ASCII 字符时输出回显非 ASCII 字节

`contacts read`/`add` 对含中文的路径原值回显（字节级实测 6 个非 ASCII 字节）。与先例一致：`--plain` 输出文件本体、post read 回显用户正文均含用户数据——**纯 ASCII 契约的适用域应解释为 CLI 自有文本**（help/信封结构/fix/example），用户数据回显不在约束内。本轮新命令的 CLI 自有文本面（4 个 help + 3 类错误信封）字节级扫描 0 非 ASCII（§5）。建议在 spec 冻结条款中把该边界写明，避免后续轮次误判。

### OBS-2 · 观察（文档面）· 本轮规格修订稿未提交且不在实现 worktree 内

实测依据的 spec §3.5/§3.6/§3.9 修订稿为主工作区**未提交**工作副本；worktree（实现所在分支 `cli-grammar-v0.6`）内 `spec.md` 仍为旧基线（§3.6 仅 create/add/read、无 §3.9）。行为面实测与修订稿逐条吻合（含 Ryan m-2/m-3/m-4 逐字契约），故不影响本轮行为判定；但「实现先行、规格未落盘提交」与 SSOT 纪律存在张力，建议规格修订稿随实现同批提交进分支。

---

## 4. 回归确认（R01–R20）

| 域 | 用例 | 结果 |
|---|---|---|
| post 冒烟 | R01 send / R02 read（showing+window 恒显）/ R03 summary / R04 edit | 4/4 PASS |
| profile 冒烟 | R05 show / R06 edit（**新补锁路径**，exit 0 + `changed: description`）/ R07 list（5 profiles） | 3/3 PASS |
| brief 冒烟 | R08 remove --entry-title / R09 verify（`1/1 fresh`） | 2/2 PASS |
| validate | R10 `--type contacts` / R11 post 后缀推断 | 2/2 PASS |
| 旧文法 usage 教学 | R12 `post send smoke.post.md alice "Hi"` → usage exit 2 + v0.6 规范示例；R13 `contacts add <PATH> <裸token>`（旧位置文法）→ usage exit 2 + `--profile` 形态示例 | 2/2 PASS |
| 短形式 | R14 `-a`/`-m` 等价生效；R15 `-q` 于新动词 add 生效（隐 ok 首行、字段保留） | 2/2 PASS |
| 别名 | R16 隐藏别名 `c read` 等价 `contacts read` | 1/1 PASS |
| 新动词×既有行为交叉 | R17 remove 不可读条目（ghost）成功 / R18–R20 连续 remove 后 read 计数与富化如实 | 4/4 PASS |
| 门禁 | `cargo test --workspace` 270 全绿（CLI 集成 129 + core 75 + ops_contacts_crud 15 + ops 51） | PASS |
| 版本纪律 | `paperwork -V` = 0.5.0，未 bump、无 tag | PASS |

**回归结论：20/20 通过 + 门禁全绿，本轮未引入既有行为回归。**

---

## 5. 输出协议核对

### 5.1 JSON key 面抽查（P01–P07，只增不改不删）

| 用例 | key 面 | 判定 |
|---|---|---|
| P01 create --json | status/command/conclusion/path/title（既有形态） | PASS |
| P02 add --json | 新增 `contacts`/`profile`；conclusion 箭头串 | PASS |
| P03 **update --json** | 新增 `contacts` + `updated: "alice.profile.md -> bob.profile.md"`（箭头串与 conclusion 同构，Ryan m-4 定案形态） | PASS |
| P04 remove --json | 新增 `contacts` + `removed` | PASS |
| P05 read --json（空名单） | `contacts: []` + conclusion `0 contacts` | PASS |
| P06 update 缺两必填 --json | usage 信封 JSON：category/exit_code=2/fix/example/message 齐 | PASS |
| P07 remove 目标文件不存在 --json | 运行时信封 JSON：category=not-found/exit_code=1/fix/example 齐 | PASS |

新 command id `contacts.remove`/`contacts.update` 出现且既有 id 零变更；JSON 全为增项。

### 5.2 退出码矩阵

| 形态 | category | exit | 实测 |
|---|---|---|---|
| 全部 ok 路径 | — | 0 | 命中（C/B/L/R/Q 全组） |
| not-found（remove/update 未命中、brief entry 未命中、目标文件不存在） | not-found | 1 | 命中（C10/C12/C15/B07/B09/E12/E13/E15/E16） |
| already-exists（NEW 在册、NEW==OLD 在册、重复 create） | already-exists | 1 | 命中（C16/C20/E14） |
| 缺必填 flag / 旧文法裸 token | usage | 2 | 命中（C05/C11/C17/C18/R12/R13） |

### 5.3 纯 ASCII 字节级抽查（8 探针）

新命令 CLI 自有文本面：`contacts remove --help` / `contacts update --help` / `contacts --help` / `brief read --help` 4 个 help + 3 类错误信封（缺 flag usage、not-found、update usage），逐一 ReadAllBytes 扫描——**0 非 ASCII 字节**。用户数据回显面（中文路径）单列 OBS-1。

---

## 6. 遗留与建议

| 编号 | 事项 | 建议 |
|---|---|---|
| BUG-1 | add 空键静默面 | owner 裁决是否一行级 validation 护栏；不阻塞验收 |
| OBS-1 | ASCII 契约适用域（用户数据回显） | 下轮 spec 修订写明边界 |
| OBS-2 | spec 修订稿未随实现提交 | 规格文档与实现同批提交进分支 |
| 登记备查 | io 信封 zh-CN OS 文案（v0.6 轮遗留，本轮未触及该路径） | 维持 v0.6 §8 处置：非阻塞，后续轮次 |
| 未覆盖 | Linux flock 下的锁语义、阻塞超时极端场景（持锁者挂起） | 依赖 CI Linux 面与 spec §3.9 阻塞行为契约声明；本轮 Windows 实测已覆盖正常阻塞-等待-释放全链 |

---

## 7. 最终判定

**判定：可验收稳定候选（非可发布候选——本轮延续不发布约束，spec §7.4）。**

依据：
1. **契约面**：spec §3.5/§3.6/§3.9 修订稿的逐条契约（含逐字项：usage 规范示例、键口径教学句、`updated` 箭头串格式、NEW 不存在 exit 0 + R11 label 回退、OLD 先于 NEW 的判定顺序）全部实测命中；brief `--entry-title` 三档渐进阅读（TOC/单条详情/全量）口径一致（plain 与 JSON 同字段面）。
2. **并发面**：10 轮 51 个真实进程，补锁六路径并集无丢失、无损坏、幂等保持、锁等待最终一致；v0.6 BUG-2 型失败未在任何补锁路径复现。
3. **回归面**：20/20 冒烟 + 270 测试全绿 + 版本纪律合规，零回归。
4. **发现面**：仅 BUG-1（minor，空键静默，既有函数缺陷、触发条件苛刻、无数据损坏）+ 2 项观察登记——均不构成验收阻塞项。BUG-1 建议在发布前批次销账（一行级护栏）。

**实测统计**：共 104 项（CRUD 22 + brief 选择性详情 9 + 锁并发 10 轮 + 边界 21 + `-q` 5 + 回归 20 + JSON 协议 7 + ASCII 字节级 8 + 门禁/版本 2），通过 101，3 项触发 BUG-1（minor）。发现分级：**Critical ×0、Major ×0、minor ×1**，另有非缺陷观察 ×2。

---

*End of QA review book.*
