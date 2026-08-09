# CLI UX 重设计 v0.5.0 — TDD（测试计划）

- 日期：2026-08-09
- 版本：v0.5.0
- 文档性质：测试改写与新增计划（对照 `repos/paperwork-cli/tests/cli_integration.rs` 现状断言结构，行号基于 v0.4.0 源码）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.5_feedbacks.md`（owner 指令）
  - `docs/dev/adr-v1.md`（ADR-011）、`docs/ssot/adr/feedbacks/v0_feedbacks.md`
  - `docs/researches/cli-ux-agent-visible-output-research-2026-08-08.md`
  - `docs/researches/ux-open-items-backlog-2026-08-08.md`
- 契约基准：本目录 `spec.md`（签名）与 `bdd.md`（场景编号 S-xxx 与本文用例一一对应）

---

## 1. 需改写的旧文法调用点（29 处）

原则：**只改参数层（flag → 位置参数），不改断言语义**；每处改写后断言仍指向输出协议（见 §2 保留清单）。

### 1.1 profile create `--name` → 位置参数 NAME

| 行号 | 现状 | 改写为 |
|---|---|---|
| L19 | `["profile","create",path,"--name","alice","--model","gpt-4"]` | `["profile","create",path,"alice","--model","gpt-4"]` |
| L52 | `["profile","create",path,"--name","x"]` | `["profile","create",path,"x"]` |
| L57 | `["profile","create",path,"--name","y"]` | `["profile","create",path,"y"]` |
| L69 | `["profile","create",path,"--name","agent"]` | `["profile","create",path,"agent"]` |
| L93-94 | `["profile","create",p1/p2,"--name","a"/"b"]` | `["profile","create",p1/p2,"a"/"b"]` |

清单外补充（现状核验新发现，同类改写）：**L39**（`profile_create_json` 中 `--name bob`）、**L297**（`contacts_create_add_read` 前置 profile `--name agent`）、**L358**（`quiet_suppresses_status_line` 中 `--name q`）。

### 1.2 post create `--title` → 位置参数 TITLE

| 行号 | 现状 | 改写为 |
|---|---|---|
| L112 | `["post","create",path,"--title","Design Discussion"]` | `["post","create",path,"Design Discussion"]` |
| L137 | `["post","create",path,"--title","T"]` | `["post","create",path,"T"]` |
| L161 | 同上（empty 用例） | 同上 |
| L177 | 同上（edit 用例） | 同上 |
| L199 | 同上（summary 用例） | 同上 |
| L325 | 同上（validate 用例） | 同上 |

### 1.3 post send `--from` → 位置参数 NAME（紧跟 PATH）

| 行号 | 现状 | 改写为 |
|---|---|---|
| L118 | `["post","send",path,"--from","alice","I think we should use Rust."]` | `["post","send",path,"alice","I think we should use Rust."]` |
| L142 | `["post","send",path,"--from","alice","--stdin"]` | `["post","send",path,"alice","--stdin"]` |
| L166 | `["post","send",path,"--from","alice","   "]` | `["post","send",path,"alice","   "]` |
| L178 | `["post","send",path,"--from","bob","original"]` | `["post","send",path,"bob","original"]` |
| L200 | `["post","send",path,"--from","x","hello"]` | `["post","send",path,"x","hello"]` |

### 1.4 post edit `--seq/--from` → 位置参数 NAME、SEQ

| 行号 | 现状 | 改写为 |
|---|---|---|
| L182 | `["post","edit",path,"--seq","2","--from","bob","edited"]` | `["post","edit",path,"bob","2","edited"]` |

### 1.5 post read `--from/--to`（保留）

| 行号 | 处理 |
|---|---|
| L188 | `["post","read",path,"--from","2","--to","2"]` **原样保留**——`--from/--to` 在 read 中语义不变（seq 范围），是规则 2 唯一语义的正面用例 |

### 1.6 brief add `--entry` / brief remove `--entry-title` → 位置参数

| 行号 | 现状 | 改写为 |
|---|---|---|
| L226 | `["brief","add",brief_path,"--entry","notes.txt"]` | `["brief","add",brief_path,"notes.txt"]` |
| L247 | `["brief","add",brief_path,"--entry","e.txt"]` | `["brief","add",brief_path,"e.txt"]` |
| L271 | `["brief","add",brief_path,"--entry","src.txt"]` | `["brief","add",brief_path,"src.txt"]` |
| L250 | `["brief","remove",brief_path,"--entry-title","e.txt"]` | `["brief","remove",brief_path,"e.txt"]` |

### 1.7 contacts add `--profile` → 位置参数

| 行号 | 现状 | 改写为 |
|---|---|---|
| L306 | `["contacts","add",contacts_path,"--profile",profile_path]` | `["contacts","add",contacts_path,profile_path]` |

### 1.8 brief create `--title` → 位置参数 TITLE（rework 轮补漏，可行性评审 M-1）

| 行号 | 现状 | 改写为 |
|---|---|---|
| L220 | `["brief","create",brief_path,"--title","My Brief"]` | `["brief","create",brief_path,"My Brief"]` |
| L246 | `["brief","create",brief_path,"--title","B"]` | `["brief","create",brief_path,"B"]` |
| L270 | `["brief","create",brief_path,"--title","V"]` | `["brief","create",brief_path,"V"]` |

注意：L300 `contacts create --title` 因 spec §2 保留 flag（有默认值、属可选），确属不改，勿误伤。

**合计**：leader 清单 23 处 + 补充 3 处（L39/L297/L358）+ rework 补漏 3 处（L220/L246/L270）= **29 处改写** + L188 1 处保留。改写时须同步核对每处测试的 `.args(...)` 之外无其他旧文法残留。

---

## 2. 必须原样保留的输出协议断言清单（输出协议冻结防线）

以下断言**一字不改**（行号为 v0.4.0 源码）：

| 行号 | 断言内容 | 冻结依据 |
|---|---|---|
| L22-23 | stdout 含 `ok profile.create`、`name: alice` | ok 信封首行 + 字段 |
| L28-30 | stdout 含 `ok profile.show`、`name: alice` | 同上 |
| L42-43 | JSON 含 `"name":"bob"`、`"status":"ok"` | JSON key 不变 |
| L59-60 | failure + stderr 含 `error already-exists:` | 错误分类词 |
| L77-78 | stdout 含 `ok profile.edit`、`changed: model` | ok 信封 + changed 字段 |
| L84 | JSON 含 `claude-3` | JSON 字段值 |
| L100-101 | JSON 含 `a.profile.md`、`b.profile.md` | list 输出文件名 |
| L115 / L121 / L127-128 | `ok post.create`、`ok post.send`、`ok post.read` + body 内容 | ok 信封 command 标识 |
| L146 / L152 | `ok post.send`、stdin 正文回读 | stdin 行为 |
| L168-169 | failure + stderr 含 `error validation:` | 空正文分类 |
| L185 / L191 | `ok post.edit`、edit 后正文 | edit 信封 |
| L206 | JSON 含 `"messages":2` | summary JSON key |
| L223 / L229 / L235 | `ok brief.create`、`ok brief.add`、条目回读 | brief 信封 |
| L253 / L259 | `ok brief.remove`、JSON `"entries":[]` | remove 信封 |
| L278 / L286 | verify JSON 含 `fresh`、`shifted` | 三态判定词 |
| L303 / L309 / L315 | `ok contacts.create`、`ok contacts.add`、read 含 `agent` | contacts 信封 |
| L333 / L346 | `ok validate`、`error format:` | validate 信封 |
| L352-363 | `-q` 隐状态首行但字段保留（`name: q` 且无 `ok`） | -q 语义 |
| L366-370 | 运行时错误 exit code = 1 | 退出码语义 |
| L374-381 | `--json` 错误输出到 stdout，含 `"status":"error"`、`"exit_code":1` | JSON 错误形态 |

注意：L358 的参数层需按 §1.1 改写（`--name q` → 位置参数），但 L361-362 的**断言原样保留**。

---

## 3. 新增用例清单

| 用例 | 对应 BDD | 断言要点 |
|---|---|---|
| usage 信封与 exit 2：仅 PATH 缺必填位置参数（`post send <path>`） | S-SEND-08 | `.code(2)`；stderr 含 `error usage:`；含 `example:` 行（规范形态、具体值） |
| NAME/BODY 混淆面：PATH+单字符串→validation | S-SEND-12 | `.code(1)`；stderr `error validation:`；example 含 NAME 槽完整命令形态；fix 含 `--` 用法（F1 裁定：不可实现 usage exit 2） |
| usage 信封与 exit 2：旧文法 `--from alice`（send） | S-SEND-09 | `.code(2)`；stderr `error usage:`；example 含 post send 规范形态示例（不携带用户原参数值，F2 裁定） |
| usage 信封与 exit 2：旧文法 `profile create --name` / `brief add --entry` / `contacts add --profile` / `post edit --seq` | S-PROF-03 / S-BRIEF-03 / S-CONTACTS-03 / S-EDIT-04 | `.code(2)` + `error usage:`；example 含对应命令规范形态示例 |
| usage 信封 exit 2：SEQ 非数字 | S-EDIT-03 | `.code(2)`；`error usage:` |
| usage 信封 exit 2：多余位置参数（send 四参） | S-SEND-13 | `.code(2)`；`error usage:` |
| `--json` usage 错误 | S-OUT-03 | `.code(2)`；stdout 单行 JSON 含 `"category":"usage"`、`"command"`、`"example"`、`"exit_code":2`（如实反映进程退出码） |
| `--json` 运行时错误带 command 字段 | S-OUT-02 | stdout JSON 错误对象含 `"command":"post.read"`（既有断言 L374-381 之上增量） |
| 顶层解析失败 command 标识为 usage | S-OUT-06 | 组/动词层失败（如缺子命令）时 stderr 信封 `error usage: ...` 且 JSON `"command":"usage"`，exit 2 |
| --help/-V 穿透冻结（F5） | S-OUT-07 | `--help`（含子命令层）与 `-V` 均 `.code(0)`，stdout 含 clap 帮助/版本输出；不进 usage 信封 |
| 原路径优先：存在的 `x.md` 不被改写 | S-PATH-01 | 先写裸 `x.md` 合法线程，`post read x.md` exit 0 且读到内容 |
| 补后缀回退：`standup` → `standup.post.md` | S-PATH-02 | 仅存在带后缀文件时裸名可读 |
| 三级解析：x.md 与 x.post.md 同时存在用 x.md | S-PATH-05 | 两者并存时读写命中原路径 x.md |
| 三级解析：send 自动创建落点（第③级路径决策） | S-PATH-06 | 原路径与补后缀路径均不存在时 send 创建补后缀路径 |
| 第①级命中异型文件 → format 不改道（F4） | S-PATH-07 | 非线程 `notes.md` 存在时 send 报 `error format:` exit 1，**不**创建 notes.post.md |
| 传入目录路径（is_file() 判据，F4） | S-PATH-08 | 已存在目录不命中第①级，read 报 not-found exit 1，不创建文件 |
| implicit-mention 字段（单数） | S-SEND-03 | send --reply-to 后 stdout 字段区含 `implicit-mention:`（单数，additive，仅触发时出现）；`--json` 含同名 key；未触发时不出现该字段 |
| implicit-mention 不触发边界 | S-SEND-10b / S-SEND-11 | 自回复、已显式 mention、reply-to 不存在三种情形均不出现 `implicit-mention` 字段 |
| read 窗口字段恒显（字段区形态） | S-READ-01 / S-READ-02 / S-READ-06 | 恒显 `showing: n/total`；未超限含 `window: #1-#6`；超限含 `showing: 20/50` 与 `window: #31-#50`；空线程含 `showing: 0/0` 但**不**含 window |
| 过滤 + limit 组合的 total 口径（F3） | S-READ-07 | 50 条线程 alice 25 条：`--mention alice --limit 20` → `showing: 20/25`（非 20/50） |
| validate --type | S-VAL-02 / S-VAL-03 / S-VAL-05 / S-VAL-06 | 非类型后缀 + `--type post` exit 0；无 `--type` 时 `error format:`；`--type bogus` → usage exit 2；`x.profile.md --type post` → format exit 1 |
| `--` 边界：`-` 开头 body | S-SEND-07 / S-EDIT-05 | send 与 edit 正文逐字含 `-` 开头文本，exit 0 |
| `--` 边界负形态：`-` 开头 body 未加 `--`（NF-2 补录） | S-SEND-14 | `.code(2)`；stderr `error usage:`（clap 把 `-fix` 当未知 flag）；fix 提示 `--` 边界；example 示范 `--` 用法形态（预置静态示例） |
| post create 缺 TITLE（usage）（NF-3 补录） | S-CREATE-02 | `.code(2)`；`error usage:`；example 为 post create 规范形态示例（含 TITLE 槽具体值） |
| post create 重复（already-exists）（NF-3 补录） | S-CREATE-03 | 先 create 成功后再 create：`.code(1)`；stderr 含 `error already-exists:` |
| profile create 缺 NAME（usage）（NF-3 补录） | S-PROF-02 | `.code(2)`；`error usage:`；example 形如 `paperwork profile create agents/alice alice --model gpt-4o` |
| read 旧语义误用：`--from` 传身份值（NF-3 补录） | S-READ-04 | `.code(2)`；`error usage:`（--from 只接受 u64）；example 示范 seq 范围用法 |
| brief add/remove basename 映射（NF-3 补录） | S-BRIEF-07 | add `src/main.rs` 后 remove `main.rs` 两步均 exit 0（存储标题为 basename）；remove 传原路径 `src/main.rs` 则 `.code(1)` + `error not-found:` |
| contacts create title 位置化误用（usage）（NF-3 补录） | S-CONTACTS-05 | `.code(2)`；`error usage:`（多余位置参数）；`--help` 输出含 title 为可选 flag（默认 Contacts）的注记 |
| po 隐藏别名 | S-ALIAS-01 | `po read` 等价 `post read`；`--help` 不出现 po |
| 命名政策白名单断言（SOTA C6 采纳） | — | `--help` 输出的组/动词集合精确等于 {profile,post,brief,contacts,validate}（含隐藏别名不出现断言）；全 CLI flag 名集合与 spec §2 全表一致 |

---

## 4. ops_tests.rs 零改动声明

`repos/paperwork-core/tests/ops_tests.rs` **一行不改**。理由与防线作用：

- core 公开 API 零变更（spec §6），ops_tests 全部用例应当原样通过；
- core 层唯一改动是 CLI 文法 example 字符串 **14 处**（rework 轮实测盘净，纯文案，完整清单与检索命令见 impl_plan.md 步骤④），不触及锁/seq/格式/hash 逻辑；
- ops_tests 因此成为「core 行为未被文法重设计污染」的回归防线：任何失败都意味着改动越界。

---

## 5. 测试语料目录约定

- **沿用 test-v04/ 先例**：cli_integration.rs 自身使用 TempDir，不依赖仓库内语料目录；仓库内 `test-v04/` 是人工实测/评审用样例集。
- **必要时新建 `test-v05/`**：若 QA Review Book（impl_plan.md 步骤⑨）需要新文法冒烟样例或 usage 信封演示样例，按 test-v04/ 结构复制为 test-v05/（含正常样例 + 刻意损坏样例），不得改动 test-v04/ 存量文件。
- `_fix/` 目录为历史修复样例，不纳入本次测试范围。

---

## 6. 验证门禁

1. **分阶段门禁（F6 裁定，与 impl_plan 全局门禁一致）**：步骤①~④期间 `cargo build` + paperwork-core 测试（ops_tests 恒绿）+ clippy 全绿即可推进，cli_integration 允许红；步骤⑤完成后 `cargo test`（workspace 全量）全绿为硬门禁——ops_tests 零改动通过是 core 未越界的证明；cli_integration 全部改写与新增用例通过是文法落地的证明。
2. `cargo clippy --all-targets -- -D warnings` 无警告。
3. 实测冒烟（tdd §3 全部场景 + 并发 send seq 无间隙）由 review/gate 阶段执行，impl agent 不运行长时 e2e（MainAgent工作编排.md 审查条款）。
