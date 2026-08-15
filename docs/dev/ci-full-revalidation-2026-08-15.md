# 全量 CI 复验与闭环报告（2026-08-15，任务 #40）

- 验证对象：裁决实施链（O1~O5）+ 任务 #52 v0.5-perfection 回填批 + CI 事件账目——即 master @ 8571186 上的全部工作成果。
- 基线：本地 master = origin/master = `85711868534a5b02fdda36a583657c97e5b7fbd3`（fetch 后逐字核对），验证开始时工作区干净。
- 纪律：只运行与取证，未改任何源代码/测试/ci.yml（本报告落盘除外）；未触碰 wt-v05perfection 与 wip 分支；owner 口径「不发布」全程遵守。
- 二进制：`cargo clean` 后冷重建的 release 产物（全部现场实测用它）。

## 总判定表

| # | 检查项 | 判定 |
|---|--------|------|
| 1 | 同步（本地=origin=8571186，工作区干净） | PASS |
| 2 | 线下全门禁（对齐 ci.yml 全部 job/step） | PASS |
| 3 | 裁决面抽查（回填批未改变裁决行为） | PASS |
| 4 | 线上 run（HEAD 8571186 触发，全 7 job） | PASS |
| 5 | 版本纪律（0.5.0 / tag / CHANGELOG / release.yml 未触发） | PASS |
| 6 | 禁区（wt-v05perfection 与 wip 分支零触碰） | PASS |

**最终放行结论：放行。** 线下逐门禁复现 ci.yml 全口径，444 测试全绿；线上 CI run 31880791223 全 7 job success；无失败项。

## 1. 同步 — PASS

- `git fetch origin` 后：`git rev-parse HEAD` == `git rev-parse origin/master` == `85711868534a5b02fdda36a583657c97e5b7fbd3`。
- `git status --porcelain` 空；当前分支 master。

## 2. 线下全门禁（对齐 ci.yml）— PASS

ci.yml 真 CI 面盘点（先读后跑）：`fmt`（1 job：`cargo fmt --all -- --check`）；`test`（ubuntu/macos/windows 矩阵，各含 Build `cargo build --workspace`、Test `cargo test --locked --workspace`、Clippy `--all-targets -- -D warnings`、Docs `RUSTDOCFLAGS=-D warnings cargo doc --no-deps --workspace`）；`smoke`（三 OS 矩阵，内嵌脚本：unix 用 bash、windows 用 pwsh）。**`_e2e/smoke.ps1` 与 `_e2e/concurrency.ps1` 未被 ci.yml 引用，属本地资产，不在门禁口径内**（本次未作为门禁执行，仅作事实登记）。

`cargo clean`（移除 5498 文件 / 1.4GiB）后逐项实测：

| 门禁 | ci.yml 口径 | 实测 | 判定 |
|------|-------------|------|------|
| Build | `cargo build --workspace` | exit 0 | PASS |
| Build（加强） | `cargo build --release --locked` | exit 0（paperwork-core/cli 均 0.5.0） | PASS |
| Test | `cargo test --locked --workspace` | **444 全绿**（7 + 33 + 148 + 16 + 4 + 102 + 12 + 33 + 18 + 71 + 0），逐二进制 `test result: ok` | PASS |
| Clippy | `cargo clippy --all-targets -- -D warnings` | exit 0 | PASS |
| Fmt | `cargo fmt --all -- --check` | exit 0 | PASS |
| Docs | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` | exit 0，rustdoc 零警告 | PASS |
| Smoke (windows) | ci.yml 内嵌 pwsh 脚本逐字提取复跑（Assert-Contains 硬化断言面原样保留） | 全部 PASS 断言命中 + `=== ALL SMOKE TESTS PASSED ===`，exit 0 | PASS |
| Smoke (unix) | ci.yml 内嵌 bash 脚本逐字提取，经 Git Bash（C:\Program Files\Git\bin\bash.exe）复跑 | 全链路 ok + 6 个 PASS 断言 + `=== ALL SMOKE TESTS PASSED ===`，exit 0 | PASS |
| Smoke (macos) | 本机无 macOS runner | 由线上 run 覆盖（§4：smoke (macos-latest) success） | PASS（线上证据） |

Ivy G1–G5 证据（444 中的 16 项，`ivy_gap_tests` 全绿，名单 `--list` 实测）：

- G1：ivy_g1_validate_v04_legacy_post_default_envelope
- G2：brief 缺 owner/created、部分迁移残留拒收、contacts legacy、profile 缺 model（4 项）
- G3：ivy_g3_validate_json_error_envelope_structure
- G4：edit not-final / not-most-recent / not-owned 三面（信封 + 字节不变，3 项）
- G5：并发首发竞争、CRLF 往返、错误信封 quiet/plain/json 不变、注入护栏字面换行拒收、read 过滤器空命中信封、summary not-found、unicode JSON 往返（7 项）

windows smoke 内嵌断言明细（复跑输出逐条）：PASS fence info md / error envelope / empty body / old grammar usage envelope / validate garbage / validate seq gap。unix 面同口径（含 `post read --json` 结构断言、brief read regex 面、contacts read alice 面）。

## 3. 裁决面抽查（release 二进制，确认回填批未改变裁决行为）— PASS

| 用例 | 实测 | 判定 |
|------|------|------|
| post send `--reply-to` | exit 2 usage 信封，fix 逐字含迁移教学（owner ruling 2026-08-15 @#N 教学） | PASS |
| post send `--mention` | exit 2 usage 信封，fix 逐字含 @name 教学 | PASS |
| 正文直书往返 | `@#1 hi @alice` 逐字写入；read JSON 派生 `mentions:["alice"], reply_to:1`；无 token 消息 `mentions:[], reply_to:null` | PASS |
| 读侧过滤器 | `post read --mention alice` 命中 #2（window #2-#2），exit 0 | PASS |
| advisory 三文案 | ghost → `destination 'ghost.profile.md' does not exist`；非法 → `is not a valid profile file`；目录 → `is not readable`；三者均 exit 0 写入成功 + advisory 字段 | PASS |

## 4. 线上跟踪 — PASS

- 触发确认：`gh run list --commit 8571186…` 命中 CI run **31880791223**（workflow=CI，event=push，branch=master，headSha 与 8571186 逐字一致）。
- 最终状态（`gh run view 31880791223 --json status,conclusion`）：status=completed，**conclusion=success**。
- 逐 job（`gh run view … --json jobs`）：

| Job | Conclusion |
|-----|-----------|
| fmt | success |
| test (ubuntu-latest) | success |
| test (macos-latest) | success |
| test (windows-latest) | success |
| smoke (ubuntu-latest) | success |
| smoke (macos-latest) | success |
| smoke (windows-latest) | success |

与预期「fmt / test×3 / smoke×3 全绿」完全一致。

## 5. 版本纪律 — PASS

| 项 | 实测 | 判定 |
|----|------|------|
| 版本未 bump | paperwork-core 与 paperwork-cli Cargo.toml 均 `version = "0.5.0"` | PASS |
| tag 止于 v0.5.0 | `git ls-remote --tags origin` 仅 v0.2.0 / v0.3.0 / v0.4.0 / v0.5.0 | PASS |
| 无发布段 | CHANGELOG 版本段序列：[Unreleased] → [0.5.0] 2026-08-09 → 更早，无新发布段 | PASS |
| release.yml 未触发 | release.yml 仅 tag `v*` push 触发；`gh run list --workflow release.yml` 最近两次为 v0.5.0（2026-08-08）与 v0.4.0（2026-08-02），本批无任何新 Release run | PASS |

## 6. 禁区 — PASS

- 全程在 master 工作；`git branch -a` 中的 `wip/v0.5-perfection-snapshot-2026-08-15`（本地+远端）未被 checkout/改动/推送。
- wt-v05perfection worktree 属另一工作流，本次未接触。
- 现场夹具（_verify_tmp40 与 %TEMP% smoke 目录）验证后清理，提交前 `git status --porcelain` 仅含本报告。

## 最终放行理由

1. ci.yml 的每一个 job/step 都有线下同口径复跑证据（windows/unix 双 smoke 逐字脚本、docs gate 含 RUSTDOCFLAGS -D warnings），或线上对应 job success 覆盖（macos）。
2. 444 测试全绿独立复现，Ivy G1–G5 十六项名单逐一在位。
3. 回填批未改变裁决行为：写侧拒收、正文直书派生、读侧过滤器、advisory 三文案全部现场复验。
4. 线上 CI（run 31880791223）与线下一致全绿；版本纪律与「不发布」口径完整保持。

遗留观察（非阻塞）：`_e2e/smoke.ps1`、`_e2e/concurrency.ps1` 为本地资产而非 CI 面——如后续希望纳入 CI，属新工作项，不影响本次闭环。
