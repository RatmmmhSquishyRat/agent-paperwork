# contacts CRUD 轮代码评审 — 影响面（回归/破坏性变更）

- 日期：2026-08-09
- 评审人：impact 维度评审 agent（Paul 视角）
- 评审对象：worktree agent-paperwork-wt-v06grammar，分支 cli-grammar-v0.6，基线 0f6c384 之后 6 个提交（77ab558..e7eb049，HEAD=e7eb049）
- 评审范围：仅影响面（回归/破坏性变更）。完整性与正确性维度另有专人负责，本报告不重复覆盖。
- 证据方式：git diff 0f6c384..e7eb049 全量逐文件核读；基线文件（git show 0f6c384:...）与 HEAD 文件逐项对照；git diff master 实测。

---

## Critical Issues (MUST FIX)

无。

成功路径信封、JSON key、七类 category、退出码、post/validate 零触碰、发布纪律、合并面均实测干净（详见文末"逐项核查结论"）。未发现任何会破坏既有成功行为或下游消费者的确定性回归。

---

## Warnings (SHOULD FIX)

### M-1 补锁后 io 失败路径信封 fix 文案漂移（既有信封文本冻结被突破）

位置：repos/paperwork-core/src/ops/lock.rs#L43、#L89、#L107；波及四个迁移调用点 ops/contacts.rs#L66、ops/manifest.rs#L87、ops/manifest.rs#L148、ops/profile.rs#L98

问题：基线中 contacts add / brief add / brief remove / profile edit 的写失败统一经 fs::write 映射为 IoContext，fix 文案固定为 "check that the target path is writable"（基线 contacts.rs L87，manifest.rs、profile.rs 同文）。该 fix 经 main.rs 的 pw_err.fix() 进入 error 信封正文（main.rs L132-L139），属 agent 可见输出。迁移到 locked_read_modify_write 后，同场景文案变为：

- open(read+write) 失败 → "check file permissions"（lock.rs L43）
- set_len 失败 → "check file permissions"（lock.rs L89）
- write_all 失败 → "check disk space and file permissions"（lock.rs L107）

category（io）与退出码（1）不变，但信封 fix 行文本对四条既有命令发生确定性变更，违反本轮"既有调用面信封文本完全不变"的冻结标准。触发面虽窄（只读文件/磁盘满/权限异常），但对依赖信封文案做启发式恢复的 agent 是可见漂移。

建议：二选一。a) 在 lock.rs 写侧失败分支回填冻结文案 "check that the target path is writable"（open 分支可保留 permissions 文案，因基线读失败本就是该文案）；b) 若认为新文案更准确，在 spec 冻结清单中显式记录该项变更为批准漂移，并同步 CHANGELOG。

### M-2 contacts add 幂等路径从"零写入"变为"锁内整文件重写"（新增文件级副作用）

位置：repos/paperwork-core/src/ops/contacts.rs#L70-L73（幂等分支返回 Ok(content)）叠加 repos/paperwork-core/src/ops/lock.rs#L83-L108（无条件 set_len(0)+write_all）

问题：基线 contacts_add 命中幂等分支时直接 return Ok(())，完全不触碰文件（无写入、无写打开句柄）。新实现中闭包返回原内容，helper 仍无条件执行 truncate+重写：字节内容恒等（新增测试 contacts_add_idempotent_after_lock_migration 只钉了字节级），但文件 mtime/ctime 被刷新，且一次原本无任何写 I/O 的操作现在要获取独占锁。对 mtime 敏感的外部消费者（文件监视器、增量备份、git 脏检测）会在纯 no-op 路径上看到虚假变更事件。这是"单进程行为等价性"的一个确定性缺口，属补锁引入的新副作用。

建议：在 locked_read_modify_write 中当新内容与原内容相等时跳过 set_len/write_all（仅 unlock 后返回），一次性恢复所有调用点的 no-op 语义。实现上可在 modify 前保留原 content 快照再比较，或让闭包返回 Option<String> 表达"无需写"。

---

## Suggestions (CONSIDER)

### m-1 独占锁临界区内新增外部文件 I/O，争用窗口大于 thread_edit 先例

位置：repos/paperwork-core/src/ops/contacts.rs#L76、#L170（锁内 derive_label 读目标 profile 文件）、repos/paperwork-core/src/ops/manifest.rs#L115（锁内 hash_file 读 entry 文件）

问题：thread_edit 先例的锁内 I/O 只作用于被锁文件本身；本轮迁移把对外部文件（profile/entry）的读取也放进了独占临界区。多进程争用时阻塞窗口被外部文件的存在性/大小放大（慢盘、网络盘上更明显）。无正确性问题，纯争用特性变化。

建议：将 derive_label / hash_file 提到取锁之前预计算，锁内只做纯内存变换。

### m-2 pub mod lock / pub fn locked_read_modify_write 扩大 core 的 semver 面，当前无外部消费者

位置：repos/paperwork-core/src/ops/mod.rs#L7、repos/paperwork-core/src/ops/lock.rs#L32

问题：该 helper 仅被 core 内部 ops 使用，但以 pub 暴露后即成为 paperwork-core 0.5.x 的公开 API 面，后续签名调整将受 semver 约束。新增 pub 清单本身符合"只增不改"，此处仅是收窄建议。

建议：改为 pub(crate) mod lock（或保留 mod、把函数降为 pub(crate)），待出现真实跨 crate 需求再升级可见性。

---

## 逐项核查结论（影响面专属清单）

1. **既有调用面回归**：
   - contacts create/add/read：CLI arm（cmd/contacts.rs）零 diff；core contacts_add 成功路径序列化逻辑逐行等价（parse → 去重检查 → push → serialize_contacts），信封文本/exit 码/JSON key 不变；例外见 M-1（io 失败路径文案）与 M-2（幂等路径副作用）。
   - brief add/remove/verify：CLI 零 diff；core 两函数仅把 read-modify-write 搬入锁内，判断顺序与错误分支逐行等价（含 AlreadyExists/NotFound 的 fix/example 原文）；verify 完全未触碰。brief read 新增可选 --entry-title：不传时 entries=全量、detailed=full、conclusion=总数，Default/Json/Plain 三种输出模式与基线逐字一致（Plain 仍是原始文件倾印，未受过滤器影响）。
   - profile create/show/edit/list：CLI 零 diff；core edit_profile 仅迁移锁，字段合并逻辑逐行等价。
   - post 组与 validate：git diff 0f6c384..e7eb049 -- cmd/post.rs cmd/validate.rs output.rs error.rs ops/thread.rs 实测 0 行，零触碰确认。
2. **输出协议冻结**：JSON key 只增（新命令自带 removed/updated 字段；brief read 无新 key）；error.rs 零 diff，七类 category 无新增无改义；退出码语义不变（0/1/2）；usage 信封：canonical_example() 仅新增 contacts remove/update 两臂，fallback 臂（旧文法教学位）("contacts", _) 与全部既有臂原文未动；VALUE_TAKING_FLAGS 仅追加 --new-profile，argv_wants_json 对既有 argv 的行为不变。
3. **补锁副作用**：单进程行为等价性除 M-1/M-2 两处外成立；错误路径均在写之前 unlock 返回、文件不落盘（新增测试已钉 before==after 字节级）；既有集成测试时序不受影响（无争用锁获取开销可忽略；新增并发测试为独立新增项）。Windows 句柄生命周期正确：read/seek/write 全部经被锁句柄本身（符合 os error 33 约束）；write_all 后先 unlock()，再于函数返回时 drop 关闭句柄；File 无用户态缓冲无需 flush；不会遗留句柄导致后续打开失败。File 未带 .create(true)，但四个调用点均有 exists() 前置检查，与基线竞态面等价。解锁失败映射为 io 信封与 thread_edit 基线先例（thread.rs L205/L513）完全一致，非新漂移。
4. **ops_tests.rs 与 master 零 diff 防线**：git diff master..e7eb049 -- repos/paperwork-core/tests/ops_tests.rs 实测 0 行，防线完好；新测试全部落在新文件 ops_contacts_crud_tests.rs（339 行）与 cli_integration.rs 追加区（既有断言无一删除，short_form_whitelist 由 6 探针扩为 26 探针为超集扩展）。core 公开 API 变更面清点：新增 pub 共 3 项——ops::lock::locked_read_modify_write、ops::contacts::contacts_remove、ops::contacts::contacts_update（连同 pub mod lock）；既有 pub 函数签名逐一核实零改动（contacts_add/contacts_create/contacts_read/brief_add_entry/brief_remove_entry/edit_profile 等仅函数体与 doc 注释变化）；无新增依赖（fs2 为基线既有），两 crate 的 Cargo.toml 本轮零 diff。
5. **发布纪律**：两 crate 版本均保持 0.5.0；无新 tag（现有 v0.2.0~v0.5.0 均为既有发布）；git diff 0f6c384..e7eb049 -- CHANGELOG.md 实测 0 行，无新段；README/SKILL.md 逐 hunk 核对均为纯追加（新增 remove/update/--entry-title 示例行与 key 语义说明段），未改动任何旧文法示例行。
6. **与 master 的合并面**：git merge-base master e7eb049 = e71f4ca = master 当前 tip，master 自分叉点起无新提交；本轮合入 master 为纯 fast-forward，冲突面为零，无需任何手动解决。

---

## 总判定

**有条件通过（PASS with fixes）**：0 Critical / 2 Warning / 2 Suggestion。

成功路径、输出协议、零触碰面、测试防线、发布纪律、合并面全部实测干净，不存在破坏性变更。但 M-1（io 失败路径信封 fix 文案漂移）突破了"信封文本完全不变"的冻结标准，M-2（幂等 add 引入整文件重写与 mtime 刷新）是补锁带来的确定性新副作用。建议合入前修复 M-1/M-2（均为小改动：文案回填 + 内容未变时跳过重写），m-1/m-2 可择机跟进。

---

## 修复回应销账段（2026-08-09，编排层裁定 F1-F7 落实）

| 发现 | 处置 | 销账证据 |
|---|---|---|
| M-1 io 失败 fix 文案漂移 | 已销账（F5） | lock.rs 三处写失败分支（open/set_len/write_all）fix 文案回填基线原文 `check that the target path is writable`，信封文本冻结标准恢复；spec §3.9 补录登记该冻结口径 |
| M-2 幂等路径零写入退化 | 已销账（F4） | `locked_read_modify_write` 新增字节相同跳过 truncate+write 分支（modify 前保留快照比较），六调用点 no-op 语义一次性恢复；新用例断言幂等 add 字节恒等 + mtime 不变；release 二进制实测 mtime 稳定 |
| m-1 锁内外部 I/O 前置 | 已销账（F6） | contacts.rs 两处 `derive_label` 与 manifest.rs `hash_file`（含 entry 路径解析）全部提到取锁之前预计算，锁内只剩本文件读改写；已知边界后果登记：brief add 在「entry 文件缺失且标题重复」叠加态下错误优先级由 already-exists 变为 io（前置的固有后果，主路径不受影响，编排层裁定明确要求前置） |
| m-2 pub mod lock semver 面 | 已销账（F7 Oscar m-2） | `pub mod lock` 改为 `pub(crate) mod lock`（函数保持 pub 但模块可见性收窄，等效退出公开 API 面），待真实跨 crate 需求再升级；新增 pub 清单收窄为 `contacts_remove`/`contacts_update` 两项（均为本轮 owner 指令授权的需求面 API） |

修复后验证：`cargo test --workspace` 274 全绿；clippy `-D warnings` 零警告；`git diff master -- repos/paperwork-core/tests/ops_tests.rs` 为空；版本 0.5.0 不变，无 bump/tag/publish/CHANGELOG 发布段。
