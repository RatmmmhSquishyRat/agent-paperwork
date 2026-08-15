# owner 裁决批（任务 #36）完整性评审报告（2026-08-15）

- **评审维度**：完整性（需求覆盖）——只查遗漏实现、虚假销账、声称与实况不符、验收标准未满足、裁决解释口径未显式落文档；正确性 bug 与回归影响面不在本报告范围（另有评审员）。
- **评审范围**：`git diff d920271..HEAD`（HEAD = b9b059c）。范围事实更正：该区间为 **6 个提交**（9821933 O1、14f3b57 O2、77f19e2 O3、6a36639 O4、72c85ac O5、b9b059c O5 补录），非任务书所称 7 个；基线提交 d920271 本身是任务 #35（裁决记录 + spec 增量修订），不在本区间内。
- **需求基线**：docs/dev/owner-rulings-2026-08-15.md（口径 A~D）；impl_plan.md「2026-08-15 owner 裁决实施批次」O1~O5；spec.md/bdd.md/tdd.md 修订段；rulings-execution-log-2026-08-15.md。
- **评审方式**：只读。全量 diff 逐行比对；`cargo test --workspace --locked` 现场复跑；基线 d920271 版本对照；文档面逐点位核查。未改任何源代码，未做 git 提交（本报告落盘除外）。

---

## 阻塞（MUST FIX）

无。

---

## 重要（SHOULD FIX）

### I-1 impl_plan 定义的 O5（独立 QA Review Book）交付物在 HEAD 缺失，批次号被改挂 CHANGELOG 且偏离未登记
[impl_plan.md#L187-L191](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/ssot/specs/cli-grammar-v0.6/impl_plan.md)

**Problem**：impl_plan 步骤 O5 的定义是独立验证 QA Review Book，落点 `docs/reviews/v0.6-owner-rulings-review-{实施完成日期}.md`，且明确「不得由 impl agent 自评」「O4 后执行」。实际 d920271..HEAD 中：
1. `docs/reviews/` 下不存在任何 `*rulings*` / `v0.6-owner-rulings-review*` 文件（Glob 核查为空）；
2. 提交 72c85ac 把「O5」批次号改挂给了 CHANGELOG Unreleased 条目 + 执行日志（CHANGELOG 不在 impl_plan O1~O4 任何一步的内容清单内，O4 仅声明「不写 CHANGELOG 发布段」）；
3. 执行日志（[rulings-execution-log-2026-08-15.md#L72-L75](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/rulings-execution-log-2026-08-15.md)）通篇以 O5 = CHANGELOG 记账，**没有任何一行登记「O5 定义偏离 impl_plan / QA 移交」这一口径变更**。

现场存在未跟踪文件 `docs/dev/rulings-verification-2026-08-15.md`（自称任务 #37 第一关端到端验证报告），说明 QA 可能已被编排层移交任务 #37 承载；但它（a）未提交、（b）落点与命名均非 impl_plan 点名形态、（c）移交决定未见诸执行日志或 impl_plan 回改。按「逐条核对 O1~O5 各批验收要求」口径，O5 验收标准（QA Review Book 在场）在 HEAD 未满足，且构成「声称与实况不符」（执行日志声称 O1~O5 全批执行完毕，其 O5 与 impl_plan 的 O5 不是同一件事）。若编排层确认三维评审 + 任务 #37 即 O5 的替代承载，则本条降级为「需补偏离登记」，但登记本身仍是缺项。

**Fix**：二选一并回改文档：
- 方案 A：由独立验证 agent 落 `docs/reviews/v0.6-owner-rulings-review-2026-08-15.md`（可并入/引用三维评审与任务 #37 实测结论）；
- 方案 B：在执行日志头部与 impl_plan O5 段显式登记口径变更（「O5 QA 移交任务 #37 / 三维评审承载，CHANGELOG 批为追加批 O5'」），并把 72c85ac/b9b059c 的批次标签改为不含歧义的记法。

### I-2 S-SHORT-02 收窄验收项未落实：写侧 `--reply-to` 负向探针未移除、注释仍为旧「26 项」口径
[repos/paperwork-cli/tests/cli_integration.rs#L3775-L3878](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/repos/paperwork-cli/tests/cli_integration.rs)

**Problem**：三处基线一致要求移除 send 侧探针：
- bdd S-SHORT-02（[bdd.md#L510](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/ssot/specs/cli-grammar-v0.6/bdd.md)）：「写侧 send `--reply-to`/`--mention` 已撤销，post 侧补充项仅剩 read 侧两项，**send 侧探针移除**」；
- tdd §9.2 新增用例映射表：「白名单负向清单收窄 | **send 侧 `--reply-to`/`--mention` 探针移除**；read 侧两项保留」；
- impl_plan 自查遗留差异点名：负向探针「26 -> 24，重盘归本批 O3/O5」。

实际 `short_form_whitelist_is_exact` 与基线 d920271 逐字节未变：L3867-3878 仍保留 `// --reply-to (post send side)` 探针（`post send ... -r 1`），L3775 注释仍写「full 26-flag no-short-form list」。该探针目前只是碰巧通过（`-r` 成了未知短形式），其断言语义已从「`--reply-to` 无短形式」漂移为「`-r` 未知」，与注释和 bdd 枚举口径双重失真。这是 tdd §9.2 明确列出的验收项，实际 diff 未覆盖。

**Fix**：删除 L3867-3878 的 send 侧探针条目，并把 L3775-3779 注释更新为收窄后口径（枚举制、read 侧 `--reply-to`/`--mention` 两项补充），与 bdd S-SHORT-02 现行文本对齐。

---

## 低（CONSIDER）

### L-1 design.md §2.1 同节内残留与更正签名直接矛盾的旧糖衣示例块；执行日志点名行号有漂移
[design.md#L80-L86](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/ssot/specs/cli-grammar-v0.6/design.md)

**Problem**：O4 已在 design.md §2.1 顶部签名示意（L47-49）与 §12 基线勘误行（L268）加裁决更正，但同一 §2.1 内的 after_help 文案示意块（L81 `... -m "Tests merged." --reply-to 2 --mention bob`、L86 `# --reply-to / --mention are sugar flags: their values are injected...`）仍逐字保留旧机制描述，与三行之上的更正注直接矛盾；另有 L219「`--reply-to` 指向不存在 seq 静默跳过（冻结沿用）与 Q-02 的张力」以未决项口吻存续（该问题面已随裁决消解销账）。执行日志登记「L60/L79/L84/L217 属历史论证文本，按 impl_plan 范围不改」——判断本身在 impl_plan 授权裁量内（「是否顺带回改由本批 O4 或后续文档轮裁定」），但登记行号与实际残留位置（L62/L81-86/L219）不一致，且未点明 L81/L86 属已更正的 §2.1 同节。

**Fix**：在 L81/L86 处追加一行〔2026-08-15 owner 裁决更正〕注（体例同 L47-49），或把执行日志的点名行号校正为实际行并补记「§2.1 after_help 示意块留待后续文档轮」；L219 可一并加「随 2026-08-15 裁决消解」尾注。

### L-2 执行日志「每 O 批一个提交」的纪律自述与 O5 双提交不符
[rulings-execution-log-2026-08-15.md#L5](c:/Users/15480/Desktop/AIWorkshop/repos/agent-paperwork/docs/dev/rulings-execution-log-2026-08-15.md)

**Problem**：日志头部纪律执行写「原子提交（每 O 批一个提交，带批次号）」，实际 O5 有两个提交（72c85ac + 补录 b9b059c），区间总计 6 提交对 5 个批次号。补录行为本身合理（回填提交 hash），但与自述纪律字面不符，属轻微记账失真。

**Fix**：日志纪律行补一句「O5 含一个补录提交 b9b059c（回填 O5 提交 hash）」即可。

---

## 逐条基线核对结果（证据链）

### 1. owner-rulings 四项裁决与口径 A~D
裁决原文与口径 A~D 完整落盘于基线提交 d920271（本区间不回改，符合任务 #35/#36 分工）。实施面对口径的落地核查：
- **口径 A（写侧撤销 + usage exit 2 + 迁移教学）**：post.rs send 臂 clap 定义、`--reply-to 0` 拒绝、`clean_list`/`validate_mention_value`/`inject_reference_tokens` 三助手、flag 派生 implicit-mention、dedup 管道全部删除（diff post.rs -155 行面）；usage_fix 新增 post.send/post.edit 两分支且置于通用长 flag 分支之前（main.rs L244-250）；edit 外延由 usage_fix 覆盖（golden `usage_revoked_reply_to_edit_stderr` 的 example 为 edit 完整必填形态，满足 S-EDIT-10）。**覆盖**。
- **口径 B（读侧过滤器保留显式声明）**：spec §1.4/§2/§3.3/§10 声明在场（spec.md L41/L53/L121-128/L311）；实现面 post read 臂一字未动（diff 零触及）；读侧用例（S-READ-04/06/07 等）原样冻结。**覆盖**。
- **口径 C（advisory 非阻塞契约）**：三触发条件按「存在性 -> 可读性 -> parse 合法性」顺序实现（contacts.rs `destination_advisory`）；写后探测、不改退出码、不新增 flag；三文案逐字采用 spec §3.6 建议形态；`Envelope::field` 同 key 流入 Default/--json；spec §3.6 与 bdd S-CONTACTS-16 的「实施可微调」标记已回冻为「定稿冻结」。**覆盖**。
- **口径 D（U-13 结案 + agent-first 方向）**：spec §10 表登记「不改 spec」；SKILL.md/README contacts advisory 段落均落「agent-first ruling」方向措辞；无 completions 相关代码新增。**覆盖**。

### 2. impl_plan O1~O5 批次定义与各批验收
- O1：门禁口径（build+clippy 绿、允许测试红）与日志一致；VALUE_TAKING_FLAGS 保留 `--reply-to`/`--mention` 在列（main.rs L300-301，spec §5 第 5 条钉住项）；canonical_example 无需改动的声称属实（post send 规范示例本无糖衣 flag）；after_help 换正文直书教学（`@#2 Sure, @alice I'll take it.`）。**覆盖**。
- O2：探测对象 add `--profile` / update `--new-profile` 正确；路径解析复用 R11；四形态冒烟有测试对应（S-CONTACTS-16/17 用例三形态 + 反向断言齐全）。**覆盖**。
- O3：硬门禁现场复跑验证——`cargo test --workspace --locked` 10 个二进制 **426 全绿**，分解（6+33+148+4+101+12+33+18+71+0）与日志逐字一致；clippy/fmt 日志声称未复跑（测试面已足够佐证主门禁）。改写/删除/新增清单与 tdd §9.1/§9.2 逐项吻合；cli_integration 143->148、char_tests 31->33 算术自洽（-3 删 +8 新增含 pass_through 替代）。**覆盖**。
- O4：SKILL.md/根 README 示例与说明段全改正文直书 + 撤销声明 + advisory 声明，现场 grep 确认两文件已无写侧糖衣 flag 残留（仅存 read 过滤器合法示例）；cli README「无撤销 flag 示例无需改动」声称属实（grep 零命中）；design.md 点名两处已改。**覆盖**（残留见 L-1）。
- O5（impl_plan 定义）：**未满足**，见 I-1。

### 3. spec/bdd 修订段 vs 实施
S-SEND-22/23（send 撤销 usage，含 JSON 档与无写入断言）、S-EDIT-10（双 flag 写命令外延）、S-CONTACTS-16/17（add/update advisory 全触发形态 + 不触发反向 + updated/advisory 并存）均有对应用例且断言要点与 bdd 文本一致；advisory 契约五要素（触发条件/三文案/同名 key `advisory`/纯 ASCII 字节断言/非阻塞 exit 0）全部落测试；fix 文案定稿与执行日志登记逐字一致（golden 快照佐证）。**覆盖**。

### 4. 执行日志声称 vs 实际 diff
逐项抽查：426 测试与分解（复跑证实）、VALUE_TAKING_FLAGS 保留、读侧一字未动、FROZEN 表 150 条目、core 与 ops_tests 零触碰（diff --stat 无 core 文件）、cli README 无需改动、O5 hash 补录（b9b059c 内容证实）——**无虚假销账**。失实/未登记项见 I-1（O5 定义偏离）与 L-2（提交纪律自述）。

### 5. tdd §9 盘点与黄金快照重冻预告 vs 实际测试 diff
- §9.1 五类处置全部落实（7 改写 + flag_inventory 翻转 + 3 删除 + advisory 叠加 + after_help 文案同步）；
- §9.2 十行映射表 9 行落实，唯「白名单负向清单收窄（send 侧探针移除）」未落实，见 I-2；
- §9.3 重冻预告 vs 实际 FROZEN diff：**变更恰 4 项**（contacts_add_second_json_stdout、post_send_implicit_mention_file、post_send_mention_file、post_send_reply_missing_seq_file）、**新增恰 7 项**（3 advisory 金样 + 4 revoked usage 金样），与预告及日志清单逐一对应；替换关系已在 77f19e2 提交信息登记（满足 §9.3「不得就地删除、提交信息登记」要求）；
- §9.4 ops_tests.rs 字节级零改动证实（core 无任何 diff）。**覆盖（除 I-2）**。

### 6. O4 文档点名项与 O5 CHANGELOG
SKILL.md、根 README、cli README、design.md 四点均已处置（见第 2 节 O4）；CHANGELOG `[Unreleased]` 顶部两段（Removed 糖衣 flag / Added advisory）内容与行为变更清单一致，显式声明「no bump, no tag, no publish；crate version stays 0.5.0」（Cargo.toml 两 crate 均 0.5.0 证实），未新增发布段、未回改既有发布段，符合交付边界。**覆盖**。

---

## 维度结论

**完整性判定：有条件通过（1 阻塞缺项被降级为重要后的「基本覆盖，两项应补」）。**

裁决 1/2/3 的实施面（写侧撤销 + usage 迁移教学 + advisory 非阻塞校验）与裁决 4 的登记面全部落地；执行日志核心声称（测试数、快照重冻、零触碰防线）经现场复跑与逐项比对无虚假销账；口径 A~D 均有实现与文档双重落点。遗留两项重要缺口：**I-1** impl_plan O5 QA Review Book 在 HEAD 缺失且批次号改挂未登记（属验收标准未满足 + 声称与实况不符，若编排层确认三维评审/任务 #37 为替代承载则只需补偏离登记）；**I-2** S-SHORT-02 收窄的 send 侧探针移除这一明确验收项未执行。两项低风险为文档残留与记账措辞。修复 I-1/I-2 后本维度可判全量覆盖。

（完整性评审完。评审 agent：三维评审·完整性维度；2026-08-15。只读评审，未改源代码，未 git 提交。）
