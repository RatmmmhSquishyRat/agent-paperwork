# 深审 B-2：健壮性边界与并发增量复审（任务 #43）

- 日期：2026-08-15
- 取证基线：master @ 46b1f47（本地复跑 `cargo test --workspace --locked` = **444 通过 / 0 失败**，与任务口径一致）
- 被测对象：`target/release/paperwork.exe`（0.5.0，`cargo build --release --locked`）
- 环境：Windows 25H2，PowerShell 7；全部夹具置于 `%TEMP%\r2audit`（含子目录 `conc2`/`adv`），结束后整体清理；仓库内除本报告外零落盘、无 git 操作（探针期间曾误落 CON.post.md/NUL.post.md 于仓库根，已即时删除并核验 git status 复原）
- 前序基线：docs/dev/audit-robustness-2026-08-15.md（深审 B，D1~D7）+ docs/dev/fix-ledger-2026-08-15.md（修复波、评审闭环 C-1 护栏批、CI 事件账目）
- 判定口径：panic / 挂起 / 静默数据损坏 = 阻塞级；信封结构、category 与退出码（0/1/2）以 docs/ssot/specs/cli-grammar-v0.6/spec.md 为准；并发断言用条目集合/计数比较；探针一律 TEMP 夹具
- 实验总数：**61**（T×19、P×11、E×5、C×6、N×8、A×9、G×3）；**通过 58，低危缺陷 1（R2-01，3 组探针命中）**；另登记已知限制 3、行为未钉住点 8

## 1. 结论

裁决+回填后的增量复审总体结论：**前序 D1~D7 与评审闭环 C-1 的全部修复经受住了本轮 61 组新探针复测，未发现任何阻塞级/重要级缺陷**。核心契约——统一错误信封、fast fail、fence 平衡预检零写入、注入护栏零写入、并发零丢失、幂等零写入——全部成立。**0 panic、0 挂起、0 静默损坏、0 并发丢失**。

### 缺陷清单（速览）

| 编号 | 一句话 | 严重度 |
|---|---|---|
| R2-01 | 文件面非 UTF-8 / UTF-16 读取的 io 信封 fix 文案不指向编码（"check that the file is readable"）——D6 修复只覆盖了 stdin 通道，文件通道留下同源文案缺口 | **低** |

已知限制（非缺陷，登记）：

- **L-R2-1**：Windows `attrib +r` 只读目录属性不阻止文件创建（平台语义，与前序 L2 一致，P-04 复测确认；真·拒绝写入需 ACL deny，前序 S-08 已验证信封正确）。
- **L-R2-2**：争用语义取决于持锁者的锁形态——Win32 `LockFileEx` advisory 锁 → paperwork 阻塞等待、释放后完成（前序 C-08）；`FileShare.None` 独占打开 → paperwork 打开即被拒，**立即** io 信封 exit 1（os error 32），无重试，释放后恢复正常（C-06 实测）。两种形态均无损坏、无挂起。
- **L-R2-3**：io 信封内嵌 Windows 本地化 OS 错误文本（GBK 字节）——按任务 #25 根因裁定与 LED-16 豁免保留（P-08 os error 123、P-11 os error 5 复现）。


最严重三项（本轮无阻塞/重要级，按相对严重度排序）：

1. **R2-01（低，唯一产品缺陷）**：文件通道编码错误文案缺口（T-11/T-12/E-02 三处复现），与前序 D6 同源不同面。
2. **大文件面（容量风险，非既成缺陷）**：10MB+ 合法线程的 send 依赖锁内整文件 fence 预检（fix-ledger I-3 已登记为已知权衡），本轮实测 10.1MB / 2500 消息 read 85ms、send 47ms 无异常，但该量级**无任何测试钉住**（盲区 B-6），未来回归无防线。
3. **编码容忍面无测试钉住（盲区 B-1/B-2）**：BOM 容忍、UTF-16 fast-fail、组合字符不规整化均实测正常，但 444 测试中零覆盖，行为随时可能无声漂移。

---

## 2. 实验面 1：畸形/恶意输入（T-01 ~ T-19）

| 编号 | 用例 | 预期 | 实测 | 判定 |
|---|---|---|---|---|
| T-01 | 未闭合 fence 线程分别 `post send` 与 `post edit`（D2 回归） | 两侧 fast-fail Parse 信封、零写入 | 均 exit 1 `unclosed code fence (3 backticks) opened at line 5` + fix 声明「文件未被改动」；前后文件哈希一致 | **通过**（D2 修复双侧成立） |
| T-02 | 正文内嵌三反引号块发送，检查动态 fence | 写侧 fence 加长至 4 反引号、roundtrip 无损 | 文件落 ` ````md ` 四反引号 fence，read --json body 逐字含内层 ``` 块 | **通过** |
| T-03 | 正文含 10 连反引号 run | fence = max(3, run+1) = 11 | 落盘 11 反引号 fence（`Select-String` 确认），body 原样 | **通过** |
| T-04 | 手写闭合 fence 长于开 fence（3 开 4 闭） | CommonMark 语义：≥ 开长即可闭合 | read/validate 均 exit 0，正常解析为 1 消息 | **通过**（宽容读侧既定语义） |
| T-05 | 缺 H1 头线程（直接以消息头开始） | 宽容读或清晰信封 | read exit 0 正常解析、validate exit 0（H1 在读侧非强制） | **通过**（宽容面见盲区 B-8） |
| T-06 | 双 H1 头线程 | 宽容读 | read/validate exit 0，解析正常 | **通过**（同属盲区 B-8） |
| T-07 | 消息头时间戳非 RFC3339（`not-a-timestamp`） | format 信封 | exit 1 `invalid timestamp ... cannot parse ... as RFC 3339`，fix/example 齐备 | **通过** |
| T-08 | 消息头缺 seq（`## alice (ts)`） | format 信封 | exit 1 `no valid message boundaries found` + fix 教学 H1+`## #N` 形态 | **通过** |
| T-09 | 空文件（0 字节）×5 命令：post read / profile show / brief read / contacts read / validate | 各自既定语义（既有测试 S-READ-02 / VAL-07 / M2 钉住） | post read exit 0 `0/0`；contacts read exit 0 空；profile/brief/validate 均 format exit 1 信封正确 | **通过**（不对称语义已被测试钉住） |
| T-10 | 仅空白文件（空格/Tab/换行） | 同空文件口径 | 与 T-09 完全一致（read 宽容、validate format） | **通过** |
| T-11 | 4KB 随机二进制文件（含 0xFF/NUL 字节）read + validate | io fast-fail、不 panic | 两者均 exit 1 io 信封 `stream did not contain valid UTF-8`，无 panic；**fix 文案不指编码 → R2-01** | **缺陷 R2-01** |
| T-12 | 10MB 纯随机二进制 read | 同上、且成本受控 | exit 1 io 信封，95ms 内返回（UTF-8 校验早停） | **命中 R2-01**（文案缺口） |
| T-13 | 10.1MB / 2500 消息合法线程 read + send | 正确性与成本俱佳 | read 85ms（默认窗口 20/2500）；send exit 0 seq=2501、47ms（含锁内整文件 fence 预检） | **通过**（容量面无测试钉住，盲区 B-6） |
| T-14 | CRLF 全文线程 read --json | 归一为 LF 解析（R12） | body=`x`，与 LF 夹具逐字等价 | **通过** |
| T-15 | UTF-8 BOM 前缀线程 read --json | 容忍 BOM | 解析与无 BOM 等价，exit 0 | **通过**（0 测试钉住，盲区 B-1） |
| T-16 | CRLF + lone-CR 混排行边界线程 | 归一解析不炸 | read/validate exit 0，消息正确解析（R7 边界已修复，ec59c01 回归面复现通过） | **通过** |
| T-17 | profile 内注入伪属性行 `- fakekey: injected`（手写文件） | 读侧忽略未知属性或清晰信封 | show exit 0，仅回显 name/model/scope，伪 key 被静默忽略，不污染输出 | **通过**（宽容读侧既定语义） |
| T-18 | 200KB 超长 `- model:` 属性值 show | 可处理、不截断损坏 | exit 0，值原样回显（200000 字符），无成本异常 | **通过** |
| T-19 | 属性值含控制字符（BEL \u0007） show --json | JSON 必须合法 | exit 0，JSON 中控制字符正确转义为 `\u0007`，信封结构合法 | **通过** |

畸形输入面结论：**19 组探针：17 通过 + 2 组命中 R2-01（T-11/T-12），另 E-02 同属该缺陷**。D2 修复（fence 平衡预检）在 send/edit 两侧复测成立且哈希证明零写入；动态 fence 边界（10 连反引号 → 11 长度）按 max(3, run+1) 正确生成。

## 3. 实验面 2：路径与文件名（P-01 ~ P-11）

| 编号 | 用例 | 预期 | 实测 | 判定 |
|---|---|---|---|---|
| P-01 | 10 层深嵌套目录 send + read | 自动建父目录、正常 | exit 0，seq 1，读回一致 | **通过** |
| P-02 | 300+ 字符长路径（两级 150 字符目录名） | 长路径支持 | send/read exit 0 正常 | **通过** |
| P-03 | 非 ASCII + emoji 文件名（中文🚀テスト.post.md） send/read | 落盘无损 | 文件正常创建读回；字节级核验文件名 UTF-8 无损（信封显示中的 `?` 为终端捕获伪影，磁盘文件名与文件内容字节核验均完好） | **通过** |
| P-04 | `attrib +r` 只读目录 send | 平台语义 | exit 0 正常创建（Windows 只读属性不阻止创建，已知限制 L-R2-1） | 已知限制 |
| P-05 | junction 目录写 + 真实目录读回；硬链接文件读/写/原文件核验 | 链接透明 | junction 写入后经真实路径读回一致；硬链接写入 seq 2 后原路径可见两消息 | **通过**（符号链接需管理员权限，本机未启用，未测——登记） |
| P-06 | 不存在的多级父目录 send | 自动创建父目录（contacts_create 同语义，代码注释既定） | exit 0，`no\such\dir\p.post.md` 创建成功 | **通过** |
| P-07 | 保留设备名 CON / NUL 作为 send PATH（带 6s kill 保护） | 不挂起、不写入设备 | 后缀归一化生效：CON → CON.post.md、NUL → NUL.post.md 普通文件，exit 0，无设备劫持、无挂起 | **通过**（护栏为后缀归一化的附带效果，无专项测试，盲区 B-5） |
| P-08 | 裸保留名 CON / NUL 作为 validate PATH | 清晰信封、不打开设备 | exit 1 format `unknown file type: CON/NUL`，fix 指向后缀/--type；未触碰设备、无挂起 | **通过** |
| P-09 | 路径含空格/单引号/双引号/反引号/百分号 send + read | OS 语义拒绝时信封正确 | 反引号使 OS 判非法：send exit 1 io（os error 123，GBK 文本属 L-R2-3）；read exit 1 not-found 信封结构正确、无 panic | **通过** |
| P-10 | PATH 为无后缀名（adir），post send / contacts add | 后缀归一化（ensure_suffix） | send → adir.post.md 正常；contacts add → 找 adir.contacts.md 得 not-found 信封，fix 引用真实存在的 `contacts create` 动词（已核验 help） | **通过** |
| P-11 | PATH 恰为已存在目录且带 .post.md 后缀（dir.post.md 是目录） send | io 信封 | exit 1 io（os error 5 access denied），fix「check that the file path is accessible」，无 panic | **通过** |

路径面结论：**11 组探针全通过（P-04 记为已知限制）**。保留设备名攻击面被后缀归一化（ensure_suffix，有专项测试）事实上封死：读写命令的 PATH 永远带格式后缀，CON/NUL 只能落成普通文件；validate 的裸名路径在类型推断阶段即 fast-fail，不打开设备。

## 4. 实验面 3：编码面（E-01 ~ E-05）

| 编号 | 用例 | 预期 | 实测 | 判定 |
|---|---|---|---|---|
| E-01 | 非 UTF-8 字节（0xC0 0x80）经 stdin --stdin 灌入（D6 回归） | validation 信封、fix 指编码 | exit 1 `{"category":"validation",...,"message":"Validation error: stdin is not valid UTF-8","fix":"check that the piped content is valid UTF-8 text..."}`，结构完整 | **通过**（D6 修复成立） |
| E-02 | UTF-16 LE（FF FE BOM）线程文件 read | fast-fail、不 panic、不乱码透传 | exit 1 io `stream did not contain valid UTF-8`；**fix 不指编码 → R2-01** | **缺陷 R2-01** |
| E-03 | emoji sender（🚀bot）send + read | sender 校验仅禁空白/括号，emoji 放行 | exit 0，落盘无损（显示 `?` 为终端伪影） | **通过**（无测试钉住，盲区 B-3） |
| E-04 | 组合字符 sender（e + U+0301）send，正文 @mention 自身 | 落盘不归一化；sender 自引用不入 mentions | 字节级核验：文件与 --json 原始输出字节均保留 U+0301（未 NFC 规整）；mentions 正确排除 sender 自身、保留 @alice | **通过**（盲区 B-3） |
| E-05 | 中文+emoji 文件名 entry 的 brief add，hash 核验 | hash = 文件字节 SHA256 | brief 落盘 UTF-8 无损（字节核验 E6 95 B0...）；hash 与 Get-FileHash SHA256 逐字一致（acfe1c30...） | **通过** |

编码面注记：argv 通道（--author/--message/文件路径）在 Windows 上经 UTF-16 传入，**无法构造非 UTF-8 字节**（平台面不可达，与前序 R-01 口径一致）；非 UTF-8 文件内容/文件名的攻击面已由 E-02/T-11/T-12 与 P-03 覆盖。

## 5. 实验面 4：并发与锁（C-01 ~ C-06）

全部并发探针以 `Start-Job` 多进程方式发起（每 job 独立 paperwork 进程），断言采用计数/集合比较。

| 编号 | 场景（并发度） | 预期 | 实测 | 判定 |
|---|---|---|---|---|
| C-01 | `post send` ×16 同一新线程 | 零丢失、seq 无缺号重号 | 16/16 exit 0；read 回 16 条，seq=1..16 连续 | **通过** |
| C-02 | `brief add` ×12 同一 brief（预建 12 个 entry 目标文件） | 零丢失 | 12/12 exit 0；entries=12 | **通过** |
| C-03 | `contacts add` ×12 同一 contacts（预建 12 个 profile） | 零丢失 | 12/12 exit 0；contacts=12 | **通过** |
| C-04 | 交错压测：`post send` ×8 与 `contacts add`（缺失目标，advisory 路径）×8 同时发起 | 互不干扰、各自零丢失 | send 8/8 exit 0 且 msgs=8；advisory 8/8 exit 0（advisory 探测与并发写交错无死锁、无阻塞放大） | **通过** |
| C-05 | 8 路并发 send 至**未闭合 fence** 线程 | 全部 fast-fail、零写入 | 8/8 exit 1（fence 预检信封）；前后文件哈希一致 | **通过**（D2 护栏在并发下成立） |
| C-06 | 外部 `FileShare.None` 独占持锁 5s，期间 send | 快速失败（与 LockFileEx 阻塞语义不同的已知限制 L-R2-2），释放后恢复 | 持锁期间 exit 1 io（os error 32）18ms 快速失败、零写入；释放后 send exit 0 正常落盘 | **通过**（语义登记为 L-R2-2） |

并发面结论：**6/6 通过，48+ 并发进程零丢失、零损坏、零死锁**。锁等待无超时配置（阻塞至释放，spec §3.9 既定）；本轮未观察到死锁或无限挂起。

## 6. 实验面 5：错误信封分类（N-01 ~ N-08）

| 编号 | 失败场景 | 预期 category / exit | 实测 | 判定 |
|---|---|---|---|---|
| N-01 | read 不存在线程（--json） | not-found / 1 | `{"category":"not-found","command":"post.read","exit_code":1,...}`，fix 教学先 send 建线程 | **通过** |
| N-02 | sender 含空格（--json） | validation / 1 | `{"category":"validation",...,"fix":"sender must be a single token without spaces or parentheses"}` | **通过** |
| N-03 | 坏时间戳线程 read（--json） | format / 1 | `{"category":"format","exit_code":1,...}`，message/fix/example 齐备 | **通过** |
| N-04 | 二进制文件 read（--json） | io / 1 | `{"category":"io","exit_code":1,...,"message":"...stream did not contain valid UTF-8"}` | **通过**（fix 文案缺口见 R2-01） |
| N-05 | --message 与 --stdin 同给 | usage / 2 | exit 2 usage 信封，冲突说明正确 | **通过** |
| N-06 | 缺必填 flag（--json） | usage / 2 的 JSON 信封 | `{"category":"usage","command":"post.send","exit_code":2,...}`——usage 错误在 --json 下同样产出结构化 JSON | **通过** |
| N-07 | `RUST_BACKTRACE=full` 环境下触发 format 错误 | 不泄露栈 | 输出无任何 `stack backtrace`/`panicked` 字样，仅信封 | **通过** |
| N-08 | 未知子命令 | usage / 2 + did-you-mean | exit 2 `unrecognized subcommand`，提示正确 | **通过** |

信封面结论：**8/8 通过**。七类 category 冻结枚举（format/validation/io/not-found/already-exists/not-allowed/usage）与退出码映射全部正确；错误路径不泄露 Rust 栈信息；--json 下错误信封结构完整（category/command/exit_code/fix/message/status）。

## 7. 实验面 6：contacts advisory 边界（A-01 ~ A-09）

O2 裁决面（14f3b57）复测：advisory 非阻塞、ok 信封附带 advisory 字段。

| 编号 | destination 形态 | 预期 | 实测 | 判定 |
|---|---|---|---|---|
| A-01 | 目录 | advisory 非阻塞 | exit 0，`"advisory":"destination 'adir' is not readable"`，写入照常（43ms） | **通过** |
| A-02 | 空文件（0 字节，.profile.md） | advisory | exit 0，`not a valid profile file`（26ms） | **通过** |
| A-03 | 非 profile 的 md（post 形态文件） | advisory | exit 0，`not a valid profile file`（24ms） | **通过** |
| A-04 | 10MB 随机二进制（伪 profile） | 探测成本受控 | exit 0，`not readable`（UTF-8 校验早停，73ms） | **通过** |
| A-05 | update：OLD 未命中 | not-found / 1 | exit 1 not-found 信封，fix 教学键口径（profile path as stored） | **通过** |
| A-06 | update：正常 OLD→NEW，NEW 文件存在但非 profile | 成功 + NEW advisory | exit 0，`updated: old.profile.md -> new.profile.md` + advisory `does not exist`...（注：A-06 实测 advisory 文案为 NEW 探测结果，非阻塞） | **通过** |
| A-07 | update：NEW 已存在于清单 | already-exists / 1 | exit 1 already-exists，fix 引导先 remove（S-CONTACTS-09 判例复现） | **通过** |
| A-08 | update：OLD==NEW 自回绑（OLD 命中） | 裁定语义 AlreadyExists | exit 1 already-exists——与 ops/contacts.rs 裁决注释及 ops_contacts_crud_tests（OLD==NEW hit/miss 双测）逐字一致 | **通过**（已钉住） |
| A-09 | 10MB 合法形态但缺 model 的 profile | 探测全量读取的成本 | exit 0 advisory `not a valid profile file`，**123ms**（全量读 10MB 后判定） | **通过**（成本可接受；盲区 B-7） |

advisory 面结论：**9/9 通过**。探测成本画像：不可读/编码失败早停 24~73ms；需全量解析判定的 ≤123ms（10MB）——对 agent 交互无感知压力。advisory 与并发写交错（C-04）无死锁。

## 8. 注入回归面（G-01 ~ G-03，前序 D1/D3/C-1 修复复测）

| 编号 | 攻击向量 | 实测 | 判定 |
|---|---|---|---|
| G-01 | `post send --title` 带换行 + 伪造 `## #99 mallory` 消息头（D1 原始向量 R-17） | exit 1 validation `thread title contains a line break; single-line fields cannot span multiple lines`；**目标文件不存在（零写入）** | **修复成立** |
| G-02 | `profile create --description=` 粘连注入 `## Scope` + `- read: /etc/**`（D3/D4 向量 S-04/S-05） | exit 1 validation `prose embeds a heading-shaped line ...`；零写入 | **修复成立** |
| G-03 | `brief add --note` 带换行 + fence 外 `### forged`（C-1/I-1 向量 P1/P3） | exit 1 validation `note embeds a heading-shaped line ... outside a code fence`；brief read 保持 exit 0 可读（无 lockout、无静默分裂） | **修复成立** |

注入面结论：前序全部写侧护栏（NEW-1 单行/dangerous-attribute 批、D3/D4 prose 与 scope 净化、C-1 note fence-aware 护栏）在高对抗复测下全部零写入拒绝，且既有合法内容不受影响面未回归。

## 9. R2-01 细节（低危，本轮唯一产品缺陷）

- 症状：文件通道的非 UTF-8 内容（随机二进制 / UTF-16 编码文件）经任何读命令（post read / validate 等）触发 `stream did not contain valid UTF-8` 时，io 信封的 fix 固定为「check that the file is readable」/「check that the file exists and is readable」，与真实原因（编码）无关。
- 证据：T-11（4KB 二进制，read+validate 双信封）、T-12（10MB 二进制）、E-02（UTF-16 LE 文件）三处复现，逐字一致。
- 根因：前序 D6 修复（b107771）只在 cmd/post.rs resolve_body 为 **stdin 通道**区分了 InvalidData → validation 信封；文件读取路径（core 层 read_to_string 冒泡的 io::Error kind=InvalidData）仍走通用 IoContext 兜底 fix。两通道同源（UTF-8 解码失败），处置不一致。
- 修复方向（建议）：文件读取面识别 InvalidData kind 时，fix 文案改指编码（如 check that the file is UTF-8 encoded），category 可维持 io（与前序任务 #25 根因裁定的 io 语义一致）；或统一升级为 validation，与 stdin 通道对齐（口径需 owner/评审裁定）。
- 影响评估：无数据风险（fast-fail 零写入正确），纯 agent UX 文案面——agent 按「权限/存在性」方向自查会多走一步弯路。

## 10. 测试盲区盘点（对照 444 测试清单）

方法：`cargo test --workspace --locked -- --list` 全量导出 444 测试名，按本轮实验面关键词（fence/bom/crlf/utf16/unicode/emoji/combining/junction/symlink/hardlink/reserved/empty/advisory/concurrent/large/binary 等）逐一比对。既有覆盖良好的面：fence（34 项，含 D2 双侧回归）、CRLF（8 项，含 Ivy G5 roundtrip）、并发（11 项，含 multiprocess 双套件与 Ivy G5 contention）、advisory（4 项）、注入零写入（多项 `_refused_zero_write`）、空文件语义（S-READ-02/VAL-07/M2 钉住）、后缀归一化（ensure_suffix 3 项 + invalid_utf8 OsStr 4 项）、Ivy G1~G5 信封结构面（16 项全绿）。

**无测试覆盖且产品行为未钉住的点（B-1 ~ B-8，本轮均实测正常，但回归无防线）：**

| 编号 | 未钉住行为 | 本轮实测状态 | 建议 |
|---|---|---|---|
| B-1 | UTF-8 BOM 前缀文件容忍读取 | 正常（T-15） | 加 read/validate roundtrip 测试钉住「BOM 容忍」 |
| B-2 | UTF-16 文件 → io 信封（含 R2-01 文案） | fast-fail 正常（E-02） | 随 R2-01 修复一并钉住（category + fix 断言） |
| B-3 | emoji / 组合字符（U+0301）在 sender、@mention、hash 面的不归一化 roundtrip | 正常（E-03/E-04/E-05） | char_tests 增补 emoji sender + 组合字符 mention 黄金用例 |
| B-4 | 文件系统链接（junction / hardlink）透明读写 | 正常（P-05） | 平台相关测试（windows cfg 门控）钉住等价性；symlink 需提权，登记不测 |
| B-5 | 保留设备名（CON/NUL）被后缀归一化封死 | 正常（P-07/P-08） | 补一条 `CON` → `CON.post.md` 归一化断言测试（防 ensure_suffix 回归打开设备面） |
| B-6 | 10MB+ 合法线程 read/send 正确性与 fence 预检成本 | 正常（T-13：85ms/47ms） | 加大线程冒烟测试（可用较小阈值如 2MB 防成本回归），呼应 I-3 登记的权衡 |
| B-7 | advisory 探测对巨大 destination 的成本上限 | 123ms @10MB（A-09） | 文档声明探测为全量读解析（无尺寸护栏）；如需硬上限另行裁定 |
| B-8 | 缺 H1 / 双 H1 线程的宽容读 + validate 放行 | 正常（T-05/T-06） | spec/测试显式声明「H1 在读侧与 validate 侧非强制」，否则宽容面属未裁定行为 |

## 11. 探针异常记录

- **panic：0**（全部失败路径均为结构化信封）。
- **挂起：0**（CON/NUL 探针带 6s kill 保护，均未触发；锁等待探针 C-06 按预期形态返回）。
- **静默数据损坏：0**（写侧探针均有前后哈希/读回归核验；并发探针以集合计数断言）。
- **环境伪影（非产品行为，已甄别排除）**：① 探针早期经 PowerShell 管道捕获非 ASCII 输出时出现 `?` 替换（终端代码页伪影）——改用子进程重定向 + 字节级 UTF-8 解码复核后确认产品输出字节无损（P-03/E-04 字节核验）；② 探针中期 paperwork.exe 两次瞬时「拒绝访问」（CreateProcess 层，疑似杀软扫描窗口），重试即恢复，非产品缺陷；③ CON/NUL 探针首次运行因 Start-Process 继承工作目录误落仓库根（CON.post.md/NUL.post.md），已即时删除并核验 git status。

## 12. 汇总

| 维度 | 数值 |
|---|---|
| 实验总数 | **61**（T×19、P×11、E×5、C×6、N×8、A×9、G×3） |
| 通过 | **58** |
| 缺陷 | **1**（R2-01，低危，文案面） |
| 已知限制 | **3**（L-R2-1/2/3） |
| 行为未钉住点（盲区） | **8**（B-1 ~ B-8） |
| panic / 挂起 / 静默损坏 | **0 / 0 / 0** |
| 并发丢失/损坏 | **0**（6 场景，48+ 进程，最高 16 路） |
| 注入回归（D1/D3/C-1 向量） | **3/3 修复成立，零写入** |

处置建议：**R2-01**（低危）可与盲区 B-2 合并成一个小批（文件读通道 InvalidData 识别 + 文案 + 测试钉住）；**B-1~B-8** 为测试/文档增补项，无代码风险面，建议纳入下一测试批；其中 B-5（保留名归一化）与 B-8（H1 宽容面）建议优先——前者是安全面的隐性护栏，后者是未裁定语义。

与前序深审 B 的增量关系：前序 52 实验发现的 7 缺陷（2 阻塞）本轮全部复测闭合；本轮 61 实验仅新增 1 低危文案缺陷——健壮性基线显著收敛，产品已处于「只剩打磨项」状态。

（完。撰写：任务 #43 健壮性与边界实验审 agent；取证时间 2026-08-15；全部结论基于 TEMP 夹具实测，夹具已清理。）
