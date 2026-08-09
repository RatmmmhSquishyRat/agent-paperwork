# CLI 文法 v0.6 重评估研究（三视角重评估 + path-first 复评 + owner 裁决记录）

> **文档性质**：v0.5 文法（`paperwork <组> <动词> <PATH> [<NAME>] [载荷]`）被 owner 反馈不通顺后的无偏见重评估整合落盘（调研设计文档），是 `docs/ssot/adr/feedbacks/v0.6_feedbacks.md` 的论证依据。只读本文即可理解 v0.6 文法修正的全部决策脉络。
>
> **来源索引**：
> - 视角 A（简洁可维护，研究员 Sena）重评估探索报告
> - 视角 B（agent 效率与扩展性，研究员 Vera）重评估规划报告
> - 视角 C（最小变更低风险，研究员 Milo）重评估探索报告
> - owner 裁决原文：`docs/ssot/adr/feedbacks/v0.6_feedbacks.md` §一
> - 前序治理文档：`docs/ssot/adr/feedbacks/v0.5_feedbacks.md`、`docs/ssot/specs/cli-ux-redesign/design.md`（§1.1、§2.5、§8）

---

## 1. 重评估起因

v0.5.0 发布后（文法总纲：`paperwork [global flags] <group> <verb> <PATH> [<NAME>] [<payload>] [--optional flags]`，NAME/BODY 为位置参数），owner 反馈该文法仍不通顺，痛点有二：

1. **NAME 与 content 混淆**：`post send <PATH> <NAME> [BODY]` 中两个连续位置参数无类型区分，任何字符串都可能是名字也可能是正文，agent 忘写 NAME 时正文会被静默绑定到 NAME 槽（owner 原话：「name这里确实会和content歧义」）；
2. **动词先于路径的语序直觉问题**：owner 原始示例是 path-first 语序（`paperwork <Post文件路径> <使用者的名字> send {...}`），v0.5 将其否决为 action-first，owner 要求对该分歧重评（「动词先于路径」是否真的不成立）。

编排层据此派出三个研究员做无偏见重评估：Sena（简洁可维护视角）、Vera（agent 效率视角）、Milo（最小变更视角）。三人各自独立核实基线、枚举混淆面、评估候选方案，结论汇报后由 owner 作出最终裁决（§7）。

---

## 2. 基线核实结论

三份报告独立核实，结论一致（git 状态以 Sena 逐项举证、Milo 交叉确认为准）：

| 项 | 结论 |
|---|---|
| master HEAD | `a7ea07c`（v0.4 最后一次 docs 提交），master 生效文法为 v0.4 风格（`--from` flag 表身份） |
| cli-ux-v0.5 分支 | 存在，9 commits ahead of master，含完整 format-v2 + v0.5 文法重设计（NAME 位置化），**未合入 master** |
| format-v2 | **仅以未提交脏修改**存在于主工作区工作树（post create 删除、`--title/--participants/--to` 加入 send 等），未形成独立分支，未合入 master |
| crates.io 0.5.0 | 已发布，其内容对应 **cli-ux-v0.5 分支**（非 master、非当前工作树） |

**含义**：v0.6 文法修正的实现基线是 cli-ux-v0.5 分支 + format-v2 工作树变更的合并结果；主工作区 `repos/` 下的 format-v2 脏变更属并行未提交工作，本轮治理文档不触碰。证据：`git log master..cli-ux-v0.5` 显示 9 个 commit；`git status` 显示 22 个 modified + format-v2 docs 为 untracked。

---

## 3. 三视角重评估结论摘要

### 3.1 Sena（简洁可维护视角）

**探索发现**：

- 完整枚举混淆/不通顺场景四类：A. NAME/BODY 混淆（cli-ux-v0.5 设计固有）；B. `--from` 双语义（v0.4 工作树仍存在：send 中=身份、read 中=seq 起点，post.rs L33 vs L71 同名不同类型）；C. 动词先于路径的直觉争议；D. 其他不一致（`contacts create` title 有默认值保留 flag 与 brief/post create title 位置化的用户感受分裂、`post edit --seq` 必填却为 flag、validate 无 `--type`）。
- 判定 NAME/BODY 混淆的**根本原因**：两个连续、可选度不同的位置参数（NAME 必填、BODY 可选）之间无类型区分；design.md §2.5 已承认这是「位置文法固有代价」，只能靠三重教学补偿缓解而非根治。

**方案与取舍**：评估四个候选后推荐混合方案 (d)，即 NAME 回退为具名必填 flag `--from`（send/edit 专用），BODY 保持 PATH 之后唯一位置参数，post read 的 `--from/--to` 改名 `--after/--before` 以消解双语义；其余全部保留 v0.5 设计。取舍核心：牺牲 owner「NAME 前置位置参数」的字面表述，换取混淆面**结构性消除**（不再需要教学补偿）、clap derive 全量利用、help/error 表面积最小；并判定 path-first 工程成本过高（见 §4）。

### 3.2 Vera（agent 效率视角）

**探索发现**：从 agent 误操作代价出发构造 **agent 错误注入矩阵**，逐一检验 v0.5 位置文法下每种错误形态的失败模式。矩阵整理如下（据 Vera 报告结论与其引用的 Sena 枚举发现整合；Vera 报告中编号 #3 即矩阵第 3 行，被其判定为「最危险的静默写入」）：

| # | 触发形态（agent 误操作） | clap 实际解析 | agent 意图 | 失败模式与错误等级 |
|---|---|---|---|---|
| 1 | `post send file.post.md "Hello"`（漏 NAME，把正文放在了 NAME 槽） | PATH=file.post.md，NAME="Hello"，BODY 缺省 | 发送正文 "Hello" | validation exit 1（body 空）；但 agent 无法区分「漏 NAME」与「给了 NAME 缺 body」，重试方向可能错误。显式报错，**中** |
| 2 | `post send file.post.md alice`（给了 NAME，漏 BODY） | PATH=file.post.md，NAME=alice，BODY 缺省 | 发送正文但忘写 | validation exit 1，正常报错。显式报错，**低** |
| 3 | `post send file.post.md "alice bob" "msg"`（NAME 含空格，或多打了一个词） | PATH=file.post.md，NAME="alice bob"，BODY="msg" | "alice" 发送 "bob msg" 一类 | **静默写入错误 sender**：exit 0，错误署名落盘，无恢复出口。**高（最危险）** |

**方案与取舍**：推荐方案 (D) 混合式，即 NAME 保留位置参数（满足当时 owner 硬指令的字面），BODY 改 `--body` flag，从结构上消除 NAME/BODY 混淆面，矩阵第 3 行的静默写入被彻底消除；全具名方案 (C) 安全性最高但违背当时 owner 指令，列为 fallback。path-first 重评结论：即使后缀路由消解了路由前 I/O 问题，**clap 丧失**（help/usage/error 生成全部失效）与**无后缀命令安放**（profile list / validate 无处路由）两个致命缺陷仍在，且引入新的静默错误（裸 `.md` 判型猜错即静默走错 schema），重评后仍应否决。实施前提：先完成 format-v2 提交、研究文档落盘、获得 owner 对 BODY flag 化的确认，然后基于 cli-ux-v0.5 分支修改。

### 3.3 Milo（最小变更视角）

**探索发现**：独立核实基线真相（见 §2），并量化各方案的改动面：BODY flag 化的 CLI 侧 diff 约 60 行，是所有候选中改动量最小且收益最直接的。

**方案与取舍**：推荐方案 (d)，即将 BODY 从位置参数转为 `-m/--message` 具名 flag，结构性消除 NAME/BODY 混淆；path-first 的重评结论独立分离为单独 ADR 处理，不与文法修正耦合。关键约束：format-v2 必须先稳定合入 master；输出协议冻结不可触碰；0.x semver 允许再次 breaking 但需完善 CHANGELOG。

**三视角收敛点**：三份报告对两点的判断完全一致：(1) path-first（含折中形态）否决理由成立；(2) NAME/BODY 混淆面必须**结构性消除**（具名化 BODY，进而具名化 NAME），教学补偿不可作为长期方案。分歧仅在过渡形态（NAME 是否暂留位置槽），该分歧由 owner 裁决一锤定音（§7）。

---

## 4. path-first 复评结论

对两种形态分别再评估：

**形态一：完全 path-first**（`paperwork <PATH> [<NAME>] <verb> [payload]`，owner 字面示例）：

| 实现手段 | 可行性 | 代价 |
|---|---|---|
| `#[command(external_subcommand)]` | clap 将未识别子命令收入 `Vec<OsString>`，可拿到 PATH，但动词分发必须手写 | 丢失全部 clap help/usage/error 生成 |
| 手写两级分发 | 完全可行 | 需自建 help 渲染、usage error 信封、参数解析，等于重写 clap 核心功能，与 v0.5 新增的 usage 信封（错误即指导）直接冲突 |
| 后缀路由 | 读 PATH 后缀确定 group（.post.md -> post 命令集） | 需路由前 I/O；裸 `.md`、文件不存在、无后缀目录全是歧义，判型猜错即静默走错参数 schema（新增静默错误面） |
| `profile list <DIR>` / `validate <PATH>` | 无文件类型实例，path-first 下无法路由 | 必须例外处理或引入 escape hatch，槽位一致性破坏 |

**形态二：组内路径先行折中**（`paperwork post <PATH> send ...`，组已定故无需后缀路由）：虽消解了后缀判型问题，但仍需绕过 clap 的 subcommand 层级手写「路径夹在组与动词之间」的分发与 help 渲染；`profile list`（操作目录而非文件实例）与 `validate` 的槽位依旧被破坏；help/usage/error 表面积反而大于 action-first。折中不成立。

**复评结论**：v0.5 design.md §1.1 四点否决论证（后缀探测路由需 I/O、歧义面随「文件类型 x 动词」平方增长、绕过 clap、破坏槽位一致性）经本轮复评**全部成立**，path-first 两形态维持否决。owner 最终裁决亦明确接受 action-first（「看来action first是cli中的基本设计, 那么我接受你们的这个设计」），该分歧结案。

---

## 5. NAME 位置化混淆面枚举

v0.5 文法 `post send <PATH> <NAME> [BODY]` 的混淆面完整枚举（错误等级沿用 Vera 矩阵标注）：

| # | 触发形态 | clap 实际解析 | 用户/agent 意图 | 结果 | 错误等级 |
|---|---|---|---|---|---|
| 1 | `post send file.post.md "Hello"` | NAME="Hello"，BODY=None | 忘写 NAME，"Hello" 是 BODY | validation exit 1（body 空），无法区分「漏 NAME」与「给了 NAME 缺 body」 | 显式报错，中 |
| 2 | `post send file.post.md alice` | NAME=alice，BODY=None | 正确但忘写 body | validation exit 1，正常报错 | 显式报错，低 |
| 3 | `post send file.post.md "alice bob" "msg"` | NAME="alice bob"，BODY="msg" | NAME 含空格被误解 | **静默错误**（exit 0，写入错误 sender，无恢复出口） | 静默写入，高 |

该混淆面在 v0.5 design.md §2.5 / §7.5 F1 裁定中被认定为「位置文法固有代价」，只能靠三重教学补偿（validation example 提示、after_help 完整三槽示例、SKILL.md 首要提示）缓解。本轮重评估确认：教学补偿无法消除第 3 行的静默写入，唯一结构性解法是 NAME/BODY 双双具名化，这正是 owner 裁决的方向（§6）。

---

## 6. owner 裁决与最终方案

owner 在听取三份重评估汇报后作出最终裁决（原文落盘见 `docs/ssot/adr/feedbacks/v0.6_feedbacks.md` §一）：

1. **接受 action-first**：「看来action first是cli中的基本设计, 那么我接受你们的这个设计」。path-first 结案。
2. **NAME/BODY 具名化**：「name这里确实会和content歧义, 因此我们改为不定位置的必选参数, 也就是用户名使用--author这个全称, 内容使用--message这个全称, 然后简称自己设计」。即 `--author`（名字全称）、`--message`（内容全称）均为不定位置必选 flag；短形式授权实现方设计，编排层裁定 `-a` / `-m`（git `-m` 行业惯例），post read `--mention` 不给短形式（避免 `-m` 双义）。
3. **本轮不发布**：「我没让你发布0.6, 你现在功能都还没有稳定下来」。不 bump 版本、不打 tag、不 publish、不写 CHANGELOG 发布段。

**最终方案**（逐命令签名表落盘于 v0.6_feedbacks.md §2.4，此处不重复）：位置参数仅剩 PATH；`--author` / `--message` 具名必填；`--message` 与 `--stdin` 互斥（同时给报 usage 错误 exit 2，语义上 stdin 优先）；输出协议冻结不变。三个研究员的方案在裁决后收敛为同一形态：Vera 的 `--body` flag 化与 Milo 的 `-m/--message` flag 化被 owner 采纳全称后合流；Sena 提议的 `post read --from/--to` 改名不在 owner 指令范围内，且 v0.5 已通过 NAME 位置化使 `--from` 身份语义消亡、read 的 `--from/--to` 成为唯一语义，故不改名。

---

## 7. 对 v0.5 design.md Rejected Alternatives 的再评估

| # | 原被拒方案 | 再评估状态 | 说明 |
|---|---|---|---|
| 1 | path-first 字面文法 | **维持否决** | 本轮两形态复评确认四点否决理由成立（§4）；owner 裁决接受 action-first |
| 2 | 隐藏弃用别名窗口（旧 flag 保留一版过渡） | **维持否决** | 双文法表面积翻倍的论证不变；v0.6 迁移教学继续由 usage 信封 + SKILL.md + after_help 承担 |
| 3 | `--as` flag 方案 | **被翻转** | 原否决理由是 owner 要求名字为位置参数；owner 本轮显式裁决名字改具名必填 flag（`--author`），名字具名化方向回归，全称由 owner 指定 |
| 4 | SEQ 保留 flag（`edit --seq N`） | **被翻转（改判采纳）** | v0.6 规则「必填与可选一律具名 flag」下，SEQ 回归必填 flag `--seq`；原否决依据（必填即位置参数判据）已废止 |
| 5 | `--seq-from/--seq-to` 改名 | **维持否决** | 前提不变：`--from` 身份语义消亡后仅剩 read 的 seq 范围唯一语义，改名徒增迁移成本 |
| 6 | usage 信封 argv 值迁移重建逐字修正命令 | **维持否决** | clap try_parse 错误对象不携带重建信息的论证不变；静态规范示例裁定继续有效 |
| 新增 | BODY/MESSAGE 具名化（`--body` / `-m --message`） | **采纳** | Vera (D) 与 Milo (d) 方案合流，owner 指定全称 `--message`；结构性消除混淆面矩阵第 3 行静默写入 |
| 新增 | NAME/AUTHOR 具名化（`--author`） | **采纳** | owner 显式裁决；v0.5 曾以 owner 指令为由否决 `--as`（#3），本轮 owner 本人翻转该前提 |

---

## 8. 后续落实项（指向性，非本文档职责）

- 治理：`docs/ssot/adr/feedbacks/v0.6_feedbacks.md`（owner 指令落盘）、`docs/ssot/adr/feedbacks/v0.5_feedbacks.md` §三 追加翻转记录（均已于 2026-08-09 完成）。
- 设计与实现：基于 cli-ux-v0.5 分支 + format-v2 工作树变更，按 v0.6_feedbacks.md §2.4 签名表落实 `--author/-a`、`--message/-m` 具名化；输出协议冻结；本轮不发布。
