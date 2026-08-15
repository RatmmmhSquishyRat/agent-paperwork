# v0.5 perfection wip 净增量回填决策记录（方案 C，2026-08-15）

任务 #52 执行阶段。本文先于任何回填改动落盘，随执行逐项更新，最终随文档批提交。

## 一、方案 C 裁决依据（调研结论）

1. **直接 merge/rebase 不可行**：`master` 与 `wip/v0.5-perfection-snapshot-2026-08-15` 直接合并会产生 28 处冲突；master 已通过 P-0~P-9 修复波将 wip 本线主体（NEW-7/11/12、P-3/P-6、D2~D6、C-1 等）移植到 v0.6 基线并扩展（426 测试 vs wip 326）。
2. **约 90% 的 wip 增量已被 master 吸收**：性能优化、格式守卫（D3/D4/C-1）、tail-scan、流式哈希等均已在 master 有等价或更强实现。
3. **LockedFile RAII（`ops/lock.rs`）已被 master 裁决拒绝**：master 采用 fs2 `lock_exclusive` + 显式 unlock / lock-helper 模式，`ops/lock.rs` 不存在于 master。凡涉及该文件的回填一律跳过。
4. 故 owner 裁决**方案 C：选择性回填净增量**，不做 git merge/rebase 整个分支。

铁约束执行声明：master 现状优先；v0.6 具名文法（docs/ssot/specs/cli-grammar-v0.6/）为 SSOT，回填测试一律按 v0.6 文法书写；版本保持 0.5.0，不涉及任何发布/bump 表述。

## 二、五项清单与最终处置

### 1. 决策记录先行（本文档）——采纳

本文档即处置载体，边做边更新，随文档批提交。

### 2. ivy_gap_tests 移植——采纳（修订：全部改写为 v0.6 具名文法 + master 现行信封）

来源：wip commit `5bfb061`（`repos/paperwork-cli/tests/ivy_gap_tests.rs`，16 测试）。
移植为 master 新文件 `repos/paperwork-cli/tests/ivy_gap_tests.rs`，逐场景处置：

| 场景 | wip 原面 | master 处置 |
|---|---|---|
| G1 v0.4 legacy post validate | 默认信封 byte-exact | 采纳；example 按 v0.6 文法（`--author/--message` 形态，master 实测钉住） |
| G2 profile 缺 model | byte-exact | 采纳（信封与 master 实测一致） |
| G2 brief 缺 owner/created | byte-exact | 采纳（信封与 master 实测一致） |
| G2 contacts legacy 全信封 | byte-exact | 采纳（master `cmd/validate.rs` 信封与 wip 逐字一致；既有 `validate_rejects_legacy_contacts` 仅 JSON 面子串断言，本测试补默认面全信封） |
| G2 brief 部分迁移残差 | 子串断言 | 采纳（master 残差信封 message/fix 子串不变） |
| G3 validate --json 错误信封 | byte-exact | 修订：master JSON 错误信封含 `"command"` 键（v0.5 spec 冻结面），按 master 实测字节钉住 |
| G4 edit 三拒绝 + 字节不变 | v0.5 位置文法 | 修订：改 `--author/--seq/--message` 具名文法；保留「拒绝后文件字节不变」独有断言面（既有 `edit_triple_guardrail_cli` 无字节稳定性断言） |
| G5 read 过滤无匹配空信封 | `ok post.read 0 messages` | 修订：master read 恒显 `showing: n/total`，空窗口无 `window` 字段；默认面与 JSON 面按 master 实测钉住（补 `--reply-to` 过滤未命中与 JSON 空数组面，既有测试仅覆盖 `--mention` 默认面） |
| G5 summary 缺文件宽容空 summary | 宽容空信封 | **修订（语义变更）**：master 经 Kim M1 对称守卫将缺文件 summary 改为 not-found exit 1（`cmd/post.rs` Summary 分支显式登记）；测试改写为钉住 not-found 信封（默认 + JSON），原宽容语义已废止 |
| G5 quiet 错误路径 | quiet==非 quiet | 采纳并扩展：quiet/plain/json-quiet 三组合错误信封字节不变 |
| G5 CRLF roundtrip | v0.5 文法 | 修订：v0.6 具名文法，断言面不变 |
| G5 Unicode roundtrip | v0.5 文法 | 修订：v0.6 具名文法，断言面不变 |
| G5 注入护栏（title/model 换行） | 子串断言 | 采纳（master `check_single_line` 信封含子串 `thread title contains a line break` / `model contains a line break`，零写入断言保留） |
| G5 CONC-02 首次 send 并发 | v0.5 文法 | 修订：v0.6 具名文法；preamble 恰一次 + seq {1,2} + validate 通过断言保留（与既有 10 进程不丢消息测试互补：本测试钉 preamble 单次写入面） |

计数：16 个 `#[test]` 全数移植，无删减。

### 3. Ultra Review 代码修复残差（wip `633a6fe` F1 / `8539a08` F3/F4/F5/F7/F9）

逐 hunk 对拍结论：

- **F1（写侧对称 reserved-heading 护栏 + fence-aware 检测 + parse_entry_body roundtrip）**——跳过（master 已等价吸收且更强）：
  - prose 侧：master 修复波 `2c7a180`（D3）的 `contains_heading_line` 拒绝 preamble 散文**任何**标题形行（含 `## Entries`/`### `），且按 master 文档化立场**刻意不 fence-aware**（preamble 解析器不做围栏跟踪，围栏不能屏蔽嵌入标题）——覆盖面严格包含 F1 的 fence-aware 版本，二者立场冲突时以 master D3 为准。
  - note 侧：master C-1 守卫 `note_representation_issue` + `note_contains_heading_outside_fence` 拒绝围栏外任何标题形行 + 未闭合围栏拒绝，严格强于 F1 的 reserved-shape 检查。
  - `parse_entry_body` 保留 note 围栏闭合行：master 已有（C-1 roundtrip 保证，manifest.rs 注释与实现俱在）。
  - 残留 message 扩写（自产散文成因）：纯文案，且 master 信封字节被 char_tests 钉住，不改。
- **F3（ci.yml `RUSTDOCFLAGS: -D warnings`）**——采纳：master CI `Docs` 步骤（test job 内）无该 env，门禁实际不因 rustdoc 警告失败；按 wip 补上。
- **F4（JsonBuilder 键序机制文档 + 字母序单测）**——采纳：master `output.rs` JsonBuilder 文档无 BTreeMap 字母序事实披露，亦无钉住单测；属 8539a08 净缺口，docs+test 一并移植。
- **F5③（reply-to 尾扫窗口限制披露）**——采纳（措辞适配 master）：master `ops/thread.rs::find_message_sender` 与 `ops/thread_scan.rs::find_message_sender_locked` 文档仍写「residual limitation documented in spec §5.5」，未披露「窗口仅对 SEQ 解析为 spec 强制、sender 复用为实现决策」；CHANGELOG（NEW-12 条）已有等效披露，故移植 wip 的文档措辞修正（保留 master 的锁立场表述）。
- **F7（create_profile_full 陈旧「CLI wiring pending」注释）**——跳过：master 已重写该文档（SAM-2 表述），陈旧注释不存在，缺口已消。
- **F9（ops/lock.rs LockedFile Drop 注释）**——跳过（铁约束 2）：master 拒绝 LockedFile RAII，`ops/lock.rs` 不存在；按裁决留痕，不回填。

**附带修复（同批，ci.yml）**：master CI smoke 两段（unix L77 / windows L176）仍使用已被 owner 裁决撤销并经任务 #36 O1（commit `9821933`）实施的 post send 写侧 `--reply-to`/`--mention` flag，实测落 usage exit 2，smoke job 必失败。按 v0.6 SSOT 改为正文直书 `@#N` token 形态。此为 master 既有失同步的纠偏，非 wip 回填项，特此留痕。

### 4. F2 per-spec suffix 链（wip `ffb7a54`）——采纳（适配 master 结构）

spec 相容性核对：master 现行 `docs/dev/format-v2/spec.md` §5.7 明确缺省标题算法为「剥 `.post.md`，否则剥 `.md`，否则原名」；§7.3 R11 明确 label 回退为「先剥 `.profile.md`，再剥 `.md`，否则原名」。master 现状两处均为 `.profile.md -> .post.md -> .md` 超集链：

- `cmd/post.rs::default_title` 循环含 `.profile.md`（退化输入 `x.profile.md` 派生 `x`，spec 应为 `x.profile`）；
- `format::strip_known_suffix`（仅 `derive_label` 消费）含 `.post.md`（退化条目 `x.post.md` 派生 `x`，R11 应为 `x.post`）。

与 spec 相容，移植：core 增 `strip_first_of` 原语 + `strip_title_suffix`（§5.7）+ `strip_label_suffix`（§7.3 R11），`derive_label` 改挂 label 链；`default_title` 保留 master 的 P-6 无损 OsStr 剥离机制、仅收窄后缀表为 §5.7 链；`strip_known_suffix` 删除（无其他消费方、无测试钉住退化行为）。wip 的两链单测（含两退化场景）移植到 core format 测试。

### 5. wip 独有文档——采纳

- `docs/reviews/v0.5-debt-closure-ledger-2026-08-15.md`：master 无同名文件，整份合入，文件头加注「源自 wt-v05perfection 分支，经方案 C 回填归档」。
- `docs/dev/format-v2/test-matrix-2026-08-15.md`：master 无同名文件，整份合入（原样，不改动其内容）。
- `CHANGELOG.md` [Unreleased]：补一条 internal backport 登记（internal backport of residual v0.5-perfection increments，列实际采纳项）。
- README 不改数字（维持 master 不写具体计数的口径）。
- `docs/dev/open-items-ledger-2026-08-15.md` 第十二节：追加一行登记「已完成方案 C 回填，分支存档保留」（worktree 退役后补记）。

## 三、门禁与提交计划

- 分批：测试批（ivy_gap_tests）/ 代码修复批（F2/F3/F4/F5③ + ci smoke 纠偏）/ 文档批（ledger、test-matrix、CHANGELOG、本文档、open-items-ledger 登记）。
- 每批后 `cargo test --workspace --locked`；终门禁加 clippy -D warnings、fmt --check、RUSTDOCFLAGS=-D warnings cargo doc --no-deps。
- 全绿后普通 `git push origin master`；随后退役 worktree（不 --force），wip 分支本地与远端保留存档。

## 四、执行结果

- 测试批：commit `0b648d7` — `repos/paperwork-cli/tests/ivy_gap_tests.rs`（16 测试，v0.6 具名文法），426 -> 442 全绿。
- 代码批：commit `f94b65f` — F2 suffix 双链 + F3 RUSTDOCFLAGS + F4 JsonBuilder docs/单测 + F5③ 措辞 + ci smoke 纠偏，442 -> 444 全绿。
- 文档批：本提交 — debt-closure-ledger（加来源头注）+ test-matrix（原样）+ CHANGELOG [Unreleased] 回填登记 + 本文档 + open-items-ledger 第十二节追加登记。
- 门禁（文档批后终验）：`cargo fmt --all --check` 通过；`cargo test --workspace --locked` 444/444 绿；`cargo clippy --workspace --all-targets --locked -- -D warnings` 零警告；`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` 零警告。
- push 与 worktree 退役：文档批全绿后随即执行普通 `git push origin master`；push 成功后 `git worktree remove ../agent-paperwork-wt-v05perfection`（不 --force）；wip 分支本地与远端保留存档（已登记于 open-items-ledger 第十二节）。执行结果见任务 #52 终报。
