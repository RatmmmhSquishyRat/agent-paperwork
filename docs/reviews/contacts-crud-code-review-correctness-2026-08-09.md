# contacts CRUD 轮代码评审 —— 正确性维度

- **评审对象**：worktree `agent-paperwork-wt-v06grammar`，分支 `cli-grammar-v0.6`，基线 `0f6c384` 之后 6 个提交 `77ab558..e7eb049`
- **评审维度**：仅正确性（逻辑 / 并发 / 安全 / 测试正确性）；完整性与影响面另有专人负责
- **日期**：2026-08-09
- **方法**：diff 全量逐行核查；lock.rs 与 thread_edit 模板逐步对照；本机 `cargo test --workspace` 全量运行（270 用例全绿）；空字符串边界以本分支二进制实测复现

## Critical Issues (MUST FIX)

无。

## Warnings (SHOULD FIX)
### M-1 `--profile`/`--new-profile` 空字符串值静默损坏 contacts 文件（既有条目丢失）

[repos/paperwork-core/src/ops/contacts.rs#L129-L176](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-core/src/ops/contacts.rs)；[repos/paperwork-cli/src/cmd/contacts.rs#L127-L136](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-cli/src/cmd/contacts.rs)
**问题**：clap 层与 core 层均未校验空字符串值。实测复现（Windows，本分支二进制）：

```
paperwork contacts add team --profile alice.profile.md       # exit 0
paperwork contacts update team --profile alice.profile.md --new-profile ""   # exit 0
# 文件内容变为 "- []()"
paperwork contacts read team                                  # exit 0，0 contacts
```
parse_link_bullet 对裸形式空 dest 直接返回 None（format/contacts.rs L138-L140），该 bullet 在下次解析时静默消失：一个合法既有条目被替换成不可解析的 bullet，全程无错误，属静默数据丢失；`validate --type contacts` 对该文件判 exit 1（结构已坏）。此路径由本轮 update 动词新开（add `--profile ""` 同样写入 `- []()`，为既有缺陷，但 update 把它升级为「破坏既有条目」）。
**修复**：contacts_add/contacts_update 入口对 `--profile`/`--new-profile` 增加 trim 非空判定，空值落 validation 错误 exit 1（镜像 post send `--message` 空值 trim 判定先例）。建议在 core 函数入口实现，同时覆盖库直调。
### M-2 --stdin 负向探针存在假阳性结构，无法检出其钉住的回归

[repos/paperwork-cli/tests/cli_integration.rs#L2489-L2490](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-cli/tests/cli_integration.rs)（探针）与同文件 L2542-L2547（断言）

**问题**：探针为 `["post","send",path,"-S","body"]`，而 `--stdin` 是 bool flag（cmd/post.rs L68-L70）。若未来有人给 `--stdin` 挂上短形式 `-S`，clap 会把 `-S` 解析为 bool flag，随后把 `body` 报为 UnexpectedArgument —— code(2)+`error usage:`+`unexpected argument` 三元断言仍然全部满足，探针静默通过，回归无法检出。这正是 BUG-5 教训所警示的假阳性结构。其余 26 条探针经逐一推演，结构有效。
**修复**：探针改为 `["post","send",path,"--author","a","--message","m","-S"]`：若 `-S` 挂上则落入 `--message`/`--stdin` conflicts（消息不含 unexpected argument，断言失败→检出回归）；未挂则 `-S` UnexpectedArgument→照常通过。
### M-3 测试与注释钉住的 bdd/spec 场景号在本分支 SSOT 中不存在，判定口径悬空

[repos/paperwork-cli/tests/cli_integration.rs#L2479-L2483](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-cli/tests/cli_integration.rs)；[repos/paperwork-cli/src/cmd/brief.rs#L170-L172](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-cli/src/cmd/brief.rs)
**问题**：对照 worktree `docs/ssot/specs/cli-grammar-v0.6/` 实测：

- bdd.md 无 S-BRIEF-07~09（brief 节仅到 S-BRIEF-06）；contacts 节止于 S-CONTACTS-05（L300-L325）；全文无 S-LOCK-01/02、无 §12；唯一存在的 S-BRIEF-07 在 cli-ux-redesign bdd，语义为 basename 映射，与本轮 entry-title 过滤无关。
- 测试注释声称 bdd S-SHORT-02 full 26-flag 清单，但 bdd S-SHORT-02 原文为共 24 项且不含 `--new-profile`。
- spec.md §3.5 明言 read/verify 不变（L145），§3.6 文法表无 remove/update（L150-L154）。
后果：conclusion 保持全量、OLD 命中先于 NEW 已存在、NEW 不存在静默成功等关键判定口径仅由实现与测试互为见证，分支上无规范出处；本轮要求的与 spec/bdd 一致性核对无法成立（实现自身自洽、无逻辑矛盾，问题在引用链断裂）。
**修复**：要么在本分支落地本轮 spec/bdd 增量（含 S-SHORT-02 清单扩列与计数订正），要么把测试/代码注释改为引用实际存在的 SSOT 出处。spec 缺失部分由完整性维度专人覆盖，此处仅以测试正确性单列（引用链断，断言基准悬空）。

## Suggestions (CONSIDER)
### m-1 contacts_add 幂等分支由零写入退化为整文件重写

[repos/paperwork-core/src/ops/contacts.rs#L66-L82](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-core/src/ops/contacts.rs)（幂等分支 L71-L73）；[repos/paperwork-core/src/ops/lock.rs#L84-L110](/c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork-wt-v06grammar/repos/paperwork-core/src/ops/lock.rs)
**问题**：基线（0f6c384）幂等分支为 `return Ok(())`，完全不触碰文件；迁锁后闭包在幂等时返回 `Ok(content)`（原内容），但 locked_read_modify_write 对任何 Ok 都无条件 set_len(0)+seek(0)+write_all —— 纯 no-op 也走一次 truncate 崩溃窗口，且 mtime 抖动。这是六写路径等价性核查中除加锁本身外唯一的行为漂移。
**修复**：锁助手在新内容与旧内容字节一致时跳过重写（modify 前克隆一份留作比较，或闭包签名改返回 `Option<String>`、None 表示无需写）。
## 已核实无问题项（对应评审清单）

**1. 锁实现**：locked_read_modify_write 与 thread_edit 六步模板逐步一致 —— 同一 r/w 句柄内 lock_exclusive → seek(0) → read_to_string（同一持锁句柄读，满足 Windows os error 33 判例）→ 闭包 mutate → set_len(0)+write_all（锁内 truncate 重写）→ unlock。全部 6 条错误路径先 `file.unlock().ok()` 再返回，无锁泄漏；成功路径末尾 unlock 失败映射为 IoContext。锁获取失败 fast-fail 为 IoContext，无降级旁路。
**2. remove/update 逻辑**：键为未转义 profile_path 精确匹配；retain 后按 len 对比判空；update 判定顺序 OLD 命中先于 NEW 已存在（OLD==NEW 且命中→AlreadyExists，与注释及测试一致）；原位 `contacts[index]` 替换保序；derive_label 读 H1 失败回退 stem。
转义条目（尖括号形式）的 update/remove 以未转义串为键可命中，序列化往返合法（ops_contacts_crud_tests 与 CLI 集成测试覆盖）。

**3. 六写路径等价性**：brief_add/brief_remove/profile edit 迁锁后闭包逻辑与原版逐行等价，仅加锁；除 m-1 外无行为漂移。
**4. brief read --entry-title**：JSON 与 Default 两路均用 `total = manifest.entries.len()` 生成 conclusion、用过滤后的 entries 生成列表，口径一致、无 off-by-one；conclusion 保持全量 N entries 的实现口径成立（但该口径的规范出处缺失，见 M-3）；miss 落 not-found exit 1；`detailed = full || entry_title.is_some()`，--full 与 --entry-title 同给合法等价；--plain 为全文原样输出档，不参与过滤（既有语义）。
**5. 测试正确性**：多进程锁测试用 BTreeSet 集合比较（BUG-5 教训落实，无位置配对）；并发测试断言子进程全成功+最终集合相等，无假阳性结构（除 M-2 单点）；全工作区 270 用例本机全绿（Windows）。

**6. 边界**：空 contacts remove→not-found 且零写入；仅 title 文件 remove→not-found；NEW==OLD→AlreadyExists；空字符串 flag→见 M-1（brief 侧 `--entry-title ""` 安全落 not-found，contacts 侧不安全）。
## 总判定

**C = 0，M = 3，m = 1。有条件通过。**

M-1 属边界路径静默数据丢失，一旦 agent 传入空字符串值即不可逆地破坏既有条目，必须合入前修复；M-2/M-3 属测试防线与引用链问题，不阻塞功能正确性，但建议本轮一并处理。m-1 为性能/持久化层面的退化，可作为跟进项。

---

## 修复回应销账段（2026-08-09，编排层裁定 F1-F7 落实）

| 发现 | 处置 | 销账证据 |
|---|---|---|
| M-1 空字符串值静默损坏 | 已销账（F1） | core `contacts_add`/`contacts_update` 入口 trim 非空校验（Validation exit 1，镜像 post send `--message`/`--author` 空值判定先例，库直调同覆盖）；CLI `brief read --entry-title` 同护栏；行为变更已登记主工作区 spec.md §3.5/§3.6 与 bdd.md S-BRIEF-10/S-CONTACTS-15；新增 core 用例 2 个（add 空键 / update 双空键）+ CLI 集成用例 1 个（三类命令全覆盖）；release 二进制实测三命令均 exit 1，拒绝调用零写入 |
| M-2 --stdin 探针假阳性 | 已销账（F2） | 探针改为 `["post","send",PATH,"--author","a","--message","m","-S"]`：未来误挂短形式将落入 `--message`/`--stdin` conflicts 分支（消息不含 unexpected argument，断言失败可检出）；未挂则 -S UnexpectedArgument 照常通过；其余探针不动 |
| M-3 场景号引用链断裂 | 已销账（F3） | 主工作区已修订治理文档同步进 worktree 并随修复批提交（spec/bdd/tdd/impl_plan/design/README + v0.7_feedbacks + 研究文档 + 四份文档评审）；S-BRIEF-07~09、S-CONTACTS-06~14、S-LOCK-01~03、S-SHORT-02 26 项清单（含 --new-profile）已实测在 worktree SSOT 中真实存在 |
| m-1 幂等分支零写入退化 | 已销账（F4） | `locked_read_modify_write` 在闭包返回内容与原内容字节相同时跳过 truncate+write（仅解锁返回）；contacts add 幂等路径恢复基线零写入语义；新增用例 `idempotent_add_keeps_bytes_and_mtime_stable` 断言字节恒等 + mtime 不变（实测通过）；spec §3.9 补录登记 |

修复后验证：`cargo test --workspace` 274 全绿（新增 4 用例计入）；`cargo clippy --all-targets -D warnings` 零警告；`git diff master -- repos/paperwork-core/tests/ops_tests.rs` 为空；版本 0.5.0 不变。
