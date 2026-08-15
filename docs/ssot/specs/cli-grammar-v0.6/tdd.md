# CLI 文法 v0.6: TDD（测试计划）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布）
- 文档性质：测试改写与新增计划（对照 `repos/paperwork-cli/tests/cli_integration.rs`）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令）
  - `docs/ssot/adr/feedbacks/v0.7_feedbacks.md`（本轮 owner 指令：contacts CRUD + 锁统一 + 渐进阅读；§8 本轮测试计划的依据）
  - `docs/dev/adr-v1.md`（ADR-011）、`docs/ssot/adr/feedbacks/v0_feedbacks.md`
- 契约基准：本目录 `spec.md`（签名）与 `bdd.md`（场景编号 S-xxx 与本文用例一一对应）
- **编号约定（rework 补录，Nora ISSUE-m5）**：本文引用的 S-xxx 未带前缀一律指本目录 v0.6 bdd 编号；引用 v0.5 bdd 场景一律带 v0.5 前缀（两版存在同名异义撞号）。

---

## 0. 基线与行号约定

- 测试基线为 **合并提交 a07ad4c 落盘的合并基线**（cli-ux-v0.5 的 v0.5.0 发布形态 + master format-v2：v0.5 位置文法 + post create 删除 + send `--title` 建线程载荷 + core v2 格式；基线事实链见 design §11）。
- **勘误（基线合并后重盘点）**：本节原文以 cli-ux-v0.5 worktree（HEAD 70f7e43）为 §1b 行号基线；合并提交 a07ad4c 落盘后该行号基线失效，§1b 已按合并后基线（cli_integration.rs 1885 行、main.rs 355 行）逐行实测全量重盘点，见 §1b 现行表。
- v0.5 tdd 的 29 处行号清单基于 v0.4 基线，已随 v0.5 实施消耗完毕，不构成本文行号依据。

## 1. 需改写的 v0.5 位置文法调用点（按类清单）

原则：**只改参数层（位置 NAME/BODY/主载荷 -> 具名 flag），不改断言语义**；每处改写后断言仍指向输出协议（§3 保留清单）。

| 类 | 现状形态（v0.5 文法） | 改写为（v0.6 文法） |
|---|---|---|
| post send | `["post","send",path,"alice","body"]` | `["post","send",path,"--author","alice","--message","body"]` |
| post send（stdin） | `["post","send",path,"alice","--stdin"]` | `["post","send",path,"--author","alice","--stdin"]` |
| post edit | `["post","edit",path,"bob","3","edited"]` | `["post","edit",path,"--author","bob","--seq","3","--message","edited"]` |
| profile create | `["profile","create",path,"alice",...]` | `["profile","create",path,"--name","alice",...]` |
| brief create | `["brief","create",path,"Title",...]` | `["brief","create",path,"--title","Title",...]` |
| brief add | `["brief","add",path,"notes.txt",...]` | `["brief","add",path,"--entry","notes.txt",...]` |
| brief remove | `["brief","remove",path,"e.txt"]` | `["brief","remove",path,"--entry-title","e.txt"]` |
| contacts add | `["contacts","add",path,profile_path]` | `["contacts","add",path,"--profile",profile_path]` |
| post read `--from/--to` | `["post","read",path,"--from","2","--to","2"]` | **原样保留**（seq 范围语义冻结，规则 3 正面用例） |

- 覆盖范围：v0.5 基线中全部含位置 NAME/BODY/TITLE/ENTRY/ENTRY-TITLE/PROFILE-PATH 的 `.args(...)` 调用点（含 happy path、stdin、空正文、edit、usage 信封教学用例的前置构造调用）；v0.5 时期新增的 usage 信封测试中以 v0.5 位置文法为「触发样例」的用例，其触发样例改为「v0.5 文法作为旧文法」的迁移教学形态（见 §4 新增用例），断言语义不变。
- 盘点命令（实施第一步执行，输出即改写清单，逐处勾销）：

```
rg -n "\"(send|edit|create|add|remove)\"" repos/paperwork-cli/tests/cli_integration.rs
```

- **盘点输出过滤规则（rework 补录 Quinn minor m-3）**：命中行需人工甄别，仅「参数层携带位置 NAME/BODY/主载荷」的调用点属改写范围；命中但属于断言字符串、错误文案断言、注释的用例不属改写而属 §1b 语义翻转或 §3 冻结保留甄别对象。core 层 14 处 example 口径同理：14 处为「需改写文案」口径（v0.5 rework 实测盘净），非全量命中数。

## 1b. 断言语义翻转点清单（rework 补录 Quinn M-3；闭合复核 NF-1 以实测全量重做；基线合并后按 a07ad4c 全量重盘点，实施第四步处理）

**行号口径（表头注明）**：本清单全部行号为 **合并基线（提交 a07ad4c，分支 cli-grammar-v0.6，worktree agent-paperwork-wt-v06grammar）** 内 `repos/paperwork-cli/tests/cli_integration.rs`（1885 行）与 `repos/paperwork-cli/src/main.rs`（355 行）逐行实测值；前一版行号基于 cli-ux-v0.5 worktree（HEAD 70f7e43），随合并失效，已整体作废重盘。

v0.6 再合法化 `--name/--seq/--title/--entry/--profile` 等 flag、位置槽收窄为仅 PATH、post create 随 format-v2 删除后，下列点位的**断言语义本身**需翻转或失效（非仅参数层改写）。处置方式图例：改写=用例保留但断言/触发器换新；翻转=断言方向反转（负翻正、validation 翻 usage、usage 翻成功）；删除=用例整体废止（由 §4 新用例取代或随命令删除）。

### 1b-A 再合法化 flag 触发器失效（旧文法 usage 教学用例的触发 flag 在 v0.6 变合法）

| 实测行号 | 测试函数 | 现行断言语义（合并基线，v0.5 文法） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L478 | usage_old_grammar_profile_create_name | `profile create <PATH> --name alice` -> `.code(2)` + error usage + example 断言（L488 "paperwork profile create agents/alice alice"） | `--name` 再合法化为必填 flag，该调用翻转为 exit 0 成功，`.code(2)` 必红且不可只改参数层修复 | 改写：触发器换为 v0.5 位置文法形态（`profile create <PATH> alice` 多余位置参数 -> usage exit 2）或删除并入 S-PROF-03；example 断言随 canonical_example 换新（新形态含 --name，L488 断言串不再被包含，须同步） |
| L492 | usage_old_grammar_brief_add_entry | `brief add <PATH> --entry e.txt` -> `.code(2)` usage | `--entry` 再合法化为必填 flag，翻转为 exit 0 成功 | 同上：改触发器为 v0.5 位置文法形态或删除并入 S-BRIEF-04 |
| L507 | usage_old_grammar_contacts_add_profile | `contacts add <PATH> --profile x.profile.md` -> `.code(2)` usage | `--profile` 再合法化为必填 flag，翻转为 exit 0 成功 | 同上：改触发器或删除并入 S-CONTACTS-04 |
| L522 | usage_old_grammar_post_edit_seq | `post edit <PATH> --seq 1 --from alice new` -> `.code(2)` usage | `--seq` 再合法化后用例仅靠未知 `--from` 维持 usage exit 2，语义与用例名（旧文法 --seq）脱节 | 改写：用例改名并改触发器为仍非法的 flag（--from 作身份），并入 S-SEND-13 迁移链；L533 example 断言随文法换新 |

### 1b-B `--` 边界与 dash body 用例（v0.6 废止 `--` 边界教学）

| 实测行号 | 测试函数 | 现行断言语义（合并基线，v0.5 文法） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L1008 | dash_body_with_double_dash_send_and_edit | `-` 开头正文置 `--` 后（L1015 send、L1027 edit）-> exit 0 逐字写入 | `--message` 直传 `-` 开头值 exit 0（依赖 allow_hyphen_values，裁定 F4）；`--` 边界形态废止 | 翻转改写为 S-SEND-10 直传用例（用例注释钉住属性名） |
| L1040 | dash_body_without_double_dash_is_usage | `send <PATH> alice "-fix flag text"` 无 `--` -> `.code(2)` usage + fix 教学 `--` + example `-- "-fix flag text"`（L1052-L1053） | 该形态在 v0.6 合法（--message allow_hyphen_values）-> exit 0 成功，用例整体翻转 | 翻转：改为 S-SEND-11 裸 `-` 开头 token（疑似误写 flag）教学用例，fix 引导 `--message` 形态 |

### 1b-C conflicts / required_unless_present 语义升级（validation exit 1 -> usage exit 2）

| 实测行号 | 测试函数 | 现行断言语义（合并基线，v0.5 文法） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L1319 | send_body_and_stdin_mutually_exclusive | 位置 body + `--stdin` 同给（L1326 args 行）-> `.code(1)` error validation | `--message` 与 `--stdin` 同给 -> clap conflicts -> usage exit 2（裁定 F2） | 翻转：exit 1 -> exit 2、validation -> usage；参数层改 `--message` + `--stdin`（S-SEND-07） |
| L1334 | send_missing_body_no_stdin_is_validation | PATH + NAME 位置、正文与 --stdin 皆缺（L1341 args 行）-> `.code(1)` error validation | `--message`/`--stdin` 皆缺 -> clap required_unless_present -> usage exit 2（裁定 F2） | 翻转：同上（S-SEND-06） |
| L445 | name_body_confusion_single_string | `send <PATH> "some body text"`（单字符串落入 NAME 槽）-> `.code(1)` error validation + NAME/`--` 教学断言（L456-L458） | 位置 NAME 槽消失，该形态 -> clap 多余位置参数/缺必填 -> usage exit 2，静默写入路径不可达 | 翻转：validation -> usage；并入 S-SEND-15 混淆面消亡确认（v0.5 `--` 教学断言拆除） |

### 1b-D 缺必填 message 列表断言（NAME 位置槽消失）

| 实测行号 | 测试函数 | 现行断言语义（合并基线，v0.5 文法） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L649 | usage_missing_required_argument_full_message | 断言 message 含 "required arguments were not provided: <NAME>"（L664 默认信封、L675 --json 信封） | NAME 位置槽消失，必填改 `--author/--message`，缺失列表文案翻转 | 翻转：断言文案随 clap 新缺失列表（`--author <AUTHOR>`/`--message <MESSAGE>` 形态）更新 |

### 1b-E 帮助面负向断言与 post create 块（flag_inventory / format-v2 删除）

| 实测行号 | 测试函数 | 现行断言语义（合并基线，v0.5 文法） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L1223 | flag_inventory_matches_spec | 负向断言 `!profile_create_help.contains("--name")`（L1256） | profile create 在 v0.6 有 `--name` 必填 flag | 翻转为正向：`profile_create_help.contains("--name")` |

注：前一版表中 flag_inventory 的 post create 块（旧 L1244-L1249）、post_create_missing_title_usage（旧 L1038）、post_create_duplicate_already_exists（旧 L1052）、post_group_help_lists_verbs 去 create（旧 L1424）四项已随合并基线消耗完毕：post create help 块已删除；前两用例分别由 post_create_removed_is_usage（L1059）与 post_send_title_ignored_on_existing_thread（L1075）取代；后者（L1455，断言 L1461-L1465）已不含 create。均不再属 §1b 处置面。

### 1b-F 迁移链用例的 example 跟随（断言语义保留、示例换新）

| 实测行号 | 测试函数 | 现行断言语义（合并基线，v0.5 文法） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L462 | usage_old_grammar_send_from | `send <PATH> --from alice body` -> `.code(2)` usage（`--from` 于 send 在 v0.6 仍非法，断言语义保留） | 语义不变，承担 v0.4->v0.5->v0.6 迁移链（S-SEND-13） | 改写：L474 example 断言随 canonical_example 换 v0.6 具名形态 |

### 1b-G example 断言跟随（canonical_example 换新引发的断言串更新，随 §3 冻结防线口径）

| 实测行号 | 测试函数 | 现行 example 断言串 | v0.6 目标 | 处置 |
|---|---|---|---|---|
| L429 | usage_missing_body_post_send | "paperwork post send standup.post.md alice"（L441） | send 规范示例换具名形态（采 --message 通道，裁定 F5） | 改写断言串 |
| L478 | usage_old_grammar_profile_create_name | "paperwork profile create agents/alice alice"（L488） | profile create 规范示例含 `--name alice` | 改写断言串 |
| L492 | usage_old_grammar_brief_add_entry | "paperwork brief add onboarding.brief.md src/main.rs"（L503） | brief add 规范示例换 `--entry src/main.rs` 形态，contains 断言必红 | 改写断言串 |
| L507 | usage_old_grammar_contacts_add_profile | "paperwork contacts add team.contacts.md"（L518） | contacts add 规范示例换 `--profile` 形态 | 改写断言串 |
| L522 | usage_old_grammar_post_edit_seq | "paperwork post edit standup.post.md alice 3"（L533） | edit 规范示例具名形态（--author/--seq/--message） | 改写断言串 |
| L1099 | profile_create_missing_name_usage | "paperwork profile create agents/alice alice --model gpt-4o"（L1109） | profile create 规范示例换 `--name` 形态 | 改写断言串 |

注：前一版「L482（brief add）、L497（contacts add）example 断言不变」注记随重盘点勘误：合并基线 main.rs canonical_example 尚不含 `--entry`/`--profile` 形态（brief.add 示例为 `--regex` 形态 L332、contacts.add 示例为位置 profile 路径形态 L347），v0.6 换新后 L503/L518 两处 contains 断言必红，已入表。

### 1b-H main.rs 文案点位实测清单（usage_fix base 与旧 flag 教学，列入 impl_plan 步骤(3)）

| 实测行号（main.rs） | 点位 | 现行文案（合并基线，v0.5 文法） | v0.6 目标 | 处置 |
|---|---|---|---|---|
| L1-L7 | 文件头文法注释 | v0.5.0 grammar 描述 | v0.6 文法描述 | 改写 |
| L25 | after_help Grammar 行 | `Grammar: paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]` | 位置槽仅剩 PATH；必填与可选一律具名 flag（必填段移出方括号） | 改写 |
| L85-L90 | dash_body 判定（L87-L89）与 canonical_example 切换 | 疑似 `-` 开头 body 时 example 切 `--` 边界形态 | `--` 边界教学废止，换 `--message` 直传形态；双形态收敛后该切换可删除 | 改写 |
| L214-L215 | usage_fix base 文案 | `required values are positional (PATH first; NAME second for post send/edit); see the canonical example below` | 必填改具名 flag（--author/--message/--name 等），base 重写 | 改写 |
| L219-L221 | 长未知 flag 分支旧文法教学清单 | `pre-v0.5 grammar (--from/--seq/--title/--entry/...), give its value as a positional argument` | `--seq/--title/--entry`（及 `--name/--profile`）再合法化后出列；清单收窄为仍非法 flag（如 --from 作身份）；「give its value as a positional argument」引导随位置槽收窄失效 | 改写 |
| L224-L227 | dash_body fix 分支（post.send/post.edit） | `if a body value starts with '-', place it after -- (e.g. ... alice -- "-fix flag text")` | `--message` 直传（allow_hyphen_values），`--` 边界教学废止 | 改写 |
| L228-L231 | 其余命令 dash 值 fix 分支 | `if a value starts with '-', place it after --` | 按 v0.6 具名 flag 口径重写 | 改写 |
| L274/L314/L352 | 三级 fallback 示例 | `paperwork post send standup.post.md alice "Hello"` | 具名形态 | 改写 |
| L279-L303 | canonical_example post send/edit 臂（L283/L288 send、L296/L301 edit，dash_body 双形态） | 位置文法 + `--` 边界形态示例 | 具名形态单一规范示例（--message 通道，F4/F5）；dash_body 双形态收敛 | 改写 |
| L309-L312 | canonical_example post create 臂 | `paperwork post create standup "Daily Standup" --participants alice,bob` | post create 随 format-v2 删除 | 删除（落入 post fallback；示例串同步勘误 `--participants` 失实内容，见 design §11） |
| L317/L325 | profile create 臂与 profile fallback | `paperwork profile create agents/alice alice --model gpt-4o` | `--name alice` 形态 | 改写 |
| L328/L340 | brief create 臂与 brief fallback | `paperwork brief create onboarding "Codebase Onboarding" --owner alice` | `--title` 形态 | 改写 |
| L332 | brief add 臂 | `paperwork brief add onboarding.brief.md src/main.rs --regex "fn main"` | `--entry src/main.rs` 形态 | 改写 |
| L336 | brief remove 臂 | `paperwork brief remove onboarding.brief.md main.rs` | `--entry-title main.rs` 形态 | 改写 |
| L343/L350 | contacts create 臂与 fallback | `paperwork contacts create team --title "Core Team"` | --title 本为 flag 形态，不变 | 保留 |
| L347 | contacts add 臂 | `paperwork contacts add team.contacts.md agents/alice.profile.md` | `--profile agents/alice.profile.md` 形态 | 改写 |
| L351 | validate 臂 | `paperwork validate mystery.md --type post` | 不变 | 保留 |

**同源文案刷新清单（列入 impl_plan 步骤(3)）**：上表 1b-H 全部点位 + 各命令 after_help/long_about 中旧 flag 教学清单同步刷新；刷新后 cli_integration.rs 中断言跟随文案同步（1b-G），ASCII 契约防线覆盖新文案。

## 2. core 层 example 断言同步（防线说明）

`repos/paperwork-core/tests/ops_tests.rs` 不引用 example 字符串（断言面向解析/锁/seq/hash 行为），故 core 层 14 处 example 文案换新（impl_plan 步骤(1)）不引发 ops_tests 任何改动；若盘点发现个别用例断言含 example 片段，按「断言跟随文案同步更新、行为断言不动」处理并在 review 中点名。

## 3. 必须原样保留的输出协议断言清单（冻结防线）

沿用 v0.5 tdd §2 的全部保留项（以合并后基线实际行号为准），类别清单不变：

| 类别 | 断言内容 | 冻结依据 |
|---|---|---|
| ok 信封首行 | `ok post.send` / `ok post.read` / `ok post.edit` / `ok profile.create` / `ok profile.show` / `ok profile.edit` / `ok brief.create` / `ok brief.add` / `ok brief.remove` / `ok contacts.create` / `ok contacts.add` / `ok validate` | command 标识与参数文法解耦（spec §7） |
| 字段断言 | `name: alice`、`sender: alice`、`changed: model`、`showing: n/total`、`window: #a-#b`、`implicit-mention` 触发与不触发边界 | 输出增补字段冻结 |
| 错误分类词 | `error already-exists:` / `error validation:` / `error format:` / `error not-found:` / `error not-allowed:` / `error usage:` | 七类 category 冻结枚举 |
| JSON 断言 | `"status":"ok"/"error"`、既有字段 key、错误对象 `command` 与 `exit_code`（运行时 1 / usage 2） | JSON 只增不改不删 |
| 退出码断言 | 运行时错误 exit 1；usage 错误 exit 2；--help/-V exit 0 | 退出码语义冻结 |
| `-q` 语义 | 隐 `ok` 首行、字段与 body 保留 | 全局 flag 冻结 |
| stdin 回读 | stdin 正文逐字回读 | 行为冻结 |
| verify 三态 | JSON 含 `fresh` / `shifted` | 三态契约冻结 |
| ASCII 契约 | `ascii_output_contract_guard`：usage + 运行时错误 stderr 原始字节逐一 `is_ascii` | 纯 ASCII 输出契约（spec §5 第 4 条） |

## 4. 新增用例清单

| 用例 | 对应 BDD | 断言要点 |
|---|---|---|
| 缺 `--author`（send/edit） | S-SEND-05 / S-EDIT-02 | `.code(2)`；stderr `error usage:`；example 含 `--author` 与 `--message` 完整必填形态 |
| 缺 `--message` 且无 `--stdin`（send/edit） | S-SEND-06 / S-EDIT-04 | `.code(2)`；`error usage:`（clap `required_unless_present` 组合，rework 裁定 F2）；example 为单一静态规范可执行示例（采 `--message` 形态，rework 裁定 F5） |
| 缺 `--seq`（edit） | S-EDIT-03 | `.code(2)`；`error usage:`；example 含 `--seq` 完整形态 |
| `--message` 与 `--stdin` 同给（send/edit） | S-SEND-07 / S-EDIT-05 | `.code(2)`；`error usage:`（clap conflicts）；无文件写入；example 为单一静态规范示例（F5） |
| 短形式与全称等价 | S-SEND-02 / S-SHORT-01 | `-a/-m` 与 `--author/--message` 行为逐字等价；spec §4 全表逐 flag 等价抽查（F3 收窄后短形式仅 {-a, -m, -q}） |
| v0.5 位置文法迁移（send/edit/profile/brief/contacts） | S-SEND-12 / S-EDIT-08 / S-PROF-03 / S-BRIEF-04 / S-CONTACTS-04 | `.code(2)`；`error usage:`（多余位置参数）；example 为对应命令 v0.6 规范形态（不携带用户原参数值） |
| v0.4 旧 flag 迁移链延伸 | S-SEND-13 | `--from` 于 send 不存在 -> `.code(2)` + `error usage:` + v0.6 规范示例 |
| 混淆面消亡确认 | S-SEND-15 | `send <PATH> "text"` -> `.code(2)` usage（不再是 v0.5 的 validation exit 1）；无文件写入；确认静默写入路径不可达 |
| `--message` 值以 `-` 开头直传 | S-SEND-10 | exit 0；正文逐字含 `-` 开头文本；**无** `--` 边界；**属性依赖注明（rework 裁定 F4）**：本用例通过前提是 send/edit 两处 `--message` 均设 `allow_hyphen_values = true`（impl_plan 步骤(2) 硬性指令），用例注释需钉住该属性名，防止后续重构误删属性后用例静默降级 |
| 裸 `-` 开头 token 教学 | S-SEND-11 | `.code(2)`；`error usage:`；fix 引导 `--message` 形态；example 为 `--message "-fix flag text"` 形态 |
| `--mention` 无短形式 | S-READ-04 | `read -m alice` -> `.code(2)` usage；`--mention alice` exit 0 |
| profile create 缺 `--name` | S-PROF-02 | `.code(2)`；`error usage:`；example 含 `--name` 完整形态 |
| brief 三命令缺必填 flag | S-BRIEF-03 | 三条 `.code(2)` usage；example 分别含 `--title/--entry/--entry-title` |
| contacts add 缺 `--profile` | S-CONTACTS-03 | `.code(2)`；`error usage:`；example 含 `--profile` |
| SEQ 非数字 | S-EDIT-06 | `.code(2)`；`error usage:` |
| 空正文（`--message "   "`） | S-SEND-09 | `.code(1)`；`error validation:` |
| ASCII 契约回归 | S-OUT-05 | 新增 usage 形态（缺必填 flag、conflicts、多余位置参数）纳入 stderr 逐字节 ASCII 断言 |
| 命名政策白名单 | S-SHORT-02 | 组/动词集合精确等于 {profile,post,brief,contacts,validate}；flag 集合与 spec §4 一致；短形式集合精确等于 {-a, -m, -q}（F3）；全量无短形式负向断言 |
| send 元数据 flag 既有线程静默忽略 | S-SEND-17 | exit 0；title 不变（F6 行为登记；基线勘误后缩减为仅 `--title`；既有用例 post_send_title_ignored_on_existing_thread L1075 已在合并基线落盘，步骤(4) 仅改参数层为具名形态） |
| `--author` 空值 | S-SEND-18 | `.code(1)`；`error validation:` |
| 缺 PATH（send） | S-SEND-19 | `.code(2)`；`error usage:` |
| edit 仅 `--stdin` | S-EDIT-09 | exit 0；正文逐字为 stdin 内容 |
| read total 口径与空 window | S-READ-06 / S-READ-07 | `showing: 0/0` 无 window 字段；`showing: 20/25`（两者均为过滤后口径，fix-ledger A-01） |
| read `--to` 身份值 / read `--author` 迁移 | S-READ-08 / S-READ-09 | 两者均 `.code(2)` usage（F1 显式方向防线；习惯迁移 fix 点名 `--mention`） |
| `--json` 与 `--plain` 同给 | S-OUT-06 | `.code(2)`；JSON 错误对象 `category:"usage"` |
| 冻结回归抽查 | v0.5 bdd S-READ-01~03 / S-SUM-01 / S-PATH-* / S-ALIAS-* / S-OUT-01~04 | v0.5 既有对应用例改参数层后断言原样通过（showing/window/implicit-mention/三级解析/别名/三档输出） |

## 5. ops_tests.rs 零改动声明

`repos/paperwork-core/tests/ops_tests.rs` **一行不改**。理由与防线作用：

- core 公开 API 零变更（spec §7），ops_tests 全部用例应当原样通过；
- core 层唯一改动是 CLI 文法 example 字符串 **14 处**（沿用 v0.5 rework 轮实测盘净结论；合并 format-v2 后行号漂移，实施前以检索命令重新盘点），不触及锁/seq/格式/hash 逻辑；
- ops_tests 因此成为「core 行为未被文法重设计污染」的回归防线：任何失败都意味着改动越界。

## 6. 测试语料目录约定

- cli_integration.rs 自身使用 TempDir，不依赖仓库内语料目录（沿用 v0.4/v0.5 先例）。
- 仓库内 `test-v03/`、`test-v04/`、`test-v05/` 为历史版本人工实测样例集，**不得改动**；若 QA 需要 v0.6 冒烟样例或 usage 信封演示样例，按 test-v05/ 结构新建 `test-v06/`（含正常样例 + 刻意损坏样例 + v0.5 旧文法迁移演示）。
- `_fix/` 目录为历史修复样例，不纳入本次测试范围。

## 7. 验证门禁

1. **分阶段门禁（沿用 v0.5 F6 裁定）**：core 文案步与 CLI 签名步期间，`cargo build` + `cargo test -p paperwork-core`（ops_tests 恒绿）+ clippy 全绿即可推进，cli_integration 允许红；集成测试步完成后 `cargo test`（workspace 全量）全绿为硬门禁，后续步骤不得带红推进。
2. `cargo clippy --all-targets -- -D warnings` 无警告。
3. 实测冒烟（本文 §4 全部场景 + 并发 send seq 无间隙）由 review/gate 阶段执行，impl agent 不运行长时 e2e（MainAgent工作编排.md 审查条款）。

## 8. 本轮增量测试计划（contacts CRUD additive 轮，2026-08-09）

- 基线：cli-grammar-v0.6 分支（worktree agent-paperwork-wt-v06grammar）的 v0.6 实施完成态；本轮断言对象为 spec §2 本轮新增行、§3.5/§3.6/§3.9 契约与 bdd S-BRIEF-07~09、S-CONTACTS-06~11、§12 锁场景。（**分支已合并删除标注，2026-08-15 任务 #47 / Evan S-2**：cli-grammar-v0.6 分支已于 2026-08-15 删除（本地 + origin，INV-05），实施成果已全量合入 master；本句为历史基线叙述，勿按分支名 checkout。）
- 原则：**新增行为用新用例钉住，既有行为用既有用例冻结回归**；§3 输出协议保留清单与 §5 ops_tests 零改动防线对本轮继续有效。

### 8.1 core 独立测试文件：`repos/paperwork-core/tests/ops_contacts_crud_tests.rs`（新建）

| 用例 | 断言要点 |
|---|---|
| contacts_remove 命中 | 目标条目消失；title（H1）与其余条目顺序不变；Ok |
| contacts_remove 未命中 | `NotFound`（resource Contacts entry）；文件字节逐字不变 |
| contacts_remove 文件不存在 | `NotFound`（resource Contacts） |
| contacts_update 命中（label 重派生） | destination 原地替换；label = NEW profile H1（R11）；其余条目位置不变（顺序保留） |
| contacts_update label 回退 | NEW profile 不可读时 label = 文件名主干（先剥 `.profile.md` 再剥 `.md`） |
| contacts_update OLD 未命中 | `NotFound`；文件不变 |
| contacts_update 文件不存在 | `NotFound`（resource Contacts）；与 remove 文件不存在行对等（共享 exists 预检，rework 补录 Daniel m-6） |
| contacts_update NEW 已存在 | `AlreadyExists`；文件不变 |
| contacts_update OLD == NEW | `AlreadyExists`（NEW 已在清单，边界形态）；判定顺序：OLD 命中检查先于 NEW 已存在检查（OLD==NEW 且 OLD 未命中时落入 `NotFound`，与 OLD 未命中行重合，rework 补录 Daniel m-6） |
| contacts_update NEW 不存在静默成功 | 仍 Ok：destination 按原值落盘、label 回退文件名主干（与「label 回退」行互补，钉住 CLI 级静默面契约的 core 侧，spec §3.6 行为契约，rework 补录 Ryan M-3） |
| contacts_remove 最后一条目 | 文件仅剩 title H1 + 空行（与 create 初态同形）；parse 合法；再 remove 同键 `NotFound`（rework 补录 Daniel M-4） |
| remove/update 特殊字符路径往返 | 键 = 未转义原串命中含空格/括号/尾随反斜杠路径条目；update 后新路径含空格走 angle-bracket 形态；往返后其余条目字节不变；二次操作仍命中（rework 补录 Daniel M-4） |
| contacts_add 幂等回归 | 补锁后行为不变：已存在条目再 add 仍 no-op Ok（spec §3.6） |
| 锁内序列化等价 | brief add/remove、profile edit、contacts 三写路径锁内产物与既有 serialize 函数输出逐字节一致（同一 `serialize_contacts`/`serialize_manifest`/profile 序列化） |
| 多线程并发 add/remove | 条目无丢失；结果可被 `parse_contacts` 解析（无交错损坏） |

注：本文件不引用 example 字符串（口径同 §2）；锁行为断言面向结果（条目集合/字节不变），不依赖锁实现细节。

### 8.2 cli_integration.rs 新增用例表

| 用例 | 对应 BDD | 断言要点 |
|---|---|---|
| contacts remove 成功 | S-CONTACTS-06 | `.code(0)`；stdout 首行 `ok contacts.remove <profile> -> <path>`；字段 `contacts`/`removed`；文件断言条目消失、余条保留；`--json` 含 `command:"contacts.remove"` 与同名 key |
| contacts remove 未命中 | S-CONTACTS-07 | `.code(1)`；`error not-found:`；fix 含 `contacts read`；文件不变 |
| contacts update 成功 | S-CONTACTS-08 | `.code(0)`；`ok contacts.update <OLD> -> <NEW>`；字段 `contacts`/`updated`（值逐字为 `<OLD> -> <NEW>` 单空格三段拼接，spec §3.6）；顺序保留 + label 重派生文件断言；`--json` 含 `command:"contacts.update"` |
| contacts update 错误路径 | S-CONTACTS-09 | OLD 未命中 `.code(1)` `error not-found:`（fix 含键口径教学句 `the key is the profile path`）；NEW 已存在 `.code(1)` `error already-exists:`；均无文件写入 |
| remove/update 缺必填 flag | S-CONTACTS-10 | 三形态均 `.code(2)`；`error usage:`；example 逐字断言（spec §5 第 2 条钉住的两条规范示例，rework 补录 Ryan m-2） |
| contacts remove 位置参数误用 | S-CONTACTS-11 | `.code(2)`；`error usage:`（多余位置参数）；example 为 `--profile` 形态 |
| remove 最后一条目形态 | S-CONTACTS-12 | `.code(0)`；文件仅剩 title H1 + 空行；validate 合法；再 remove 同键 `.code(1)`（rework 补录 Daniel M-4） |
| 特殊字符路径往返 | S-CONTACTS-13 | 转义条目键匹配命中；其余条目字节不变；二次操作仍命中（rework 补录 Daniel M-4） |
| update NEW 不存在静默成功 | S-CONTACTS-14 | `.code(0)`；条目落为原值 destination + 回退 label；`updated` 回显原值（rework 补录 Ryan M-3） |
| label-as-key 触发形态 | S-CONTACTS-07 And 段 | `--profile alice`（label 当键）`.code(1)` `error not-found:`；fix 含键口径教学句（rework 补录 Ryan m-3） |
| add/update 空键护栏 | S-CONTACTS-15 | `--profile ""` 与 `--new-profile ""`（含全空白）均 `.code(1)` `error validation:`；message/fix 逐字断言（spec §3.6 空键护栏）；无文件写入；护栏先于 not-found 判定；既有覆盖：core 单测 ops_contacts_crud_tests.rs（`profile path (--profile) is empty` 断言）与集成测试 cli_integration.rs（任务 #34 文档轮补映射，grep 实证测试在场） |
| brief read 选择性详情 | S-BRIEF-07 | exit 0；stdout 首行 `ok brief.read <N> entries`（N 为全量条目数，现状冻结形态，rework 修订 Daniel M-2）；输出仅含目标条目详情字段（path/hash/regex/note）；`--json` entries 仅含该条目且含 path/hash/regex/note（命中即 --full 档字段，不受 `--full` 门控，Daniel m-4） |
| brief read 无匹配 | S-BRIEF-08 | `.code(1)`；`error not-found:`；fix 含 `brief read` |
| brief read 组合 --full | S-BRIEF-09 | exit 0；与单条目详情等价；未给 `--entry-title` 时 TOC/--full 两档冻结回归（既有 brief read 用例原样通过） |
| brief read `--entry-title` 空值守栏 | S-BRIEF-10 | `--entry-title ""`（含全空白）`.code(1)` `error validation:`；message 逐字 `entry title (--entry-title) is empty`、fix 逐字 `provide a non-empty --entry-title value`、example 逐字断言（spec §3.5 空值守栏）；无文件写入；既有覆盖：集成测试 cli_integration.rs（`entry title (--entry-title) is empty` 断言，任务 #34 文档轮补映射，grep 实证测试在场） |
| 多进程并发 contacts/brief 写 | S-LOCK-01 | 全部 exit 0；条目集合 = 并集；validate 合法；Given 预创建 N 个 entry 目标文件（brief add 快照前置，rework 补录 Daniel m-2） |
| profile edit 并发 | S-LOCK-02 | 两者 exit 0；最终文件 validate 通过，终态为两次编辑的字段并集（不重叠字段无丢失写；同字段变体则最后写入者胜，二选一在用例内写清，rework 修订 Daniel M-1） |
| ASCII 契约扩展 | S-OUT-05 延伸 | remove/update 的 usage/not-found/already-exists 信封 stderr 纳入逐字节 ASCII 断言；`all_help_output_is_pure_ascii` 动词清单追加 `contacts remove`、`contacts update` 两行（现状清单止于 contacts create/add/read，rework 补录 Daniel m-3） |

### 8.3 白名单测试更新项（flag_inventory_matches_spec 及配套；rework 修订：措辞由「追加」改为如实的「新建/扩展断言面」，Mark M-3/Ryan m-1/Daniel M-3 定案）

**现状基线（worktree cli_integration.rs 实测，Daniel 评审 §六）**：`short_form_whitelist_is_exact` 仅 6 个负向短形式探针（`-s/-l/-n/-t/-e/-p`），不存在 26 项逐 flag 负向清单；组级动词集合断言仅 post 组存在（`post_group_help_lists_verbs` 先例），contacts 组无任何动词断言；`all_help_output_is_pure_ascii` 动词清单止于 contacts create/add/read。

1. 无短形式负向断言清单：本轮**新建/扩展**为 bdd S-SHORT-02 枚举的全量清单（项数以 bdd 枚举分项口径为准，不维护硬编码总数；含本轮 additive 的 `--new-profile`，spec §4 全表同步）；`--new-profile` 探针建议形态：`contacts update <PATH> --profile a.profile.md --new-profile b.profile.md` 加 `-N`/`-w` 类短形式误写触发 usage exit 2；
2. contacts 组 help 动词列表断言：**新建**（现状不存在可追加点位），仿 `post_group_help_lists_verbs` 体例断言 contacts 组动词集合精确等于 {create,add,remove,update,read}，含反向断言（不出现清单外动词）；`update` 的白名单扩容来源登记见 spec §7 第 5 条与 v0.7_feedbacks §2.5（owner 指令 (1) 授权）；
3. 短形式集合断言 {-a, -m, -q} 不变（新 flag 一律仅长形式）；
4. 组集合断言 {profile,post,brief,contacts,validate} 不变；
5. `all_help_output_is_pure_ascii` 动词清单追加 `contacts remove`、`contacts update`（ASCII 逐字节防线覆盖新 verb help 面，Daniel m-3）。

### 8.4 ops_tests.rs 零改动防线（本轮延续）

`repos/paperwork-core/tests/ops_tests.rs` 本轮继续**字节级零改动**：core 本轮仅新增函数（`contacts_remove`/`contacts_update`）与既有写路径补锁，不改既有函数签名与序列化逻辑，锁内产物与无锁产物逐字节一致（§8.1 锁内序列化等价用例钉住）；ops_tests 任何失败都意味着改动越界（越界即回滚，口径同 impl_plan 全局门禁）。新 core 测试一律落独立文件 `ops_contacts_crud_tests.rs`，不得并入 ops_tests.rs。

### 8.5 测试语料

- TempDir 约定不变（§6）；历史语料目录 test-v03/test-v04/test-v05 不得改动；若 QA 需要本轮冒烟样例，按 §6 既有 test-v06/ 先例结构补充（含 contacts remove/update 正常与错误形态、brief 多条目选择性详情样例），样例全 ASCII。

### 8.6 验证门禁（本轮）

1. `cargo test`（workspace 全量）全绿（含 ops_tests 字节级零改动恒绿 + ops_contacts_crud_tests 新绿）；
2. `cargo clippy --all-targets -- -D warnings` 无警告；
3. ASCII 审计：本轮新信封文案（contacts.remove/update 的 ok/usage/not-found/already-exists/io 各形态）纳入 stderr 逐字节 ASCII 断言（S-OUT-05 防线延伸）；
4. 锁调用点位盘点：`rg -n "lock_exclusive" repos/paperwork-core/src` 输出含六写路径新锁点位，且无任何无锁 read-modify-write 残留（S-LOCK-03 不变量）；
5. 明确不含发布步骤（不 bump、不 tag、不 publish、不写 CHANGELOG 发布段，口径同 spec §7 第 4/5 条）。

---

## 9. owner 裁决批测试计划（2026-08-15 裁决轮；文档落盘任务 #35，实施与测试落地归任务 #36）

- 依据：docs/dev/owner-rulings-2026-08-15.md；spec §1.4/§2/§3.1/§3.3/§3.6/§4/§5/§7 第 6 条/§10；bdd S-SEND-04/20（改写）、S-SEND-22/23、S-EDIT-10、S-CONTACTS-14（追加）、S-CONTACTS-16/17、S-SHORT-02（枚举收窄）。
- 原则沿用：新增行为用新用例钉住，既有行为用既有用例冻结回归；§3 输出协议保留清单对未撤销面继续有效。

### 9.1 既有用例改写/失效盘点（实施首步执行，逐处勾销）

盘点命令：

```
rg -n "reply[-_]to|--mention|mention" repos/paperwork-cli/tests/cli_integration.rs repos/paperwork-core/tests
```

| 类 | 处置 |
|---|---|
| send 侧 `--reply-to`/`--mention` 成功路径用例（糖衣注入断言：正文首行 `@#N`/`@name` 注入形态、implicit-mention 由 flag 触发形态、`--reply-to 0` validation 分支） | 改写为正文直书形态（断言语义保留：derive/implicit-mention 边界不变，触发方式改为正文 token，对应改写后 S-SEND-04/20）或翻转为 usage 断言（对应 S-SEND-22/23）；`--reply-to 0` 分支随 flag 撤销整体删除 |
| send 侧 mention 名单清洗/校验分支用例（trim/空段/非法名字） | 随写侧 flag 撤销删除（正文内 `@name` 不做名字合法性校验，透传冻结） |
| read 侧 `--mention`/`--reply-to` 过滤用例（S-READ-04/06/07 等） | 原样冻结回归，一字不改（读侧保留声明，spec §3.3） |
| contacts add/update 成功路径用例（destination 不存在/非法形态，含 S-CONTACTS-14 对应用例） | 保留 exit 0/条目/label 断言，叠加 `advisory` 字段断言（S-CONTACTS-16/17）；destination 合法路径补反向断言「无 advisory 字段」 |
| post.rs send after_help / core-cli example 中含糖衣 flag 的文案断言 | 随 impl_plan O1 文案同步改写 |

### 9.2 新增用例映射表

| 用例 | 对应 BDD | 断言要点 |
|---|---|---|
| send 传入已撤销 `--reply-to` | S-SEND-22 | `.code(2)`；`error usage:`（未知 flag）；fix/example 引导正文直书 `@#N`；无文件写入 |
| send 传入已撤销 `--mention` | S-SEND-23 | `.code(2)`；`error usage:`；fix/example 引导正文直书 `@name`；无文件写入 |
| edit 传入已撤销 flag（写命令外延） | S-EDIT-10 | `.code(2)`；`error usage:`；example 为 edit 完整必填形态；无文件写入 |
| 正文直书 `@#N` 的 implicit-mention 往返 | S-SEND-04（改写） | exit 0；`implicit-mention` 派生自正文 token（derive 冻结）；`--json` 同名 key |
| 正文直书 `@#N`/`@name` 与读侧过滤往返 | S-SEND-20（改写） | exit 0；正文逐字含 token；read 过滤器命中（读侧保留声明防线） |
| add advisory 触发（不存在/不可读/格式非法） | S-CONTACTS-16 | 三形态均 `.code(0)`；条目照常落盘；`advisory` 字段存在且纯 ASCII；`--json` 含 `advisory` key |
| add advisory 不触发 | S-CONTACTS-16 And 段 | destination 合法时信封**无** `advisory` 字段（反向断言）；S-CONTACTS-02 既有断言原样通过 |
| update advisory 触发 | S-CONTACTS-17 | `.code(0)`；`updated` 与 `advisory` 两 key 并存；行为面与 S-CONTACTS-14 逐条一致 |
| 白名单负向清单收窄 | S-SHORT-02（更新） | send 侧 `--reply-to`/`--mention` 探针移除；read 侧两项保留；枚举口径见 bdd 现行文本 |
| ASCII 契约延伸 | S-OUT-05 延伸 | 撤销 usage 信封与 advisory 字段/提示文案纳入逐字节 ASCII 断言 |

### 9.3 黄金快照重冻预告

- 本批属行为变更（flag 面撤销 + 信封字段新增），凡以下面的既有测试/黄金快照须在任务 #36 行为定稿后**一次性重冻**，重冻 diff 需在 review 中逐处点名：① 涉 send 糖衣注入逻辑的一切断言（正文首行 token 注入形态废止，正文即用户所给原值）；② implicit-mention 触发路径（flag 触发 -> 正文 token 触发）；③ contacts add/update ok 信封快照（新增 `advisory` 字段面）；④ post send/edit after_help 与 usage 信封文案快照（含糖衣 flag 的示例行删除/换向）；⑤ S-SHORT-02 负向清单快照（探针净减两项）。重冻前旧快照不得就地删除，须在提交信息中登记替换关系。

### 9.4 冻结面与零改动防线

- §3 输出协议保留清单中未撤销面（ok/error 信封结构、七类 category、退出码、showing/window、read 侧过滤器、ensure_suffix、别名、三档输出）继续有效；§5 ops_tests.rs 零改动防线延续：本批改动全部位于 CLI 层（糖衣注入本就发生在调用 core 之前，advisory 校验为 CLI 层写后探测），core 公开 API 与文件格式零变更；若盘点发现 core 层确有必要改动，先回报编排层裁定，不得径行。
- 交付边界：本批不含 bump/tag/publish/CHANGELOG 发布段（spec §7 第 4/6 条）。
