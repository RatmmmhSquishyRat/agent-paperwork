# contacts CRUD 轮实施代码评审 —— 完整性（需求覆盖）视角

- 日期：2026-08-09
- 评审视角：completeness（需求覆盖；正确性与影响面由专人另行评审，本文不越界）
- 评审对象：worktree agent-paperwork-wt-v06grammar，分支 cli-grammar-v0.6，基线 0f6c384 之后 6 个提交（77ab558..e7eb049，HEAD=e7eb049 实测）
- 需求基准：主工作区 docs/ssot/specs/cli-grammar-v0.6/ 的 spec.md 本轮增量（§2 三行、§3.5、§3.6、§3.9、§4）、bdd.md（S-BRIEF-07~09、S-CONTACTS-06~14、S-LOCK-01~03、S-SHORT-02）、tdd.md §8、impl_plan.md R1-R6、v0.7_feedbacks.md owner 指令面
- 事实依据：diff 逐行阅读（12 文件 +1397/-153）；实测 ops_contacts_crud_tests 15/15、cli_integration 129/129、ops_tests 51/51（ops_tests 经 diff --stat 确认零改动，防线成立）

## 一、bdd 场景逐条映射（S-BRIEF-07~09 / S-CONTACTS-06~14 / S-LOCK-01~03 / S-SHORT-02）

| 场景 | 实现落点（worktree 实测） | 测试落点（实测） | 覆盖判定 |
|---|---|---|---|
| S-BRIEF-07 选择性详情 | cmd/brief.rs entry_title 过滤 + detailed=full 或 entry-title 命中（Default/JSON 同口径）；conclusion 恒为全量条目数 | cli_integration.rs L2907 brief_read_entry_title_selective_details | 全覆盖 |
| S-BRIEF-08 无匹配 not-found | cmd/brief.rs NotFound resource "Brief entry"，fix 引导 brief read | L2936 | 全覆盖 |
| S-BRIEF-09 与 --full 组合 | cmd/brief.rs 组合合法等价；未给 filter 时 TOC/--full 冻结（既有用例回归通过） | L2951 | 全覆盖 |
| S-CONTACTS-06 remove 成功 | cmd/contacts.rs Remove 臂 + core contacts_remove；ok 首行/contacts/removed 字段齐备 | L2616 | 全覆盖（断言强度见 m-2） |
| S-CONTACTS-07 remove 未命中+label-as-key | core contacts_remove fix 含键口径教学句；两形态同判 | L2648（ghost 与 alice 双键循环） | 全覆盖 |
| S-CONTACTS-08 update 成功 | core contacts_update 原地替换+R11 重派生+顺序保留；updated 箭头串 | L2671 | 全覆盖 |
| S-CONTACTS-09 update 错误路径 | OLD 未命中 NotFound 先于 NEW 已存在 AlreadyExists；均无写入 | L2713 | 全覆盖 |
| S-CONTACTS-10 缺必填 flag usage | clap 必填判定；main.rs canonical_example 两臂（L400/L404）逐字钉住 | L2746 三形态+逐字 example 断言 | 全覆盖 |
| S-CONTACTS-11 旧文法位置参数+幂等回归 | PATH 唯一位置槽；add 幂等语义不变 | L2779 + ops_contacts_crud_tests.rs contacts_add_idempotent_after_lock_migration | 全覆盖 |
| S-CONTACTS-12 删最后条目形态 | serialize_contacts(title, &[]) 与 create 初态同形 | L2796 + core contacts_remove_last_entry_matches_create_initial_shape | 全覆盖 |
| S-CONTACTS-13 特殊字符路径往返 | 键=未转义原串；angle-bracket 序列化；二次操作仍命中 | L2827 + core remove_update_roundtrip_with_special_character_paths | 全覆盖 |
| S-CONTACTS-14 NEW 不存在静默成功 | exit 0、destination 原值落盘、label 回退主干；known silent surface | L2865 + core contacts_update_nonexistent_new_is_silent_success | 全覆盖（read 回显子句见 m-3） |
| S-LOCK-01 多进程并发写不丢失 | 六写路径锁内读改写 | L3002（N=8 双侧、预创建 entry 目标文件、集合口径、validate 合法） | 全覆盖 |
| S-LOCK-02 profile edit 并发串行化 | edit_profile 锁内读改写 | L3083（主形态：不重叠字段并集，变体选择已在用例注释写清，符合 Daniel M-1 二选一要求） | 全覆盖 |
| S-LOCK-03 fast fail 无降级（代码级不变量） | ops/lock.rs：lock_exclusive 失败即 IoContext exit 1，fix 沿用 thread 既有文案；无任何无锁降级写入分支；错误路径先 unlock 再返回 | bdd 明示以 code review + 点位盘点断言为准，不强制集成模拟；本评审盘点确认（见三-2） | 全覆盖 |
| S-SHORT-02 白名单断言 | 短形式集 {-a,-m,-q} 冻结；26 项无短形式负向清单新建/扩展；contacts 组动词断言新建；ASCII 清单补两动词 | L2449（27 探针，含 --new-profile -N/-w 双探针）、L2552（含 edit/delete/list 反向断言）、L2581 | 覆盖（read 侧 --reply-to 逐 flag 探针缺位见 m-1） |

**映射结论：无任何「无实现」或「无测试」的 bdd 场景。** 全部新用例实测通过（cli_integration 129/129、ops_contacts_crud_tests 15/15）。

## 二、spec 钉住的逐字文案核对（全部逐字落实）

| 钉住文案 | spec 出处 | 代码落点（实测） | 判定 |
|---|---|---|---|
| remove 规范示例 `paperwork contacts remove team.contacts.md --profile alice.profile.md` | §5 第 2 条 / §3.6（Ryan m-2） | main.rs L400 canonical_example 臂；测试 L2746/L2779 逐字断言 | 逐字一致 |
| update 规范示例 `paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md` | §5 第 2 条 / §3.6 | main.rs L404；测试 L2746 逐字断言 | 逐字一致 |
| 键口径教学句 `the key is the profile path as stored in the contacts file, not the label`（纯 ASCII） | §3.6（Ryan m-3） | ops/contacts.rs L111（remove）与 L151（update）fix 内嵌；测试 L2648/L2713 断言 | 逐字一致，remove/update 双侧均带 |
| updated 箭头串 `<OLD> -> <NEW>`（单空格三段拼接） | §3.6（Ryan m-4 定案） | cmd/contacts.rs update 臂 conclusion 与 updated 字段同一 format!；测试 L2671 逐字断言两种形态（常规与 NEW 不存在回显原值） | 逐字一致 |
（注：上行的逐字断言实为 L2671 常规形态 + L2865 NEW 不存在回显原值形态，两处合起来覆盖双形态。）
| fix 教学句（brief read 无匹配引导 `brief read <PATH>` 列出条目） | §3.5 | cmd/brief.rs fix `run paperwork brief read {path} to list entries`；测试 L2936 断言 | 覆盖一致 |
| not-found example 形态 `paperwork contacts read <PATH>`（PATH 取实际解析值） | §5 第 2 条 | ops/contacts.rs remove/update NotFound example 字段均为 `paperwork contacts read {path}`（ensure_suffix 解析后路径，与 brief remove 既有先例同口径） | 覆盖一致 |

## 三、治理文档逐项核对

### 1. tdd §8
- §8.1 core 用例表（15 行）：ops_contacts_crud_tests.rs 15 个 test fn 与表中 15 行逐行对应（remove 命中/未命中/文件不存在、update 命中重派生/label 回退/OLD 未命中/文件不存在/NEW 已存在/OLD==NEW 判定顺序双支/NEW 不存在静默、删末条形态、特殊字符往返、add 幂等回归、锁内序列化等价、多线程并发），实测 15/15 全绿。
- §8.2 cli 用例表（16 行）：逐行对应第一节映射表，无缺行；label-as-key 触发形态（S-CONTACTS-07 And 段）并入 contacts_remove_miss_and_label_as_key_are_not_found 覆盖。
- §8.3 白名单更新五项：① 26 项负向清单新建/扩展（由 6 个一次性探针扩容，口径为新建而非追加，与 rework 修订一致）；② contacts 组动词断言新建（仿 post_group_help_lists_verbs 先例，含反向断言）；③ 短形式集 {-a,-m,-q} 断言不变；④ 组集合断言不变；⑤ ASCII 动词清单补 contacts remove/update 两行。五项全部落实。
- §8.4 ops_tests.rs 零改动防线：diff --stat 确认该文件不在 6 个提交的变更面内（字节级零改动），实测 51/51 恒绿；新 core 测试全部落独立文件 ops_contacts_crud_tests.rs，未并入 ops_tests.rs，符合禁令。
- §8.5 测试语料：test-v03/v04/v05 未被触碰（diff 不含）；test-v06/ 为条件条款（「若 QA 需要」），未新建不构成缺口。
- §8.6 门禁：第 4 条锁点位盘点由本评审复核——lock_exclusive 点位 = thread.rs L94/L366（既有冻结）+ ops/lock.rs L47（helper），六写路径全部经 helper 进入锁内读改写；contacts.rs/manifest.rs/profile.rs 中残留的三处 fs::write 均属 create 路径（新建文件，不在六写路径清单内），无无锁 read-modify-write 残留；第 5 条不发布约束满足（无 bump/tag/publish/CHANGELOG 发布段，Cargo.toml 与 CHANGELOG.md 零改动）。

### 2. impl_plan R1-R6
- R1 锁 helper：ops/lock.rs 独立落点（spec 治理授权的实施方自决范围）；六步模板完整（开 read+write 句柄 -> lock_exclusive -> 经持锁句柄 seek(0)+read_to_string -> 变更 -> set_len(0)+seek(0)+write_all -> unlock）；错误路径先 unlock 再返回；锁获取失败 fast fail 落 IoContext，fix 沿用 `another process may hold the lock; retry shortly`；Windows 持锁句柄读取约束在注释钉住；无 temp+rename（崩溃窗口判例沿用）。全部满足。
- R2 contacts 三写路径：contacts_add 改造为锁内读改写（幂等语义保持，core 测试钉住字节级 no-op）；contacts_remove/contacts_update 新增，键口径/错误 category/重派生/顺序保留均按契约；序列化复用 serialize_contacts，格式零触碰。满足。
- R3 brief/profile 补锁：brief_add_entry（manifest.rs L87）、brief_remove_entry（L148）、edit_profile（profile.rs L98）三处改为 helper 锁内读改写；行为与序列化产物不变（锁内序列化等价用例实测通过）。满足。
- R4 CLI 接线：两新动词接线（--profile 必填、update 另 --new-profile 必填、均仅长形式）；brief read --entry-title 可选过滤（命中按 --full 档字段、Default/JSON 同口径、无匹配 not-found）；command id contacts.remove/contacts.update；ok 信封字段 contacts/removed/updated；键口径教学句；canonical_example 两条逐字补齐；main.rs VALUE_TAKING_FLAGS 同步收录 --new-profile（usage 信封解析面配套）；新文案纯 ASCII（ASCII 用例实测覆盖）。满足。
- R5 测试落地：tdd §8.1/§8.2/§8.3 全部用例落地（见三-1）；workspace 全量实测通过（硬门禁口径：ops_tests 51/51、ops_contacts_crud_tests 15/15、cli_integration 129/129）。满足。
- R6 文档面：根 README.md 补 brief read --entry-title 与 contacts remove/update 示例，附键口径说明与 update/edit 区分注记；SKILL.md 速查表同步（brief 选择性详情+miss 行为、contacts remove/update+键口径+update/edit 区分注记）。满足。
  - 注：R6 文件清单含 repos/paperwork-cli/README.md，实测该文件仅含组级一句话描述与 post/profile Quick Example，无任何 contacts/brief 动词级示例面，本轮无文案需同步，未改动属合理处置（点名备查）。

### 3. v0.7_feedbacks owner 指令面
- 指令 (1) contacts 完整 CRUD 两动词：contacts remove / contacts update 全链路落地（core API + CLI + command id + 信封字段 + 测试 + 文档），无缺项。
- 指令 (1) 锁 + fast fail：六写路径（contacts add/remove/update、brief add/remove、profile edit）全部锁内读改写，fast fail 无降级（见三-1 §8.6 盘点）。
- 指令 (2) 渐进阅读登记：唯一真实缺口「brief 按条目选择性看详情」以 brief read --entry-title 补齐（渐进阅读第三档），post/profile/contacts 三面登记结案无需代码变更；spec §9 登记结案条款与实现一致。
- §2.5 SOTA C6 白名单扩容：update 动词纳入 contacts 组动词集合并由白名单断言钉住（组动词断言含 update、反向断言含 edit/delete/list）；update/edit 语义分工教学随 README/SKILL 披露（§2.5 第 3 条要求）。落实。

### 4. 过度实现核查（文档范围外实现）
- 未发现超出文档范围的实现。逐项核验：ops/lock.rs 独立 pub 模块为 R1 明确授权的落点自决（v0.7_feedbacks §2.3）；main.rs VALUE_TAKING_FLAGS 收录 --new-profile 属 usage 信封机制的必要配套；contacts remove/update 的 after_help 教学注记（键口径、update/edit 区分）均有 spec §3.6 教学条款与 R6 披露要求背书；cmd/core 模块 doc comment 更新属 impl_plan 步骤(1) 同源文案刷新口径。
- ASCII 门禁口径注记：tdd §8.6(3) 行文枚举「ok/usage/not-found/already-exists/io 各形态」，实际测试面按 tdd §8.2 新增用例表（权威口径）覆盖 usage/not-found/already-exists 三态（contacts_crud_error_envelopes_are_ascii，含 stdout+stderr 双侧断言）；ok 信封不走 stderr、io 形态集成层不可确定触发，§8.6(3) 为枚举性行文而非超出 §8.2 的额外用例要求，不构成缺口（点名备查）。

## 四、发现清单

### Critical Issues（MUST FIX）
无。

### Warnings（SHOULD FIX）
无。

### Suggestions（CONSIDER）

#### m-1 S-SHORT-02 逐 flag 负向清单缺 post read 侧 --reply-to 探针
[cli_integration.rs#L2486-L2541](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-cli/tests/cli_integration.rs)
- 问题：bdd S-SHORT-02 要求「spec §4 其余全部 flag 行枚举的全量清单**逐一断言**无短形式」，清单明列「post send/read **两侧** --reply-to」（26 项口径中该项计 1 项、要求两侧覆盖）。实测负向探针清单 27 条覆盖 26 个 flag，其中 --reply-to 仅有 send 侧探针（L2537-2538，`post send ... -r 1`），无 read 侧探针（`post read <PATH> -r 1`）。风险已被 L2467-2471 的 post read help 短形式精确集合断言（恰为 [-h,-q]）系统性兜底，故降为建议级，但逐 flag 清单的字面完整性差一项。
- 建议：探针数组补一行 `vec!["post", "read", path.to_str().unwrap(), "-r", "1"]`（期望 usage exit 2），使 26 项逐 flag 负向清单与 bdd 枚举逐字对齐。

#### m-2 S-CONTACTS-06 ok 首行逐字形态在测试中仅以片段断言
[cli_integration.rs#L2616-L2645](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-cli/tests/cli_integration.rs)
- 问题：bdd S-CONTACTS-06 与 tdd §8.2 将 ok 首行钉为 `ok contacts.remove <profile> -> <path>`；contacts_remove_success 实测仅断言 `contains("ok contacts.remove")`、`contains("alice.profile.md ->")`、`contains("contacts: ")` 三个片段（L2625-2627），未逐字钉住完整首行（箭头串尾部的 contacts 路径、首行整体性未断言）。实现侧 conclusion 形态正确（cmd/contacts.rs），且 update 侧对偶用例（L2691）已逐字断言完整首行，两动词断言强度不对称。场景有实现、有测试，仅断言粒度弱于钉住文案，故为建议级。
（勘误：上行「update 侧对偶用例 L2691」应为 L2690 逐字断言 ok 首行、L2691 断言 updated 字段。）
- 建议：remove 用例补一条完整首行断言，如 `.stdout(predicate::str::contains(&format!("ok contacts.remove alice.profile.md -> {}", path.display())))`，并把 `contacts` 字段值一并逐字断言，与 update 侧断言强度对齐。

#### m-3 S-CONTACTS-14 Then 段的 contacts read 回显子句未被任何用例断言
[cli_integration.rs#L2865-L2884](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-cli/tests/cli_integration.rs)
- 问题：bdd S-CONTACTS-14 Then 末句要求「下一轮 contacts read 该条目显示 (unreadable) 类容错形态」；CLI 用例 contacts_update_nonexistent_new_is_silent_success 止步于落盘内容与 updated 回显断言，未做后续 contacts read 回显断言（core 侧对偶用例同样未含）。该子句落在 contacts read 既有富化输出的冻结行为面（S-CONTACTS-05），且 tdd §8.2 该用例行的断言要点未列此子句，故不构成实施缺项，仅为 bdd 场景文本与测试面的可追溯性缝隙。
- 建议：在该用例尾部追加一次 `contacts read`（Default 或 --json）断言，确认 carol 条目呈 unreadable 容错形态，闭合 S-CONTACTS-14 全部 Then 子句。
（实测佐证：cmd/contacts.rs L187-195 enrich_profile 对不可解析 profile 返回 "(unreadable)"，该回显形态实现侧在场，仅测试面未闭环。）

## 五、总判定

**判定：通过（完整性维度验收成立）。**

- 统计：C = 0，M = 0，m = 3。
- 依据：本轮全部需求面——spec §2 三行新增签名、§3.5 brief read --entry-title、§3.6 contacts remove/update 全部契约（含 NEW 不存在静默面、键口径、updated 箭头串逐字格式、判定顺序）、§3.9 六写路径锁语义与阻塞行为契约、§4 --new-profile 长形式约束；bdd S-BRIEF-07~09 / S-CONTACTS-06~14 / S-LOCK-01~03 / S-SHORT-02；tdd §8 三层用例表与 ops_tests 零改动防线；impl_plan R1-R6；v0.7_feedbacks owner 指令三面（CRUD 两动词、六写路径补锁、渐进阅读登记）——逐条核对均有实现且有测试钉住，spec 钉住的四处逐字文案（两条 canonical example、键口径教学句、updated 箭头串）在代码中逐字落实，README/SKILL.md 同步全部新文法并含 update/edit 区分注记，未发现过度实现或文档要求而代码缺失的项。

## 六、最严重三条摘要

1. m-1：S-SHORT-02 逐 flag 负向清单缺 post read 侧 --reply-to 探针（cli_integration.rs L2486-L2541 仅 send 侧），已被 read help 短形式精确集合断言系统性兜底，属字面完整性瑕疵。
2. m-2：S-CONTACTS-06 ok 首行在 contacts_remove_success（L2616-L2645）中仅以三个片段断言，未逐字钉住完整首行，与 update 侧 L2690 逐字断言强度不对称。
3. m-3：S-CONTACTS-14 Then 末句「下一轮 contacts read 显示 (unreadable) 容错形态」无任一用例断言；实现侧在场（cmd/contacts.rs L187-195），属 bdd 文本与测试面的可追溯性缝隙。

三条均为建议级，不阻塞验收。

---

- 评审人：完整性视角评审 agent（Experts Mode，任务 ID:20 子项）
- 约束遵守：本次评审仅创建本文件，未修改任何代码/文档，未做任何 git 提交；所有行号均按 worktree HEAD=e7eb049 实测。

---

## 修复回应销账段（2026-08-09，编排层裁定 F1-F7 落实）

| 发现 | 处置 | 销账证据 |
|---|---|---|
| m-1 read 侧 --reply-to 探针缺位 | 已销账（F7 Ray m-1） | 探针数组补 `vec!["post", "read", path.to_str().unwrap(), "-r", "1"]`（期望 usage exit 2），26 项逐 flag 负向清单与 bdd S-SHORT-02 枚举逐字对齐（现 28 条探针覆盖 26 个 flag，--new-profile 双探针与 --reply-to 双侧探针均在列），实测通过 |
| m-2 remove ok 首行断言强度不对称 | 已销账（F7 Ray m-2） | contacts_remove_success 改为逐字断言完整首行 `ok contacts.remove alice.profile.md -> {path}` 与 `contacts: {path}` 字段值，与 update 侧断言强度对齐，实测通过 |
| m-3 S-CONTACTS-14 read 回显子句未断言 | 已销账（F7 Ray m-3） | contacts_update_nonexistent_new_is_silent_success 尾部追加 contacts read 断言：`ok contacts.read 1 contacts` + `carol: (unreadable)` 容错形态，S-CONTACTS-14 全部 Then 子句闭合，实测通过 |

修复后验证：`cargo test --workspace` 274 全绿；clippy `-D warnings` 零警告；新增护栏用例见 S-BRIEF-10/S-CONTACTS-15（spec/bdd 已同步登记，F1/F3 销账见 correctness 报告同名段）。
