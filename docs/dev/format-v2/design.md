# Managed File Format v2 设计说明（Informative）

> **文档性质**：Informative（说明性）。解释 spec.md 每项语法选择的设计理由与替代方案取舍；与 spec.md 冲突时以 spec.md 为准。
>
> **上游依据**：synthesis（`docs/researches/format-v2-design-synthesis-2026-08-09.md`）§3 三视角取舍记录、§4 Rejected Alternatives；用户最高指令 `v0_feedbacks.md` 第 23、27 行。
>
> **版本说明**：本文为阶段 1 对抗性评审后按 leader 裁决（R1–R15）的 rework 定稿。§8 为"规格完备化与评审裁决记录"，是一致性审计锚点。**2026-08-09 owner 追裁（D1–D3，仅涉 post/thread，并入 0.5.0 不 bump 版本）记录于 §8.5**；与追裁冲突的早期表述以追裁与 spec.md 为准。

---

## §1 设计立场

v0_feedbacks.md 第 23 行确立了本项目的格式宪法：

> 既然我们选择使用md作为文件格式, 那么就以正规简洁的方式组织信息结构, 严谨克制, 但是自由灵活地使用各个标题, 列表等等语法.

由此导出本次重设计的单一判据：**每个语法构造必须能在 Markdown 原生语义（CommonMark 为主、GFM 扩展须明示）中找到公认的含义背书，结构符全 ASCII**。凡自造 pattern（复合前瞻边界、非 ASCII 分隔符、魔法值占位、固定节名保留字）一律废除或补证。同时第 27 行确立了 fence 包裹用户正文的硬性要求（支持多层 markdown）——其适用域经裁决限定为 **post 消息正文**（R15，见 §8）——故正文围栏不可废除，只能标准化。

## §2 逐构造的"原生 Markdown 语义"论证

| 构造 | 定稿选择 | 原生语义背书 | 被否决的替代 |
| --- | --- | --- | --- |
| 文档身份 | H1 = name/title | ATX 一级标题是 Markdown 公认的文档标题 | frontmatter `title:` 字段（YAML，被用户裁决否决） |
| 文档描述 | H1 后首个 H2 前的散文段（profile/brief/contacts；post 例外：追裁 D1 后 preamble 仅 H1，其后散文解析忽略） | 标题下的引导段落是文档惯例（README 范式） | `- Description:` 属性行（单行限制长描述；blockquote 语义是"引用/旁注"，不贴切） |
| 扁平标量属性 | `- key: value` bullet 列表（小写键），两处有效区（spec §3.2，R4；原"post 消息属性区"随追裁 D2 废除，post preamble 亦无属性语义，D1） | 如实表述：Markdown 原生并无"键值"语义——bullet 提供列表语义，`key: value` 冒号约定沿用业界通用键值习惯（email header / RFC 822 风格）；在纯原生约束下这是噪声最小的组合 | YAML frontmatter（禁）；blockquote `> key: value`（引用语义，视角 C 主张，否决）；逐消息 ` ```yaml ` 块（视角 B 主张，重新引入 YAML 且视觉噪声大，否决）；`- **key**: value` 粗体键（强调语法充当结构标记，视觉噪声更大，否决） |
| 参与者 | **废除落盘，由消息 sender 集合派生**（追裁 D1）：对话消息中已包含全部发言者，维护名单是冗余负担；需要时（如 summary）按消息首次出现顺序去重派生 | 零维护负担、无双源漂移；线程语义本就允许伪造身份（v0_feedbacks 第 9 行弱耦合立场），名单的结构化收益不成立 | preamble 属性行 `- participants: alice, bob`（原 R2 方案，被 D1 取代：名单与消息 sender 双源共存需人工同步，冗余）；`## Participants` 保留标题（与旧保留字 `system`、魔法值 `all` 同构，否决）；链接列表（硬依赖目标 profile 存在，违背弱耦合，否决） |
| profile Scope | 属性行列表 `- <perm>: <glob>`（键可重复，R3） | 复用属性行文法，零新构造；一行一对 (permission, glob)，手写成本低 | **GFM 表格**（否决理由：表格是 GFM 扩展、不在 CommonMark 规范内；synthesis §5 风险表自认"表格手写不便"需专门缓解；属性行方案可零新构造达成同等表达）；反引号包裹的逗号分隔 glob（旧格式，ad-hoc） |
| 文件引用 | Markdown 链接（contacts） | 链接是原生引用语义，label 天然充当简介锚点 | 裸路径 bullet（视角 A 主张；无 label、无引用语义，否决） |
| 用户正文 | 动态长度 ` ```md ` 围栏（仅 post 消息正文，R15；追裁 D3：写严格 `md`、解析宽容 `md`/`markdown`） | CommonMark 明定围栏长度可变：N 反引号开的围栏仅被 ≥N 关闭；info string 缩写 `md` 是社区惯例（`markdown` 前缀） | 固定 4 反引号（非规范依据；正文含 4 连反引号即破防）；写 `markdown` 全称（D3 裁决简化为 `md`）；brief note / profile description 也过 fence（文档元叙述是裸散文，fence 只保护"被转发的用户内容"，否决，R15） |
| 记录单元 | H2 标题（消息/条目） | heading 即边界，标题文本即元数据载体，无需辅助定界符。**contacts 例外限定**：contacts 条目无正文，H2 过重，退化为链接 bullet（统一设计语言第 5 条例外，此前漏记） | `---` + 前瞻 H3（复合算法，无规范依据）；逐消息 yaml 块（否决理由同上） |
| 消息头编号 | `## #N sender (ts)` 中的 `#N` 前缀 | H2 本身有原生标题边界背书；`#N` 是**业界编号引用惯用法**（GitHub issue 引用风格 `#123`），且与正文回复引用文法 `@#N` 构成**同一 token 族**（追裁 D2 后引用迁入正文）——写入、引用、解析三处同形互证。`#` + 纯数字 seq 前缀是任何自然 H2 不可能撞上的形态（碰撞规避是附加收益，不是唯一理由） | `## N. sender`（有序编号 `N.` 是列表项文法，置于标题文本内语义错位，且与 `@#N` 引用失去 token 族同一性）；`## Message N: sender`（引入英文固定词 `Message`，冗长且属保留字构造）；去 `#` 的 `## sender (ts)`（自然标题如 "## alice (notes)" 可能撞形，seq 连续性失去可锚定前缀，否决） |
| 复杂 regex 逃生口 | ` ```regex ` info-string 围栏 | CommonMark 允许任意 info string；`regex` 为项目私有标注，无高亮生态、渲染为纯代码块，可接受 | YAML 块标量（否决：重新引入 YAML） |

## §3 preamble 与消息同为 H2 的歧义消解

**问题**：preamble 中用户手写的任意 H2 与消息头 `## #N ...` 同为 H2；且用户正文（fence 内）可能包含形似消息头的行。若不做消解，解析会把 preamble 节误认为消息、或把正文内容误切为新消息。

**策略**（spec.md §5.2/§5.3 的规范性来源）：

1. **文法收窄为唯一判据**：消息头必须精确匹配 `^##\s+#(\d+)\s+([^\s()]+)\s+\((.+)\)\s*$`——`#` + 纯数字 seq 前缀（token 族论证见 §2）使任意自然 H2 不可能撞形；sender 由 `[^\s()]+` 强制无空格无括号（解析侧与写入侧一致拒绝，R1）。字段间空白宽容（`\s+`/`\s*$`，R9），手写单空格偏差不再静默失败。
2. **fence 感知先行**：边界扫描全程维护围栏状态机（CommonMark 长度规则，§3.3 缩进与 tilde 立场），fence 内的 `## #N ...` 永不参与匹配（BDD:POST-05 为对抗用例）。
3. **默认归 preamble**：一切不匹配消息头文法的 H2（含用户手写的 `## Notes` 等）归入 preamble 忽略——歧义方向统一倒向"宁可多算 preamble，不可错切消息"。R2 废除 `## Participants` 保留标题后，preamble 不再含任何格式保留的 H2，本条规则成为 preamble H2 的唯一处置。
4. **写入侧防御**：sender 字符集校验（无空格/括号）保证 CLI 产出的头不可能被正则歧义解析（BDD:POST-17）。

**残余风险评估**（含评审披露项）：

- 用户手写文件若出现恰合文法的伪造 H2 且不在 fence 内，会被解析为消息——这是 heading 即边界方案的固有属性，由 `validate` 的 seq 连续性校验兜底暴露（伪造头几乎必然造成 seq gap）。
- **时间戳解析失败的后果（C1 披露）**：头正则对 timestamp 贪婪捕获（`\((.+)\)`），头行尾部垃圾（如 `## #1 alice (ts) (备注)`）会整体吃进时间戳字段 → 时间戳解析失败 → **整文件 `Parse`，`post read/summary` 全部不可用**。这是 fence 外头形行不受围栏保护的必然结果；缓解手段是 validate 的"疑似消息头"启发式 warning（R9）与写入侧规范输出，不降低解析严格度。
- sender 允许非 ASCII、无长度上限（受 64KB 隐式约束，spec §5.6），本身无歧义。
- 尾扫 fence 盲区：缓冲区内已做开合追踪（R6），残留限制（缓冲起点切断围栏）由 validate 兜底，spec §5.5 明文声明（BDD:CONC-03）。

## §4 废除 system 消息后的线程元数据方案

**旧方案缺陷**（技术债 #2）：`post create` 写入 sender=`system` 的 #1 消息，正文文本 `[Thread created: X | participants: Y]` 编码 title/participants；`post summary` 靠字符串切分反解——脆弱编码，且 system 消息污染消息序列（真实首条是 #2）。

**新方案**（spec.md §5.7；2026-08-09 追裁 D1/D2 后形态）：

1. title 升格为 **preamble**：H1 = title；**preamble 仅剩 H1 标题**（D1：participants 废除，标题行后允许自由散文但解析忽略）。
2. `post create` 命令整体删除；`post send` 承担建文件职责：新建时（锁内 size = 0）以 `--title`（缺省算法：剥 `.post.md`，否则剥 `.md`，否则原名）**先写 preamble（仅 H1）再写首条消息**，同锁单次完成（不变量 I9）；`--to`/`--participants` flag 随追裁删除（D1/D2）。
3. 竞争处理：两写者并发首写时，fs2 排他锁串行化；后到者锁内复扫发现 size > 0，只做追加——preamble 恰好写一次（BDD:CONC-02）；首写者崩溃遗留 0 字节文件时，下一 send 锁内按 size == 0 补写 preamble，preamble 仍恰一次（BDD:CONC-04）。
4. `post summary` 直读 preamble 取 title（`thread_meta`）；**participants 由消息 sender 集合派生**（按首次出现顺序去重，D1），删除字符串切分反解逻辑。
5. `thread_edit` 全文件重写时对首个消息头之前的字节区间**原样搬运**（不做规范化重序列化），preamble 中手写内容逐字节不丢。`ThreadMeta` 仅用于解析读取视图。
6. 数据模型：`lib.rs` 的 `ThreadMeta { title }`（participants 字段随 D1 删除）；`Message` 删除 reply_to/mentions/to 字段（D2：引用状态为正文文本派生，不落盘）。

## §5 废弃构造清单与替代

| 废弃构造 | 旧用途 | 替代方案 | spec 章节 |
| --- | --- | --- | --- |
| `·`（U+00B7）消息头分隔符 | `### #N sender · time` | H2 头 `## #N sender (time)`，纯 ASCII 括号 | §5.3 |
| `—`（em dash）空值占位 | 空 scope / 空 reply-to / 空 regex | 空/缺省即省略该行（不写） | §3.2、§4.2、§6.2 |
| `all` 魔法值 | 广播 `To: all` | 广播 = 省略 `to` 行 | §5.4 |
| 固定 4 反引号围栏 | 正文包裹 | 动态围栏 max(3, 最长串+1) | §3.4 |
| `---` + "2 行内前瞻 H3" 复合边界 | 消息定界 | H2 文法头即边界（fence 感知） | §5.3 |
| `### #N` H3 消息头 | 消息定界 | 升格 H2 | §5.3 |
| 大写键 `- To:`/`- Model:` 等与反引号剥除 | 属性行 | 小写 ASCII 键 `- key: value`，值裸文本 | §3.2 |
| system 消息 `[Thread created: ...]` | 线程元数据 | preamble + `post send` 锁内首写 | §5.7 |
| blockquote note（brief 条目） | 条目说明 | 散文 note 段 | §6.2 |
| `## Entries` 包装节（brief） | 条目容器 | 条目直接 H2 | §6.2 |
| 裸路径 bullet（contacts） | 文件引用 | Markdown 链接（含 `[](<path>)` 转义） | §7 |
| `## Participants` 保留标题 | 参与者名单 | 随 D1 一并废除：participants 不落盘，由消息 sender 集合派生 | §5.2/§5.4 |
| preamble 属性行 `- participants:`（原 R2 方案） | 参与者名单 | 同上（追裁 D1 取代） | §5.2 |
| 消息属性行 `- reply-to:`/`- mentions:`/`- to:` | 引用状态结构化字段 | 正文文本引用 `@somebody`（mention）/`@#N`（reply 引用），读取/统计时实时派生不落盘（追裁 D2）；`to` 彻底删除 | §5.4 |
| 围栏 info `markdown`（全称） | 正文围栏语言标注 | 简化为 `md`：写严格 `md`，解析宽容 `md`/`markdown`（追裁 D3） | §3.4、§5.4 |
| GFM 表格 Scope（profile） | (permission, glob) 集合 | 属性行列表 `- <perm>: <glob>`（R3） | §4.2 |

## §6 hard breaking 与迁移指南策略

**裁决**（synthesis §1.3）：项目默认不向前兼容，hard breaking v0.5，CHANGELOG 附迁移指南，**不做 migrate 命令**。

理由与后果：

1. 双版本共存（视角 C 的 Expand–Migrate–Contract）需要解析代码翻倍与结构探测分发，在"存量文件仅限自研语料"的现实下成本大于收益；用户已明确裁决。
2. 格式版本字段/frontmatter 类型标记因此无必要——格式版本以 spec 文档与 CHANGELOG 为准（Rejected Alternatives 第 6 条）。
3. 迁移指南义务（impl_plan.md S5 交付）：CHANGELOG 0.5.0 Breaking 段必须含**逐格式 before/after 对照**与手工迁移步骤（旧 `·` 头改 H2 括号头、`—`/`all` 删除、4 反引号改动态、system #1 提取为 preamble 的手工步骤），并**明示 hard breaking 后的已知症状**：旧格式 profile（大写键 `- Model:`）在 `profile list` 中显示 `(unreadable)`、`contacts read` 即时增强同理——这是宽容解析的预期降级而非故障（C7）。
4. 旧语料目录 `test-v03/`、`test-v04/`、`_fix/` 保持原样作历史记录，不迁移；新冒烟语料建 `test-v05/`。
5. 存量文件在新解析器下的行为由宽容解析决定（旧头不匹配 → 归 preamble/忽略 → `validate` 以零消息或 seq 校验拒绝），不会静默产生错误数据。

## §7 与既有架构原则的一致性

- **输出协议零改动**：`output.rs` 信封（`ok/error` + `fix:` + `example:`、JSON/plain/default 三模式）不在本次范围；错误文案更新仅限 fix/example 字符串内容（纯 ASCII 化）。validate 的错误信封直出底层变体（R10），不改变信封协议本身。
- **无新依赖**：不引入 `serde_yaml` 或任何 YAML 相关 crate；仍仅用 `regex`/`chrono`/`sha2`/`fs2`。
- **并发模型不变**：fs2 锁、O(1) 尾扫（缓冲区内 fence 感知为算法增强，机制不变）、64KB 上限（`thread_edit` 同守，R8）、锁内单次 write 全部保留（spec.md §10 不变量 I1–I4）。
- **写入原子性措辞校准（C6）**：I4 的"单次 `write_all`"不承诺 syscall 级原子性——Rust `write_all` 内部对部分写循环重试；真实保护来自 fs2 排他锁，`write_all` 的作用是避免交错写。`thread_edit` 的截断+重写崩溃窗口已接受并声明（spec §5.7）；若未来加固，方向是锁内 temp+rename（需评估 fs2 锁与 inode 替换的交互，本次不做）。

## §8 规格完备化与评审裁决记录

> 本节为一致性审计锚点：相对 synthesis 的全部增量与阶段 1 三视角对抗评审（格式正统性 F1–F15、解析健壮性 B-1/M-1..M-8/m-1..m-8、流程符合性 F-1..F-13）的 leader 裁决逐条记录。spec.md 与 bdd/tdd/impl_plan 均已按本节联动。

### 8.1 规格完备化补充清单（相对 synthesis 的增量）

synthesis §2 声明"schema 逐字保留不得删改语义"；以下为实现前规格完备化阶段补充的构造细节（均为保守扩展，后经评审裁决部分调整）：

1. contacts 尖括号转义触发集：synthesis 列空格/`(`/`)`；补充 tab；评审后扩至 `<`/`>`（C3）。
2. label 含 `]` 反斜杠转义（§7.3）及**解析侧反转义义务**（C3，roundtrip 闭合）。
3. reply-to 接受 `#N`/`N` 双形（§5.4）；评审后补非法值宽容为 None（C2）。
4. 空文件 validate 拒绝（§8）——对 v0.4 现状的行为变更（现状豁免空文件），impl_plan S3.2 明示。
5. `[label](path "title")` 形式立场：不接受 title，解析忽略 title 提取 destination（C3）。
6. 正文围栏 info string 任意均接受、多围栏取首个（C2）。
7. 疑似消息头启发式 warning（R9，§8 步骤 4）。
8. body 提取规范化规则（首尾空白行 trim，roundtrip 仅对规范化 body 成立，R12）。
9. `--title` 缺省派生算法精确化（剥 `.post.md` → 剥 `.md` → 原名）。
10. contacts label 派生规则（H1 优先、主干回退，R11）。

### 8.2 评审裁决记录（R1–R15，规格级）

| 编号 | 裁决 | 理由与被否决替代 |
| --- | --- | --- |
| R1 | 消息头正则改为 `^##\s+#(\d+)\s+([^\s()]+)\s+\((.+)\)\s*$`（sender 排除括号；行尾容尾随空白），解析侧与写入侧一致拒绝括号 sender；POST-07/T-FT-07 断言同步成立 | 原正则 `(\S+)` 不能排除括号（`\S` 只排除空白），spec 声称"无括号由 `\S+` 强制"系事实错误（Grace B-1、Ben F-1）。被否决替代：维持旧正则并反转 POST-07 断言（解析侧宽容接受括号 sender）——会造成写入/解析两侧语义分裂，否决。 |
| R2 | 废除 `## Participants` 保留标题，参与者改为 preamble 属性行 `- participants: alice, bob` | "固定节名"不在统一设计语言的构造类型内（定稿 6 条无此构造），英文字面量保留标题与旧保留字 `system`、魔法值 `all` 同构（Jack F3）。被否决替代：保留保留标题并补第 8 条设计语言——为单一用途新增构造类型，违背克制原则，否决。属性行方案零新构造且 post preamble 无键冲突。 |
| R3 | profile Scope 由 GFM 表格改为属性行列表（一行一个 (permission, glob) 对，键 read/write/owns 可重复；空 scope 省略整节） | GFM 表格非 CommonMark 语法（表格是 GFM 扩展，原判词"原生制表语义"失真）；synthesis 风险表自认"表格手写不便"；属性行文法零新构造（Jack F6）。被否决替代：保留表格并补 bullet 方案论证——一个需要缓解可读性缺陷的选择难称克制，否决。 |
| R4 | 统一设计语言第 2 条改写：属性行在三处有效区生效——preamble 区、post 消息属性区、brief 条目属性区；其余位置同形行是普通正文。brief 条目属性区 = 条目 H2 之后至首个非属性非空行，其后同形行归 note | 原第 2 条"首个 H2 之前"覆盖不到消息区与条目区，与四个 schema 的落实自相矛盾（Jack F4、Grace M-5）。被否决替代：承认"记录单元内属性"为第 8 条并列设计语言——同一文法拆成两条徒增复杂度，否决。（注：本条"三处有效区"中的 post 消息属性区已被 owner 追裁 D2 废除，现行规格为两处有效区，见 §8.5 与 spec §3.2。） |
| R5 | preamble **原文字节保留**：`thread_edit` 全量重写时对首个消息头之前的字节区间原样搬运，不做规范化重序列化；post preamble 允许 description 散文（第 1 条对 post 生效）；`ThreadMeta` 仅用于解析读取 | 原 I9"保留 preamble"实际是 `ThreadMeta{title,participants}` 再序列化，手写 preamble 内容首次 edit 即被静默销毁，与"忽略但保留"承诺矛盾（Jack F7/F14、Grace M-3）。被否决替代：明确"保留 = 仅 title/participants 投影，其余丢弃"（Grace 建议项）——违背第 23 行"自由灵活"，且成本高于原样搬运，否决。 |
| R6 | 尾扫 fence 感知（缓冲区内）：`read_last_seq_locked` 对缓冲区内围栏行做 CommonMark 长度规则的开合追踪，开启围栏内部的候选头跳过；残留限制（缓冲起点切断围栏的奇偶不可知）以 validate seq 连续性校验兜底，spec §5.5 明文声明 | 正文内伪造 `## #99` 是合法且更常见的语料（多层 markdown 是本次卖点），非 fence 感知尾扫会把伪造 seq 写入数据面（Grace M-2、Ben F-6）。被否决替代：(a) 尾扫保持 fence 盲仅文档化风险——数据面污染可避免却放任，否决；(b) 从候选向前校验围栏配对、失败回退上一候选——缓冲起点之前状态仍不可知，收益与 (R6) 相同而成本更高，否决。 |
| R7 | 尾扫丢弃首行规则：仅当 `read_start > 0` 时检查前一字节，非 `\n` 才截到第一个 `\n` 之后；`read_start == 0` 不丢弃 | 无条件丢弃在缓冲区覆盖全文件时吞掉完整首行——无 preamble 文件（首行即消息头）将 last_seq = 0 → 下次 send 静默 seq 重复，恰击穿 I2（Grace M-1）。 |
| R8 | `thread_edit` 对新 body 序列化后执行 ≤64KB 校验，超限 `MessageTooLarge` 且文件不变 | edit 制造 >64KB 消息会使末条头落在尾扫缓冲区外 → 下次 send 读到倒数第二条 seq → seq 重复，I3 缺口击穿 I2（Grace M-4）。现状 `thread_edit` 无此检查，属既有缺陷，本次以规格堵上。 |
| R9 | 消息头解析空白宽容（字段间 `\s+`、行尾 `\s*$`），序列化仍输出规范单空格；validate 增加"疑似消息头"启发式（形似 `## #N` 但不严格匹配的行报 warning + fix） | 原文法刚性过度：手写多一个空格即静默吞消息，与时间戳失败显式报错不对称，违背"自由灵活"（Jack F5、Grace 关联）。被否决替代：仅放宽空白不加启发式——括号/缺右括号等失配仍静默，否决；仅启发式不放宽——最常见偏差（多余空格）仍吞消息，否决。 |
| R10 | validate 错误信封直接透出底层错误变体（seq gap → category `validation`），不再统一重包为 Parse；修正 VAL-02；空文件 validate → Parse（VAL-07），impl_plan 标注行为变更 | 原状三处文档互相矛盾：spec §9.1 列 `Validation`/`validation`，VAL-02 断言 `format`，现状 validate.rs 统一重包 Parse（Jack F1、Grace M-8、Ben F-11）。被否决替代：维持统一 Parse 重包并修 spec §9.1——丢失错误分类信息且与 error.rs 事实相悖，否决。 |
| R11 | contacts label 派生：写入时读取目标 profile 的 H1 作为 label，读取失败回退文件名主干（先剥 `.profile.md` 再剥 `.md`，否则原名）；OQ-2 论据改写、CONT-03 期望值修正 | 第 1 条定义 H1 = 文档身份，label 取文件名主干会造成 H1 与 label 漂移、人类可读性差（Jack F12）；原 OQ-2"与既有派生行为一致"论据失实（现状无 label 派生逻辑，Ben F-2）。被否决替代：维持文件名主干并声明漂移为已知限制——把可解决问题降级为限制的惰性方案，否决。 |
| R12 | body 提取规范：围栏开启行与关闭行之间的行序列，去除首尾空白行后 `\n` 连接；roundtrip 保证仅对规范化后 body 成立；POST-06/POST-14 断言改为"规范化相等" | 原规格 body 提取规则只出现在 BDD 括号注内，且 POST-01"trim"与 POST-06"逐字节还原"冲突（Grace M-6）。现状代码确实做首尾空行 trim，规格继承并明说。 |
| R13 | 围栏判定立场对齐 CommonMark：前导空白 ≤3 空格才算围栏行，≥4 空格按缩进代码块处理（不作围栏）；tilde `~~~` 不识别、按普通行；§1.4 表述由"完全遵循 CommonMark"改为精确的子集立场 | 原 §3.3"trim 后比较"使 ≥4 空格缩进反引号行也被当围栏，与 §1.4 合规声明互斥（Grace M-7）。被否决替代：显式声明"本项目放宽：任意缩进均按围栏"——制造与 CommonMark 渲染器的结构分歧且无收益，否决。 |
| R14 | 消息头 `#N` token 保留，§2 补专论行 | `#N` 为业界编号引用惯用法（issue 引用风格），与 reply-to 值文法构成同一 token 族（Jack F2）。被否决替代逐一列明：`## N. sender`（列表项文法错位、失去 token 族同一性）、`## Message N: sender`（英文保留词、冗长）、去 `#`（自然标题撞形风险）。论证补全后保留。 |
| R15 | v0_feedbacks 第 27 行适用域裁决：fence 包裹仅适用于 post 消息正文；brief note 与 profile description 属文档元叙述，为裸散文 | 第 27 行"使用者输入的内容"未界定适用域，brief note/profile description 裸散文落盘在语义上可辩护（元叙述非转发内容）但六文档无一处裁决（Jack F8）。写明适用域后，note/description 中的结构同形行按 §3.2/§6.2 的边界规则处置（BDD:BRIEF-12、PROF-11）。 |

### 8.3 评审裁决记录（C1–C10，补全级）

| 编号 | 落实位置 |
| --- | --- |
| C1 | design §3 残余风险披露时间戳失败后果；spec §5.6 声明 sender 非 ASCII/无长度上限 |
| C2 | spec §5.4 三句（info 任意、多围栏取首、reply-to 宽容 None） |
| C3 | spec §7.2/§7.3（`<`/`>` 触发集、解析反转义、title 忽略）；BDD:CONT-08 |
| C4 | spec §5.3（消息头必须顶格） |
| C5 | spec §5.7（preamble 不受 64KB 约束） |
| C6 | spec §10 I4 措辞；spec §5.7 与 design §7 崩溃窗口声明 |
| C7 | BDD:POST-31（plain 无 preamble）；impl_plan S5.3（unreadable 症状入迁移指南）；impl_plan S1.5（brief verify trim 核对项） |
| C8 | bdd 增补 16 场景（POST-21..30、CONC-03/04、VAL-07、BRIEF-12、CONT-08、PROF-11）+ POST-31/POST-32 + VAL-08；既有修订（POST-06/07/14、VAL-02、CONT-03）；tdd §6 同步 |
| C9 | impl_plan（三 README、验收精确检索式、`--title` 算法、S3.1 统一、前置条件条件式）；tdd §1 现状清点修正；role 补 rust-version/pwsh 前置；本节本身 |
| C10 | design §2（participants 身份名论证 F9、属性行如实表述+粗体键否决 F10、记录单元 contacts 例外 F11、regex 私有标注 F13）；POST-32/T-OPS 尾扫缓冲截断（F15） |

### 8.4 流程符合性裁决落实索引（Ben F-1..F-13）

F-1 → R1；F-2 → R11；F-3 → R10/VAL-07/impl_plan S3.2；F-4/F-5 → impl_plan S5.2 三 README 与 §7 验收精确检索式；F-6 → R6；F-7 → tdd §1 现状清点按真实数字/条目名修正；F-8 → `--title` 缺省算法（spec §5.7、impl_plan S3.1、BDD:POST-19）；F-9 → impl_plan S3.1"CLI 恒传 Some(meta)，锁内 size 判定守门"；F-10 → §8.1；F-11 → R10；F-12 → impl_plan 前置条件改条件式；F-13 → role 文档环境前置。

### 8.5 owner 追裁记录（D1–D3，2026-08-09，定稿级）

> v0.5.0 "Format Renewal" 实现完成后（hard breaking、未发布 crates.io），owner 对 post/thread 格式给出三项定稿级裁决；profile/brief/contacts 三格式固定不动。**版本决策：0.5.0 未发布，本轮变更并入 0.5.0，不 bump 版本。**本节与早期裁决（R 系列）冲突处以本节为准；spec.md/bdd.md/tdd.md/impl_plan.md 已同步联动。

| 编号 | 裁决 | 理由与落点 |
| --- | --- | --- |
| D1 | 废除 participants：preamble 中 `- participants:` 属性行删除；preamble 仅剩 H1 标题（标题行后允许自由散文，解析忽略）；participants 语义在需要时（如 summary）由消息 sender 集合派生（按首次出现顺序去重） | 对话消息中已包含全部发言者，维护名单是冗余负担且与 sender 双源漂移。落点：spec §2/§5.2/§5.4/§5.7/§5.9；`ThreadMeta { title }`；BDD POST-01/08/09/14/15/16/19/27、CONC-02/04 |
| D2 | 废除消息属性行 `reply-to`/`mentions`/`to`：引用状态不再是结构化字段，而是正文文本引用——`@somebody`（正文内出现）= mention；`@#N`（N 为消息序号）= reply 引用；读取/统计（summary 等）时从正文文本实时派生，不落盘；`to` 字段彻底删除（不再有定向发送的结构化概念，如需定向用户在正文自行 @） | 引用状态是正文语义的冗余投影，单源化后零维护、零新语法（`@name`/`@#N` 均为业界惯例）。落点：spec §2/§3.2/§5.4/§5.6/§5.9/§8/I10；`Message` 删三字段；BDD POST-01/02/03/26/33/34/35 |
| D3 | 正文围栏简化：` ```markdown ` 改为 ` ```md `；序列化严格写 `md`；解析宽容，`md` 与 `markdown` 均接受（info-string 前缀匹配，CommonMark 规则）；动态围栏长度算法不变（max(3, 正文最长连续反引号串+1)） | `md` 是社区惯例缩写，写入更短；前缀匹配使存量 `markdown` 围栏仍可读。落点：spec §2 第 4 条/§3.4/§5.4/§5.9；BDD POST-06/25 |

**leader 默认规则（写入规格，spec §5.4 规范性条文）**：

- mentions 派生：扫描正文文本，正则形如 `@([^\s@()]+)`，按出现顺序去重；排除 sender 本人的自提及；`@#N` 形态 token 归 reply 引用不计入 mentions（规格完备化补充，避免 reply token 污染 mentions）。
- reply-to 派生：扫描正文 `@#(\d+)`，取首个合法引用（其余忽略）；不校验引用目标是否存在（宽容）。
- 消息头正则 `^##\s+#(\d+)\s+([^\s()]+)\s+\((.+)\)\s*$` 不变；seq 校验、64KB 上限、fs2 锁、fence 感知尾扫全部不变。
- `post send` 的 `--to`/`--participants` flag 删除（保留 `--title`）；新文件首写 preamble 仅剩标题；`--reply-to`/`--mention` 糖衣 flag 去留见 spec §11 OQ-4（默认：保留，语义改为正文 token 注入）。

**Rejected：结构化属性行方案**（即维持原 R2/R4 的 preamble `- participants:` + 消息 `- reply-to:/- mentions:/- to:` 属性行）：

1. participants 名单与消息 sender 双源共存，需人工同步且必然漂移；对话消息本身已包含全部发言者，名单是冗余负担。
2. reply/mention 本质是正文语境中的文本引用，结构化属性行是正文语义的冗余投影：写侧双重维护（正文写 @ 同时还要同步属性行）、读侧双通道一致性无保障。
3. 结构化字段的类型安全收益在本项目不成立：引用目标本就不校验存在性（宽容解析立场不变），属性行徒增解析/序列化/校验代码面。
4. 正文文本引用（`@somebody`/`@#N`）零新语法、单源自洽，与消息头 `#N` token 族同形互证（R14 论证在正文引用形态下继续成立）。
