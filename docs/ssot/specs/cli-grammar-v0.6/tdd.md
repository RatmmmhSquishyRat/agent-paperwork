# CLI 文法 v0.6: TDD（测试计划）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布）
- 文档性质：测试改写与新增计划（对照 `repos/paperwork-cli/tests/cli_integration.rs`）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令）
  - `docs/dev/adr-v1.md`（ADR-011）、`docs/ssot/adr/feedbacks/v0_feedbacks.md`
- 契约基准：本目录 `spec.md`（签名）与 `bdd.md`（场景编号 S-xxx 与本文用例一一对应）
- **编号约定（rework 补录，Nora ISSUE-m5）**：本文引用的 S-xxx 未带前缀一律指本目录 v0.6 bdd 编号；引用 v0.5 bdd 场景一律带 v0.5 前缀（两版存在同名异义撞号）。

---

## 0. 基线与行号约定

- 测试基线为 **cli-ux-v0.5 分支 + format-v2 工作树变更的合并结果**（v0.5 位置文法 + post create 删除 + send 建线程载荷已就位）；实施第一步即执行全量盘点（检索命令见 §2 末）。
- **勘误（闭合复核 NF-1 定点修复）**：本节原文「本文行号以合并后基线实测为准」失实：合并后基线尚不存在（impl_plan 步骤(0) 合并为编排层未来前置步骤）。§1b 行号基线更正为 **cli-ux-v0.5 worktree（agent-paperwork-wt-cliux，HEAD 70f7e43，git 干净）逐行实测**；实施时若基线变化（合并完成后）须重新盘点。
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

## 1b. 断言语义翻转点清单（rework 补录 Quinn M-3；闭合复核 NF-1 以实测全量重做，实施第三步处理）

**行号口径（表头注明）**：本清单全部行号为 **cli-ux-v0.5 worktree（agent-paperwork-wt-cliux，分支 cli-ux-v0.5，HEAD 70f7e43，git 干净）** 内 `repos/paperwork-cli/tests/cli_integration.rs`（1471 行）与 `repos/paperwork-cli/src/main.rs`（355 行）逐行实测值，逐行经行号定位核验后写入；实施时若基线变化（impl_plan 步骤(0) 合并完成后）须重新盘点，禁止沿用本表行号。（本表前一版本行号与语义映射系未经实测填报，经闭合复核逐行核验无一吻合，已按实测全量重做并勘误，见 §0 勘误注记与闭合复核报告 NF-1。）

v0.6 再合法化 `--name/--seq/--title/--entry/--profile` 等 flag、位置槽收窄为仅 PATH、post create 随 format-v2 删除后，下列点位的**断言语义本身**需翻转或失效（非仅参数层改写）。处置方式图例：改写=用例保留但断言/触发器换新；翻转=断言方向反转（负翻正、validation 翻 usage、usage 翻成功）；删除=用例整体废止（由 §4 新用例取代或随命令删除）。

### 1b-A 再合法化 flag 触发器失效（旧文法 usage 教学用例的触发 flag 在 v0.6 变合法）

| 实测行号 | 测试函数 | 现行断言语义（v0.5） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L457 | usage_old_grammar_profile_create_name | `profile create <PATH> --name alice` -> `.code(2)` + error usage + example 断言（L467 "paperwork profile create agents/alice alice"） | `--name` 再合法化为必填 flag，该调用翻转为 exit 0 成功，`.code(2)` 必红且不可只改参数层修复 | 改写：触发器换为 v0.5 位置文法形态（`profile create <PATH> alice` 多余位置参数 -> usage exit 2）或删除并入 S-PROF-03；example 断言随 canonical_example 换新（新形态含 --name，L467 断言串不再被包含，须同步） |
| L471 | usage_old_grammar_brief_add_entry | `brief add <PATH> --entry e.txt` -> `.code(2)` usage | `--entry` 再合法化为必填 flag，翻转为 exit 0 成功 | 同上：改触发器为 v0.5 位置文法形态或删除并入 S-BRIEF-04 |
| L486 | usage_old_grammar_contacts_add_profile | `contacts add <PATH> --profile x.profile.md` -> `.code(2)` usage | `--profile` 再合法化为必填 flag，翻转为 exit 0 成功 | 同上：改触发器或删除并入 S-CONTACTS-04 |
| L501 | usage_old_grammar_post_edit_seq | `post edit <PATH> --seq 1 --from alice new` -> `.code(2)` usage | `--seq` 再合法化后用例仅靠未知 `--from` 维持 usage exit 2，语义与用例名（旧文法 --seq）脱节 | 改写：用例改名并改触发器为仍非法的 flag（--from 作身份），并入 S-SEND-13 迁移链；L512 example 断言随文法换新 |

### 1b-B `--` 边界与 dash body 用例（v0.6 废止 `--` 边界教学）

| 实测行号 | 测试函数 | 现行断言语义（v0.5） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L987 | dash_body_with_double_dash_send_and_edit | `-` 开头正文置 `--` 后（L994 send、L1006 edit）-> exit 0 逐字写入 | `--message` 直传 `-` 开头值 exit 0（依赖 allow_hyphen_values，裁定 F4）；`--` 边界形态废止 | 翻转改写为 S-SEND-10 直传用例（用例注释钉住属性名） |
| L1019 | dash_body_without_double_dash_is_usage | `send <PATH> alice "-fix flag text"` 无 `--` -> `.code(2)` usage + fix 教学 `--` + example `-- "-fix flag text"`（L1029-L1032） | 该形态在 v0.6 合法（--message allow_hyphen_values）-> exit 0 成功，用例整体翻转 | 翻转：改为 S-SEND-11 裸 `-` 开头 token（疑似误写 flag）教学用例，fix 引导 `--message` 形态 |

### 1b-C conflicts / required_unless_present 语义升级（validation exit 1 -> usage exit 2）

| 实测行号 | 测试函数 | 现行断言语义（v0.5） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L1288 | send_body_and_stdin_mutually_exclusive | 位置 body + `--stdin` 同给（L1295 args 行）-> `.code(1)` error validation | `--message` 与 `--stdin` 同给 -> clap conflicts -> usage exit 2（裁定 F2） | 翻转：exit 1 -> exit 2、validation -> usage；参数层改 `--message` + `--stdin`（S-SEND-07） |
| L1303 | send_missing_body_no_stdin_is_validation | PATH + NAME 位置、正文与 --stdin 皆缺 -> `.code(1)` error validation | `--message`/`--stdin` 皆缺 -> clap required_unless_present -> usage exit 2（裁定 F2） | 翻转：同上（S-SEND-06） |

### 1b-D 缺必填 message 列表断言（NAME 位置槽消失）

| 实测行号 | 测试函数 | 现行断言语义（v0.5） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L628 | usage_missing_required_argument_full_message | 断言 message 含 "required arguments were not provided: <NAME>"（L643 默认信封、L654 --json 信封） | NAME 位置槽消失，必填改 `--author/--message`，缺失列表文案翻转 | 翻转：断言文案随 clap 新缺失列表（`--author <AUTHOR>`/`--message <MESSAGE>` 形态）更新 |

### 1b-E 帮助面负向断言与 post create 块（flag_inventory / format-v2 删除）

| 实测行号 | 测试函数 | 现行断言语义（v0.5） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L1224 | flag_inventory_matches_spec | 负向断言 `!profile_create_help.contains("--name")` | profile create 在 v0.6 有 `--name` 必填 flag | 翻转为正向：`profile_create_help.contains("--name")` |
| L1244 | flag_inventory_matches_spec（post create 块，L1244-L1249） | `post create --help` 断言块：含 `--participants`（L1248）、不含 `--title`（L1249） | post create 随 format-v2 删除，整块失效 | 删除：L1244-L1249 块整体删除 |
| L1038 | post_create_missing_title_usage | 缺 TITLE -> `.code(2)` usage + example 断言 | post create 命令删除，用例失效 | 删除 |
| L1052 | post_create_duplicate_already_exists | 重复 post create -> `.code(1)` already-exists | post create 命令删除，用例失效 | 删除 |
| L1424 | post_group_help_lists_verbs | group help 动词清单断言含 "create"（L1430） | post create 删除，动词清单为 {send, read, summary, edit} | 改写：清单断言去除 "create" |

### 1b-F 迁移链用例的 example 跟随（断言语义保留、示例换新）

| 实测行号 | 测试函数 | 现行断言语义（v0.5） | v0.6 目标语义 | 处置 |
|---|---|---|---|---|
| L441 | usage_old_grammar_send_from | `send <PATH> --from alice body` -> `.code(2)` usage（`--from` 于 send 在 v0.6 仍非法，断言语义保留） | 语义不变，承担 v0.4->v0.5->v0.6 迁移链（S-SEND-13） | 改写：L453 example 断言随 canonical_example 换 v0.6 具名形态 |

### 1b-G example 断言跟随（canonical_example 换新引发的断言串更新，随 §3 冻结防线口径）

| 实测行号 | 测试函数 | 现行 example 断言串 | v0.6 目标 | 处置 |
|---|---|---|---|---|
| L420 | usage_missing_body_post_send | "paperwork post send standup.post.md alice" | send 规范示例换具名形态（采 --message 通道，裁定 F5） | 改写断言串 |
| L467 | usage_old_grammar_profile_create_name | "paperwork profile create agents/alice alice" | profile create 规范示例含 `--name alice` | 改写断言串 |
| L512 | usage_old_grammar_post_edit_seq | "paperwork post edit standup.post.md alice 3" | edit 规范示例具名形态（--author/--seq/--message） | 改写断言串 |

注：L482（brief add）、L497（contacts add）的 example 断言串即现行 main.rs canonical_example 已含 `--entry`/`--profile` 形态，v0.6 不变，不入翻转清单；L408/L415、L516/L523、L530/L537 等触发样例仅位置文法改写（§1 按类清单覆盖），断言语义不变。

### 1b-H main.rs 文案点位实测清单（usage_fix base 与旧 flag 教学，列入 impl_plan 步骤(3)）

| 实测行号（main.rs） | 点位 | 现行文案（v0.5） | v0.6 目标 | 处置 |
|---|---|---|---|---|
| L25 | after_help Grammar 行 | `Grammar: paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]` | 位置槽仅剩 PATH；必填与可选一律具名 flag | 改写 |
| L215 | usage_fix base 文案 | `required values are positional (PATH first; NAME second for post send/edit); see the canonical example below` | 必填改具名 flag（--author/--message/--name 等），base 重写 | 改写 |
| L219-L221 | 长未知 flag 分支旧文法教学清单 | `pre-v0.5 grammar (--from/--seq/--title/--entry/...), give its value as a positional argument` | `--seq/--title/--entry`（及 `--name/--profile`）再合法化后出列；清单收窄为仍非法 flag（如 --from 作身份）；「give its value as a positional argument」引导随之失效 | 改写 |
| L224-L227 | dash_body fix 分支（post.send/post.edit） | `if a body value starts with '-', place it after -- (e.g. ... alice -- "-fix flag text")` | `--message` 直传（allow_hyphen_values），`--` 边界教学废止 | 改写 |
| L228-L231 | 其余命令 dash 值 fix 分支 | `if a value starts with '-', place it after --` | 按 v0.6 具名 flag 口径重写 | 改写 |
| L85-L90 | dash_body 判定与 canonical_example(dash_body) 切换 | 疑似 `-` 开头 body 时 example 切 `--` 边界形态 | 直传形态下该切换废止或收窄 | 改写 |
| L279-L303 | canonical_example post send/edit 臂（L283/L288 send、L296/L301 edit，dash_body 双形态） | 位置文法 + `--` 边界形态示例 | 具名形态单一规范示例（--message 通道，F4/F5）；dash_body 双形态收敛 | 改写 |
| L309-L312 | canonical_example post create 臂 | `paperwork post create standup "Daily Standup" --participants alice,bob` | post create 随 format-v2 删除 | 删除（含 L274/L314 fallback 引用核对） |
| L317 | canonical_example profile create 臂 | `paperwork profile create agents/alice alice --model gpt-4o` | NAME 位置 -> `--name alice` | 改写 |
| L1-L7 | 文件头文法注释 | v0.5.0 grammar 描述（NAME 第二位置参数等） | v0.6 文法描述 | 改写（impl_plan 步骤(1) m-4 条款已覆盖，此处登记防漏） |

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
| send `--to` 数字串登记（已知行为） | S-SEND-16 | exit 0；"5" 写入收件人名单（F1 类型判别例外登记，非缺陷） |
| send 元数据 flag 既有线程静默忽略 | S-SEND-17 | exit 0；title 不变（F6 行为登记，本轮不改运行时行为） |
| `--author` 空值 | S-SEND-18 | `.code(1)`；`error validation:` |
| 缺 PATH（send） | S-SEND-19 | `.code(2)`；`error usage:` |
| edit 仅 `--stdin` | S-EDIT-09 | exit 0；正文逐字为 stdin 内容 |
| read total 口径与空 window | S-READ-06 / S-READ-07 | `showing: 0/4` 无 window 字段；`showing: 20/25`（过滤后口径） |
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
