# Managed File Format v2 行为场景（BDD）

> **文档性质**：四格式的 Given/When/Then 行为场景全集（Normative 行为基线）。每个场景标注对应 spec.md 章节；tdd.md 以本文场景编号（如 `POST-05`）建立测试映射。
>
> **版本说明**：本文为阶段 1 对抗性评审后按 leader 裁决的 rework 定稿——新增 POST-21..32、CONC-03/04、VAL-07/08、BRIEF-12、CONT-08、PROF-11；修订 POST-06/07/14、VAL-02、CONT-03 等既有场景。编号顺延不冲突。
>
> **2026-08-09 owner 追裁（D1–D3）联动**：改写 POST-01/02/03/06/08/09/14/15/16/19/25/26/27、CONC-02/04；新增 POST-33/34/35（正文 @mention 派生、@#N reply-to 派生、派生不落盘与非法 token 宽容）。
>
> **场景编号约定**：`PROF-*`（profile）、`POST-*`（post/thread）、`BRIEF-*`（brief）、`CONT-*`（contacts）、`CONC-*`（并发）、`VAL-*`（validate 语义）。
>
> **通用前提**（适用所有场景，不再重复书写）：
> - 解析前执行 CRLF 归一化（spec §3.1，不变量 I11）；
> - 解析全程 fence 感知（spec §3.3：≤3 空格缩进、tilde 不识别）；
> - 未识别内容按宽容解析忽略（spec §3.6）。

---

## 1. profile（PROF-*）

### PROF-01 最小有效 profile（spec §4.2）

```gherkin
Given 文件内容为 "# alice\n\n- model: gpt-4o\n"
When  parse_profile 解析该文件
Then  解析成功
And   name == "alice"，model == "gpt-4o"
And   description 为空，scope_read/scope_write/scope_owns 均为空
```

### PROF-02 description 散文段与 Scope 属性行列表（spec §4.1/§4.2，R3）

```gherkin
Given 文件含 H1 "# alice"、H1 后散文段 "Parser module implementer"、
      属性行 "- model: gpt-4o" 与 "## Scope" 下的属性行列表
      （行：- read: src/**、- write: src/parser/**、- owns: src/parser/**）
When  parse_profile 解析该文件
Then  description == "Parser module implementer"
And   scope_read == ["src/**"]，scope_write 与 scope_owns == ["src/parser/**"]
```

### PROF-03 同一 permission 多行多 glob（spec §4.2，R3）

```gherkin
Given Scope 属性行列表含两行 read（"- read: src/**" 与 "- read: docs/**"）
When  parse_profile 解析该文件
Then  scope_read == ["src/**", "docs/**"]（按行序保序聚合）
```

### PROF-04 空 scope = 省略整节（spec §4.2/§4.3）

```gherkin
Given Profile 结构体三个 scope 均为空
When  serialize_profile 序列化
Then  输出不含 "## Scope" 字样、不含任何 scope 属性行、不含 "—" 占位
And   输出再解析回的结构体与原结构体相等（roundtrip，同 PROF-10）
```

### PROF-05 坏例：缺 H1 拒绝（spec §4.4）

```gherkin
Given 文件不含任何 H1，仅有 "- model: gpt-4o"
When  parse_profile 解析该文件
Then  返回 Parse 错误，message 含 "missing agent name heading"
```

### PROF-06 坏例：缺 model 拒绝（spec §4.4）

```gherkin
Given 文件含 "# alice" 但无 "- model:" 行
When  parse_profile 解析该文件
Then  返回 Parse 错误，message 含 "missing - model:"
```

### PROF-07 宽容解析：未知内容忽略（spec §3.6）

```gherkin
Given 文件含未知属性行 "- favorite: rust"、未知节 "## Notes" 及其正文、
      Scope 节中未知 permission 行 "- admin: /etc/**"
When  parse_profile 解析该文件
Then  解析成功；未知属性、未知节被忽略
And   "- admin:" 行不计入任何 scope 集合
```

### PROF-08 CRLF 归一化（spec §3.1）

```gherkin
Given 有效 profile 内容以 CRLF 换行
When  parse_profile 解析该文件
Then  解析结果与同内容 LF 版本完全一致
```

### PROF-09 Unicode（spec §3.6）

```gherkin
Given name 为 "ünïcödé"、model 为 "mödel-π"、description 含 emoji 与中文
When  parse_profile 解析该文件
Then  各字段按原 Unicode 值解析成功
```

### PROF-10 序列化 roundtrip（spec §4.3）

```gherkin
Given 任意合法 Profile 结构体（含与不含 description/scope 各一例）
When  serialize_profile 后 parse_profile
Then  结果与原结构体逐字段相等
And   序列化输出不含 "·"、"—"、反引号包裹的 glob、GFM 表格行等旧构造
```

### PROF-11 description 散文含 bullet 同形行的归属（spec §3.2/§3.6，R4/R15）

```gherkin
Given profile 的 description 散文段中含一行 "- anything: value"
When  parse_profile 解析该文件
Then  该行按 preamble 区属性行识别（未知键忽略，宽容解析）
And   description 仅由非 bullet 散文行构成，不含该行
And   解析成功，不报错
```

## 2. post / thread（POST-*）

### POST-01 正常路径：preamble 仅 H1 + 正文引用消息（spec §5.1–§5.4，D1/D2）

`````gherkin
Given 文件内容为（示意，正文均在 ```md 动态围栏内）：
      H1 "Daily Standup"（preamble 仅 H1，无 - participants: 行）、
      消息 "## #1 alice (2026-08-01T19:38:22Z)"、
      消息 "## #2 bob (2026-08-01T19:38:22Z)"，正文含 "@alice @#1 tests merged"
When  parse_preamble + parse_messages 解析
Then  ThreadMeta.title == "Daily Standup"（无 participants 字段，D1）
And   消息数 == 2；消息结构无任何属性行（无 reply-to/mentions/to 字段，D2）
And   各消息 body 为围栏内文本（首尾空白行已规范化去除，§5.4/R12）
And   #2 的 mentions/reply-to 仅可由正文派生得出（派生规则见 POST-03/POST-33）
`````

### POST-02 广播 = 无 @ 的普通消息（spec §5.4，D2）

```gherkin
Given 消息正文完全不含任何 @ 引用（广播即普通消息）
When  解析并派生
Then  派生 mentions == []，reply-to == None
And   序列化广播消息的输出仅含消息头与 ```md 围栏正文，无任何属性行
```

### POST-03 正文 @mention 派生（spec §5.4，D2）

```gherkin
Given sender 为 alice，正文含 "@bob @carol @bob please review"
When  derive_mentions 派生
Then  mentions == ["bob","carol"]（按出现顺序、去重）
Given sender 为 alice，正文含 "@alice please double-check"
When  derive_mentions 派生
Then  排除 sender 本人的自提及，mentions == []
Given sender 为 alice，正文含 "@#1 @bob noted"
When  derive_mentions 派生
Then  mentions == ["bob"]（`@#N` 是 reply 引用 token，不计入 mentions）
```

### POST-04 坏例：非法时间戳拒绝（spec §3.5/§9.2）

```gherkin
Given 消息头为 "## #1 alice (not-a-timestamp)"
When  parse_messages 解析
Then  返回 Parse 错误，message 提及 invalid timestamp，fix 指向 RFC 3339 格式
```

### POST-05 边界：fence 内伪造消息头不是边界（spec §3.3/§5.3）

`````gherkin
Given 消息 #1 的 ```md 围栏正文内含一行 "## #99 mallory (2026-01-01T00:00:00Z)"，
      其后是合法消息 "## #2 bob (...)"
When  parse_messages 解析
Then  消息数 == 2（#1、#2）；伪造头保留在 #1 的 body 中
And   不存在 seq 99 的消息
`````

### POST-06 边界：动态围栏 3–6 连续反引号（spec §3.4，I6；R12）

```gherkin
Given 四个 body，其最长连续反引号串长度分别为 3、4、5、6
When  serialize_message 序列化
Then  开启行反引号数依次为 4、5、6、7（= max(3, k+1)），info string 均为 "md"（D3）
And   关闭行反引号数与开启行一致
When  各自再经 parse_messages 解析
Then  规范化 body 逐字段相等（roundtrip 仅对规范化后 body 成立，spec §5.4）
补充边界：body 不含反引号时开启行为恰好 3 个反引号；
         解析侧必须接受关闭行反引号数 > 开启数的合法 CommonMark 变体
```

### POST-07 边界：sender 含空格/括号在解析侧不构成边界（spec §5.3，R1）

```gherkin
Given 文件含 H2 "## #1 two words (2026-01-15T10:30:00Z)"（sender 含空格）
      与 H2 "## #2 bob(x) (2026-01-15T10:31:00Z)"（sender 含括号）
When  parse_messages 解析
Then  两个 H2 均不匹配消息头正则（sender 由 [^\s()]+ 强制无空格无括号），归入 preamble
And   消息数 == 0（整个文件无合法消息）
And   写入侧对同形 sender 的拒绝见 POST-17（解析/写入两侧一致，R1）
```

### POST-08 空文件（spec §5.2）

```gherkin
Given 文件为空串或仅空白字符
When  parse_preamble / parse_messages 解析
Then  ThreadMeta 为缺省值（title 空），消息数 == 0，无错误
```

### POST-09 preamble-only，0 消息（spec §5.2）

```gherkin
Given 文件仅含 H1（可含其后被忽略的散文），无任何消息头
When  解析
Then  ThreadMeta 正常解析，消息数 == 0
And   （validate 语义见 VAL-04：validate 对 .post.md 拒绝零消息文件）
```

### POST-10 边界：seq gap（spec §8）

```gherkin
Given 文件含合法消息 #1 与 #3（缺 #2）
When  parse_messages 解析
Then  解析成功（宽容），得到 2 条消息
When  validate_seq_monotonicity 校验
Then  返回 Validation 错误，message 含 "gap"
Given 另一文件首条消息为 #5
When  validate_seq_monotonicity 校验
Then  返回 Validation 错误，message 含 "expected 1"
```

### POST-11 边界：断 fence（spec §3.3/§8）

`````gherkin
Given 文件的 ```markdown 围栏缺少关闭行（消息正文未闭合）
When  validate_markdown 校验
Then  报告 "unclosed" 围栏问题（含开启行号）
And   validate 命令据此拒绝该文件（VAL-03）
`````

### POST-12 CRLF（spec §3.1）

```gherkin
Given 完整合法 thread 文件以 CRLF 换行
When  解析
Then  结果与同内容 LF 版本一致，body 内不含 "\r"
```

### POST-13 Unicode 正文（spec §3.5/§3.6/§5.6，C1）

```gherkin
Given 消息 body 含 emoji、中文、重音字符；sender 为 Unicode 无空格无括号 token
When  解析
Then  body 逐字符还原；sender 按原值解析（[^\s()]+ 不排除 Unicode；无长度上限，受 64KB 隐式约束）
```

### POST-14 序列化 → 解析 roundtrip（spec §5.9；R12；D1/D2/D3）

```gherkin
Given ThreadMeta（title 非空）与若干 Message
      （覆盖：无 @ 广播、正文含 @mention、正文含 @#N、空 body、含 ``` 的 body）
When  serialize_thread(meta, messages) 后重新解析
Then  ThreadMeta 与全部 Message（body 按规范化相等）逐一相等
And   输出以 "# <title>" 开头；无 "- participants:" 行；无任何消息属性行；
      无 "---" 分隔行；无 "·"/"—"；围栏 info 均为 "md"
```

### POST-15 preamble 变体（spec §5.2，D1）

```gherkin
Given 变体一：preamble H1 后含自由散文
Then  散文被解析忽略，title == H1 文本，解析成功
Given 变体二：preamble 含额外 H2 "## Notes" 及其正文、或历史形态 "- participants: alice, bob" 属性行
Then  一律归 preamble 忽略，不影响消息解析
Given 变体三：preamble 无 H1（文件直接以消息头开始）
Then  title == ""（宽容解析），消息正常解析
```

### POST-16 thread_edit 重写保留 preamble 原文（spec §5.7）

```gherkin
Given 含 preamble（H1，可含手写散文/额外 H2）与消息 #1(alice)、#2(bob) 的线程文件
When  thread_edit 将 #2 正文改为新内容（锁内全文件重写）
Then  重写后文件的首个消息头之前的字节区间与原文件逐字节相等（原样搬运）
And   消息序仍为 #1、#2，仅 #2 body 变化
And   三重约束坏例仍拒绝：非本人消息、非本人最新、非线程末条
      均返回 NotAllowed（行为与现状一致）
```

### POST-17 写入侧 sender 字符集校验（spec §5.6）

```gherkin
Given sender 分别为 "two words"、"bob(x)"、"line\nbreak"、""
When  thread_send 发送
Then  全部返回 Validation 错误（category == "validation"），
      fix 指明 sender 必须是无空格无括号的单 token，文件不被写入
Given sender == "alice"
When  thread_send 发送
Then  成功，返回分配的 seq
```

### POST-18 单条 > 64KB 拒绝（spec §5.8，I3）

```gherkin
Given 序列化后长度超过 64KB 的消息
When  thread_send 发送
Then  返回 MessageTooLarge（category == "validation"），文件不增长
```

### POST-19 CLI：post send 新参数面（spec §5.7/§8，D1/D2）

```gherkin
Given 不存在 standup.post.md
When  paperwork post send standup --from alice --title "Daily Standup" "Hello"
Then  创建文件；preamble 仅含 H1 "Daily Standup"（无 participants 行，D1）；
      首条消息 seq == 1（无 system 消息）
When  未传 --title
Then  title 缺省算法为"剥 .post.md，否则剥 .md，否则原名"：
      参数 "standup" → "standup"；参数 "standup.post.md" → "standup"；
      参数 "notes.md" → "notes"；参数 "rawname" → "rawname"
When  paperwork post send standup --to charlie "Hi"（或传 --participants alice,bob）
Then  CLI 报未知 flag 错误（clap 默认行为；--to/--participants 已删除，D1/D2）
Given 文件已存在且非空
When  再传 --title 发送
Then  参数忽略，仅追加消息（OQ-1 默认行为）
```

### POST-20 post create 已删除（spec §5.7）

```gherkin
When  paperwork post create x --title "T"
Then  CLI 报未知子命令错误（clap 默认行为），不再存在该命令
```

### POST-21 preamble 内已闭合围栏后的消息头仍识别（spec §3.3/§5.2）

````gherkin
Given preamble 的 description 散文后含一个已闭合的 ``` 围栏块
      （如用户手写示例代码），其后是合法消息头 "## #1 alice (...)"
When  parse_messages 解析
Then  围栏开合状态正确复位，消息头被识别，消息数 == 1
And   fence 感知贯穿 preamble 扫描
````

### POST-22 preamble 未闭合围栏吞掉全部消息头（spec §3.3/§8）

`````gherkin
Given preamble 含一个只有开启行、无关闭行的 ``` 围栏，
      其后各行均为形似消息头的 "## #N sender (ts)"
When  parse_messages 解析
Then  围栏处于开启状态，其后所有头均不作边界，消息数 == 0
When  validate
Then  以 Parse 拒绝（零消息 + "unclosed" 围栏，spec §8 步骤 1/3）
`````

### POST-23 body 首尾空白行规范化（spec §5.4，R12）

```gherkin
Given 消息正文围栏内含首尾各一个（或多个）空白行的文本
When  parse_messages 解析
Then  body 为首尾空白行去除后以 "\n" 连接的结果
And   roundtrip 语义为"规范化后相等"而非逐字节还原
```

### POST-24 围栏行缩进立场（spec §3.3，R13）

````gherkin
Given 变体一：围栏开启行前导 3 空格（"   ```markdown"）
Then  识别为围栏行（≤3 空格接受，CommonMark）
Given 变体二：围栏开启行前导 4 空格（"    ```markdown"）
Then  不识别为围栏（按缩进代码块内容处理）；该消息 body 为空（围栏缺失宽容），
      其后合法消息头仍正常切分
````

### POST-25 body 围栏 info md/markdown 双接受（spec §5.4，C2/D3）

````gherkin
Given 变体一：body 围栏开启行为 ```md
      变体二：开启行为 ```markdown
      变体三：开启行为无 info 的 ```
When  parse_messages 解析
Then  三者均接受为正文围栏，body 正常提取（md 与 markdown 前缀匹配接受；
      其余 info 仍宽容接受，C2）
And   写入侧输出统一严格为 ```md（D3）
````

### POST-26 一条消息多围栏取首个（spec §5.4，C2/D2）

`````gherkin
Given 一条消息的消息头后出现两个围栏块
When  parse_messages 解析
Then  body == 首个围栏内容；第二个围栏及其内容忽略（不报错）
Given 消息头与围栏之间残留历史属性同形行（如 "- reply-to: #1"）
When  解析
Then  该行不具属性语义，解析忽略（宽容，D2）
`````

### POST-27 preamble-only 文件上 post send（spec §5.7，OQ-1/D1）

```gherkin
Given 已存在的 .post.md 仅含 preamble（H1，可含被忽略的散文），无任何消息
When  paperwork post send <file> --from alice --title "Other" "Hi"
Then  首条消息 seq == 1
And   原 preamble 原样保留（锁内 size > 0，--title 被忽略，
      preamble 不重复写、不重写）
```

### POST-28 消息头尾部垃圾 → 整文件 Parse（spec §3.5/§5.3，C1）

```gherkin
Given 消息头为 "## #1 alice (2026-01-15T10:30:00Z) (备注)"（头行尾部垃圾）
When  parse_messages 解析
Then  贪婪捕获使 timestamp == "2026-01-15T10:30:00Z) (备注"，时间戳解析失败
And   返回 Parse 错误，整个线程文件不可读（post read/summary 均失败）
```

### POST-29 thread_edit 后手写 preamble 内容保留（spec §5.7，R5）

```gherkin
Given 线程文件 preamble 含手写 description 散文与额外 H2 节 "## Notes"
When  thread_edit 修改末条消息正文
Then  重写后 description 散文与 "## Notes" 节逐字节保留（原样搬运，非 title/participants 投影）
```

### POST-30 thread_edit 新 body 超 64KB 拒绝（spec §5.8，R8/I3）

```gherkin
Given 线程末条消息为本人最新消息
When  thread_edit 将其 body 改为序列化后 > 64KB 的内容
Then  返回 MessageTooLarge（category == "validation"），文件内容完全不变
```

### POST-31 post read --plain 子集输出无 preamble（spec §5.9，C7）

```gherkin
Given 含 preamble 与消息 #1..#3 的线程文件
When  paperwork post read <file> --plain（或带 --from/--to 区间过滤）
Then  输出为 serialize_messages 的纯消息序列化，不含 preamble（无 H1 行）
And   该子集输出再解析得到 title == ""（宽容解析的合理行为）
```

### POST-32 尾扫缓冲区截断边界（spec §5.5，R7/F15）

```gherkin
Given 文件 > 64KB+256B，尾扫缓冲区起点落在某行中间
When  read_last_seq_locked 读取 last_seq
Then  首个不完整行被丢弃（前一字节非 \n 时截到第一个 \n 之后），
      其后末条消息头 seq 正确读出
Given 缓冲区起点恰好落在行首（前一字节为 \n）
Then  该行完整保留，不被误丢
Given 文件 ≤ 缓冲区大小（read_start == 0）且首行即消息头（无 preamble）
Then  不丢弃任何行，last_seq 正确（下次 send 的 seq 不重复）
```

### POST-33 @#N reply-to 派生（spec §5.4，D2）

```gherkin
Given 正文含 "@#1 got it"
When  derive_reply_to 派生
Then  reply-to == Some(1)
Given 正文含 "@#2 first" 后又含 "@#3 second"（多个 @#N）
When  derive_reply_to 派生
Then  首个合法引用生效：reply-to == Some(2)，其余忽略
Given 正文含 "@#999" 且线程中不存在 #999 消息
When  derive_reply_to 派生
Then  reply-to == Some(999)（不校验引用目标是否存在，宽容）
```

### POST-34 正文含 @ 但非合法 token（spec §5.4，D2）

```gherkin
Given 正文含孤立 "@"（后随空白或行尾）、"@)" 等不构成合法 token 的形态
When  derive_mentions / derive_reply_to 派生
Then  均不产生任何派生结果（mentions == []，reply-to == None）
And   解析不报错（宽容）
```

### POST-35 派生结果不落盘（spec §5.4/§5.9，D2）

```gherkin
Given 任意消息（无论正文是否含 @ 引用）
When  serialize_message 序列化
Then  输出仅含消息头行与 ```md 围栏正文，无任何派生 mentions/reply-to 行或字段
And   派生仅发生在读取/统计路径（summary、read 过滤等），实时从正文文本计算
```

### POST-36 无尾换行文件追加修复（spec §5.7/§5.9，I4；终审 review F1）

````gherkin
Given 合法线程文件的最后一个字节不是 "\n"（如末行为闭合围栏 ```，被外部编辑/管道截断尾换行）
When  thread_send 追加新消息
Then  追加前锁内探测文件末字节，非 "\n" 时在 payload 前补 "\n"
And   新消息头落在独立行（不与围栏行黏连），读回得到新旧全部消息且 body 完整
Given 正常文件（末字节已是 "\n"）
When  thread_send 追加
Then  不注入任何额外空行，序列化形态与历史行为一致
````

## 3. brief（BRIEF-*）

### BRIEF-01 正常路径：完整条目（spec §6.1/§6.2）

```gherkin
Given 文件含 H1、description 散文段、"- owner:"、"- created:" 与
      条目 "## main.rs"（- path: src/main.rs、- hash: <64位hex>、
      - regex: fn main、散文 note "Entry point"）
When  parse_manifest 解析
Then  name/author/created/description 正确
And   条目 title == "main.rs"，path/hash/regex/note 均按裸文本解析（无反引号剥除）
```

### BRIEF-02 无 regex = 省略该行（spec §6.2）

```gherkin
Given 条目无 "- regex:" 行
When  解析
Then  entry.regex == None，groups 为空
And   序列化 regex == None 的条目不输出 "- regex:" 行、不输出 "—"
```

### BRIEF-03 复杂 regex 用 ```regex 围栏（spec §6.2）

````gherkin
Given 条目以 ```regex 围栏块给出含换行与反引号的模式
When  解析
Then  entry.regex == 围栏内原文（含换行），groups 由命名捕获组派生
When  序列化含 "\n" 或 "`" 的 regex
Then  输出采用 ```regex 围栏块形式而非内联
````

### BRIEF-04 hash 全量不截断（spec §6.2）

```gherkin
Given brief add 对目标文件计算 SHA-256
When  序列化条目
Then  "- hash:" 值为完整 64 位小写 hex（无截断）
And   解析后 entry.hash 与计算值逐字符相等
```

### BRIEF-05 groups 派生不落盘（spec §6.2）

```gherkin
Given regex 含命名捕获组 (?<year>...) 与 (?<month>...)
When  解析
Then  groups == ["year","month"]
And   序列化输出不含任何 groups 字段/行
```

### BRIEF-06 坏例：缺必需属性拒绝（spec §6.5）

```gherkin
Given 文件缺 "- owner:" 行（或缺 "- created:" 行、或 created 值非 RFC 3339）
When  parse_manifest 解析
Then  返回 Parse 错误，fix 分别为小写键文案（spec §9.2）
Given 文件缺 H1
Then  返回 Parse 错误 "missing title heading"
```

### BRIEF-07 note 散文段（spec §6.2）

```gherkin
Given 条目属性行之后为多行散文段（无 ">" 前缀）
When  解析
Then  entry.note 为该散文段文本（多行保留）
And   序列化 note 输出为裸散文段，不含 blockquote ">" 前缀（R15：note 为文档元叙述裸散文）
```

### BRIEF-08 verify 三态不变（spec §6.4）

```gherkin
Given 条目目标文件未变 / 内容改变但 regex 仍匹配 / regex 不再匹配（或文件缺失）
When  brief_verify
Then  分别返回 Fresh / Shifted / Stale（与现状行为完全一致）
```

### BRIEF-09 hash 换行敏感性（技术债 #5 文档化，spec §6.4）

```gherkin
Given 目标文件仅换行符由 LF 变为 CRLF（其余字节不变）
When  brief_verify
Then  结果为 Shifted（字节级 hash 差异）——此为已文档化的预期行为，不修复
```

### BRIEF-10 CRLF 与 Unicode（spec §3.1/§3.6）

```gherkin
Given brief 文件以 CRLF 换行；title/note 含 Unicode
When  解析
Then  与 LF 版本结果一致；Unicode 原值保留
```

### BRIEF-11 序列化 roundtrip（spec §6.3）

```gherkin
Given 任意合法 Manifest（含与不含 description/note/regex、简单与复杂 regex 各覆盖）
When  serialize_manifest 后 parse_manifest
Then  全字段逐一相等
And   输出无 "## Entries" 节、无 "—" 占位、无大写键
```

### BRIEF-12 条目属性区终止边界（spec §3.2/§6.2，R4）

```gherkin
Given 条目 H2 之后依次为 "- path: a"、空行、"- hash: b"、
      首个非属性散文行 "Note starts"、其后又一属性同形行 "- path: c"
When  parse_manifest 解析
Then  属性区延伸至首个非属性非空行：path == "a"、hash == "b"（空行不终止属性区）
And   note == 自 "Note starts" 起的散文（含其后的 "- path: c" 行原文）
And   note 中的同形行归 note，不覆盖 path 属性
```

## 4. contacts（CONT-*）

### CONT-01 正常路径：链接条目（spec §7.2）

```gherkin
Given 文件含 H1 "# Core Team" 与链接 bullets
      "- [alice](agents/alice.profile.md)"、"- [bob](agents/bob.profile.md)"
When  parse_contacts 解析
Then  得到 2 个条目，label 与 profile_path 分别为 alice/bob 与两条路径
And   parse_contacts_title 返回 "Core Team"
```

### CONT-02 尖括号形式解析（spec §7.2）

```gherkin
Given 条目为 "- [alice](<agents/my profile.md>)"
When  parse_contacts 解析
Then  profile_path == "agents/my profile.md"（尖括号剥离，空格保留）
```

### CONT-03 Windows 带空格路径序列化（spec §7.3，R11）

```gherkin
Given 条目路径为 "C:\team docs\alice.profile.md"（含空格），
      目标 profile 的 H1 为 "alice"（或读取失败回退主干 "alice"：
      先剥 .profile.md 再剥 .md）
When  serialize_contacts 序列化
Then  该行输出为 "- [alice](<C:\team docs\alice.profile.md>)" 形式
      （destination 采用尖括号；label 按 R11 取目标 profile H1，回退文件名主干）
And   输出再解析后 profile_path 与原路径相等
```

### CONT-04 roundtrip 含括号路径（spec §7.3）

```gherkin
Given 路径含 "(" 与 ")" 字符
When  serialize_contacts 后 parse_contacts
Then  自动采用尖括号形式；roundtrip 相等
Given 路径不含空格/tab/括号/尖括号
When  序列化
Then  采用裸形式 "[label](path)"
```

### CONT-05 坏例：缺 H1 拒绝（spec §7.4）

```gherkin
Given 文件无任何 H1
When  parse_contacts_title 解析
Then  返回 Parse 错误 "missing contacts title heading"
```

### CONT-06 裸路径 bullet 不再识别（spec §7.2/§3.6）

```gherkin
Given 文件含旧格式裸路径 bullet "- ./agents/alice.profile.md"
When  parse_contacts 解析
Then  该行被忽略，不计入条目（hard breaking）
```

### CONT-07 Unicode（spec §3.6）

```gherkin
Given 标题与路径含 Unicode（如 "# équipe"、"agents/alicé.profile.md"）
When  解析与序列化
Then  原值保留，roundtrip 相等
```

### CONT-08 尖括号与反斜杠转义 roundtrip（spec §7.2/§7.3，C3）

```gherkin
Given 路径含 "<" 或 ">" 字符
When  serialize_contacts 序列化
Then  destination 采用尖括号形式，其中 "<"/">" 转义为 "\<"/"\>"
When  parse_contacts 解析
Then  反转义后 profile_path 与原路径相等（roundtrip）
Given label 含 "]" 字符（防御性用例）
When  序列化后再解析
Then  序列化为 "\]"，解析侧反转义，label roundtrip 相等
Given 条目为 "[alice](agents/a.md \"title\")"（带 title 形式）
When  解析
Then  title 被忽略，destination 正常提取（宽容，title 语法不接受）
```

## 5. 并发（CONC-*）

### CONC-01 两写者竞争追加（spec §5.8，I1/I2/I4）

```gherkin
Given 一个已含 preamble 的空线程文件
When  两个线程各自 thread_send N 条消息（沿用现有并发用例结构）
Then  共 2N 条消息全部落盘
And   seq 集合恰为 1..=2N，无重复无 gap（validate_seq_monotonicity 通过）
And   每条消息 body 完整无损（无交错写）
```

### CONC-02 两写者竞争首写（spec §5.7，I9）

```gherkin
Given 一个不存在的线程文件路径
When  两个线程并发执行 thread_send（各自携带 title 首写参数）
Then  preamble 在文件中恰好出现一次（H1 唯一）
And   首条消息 seq == 1，共 2 条消息 seq == {1, 2}
And   后到写者锁内发现 size > 0，只追加消息、不重复写 preamble
```

### CONC-03 尾扫 fence 感知与残留限制（spec §5.5，R6）

`````gherkin
Given 线程末条消息的 ```markdown 正文（已闭合围栏）内含伪造头
      "## #99 mallory (2026-01-01T00:00:00Z)"，且该区间落入尾扫缓冲区
When  执行下一次 thread_send
Then  缓冲区内围栏开合追踪跳过开启围栏内部的候选头，
      分配 seq == 真实末条 seq + 1（伪造头不污染 seq）
And   残留限制声明：缓冲起点之前的围栏奇偶状态不可知（起点切断围栏的构造下
      候选头仍可能污染 seq），由 validate seq 连续性校验兜底暴露
`````

### CONC-04 首写崩溃遗留 0 字节文件的恢复（spec §5.7，I9）

```gherkin
Given 首写者创建文件后崩溃，遗留 0 字节 .post.md
When  下一写者 thread_send（携带 title 参数）
Then  锁内 size == 0 判定触发补写 preamble，preamble 仍恰好一次
And   首条消息 seq == 1
```

## 6. validate 语义（VAL-*）

### VAL-01 validate post 全通过（spec §8）

```gherkin
Given 合法 .post.md（preamble + seq 1..N 连续 + 全部围栏闭合）
When  paperwork validate <path>
Then  输出 "ok validate <path>"，exit 0
```

### VAL-02 validate 拒绝 seq 问题（spec §8/§9.1，R10）

```gherkin
Given .post.md 含 seq gap（或缺 #1、首条为 #2）
When  validate
Then  输出 error 信封（category validation——直接透出 Validation 变体，
      不再重包为 Parse），detail 指向 sequence 问题，exit 1
```

### VAL-03 validate 拒绝断 fence（spec §8）

```gherkin
Given .post.md / .brief.md 存在未闭合围栏
When  validate
Then  输出 error 信封（category format），detail 含 "unclosed" 与开启行号，exit 1
```

### VAL-04 validate 拒绝零消息/垃圾文件（spec §8）

```gherkin
Given 非空 .post.md 无任何合法消息头（含 "garbage" 文本与 v0.4 旧格式文件）
When  validate
Then  输出 error 信封，fix 指向新消息头文法与动态围栏，exit 1
```

### VAL-05 其余格式 validate（spec §8）

```gherkin
Given 合法的 .profile.md / .brief.md / .contacts.md 各一份
When  validate
Then  全部 ok（parser 成功 + 围栏闭合）
Given 缺必需字段的对应坏例各一份
Then  全部 error 信封，exit 1
```

### VAL-06 未知后缀拒绝（spec §8）

```gherkin
Given 路径后缀不属于四种 managed 类型
When  validate
Then  Parse 错误，fix 列出四种合法后缀，exit 1
```

### VAL-07 空文件 validate 拒绝（spec §8，行为变更）

```gherkin
Given 空的 .post.md（0 字节或仅空白）
When  validate
Then  输出 error 信封（Parse，零消息拒绝），exit 1
And   注：v0.4 现状豁免空文件，此为 spec §8 明示的行为变更
```

### VAL-08 疑似消息头启发式 warning（spec §8 步骤 4，R9）

```gherkin
Given .post.md 含 1 条合法消息，另有一行 "## #2 alice (2026-01-15T10:31:00Z"
      （形似 "## #<数字>" 但不严格匹配消息头文法，且不在围栏内）
When  validate
Then  validate 结论仍为 ok（合法消息存在、seq 连续、围栏闭合）
And   输出含针对该行的 warning 与 fix（expected format: ## #<seq> <sender> (<timestamp>)）
```

---

**场景总计**：PROF 11、POST 36、BRIEF 12、CONT 8、CONC 4、VAL 8，共 79 个场景。tdd.md 必须覆盖全部场景（允许一个测试函数覆盖多个同族场景，但映射表不得遗漏任何编号）。
