# CLI 文法 v0.6 文档对抗评审 —— 实现可行性与格式健壮性（视角：feasibility）

- 日期：2026-08-09
- 评审立场：批判性技术评审（挑错而非背书）；仅评审实现可行性与格式健壮性，不涉及 SSOT 程序与 UX 主观偏好
- 评审对象：docs/ssot/specs/cli-grammar-v0.6/ 全部六份（spec.md / design.md / bdd.md / tdd.md / impl_plan.md / README.md）
- 对照基准：cli-ux-v0.5 分支工作树（HEAD 70f7e43，worktree: agent-paperwork-wt-cliux）实际代码——repos/paperwork-cli/src/main.rs、cmd/*.rs、output.rs，repos/paperwork-core/src/ops/*.rs、tests/ops_tests.rs，repos/paperwork-cli/tests/cli_integration.rs，.github/workflows/ci.yml；v0.5 文档集 docs/ssot/specs/cli-ux-redesign/design.md（§2.6 与 F2/F5/F6/F7 裁定）；clap 4（derive，见 Cargo.toml/Cargo.lock）
- 方法：六份文档全文精读 + 工作树代码逐行核对 + 全量 grep 盘点（core example、cli_integration 调用点、ops_tests 断言面）+ clap 4 语义推演
- 核验前提声明：spec 声称的实施基线为「cli-ux-v0.5 + format-v2 工作树变更的合并结果」，但当前 worktree 尚未合并（post create 仍在、send 尚无 --title/--participants/--to）。post create 删除与建线程载荷相关声称在本工作树不可核验，以 impl_plan 步骤(0) 合并后盘点为准；本评审凡涉基线处以 cli-ux-v0.5 HEAD 实测为准并注明该前提。

---

## 一、事实层核实结果

| 核验项 | 结果 |
|---|---|
| tdd/impl_plan 行号引用 | v0.6 tdd 吸取 v0.5 轮教训，不预置行号清单（tdd §0：行号以合并后基线实测为准，实施第一步盘点），行号准确性风险被结构性规避；本文以全量 grep 替代核验，见下 |
| core example 盘点命令（impl_plan 步骤(0) `rg -n "paperwork (post\|brief\|contacts\|profile)" repos/paperwork-core/src`） | 当前基线实测输出 31 行；其中含命令文法的 example 字符串共 19 处：**需换 v0.6 文法 14 处**（thread.rs L138/L228/L275/L305/L326/L341 共 6 处、manifest.rs L32/L80/L105/L151/L194 共 5 处、contacts.rs L22 共 1 处、profile.rs L61/L91 共 2 处）+ 文法不变 5 处（thread.rs L288 post read、profile.rs L20 profile edit、manifest.rs L172 brief read、contacts.rs L56/L98 contacts create）。**「14 处」之数与当前基线改写集吻合**（与 v0.5 可行性评审 M-2 修正后的实测值一致），但盘点命令原始输出含 fix 文案与 String::new() 占位行，需过滤后才得 14（见 m-3） |
| core 层 fix 字段内嵌文法 | manifest.rs L79/L150/L193 三处 fix 文案 `run \`paperwork brief create {} "My Brief"\` first` 内嵌 v0.5 位置文法，v0.6 下过时（见 M-4） |
| ops_tests 零改动防线（tdd §5） | **成立**：ops_tests.rs 全文 grep `example`/`paperwork `/fix 片段均 0 命中，断言全部面向解析/锁/seq/hash 行为，core 14 处 example 文案改动不触碰任何断言 |
| cli_integration 调用点盘点命令（tdd §1 `rg -n "\"(send\|edit\|create\|add\|remove)\""`） | 当前基线实测命中约 84 行 .args 调用（含 post create 类与 --help 动词串，前者归 format-v2 合并处理）；tdd §1 按类清单覆盖改写目标形态，但遗漏若干断言语义翻转点（见 M-3/m-2） |
| v0.5 裁定引用有效性 | cli-ux-redesign/design.md §2.6（L140，静态规范示例）、F2（L332）、F5（L335）、F6（L336）、F7（L333）全部存在且被 v0.6 文档正确援引；分阶段门禁与 F6 裁定一致 |
| 短形式冲突核对（逐一） | 当前基线全 CLI 仅一个短形式：全局 `-q`（main.rs L37）。spec §4 计划短形式 -a/-m/-r/-p/-t/-l/-d/-o/-n 逐一核对：post send 内 -a/-m/-r/-p/-t 互异；post read 仅 -l；profile create/edit -m/-d 组内唯一；brief create -t/-o/-d、brief add -n、brief verify -d（不同动词）；contacts add -p；validate -t；与全局 -q 无冲突。**与既有 flag 短形式无冲突成立**；但表自身有两处问题（见 m-1/m-2） |
| usage 信封机制核对 | main.rs：try_parse + DisplayHelp/DisplayVersion 穿透 exit 0（F5）、argv 扫描感知 --json（take_while 止于 `--`）、顶层失败 command 填 "usage"、canonical_example 静态表（F2）——机制与 spec §5「机制冻结、仅换示例」声称一致；output.rs emit_err 硬编码 exit_code 1、emit_usage_error 硬编码 exit_code 2 且仅在 main.rs try_parse 失败分支调用（与 M-1 直接相关） |
| 输出协议冻结面 | error JSON command 字段走 argv 映射与运行时 command_id 双通道，新文法不改映射结构；usage 信封 message 取自 clap 渲染文本（含 `--message <MESSAGE>` 类字样，全 ASCII）；新示例文案全 ASCII；ascii_output_contract_guard 防线（cli_integration.rs L1257）保留且 tdd §4 明确扩展新 usage 形态。除 C-1/M-2 所涉外未见破坏点 |
| 遗漏波及面（聚焦 5） | ci.yml smoke：步骤(5)「逐条换 v0.6 文法」措辞覆盖充分（注意 L94/L170 空正文 smoke `post send standup alice ""` 若不如实改 `--message`，断言 category 将从 validation 翻转为 usage，属步骤(5) 字面覆盖内）；根 README / cli README / SKILL.md 由步骤(6) 覆盖（worktree 确有 SKILL.md）；Cargo.toml description 与 publish.ps1 实测不含任何命令文法，确认无关；隐藏别名 po/p/b/c/v 由 main.rs 别名表与 S-ALIAS-01/S-SHORT-02 覆盖。另见 M-3/M-4/m-4 的未覆盖点 |

---

## 二、Issue 清单

### Critical

**C-1 `--message` 值以 `-` 开头在 clap 默认行为下必然解析失败，文档缺 allow_hyphen_values 指令，spec/BDD 承诺不可兑现**

- 证据：spec §3.1 表格（`--message/-m` 行「flag 值直传，以 `-` 开头的正文无需 `--` 边界」）、spec §5 第 3 条、design §2.1（after_help 示例 `--message "-starts with dash is fine"`）、bdd S-SEND-10（exit 0、正文逐字为 `-fix flag text`、无 `--` 边界）、tdd §4 对应行。
- 分析：clap 4（derive）默认 `allow_hyphen_values = false`。对期望取值的 option，其后一个以 `-` 开头的 argv token 不会被消费为值，clap 在解析层报错（「a value is required for '--message <MESSAGE>' but none was supplied」类），落 usage exit 2——与 S-SEND-10 断言的 exit 0 **必然矛盾**。要兑现承诺，唯一实现路径是在 send/edit 的 `--message` Arg 上设置 `allow_hyphen_values(true)`。impl_plan 步骤(2) 逐条列出 required/conflicts_with 等 clap 指令，唯独漏掉这一条；design §6 论证互斥语义时同样未及。v0.5 时代该问题被 `--` 边界机制掩盖（位置参数在 `--` 后不受 flag 检测约束），v0.6 废止 `--` 教学后此属性成为硬前提。
- 影响：impl agent 照文档实现后 S-SEND-10 必红，且错误形态（usage exit 2）极易被误诊为「裸 -xxx 教学场景」（S-SEND-11），诱发反向错误修复（复活 `--` 教学或放宽断言），动摇本版本「`--` 边界需求废止」的核心卖点。
- 建议：spec §3.1 / design §6 / impl_plan 步骤(2) 明确「`--message`（send/edit 两处）设 `allow_hyphen_values = true`」；同时在 design §6 记录副作用边界（如 `--message --stdin` 连写时 `--stdin` 被吞为正文值，属显式输入、不另设护栏）；bdd/tdd 断言无需改动。

### Major

**M-1 「--message 与 --stdin 皆缺」的报告层级归属错误，与冻结机制自相矛盾**

- 证据：design §6「两者皆缺时由**命令层**报缺必填（usage exit 2）」；impl_plan 步骤(2)「正文通道缺省判定：--message 与 --stdin 皆缺时由**命令层**报缺必填（落 usage 信封...）」。对照：spec §3 开头「clap 层用法错误（缺必填 flag...）一律 usage exit 2」（同文档集内互相矛盾）；output.rs L99-126 emit_err 硬编码 `exit_code = 1`；main.rs L134-150 运行时错误一律 exit 1，usage 信封（exit 2）仅产生于 L68-96 try_parse 失败分支。
- 分析：在「usage 信封机制冻结」（spec §5：机制沿用 v0.5 spec §4.3，仅示例文案更新）前提下，命令层**没有任何既有管道**能产出 usage category + exit 2 的信封——命令层抛 PaperworkError 走 emit_err（exit 1），照字面实现只能得到 validation/io exit 1，违反 bdd S-SEND-06/S-EDIT-04（exit 2）；若为兑现而新增命令层 usage 管道，则违反 spec §5 机制冻结与 spec §7「破坏面仅限命令参数文法」。正确且唯一与冻结机制相容的实现是 clap 层 `required_unless_present = "stdin"`（解析层报 MissingRequiredArgument，自然走 try_parse 失败分支落 usage 信封），此时报告主体是 clap 层而非命令层。该组合（required_unless_present + conflicts_with）经推演成立：两者同给 -> ArgumentConflict exit 2；皆缺 -> MissingRequiredArgument exit 2；单独任一 -> 通过。
- 建议：design §6 与 impl_plan 步骤(2) 将「由命令层报缺必填」改为「由 clap 层 `required_unless_present` 判定缺必填（与 `conflicts_with` 组合，四种形态：同给/皆缺/仅 message/仅 stdin）」，与 spec §3 开头口径对齐。

**M-2 BDD「example 展示二选一形态」断言与 F2/F7 静态规范示例裁定冲突，不可同时满足**

- 证据：bdd S-SEND-06「example 展示 `--message` 与 `--stdin` 二选一的完整形态」、S-SEND-07「example 为二选一规范形态」、S-EDIT-04/S-EDIT-05 同；design §2.1 错误指导样貌（「缺正文通道...example 展示二选一形态」）。对照：v0.5 design §2.6 F2 裁定——usage 信封只输出「规范 usage 行 + **一条**预置可执行示例」，不携带用户原参数值；F7——example 一律具体可复制执行、禁用占位符；v0.6 spec §5 第 2 条自己把 post send 规范示例定为单一形态 `paperwork post send standup.post.md --author alice --message "Hello"`。
- 分析：单一可执行示例无法同时表达「二选一」（写成 `(--message <BODY> | --stdin)` 即违反 F7 禁占位符且不可复制执行；写两条示例即违反 F2「一条」）。spec §5 第 2 条与 bdd 四处断言直接抵触。tdd §4 对应行（「example 展示二选一完整形态」）照抄了该矛盾。
- 建议：bdd 四处断言降级为「example 为含 `--message` 通道（二选一之一）的规范可执行形态；二选一指引由 fix/message 文案承担」，design §2.1、tdd §4 同步措辞。这与 v0.5 轮 F2 的降级手法（「断言降级为含规范形态示例」）同构，不引入新机制。

**M-3 tdd 盘点遗漏「断言语义翻转」调用点；v0.4 旧 flag 迁移链对再合法化 flag 失效且教学文案误导**

- 证据（均在 cli_integration.rs）：
  - L457 `usage_old_grammar_profile_create_name`：触发样例 `profile create <path> --name alice`。v0.6 下 `--name` 是合法必填 flag（spec §2），该调用**翻转为 exit 0 成功**，现断言 `.code(2)` 必红，且无法「只改参数层」修复——它是 v0.4 旧 flag 教学用例，触发器本身被 v0.6 再合法化。
  - L501 `usage_old_grammar_post_edit_seq`：触发样例含 `--seq 1`，v0.6 下 `--seq` 合法，用例仅靠后续未知 `--from` 维持 usage exit 2，语义已与用例名/注释（「old-grammar --seq」）脱节。
  - L1224 `flag_inventory_matches_spec` 负向断言 `!profile_create_help.contains("--name")`：v0.6 下必须翻转为正向；L1244-1249 的 post create --help 断言块随 format-v2 删除整体失效。
  - L987/L1019/L1295 三处 `--` 边界与「body + --stdin 同给」用例：v0.6 下断言语义翻转（`--` 形态废止、conflicts 从 validation exit 1 升 usage exit 2），tdd §1「只改参数层、不改断言语义」原则对它们不适用。
- 进一步波及：spec §8「v0.4 旧 flag（--from 作身份等）的迁移教学链自然延伸覆盖」对 `--name/--seq/--title/--entry/--profile` 过度声称——这些 flag 在 v0.6 重新合法，不再落入未知 flag 教学；main.rs L215-221 usage_fix 的 base 文案（「required values are positional (PATH first; NAME second for post send/edit)」）与旧 flag 清单（「pre-v0.5 grammar (--from/--seq/--title/--entry/...)」）在 v0.6 下全部失真（NAME 不再是位置参数；--seq/--title/--entry 已合法），而 impl_plan 步骤(3) 仅列「静态规范示例、Grammar 模板行、疑似 flag 残留 fix 文案」，未显式列入 usage_fix base 与旧 flag 清单两个重写点。
- 建议：tdd §1 增「语义翻转调用点处置表」，逐一给出 L457（改触发器为 v0.5 位置文法形态或删除并入 S-PROF-03）、L501、L987/L1019/L1295（删除或由 §4 新用例取代）、L1224/L1244（断言翻转/删除）的处置；impl_plan 步骤(3) 显式列入 usage_fix base 与旧 flag 教学清单的重写；spec §8 将迁移链表述修正为「仍为 v0.6 无效 flag（--from 作身份等）的教学链自然延伸」。

**M-4 impl_plan 步骤(1) 范围仅「example 字符串」，遗漏 core fix 字段内嵌的 v0.5 文法**

- 证据：manifest.rs L79/L150/L193 三处 `fix: format!("run \`paperwork brief create {} \"My Brief\"\` first", ...)`——brief create 的 TITLE 在 v0.6 改为 `--title` 必填 flag，这三处 fix 文案将残留位置文法，运行时持续向 agent 教错（fix 文案与 example 同为错误信封三行之一，消费强度相同）。impl_plan 步骤(1) 内容仅写「全部 example 字符串换 v0.6 文法」「仅改字符串文案」，盘点命令虽能命中这三行（含 `paperwork brief create`），但字面范围不覆盖 fix。
- 对照：thread.rs L287（`paperwork post read`）、contacts.rs L55/L97（`paperwork contacts create {}`）等 fix 点位文法不变，属勿误刷类。
- 建议：步骤(1) 范围改为「example 与 fix 两类字符串中的 v0.6 文法点位」，点名 manifest.rs L79/L150/L193（改 `--title "My Brief"` 形态）；步骤(0) 盘点输出按 example/fix/不变文法三类标注。

### minor

**m-1 spec §4 短形式全表遗漏 post read 的 `--reply-to`**

- 证据：worktree post.rs L93 read 子命令确有 `--reply-to` 过滤 flag；spec §4 仅给「post send `--reply-to` | -r」，「其余无短形式」行枚举了 --entry/--entry-title/--to/--from/--mention/--regex/--scope-*/--full/--json/--plain/-q，未含 read 的 --reply-to。BDD S-SHORT-02 断言「全 CLI flag 名与短形式集合与 spec §4 全表一致」，该 flag 归属成空白；且同组内 send 的 --reply-to 有 -r 而 read 的无，短形式不对称本身也应显式裁定。
- 建议：§4 「其余」行补「post read --reply-to：无短形式」并给一句理由（read 过滤低频）。

**m-2 spec §4 首行「全 CLI 短形式语义无冲突」与自身表格矛盾（措辞问题）**

- 证据：§4 表中 -m（post --message / profile --model）、-p（post send --participants / contacts add --profile）、-t（--title / validate --type）、-d（--description / --base-dir）均为跨命令同短形式异语义；design §3 的实际判据是「组内唯一、语义不冲突」。首行措辞过强，与表格自相矛盾。不影响 S-SHORT-01/02 可测性（等价性只在同一命令内断言）。
- 建议：首行改为「短形式在同一命令内唯一；跨命令复用仅限语义同构（-t 标题类）或命令互斥的场景」。

**m-3 盘点命令口径与「14 处」不一致，需过滤规则**

- 证据：impl_plan 步骤(0) 命令原始输出 31 行（含 example: String::new() 占位、fix 文案、文法不变点位）；「确认 14 处清单」需人工过滤。14 处之数本身与当前基线改写集吻合（见事实层），但 format-v2 合并可能增减点位（文档已兜底「以盘点输出为准」，可接受）。
- 建议：步骤(0) 命令收窄为 `rg -n "example: format!\(\"paperwork"`，并标注「14 处改写 + 5 处文法不变勿误刷 + fix 点位见 M-4」。

**m-4 代码内文法注释未列入刷新范围**

- 证据：post.rs L1-5 与 main.rs L6-7 的模块注释（「v0.5.0 grammar: ... NAME is the second required positional ...」）在 v0.6 下失真；无 clippy 风险，纯防误导。
- 建议：步骤(2)/(3) 各加一句「文件头文法注释同步刷新」。

---

## 三、评审焦点逐项结论

1. **clap 可行性**：required_unless_present + conflicts_with 组合成立且报错落 usage 信封（M-1 仅层级归属措辞错误）；短形式与现有全部 short（仅 -q）无冲突成立；但 `--message` 值以 `-` 开头直传需 allow_hyphen_values，文档缺指令（C-1）。
2. **清单准确性**：core「14 处」与当前基线吻合；ops_tests 零改动防线实测成立；cli_integration 按类清单基本覆盖，但遗漏 6+ 处断言语义翻转点（M-3）；tdd 放弃预置行号、以实施时盘点为准的做法正确规避了行号漂移风险。
3. **门禁合理性**：分阶段门禁可执行——步骤(1)-(3) 期间 cli_integration 走 assert_cmd 调二进制，编译/clippy 不受断言红影响，`cargo build + cargo test -p paperwork-core + clippy` 绿可推进；步骤(4) workspace 全绿硬门禁与 F6 一致。ops_tests 不引用任何 example/fix 字符串（grep 0 命中），防线在新文法下仍成立。
4. **输出协议冻结风险**：usage 信封示例刷新、error JSON command 字段、纯 ASCII 契约未见破坏点，唯 C-1/M-2 两处契约自相矛盾需先行闭合；`--message` 值以 `-` 开头的 BDD 断言（S-SEND-10）在缺 allow_hyphen_values 时不正确（C-1）。
5. **遗漏波及面**：ci.yml smoke / README / SKILL.md / core example / after_help / 隐藏别名均已覆盖；Cargo.toml description 与 publish.ps1 实测无文法耦合，确认无关；新增遗漏点为 core fix 文案（M-4）、usage_fix 教学文案（M-3）、文件头注释（m-4）。

---

## 四、问题统计与总体结论

| 级别 | 数量 | 编号 |
|---|---|---|
| Critical | 1 | C-1 |
| Major | 4 | M-1 / M-2 / M-3 / M-4 |
| minor | 4 | m-1 / m-2 / m-3 / m-4 |

**总体结论：需 rework（未闭合）。**

文档集整体质量高：三规则重构自洽、冻结条款与继承声明精确、tdd 放弃预置行号改实施盘点的做法正确、ops_tests 防线与分阶段门禁经实测成立、短形式设计无实际冲突。但存在 1 处 Critical（C-1：`-` 开头正文直传承诺在 clap 默认行为下不可兑现且无实现指令）与 4 处 Major（互斥缺省报告层级归属错误、二选一 example 断言与 F2/F7 裁定冲突、语义翻转调用点与迁移教学文案遗漏、core fix 文案越出步骤(1) 范围）。修复路径均为文档级改写（spec §3.1/§4/§5/§8、design §2.1/§6、bdd 四处断言、tdd §1/§4、impl_plan 步骤(0)/(1)/(2)/(3)），不涉及方案重设计；修复后按本清单复核变更章节即可闭合。
