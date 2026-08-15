# 端到端验证与全量回归报告 — 2026-08-15（任务 #28）

- 日期：2026-08-15
- 验证对象：master @ da954c2（工作区干净，未推送提交 23 个）
- 改造链：v0.6 具名文法合入（3829fd9，已在 origin）→ perfection P-0~P-9（16 提交：a941b3b…564206a，397 基线）→ 修复波（a81d9ad~da954c2，7 提交：D2/D3/D4/D6 代码修复 + D5 钉住 + D7/A-01/A-02 文档 + 台账）
- 验证纪律：只运行与取证；取证脚手架全部置于 gitignore 的 `_e2e/`（`_*/` 规则），未修改任何源代码与仓库文档（本报告落盘除外）；未做任何 git 提交
- 工具链：cargo/rustc 1.96.0 (30a34c682 2026-05-25)，Windows，PowerShell 7
- 二进制：`cargo build --release --locked` 冷重建产物（release profile）

---

## 一、逐项判定表

### 1. 冷重建全量回归

| 检查项 | 判定 | 证据 |
|---|---|---|
| cargo clean | 通过 | `Removed 5518 files, 1.5GiB total` |
| cargo build --release --locked | 通过 | `Finished release profile [optimized] target(s) in 10.53s`，零警告 |
| cargo test --workspace --locked | 通过 | **410 通过 / 0 失败**，与台账声称分布逐项吻合（见下） |
| cargo clippy --workspace --all-targets --locked -- -D warnings | 通过 | exit 0，零警告 |
| cargo fmt --all --check | 通过 | exit 0，无输出 |

测试分布实测（逐 suite）：

| Suite | 实测 | 台账声称 | 判定 |
|---|---|---|---|
| cli 单元（main.rs） | 6 | 6 | 一致 |
| cli char_tests | 31 | 31 | 一致 |
| cli cli_integration | 141 | 141 | 一致 |
| cli t6_cli_tests | 4 | 4 | 一致 |
| core lib 单元 | 97 | 97 | 一致 |
| core char_tests | 12 | 12 | 一致 |
| core guard_tests | 30 | 30 | 一致 |
| core ops_contacts_crud_tests | 18 | 18 | 一致 |
| core ops_tests | 71 | 71 | 一致 |
| doc-tests | 0 | 0 | 一致 |
| **合计** | **410** | **410** | **一致，全绿** |

### 2. 修复项逐项复现核验（release 二进制，`_e2e/repro-fixes.ps1`，全部现场实测）

| 项 | 判定 | 实测证据 |
|---|---|---|
| D2 send：未闭合 fence 线程 post send | 通过 | exit 1；`error format: Parse error: unclosed code fence (3 backticks) opened at line 5`；fix 声明 `the file was left untouched`；**SHA-256 前后一致（文件零写入）** |
| D2 edit：同一文件 post edit | 通过 | exit 1 同信封；SHA-256 前后一致（零写入） |
| D2 恢复：闭合 fence 后 send/read | 通过 | send exit 0 `ok post.send #2`；read exit 0 `2 messages`，#1/#2 均可见 |
| D3：`--description=` 粘连 `## Scope` 注入 | 通过 | exit 1 validation：`prose embeds a heading-shaped line ('#', '##' or '###')…`；文件不存在（零写入） |
| D3b：`--description=- model: evil` 注入 | 通过 | exit 1 validation：`note starts with an attribute-shaped line '- key: value'`；零写入（历史护栏仍有效） |
| D3 良性路径：show/validate | 通过 | 良性 create exit 0；show 回显 name/model/description 正确；`validate` exit 0 |
| D4：`--scope-read=` 换行注入 | 通过 | exit 1 validation：`scope glob contains a line break; single-line fields cannot span multiple lines`；零写入 |
| D6：非 UTF-8 stdin（0xC0 非法字节，cmd 重定向保真） | 通过 | exit 1；`error validation: Validation error: stdin is not valid UTF-8`；**fix 直指编码**（`re-encode it (e.g. to UTF-8) or pass the body with --message`），无文件权限误导文案；零写入 |
| D1/NEW-1：post send --title 带换行 | 通过 | exit 1 validation `thread title contains a line break`；零写入 |
| D1/NEW-1：brief create --title 带换行 | 通过 | exit 1 validation `title contains a line break`；零写入 |
| D1/NEW-1：contacts create --title 带换行 | 通过 | exit 1 validation `title contains a line break`；零写入 |

### 3. 核心回归冒烟（release 二进制，`_e2e/smoke.ps1`，34 个探针）

| 探针组 | 判定 | 证据摘要 |
|---|---|---|
| profile create/show/edit | 通过 | create `ok profile.create agents/alice.profile.md`；show 回显三字段；edit `changed: model`；二次 show model=gpt-4o-mini，description 保留 |
| contacts 全 CRUD | 通过 | create/add/read/update/remove 全 exit 0；**add 幂等：二次 add 文件 SHA-256 不变**；update 箭头串逐字 `agents/bob.profile.md -> agents/carol.profile.md`（conclusion 与 updated 字段同值）；remove 后 read 计数 2→1；未命中 remove → not-found 信封 + 键口径教学句 |
| post send/read/summary（--author/--message 具名 + 短形式 -a/-m） | 通过 | send #1/#2 exit 0；read `showing: 2/2`，reply:#1 mentions:alice 注入 `@#1 @alice` 回显；summary 五字段齐备；edit #2 后 body 更新且消息数不变 |
| brief create/add/read --full/--entry-title/remove | 通过 | add `entry-src.rs -> onboarding.brief.md`；read 默认档/`--full`（hash+regex）/`--entry-title` 命中档三形态正确；未命中 → not-found + fix 引导 `brief read`；remove 后 `0 entries` |
| validate 四格式 | 通过 | post/profile/contacts/brief 全部 `ok validate` exit 0 |
| --json 信封抽样 | 通过 | post read / contacts read JSON 单行合法、key 齐备；错误路径 JSON 含 category/command/example/exit_code/fix/message/status 七 key，输出于 stdout |
| --quiet 抽样 | 通过 | `-q` send：无 `ok` 状态行，字段（seq/path/sender）仍输出，exit 0 |
| 空键护栏 | 通过 | contacts update `--profile ""` → validation `profile path (--profile) is empty`；空 author → validation `sender name (--author) is empty`；remove 空键 → not-found 信封（键口径教学句在位），均 exit 1 |
| usage 信封 canonical_example 逐字 | 通过 | post send 缺 --author/--message → exit 2，example 逐字 `paperwork post send standup.post.md --author alice --message "Hello"`（与 main.rs L363 逐字一致）；contacts remove 缺 --profile → exit 2，example 逐字 `paperwork contacts remove team.contacts.md --profile alice.profile.md`（与 main.rs L411 逐字一致） |

### 4. 并发抽查（release 二进制 × 16 进程，`_e2e/concurrency.ps1`，集合比较口径）

| 场景 | 判定 | 证据 |
|---|---|---|
| contacts add × 16 并发 | 通过 | 16 个进程 exit 全 0；read --json 条目集 **16/16，与期望集合 Compare-Object 相等（零丢失）** |
| post send × 16 并发（+1 seed） | 通过 | 16 个进程 exit 全 0；消息体集合 **17/17 相等**；seq 空间连续 `1..17` 无空洞无重复；`validate` exit 0 |

### 5. 黄金快照与行为冻结 / 版本纪律

| 检查项 | 判定 | 证据 |
|---|---|---|
| char_tests（行为冻结黄金快照） | 通过 | 单独复跑：cli 31 通过、core 12 通过（并含于 410 全量） |
| 版本号 0.5.0 | 通过 | 两 crate Cargo.toml `version = "0.5.0"`；`paperwork --version` → `paperwork 0.5.0` |
| 最新 tag 仍 v0.5.0 | 通过 | `git tag --sort=-creatordate` 首位 v0.5.0；`git describe --tags --abbrev=0` → v0.5.0 |
| CHANGELOG 无新发布段 | 通过 | 版本段序列 `[Unreleased] / [0.5.0] / [0.4.0] / [0.3.0] / [0.2.0] / [0.1.0]`——修复波/perfection 变更全部落在既有 `[Unreleased]` 段，无 0.6.0 新发布段 |
| git status 干净 | 通过 | 验证开始前 `git status --short` 零输出；取证脚手架位于 gitignore `_e2e/`（`_*/` 规则，check-ignore 命中），不污染状态（本报告落盘为任务交付物，落盘后成为唯一 untracked 文件） |
| 未推送提交盘点 | 通过 | `git rev-list origin/master..master --count` = **23**，与分支状态 `ahead 23` 一致；逐条盘点：perfection 16（a941b3b…564206a）+ 修复波 7（a81d9ad…da954c2），无预期外提交 |

---

## 二、统计

- 探针总数：冷重建 5 项 + 测试 410 例 + 修复复现 15 探针 + 冒烟 34 探针 + 并发 2 场景（32 进程）+ 纪律 6 项 = **全部通过**
- 失败项：**0**
- 与台账/声称不符项：**0**（410 分布逐 suite 吻合；D2~D6/D1 行为逐条吻合；canonical_example 逐字吻合）

## 三、最终结论

**放行。**

全量回归 410/410 全绿且分布与台账逐 suite 吻合；clippy 零警告、fmt 通过；D2/D3/D3b/D4/D6/D1 六组修复项复现全部呈修复后预期形态（fast-fail + 零写入 + hash 证据 + 恢复路径）；核心冒烟 34 探针全绿（含幂等、箭头串、--json/--quiet 信封、空键护栏、usage canonical_example 逐字）；16 并发双场景零丢失（集合口径）；版本纪律五项全部冻结（0.5.0 / v0.5.0 tag / 无新发布段 / status 干净 / ahead 23 逐条可解释）。未发现任何需要阻塞放行的偏差。

备注（非阻塞，供后续轮次参考）：
- S-01（crates.io 0.5.0 语义错配）维持登记态（LED-15），留待发布轮一次性闭合，符合修复波纪律。
- io 类信封（锁获取失败路径）本轮未现场触发，沿用既有测试覆盖口径（与审计矩阵 §7 未覆盖声明一致）。

（报告完。撰写：任务 #28 验证 agent；取证时间 2026-08-15；全部判定基于现场命令输出与 hash/集合比较证据，无推测性结论。）
