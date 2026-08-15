# owner 裁决执行日志（2026-08-15，任务 #36）

- 基线：master @ d920271，419 测试全绿。
- 权威依据：docs/dev/owner-rulings-2026-08-15.md（裁决原文与口径 A~D）；docs/ssot/specs/cli-grammar-v0.6/impl_plan.md「2026-08-15 owner 裁决实施批次」O1~O5；spec.md / bdd.md / tdd.md 修订段。
- 纪律执行：原子提交（每 O 批一个提交，带批次号）；输出协议只增不改；不 bump/tag/不推送；版本纪律 0.5.0；ops_tests.rs 零改动防线延续（本批未触碰任何 core 与 ops_tests 字节）。
- 门禁口径登记：impl_plan 对 O1/O2 步显式允许 cli_integration/char_tests 红（由 O3 恢复），O1/O2 提交时门禁为 build + clippy（+fmt）绿；O3 起恢复并维持「cargo test --workspace --locked 全绿 + clippy -D warnings + fmt --check」。

## O1 写侧糖参数撤销 — 提交 9821933

改了什么：

- `repos/paperwork-cli/src/cmd/post.rs`：
  - Send clap 签名删除 `--reply-to` / `--mention`（传入落未知 flag 的 usage exit 2 拒绝面，纳入无短形式/未知 flag 统一面）；Edit 签名本就无此二 flag，按写命令外延声明由 usage_fix 覆盖。
  - 糖衣注入管道整体删除：`--reply-to 0` validation 拒绝、mention 名单清洗/校验（`clean_list`/`validate_mention_value`）、flag 派生 implicit-mention、token dedup、`inject_reference_tokens` 注入调用；三个助手函数删除。正文改为逐字写入。
  - implicit-mention 改正文 token 驱动：`derive_reply_to(&body)` -> `find_message_sender`（有界尾扫）-> 非作者且未被正文显式 `@` 时输出；v0.5 边界（自回复/显式覆盖/缺失 seq 静默）冻结不变。
  - after_help 换正文直书教学示例（`--message "@#2 Sure, @alice I'll take it."`）。
  - post read 的 `--mention`/`--reply-to` 过滤器一字未动（读侧保留声明）。
- `repos/paperwork-cli/src/main.rs`：usage_fix 新增两个撤销 flag 迁移教学分支（post.send/post.edit，置于通用长 flag 分支之前）；VALUE_TAKING_FLAGS 保留两 flag（spec §5 第 5 条钉住，usage 路径 `--json` 探针值跳过逻辑依赖）；canonical_example 无需改（post send 示例本就无糖衣 flag）。

测试结果：build + clippy + fmt 绿；cli_integration 11 红 + char_tests 3 红，全部为糖参数用例，与 tdd §9.1 盘点清单逐一对应（O3 改写恢复）。

fix 文案定稿（bdd S-SEND-22/23 口径，已随 O1 冻结于实现）：

- `--reply-to was removed from write commands (owner ruling 2026-08-15); write the reply reference into the message body itself as an @#N token (e.g. --message "@#2 Sure")`
- `--mention was removed from write commands (owner ruling 2026-08-15); write mentions into the message body itself as @name tokens (e.g. --message "@carol ping")`

## O2 contacts advisory 非阻塞校验 — 提交 14f3b57

改了什么：

- `repos/paperwork-cli/src/cmd/contacts.rs`：Add/Update 臂在写入成功之后、emit_ok 之前增补只读探测 `destination_advisory`（add 探 `--profile`，update 探 `--new-profile`）：存在性 -> 可读性（read_to_string）-> 合法性（`parse_profile`）三级探测，命中任一异常即在 ok 信封增补 `advisory` 字段（`Envelope::field` 自动流入 Default 与 `--json` 同名 key）。永不改变退出码、不引入新写入失败路径、不新增 flag；路径解析复用 `resolve_contact_path`（R11 两级解析）；空键护栏与 fs2 写路径锁不受影响（探测在锁外、写后执行）。
- spec.md §3.6 与 bdd S-CONTACTS-16 文案标记回冻（「实施时可微调/以任务 #36 定稿为准」-> 定稿冻结标记）。

测试结果：build + clippy + fmt 绿；四形态冒烟验证通过（不存在 / 目录作 destination 的不可读 / 格式非法 / 合法不触发；--json 同名 key；update 同面）。

advisory 文案定稿（逐字采用 spec §3.6 建议形态，单行纯 ASCII）：

- `destination '<P>' does not exist`
- `destination '<P>' is not readable`
- `destination '<P>' is not a valid profile file`

## O3 测试批 — 提交 77f19e2

cli_integration（143 -> 148）：

- 改写为正文直书形态：implicit_mention_triggered_on_reply、implicit_mention_not_triggered_boundaries、implicit_mention_persisted_to_file（叠加「CLI 不得注入」负向断言）、post_send_mention_body_tokens_verbatim（原 injects_body_tokens）、post_send_reply_to_body_token_verbatim（原 injects_body_tokens；JSON mentions 断言改 `[]`）、post_send_reply_token_no_injection_dedup（原 dedup，透传冻结）、post_send_oversized_body_rejected（原 after_injection，去 --mention 参数）。
- flag_inventory_matches_spec：send 侧断言翻转为负向（S-SHORT-02 收窄）；「send keeps the sugar flags」注释块更正。
- 随 flag 撤销删除：post_send_mention_rejects_malformed_values、post_send_mention_trims_whitespace_and_trailing_comma、post_send_reply_to_zero_rejected（正文 @name 不校验透传冻结由新用例 post_send_mention_tokens_pass_through_unvalidated 钉住）。
- 新增：post_send_revoked_reply_to_flag_usage_rejected（S-SEND-22，含 JSON 档与无写入断言）、post_send_revoked_mention_flag_usage_rejected（S-SEND-23）、post_edit_revoked_flags_usage_rejected（S-EDIT-10，双 flag）、revoked_flag_usage_envelopes_are_pure_ascii（S-OUT-05 延伸）、contacts_add_destination_advisory_nonblocking（S-CONTACTS-16 三形态 + JSON key + ASCII 字节断言）、contacts_add_valid_destination_no_advisory（反向断言）、contacts_update_destination_advisory_nonblocking（S-CONTACTS-17，updated/advisory 并存 + 合法不触发）。
- 读侧过滤用例（S-READ-04/06/07 等）原样冻结，一字未改。

char_tests（31 -> 33）：

- 改写：char_post_send_reply_to_implicit_mention、char_post_send_reply_to_missing_seq_envelope_unchanged（正文 token 化）；char_post_send_mention_flag_injection 更名 char_post_send_mention_body_tokens。
- 新增：char_revoked_sugar_flags_usage_gold（send/edit 双 flag + JSON 档 + 无写入）、char_contacts_advisory_gold（触发/JSON/不触发三金样）。

黄金快照重冻清单（tdd §9.3 五类面，录制模式一次性重冻，150 条目；替换关系已在提交信息登记）：

- 变更 4 项：contacts_add_second_json_stdout（新增 advisory key，serde_json 字母序排最前）、post_send_implicit_mention_file（正文不再注入 `@alice`）、post_send_mention_file（正文即原值 `@bob @carol heads up`）、post_send_reply_missing_seq_file（正文逐字 `@#5 ping`）。
- 新增 7 项：contacts_add_advisory_default_stdout、contacts_add_advisory_json_stdout、contacts_add_valid_no_advisory_stdout、usage_revoked_reply_to_stderr、usage_revoked_mention_stderr、usage_revoked_reply_to_edit_stderr、usage_revoked_mention_json_stdout。
- 其余全部 label 逐字节不变（含 post read 过滤器、showing/window、全部 error 面）——冻结面回归通过。

测试结果：全 workspace 426 测试全绿；clippy -D warnings 绿；fmt --check 绿。

## O4 文档同步 — 提交 6a36639

- SKILL.md：post send 示例改正文直书（`--message "@#1 On it, @alice"`）；post 说明段新增正文 token 语义、撤销声明（usage exit 2 + fix 教学、读侧过滤器保留）；contacts 段新增 advisory 非阻塞行为声明（agent-first UX 方向）。
- 根 README.md：快速上手与 Commands 段两处示例改正文直书；糖衣说明段整体改写为正文直书 + 撤销声明；contacts 段追加 advisory 声明。
- repos/paperwork-cli/README.md：盘点确认无撤销 flag 示例，无需改动（事实登记）。
- design.md（impl_plan 点名两处）：§2.1 签名示意删除糖衣 flag 并加裁决更正注；§12 基线勘误记录行追加〔2026-08-15 owner 裁决更正〕（注入机制废止、正文直书 + 读侧 derive、读侧过滤器保留）；L60/L79/L84/L217 属历史论证文本，按 impl_plan 范围不改。

## O5 CHANGELOG Unreleased — 提交 72c85ac

- CHANGELOG.md `[Unreleased]` 顶部新增两段：`Removed — write-side sugar flags`（breaking for 糖参数调用方：usage exit 2、fix 教学、正文直书迁移路径、读侧过滤器保留）；`Added — contacts destination advisory`（非阻塞探测、exit 0 不变、advisory 字段三形态文案、Default/--json 同名 key、只增不改协议、版本纪律 0.5.0 不 bump/tag/publish）。
- 既有发布段不回改；不新增版本段（spec §7 第 4/6 条）。

## 最终验证（O3 门禁口径，O5 提交前复跑）

- `cargo test --workspace --locked`：10 个测试二进制，426 测试全绿（6 + 33 + 148 + 4 + 101 + 12 + 33 + 18 + 71 + 0）。
- `cargo clippy --workspace --locked --all-targets -- -D warnings`：绿。
- `cargo fmt --all --check`：绿。
- 基线对比：419 -> 426（cli_integration 143 -> 148；char_tests 31 -> 33）。

## 行为变更清单（对外）

1. post send / post edit 传入 `--reply-to` 或 `--mention`：exit 2 usage 信封（含迁移教学 fix + canonical example），无任何写入（breaking for 糖参数调用方）。
2. post send 正文逐字写入：不再有正文首行 token 注入；`@#N`/`@name` 由 agent 直书；implicit-mention 改由正文 `@#N` token 派生，触发边界与字段形态不变。
3. post read 的 `--mention`/`--reply-to` 过滤器保留不变。
4. contacts add/update：destination 不存在/不可读/格式非法时仍 exit 0 照常写入，ok 信封新增 `advisory` 字段（单行纯 ASCII，仅触发时出现，Default 与 --json 同名 key）；合法 destination 无该字段；永不因 destination 问题改变退出码（additive，输出协议只增不改）。
5. 未变面：ok/error 信封结构、七类 category、退出码体系、showing/window、ensure_suffix、别名、三档输出、VALUE_TAKING_FLAGS、core 公开 API、文件格式、ops_tests 冻结面。
