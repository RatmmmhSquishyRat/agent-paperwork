# CI 失败诊断报告（2026-08-15，任务 #38）

状态：**已结案（根因确认 + 已被后续提交修复）**
诊断方式：纯取证与复现，未改动任何源代码/测试/文档（本报告除外），无 git 提交。

---

## 1. 结论（TL;DR）

CI 失败共 2 个 run（`46c637c`、`669342e`），失败 job 均为 **smoke（ubuntu/macos/windows 三平台全挂）**，test/fmt/clippy/docs 全绿。

**根因**：owner 裁决实施批的 O1 提交（`9821933`，"revoke write-side sugar flags --reply-to/--mention from post send/edit"）从 CLI 移除了写侧 `--reply-to`/`--mention` 糖标志，但 `.github/workflows/ci.yml` 的 smoke 脚本（unix + windows 两处）仍在调用 `post send standup --author bob --reply-to 1 --mention alice --message "Reply"`。CLI 以 usage 错误（exit 2）拒绝该调用，smoke 断言 `^ok post.send` 失败，step 退出码 1。从 `9821933` 到 `669342e` 的裁决批全部提交均未同步更新 ci.yml smoke 行（逐提交核验确认），因此 CI 在裁决批首个含代码提交的 run 开始持续失败。

**归属**：失败由**裁决批自身**引入（O1 移除标志时漏改 ci.yml），与 Plan-C 回填批无关。回填批 `f94b65f`（commit message 明示 "Incidental: ci.yml smoke ... corrected to the v0.6 body-token form"）已顺带修复该问题；最新 run（HEAD `3ef5dc5`）**全绿**。

**本地 426 测试全绿却未拦住的原因**：smoke 脚本内嵌于 ci.yml，不是 cargo 测试目标，`cargo test` 永远不会执行它；裁决批的验证只覆盖了测试套件，未复跑 CI smoke 段。

---

## 2. Git 状态盘点（诊断时刻）

| 项 | 值 |
|---|---|
| 本地分支 | master，工作区 clean，无未提交改动 |
| 本地 HEAD = origin/master | `3ef5dc5`（docs: archive wip-only documents and register Plan-C backport, task #52） |
| 我方基线 | `669342e`（ledger section 14，本地 426 测试全绿） |
| 回填链（任务 #52） | `0b648d7`（Ivy G1–G5 缺口测试 16 项）→ `f94b65f`（Ultra Review 残余增量回填 + ci.yml smoke 修正 + docs gate 加固）→ `3ef5dc5`（文档归档登记） |
| 衔接关系 | 回填链直接叠加于 `669342e` 之上，无冲突、无重排；fetch 后本地与 origin 完全一致 |

---

## 3. CI 配置盘点

### 3.1 `.github/workflows/ci.yml`（触发：push master / PR）

| job | 平台 | 步骤要点 |
|---|---|---|
| `fmt` | ubuntu | `cargo fmt --all -- --check` |
| `test` | ubuntu / macos / windows（fail-fast: false） | `cargo build --workspace` → `cargo test --locked --workspace` → `cargo clippy --all-targets -- -D warnings` → Docs gate：`RUSTDOCFLAGS=-D warnings` + `cargo doc --no-deps --workspace`（RUSTDOCFLAGS 为回填批 f94b65f 新增，F3 项） |
| `smoke` | ubuntu / macos / windows（needs: test） | `cargo build --release -p paperwork-cli` → 内嵌 shell 脚本（unix bash / windows pwsh）：profile/post/brief/contacts/validate 全链路 + 错误信封 + 旧文法 usage 信封 + fence `md` info-string 断言 |

要点：

- smoke 脚本**内嵌在 ci.yml 中**；`_e2e/smoke.ps1`、`_e2e/concurrency.ps1` **未被任何 workflow 引用**，仅本地脚本，不在 CI 执行面内。
- 失败发生点（`669342e` 版本 ci.yml）：unix 块第 77 行 / windows 块对应行：
  `$BIN post send standup --author bob --reply-to 1 --mention alice --message "Reply" | grep "^ok post.send"`

### 3.2 `.github/workflows/release.yml`

仅 `push tags: v*` 触发。诊断窗口内无 tag 推送，**未被触发，与本次失败无关**。

---

## 4. 线上失败证据（gh cli 取证）

`gh run list`（repo: RatmmmhSquishyRat/agent-paperwork）：

| run id | HEAD | 结论 | 失败 job |
|---|---|---|---|
| 31879040813 | `3ef5dc5`（回填后） | **success** | —（全绿） |
| **31877562381** | **`669342e`（我方基线）** | **failure** | smoke (ubuntu/macos/windows) ×3 |
| **31877484785** | **`46c637c`（三维评审修复轮）** | **failure** | smoke (ubuntu/macos/windows) ×3 |
| 31871729066 | `a91e288`（裁决批之前） | success | — |

两个失败 run 的 job 结构完全一致：fmt ✔ / test×3 ✔（含 clippy、docs）/ smoke×3 ✘。

`gh run view 31877562381 --log-failed` 关键日志（三平台同构，windows 摘录）：

```text
& $BIN post send standup --author bob --reply-to 1 --mention alice --message "Reply" | Assert-Contains "^ok post.send" "post send (reply)"
error usage: unexpected argument '--reply-to' found
fix: required values are named flags (--author/--message for post send/edit); ... --reply-to was removed from write commands (owner ruling 2026-08-15); write the reply reference into the message body itself as an @#N token (e.g. --message "@#2 Sure")
     | FAIL: post send (reply)
##[error]Process completed with exit code 1.
```

ubuntu/macos 同点失败：`error usage: unexpected argument '--reply-to' found` → grep 未命中 `^ok post.send` → `set -e`/pipefail 下 exit 1。

run 31877484785（`46c637c`）日志经 `--log-failed` 核验为**完全相同**的错误与失败点。

时间线印证：`a91e288`（裁决批前最后一次 push）CI 全绿 → 裁决批 O1 移除标志后首个含代码 run 起失败 → 回填批 `f94b65f` 修正 smoke 后 `3ef5dc5` 恢复全绿。

---

## 5. 线下复现结果表（HEAD = `3ef5dc5`，Windows pwsh 7）

按 ci.yml 实际命令逐项复跑；unix smoke 用 Git Bash（`bash --noprofile --norc -e -o pipefail`）复跑。

| # | CI 步骤 | 本地命令 | 结果 | 退出码 |
|---|---|---|---|---|
| 1 | fmt | `cargo fmt --all -- --check` | PASS | 0 |
| 2 | build | `cargo build --workspace` | PASS | 0 |
| 3 | test | `cargo test --locked --workspace` | PASS，**444 tests**（7+33+148+16+4+102+12+33+18+71，含新增 Ivy G1–G5 16 项） | 0 |
| 4 | clippy | `cargo clippy --all-targets -- -D warnings` | PASS | 0 |
| 5 | docs gate | `$env:RUSTDOCFLAGS="-D warnings"; cargo doc --no-deps --workspace` | PASS | 0 |
| 6 | smoke 构建 | `cargo build --release -p paperwork-cli` | PASS | 0 |
| 7 | smoke (windows pwsh，ci.yml 内嵌块原样提取执行) | — | PASS，`=== ALL SMOKE TESTS PASSED ===`（含 fence md / error envelope / 旧文法 usage 信封 / garbage / seq gap 全部断言） | 0 |
| 8 | smoke (unix，Git Bash 复跑 ci.yml 内嵌块) | — | PASS，`=== ALL SMOKE TESTS PASSED ===` | 0 |

与线上 run 31879040813（`3ef5dc5`）全绿一致。macos 专属路径（与 unix 块同一脚本）差异面仅 `mktemp -d`/`grep`，均为 POSIX 通用，本地无法原生复跑，但 CI 已在 `3ef5dc5` 实测通过。

**历史失败点复现**（静态+动态闭环）：`git show 669342e:.github/workflows/ci.yml` 确认该版本 smoke 仍调用 `--reply-to 1 --mention alice`；对当前二进制手工调用 `post send ... --reply-to 1` 得到的正是 CI 日志中的 `error usage: unexpected argument '--reply-to' found`（main.rs 的 usage-fix 文案与日志逐字吻合）。

---

## 6. 根因定位

### 缺陷条目（唯一）

- **位置**：`.github/workflows/ci.yml`（`669342e` 及之前版本），smoke unix 块第 77 行、windows 块对应行。
- **机制链**：
  [代码缺陷：O1（`9821933`）移除写侧 `--reply-to`/`--mention` 标志时，未同步更新 ci.yml smoke 对这两个标志的调用] →
  [触发：裁决批 push 后 CI smoke job 执行该调用，CLI 返回 usage 错误 exit 2] →
  [症状：`grep "^ok post.send"` / `Assert-Contains` 断言失败，三平台 smoke job 全部 exit 1]。
- **证据**：第 4 节线上日志原文；`git show <commit>:.github/workflows/ci.yml` 逐提交核验 `9821933..669342e` 每一版 ci.yml 均含 `--reply-to` 调用（8/8 命中）；`f94b65f` diff 明示修正该两行后 run 31879040813 全绿。
- **置信度**：**Verified**（线上失败日志 + 历史版本文件取证 + 修复后 run 全绿，因果闭环）。
- **证伪检查**：若不移除标志而保留旧 smoke，则不会失败——无其他已知替代成因（test/clippy/docs 在失败 run 中全绿，排除编译/测试/文档路径）。

### 批次归属裁决

| 批次 | 与失败的关系 |
|---|---|
| 裁决批（d920271→O1..O5→46c637c→669342e） | **引入者**：O1 移除标志但漏改 ci.yml smoke（O2–O5 与三维评审轮均未补） |
| 回填批（0b648d7/f94b65f/3ef5dc5） | **修复者**：f94b65f 附带修正 smoke 两行（body-token 形式），并将 docs gate 加固为 RUSTDOCFLAGS -D warnings；0b648d7 的 16 项 Ivy 测试与失败无关（test job 在失败 run 中亦全绿） |
| 两者交互 | 无交互成因；失败与回填批内容无因果关联 |

---

## 7. 修复建议清单

### 7.1 本缺陷（已闭合，无需新改动）

1. **[已完成]** `.github/workflows/ci.yml` smoke 两处改为 body-token 形式 `--message "@#1 Reply @alice"` —— 由 `f94b65f` 完成，`3ef5dc5` run 已全绿。**无需任何新代码改动。**

### 7.2 防复发建议（可选，均非阻塞）

1. **流程**（docs/dev/workflow-and-todo-2026-08-15.md 或 fix-ledger 登记一条教训）：凡 CLI 标志增删（尤其裁决类 breaking 变更），验证清单必须包含对 `.github/workflows/*.yml`、`SKILL.md`、`README.md`、`_e2e/*` 的全仓 grep 扫查（本次 `--reply-to` 在 SKILL/README 均已同步，唯漏 ci.yml）。
2. **结构**（可选，低优先）：ci.yml 中 unix/windows smoke 逻辑重复度高，可考虑抽为 `_e2e/smoke.ps1` + bash 对应脚本由 workflow 调用，消除双份维护面；但当前内嵌形式已被 owner 既往接受，**不建议在稳定期主动改**。
3. **账目**（建议）：在 `docs/dev/fix-ledger-2026-08-15.md` 或 `open-items-ledger-2026-08-15.md` 补一条事实登记：669342e/46c637c 两个 run 的 smoke 失败已由任务 #52 回填批 f94b65f 修复，避免后续审计误判为未决缺陷。
4. **release.yml**：无改动需求（未触发；其 test job 不含 smoke，发布前仍建议人工跑一次 `_e2e/smoke.ps1`）。

---

## 8. 取证与清理记录

- gh cli（账号 RatmmmhSquishyRat，token scope 含 repo/workflow）用于 run 列表与失败日志拉取；未做任何写操作。
- 诊断期间临时文件（失败日志导出、两个 smoke 提取脚本）均置于 `target/` 且**已全部删除**；工作区 `git status` 保持 clean（本报告新增文件除外，按任务要求落盘，未提交）。
- 未读取、未触碰 agent-paperwork-wt-v05perfection worktree 与 wip 分支；未做任何发布动作。
