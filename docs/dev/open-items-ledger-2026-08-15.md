# agent-paperwork 未决项台账（open items ledger）

- 日期：2026-08-15
- 任务：#22 全量未决项盘点（调研+落盘，零代码变更）
- 取证基线：master @ 55c916a（工作区含未提交的 perfection-plan 变更）；cli-grammar-v0.6 @ a7bc3e2；cli-ux-v0.5 @ 70f7e43；版本 0.5.0 未发布
- 纪律：全部条目基于文件/命令实证；仓库内找不到登记原文的如实标注；已闭合项不展开、仅在闭合清单标注状态

---

## 一、未决项清单（开放/进行中）

字段说明：编号 LED-xx；严重度 = 阻塞/重要/低；状态 = 开放/进行中；处置 = 修 或 钉住为已知限制；负责阶段 = 修复波/文档轮/合并轮/owner 决策。

### LED-01 分支未合 master / 未推 origin
- 来源：git 现场取证（2026-08-15）：`git branch --merged master` 仅 master；cli-grammar-v0.6 领先 master 39 提交、cli-ux-v0.5 领先 10 提交；origin 仅有 master 与 cli-ux-v0.5 两个远程分支（cli-grammar-v0.6 未推）
- 严重度：阻塞（阻塞发布线与全部下游审计任务的基线一致性）
- 状态：进行中（并行任务 #21，Taylor）
- 处置：修——按归属分批合并两分支至 master 并 push origin；合并后 worktree 与本地分支一并清理
- 负责阶段：合并轮（#21）

### LED-02 master 工作区未提交状态（27 改 + 8 新增未跟踪）
- 来源：`git status --short` 实测：27 个 M（含 core/cli 源码与 cli-grammar-v0.6 治理文档修订、四份 contacts-crud doc-review、v0.7_feedbacks 等）；8 个未跟踪（perfection-plan T1-T4 产物：core/cli 两侧 char_tests.rs、guard_tests.rs、t6_cli_tests.rs、ops/lock.rs，及 v0.5-perfection-plan、v0.6 实施 role 等文档）
- 严重度：重要（未提交即不可审计、不可回滚；且与分支合并存在提交边界交织）
- 状态：进行中（perfection-plan T1-T4 执行现场；提交归属由 T11 与 #21 协同裁定）
- 处置：修——按 perfection-plan §8.1 T11「按归属分批提交（修复/测试/文档），排除 v0.6 工作流文件」执行
- 负责阶段：修复波（perfection-plan）+ 合并轮（#21）

### LED-03 qa-tmp/ 未跟踪目录
- 来源：`git status --short` 实测 `?? qa-tmp/`；递归计数 63 个文件/目录（QA 轮临时语料）
- 严重度：低
- 状态：进行中（任务书明示由并行任务 #21 处理）
- 处置：修——合并轮清理删除或并入 .gitignore（二者取一，不留未跟踪悬置）
- 负责阶段：合并轮（#21）

### LED-04 M-2：io 信封中文 OS 错误消息乱码
- 来源：仓库 docs 内未发现登记原文（全仓检索「乱码/转码/非 UTF/系统找不到」仅命中 backlog R-09 历史记载与 closure 报告 ASCII 合规段，均非本项）；实际出处为 v0.6 现场实测验收记录（QA 14 项验收 Notes：Windows OS 错误消息非 UTF-8 未转码，io 信封 detail 出现「系统找不到指定的文件?」）
- 严重度：重要（io 信封 detail 字段是 agent 消费面；乱码破坏纯 ASCII 输出协议承诺，但不影响 exit 码/category/协议结构）
- 状态：进行中（并行任务 #25，Oscar，根因并解决）
- 处置：修——OS 错误消息转码（lossy 或按代码页转 UTF-8），配回归；修复后须补仓库内登记（本台账 + CHANGELOG Unreleased）
- 负责阶段：修复波（#25）

### LED-05 v0.5-perfection-plan 闭合批（执行链进行中）
- 来源：docs/reviews/v0.5-perfection-plan-2026-08-15.md（§1 债务对照、§3 NEW-1~NEW-13、§4 SAM-1~SAM-5、§8 执行链 T0 至 T11）
- 严重度：重要（owner「零悬置」指令承载批；含 TOCTOU、注入护栏等正确性项）
- 状态：进行中
- 实测进度痕迹（2026-08-15 取证）：T1 行为锁定已落盘（core 与 cli 两侧 char_tests.rs、guard_tests.rs、t6_cli_tests.rs 均在场，未跟踪）；T4 迁移中（ops/thread.rs 实测 713 行，plan 基线 780 行）
- T2、T3 部分落地：ops/profile.rs L25 NEW-2 注释与 L70 SAM-2 注释、cmd/mod.rs L26 NEW-3 注释、ops/lock.rs 在场；T9 终验、T10 三路 CodeReview、T11 提交与推送未见执行痕迹
- 处置：修——按 plan 闭合通则，每项只有「修复+回归」或「核实无需改+实测证据」两种终态，完成后回流更新 plan 台账
- 负责阶段：修复波（对应任务链 #23 至 #30）

### LED-06 cli-grammar-v0.6 doc-review 闭合轮观察项 O-1：tdd §1b-G 注记理由失实
- 来源：docs/reviews/cli-grammar-v0.6-doc-review-closure-2026-08-09.md 非阻塞观察项 O-1（tdd §1b-G 表后注记称「example 已含 --entry/--profile 形态」与事实不符，建议实施时勘误）
- 严重度：低（纯文档勘误，不影响行为）
- 状态：开放
- 处置：修——文档轮勘误该注记
- 负责阶段：文档轮

### LED-07 cli-grammar-v0.6 doc-review 闭合轮观察项 O-2：销账计数笔误
- 来源：同上文件 O-2（销账段「共 31 条」实为 28 行，算术笔误）
- 严重度：低
- 状态：开放
- 处置：修——文档轮勘误计数
- 负责阶段：文档轮

### LED-08 contacts-crud 闭合复核备查项：写路径计数口径
- 来源：docs/reviews/contacts-crud-doc-review-closure-2026-08-09.md 备查记录（research 文档 L87/L151「五个/五处写路径」与下游「六写路径」口径不同维；rework 已按六处统一，仅历史调研文本未回改；「如后续轮次追求计数口径绝对统一，可在发布轮顺带注明，不阻塞」）
- 严重度：低
- 状态：开放
- 处置：修（发布轮顺带注明口径）或钉住为历史文本不改——建议前者，成本极低
- 负责阶段：文档轮（发布轮顺带）

### LED-09 backlog B-01：--reply-to 指向不存在 seq 静默跳过
- 来源：docs/researches/ux-open-items-backlog-2026-08-08.md §八 B-01（Pete N3；v0.5 冻结行为，spec v0.6 §3.1 错误映射沿用；候选方向 ok 信封 reply-dropped 字段需解冻输出协议，与 F6 ignored 字段同批评估）
- 严重度：低（冻结行为登记，与 Q-02 失败自愈存在张力但已有 spec 登记）
- 状态：开放（登记为「供发布轮或后续 UX 线裁决」）
- 处置：owner 决策——修复（解冻协议增字段）或钉住为已知限制；不可无人跟进
- 负责阶段：owner 决策（发布轮）

### LED-10 backlog B-02：contacts add/update 写前 destination 存在性校验候选
- 来源：同上文件 §八 B-02（Ryan M-3 ④；行为本轮不改，声明与钉住已落 spec §3.6 + bdd S-CONTACTS-14；候选：写前校验或 ok 信封 destination-unverified 回显字段）
- 严重度：低（静默面已声明钉住，agent 路径笔误时 exit 0 为已知契约）
- 状态：开放（登记为「供发布轮裁决」）
- 处置：owner 决策——维持现状钉住或增补回显字段（涉输出协议冻结评估）
- 负责阶段：owner 决策（发布轮）

### LED-11 backlog U-04：--mention 与正文 @alice 冗余（正文自动提取）
- 来源：同上文件 §一 U-04（ux-review §4；v0.6 治理文档集 README.md L31 将其列入「延后项」沿用，design.md §8 未列拒绝亦未列解决）
- 严重度：低
- 状态：开放（延后）
- 处置：owner 决策——钉住为已知限制（现有 --mention 显式面已满足 agent 消费）或后续 UX 线立项
- 负责阶段：owner 决策（发布轮顺带裁定）

### LED-12 backlog U-13：shell completions
- 来源：同上文件 §一 U-13（P3；「延后：仅人类用户受益」；v0.6 README L31 延后项沿用）
- 严重度：低
- 状态：开放（延后）
- 处置：钉住为已知限制（agent-first 产品定位下收益低，理由充分）——建议本轮正式钉住结案，不再悬置
- 负责阶段：owner 决策（发布轮顺带裁定）

---

## 二、代码扫描统计（仅统计，不修改）

取证命令：rg 全量检索 repos/ 下 *.rs（2026-08-15 实测）。

### 2.1 遗留标记注释（TODO 类）
- 检索 TODO、FIXME、HACK、XXX 四类标记：全仓 0 命中（未发现任何遗留标记注释）。

### 2.2 unwrap 调用统计（共 213 处）
- src 生产代码仅 4 处：cmd/mod.rs 1、format/manifest.rs 2、format/thread.rs 1
- 其余 209 处全部在测试代码：cli_integration.rs 157 处、cli 侧 char_tests.rs 40 处
- guard_tests.rs 8 处、t6_cli_tests.rs 4 处

### 2.3 expect 调用统计（共 501 处）
- src 生产代码 106 处，集中点：format/thread.rs 35、ops/lock.rs 29、format/contacts.rs 28、format/manifest.rs 15
- format/profile.rs 11、ops/thread.rs 5、format/mod.rs 2、cmd/validate.rs 1
- 其余约 395 处在测试代码，最大集中点 ops_tests.rs 252 处，其余散布于 guard_tests、char_tests、t6_cli_tests、cli_integration 各测试文件
- 说明：src 侧 expect 多为静态正则编译与解析不变量断言，历轮评审均未报 panic 风险面；本节仅为统计登记，不构成独立未决项

---

## 三、钉住为已知限制（状态=已闭合-接受，非开放项，登记备查）

| 编号 | 事项 | 来源 | 依据 |
|---|---|---|---|
| KL-1 | 尾扫缓冲切断围栏的奇偶残留（seq 误判窗口极小） | docs/reviews/v0.5-review-2026-08-09.md §6 ISSUE-1 | 已有 validate 兜底；perfection-plan NEW-6 补测试 |
| KL-2 | thread_edit 锁内 truncate+rewrite 崩溃窗口（断电/杀进程丢全文件） | 同上 ISSUE-2 | format-v2 spec §5.7 已接受判例，v0.7_feedbacks §2.2.5 沿用 |
| KL-3 | brief hash 对换行敏感（内容等价但换行不同则 hash 不同） | 同上 ISSUE-3 | 已文档化；hash 用途为内容指纹非语义等价判定 |
| KL-4 | spec F6：--title 对既有线程静默忽略（exit 0 无信号） | docs/ssot/specs/cli-grammar-v0.6/spec.md L97 | rework 裁定 F6 登记，bdd S-SEND-17 钉住；ignored 字段候选与 B-01 同批评估 |

---

## 四、已闭合项状态标注（不收录为未决项，仅备查）

| 来源 | 事项 | 闭合证据 |
|---|---|---|
| contacts-crud doc-review 四份（closure/agent-ux/feasibility/ssot） | 24 条 findings | closure 报告 24/24 销账吻合，最终放行 |
| contacts-crud code-review 三份（completeness/correctness/impact，位于 cli-grammar-v0.6 分支 docs/reviews/） | m-1/2/3、M-1/2/3、m-1/2 等全部 findings | 各报告「修复回应销账段」全部已销账，274 测试全绿 |
| v0.6 code-review 三份（同分支） | M1-M3、m1-m7、C-B 等全部 findings | 统一修复轮销账段全部「已销账」 |
| format-v2-final-review 四份（merged/kim/paul/ray） | F1-F5 全部 findings | 终审放行，无残留登记 |
| cli-grammar-v0.6 doc-review 四份（feasibility/agent-ux/ssot/closure） | 全部 rework 项 | closure 报告最终闭合放行（仅余 O-1/O-2 观察项，见 LED-06/07） |
| cli-ux-redesign doc-review closure NF-1 | 两线程版本顺序裁定 | 已由「均并入 0.5.0、v0.6 不发布」事实解决 |
| cli-ux-redesign closure NF-2 | `--` 负形态 BDD 残留 | v0.6 轮 allow_hyphen_values 废止 `--` 教学后实质淘汰 |
| cli-ux-redesign closure NF-3 | tdd §3 缺 6 行场景映射 | v0.6 closure 报告实测「tdd §4 用例映射闭环，无 NF-3 型缺口」 |
| v0.5-review §6 ISSUE-4 | target-review/ 构建产物残留 | 实测 Test-Path target-review = False，目录已不存在，闭合 |
| v0.5-review §6 ISSUE-5 | v0.4 旧文件静默空读 | v0.6 code-review correctness M1 销账：read/summary 加 reject_foreign_thread 守卫（cli-grammar-v0.6 分支，随 LED-01 合并生效） |
| v0.5-full-review B1 + 8 MAJOR + 15 MINOR | 全部 findings | commit 67eb049 修复 + 55c916a review book 落盘；其 §3.4 延期项已被 perfection-plan 升级为本轮闭合（见 LED-05） |
| feedbacks 三文件 | U-02 env 回退提案 | v0.5_feedbacks L42 裁决拒绝；v0.6/v0.7 feedbacks 无新增未决项（仅延续不发布约束） |

---

## 五、backlog（ux-open-items-backlog-2026-08-08）逐条现状裁定

裁定基线说明：v0.6 具名文法的代码实现位于 cli-grammar-v0.6 分支（未合 master）；master 工作区代码面仍为 v0.5 文法（实测：post.rs 无 --author 命中、send 信封无 implicit-mention 字段、无 showing/window 恒显、validate.rs 无 --type、main.rs 别名仅 p/b/c/v）。故下表「已解决」凡属 v0.6 轮成果者均标注「待合并」（随 LED-01 生效）。

状态图例：已解决 / 已解决-拒绝（裁决拒绝）/ 已解决-接受现状 / 开放-延后 / 开放-待裁。

### 5.1 §一 U-01 至 U-15

| 编号 | 原描述（摘要） | 现状裁定 | 证据 |
|---|---|---|---|
| U-01 | --from 双语义冲突 | 已解决（待合并）：v0.6 身份改 --author/-a，--from 仅存于 post read 过滤 | cli-grammar-v0.6 分支 post.rs author 23 处命中；spec §2 |
| U-02 | PAPERWORK_AGENT env 回退 | 已解决-拒绝 | v0.5_feedbacks L42；v0.6 design.md §8 拒绝清单；代码全仓无 env::var |
| U-03 | post create/send 双轨创建 | 已解决：v0.5 format-v2 轮删除 post create，send 锁内首写 preamble | v0.5-review §1.3 |
| U-04 | mention 正文自动提取 | 开放-延后 | v0.6 README L31 延后项登记（见 LED-11） |
| U-05 | 内容优先/路径可省略 | 已解决-拒绝 | backlog 原建议拒绝；v0.6 design.md §8 拒绝清单（PATH 恒必填） |
| U-06 | profile create --name 与路径名冗余 | 已解决（方向消解）：v0.6 裁决位置参数仅剩 PATH、名字不位置参数化，--name 必选具名 flag 为定案 | v0.6 spec §1 三规则；backlog 未列入延后/拒绝清单即视为消化 |
| U-07 | brief/contacts add 主载荷位置参数化 | 已解决（反向裁定）：v0.6 裁决主载荷一律具名 flag（--entry/--profile），位置参数化提议作废 | v0.6 spec §1/§2 |
| U-08 | 命令别名重排 | 已解决（待合并）：v0.6 分支别名 p/b/c/v/po | v0.6 README L30；master 实测仅 p/b/c/v（main.rs L38-54） |
| U-09 | summary 并入 read --summary | 已解决-拒绝（保留独立动词结案） | backlog §一 U-09 原裁决；v0.7_feedbacks §2.4 post 面盘点再次确认 |
| U-10 | 隐式 mention 输出不可见 | 已解决（待合并）：v0.6 输出增补 implicit-mention 字段（单数、仅触发时出现） | v0.6 bdd.md L40-41、spec §3.1 L98、tdd L144 冻结；master 信封尚无该字段（post.rs L218-221 实测） |
| U-11 | read 无截断提示 | 已解决（待合并）：v0.6 恒显 showing n/total 与 window 区间 | v0.6 tdd L144 字段冻结（showing/window）；master 仅超限时显示 showing（post.rs L266-284 实测） |
| U-12 | help 全局 flag 噪音 | 已解决-接受（backlog 原裁决：clap 惯例，无需动作） | backlog §一 U-12 |
| U-13 | shell completions | 开放-延后 | backlog §一 U-13；v0.6 README L31 延后项（见 LED-12） |
| U-14 | 后缀解析原路径优先 | 已解决（待合并）：v0.6 分支 ensure_suffix 已改三级解析（原路径 is_file 优先 -> 后缀变体存在 -> 落点路径），见 cli-grammar-v0.6 cmd/mod.rs 实测；master 工作区为 NEW-3 OsStr 无损版但仍属无条件改写，随合并被 v0.6 版取代 | git show cli-grammar-v0.6 cmd/mod.rs 三级解析注释与实现 |
| U-15 | validate --type | 已解决（待合并）：v0.6 README L30 收录 validate --type；v0.6 code-review argv_wants_json flag 集含 --type | v0.6 README L30；master validate.rs 尚无（L34-48 实测） |

### 5.2 §二 R-01 至 R-14

| 编号 | 现状裁定 | 证据 |
|---|---|---|
| R-01 | 已解决（并入 U-03） | v0.5 format-v2 删除 post create，系统消息不再占 #1 |
| R-02 至 R-07 | 已解决（backlog 原标注已修复，v0.2-v0.4 各版） | backlog §二原表 |
| R-08 | 已解决-拒绝 | v0.6 design.md §8 拒绝清单；输出已纯 ASCII 无 ANSI |
| R-09 至 R-14 | 已解决（backlog 原标注已修复/已消化） | backlog §二原表 |

### 5.3 §三 F 系列 / §四 Q 系列 / §五 N 系列

| 编号 | 现状裁定 | 证据 |
|---|---|---|
| F-01 至 F-08、F-10 | 已实现（v0.2-v0.4，backlog 原标注） | backlog §三原表 |
| F-09 | 已解决-接受现状（validate 不校验正文内 markdown，围栏透传为设计） | backlog §三 F-09 裁决建议；v0.6 design.md §8 拒绝清单收录 |
| Q-01、Q-02 | 已兑现（backlog 原标注） | backlog §四原表 |
| Q-03 | 已兑现（唯一重大违背 U-01 已由 v0.6 消除，待合并） | 见 U-01 |
| Q-04 | 已兑现（依赖的 U-02 拒绝结案、U-07 反向裁定结案） | 见 U-02/U-07 |
| N-01 | 已解决（并入 U-08，v0.6 别名表 p/b/c/v/po，待合并） | 见 U-08 |
| N-02 | 已解决（并入 U-14，v0.6 三级解析，待合并） | 见 U-14 |
| N-03 | 已解决（并入 U-03，send 首写携带 --title/--participants 元数据） | v0.5-review §1.3 |

### 5.4 §八 v0.6 rework 补录项

| 编号 | 现状裁定 | 证据与去向 |
|---|---|---|
| B-01 | 开放-待裁（发布轮/owner） | 见 LED-09 |
| B-02 | 开放-待裁（发布轮/owner） | 见 LED-10 |

---

## 六、汇总表

未决项（开放+进行中）共 12 项：

| 编号 | 标题（简） | 严重度 | 状态 | 建议负责阶段 |
|---|---|---|---|---|
| LED-01 | 分支未合 master / 未推 origin | 阻塞 | 进行中 | 合并轮（#21） |
| LED-02 | master 工作区未提交状态 | 重要 | 进行中 | 修复波 + 合并轮 |
| LED-03 | qa-tmp/ 未跟踪 | 低 | 进行中 | 合并轮（#21） |
| LED-04 | M-2 io 信封中文 OS 错误乱码 | 重要 | 进行中 | 修复波（#25） |
| LED-05 | perfection-plan 闭合批 | 重要 | 进行中 | 修复波（#23 至 #30） |
| LED-06 | v0.6 doc-review O-1 tdd 注记勘误 | 低 | 开放 | 文档轮 |
| LED-07 | v0.6 doc-review O-2 计数勘误 | 低 | 开放 | 文档轮 |
| LED-08 | 写路径计数口径备查 | 低 | 开放 | 文档轮（发布轮顺带） |
| LED-09 | B-01 reply-to 静默跳过裁决 | 低 | 开放 | owner 决策 |
| LED-10 | B-02 destination 校验候选裁决 | 低 | 开放 | owner 决策 |
| LED-11 | U-04 mention 自动提取延后项 | 低 | 开放 | owner 决策 |
| LED-12 | U-13 completions 延后项 | 低 | 开放 | owner 决策 |

统计摘要：

- 未决项总数：12（开放 7，进行中 5）
- 按严重度：阻塞 1（LED-01）；重要 3（LED-02、LED-04、LED-05）；低 8
- 已在处理通道内的：5 项（LED-01 至 LED-05 分别由 #21/#25/修复波承接）；真正无人认领的开放项：0（LED-06 至 12 均已在本台账指定建议负责阶段，其中 4 项需 owner 在发布轮一次性裁决）
- 已闭合备查：KL-1 至 KL-4 钉住项 + 第四节闭合清单 + backlog 已解决项（含拒绝/接受现状/待合并）
- 代码面遗留标记：TODO/FIXME/HACK/XXX 零命中；unwrap/expect 集中点见第二节（src 生产代码 unwrap 仅 4 处）

裁定纪律说明：

1. 「待合并」口径——v0.6 轮与 contacts CRUD 轮成果均已评审销账但位于未合并分支，合并前 master 代码面仍为 v0.5 文法；本台账将其计为已解决但依赖 LED-01 生效，避免重复立项。
2. M-2 乱码项在仓库 docs 内无登记原文，出处为 v0.6 现场验收记录；本台账即为其首个仓库内登记点。
3. contacts-crud code-review 三份与 v0.6 code-review 三份报告不在 master 工作区，位于 cli-grammar-v0.6 分支 docs/reviews/；master 侧检索为「未发现」属分支未合并所致，非缺失。

---

（台账完。盘点人：任务 #22 调研 agent；取证时间 2026-08-15；全部结论基于当日上游文件与命令实测。）

---

## 七、任务 #31 补充登记（2026-08-15 追加，append-only，未改动第一至六节）

追加依据：docs/dev/perfection-and-branches-assessment-2026-08-15.md（同日落盘）。取证基线补充：master @ 3829fd9（cli-grammar-v0.6 三方合并已完成，288 测试）；wip/v0.5-perfection-snapshot-2026-08-15 @ d679b9a。

### LED-13 cli-ux-v0.5 分支处置
- 来源：任务 #31 只读核查（git 实测）：cli-ux-v0.5 @ 70f7e43 为 cli-grammar-v0.6 的直系祖先（merge-base = 其顶点），十提交无独有内容；其 v0.5 位置参数文法已被 v0.6 具名文法整体取代（d6e9ff3 重写、旧文法落入 usage 信封 exit 2 拒绝路径）；规格集 docs/ssot/specs/cli-ux-redesign 已随合并成为历史治理档案（adr-v1 两次 superseded 注记）
- 严重度：低（无内容风险，纯分支卫生）
- 状态：开放（待合并轮清理执行）
- 处置建议：废弃，不做独立合并——git branch --merged master 确认含 70f7e43 后移除本地分支，并移除 origin/cli-ux-v0.5 远端引用（owner 亦可裁决保留远端存档仅清本地）；cli-ux-redesign 规格集与评审书原地保留为历史档案，不随分支移除
- 负责阶段：合并轮（LED-01 收尾动作，随 push origin 一并执行）

### LED-14 perfection 闭合批续做（任务书口径 T4/T9/T10/T11；核查实测完整续做面为 P-0 至 P-9）
- 来源：任务 #31 只读核查（对 docs/reviews/v0.5-perfection-plan-2026-08-15.md 与 wip/v0.5-perfection-snapshot-2026-08-15 @ d679b9a 的逐批实证）：T1–T4 成果已落快照分支（T4 达终态：ops/thread.rs 647 行、d679b9a 明示 T4 final state）；T5/T9/T10/T11 未动；T6/T7/T8 部分；两处硬冲突——锁层两代设计（wip LockedFile RAII 251 行 vs 合并后 master locked_read_modify_write 127 行）与 ensure_suffix 两代（分支三级解析仍 lossy vs wip OsStr 无损）
- 严重度：重要（owner 零悬置指令承载批；T4 迁移与 T2 护栏含 TOCTOU、注入防护等正确性项；快照分支未推 origin 存在单机丢失风险）
- 状态：开放（等待修复波按工单派工）
- 处置建议：按评估报告第五节工单执行——执行序 P-0 基线重定 → P-1 锁层融合（分支版为 SSOT）→ P-5 黄金快照 v0.6 口径重冻（行为锁定先行）→ P-2/P-3/P-4 护栏与迁移移植（含 SAM-5 Io 变体移除及 CHANGELOG 披露）→ P-6 JSON 收口与 ensure_suffix 融合 → P-7 拆分性能批 → P-8 文档 CI 批 → P-9 终验/评审/提交推送；每批沿用闭合通则（修复+回归 或 核实无需改+实测证据）；wip 快照分支在成果全部回流前不得删除，建议尽早推送 origin 保全
- 负责阶段：修复波（派工依据 = 评估报告第五节；T9/T10/T11 门禁收口归修复波末段）

---

## 八、任务 #34 文档轮追加（2026-08-15，append-only，未改动第一至七节）

追加依据：深审 C 报告 docs/dev/audit-ssot-agentux-2026-08-15.md S-12（ledger/backlog「待合并」口径过期；审计不提交、不修改历史裁定，由本文档轮追加刷新）。

### 口径刷新：「待合并」一律改读为「已随 master @ 3829fd9 合入」

- 事实：cli-grammar-v0.6 三方合并已于 2026-08-15 完成（master @ 3829fd9，288 测试基线）；LED-01 的分支合并面已完成（推送 origin 与 worktree/本地分支清理的收尾动作归属不变，仍记合并轮）。
- 本台账第五节裁定基线说明（「v0.6 具名文法的代码实现位于 cli-grammar-v0.6 分支（未合 master）；master 工作区代码面仍为 v0.5 文法」）与裁定表中全部「待合并」标注随合入而过期：U-01/U-08/U-10/U-11/U-14/U-15/Q-03/N-01/N-02 等凡标「已解决（待合并）」者，一律读作「已解决（已随 3829fd9 合入 master）」。
- 交叉验证（审计实测，PROBE 面）：po 别名、implicit-mention 字段、showing/window 恒显、validate --type、ensure_suffix 三级解析均已在 master 在场；「待合并」依赖条件已消失。
- backlog（docs/researches/ux-open-items-backlog-2026-08-08.md）本体无「待合并」字样（grep 实证），其 B-01/B-02「供发布轮裁决」状态不受合入影响，裁定原样有效，无需追加。
- 纪律：本追加不改写第一至七节任何字句；全部裁定内容本身仍成立，仅时态/状态口径刷新（与 S-12 交叉验证结论一致）。
