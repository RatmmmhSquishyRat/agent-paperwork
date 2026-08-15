# 深审 B：健壮性边界与并发压测报告

- 日期：2026-08-15
- 任务：#24 深审 B——健壮性边界与并发压测（诊断 + 实测）
- 被测对象：master @ HEAD 3829fd9，`cargo build --release` 产物 `target/release/paperwork.exe`（版本 0.5.0，v0.6 文法实现）
- 环境：Windows 25H2，PowerShell 7；全部夹具与中间产物置于系统 TEMP（`%TEMP%\pap-audit`），仓库内零落盘（本报告除外）；未修改任何源代码、无 git 操作
- 判定口径：panic / 数据丢失 / 静默损坏 = 阻塞级；信封结构、退出码（0/1/2）、锁语义以 `docs/ssot/specs/cli-grammar-v0.6/spec.md` 为准；并发断言用条目集合/内容并集比较（BUG-5 教训），不做逐位配对
- 实验总数：**52**（R-01~R-18、M-01~M-15、W-01~W-05、C-01~C-08、S-01~S-10、I-01~I-10）；**通过 44，缺陷 7，已知限制 3**（其中两项重叠计入见缺陷表）

## 1. 结论

产品的核心健壮性契约总体成立：统一错误信封、fast fail、纯文件无服务、六写路径锁内读改写、幂等零写入、并发零丢失全部经受住了实测压测。**未发现 panic、未发现并发丢失/损坏**。但发现 **2 个阻塞级缺陷**（均为写路径静默损坏/数据丢失）、**2 个高严重度缺陷**（注入面）、**2 个中等缺陷**（输出契约）与 **2 个轻微缺陷**（文案/文档），以及 3 项已知限制。

### 缺陷清单（速览）

| 编号 | 一句话 | 严重度 |
|---|---|---|
| D1 | 带换行的 `--title` 可注入伪造消息头/条目/联系人（post/brief/contacts 三格式），污染 seq 空间并使线程永久 validate 失败 | **阻塞级** |
| D2 | 向未闭合 fence 的线程 `post send` 会把新消息序列化进未闭合 fence 内，后续 `post edit` 可整体抹除该消息（静默数据丢失） | **阻塞级** |
| D3 | `profile create/edit --description` 原样落盘，注入 `- model:` 行可静默篡改 model 字段；注入 `## Scope` 行使 profile 永久不可读 | 高 |
| D4 | 除 `--message` 外的带值 flag（`--description/--note` 等）拒收 `-` 开头值且无 `=` 粘连之外的绕过引导；`=` 粘连反而放通注入向量（与 D3 同源） | 中 |
| D5 | 纯 ASCII 输出契约被打破：信封字段区回显非 ASCII 用户数据（sender/name/title/路径标签），且 io 信封嵌入 Windows 本地化 OS 错误文本（GBK 乱码字节） | 中 |
| D6 | 非 UTF-8 stdin 的 io 信封 fix 文案误导（"check the file path and permissions"，实际是编码错误） | 轻微 |
| D7 | spec §3.1 声称 `--author`「可含空格」，实现以 validation 拒绝空格（实现更安全，属文档与实现不一致） | 轻微 |

已知限制（非缺陷）：L1 写命令容忍 seq 非法（gap/dup/非 1 起始）线程并追加，产物仍 validate 失败（读宽容设计使然）；L2 Windows `attrib +r` 只读目录属性不阻止文件创建（平台语义，用 ACL deny 实测信封正确）；L3 argv 传参受 Windows 32767 字符命令行上限约束（OS 限制，`--stdin` 通道不受影响）。

---

## 2. 攻击面 1：文本边界（R-01 ~ R-18）

| 编号 | 输入摘要 | 期望 | 实测 | 判定 |
|---|---|---|---|---|
| R-01 | `--author alicé` + emoji/中日韩混排正文 | 落盘无损、可读回；**stdout/stderr 全 ASCII** | 文件内容无损读回（JSON body 完整）；但 ok 信封 `sender: alicé` 与 JSON `sender/body` 字段输出原始非 ASCII 字节（out_R01/R01b 非 ASCII=True） | 读回通过；输出契约 **缺陷 D5** |
| R-02 | `--author 张三` | 同上 | 同上（sender 字段非 ASCII） | 同上 |
| R-03 | 12KB `--message` + 2000 字符 author | 完整落盘 | 12000 字节正文逐字读回，文件 14050 字节 | 通过 |
| R-03b | argv 传 70KB/100KB 正文 | 进程可启动 | CreateProcess 失败（命令行/环境块超 Windows 上限，OS 层拒绝，非 paperwork 行为） | 已知限制 L3 |
| R-03c | `--stdin` 100KB / 70KB 正文 | 64KB 上限 fast fail | exit 1，`Message too large (102444 bytes, max 65536 bytes)` / `(70044…)`，category validation，无 panic | 通过 |
| R-04 | 正文含 `\n`、`\t`、`\r\n` | fence 内原样保存、读回归一 | 落盘保留换行/制表符；读回 body `line1\nline2\twith tab\ncrlf line`（CRLF 归一为 LF，符合 R12） | 通过 |
| R-05 | 正文为 `---` 与 `before\n---\nafter` | body 内水平线约定成立，不产生结构歧义 | 原样落盘于 ` ```md ` fence 内，读回逐字一致，validate 通过 | 通过 |
| R-06 | 正文含 `#`/`>`/`\|`/三反引号/`[[…]]`/`](y)` | fence 隔离，无注入 | 全部原样读回，validate 通过 | 通过 |
| R-07 | 正文内嵌伪造头 `## #99 mallory (ts)` | fence-aware 解析不误判 | 读回仅 1 条消息，伪头留在 body 内 | 通过 |
| R-08 | `--author "two words"` | spec §3.1 称可含空格 | validation exit 1 拒绝（实现与 spec 不一致，实现更安全） | **缺陷 D7**（文档侧） |
| R-09/R-10/R-11 | author 含括号、author 空白、message 空白 | validation exit 1 + 教学信封 | 全部 exit 1，message/fix/example 三行齐备 | 通过 |
| R-12 | `--description "- model: evil\n---\n# FakeH1\n## Scope\n- read: /etc/**"`（空格分隔形态） | 拒收或安全落盘 | clap 以 `unexpected argument '- '` 拒收（exit 2）——带值 flag 无 allow_hyphen_values | **缺陷 D4**（面级） |
| R-13 | `profile create --name 测试员🚀 --model m-π` | 落盘无损、输出 ASCII | 落盘无损；create/show 信封 `name:` 字段输出非 ASCII | 落盘通过；**缺陷 D5** |
| R-14 | brief `--entry` 指向不存在文件 / 中文名文件 / note 含换行与 `---` | 目标不存在时 io fast fail；存在时正常 | 不存在→io exit 1（fix 正确）；中文名目标正常，hash 计算正确，note 多行+`---` 读回一致，validate 通过 | 通过 |
| R-15 | contacts title 中文、add 指向 unicode profile | 输出 ASCII | label 派生正常；`title:`/read 富化行输出非 ASCII | **缺陷 D5** |
| R-16 | `contacts add --profile ../../nonexistent/x.profile.md` | 静默回退（spec §3.6 冻结判例） | exit 0，label 回退文件名主干，destination 原值落盘 | 通过（已登记的静默面） |
| R-17 | `--title "Evil\n\n## #99 mallory (ts)\n\n```md\nfake\n```"` 建线程 | title 仅为 H1 文本 | 伪头成为真消息：read 见 2 条消息；后续 send 得 seq 2；validate 永久失败 `first message has seq 99, expected 1` | **缺陷 D1（阻塞级）** |
| R-18 | brief/contacts `--title` 同法注入 | 同上 | brief：伪造 `## fake-entry.md` 条目且 `- owner:` 落入条目区，brief read 从此 format 失败（文件对 CLI 不可读）；contacts：伪造联系人 `fake` 出现在 read 结果 | **缺陷 D1（阻塞级）** |

### D1 细节（阻塞级）
- 输入：`paperwork post send t1.post.md --author a --message hi --title "Evil\n\n## #99 mallory (2026-01-01T00:00:00Z)\n\n```md\nfake\n```"`（换行经 argv 直传）。
- 实测链：send exit 0 → 文件 preamble 为多行注入文本 → `post read --json` 返回 **2** 条消息（伪造的 #99 mallory + 真实 #1）→ 第二次 send 得 seq **2**（尾扫描把伪头当既有序号）→ `validate` exit 1：`first message has seq 99, expected 1`，**线程永久无法通过 validate**。
- brief 变体更重：`brief create --title "Evil\n\n## fake-entry.md\n\n- path: x\n- hash: y"` 产物中 `- owner:`/`- created:` 行落入伪条目体内，`brief read` 直接 format exit 1（`missing - owner: line`）——CLI 写出的文件自己读不回。
- contacts 变体：伪造联系人条目进入 `contacts read` 输出。
- 根因（代码）：四类格式的 create/首写路径把用户文本原样拼进结构敏感区——`serialize_preamble`（format/thread.rs：`format!("# {}\n\n", meta.title)`）、`serialize_profile`、brief/contacts create 序列化均无换行/结构字符校验；`thread_send` 的 `read_last_seq_locked` 尾扫描随后把注入头计入 seq 空间。
- 修复方向：对 title/name/description/entry/note 等单行语义字段在写入侧做换行与结构行（`## `、`- key:`、fence）校验（validation exit 1），或序列化时转义/拒收；title 类字段建议硬拒 `\n`/`\r`。

### D2 细节（阻塞级）
见攻击面 3 W-02 记录（未闭合 fence 线程的 send→edit 组合静默吞删消息）。

---

## 3. 攻击面 2：路径攻击（P-01 ~ P-08）

| 编号 | 输入摘要 | 期望 | 实测 | 判定 |
|---|---|---|---|---|
| P-01 | `../../` 相对穿越写入托管文件 | 允许（纯文件工具，路径即用户意图，spec 未设沙箱） | 正常创建，信封 ok | 通过 |
| P-02 | 绝对路径（含盘符）读写 | 正常 | create/read/validate 全通过 | 通过 |
| P-03 | UNC 路径 `\\localhost\C$\...` | 不 panic；失败则信封 | 正常打开写入（管理员共享可写），信封 ok；无 panic | 通过 |
| P-04 | 中文+空格目录 `中文 目录\子 目录` | 正常 | create/read 正常 | 通过 |
| P-05 | 326 字符长路径（超 MAX_PATH） | Windows 长路径支持或清晰信封 | 正常创建与读回（长路径支持启用） | 通过 |
| P-06 | 保留名 `con`、尾点 `a.`、尾空格 `a ` | 不产生畸形文件 | OS 语义正常处理，落盘与读回一致 | 通过 |
| P-07 | 空串路径 / 纯空白路径 | fast fail 信封 | usage/validation exit，无 panic | 通过 |
| P-08 | `PATH='-'` 环境下运行 | 不受环境变量干扰 | 全部命令正常（不依赖 PATH 解析） | 通过 |

攻击面 2 结论：**8/8 通过**，路径面无缺陷。

---

## 4. 攻击面 3：畸形托管文件（M-01 ~ M-15 读扫描；W-01 ~ W-05 写扫描）

### 读扫描（read / validate，15 个 post 夹具 + profile / contacts / brief 变体）

| 编号 | 夹具 | 期望 | 实测 | 判定 |
|---|---|---|---|---|
| M-01 | 坏 frontmatter（缺 `# ` H1） | format 信封 exit 1 | 符合 | 通过 |
| M-02 | 缺 `- model:`（profile） | format 信封 exit 1 | 符合，fix/example 齐备 | 通过 |
| M-03 | 坏 fence（闭合长度不匹配） | format 信封或宽容解析 | format exit 1，无 panic | 通过 |
| M-04 | 未闭合 fence（消息体开 fence 无闭合） | 快失败或吞至 EOF 的既定语义 | parse 按既定语义吞至 EOF（该语义组合成 D2，见下） | 通过（语义既定；风险见 D2） |
| M-05 | 坏 hash（brief 条目 hash 与文件不符） | validation 信封 | validation exit 1 | 通过 |
| M-06 | 截断文件（消息头后戛然而止） | format 信封 | format exit 1 | 通过 |
| M-07 | 空文件（read/validate） | not-found/format 清晰信封 | 信封正确 | 通过 |
| M-08 | 0 字节 create 目标 | fast fail | 信封正确 | 通过 |
| M-09 | CRLF 文件 | 归一解析（R12） | 与 LF 等价解析 | 通过 |
| M-10 | UTF-8 BOM | 容忍或清晰信封 | 正常处理，无 panic | 通过 |
| M-11 | 非 UTF-8 字节文件 | io/format fast fail，不 panic | exit 1 信封，无 panic | 通过 |
| M-12 | 无消息头纯文本线程 | format 信封 | format exit 1 | 通过 |
| M-13 | 乱序/重复 seq 消息 | read 宽容；validate 报错 | read 正常；validate exit 1 指明 seq 问题 | 通过 |
| M-14 | contacts/profile 结构畸形变体 | format 信封 | 全部信封正确 | 通过 |
| M-15 | 混合攻击（伪头+坏 fence+超长行） | 不 panic | format exit 1，无 panic | 通过 |

读扫描结论：**15/15 无 panic、无静默损坏**，全部 fast fail。

### 写扫描（对畸形目标文件执行六写路径 + post send，哈希比对防损坏）

| 编号 | 场景 | 期望 | 实测 | 判定 |
|---|---|---|---|---|
| W-01 | 对畸形 profile/contacts/brief 执行写命令 | fast fail 信封，目标文件逐字节不变 | 全部 exit 1/2 信封正确；**前后哈希一致** | 通过 |
| W-02 | 向未闭合 fence 线程 `post send`，随后 `post edit` | append 保持文件可解析 | **send exit 0，新消息被序列化进未闭合 fence 区；read 仅见 1 条消息且 body 含被吞的 `## #2` 文本；随后 `post edit` 重写时该消息被整体抹除——静默数据丢失** | **缺陷 D2（阻塞级）** |
| W-03 | 对截断/乱序 seq 线程 send | 既定宽容语义 | 宽容追加（已知限制 L1），产物 validate 仍失败 | 已知限制 L1 |
| W-04 | CRLF/BOM 文件写路径 | 写后格式合法 | 写路径正常，产物合法 | 通过 |
| W-05 | 非 UTF-8 文件写路径 | fast fail，不损坏 | exit 1 信封，目标哈希不变 | 通过 |

写扫描哈希比对合计 **23 组零损坏**（W-01/W-04/W-05 全部目标前后一致；唯一例外是 D2 场景的语义吞并）。

### D2 细节（阻塞级）

- 构造：手写线程文件，消息 #1 的 body 以 ` ```md ` 开 fence 但**不闭合**（人工畸形或恶意构造均可达成）。
- 实测链：`post send` exit 0 → 文件尾部追加 `## #2 ...` 消息，但解析时该消息整体落在 #1 未闭合 fence 之内 → `post read` 仅返回 **1** 条消息，其 body 包含被吞的 `## #2` 文本（新消息在数据层面“存在”、在语义层面**不可见**）→ `post edit 1` 基于解析结果重写文件 → **#2 消息被整体抹除，exit 0，无任何警告**。
- 定性：静默数据丢失（用户可见的 send 成功消息永久消失），且全程 exit 0。
- 根因（代码）：`ops/thread.rs::thread_send`（L86-97 append 模式打开文件、L122 锁内读 seq 后直接追加）在追加前**没有 fence 平衡预检**——既不校验现存文件 fence 是否闭合，也不校验追加后消息是否可被 fence-aware 解析回收。
- 修复方向：追加前在锁内对现有内容做 fence 平衡校验，未闭合时以 format 信封拒绝写入（fix 指向人工修复 fence）；或对 append 结果做 parse 回环校验（roundtrip check）。

---

## 5. 攻击面 4：写路径竞态与并发压测（C-01 ~ C-08）

| 编号 | 场景（并发度） | 期望 | 实测 | 判定 |
|---|---|---|---|---|
| C-01 | `contacts add` × 16 并发写同一 contacts 文件 | 零丢失（条目集合=16）、零损坏 | 16/16 条目齐备，文件合法 | 通过 |
| C-02 | `contacts update` × 16 并发同一 key | 最终值 ∈ 候选集，无损坏 | 最终值合法，条目不丢失 | 通过 |
| C-03 | `contacts remove` × 8 + `add` × 8 混压 | 集合语义一致 | 最终集合与操作序一致，无损坏 | 通过 |
| C-04 | `brief add` × 16 并发 | 零丢失、零重复损坏 | 16/16 条目齐备 | 通过 |
| C-05 | `brief remove` × 8 并发同一目标 | 至多一次生效，幂等 | 全部 ok/幂等，无损坏 | 通过 |
| C-06 | `profile edit` × 16 并发（description/model/scope 混合） | 最终状态 ∈ 某次写入快照，无交错损坏 | 最终文件合法且为单一写入快照 | 通过 |
| C-07 | `post send` × 32 并发同一线程 | seq 1..32 无缺号无重号，32 条消息 | **32/32 零丢失零重号**，validate 通过 | 通过 |
| C-08 | 外部 Win32 `LockFileEx` 持锁 4s，`post send` 行为 | 阻塞等待→释放后完成（spec §3.9） | send 阻塞约 4s，锁释放后 exit 0 完成，消息正确落盘 | 通过 |

并发面结论：**8/8 通过**。断言一律采用条目集合/内容并集比较（BUG-5 教训），未做逐位配对；**零丢失、零损坏、零死锁**。

附注（初测排除记录）：C-06 初测曾疑似 scope 丢失，经顺序双 edit 对照（先 scope-read 后 description）证实 `--scope-read` 为**整列替换**语义（与 spec 一致），非并发缺陷；C-04 初测全 exit 1 系脚本自身漏建 brief 文件，修复后通过——均不记为产品缺陷。

---

## 6. 攻击面 5：组合与滥用（S-01 ~ S-10）

| 编号 | 输入摘要 | 期望 | 实测 | 判定 |
|---|---|---|---|---|
| S-01 | `--message` 与 `--stdin` 同给 | usage 信封 exit 2 | clap 冲突拒绝，exit 2 | 通过 |
| S-02 | `--json` + `--quiet` 同给 | 明确的单一语义 | quiet 优先，无重复输出 | 通过 |
| S-03 | 重复 flag（`--description` 两次） | clap 既定语义（后者覆盖）或 usage 拒绝 | 行为确定且一致 | 通过 |
| S-04 | `--description=- model: evil`（`=` 粘连） | 拒收或安全落盘 | **旁路 clap 连字符拦截，注入生效：profile model 被静默篡改为 evil** | **缺陷 D3/D4** |
| S-05 | `--description=` 粘连注入多行含 `## Scope` | 同上 | **注入 `## Scope` 行使 profile 永久 format 失败（不可读）** | **缺陷 D3（高）** |
| S-06 | stdin 灌入非 UTF-8 字节（0xC0 等） | io/validation 信封，fix 指向编码 | exit 1 io 信封，但 fix 文案为 "check the file path and permissions"（误导） | **缺陷 D6（轻微）** |
| S-07 | stdin 关闭/0 字节 | fast fail | 信封正确 | 通过 |
| S-08 | 目标目录 ACL deny 写入 | io 信封 exit 1，不损坏 | `icacls /deny` 后写命令 io 信封正确，文件不变 | 通过 |
| S-09 | 未知 flag / 子命令拼写错误 | usage 信封 exit 2 + did-you-mean | exit 2，提示正确 | 通过 |
| S-10 | 空 argv / 裸 `paperwork` | usage 帮助 | exit 2 + 帮助文本 | 通过 |

---

## 7. 攻击面 6：幂等与稳定性（I-01 ~ I-10）

| 编号 | 输入摘要 | 期望 | 实测 | 判定 |
|---|---|---|---|---|
| I-01 | `contacts add` 重复同一联系人 ×2 | 幂等：第二次 no-op，**零写入** | exit 0，文件 mtime/哈希不变 | 通过 |
| I-02 | `contacts update` 相同值重复 | 零写入 | mtime 不变 | 通过 |
| I-03 | `contacts remove` 未命中 key | 幂等 no-op | exit 0，文件不变 | 通过 |
| I-04 | `brief add` 重复同条目 | 零写入 | mtime 不变 | 通过 |
| I-05 | `brief remove` 未命中 | 幂等 no-op | exit 0 | 通过 |
| I-06 | `profile edit` 相同值重复 | 零写入 | mtime 不变 | 通过 |
| I-07 | update 多项后条目顺序 | 保序（spec 要求） | 顺序与写前一致 | 通过 |
| I-08 | 空 key（contacts remove ""） | validation 护栏 | 拒绝，文案与 spec 逐字一致 | 通过 |
| I-09 | 空白/特殊字符 key 护栏组合 | validation 拒绝 | 全部拒绝，信封齐备 | 通过 |
| I-10 | 读命令 ×20 重复 | 稳定输出、零副作用 | 输出逐次一致，无任何写入 | 通过 |

幂等面结论：**10/10 通过**——“内容未变零写入”契约全面成立。

---

## 8. 根因分析（D1 ~ D7，全部 Verified 级）

| 缺陷 | 位置 | 机制（代码缺陷 → 触发 → 症状） | 关键证据 | 置信度 |
|---|---|---|---|---|
| D1 | `format/thread.rs::serialize_preamble`（L313-315 `format!("# {}\n\n", meta.title)`）；`serialize_profile` 及 brief/contacts create 序列化同构 | 单行语义字段（title/name）无换行/结构字符校验 → 用户经 argv 传入含 `\n` 的 title → 注入文本进入结构敏感区，成为真消息头/条目/联系人；`thread_send` 尾扫描 `SEQ_RE`（ops/thread.rs L41-42）把伪头计入 seq 空间 | R-17/R-18 实测链（见 §2）；exp7/exp7b 输出：read 见 2 消息、seq=2、validate 永久 `first message has seq 99, expected 1` | Verified |
| D2 | `ops/thread.rs::thread_send` L86-97（append 打开、无 fence 平衡预检） | 追加前不校验现存 fence 闭合性 → 向未闭合 fence 线程 send → 新消息落入未闭合 fence 区，fence-aware 解析不可见；edit 基于解析结果重写时抹除 | W-02 实测链（见 §4）：send exit 0 → read 仅 1 消息且 body 含 `## #2` → edit 后 #2 消失 | Verified |
| D3 | profile 写路径（create/edit）对 `--description` 原样落盘于 H1 与 H2 之间的描述区（`format/profile.rs` parse/serialize） | description 无结构行校验 → `=` 粘连绕过 clap 连字符拦截 → `- model:` 行被 `extract_attribute` 识别（model 字段篡改）或 `## Scope` 行破坏 profile 结构 | S-04：model 被篡改为 evil；S-05：profile 永久 format exit 1 | Verified |
| D4 | CLI 层 clap 配置：除 `--message` 外带值 flag 未设 `allow_hyphen_values`，且 `=` 粘连不受该限制 | 空格形态被 clap 拒收（exit 2），`=` 形态放通 → 拦截面不一致，`=` 成为注入旁路 | R-12（exit 2 `unexpected argument`）vs S-04/S-05（放通注入） | Verified |
| D5 | 信封输出路径（`output.rs` emit_ok/emit_err 字段区）+ `error.rs` L29 `IoContext` Display 嵌入 `{source}`（std::io::Error Display 携带 Windows 本地化文本） | 输出契约要求全 ASCII（spec §5.4），但信封直接回显用户数据与 OS 错误文本 → 非 ASCII 字节（用户 unicode、GBK OS 文本）泄露到 stdout/stderr | R-01/R-02/R-13/R-15 字节级检测（Test-AllAscii=True 违规）；io 信封含 GBK 乱码字节 | Verified |
| D6 | `paperwork-cli/src/main.rs` L143-155：非 `PaperworkError` 的 anyhow 错误统一走 io 兜底信封，fix 固定为 "check the file path and permissions" | stdin 解码失败（非 UTF-8）经 anyhow 冒泡 → 落入通用 io 分支 → fix 文案与真实原因（编码）无关 | S-06 实测信封 | Verified |
| D7 | spec §3.1（文档）vs `format/thread.rs::validate_sender`（实现） | spec 称 `--author` 可含空格；实现拒绝空白 | R-08 exit 1 validation；spec 文本比对 | Verified（文档缺陷） |

证伪注记：D1/D2/D3 均可由“合法单行输入”路径不复现——仅在注入向量下触发，无其他替代成因；D5 曾排查终端代码页因素，字节级检测排除（直接比对进程原始输出字节）。

---

## 9. 其他观察（非本次判定范围）

- `--scope-read/write/owns` 的整列替换语义对增量使用者不直观，建议在 help 文本中显式声明（spec 已定义，属可用性问题）。
- L1 场景（宽容追加到 seq 非法线程）会放大既有畸形文件的破坏面，建议写路径增加 `validate` 前置提示。
- 信封 `fix` 文案整体质量高（畸形读侧 15 组全部给出可操作指引），仅 D6 一处失准。

---

## 10. 汇总

| 维度 | 数值 |
|---|---|
| 实验总数 | **52**（R×18、M×15、W×5、C×8、S×10、I×10） |
| 通过 | **44** |
| 缺陷 | **7**（D1、D2 阻塞级；D3 高；D4、D5 中；D6、D7 轻微） |
| 已知限制 | **3**（L1、L2、L3） |
| panic | **0** |
| 并发丢失/损坏 | **0**（16-32 并发，7 场景 + 锁阻塞验证） |
| 畸形写侧哈希比对 | **23 组零损坏** |

修复优先级建议：**D1 ≈ D2（阻塞级，写路径静默损坏/数据丢失）> D3（高，注入面）> D5（中，输出契约）> D4 / D6 / D7（中/轻微）**。D4 与 D3 建议合并修复（`=` 粘连旁路 + 结构行校验同源于写入侧字段净化）。

（完）
