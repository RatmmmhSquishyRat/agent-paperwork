# 发布登记：paperwork 0.6.0（准备轮，任务 #49）

- 日期：2026-08-16
- 性质：发布**准备**登记——版本 bump、CHANGELOG 晋升、文档版本面同步已在 master 落盘；**本任务不打 tag、不发布 crates.io**（那是任务 #50）
- 准备基线：master @ 3ea5e1e（任务 #48 终局闭合，CI run 31885972616 全绿，测试基线 452）

## 一、发布授权（owner 原话）

> 「当前属于小里程碑, 可以发布0.6」

版本纪律据此解冻：0.5.0 时期「不 bump/tag/发布」约束（spec §7 第 4 款、台账 LED-15 事实登记面）解除；解除注记已联动落盘于 docs/ssot/specs/cli-grammar-v0.6/spec.md §7 第 4 款与该目录六文档的版本状态行。本发布同时闭合审计发现 S-01（crates.io 0.5.0 与仓库 v0.6 文法的版本语义错配）。

## 二、发布机制侦查结论（任务 #49 步骤 1）

### 2.1 `.github/workflows/release.yml`（GitHub Release 面，全自动）

- **触发条件**：push `v*` tag（即 `git push origin v0.6.0` 即触发）。
- **job 链**：`test`（cargo test --workspace + clippy -D warnings；注意：无 --locked、无 fmt、无 docs gate，门禁强度低于 CI workflow）→ `build`（五目标矩阵：x86_64-unknown-linux-gnu / x86_64-unknown-linux-musl / x86_64-apple-darwin / aarch64-apple-darwin / x86_64-pc-windows-msvc，`cargo build --release -p paperwork-cli` 后打包 tar.gz/zip）→ `release`（下载全部 artifact，用 awk 从 CHANGELOG.md 抽取 `## [0.6.0]` 段落作为 release notes，经 softprops/action-gh-release@v2 创建 GitHub Release 并附二进制）。
- **凭证**：仅用仓库自带 `GITHUB_TOKEN`（permissions: contents: write）；**不使用任何 crates.io secret**。
- **结论：release.yml 不自动 publish crates.io**；它只产出 GitHub Release + 五平台预编译二进制。
- release notes 抽取依赖 CHANGELOG 标题形态 `## [0.6.0]`——本次晋升已按该形态落盘，兼容。

### 2.2 `publish.ps1`（crates.io 面，手工，任务 #50 执行）

- 前置：`cargo login <CRATES_IO_TOKEN>`（本地凭证；仓库无该 secret 的 CI 集成）。
- 流程与顺序：① `cargo publish -p paperwork-core` → ② 轮询 crates.io 索引最长 5 分钟等待 paperwork-core 0.6.0 可见（每 10 秒 cargo search 一次）→ ③ `cargo publish -p paperwork-cli`。顺序不可颠倒（cli 依赖 core）。
- 版本号从 `repos/paperwork-core/Cargo.toml` 运行时解析（review Ethan m-2 后无硬编码字面量），bump 后脚本零改动即可用。
- 超时兜底：若 5 分钟内索引不可见，脚本退出 1 并提示手工补跑 `cargo publish -p paperwork-cli`。

### 2.3 历史发布方式

- tag 列表：v0.2.0 / v0.3.0 / v0.4.0 / v0.5.0（v0.5.0 = 70f7e43，旧文件格式 + v0.5 位置文法，审计取证在案）。
- 即历史发布 = 「master 准备提交 → 打 v 系列 tag → push tag 触发 release.yml」，crates.io 面另行手工 publish。

## 三、bump 与晋升清单（本任务落盘面）

| 面 | 变更 |
|---|---|
| repos/paperwork-core/Cargo.toml | version 0.5.0 → 0.6.0 |
| repos/paperwork-cli/Cargo.toml | version 0.5.0 → 0.6.0；依赖 paperwork-core `version = "0.5"` → `"0.6"`（path 依赖不变） |
| Cargo.lock | cargo build 后两 crate 版本行刷新（--locked 复验通过） |
| CHANGELOG.md | [Unreleased] 整段晋升为 [0.6.0] - 2026-08-16，按 Keep a Changelog 归并（Added/Changed/Removed/Fixed/Internal 分组重排，条目文字不改）；顶部留空 [Unreleased]；新增发布首段（0.6.0 为首个携带 v2 格式 + v0.6 具名文法 + contacts CRUD 的发布）；[0.5.0] 段内 superseded 注记「in progress, not yet released」更正为「released as 0.6.0 on 2026-08-16」 |
| README.md（根） | Quick start 与 Install 两处版本警示块更新为 0.6.0 实况（crates.io 安装将在发布步骤后可用） |
| repos/paperwork-cli/README.md | Install 节补 0.6.0 版本陈述 |
| SKILL.md | 经核查无版本号陈述与安装引导，无需改动（事实登记） |
| docs/ssot/specs/cli-grammar-v0.6/ 六文档 | 版本状态行补「2026-08-16 状态刷新：owner 批准发布，随 0.6.0 发布」；README 状态行「发布待 owner 裁定」→「发布已获 owner 批准」；spec §7 第 4 款补不发布约束解除注记 |
| --version 断言面 | 经核查：无测试断言 `--version` 输出或 "0.5.0" 版本字面量（char_tests/contacts.rs/thread.rs 中的 `[0.5.0]` 为 CHANGELOG 章节引用，不受 bump 影响）；ci.yml smoke 无版本断言 |

## 四、门禁（FR-2 四件套）

见本任务提交链的门禁实测记录（提交信息内引用）：`cargo test --workspace --locked` 452 全绿 + clippy -D warnings 零警告 + fmt --check 通过 + docs gate（cargo clean -p 后 RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace）通过。

## 五、后续步骤预告

- **任务 #50（发布执行）**：在本准备提交推送且 CI 全绿后——① `git tag v0.6.0`（打在推送后的 master HEAD 上）；② `git push origin v0.6.0` 触发 release.yml（自动产出 GitHub Release + 五平台二进制，release notes 取 CHANGELOG [0.6.0] 段）；③ 本地 `cargo login` 后运行 `.\publish.ps1`（core 先、索引轮询、cli 后）；④ 核验 crates.io 两 crate 0.6.0 可见、`cargo install paperwork-cli` 可用、GitHub Release 资产齐五平台；⑤ 台账 LED-15 事实登记闭合刷新（append-only 新节）。
- **任务 #51（发布后登记）**：发布终态登记、台账联动与轮次收口（以任务书为准）。

（登记完。任务 #49 执行 agent，2026-08-16。）

---

## 六、发布执行证据（任务 #50，2026-08-16 追加，append-only）

未改动第一至五节。以下全部为实测记录：

### 6.1 前置门禁

- CI run **31898321976**（准备提交 ea74948）终局 **success**：fmt ✓ + test（windows/ubuntu/macos）✓ + smoke（三平台）✓，无失败 job。

### 6.2 tag 与 GitHub Release

- tag **v0.6.0** 打在 **ea74948**（master = origin/master，工作区干净），推送成功；未打任何其他 tag。
- Release workflow run **31898532108** 终局 **success**（约 3 分钟）：test 47s → build 五平台矩阵全绿（x86_64-linux-gnu 36s / x86_64-linux-musl 54s / x86_64-apple-darwin 46s / aarch64-apple-darwin 36s / x86_64-pc-windows-msvc 1m30s）→ release job 12s。
- GitHub Release **v0.6.0** 已发布（published 2026-08-15T17:33:37Z，作者 github-actions[bot]）；资产齐五平台：paperwork-v0.6.0-{aarch64-apple-darwin.tar.gz, x86_64-apple-darwin.tar.gz, x86_64-pc-windows-msvc.zip, x86_64-unknown-linux-gnu.tar.gz, x86_64-unknown-linux-musl.tar.gz}；release notes 由 awk 从 CHANGELOG [0.6.0] 段抽取，首段与晋升段一致。

### 6.3 crates.io 发布

- 凭证：`~/.cargo/credentials.toml` 在案（token 不回显）；publish.ps1 退出码 0。
- 顺序实测：paperwork-core 0.6.0 打包（25 文件，385.5KiB）→ verify 编译通过 → uploaded → 索引轮询确认可见 → paperwork-cli 0.6.0 打包（17 文件，407.5KiB）→ verify 从 crates.io 下载 paperwork-core 0.6.0 编译通过（依赖解析实证）→ uploaded。两 crate 均见 `Published ... at registry crates-io`。
- 核验：`cargo search paperwork-core --limit 1` = "0.6.0"；`cargo search paperwork-cli --limit 1` = "0.6.0"。（crates.io REST API 直连返回 403（无 User-Agent 拦截），改以 cargo 索引口径核验，同为权威面。）

### 6.4 安装面与冒烟

- `cargo install paperwork-cli --version 0.6.0 --locked` 成功：release 编译 17.39s，替换旧 paperwork-cli 0.4.0 二进制为 0.6.0。
- 冒烟（临时目录 %TEMP%\paperwork-v060-smoke，装出的二进制）：
  - `paperwork --version` → `paperwork 0.6.0`；
  - `paperwork profile create alice --name alice --model gpt-4o --description "release smoke"` → `ok profile.create alice.profile.md`；
  - `paperwork post send standup --author alice --title "Release Smoke" --message "0.6.0 release smoke test"` → `ok post.send #1 -> standup.post.md`（正文直书，首次 send 建线程）；
  - `paperwork post read standup` → `ok post.read 1 messages`，showing 1/1，window #1-#1，正文逐字回显。

### 6.5 纪律自查

- wip 存档分支零触碰；除 v0.6.0 外未打任何 tag；CHANGELOG [0.6.0] 发布段内容未改动；S-01（crates.io 与仓库版本语义错配）随本次发布实质闭合，台账 LED-15 闭合刷新归任务 #51 台账联动轮。

（执行证据追加完。任务 #50 执行 agent，2026-08-16。）
