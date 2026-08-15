# 0.6.0 发布后独立验证报告（2026-08-16，任务 #51）

- 验证对象：0.6.0 发布声称（任务 #49 准备 + 任务 #50 执行，docs/dev/release-v0.6.0-2026-08-16.md）。
- 基线：本地 master = origin/master = `72c310ea70a335f257ec69a178e1fdee20f4b93a`（fetch 后核对），tag v0.6.0 = `ea74948`；验证开始时工作区干净。
- 方法论：**独立取证**——全部证据由本验证直接采集（git/gh/crates.io REST/cargo install 实测），不引用发布执行者自报；与发布登记文档的比对仅作一致性交叉核验。
- 纪律：未改任何源代码/测试（本报告与台账节落盘除外）；未动任何 tag；wip 存档分支零触碰。

## 总判定表

| # | 检查项 | 判定 |
|---|--------|------|
| 1 | tag 核验（v0.6.0 → ea74948，远端已推） | PASS |
| 2 | GitHub Release（存在、五平台资产、notes 与 CHANGELOG [0.6.0] 逐字一致） | PASS |
| 3 | crates.io 双核验（cargo search + REST API） | PASS |
| 4 | 端到端安装验证（cargo install 默认装 0.6.0 + 25 项冒烟环 + 裁决面） | PASS |
| 5 | 仓库一致性（CHANGELOG/README/SKILL/spec/发布登记证据链/master CI） | PASS |
| 6 | 台账联动（第十九节落盘） | 随本提交完成 |

**最终结论：放行。** 发布声称逐项独立核实成立，无差异项。

## 1. tag 核验 — PASS

- `git fetch origin --tags` 后：本地 `v0.6.0` = `ea7494814e92028d69b10054809326166edd88f8`。
- `git ls-remote --tags origin`：远端 `refs/tags/v0.6.0` 指向同一哈希——tag 已推送。
- tag 列表仅 v0.2.0 / v0.3.0 / v0.4.0 / v0.5.0 / v0.6.0，无多余 tag；HEAD = origin/master = 72c310e，工作区干净。

## 2. GitHub Release — PASS

- `gh release view v0.6.0`：name=v0.6.0，非 draft、非 prerelease，published 2026-08-15T17:33:37Z，作者 github-actions[bot]（release.yml 自动产出，与机制侦查结论一致）。
- **五平台资产齐**（逐名逐大小取证）：
  | 资产 | 字节 |
  |---|---|
  | paperwork-v0.6.0-aarch64-apple-darwin.tar.gz | 1259977 |
  | paperwork-v0.6.0-x86_64-apple-darwin.tar.gz | 1330903 |
  | paperwork-v0.6.0-x86_64-pc-windows-msvc.zip | 1218339 |
  | paperwork-v0.6.0-x86_64-unknown-linux-gnu.tar.gz | 1428042 |
  | paperwork-v0.6.0-x86_64-unknown-linux-musl.tar.gz | 1509008 |
- **notes 一致性（逐字比对）**：`gh release view v0.6.0 --json body` 取全文（UTF-8 落盘后脚本比对）：release body = 「## v0.6.0」标题行 + CHANGELOG.md `[0.6.0]` 段正文；CRLF 归一后 **body 与 CHANGELOG 段正文 12787 字符逐字相同（IDENTICAL）**——awk 抽取链路无内容漂移，含全部 Added/Changed/Removed/Fixed/Internal 分组与两条 owner-ruling correction 注记。
- 过程登记：releases 网页 HTML 因 JS 动态渲染无法直接 grep 资产行（http 200），以 gh API JSON 为权威证据面。

## 3. crates.io 双核验 — PASS

| 面 | paperwork-core | paperwork-cli |
|---|---|---|
| cargo search | `"0.6.0"` | `"0.6.0"` |
| REST `/api/v1/crates/<name>`（带 User-Agent） | max_version=0.6.0，newest_version=0.6.0 | max_version=0.6.0，newest_version=0.6.0 |
| REST `/versions` 首条 | 0.6.0 @ 2026-08-15T17:35:09Z | 0.6.0 @ 2026-08-15T17:35:20Z |

- 时间线自洽：GitHub Release（17:33:37Z）先于 crates.io core（17:35:09Z）先于 cli（17:35:20Z）——与「tag push 触发 release.yml，随后手工 publish.ps1 core→cli」机制一致。
- 补强登记：发布登记 6.3 曾记 REST API 直连 403（无 User-Agent）；本验证带 User-Agent 复打成功，REST 与 cargo 索引两面口径一致。
- **LED-15/S-01 错配消除**：crates.io 最新版即 0.6.0（v0.6 具名文法 + v2 格式），与仓库 HEAD 版本语义一致。

## 4. 端到端安装验证 — PASS

- `cargo install paperwork-cli --locked`（**不加 --version**，验默认装最新）：从 crates.io 下载并编译 paperwork-core v0.6.0 + paperwork-cli v0.6.0，装出 paperwork.exe；`--version` 输出含 **0.6.0**。
- TEMP 夹具冒烟环（%TEMP%\paperwork-task51-smoke，装出的 crates.io 二进制直接驱动）：**25/25 PASS**：
  - profile create → contacts create → contacts add（有效 destination，无 advisory 字段）→ **contacts add 缺失 destination：exit 0 不变 + advisory 文案逐字 `destination 'nowhere/ghost.profile.md' does not exist`**（advisory 触发一例成立）；
  - post send 正文直书 `@alice` / `@#1`（无糖标志）→ post read 正文逐字回显 → 第二条 `@#1 cc @bob` 后 `--json` 读出 **reply_to=1 派生**与 mention 派生；
  - brief create/add/read 全环；validate post/brief 双 exit 0。
- **裁决面（0.6.0 行为面关键断言）**：
  - `post send --reply-to 1` → **usage exit 2**，fix 教学正文 token 迁移路径（@#N/@name）；
  - `post send --mention alice` → **usage exit 2**；
  - `post edit --reply-to 1` → **usage exit 2**；
  - 读侧 `post read --mention alice` 过滤器保留（exit 0）——撤销仅限写侧，与 owner 裁决 O1 逐条一致。
- 结论：crates.io 装出的 0.6.0 行为面与仓库 master 实现一致。

## 5. 仓库一致性 — PASS

| 面 | 实测 | 判定 |
|---|---|---|
| CHANGELOG | [Unreleased] 空；[0.6.0] - 2026-08-16 段在位（12787 字符，与 release notes 逐字同源） | PASS |
| Cargo.toml 双 crate | `version = "0.6.0"` ×2 | PASS |
| README.md（根） | L25-26/L72：0.6.0 为本文档所述发布、0.5.0 superseded 口径在位 | PASS |
| SKILL.md | 经核查无版本号陈述（Grammar (v0.6) 口径），与发布登记「无需改动」事实登记一致 | PASS |
| spec 六文档（docs/ssot/specs/cli-grammar-v0.6/） | 版本状态行均带「2026-08-16 状态刷新：owner 批准发布，v0.6 线以 0.6.0 发布」并指向本发布登记；README 状态行「发布已获 owner 批准」 | PASS |
| release-v0.6.0-2026-08-16.md 证据链 | 六节完整（准备登记 + 任务 #50 执行证据 append-only）；与独立取证交叉一致（tag/release 资产/发布时间线/crates.io 顺序）；其「REST 403」注记由本验证带 UA 复打补强 | PASS |
| master CI @ 72c310e | run **31898829484** conclusion=**success**，逐 job：fmt / test×3 / smoke×3 全 7 job success | PASS |

## 6. 台账联动

- append-only 追加「第十九节：0.6.0 发布终局」于 docs/dev/open-items-ledger-2026-08-15.md：发布事实登记、**LED-15（S-01，crates.io 版本错配）正式闭合**（发布已发生，错配消除，闭合依据为本报告第 3/4 节独立取证）、S-01 闭合确认、发布后开放项终态盘点（仅 LED-16 已裁定备查 + LED-24/25 开放盲区登记，承接方在案）。
- git diff 核验：台账改动为纯追加（零删除行、零既有行修改），符合 append-only 纪律。

## 纪律自查

- 未改任何源代码/测试文件；仅新增本报告与台账追加节。
- 未动任何 tag（v0.6.0 哈希前后一致）；wip/v0.5-perfection-snapshot-2026-08-15 分支零触碰。
- 验证夹具（_verify_tmp51/、TEMP 冒烟目录）验证后清理。

（报告完。任务 #51 验证执行，2026-08-16。）
