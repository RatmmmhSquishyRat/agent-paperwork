# CLI 文法 v0.6: TDD（测试计划）

- 日期：2026-08-09
- 版本：v0.6（本轮不发布）
- 文档性质：测试改写与新增计划（对照 `repos/paperwork-cli/tests/cli_integration.rs`）
- architectural basis：
  - `docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令）
  - `docs/dev/adr-v1.md`（ADR-011）、`docs/ssot/adr/feedbacks/v0_feedbacks.md`
- 契约基准：本目录 `spec.md`（签名）与 `bdd.md`（场景编号 S-xxx 与本文用例一一对应）

---

## 0. 基线与行号约定

- 测试基线为 **cli-ux-v0.5 分支 + format-v2 工作树变更的合并结果**（v0.5 位置文法 + post create 删除 + send 建线程载荷已就位）；本文行号以合并后基线实测为准，实施第一步即执行全量盘点（检索命令见 §2 末）。
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
| 缺 `--message` 且无 `--stdin`（send/edit） | S-SEND-06 / S-EDIT-04 | `.code(2)`；`error usage:`；example 展示二选一完整形态 |
| 缺 `--seq`（edit） | S-EDIT-03 | `.code(2)`；`error usage:`；example 含 `--seq` 完整形态 |
| `--message` 与 `--stdin` 同给（send/edit） | S-SEND-07 / S-EDIT-05 | `.code(2)`；`error usage:`（clap conflicts）；无文件写入 |
| 短形式与全称等价 | S-SEND-02 / S-SHORT-01 | `-a/-m` 与 `--author/--message` 行为逐字等价；spec §4 全表逐 flag 等价抽查 |
| v0.5 位置文法迁移（send/edit/profile/brief/contacts） | S-SEND-12 / S-EDIT-08 / S-PROF-03 / S-BRIEF-04 / S-CONTACTS-04 | `.code(2)`；`error usage:`（多余位置参数）；example 为对应命令 v0.6 规范形态（不携带用户原参数值） |
| v0.4 旧 flag 迁移链延伸 | S-SEND-13 | `--from` 于 send 不存在 -> `.code(2)` + `error usage:` + v0.6 规范示例 |
| 混淆面消亡确认 | S-SEND-15 | `send <PATH> "text"` -> `.code(2)` usage（不再是 v0.5 的 validation exit 1）；无文件写入；确认静默写入路径不可达 |
| `--message` 值以 `-` 开头直传 | S-SEND-10 | exit 0；正文逐字含 `-` 开头文本；**无** `--` 边界 |
| 裸 `-` 开头 token 教学 | S-SEND-11 | `.code(2)`；`error usage:`；fix 引导 `--message` 形态；example 为 `--message "-fix flag text"` 形态 |
| `--mention` 无短形式 | S-READ-04 | `read -m alice` -> `.code(2)` usage；`--mention alice` exit 0 |
| profile create 缺 `--name` | S-PROF-02 | `.code(2)`；`error usage:`；example 含 `--name` 完整形态 |
| brief 三命令缺必填 flag | S-BRIEF-03 | 三条 `.code(2)` usage；example 分别含 `--title/--entry/--entry-title` |
| contacts add 缺 `--profile` | S-CONTACTS-03 | `.code(2)`；`error usage:`；example 含 `--profile` |
| SEQ 非数字 | S-EDIT-06 | `.code(2)`；`error usage:` |
| 空正文（`--message "   "`） | S-SEND-09 | `.code(1)`；`error validation:` |
| ASCII 契约回归 | S-OUT-05 | 新增 usage 形态（缺必填 flag、conflicts、多余位置参数）纳入 stderr 逐字节 ASCII 断言 |
| 命名政策白名单 | S-SHORT-02 | 组/动词集合精确等于 {profile,post,brief,contacts,validate}；flag 与短形式集合与 spec §4 一致；`--mention` 无短形式负向断言 |
| 冻结回归抽查 | S-READ-01~03 / S-SUM-01 / S-PATH-* / S-ALIAS-* / S-OUT-01~04 | v0.5 既有对应用例改参数层后断言原样通过（showing/window/implicit-mention/三级解析/别名/三档输出） |

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
