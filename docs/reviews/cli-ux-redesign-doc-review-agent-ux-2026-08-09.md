# CLI UX 重设计文档集 — agent 消费者视角对抗性评审

- 日期：2026-08-09
- 任务：Task 8，视角二（agent 消费者 / harness 视角 UX 批判）
- 评审对象：docs/ssot/specs/cli-ux-redesign/{spec,design,bdd,tdd,impl_plan}.md；docs/ssot/adr/feedbacks/v0.5_feedbacks.md
- 交叉核验：docs/researches/ 现状调研与业界 SOTA 基准（尤其结论 3/4/6/10）；v0.4.0 源码 repos/paperwork-cli/src/cmd/
- 评审方法：模拟一个 LLM agent 凭 --help / SKILL.md / usage 信封学习并使用新 CLI 的全过程，逐环节挑错；关键结论均以文档原文或源码行号为证据
- 立场声明：对抗性评审，非背书

## 总体结论

文档集整体质量高：三条文法规则自洽、owner 指令②被忠实落实、输出协议以有序 additive 方式扩展、BDD 覆盖多数正常与错误路径。但存在一处 BDD 场景与 spec 签名互斥（C1）、usage 信封核心卖点「example 即修正」未定义实现规则（M1）、对既有 v0.4 解析器的影响声明过度（M2/M3）。判定：**不闭合**，必须修复清单见文末。

---

## Issue 清单

### C1（Critical）BDD S-SEND-08 与 spec §3.1 签名互斥，不可实现；「漏 NAME」这一 agent 最高频错误得到误导性信号

agent 复现路径：agent 漏掉 NAME，执行 `paperwork post send standup.post.md "Parser done"`。按 spec §3.1 签名 `<PATH> <NAME> [BODY]`，clap 正常解析：NAME="Parser done"、BODY=None，随后按 v0.4 既有逻辑（post.rs resolve_body）报 validation「no message body provided」exit 1。但 S-SEND-08 要求 exit 2 + `error usage:`。

场景括注「NAME 位被 body 占用或位置参数不足」实为两种不同结果：仅位置参数不足（只给 PATH）才产生 usage；「NAME 被 body 占用」形态必然落 validation。按 BDD 写测试在步骤⑤必然红，或迫使实现者擅自改契约。
更深的 UX 问题：agent 明明给了正文，错误却说「缺正文」——信号指向错误槽位，agent 第一次重试（补正文）仍失败，需 ≥2 轮加查 help 才能恢复。design §2.1 只论证了 edit 的 SEQ 可区分性（「非数字即 usage」），对 send 的 NAME/BODY 混淆面只字未提。

建议：① S-SEND-08 拆为两个场景（纯缺参 → usage exit 2；NAME 被 body 占用 → validation exit 1 并另立断言）；② spec §3.1 post send 与 after_help 显式声明该混淆面；③ 考虑在「无正文」validation 错误的 fix 行补「若你已给出正文，请检查是否遗漏 NAME 槽位」。

### M1（Major）usage 信封「逐字修正 example」的生成规则未定义

agent 复现路径：v0.4 肌肉记忆的 agent 执行 `paperwork post send x.post.md --from alice "hi"`。S-SEND-09 要求信封给出逐字修正 `paperwork post send x.post.md alice "hi"`。

这要求把未知 flag 的值移入 NAME 槽位、保留尾部位置参数、还原引号——clap `try_parse()` 的错误对象不提供这类重构信息。spec §4.3 / design §2.2 / impl_plan 步骤③都只说「渲染修正 example」，未定义：（a）旧文法错误的重构算法或映射表；（b）通用缺参错误的模板形态（具体值还是占位符）。
S-SEND-09 / S-EDIT-04 / S-PROF-03 / S-BRIEF-03 / S-CONTACTS-03 五条断言全部悬空于此。实现者若退回通用模板，断言集体失败；若各自发挥，信封形态不一致。

建议：spec §4.3 增补 example 生成策略——已知旧 flag → 槽位映射表（--from→NAME、--seq→SEQ、--title→TITLE、--entry→ENTRY、--entry-title→ENTRY-TITLE、--profile→PROFILE-PATH、--name→NAME）+ 无法重构时的兜底模板形态；或将上述 BDD 断言降级为「信封含 example 行」而不要求逐字一致。

### M2（Major）输出层「零破坏」声明过度；对写死 v0.4 契约的解析器影响未逐项声明

证据：design §9 末条声称「所有解析输出层的既有代码（ok/error/JSON 消费者）零破坏」。但本版实际引入四项消费者可感知的变化。

① `showing` 出现条件由「仅超限」改为恒显（默认档与 JSON 同步；v0.4 源码 post.rs 仅 total>limit 时输出），「showing 存在 ⇒ 被截断」的消费者语义失效；② 新增退出码 2；③ category 词表由 6 类扩为 7 类（usage）；④ 新增 window / implicit-mention / 错误 command 三个字段。
spec §4.6 称 showing 为「additive 补全」，但这只是 key 层面只增；出现语义的变化未被声明。impl_plan 步骤⑦的 CHANGELOG 要求也未要求列出消费者侧影响。

agent 复现路径：按 v0.4 写死的 harness 解析器 `if "showing" in fields: 进入翻页循环` 将在每次 read 误触发；按 `exit_code == 1` 写重试分支的脚本会漏接 usage 错误（exit 2）。
建议：design §9 末条改写为「信封结构与既有 key 零变更，但含四项消费者可感知变化」；CHANGELOG `Changed (Breaking)` 小节逐项列出上述四点并附消费者迁移说明。

### M3（Major）`showing`/`window` 的 `<total>` 语义未定义，过滤场景下增量读取游标无依据

源码核验：v0.4 post.rs 中 total = 经过 mention/reply-to 过滤后、limit 前的条数，conclusion 行 N 同基准。

S-READ-06（`showing: 0/0`）依赖该隐式前提，S-READ-02（20/50）在无过滤时两种解读无法区分；spec §3.1 / design §2.3 均未声明「total = 过滤后、limit 前」。
agent 复现路径：agent 用 `read --mention alice` 做增量消费，见到 showing 20/25 会判定「线程共 25 条」，而实际线程 50 条——游标与去重策略失据。且 window 取「实际展示的首末 seq」（线程基准）、total 取过滤后基准，两种基准并存而未声明。
建议：spec §3.1 post read 增加一句「`<total>` 为过滤后、limit 前的消息数，与 conclusion 行一致」；BDD 补过滤+limit 组合场景。

### M4（Major）body 以 `-` 开头的自愈闭环不完整：`--` 边界零教学

agent 复现路径：agent 首次发送正文 `-fix flag text` 而未用 `--`，落入 usage 错误（未知参数）。但 BDD S-SEND-07 只测了 `--` 成功路径；未给 `--` 时的 usage 错误形态无场景；usage 信封的 fix/example 是否会教 `--` 未声明；design §2.4 post send 的三条 after_help 示例没有一条演示 `--`。无先验知识的 agent 只能盲试；即便 clap 原生提示存在，spec 未把该教学钉进信封契约。edit 的 NEW_BODY 同为 content 槽位，同样无 `--` 场景。

建议：① after_help 补一条 `--` 边界示例（send 与 edit 各一）；② BDD 补「`-` 开头 body 未加 `--` 的 usage 形态」场景，并要求该信封 fix 行显式提及 `--`；③ 同步补 edit 的 `--` 成功场景。

### M5（Major）三级路径解析第①级引入未声明的新角落：传入路径存在但为异型文件

agent 复现路径：目录中恰有 notes.md（普通笔记，非线程）。v0.4 行为：`post send notes.md alice "hi"` 被改写为 notes.post.md 并自动建线程；v0.5 第①级「传入路径原样存在 → 用原路径」则直接对 notes.md 按线程解析 → format 错误。

这是与 U-14 修复方向相反的真实行为差异，但 spec §5 清单 ensure_suffix 行只讲了修复收益；BDD S-PATH-01 的 Given 恰好预设「裸 .md 是合法线程」，回避了该角落；`profile create alice.md alice` 在 alice.md 已存在时同样命中第①级，与 already-exists 的边界含混。
建议：spec §5 ensure_suffix 行补「第①级命中异型文件时按对应类型解析器报错（format），不再自动改道补后缀路径」；BDD 补场景（已存在的非线程 .md + send → format 错误，example 引导 validate --type）。

### M6（Major）example 占位符策略不一致，且与自引基准 SOTA 结论 10 冲突

证据：design §2.2 not-found 示例 `example: paperwork post send <path> <name> <body>` 保留三处占位符。

BDD S-SEND-08 要求 example「带 NAME 槽位」（占位符形态），S-PROF-02 / S-SEND-05 却要求具体命令形态——策略自相矛盾。本版声称「全部 example 字符串刷新」，并以 SOTA 报告为 architectural basis，而该报告结论 10 明确要求「example 永远可复制执行、缺参错误尤其要给一条完整正确命令」。
agent 复现路径：agent 逐字复制 `paperwork post send <path> <name> <body>`，`<` 在 shell 中是重定向符，立即触发第二次解析错误，多摔一跤。

建议：统一策略——上下文值可得（PATH 已知）时用具体命令；确不可知量改用对 shell 无副作用的占位形态（如 `{name}`、`{body}`），并把占位符书写约定写入 spec §4.2。

---

## Minor 问题

N1（Minor）contacts create 保留 --title 的不对称：理由充分但双向负迁移无补偿教学。规则 3 默认值论证成立（design §5.1），但 agent 从 post/brief create 泛化出「create <PATH> <TITLE>」后，`contacts create team "Core Team"` 会因多余位置参数落 usage；反向在 brief create 用 `--title` 同样报错。「判据一致性本身就是最好的文档」不会自动消除迁移错误。建议 contacts create 的 after_help 加一行 title 为可选 flag（默认 Contacts）的显式注记。

N2（Minor）brief add 位置参数是「路径」，brief remove 位置参数是「标题」，title 推导规则未写入 spec。agent 复现：`brief add onboarding src/main.rs` 成功，随后 `brief remove onboarding src/main.rs` 落 not-found（应传 title "main.rs"）。spec §3.3 只说 ENTRY-TITLE 位置参数化，未声明 add 的 ENTRY 与 remove 的 ENTRY-TITLE 的映射关系（basename）。建议 spec §3.3 补一句推导规则；BDD 补 add/remove 参数形态混用场景。

N3（Minor）NAME 字符集与校验语义未声明。NAME 可否含空格、逗号、空串，全套文档未定义；含逗号的名字会干扰 `--mention a,b` 解析与 implicit-mention 比对。源码核验（post.rs send 分支）：send 对 NAME 不做 profile/contacts 存在性校验。

但现状调研文档 §6.1 却误记「send 与 edit 都校验 --from 与 profile 名一致」。建议 spec 正面声明「NAME 不与 profile/contacts 校验（行为沿用 v0.4）」，并给出最小字符集约束或明确声明无约束。

N4（Minor）BDD 缺口场景清单：① 多余位置参数（`post send` 四参 → usage）；② `--type` 非法值（usage）；③ post create already-exists；④ edit 的 `--` 边界（与 M4 联动）；⑤ 回复自己的消息不触发 implicit-mention（源码 post.rs 条件 `original.sender != from`，S-SEND-03 未覆盖）；⑥ validate `--type` 与后缀交叉形态（如 x.profile.md --type post）。

N5（Minor）规则 2 严格解读与 --mention/--reply-to 的读写双角色未裁定。send 中二者是「设置」，read 中是「过滤」，同一 flag 两种角色（值语义同构：均为 seq/名字）。规则 2 原文「全 CLI 任何 flag 只有一种含义」。建议 design §7.4 或 spec §1.3 补一句裁定：「设置/过滤视为同一语义对象的同构延伸，不构成双语义」。

N6（Minor）SOTA 结论 5/6 的未竟项无采纳/拒绝记录。C5 后半（机器可读内省 agent-context / --help --json）被静默放弃（仅 SKILL.md 半采纳）；C6（命名政策测试强制：动词白名单/flag 白名单）在 spec/tdd/impl_plan 均无踪影，而 spec 文首引用该报告为 architectural basis。建议补拒绝理由，或将 C6 白名单断言纳入 tdd §3（成本极低：复用既有精确断言模式）。

N7（Minor）文档卫生与数字出入。tdd §1 标题「约 24 处」与合计「26 处改写 + 1 处保留」不一致；现状调研文档 §3.2 brief add flag 名（写为 --title/--path）与 §6.1「send 校验 --from 与 profile 一致」均与源码不符（已对照 cmd/brief.rs、cmd/post.rs 核验），建议勘误以免误导后续实现。

N8（Minor）after_help 覆盖失衡。design §2.4 只给了 post send/edit/create、profile create、brief 系列、contacts 系列、validate 的示例；而 post read——恰是 `--from/--to` 新唯一语义的唯一承载者、迁移 agent 最可能试 `--from <name>` 的命令——没有 after_help 示例。建议补 post read 的 seq 范围示例（顺带示范 `--`）。

---

## 评审要点逐条回答

### 1. 学习成本

结论：**不能零歧义掌握**，但整体学习曲线可接受（顶层文法模板行 + 逐命令 after_help + usage 信封纠错教学，方向正确，满足 SOTA 原则 5「两次调用拿到完整形态」）。残留风险点按严重度排序：C1 NAME/BODY 混淆面（最高频错误得到错误信号）> M4 `--` 边界零教学 > N1 contacts create 负迁移 > N8 post read 示例缺失。
位置参数顺序的记忆负担集中在 post edit 三连：NAME 是字符串、SEQ 是数字，类型差是最后防线，S-EDIT-03 已钉住非数字场景；「NAME 为数字、SEQ 非数字」的对调形态也落 usage，信号明确，可接受。

### 2. owner 指令达成度

结论：**达成（意图层面 100% 落实）**。PATH 恒为全 CLI 第 1 必填位置参数，NAME 在 send/edit 为第 2 必填位置参数；token 流中 `post send` 两个固定 token 之后紧跟的即是 PATH 与 NAME，文件与身份即时可见。输出侧对称：conclusion 含路径、sender 字段回显身份。
字面 path-first 文法被否决并给出四条理由、保留为 v0.6 提案；v0.5_feedbacks §二.3 已明确「意图为硬性要求、文法细节属授权涌现」的解读并落盘——程序闭合。残留风险仅为「owner 坚持字面文法」需再次确认，属文档外事项，不构成本文档集缺陷。

### 3. 错误自愈闭环（重试次数估算）

| 失败模式 | 恢复所需重试 | 前提条件 |
|---|---|---|
| 纯缺位置参数 | 1 次 | usage 信封给具体 example |
| 旧文法调用（--from/--name/--entry/--profile） | 1 次 | M1 的重构规则落实；否则不确定 |
| 漏 NAME 被 body 吞掉 | ≥2 次 | 误导性 validation 信号（C1） |
| edit 三参顺序错 / SEQ 非数字 | 1 次 | usage 信号明确 |
| body 以 - 开头 | 1 次或 ≥2 次 | 取决于 agent 是否会 `--`（M4） |
| 文件不存在 | 1~2 次 | 取决于 example 是否占位符形态（M6） |
| validate 未知后缀 | 1 次 | fix 直接指 --type |

「错误即教学」的设计哲学执行水准高：七类 category、fix+example 三层、usage 信封承接迁移教学都是业界前列做法。闭环的两个洞在 C1（信号指向错误槽位）与 M1（修正命令的生成规则悬空）。usage 信封 example 是否「逐字可执行」：现状文档给出的示例多数可执行，但 not-found 类保留占位符（M6），且占位符形态与具体形态在 BDD 内部不一致。

### 4. 输出解析稳定性

声明不充分，见 M2/M3。正面：JSON 只增不改不删条款（spec §4.6）、退出码语义表（§4.4）、第七类 usage 的声明均在；缺失的是「消费者可感知变化」的逐项披露与 showing `<total>` 的语义定义。对按 v0.4 写死的解析器，四项变化（showing 恒显、exit 2、usage category、新字段）中只有新字段可称纯 additive。

### 5. 逐命令挑刺（18 条签名过一遍）

- post 组：send/edit 混淆面（C1）；read 的 total 未定义（M3）；create/summary 签名无反直觉点。
- profile 组：create 位置化合理；edit 无 --name 与 v0.4 源码一致（现状调研 §3.2 误记，N7）；show/list 无问题。
- brief 组：add/remove 参数名词错位（N2）；create/read/verify 无问题。
- contacts 组：create --title 不对称（N1）；add/read 无问题。
- validate：--type 新增合理；「不参与 ensure_suffix」的不对称已在 design §6.1 声明（诊断作用于确切路径），可接受，建议 help 重申。
- 横切：--mention/--reply-to 读写双角色（N5）；别名 po 与 p 不冲突已声明。

### 6. BDD 场景覆盖

正常路径与主要错误路径覆盖良好（含路径解析、输出模式、别名三组横切场景）；缺口见 C1（场景契约错误）、M4/M5（缺场景）、N4（六条清单）。

---

## 是否闭合

**不闭合。** 计数：Critical 1，Major 6，Minor 8。

### 必须修复清单（阻塞 impl_plan 前置门槛）

1. C1 — 拆分 S-SEND-08，并声明 NAME/BODY 混淆面与其错误信号
2. M1 — spec §4.3 补 usage example 生成规则（旧 flag 槽位映射表 + 兜底模板），或降级相关 BDD 逐字断言
3. M2 — 更正 design §9「零破坏」表述；CHANGELOG 要求中逐项列出四项消费者侧影响
4. M3 — 定义 showing `<total>` 语义（过滤后、limit 前），补过滤+limit 组合场景
5. M4 — `--` 边界教学进 after_help 与 usage fix，补 send/edit 相关场景
6. M5 — spec §5 声明第①级命中异型文件的行为，补 BDD 场景
7. M6 — 统一 example 占位符策略，消除 shell 敏感占位符，写入 spec §4.2

### 建议修复（不阻塞，建议随必须修复一并完成）

N1–N8；其中 N3/N4 直接影响步骤⑤测试断言，建议提高优先级。

---
（报告完）
