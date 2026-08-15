# owner 裁决实施端到端验证报告（2026-08-15，任务 #37 第一关）

- 验证对象：任务 #36 owner 裁决实施 O1~O5（docs/dev/rulings-execution-log-2026-08-15.md）。
- 基线：master @ b9b059c（工作区验证前干净，HEAD 与任务声明一致）。
- 纪律：全程只运行与取证，未改任何源代码/测试/文档（本报告落盘除外）；未做 git 提交；现场夹具用后全量清理，验证结束时 `git status --porcelain` 为空。
- 二进制：`cargo clean` 后冷重建的 `target/release/paperwork.exe`（report 内全部现场实测均用该二进制）。

## 总判定表

| # | 检查项 | 判定 |
|---|--------|------|
| 1 | 冷重建回归（build/test/clippy/fmt） | PASS |
| 2 | 撤销面（send/edit 拒收 + read 过滤器保留 + 正文直书往返） | PASS |
| 3 | advisory（三文案 / Default 与 --json 同名 key / ASCII / 护栏 / 文件与锁） | PASS |
| 4 | 全命令面冒烟（release 二进制，TEMP 夹具） | PASS |
| 5 | 冻结面（信封/category/退出码/ops_tests/char_tests 重冻一致性） | PASS |
| 6 | 版本纪律（0.5.0 / tag / CHANGELOG / 未推送盘点） | PASS |

**放行结论：放行。** 426 测试冷重建全绿，六大项逐项实测通过，无失败项；两处取证中的夹具构造失误（见 §3 A10、§4 C16）均已补测排除，非产品缺陷。

## 1. 冷重建回归 — PASS

| 命令 | 结果 |
|------|------|
| `cargo clean` | 移除 6685 文件 / 1.8GiB（真冷启动） |
| `cargo build --release --locked` | 绿，paperwork-core v0.5.0 + paperwork-cli v0.5.0 |
| `cargo test --workspace --locked` | **426 全绿**（6 + 33 + 148 + 4 + 101 + 12 + 33 + 18 + 71 + 0），与执行日志「最终验证」分布逐位一致 |
| `cargo clippy --workspace --locked --all-targets -- -D warnings` | 绿 |
| `cargo fmt --all --check` | 绿（exit 0） |

测试二进制分布证据（逐行 `test result: ok`）：paperwork 单测 6、cli char_tests 33、cli_integration 148、t6_cli_tests 4、core 单测 101、core char_tests 12、guard_tests 33、ops_contacts_crud_tests 18、ops_tests 71、doc-tests 0。

## 2. 撤销面核验 — PASS

现场构造线程（#1 alice 首帖 → #2 bob `@#1 On it, @alice` → #3 carol 普通 → #4 dave `@#2 thanks @bob`），release 二进制逐条实测：

| 用例 | 实测 | 判定 |
|------|------|------|
| post send `--reply-to 1` | exit 2，`error usage: unexpected argument '--reply-to' found`；fix 含逐字迁移教学「--reply-to was removed from write commands (owner ruling 2026-08-15); write the reply reference into the message body itself as an @#N token (e.g. --message "@#2 Sure")」；canonical example 为 post send 形态 | PASS |
| post send `--mention alice` | exit 2，fix 含「write mentions into the message body itself as @name tokens (e.g. --message "@carol ping")」 | PASS |
| 上述两例 `--json` 档 | 单行 JSON 信封同面（category=usage / exit_code=2 / fix / example / message / status），stdout 输出 | PASS |
| post edit `--reply-to` / `--mention` | 均 exit 2 usage，fix 同文案，canonical example 为 post edit 形态（含 `--seq`） | PASS |
| 零写入 | 六次拒收后线程文件仍只有 #1（逐字节 dump 核验），无部分写入 | PASS |
| 读侧过滤器 `post read --mention alice` | 命中 #2（`reply:#1 mentions:alice`，window #2-#2），exit 0 | PASS |
| `--mention bob` | 命中 #4，exit 0 | PASS |
| `--reply-to 1` / `--reply-to 2`（含 JSON 档） | 分别命中 #2 / #4，JSON `reply_to`/`mentions` 派生正确 | PASS |
| `--mention nobody`（未命中） | 0 messages，`showing: 0/0`，exit 0，无报错 | PASS |
| 正文直书往返 | 文件正文逐字 `@#1 On it, @alice`（无注入痕迹）；read JSON 派生 `mentions:["alice"], reply_to:1`（#2）与 `mentions:["bob"], reply_to:2`（#4）；无 token 的消息 `mentions:[], reply_to:null` | PASS |

## 3. advisory 核验 — PASS

夹具含：合法 profile、不存在路径、目录冒充 profile（不可读）、内容非法的 `*.profile.md`。

| 用例 | 实测 | 判定 |
|------|------|------|
| add 不存在 destination | exit 0 写入成功 + `advisory: destination 'ghost.profile.md' does not exist`（逐字） | PASS |
| add 不可读（目录） | exit 0 + `advisory: destination 'adir.profile.md' is not readable`（逐字） | PASS |
| add 格式非法 | exit 0 + `advisory: destination 'broken.profile.md' is not a valid profile file`（逐字） | PASS |
| `--json` 同名 key | add/update 的 JSON 信封均含 `"advisory":"destination '<P>' ..."` 同名 key（serde 字母序） | PASS |
| 纯 ASCII | default 与 JSON 两档 advisory 输出非 ASCII 字符计数均为 0 | PASS |
| 合法 destination | add/update 均 exit 0 且**无** advisory 字段 | PASS |
| update 面 | `updated:` 与 `advisory:` 并存（default + JSON 双档实测） | PASS |
| 空键护栏 | add `--profile ''` → exit 1 `validation: ... profile path (--profile) is empty`；update `--new-profile ''` → exit 1 同族文案；advisory 路径未干扰 | PASS |
| 文件内容不变性 | advisory 触发的幂等重复 add 前后 SHA-256 逐字节一致（8CAADE…D0AD == 8CAADE…D0AD） | PASS |
| 锁行为 | 10 次连续 advisory add 全部成功、文件良构（`contacts read` 18 条目可读）；夹具内无任何残留 lock/tmp 文件 | PASS |

取证过程说明：A10 首测因夹具把 `--new-profile` 指向已存在条目，触发 `already-exists` exit 1——这是既有护栏的正确行为，非 advisory 问题；随后补测 B1（无条目冲突的非法 destination，JSON 档）通过，update advisory 面证据完整。

## 4. 全命令面冒烟 — PASS

release 二进制 + 独立 TEMP 夹具，29 + 4 个用例覆盖五组核心动词与三档输出抽样：

- profile：create / show / edit / list + `--json` + `--quiet` + not-found 错误面，全部正确（quiet 仅去状态行、字段保留）。
- post：send（建线程）/ send `--quiet` / read `--json --limit`（showing 1/2 + window 正确）/ summary / edit / `--plain` / not-found 面，全部正确。
- brief：create / add / verify（fresh）/ read `--full --json` / read `--entry-title` / remove 全链路通过。
- contacts：create / add（无 advisory）/ read / remove，全部正确。
- validate：合法 post/profile 通过；外来文件分别落 `io`（不存在）与 `format`（no valid messages found）错误面。
- 退出码分层抽样：成功 0、运行时错误 1（not-found / validation / io / format / already-exists 五类均现场命中）、usage 2（未知 flag + 未知动词两面）。

取证过程说明：C16 首测 brief add 的 `--entry README.md` 在空夹具内不存在，正确落 `io` 错误面（含 OS 本地化 message，符合 SKILL.md 已声明的 io 文案边界）；补测 D1~D4 用夹具内真实文件跑通 add→verify→read→remove 正常链，非产品缺陷。

## 5. 冻结面核验 — PASS

| 冻结项 | 证据 | 判定 |
|--------|------|------|
| paperwork-core 零改动 | `git diff d920271..HEAD --stat -- repos/paperwork-core` 为空 | PASS |
| ops_tests 零改动 | `git diff --quiet -- repos/paperwork-core/tests/ops_tests.rs` exit 0；71 测试全绿 | PASS |
| 信封结构/七类 category/退出码 | core 未动即定义面未动（`error.rs` 七类 category 映射在位：format/validation/io/not-found/already-exists/not-allowed + usage）；现场实测 0/1/2 三档与六类 category 输出形态与冻结面一致 | PASS |
| 黄金快照总量 | char_tests 内嵌黄金表恰 **150 条**，与执行日志登记一致 | PASS |
| 重冻 4 变更项 | contacts_add_second_json_stdout / post_send_implicit_mention_file / post_send_mention_file / post_send_reply_missing_seq_file 均在位，且 contacts_add_second_json_stdout 快照逐字含 advisory key（字母序排前） | PASS |
| 重冻 7 新增项 | contacts_add_advisory_default_stdout / contacts_add_advisory_json_stdout / contacts_add_valid_no_advisory_stdout / usage_revoked_reply_to_stderr / usage_revoked_mention_stderr / usage_revoked_reply_to_edit_stderr / usage_revoked_mention_json_stdout 全部在位（各恰 1 条） | PASS |
| char_tests 31→33 | 2 改写 + 1 更名（char_post_send_mention_body_tokens 在位，旧名 char_post_send_mention_flag_injection 已消失）+ 2 新增（char_revoked_sugar_flags_usage_gold / char_contacts_advisory_gold 在位），实测 33 全绿 | PASS |
| cli_integration 143→148 | 7 个新增用例全部在位（revoked 三例 + pure_ascii + advisory 三例），实测 148 全绿 | PASS |
| 变更面范围 | d920271..HEAD 非文档变更仅 5 个文件：cmd/post.rs、cmd/contacts.rs、main.rs、char_tests.rs、cli_integration.rs，无越界 | PASS |

## 6. 版本纪律 — PASS

| 项 | 实测 | 判定 |
|----|------|------|
| 版本未 bump | paperwork-core 与 paperwork-cli 的 Cargo.toml 均为 `version = "0.5.0"` | PASS |
| tag 未动 | `git tag` 仅 v0.2.0 / v0.3.0 / v0.4.0 / v0.5.0，无新 tag | PASS |
| CHANGELOG | 新增两段（Removed — write-side sugar flags；Added — contacts destination advisory）均在 `[Unreleased]` 段内；无新版本段；既有发布段未回改 | PASS |
| 未推送盘点 | `git log origin/master..master` 共 7 个本地提交：d920271（任务 #35 裁决记录）、9821933（O1）、14f3b57（O2）、77f19e2（O3）、6a36639（O4）、72c85ac（O5）、b9b059c（O5 补记）；全部未推送 | PASS |

## 放行理由

1. 声称的 426 全绿在冷重建下独立复现，分布逐位一致；clippy/fmt 双绿。
2. 两项裁决行为（糖参数撤销、advisory 非阻塞）的每一个对外承诺点——退出码、信封字段、fix 逐字文案、JSON 同名 key、ASCII、零写入、过滤器保留、正文直书派生——都有现场实测证据，与执行日志及 SKILL.md/CHANGELOG 的对外声明逐条吻合。
3. 冻结面（core / ops_tests / 信封 / category / 退出码 / 黄金快照未变更项）经 git 差异与快照清点双重确认零越界。
4. 版本纪律完整保持（0.5.0、无 tag、无发布段、无推送）。

遗留观察（非阻塞）：`io` 类错误 message 内嵌 OS 本地化文案（zh-CN Windows）为既有已声明行为，本次实测与 SKILL.md 声明一致，不计入本关判定。
