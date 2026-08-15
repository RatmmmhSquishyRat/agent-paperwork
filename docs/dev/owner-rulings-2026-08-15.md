# owner 四项裁决记录（2026-08-15）

- 日期：2026-08-15
- 任务：#35（落盘 owner 四项裁决并做 spec 增量修订；纯治理文档，零代码变更）
- 裁决对象：台账 LED-09（backlog B-01）/ LED-10（backlog B-02）/ LED-11（backlog U-04）/ LED-12（backlog U-13）
- 文档性质：owner 裁决的权威记录点；解释口径由编排层拟定并显式声明，供 owner 后续复核推翻
- 联动落点：台账第十三节（docs/dev/open-items-ledger-2026-08-15.md）、backlog 第九节注记（docs/researches/ux-open-items-backlog-2026-08-08.md）、spec 目录增量修订（docs/ssot/specs/cli-grammar-v0.6/ 五份文档）

---

## 一、裁决原文（逐字保留）

1. 对 B-01（--reply-to 指向不存在 seq 静默跳过）：
   「reply, mention 等等语义都在 markdown 消息本身中负责表达, cli 部分不应该给出这种参数用法」
2. 对 B-02（contacts add/update destination 写前校验候选）：
   「contact 可以加入路径和格式上面的 validation, 但是不阻塞 agent 编辑即可, 也就是如果文件不存在或者错误, 也不需要阻塞编辑的 agents 添加这个 profile 文件」
3. 对 U-04（--mention 与正文 @alice 冗余）：
   「和第一个问题一样处理, cli 这边不要这种语义参数」
4. 对 U-13（shell completions）：
   「不需要, 这个 cli 主要是 agent 自己使用, 按照 agent 的使用习惯去做 qol/UX」

---

## 二、解释口径（编排层拟定，显式声明，供 owner 后续复核推翻）

### 口径 A：裁决 1/3 的作用域 = 写侧语义糖参数撤销

- 撤销对象：post send 的 `--reply-to`（糖衣 flag：值以 `@#N` token 注入正文）与 `--mention`（糖衣 flag：值以 `@name` token 注入正文）；post edit 本无该两 flag（v0.6 既成事实），口径按「写命令」外延一并声明。
- 撤销后行为：写命令传入该两 flag 一律落 usage 信封 exit 2（未知 flag 路径，usage 信封机制承担迁移教学）。
- 语义承接：reply/mention 语义由 agent 在消息正文直接书写 `@#N` / `@name` token 表达；**读取时 derive 机制不变**（post read 从正文派生 reply/mention 关系、implicit-mention 输出字段派生逻辑均冻结）。

### 口径 B：读侧过滤器保留（必须显式写明）

- post read 的 `--mention <name>` / `--reply-to <seq>` 是**查询过滤器**而非语义表达，不在撤销范围，全部保留冻结（含无短形式钉住、showing/window 口径）。
- 该区分是裁决 1/3 的边界线：撤销的是「写侧替 agent 表达语义的参数」，保留的是「读侧查询已有语义的过滤器」。此口径已同步写入 spec.md（§1.4/§2/§3.3/§10）与 bdd.md（S-SHORT-02 枚举更新）。

### 口径 C：裁决 2 = contacts add/update 非阻塞 advisory 校验

- contacts add/update 增加 destination 路径与格式的 **advisory（建议性）校验**：destination 路径不存在 / 不可读 / 格式非法时**仍照常写入、exit 0**，仅在 ok 信封增补 advisory 提示字段。
- 协议面：只增不改（JSON key 只增不改不删纪律之下新增字段），永不因 destination 问题 exit≠0；字段契约逐字定义见 spec.md §3.6「destination advisory 校验契约」与 §5 第 6 条（字段名钉住 `advisory`，候选名 `destination_note` 记录为被否替代）。

### 口径 D：裁决 4 = U-13 钉住结案 + 长期方向确立

- shell completions 正式钉住为已知限制结案（不再作延后悬置项）；理由 = owner 原话：该 CLI 主要是 agent 自己使用。
- 确立长期方向：**本 CLI 的一切 UX/QoL 决策以 agent 的使用习惯为准**（后续新增 UX 提案的默认裁决基准；人类向增强类提案默认不立项，除非 owner 另有指示）。

---

## 三、影响面（变化预告）

### 3.1 spec 条目变化

- spec.md §1.4（规则 3 边界裁定）：send 侧「设置」语义废止条款更新为撤销声明；read 侧「过滤」保留。
- spec.md §2（签名全表）：post send 行删除 `[--reply-to N] [--mention a,b]` 两个可选糖衣 flag；post read 行不变并加保留声明。
- spec.md §3.1（post send 契约）：参数表删除该两 flag 行；错误映射新增「写命令传入该两 flag -> usage exit 2」；删除「--reply-to 指向不存在 seq 静默跳过（沿用）」条款（B-01 问题面随撤销消解）。
- spec.md §3.6（contacts 组）：新增「destination advisory 校验契约」条款；S-CONTACTS-14 静默面契约之上叠加 advisory 提示（exit 0 语义不变）。
- spec.md §7 新增第 6 条：本轮裁决修订面登记（撤销面 + 新增面 + 读侧豁免）。
- spec.md 新增 §10：裁决增量修订登记结案。

### 3.2 flag 面变化

- 撤销：post send `--reply-to`、post send `--mention`（写侧两个可选带值 flag）。
- 保留：post read `--reply-to`、post read `--mention`（读侧过滤器）；短形式集合 {-a, -m, -q} 不变。
- 负向清单口径：无短形式负向断言枚举中「post send/read 两侧 `--reply-to` 与 post read `--mention`」收窄为「post read `--reply-to` / `--mention` 两项」（send 侧 flag 不复存在，探针相应移除）。
- VALUE_TAKING_FLAGS 对应表（main.rs L281-307）：`--reply-to` / `--mention` 两项**保留在列**——post read 侧仍是带值 flag；撤销仅作用于写侧 clap 签名，若从该表移除会破坏 usage 路径 `--json` 探针（如 `post read x --mention "--json"` 误判），口径写入 spec.md §5 第 5 条。

### 3.3 输出协议变化

- 新增 ok 信封字段 `advisory`（仅 contacts add/update 成功且 destination 异常时出现；Default 与 --json 档同名；纯 ASCII；只增不改协议）。
- 其余信封结构、七类 category、command 标识、退出码语义全部冻结不变。

### 3.4 测试冻结面变化预告

- **黄金快照重冻**：撤销写侧 flag 与 advisory 字段引入属行为变更，既有涉 `reply_to` / `mention` 注入逻辑、S-SEND-04/S-SEND-20 派生断言、contacts add/update 信封的测试与黄金快照需在实施批次（任务 #36）行为定稿后一次性重冻；预告与映射见 tdd.md §9。
- 新增测试面：写侧撤销 usage 场景（bdd S-SEND-22/S-SEND-23/S-EDIT-10）、正文 @语义往返场景（改写后的 S-SEND-04/S-SEND-20）、contacts advisory 场景（bdd S-CONTACTS-16/17）。

### 3.5 文档面差异（留给实施任务同步）

- SKILL.md 现有写侧糖衣 flag 示例（如 `post send ... --reply-to 1 --mention alice`）与根 README 相关示例，在 spec 修订后与 SSOT 存在差异；按治理纪律留给任务 #36 实施批次同步（impl_plan.md「2026-08-15 owner 裁决实施批次」步骤 O4 点名），本任务不回改。

---

## 四、闭合去向索引

| 台账项 | backlog 项 | 裁决结论 | 落点 |
|---|---|---|---|
| LED-09 | B-01 | 撤销写侧 `--reply-to`/`--mention` 后，「指向不存在 seq 静默跳过」问题面消解（B-01 候选方向 reply-dropped 字段一并废弃） | spec §3.1/§10；bdd S-SEND-20~23；台账第十三节 |
| LED-10 | B-02 | 非阻塞 advisory 校验（exit 0 语义不变 + ok 信封 advisory 字段） | spec §3.6/§5；bdd S-CONTACTS-16/17；台账第十三节 |
| LED-11 | U-04 | 同 B-01 处理（裁决原文「和第一个问题一样处理」），--mention 撤销后「正文自动提取 vs 显式 flag 冗余」方向消解销账（读取时 derive 机制本就覆盖正文 @提取） | spec §1.4/§3.1；台账第十三节 |
| LED-12 | U-13 | 钉住结案；确立「UX/QoL 以 agent 使用习惯为准」长期方向 | 本文 §二 口径 D；台账第十三节 |

实施承载：任务 #36（按 impl_plan.md 追加的 O1~O5 批次执行）；本记录与 spec 修订均由任务 #35 落盘提交，不推送（推送由后续统一执行）。

（裁决记录完。记录人：任务 #35 执行 agent；2026-08-15。解释口径为编排层拟定，owner 复核推翻时以 owner 新指示为准并回改本节。）
