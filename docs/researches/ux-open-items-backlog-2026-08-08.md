# Agent Paperwork CLI UX 未落地事项 Backlog

- 日期: 2026-08-08
- 目的: 完整 CLI UX 重设计(路径与使用者名字前置为必填位置参数、每个 tool 独立设计 UX)前, 一次性盘点并裁决全部历史 UX 遗留项
- 核验基线: v0.4.0 源码 repos/paperwork-cli/src/ (main.rs, output.rs, cmd/*.rs), 辅以 cli-ux-agent-visible-output-research-2026-08-08.md 实测结论
- 状态口径: 已修复=某版本已落地; 未实现=代码无对应实现; 部分=部分实现或以变形实现

---

## 一、v0.4-ux-review 提议逐项核验 (原文 15 节 / 优先级矩阵 13 项)

| 编号 | 来源 | 问题描述 | 当前代码状态 | 建议处置 |
|---|---|---|---|---|
| U-01 | ux-review §1 (P0) | --from 语义冲突: post send/edit 中=发送者身份, post read 中=seq 范围起点 | 未实现: cmd/post.rs 两处仍同名 | 本次必须解决: 身份改用 --as 或前置位置参数名字 |
| U-02 | ux-review §2 (P0) | PAPERWORK_AGENT 环境变量回退, 免去每条命令重复身份参数 | 未实现: 全仓库无 env::var 引用 | 需裁决: 与"显式优于隐式"有张力, 建议与 U-01 一并裁决 |
| U-03 | ux-review §3 (P1) | post create 与 send 自动创建双轨; 系统消息占 #1 致 off-by-one | 未实现: create 仍注入 [Thread created:...] 为 #1 | 本次必须解决: 统一线程创建语义 (v0.2/v0.3/v0.4 连续三版遗留) |
| U-04 | ux-review §4 (P1) | --mention 与正文 @alice 冗余; 提议从正文自动提取 | 未实现: send 不解析正文 @ | 本次裁决: 建议自动提取, --mention 降级为补充 |

> U-04 注记（2026-08-15 owner 裁决，append-only）：owner 裁决「和第一个问题一样处理, cli 这边不要这种语义参数」——写侧 `--mention` 随 B-01 同批撤销（post send 传入落 usage exit 2），reply/mention 语义由 agent 在正文直书 `@name`/`@#N` 表达，读取时 derive 机制不变；本条「自动提取 vs 显式 flag」的冗余张力随 flag 撤销方向消解销账。读侧 post read `--mention` 为查询过滤器，不在撤销范围。权威口径：docs/dev/owner-rulings-2026-08-15.md；台账闭合：open-items-ledger 第十三节 LED-11。
| U-05 | ux-review §5 | 内容优先/路径可省略 | 未实现 | 建议裁决拒绝: 与本次"路径前置必填位置参数"方向直接冲突 |
| U-06 | ux-review §6 (P2) | profile create 中 --name 与路径名冗余 | 未实现: --name 仍必填 | 本次解决: 并入"名字位置参数化"设计 |
| U-07 | ux-review §7 (P1) | brief add --entry / contacts add --profile 主载荷应为位置参数 | 未实现: 两者仍是 flag | 本次必须解决: 位置参数化 |
| U-08 | ux-review §8 (P2) | 命令别名 (post->p, profile->pr, 子命令 s/r/sum) | 部分: main.rs 有 p/b/c/v, 但 p=profile, post 无别名, 无子命令别名 | 本次裁决: 重排别名表 (见 N-01) |
| U-09 | ux-review §9 (P3) | summary 并入 read --summary | 未实现: summary 仍独立 | 裁决: review 自评可接受现状, 建议保留独立命令结案 |
| U-10 | ux-review §10 (P2) | reply-to 隐式 mention 原发送者, 输出不可见 | 未实现: send 输出仅 seq/path/sender | 本次解决: 输出增加 implicit-mentions 字段 |
| U-11 | ux-review §11 (P2) | read 无限量时输出不提示截断; 建议 20/50 + seq 区间 | 部分: 超限时有 showing: n/total, 但无 seq 区间, 未超限时无提示 | 本次补全: 恒显示 n/total 与窗口区间 |
| U-12 | ux-review §12 | help 中全局 flag 噪音 | 无需动作 (clap 惯例) | 已裁决接受, 结案 |
| U-13 | ux-review §13 (P3) | shell completions | 未实现 | 延后: 仅人类用户受益 |

> U-13 注记（2026-08-15 owner 裁决，append-only）：owner 裁决「不需要, 这个 cli 主要是 agent 自己使用, 按照 agent 的使用习惯去做 qol/UX」——completions 钉住结案（不再作延后悬置项），同时确立长期方向「UX/QoL 以 agent 使用习惯为准」。权威口径：docs/dev/owner-rulings-2026-08-15.md；台账闭合：open-items-ledger 第十三节 LED-12。
| U-14 | ux-review §14 (P1) | 通用后缀自动解析: 先试原路径, 再补类型后缀 | 部分/变形: ensure_suffix 无条件改写路径(不试原路径), 传 standup.md 会被改写为 standup.post.md | 本次解决: 改为"原路径优先, 再补后缀" (见 N-02) |
| U-15 | ux-review §15 (P3) | validate --type 覆盖 flag | 未实现: 未知后缀直接报 format 错误 | 裁决: 边缘场景, 建议延后或本次顺带 |

## 二、v0.2 / v0.3 / v0.4 review 遗留项

| 编号 | 来源 | 问题描述 | 当前代码状态 | 建议处置 |
|---|---|---|---|---|
| R-01 | v0.2 §5.2-1, v0.3 §4.2, v0.4 ISSUE-3 | 系统消息占 #1, off-by-one, --reply-to 1 误回系统消息 | 未实现 (即 U-03) | 并入 U-03 本次解决 |
| R-02 | v0.2 §5.2-3, v0.3 §4.2 | 无 --stdin, 多行正文难传 | 已修复 (v0.4, send/edit 均支持) | 结案 |
| R-03 | v0.2 BUG-3 | 并发 send 回执 seq 错误 | 已修复 (fs2 文件锁内定 seq, ops_tests 覆盖) | 结案 |
| R-04 | v0.2 BUG-4, v0.3 BUG-3 | 空正文被静默接受 | 已修复 (v0.4 validation 错误) | 结案 |
| R-05 | v0.2 BUG-5, v0.3 BUG-4 | contacts read 不显示 profile 简介 | 已修复 (v0.4 富化 name+description) | 结案 |
| R-06 | v0.3 BUG-1 (Critical) | validate 接受垃圾内容 | 已修复 (v0.4 真实解析) | 结案 |
| R-07 | v0.3 BUG-2 | post read --plain 忽略 --from/--to | 已修复 (v0.4) | 结案 |
| R-08 | v0.2 §5.2-4, v0.3 §4.2, v0.4 ISSUE-4 | 无 --no-color / NO_COLOR | 未实现 (但 v0.4 输出已纯 ASCII, 无 ANSI 码) | 裁决: 实质影响已消失, 建议裁决拒绝或低优延后 |
| R-09 | v0.2 §5.2-8, v0.3 §4.2 | Windows 终端 ✗/→ 乱码 | 已修复 (v0.4 全 ASCII 化) | 结案 |
| R-10 | v0.2 §5.2-7 | brief 文件头 "# Manifest:" 命名不一致 | 已修复 (v0.3 改为 "# 标题" + "- Owner:") | 结案 |
| R-11 | v0.2 §5.2-2, v0.3 §4.2 | profile show 默认非丰富渲染 / 与文件本体不一致 | 已由 v0.4 envelope 重设计消化 (结构化字段视图) | 结案 |
| R-12 | v0.2 BUG-1/BUG-2, v0.3 §4.2 | contacts read 空白 / profile list 不过滤 / list 无结构 | 已修复 (v0.3 过滤, v0.4 结构化 name/model) | 结案 |
| R-13 | v0.4 ISSUE-1/ISSUE-2 | summary title 未解析 / brief --full 缺 hash/regex/note | 已修复 (v0.4, cmd/post.rs 与 cmd/brief.rs 核验) | 结案 |
| R-14 | v0.3 §4.2 | post read 默认隐藏时间戳 | 已修复 (v0.4 头行含时间戳) | 结案 |

## 三、v0_feedbacks 消化状态

| 编号 | 来源条目 | 内容 | 状态 |
|---|---|---|---|
| F-01 | 主 ADR | 无 .paperwork 托管目录, 无状态, 任意路径操作任意文件 | 已实现 (v0.2 架构) |
| F-02 | 主 ADR | 无登录语义, 只给名字 | 已实现 (--from / name 参数) |
| F-03 | 追加 feedback | 删除 DM, GDM(post) 统一通信 | 已实现 (v0.2) |
| F-04 | 主 ADR | contact 是特殊 brief, 读取时显示路径 + profile 简介 | 已实现 (v0.4 富化输出) |
| F-05 | 主 ADR | brief 与 profile 无关联, 最多 owner name | 已实现 (--owner 可选) |
| F-06 | v0.2 feedback #1 | 各文件使用类型后缀 (.profile.md 等) | 已实现 (v0.3) |
| F-07 | v0.2 feedback #2 | 正规简洁的 Markdown 结构组织 | 已实现 (v0.3 bullet 格式) |
| F-08 | v0.2 feedback #3.1 | content 参数放最后便于多行输入 | 已实现 (body 末位位置参数 + --stdin) |
| F-09 | v0.2 feedback #3.2 | 输入后给 markdown validation 机制 | 部分: validate 只校验托管文件结构, 不校验正文内 markdown 语法(正文按围栏透传); 建议本次裁决接受现状 |
| F-10 | v0.2 feedback #3.3 | 正文以 fenced code block(markdown) 包裹 | 已实现 (v0.3 四反引号) |

## 四、agent-ux-qol 承诺兑现状态

| 编号 | 承诺 | 状态 |
|---|---|---|
| Q-01 | 快速清晰得知操作结果结论 | 已兑现: ok 首行 + conclusion + 退出码 |
| Q-02 | 失败快速得知 + 得知如何修改 | 已兑现: error 分类 + fix + example |
| Q-03 | 输出语义明确、风格一致、不产生疑惑 | 大体兑现; 唯一重大违背 = U-01 (--from 双语义), 本次必须消除 |
| Q-04 | agent 轻松快速进行各种操作 | 部分: 身份重复税(U-02)与 flag 冗余(U-07)未解决; 本次重设计(位置参数前置)正面回应 |

## 五、本次核验新发现 (历史 review 未列出)

| 编号 | 描述 | 建议处置 |
|---|---|---|
| N-01 | 别名冲突: ux-review 建议 post->p, 但 p 已分配给 profile, post 当前无别名; 若重设计引入新命令名需重排全部别名 | 本次裁决别名表 |
| N-02 | ensure_suffix 为无条件改写而非"先试原路径": 用户传入恰好存在的 x.md 会被改写为 x.post.md 后报 not-found | 并入 U-14 本次解决 |
| N-03 | send 自动创建线程无 title/participants 元数据, 与 create 行为分叉, summary 对自动创建线程返回空 title | 并入 U-03 |

## 六、未解决项严重程度汇总

| 严重度 | 项目 |
|---|---|
| 高 | U-01 (--from 冲突); U-03/R-01/N-03 (双轨创建+系统消息); U-07 (主载荷非位置参数) |
| 中 | U-02 (env 回退, 需裁决); U-04 (mention 自动提取); U-10 (隐式 mention 不可见); U-14/N-02 (后缀解析); U-08/N-01 (别名) |
| 低 | U-06 (本次方向下顺带解决); U-11 (窗口指示器补全); U-09/U-15 (裁决结案即可); R-08 (--no-color); U-13 (completions); F-09 (正文校验范围) |

## 七、本次 UX 重设计必须顺带解决的 Top 项 (建议)

1. U-01: 消除 --from 双语义 (身份改 --as 或前置位置参数名字, 与"名字前置必填"新方向对齐)
2. U-03/R-01/N-03: 统一线程创建语义, 消除系统消息占 #1 与 create/自动创建双轨
3. U-07: brief add / contacts add 主载荷位置参数化 (与"路径前置必填"新方向对齐)
4. U-04: 从正文自动提取 @mention, --mention 降级为补充
5. U-10: send 输出暴露隐式 mention 等副作用字段
6. U-08/N-01: 重排并裁决完整别名表 (含 post 别名缺位)
7. U-14/N-02: 路径解析改为"先试原路径, 再补类型后缀"
8. 裁决类 (必须给出接受/拒绝结论, 避免再次遗留): U-02 env 回退, U-05 内容优先(建议拒绝), U-09 summary 合并(建议保留独立), U-15 validate --type, R-08 --no-color(建议拒绝), F-09 正文校验范围(建议接受现状), U-13 completions(建议延后)

---

## 附: 已修复项落地版本索引 (供 CHANGELOG 追溯)

- v0.2: 无状态架构, post 统一, brief/contacts 引入, 文件锁, --json/--plain (F-01..F-03 等)
- v0.3: 类型后缀, bullet 元数据, 四反引号围栏, validate 命令, profile list 过滤, contacts read 修复 (F-06..F-08, F-10, R-10, R-12 前半)
- v0.4: envelope 输出协议, fix/example 错误自愈, --stdin, 空正文拒绝, validate 真实解析, --plain 范围过滤, contacts 富化, summary title/participants, brief --full 详情, profile list 结构化, 纯 ASCII 输出 (Q-01..Q-02, R-02..R-07, R-09, R-11..R-14)

---

## 八、v0.6 rework 轮补录项（2026-08-09 追加，不改写原文）

- B-01（低，冻结行为登记，Pete N3）：`post send/edit --reply-to` 指向不存在 seq 时「静默跳过」（reply 关系丢失且无信号，消息照常落盘）。属 v0.5 冻结行为（spec v0.6 §3.1 错误映射沿用），与 Q-02 失败自愈存在张力。本轮不改；供发布轮或后续 UX 线裁决（候选方向：ok 信封增补 reply-dropped 字段，需解冻输出协议，与 F6 的 ignored 字段同批评估）。
- B-02（低，静默面候选增强，contacts CRUD 轮 rework 补录 2026-08-09，Ryan M-3 ④）：contacts add/update 对目标 profile 路径**写前存在性校验/回显**。现状（冻结）：add/update 对目标 profile 不做任何可读性校验，不存在/不可读时静默写入 + label 依 R11 回退文件名主干（format-v2 spec §7.3，与 add 既有行为一致）；agent 路径笔误（忘后缀/传名字当路径）时 exit 0 且 destination 不可用，需下一轮 contacts read 才见 `(unreadable)`。行为本身本轮不改（裁决维持现状，避免 add/remove/update 三动词行为分叉）；声明与钉住已落 spec §3.6「NEW 不存在时的行为契约」+ bdd S-CONTACTS-14；候选方向：写前 destination 存在性校验或 ok 信封增补 destination-unverified 回显字段（需评估与输出协议冻结的关系），供发布轮裁决。

---

## 九、owner 四项裁决注记（2026-08-15 追加，append-only，不改写第八节原文）

追加依据：owner 2026-08-15 四项裁决（逐字原文与解释口径见 docs/dev/owner-rulings-2026-08-15.md）。

- **B-01 裁决注记**：owner 裁决 1「reply, mention 等等语义都在 markdown 消息本身中负责表达, cli 部分不应该给出这种参数用法」——撤销 post send 写侧糖衣 flag `--reply-to`/`--mention`（写命令传入落 usage exit 2），reply/mention 语义由 agent 正文直书 `@#N`/`@name` 表达；「指向不存在 seq 静默跳过」问题面随撤销结构性消解，本条候选方向（ok 信封 reply-dropped 字段）一并废弃；**读侧 post read `--reply-to`/`--mention` 为查询过滤器而非语义表达，不在撤销范围（此解释口径显式声明，供 owner 复核）**。spec 修订：cli-grammar-v0.6/spec.md §1.4/§2/§3.1/§10；bdd S-SEND-20~23；台账闭合：open-items-ledger 第十三节 LED-09。
- **B-02 裁决注记**：owner 裁决 2「contact 可以加入路径和格式上面的 validation, 但是不阻塞 agent 编辑即可, 也就是如果文件不存在或者错误, 也不需要阻塞编辑的 agents 添加这个 profile 文件」——contacts add/update 增加**非阻塞 advisory 校验**：destination 路径不存在/不可读/格式非法时仍照常写入 exit 0，ok 信封增补 `advisory` 提示字段（只增不改协议），永不因 destination 问题 exit≠0；本条两个候选方向中「写前校验」按非阻塞形态采纳、「destination-unverified 回显字段」由 `advisory` 字段契约承接（字段名钉住见 spec §3.6/§5）。spec 修订：cli-grammar-v0.6/spec.md §3.6/§5；bdd S-CONTACTS-16/17；台账闭合：open-items-ledger 第十三节 LED-10。
- 联动销账：同批裁决另闭合 U-04（同 B-01 处理，见本节上方行后注记）与 U-13（钉住结案，见本节上方行后注记）；台账 LED-09/10/11/12 全部刷新为已闭合。

---
(报告完)

更正注记（2026-08-15，owner 边界更正，仅追加不重写正文）：owner 从未指示发布 0.6；本 backlog B-01/B-02 的「供发布轮裁决」改读为「owner 决策（时机待 owner 指示，本工作流无发布计划）」。权威口径见 docs/dev/open-items-ledger-2026-08-15.md 第十二节。
