# owner 裁决实施（O1~O5）影响面评审报告 — 2026-08-15

- 评审维度：**影响面（回归与破坏性变更）** 单维评审（需求覆盖与内部逻辑 bug 由另两位评审员负责）。
- 评审范围：`git diff d920271..HEAD`（master @ b9b059c，提交链 9821933 O1 / 14f3b57 O2 / 77f19e2 O3 / 6a36639 O4 / 72c85ac O5 / b9b059c O5 补记，共 6 提交）。
- 方法：git 考古（diff/log/commit message/tag/branch）+ 只读实测（cargo build/test + release 行为冒烟，全程未改任何源码、未做任何 git 提交）。
- 实测基线：`cargo test --workspace --locked` 复跑 **426 全绿**（6 + 33 + 148 + 4 + 101 + 12 + 33 + 18 + 71 + 0），与执行日志逐位一致。

## 发现分级

### Warning（SHOULD FIX）

**W-1 CHANGELOG `[Unreleased]` 内残留两条已失效的糖衣 flag 条目，与新 Added/Removed 段自相矛盾**
- 位置：CHANGELOG.md L42（NEW-12）与 L45（NEW-10），均在 `[Unreleased]` 段（L6 起）内。
- 证据：L45 原文 "The last inline mention dedup loop (`post send --mention`) moved onto the shared order-preserving `dedup_preserve_order` helper" —— O1（9821933）已把 send 侧 `clean_list`/`validate_mention_value`/dedup 管道**整体删除**（diff 可证），该条目描述的代码面在 unreleased 态已不存在，属**事实错误**而非措辞过时；L42 将尾扫优化归于 `post send --reply-to` flag，而该 flag 现为 usage exit 2（L10 Removed 段自证）。
- 影响：同一 `[Unreleased]` 段内 breaking 披露自相矛盾——未来发布时该段整体晋升为发布段，会带着「教已撤销 flag 用法」的条目出库，削弱 L10 Removed 段迁移教学的权威性。
- 建议：将 L42/L45 两条按「机制现由正文 `@#N` token 驱动 / send 侧 mention dedup 已随 flag 撤销整体删除」口径更正或追加 2026-08-15 裁决注记（不触既有发布段纪律，`[Unreleased]` 段本就可改）。

**W-2 design.md §2.1 规范性 after_help 示例文案仍教旧糖衣用法，与同节更正注自相矛盾**
- 位置：docs/ssot/specs/cli-grammar-v0.6/design.md L81（示例行 `--reply-to 2 --mention bob`）与 L86（注记 "--reply-to / --mention are sugar flags: their values are injected..."）。
- 证据：同节 L47~L49 已按裁决更正（签名去糖衣 flag + 更正注），实际 shipped after_help（post.rs）也已换正文直书教学；但 §2.1 的「help / after_help 示例文案」规范块（L75~L96）未同步。impl_plan「自查遗留差异点名」仅豁免 L47/L266（已改）与 L60/L79/L84/L217（历史论证文本），**L78~L86 属规范示例文案，不在豁免清单内**。
- 影响：agent 通读 design.md §2.1 同节即遇两处互斥口径；虽 spec.md 为权威且已修订，但 SSOT 目录内部矛盾违反「冻结面相互一致」纪律。
- 建议：将该 after_help 块换正文直书形态（与 post.rs 现行 after_help 逐字同源），或按 L47 同款裁决更正注处理。

**W-3 任务 #37 验证报告未提交入库，且落点偏离 impl_plan O5 交付路径（仓库未处于完整归档态）**
- 位置：`docs/dev/rulings-verification-2026-08-15.md`（任务 #37 QA 报告，untracked 未提交）。
- 证据：`git status --porcelain` 显示该文件 untracked；impl_plan 步骤 O5 规定 QA Review Book 落 `docs/reviews/v0.6-owner-rulings-review-{日期}.md`，实际落 `docs/dev/` 且命名不同。评审开始时工作区另有任务 #37 残留的 `_review_docs.diff` / `_review_src.diff` / `_review_tests.diff` 三个临时夹具（untracked，.gitignore 的 `_*/` 仅匹配目录不覆盖文件），评审期间已被并行流程清理，此项不再追。
- 影响：QA 报告未入库则有丢失风险，「验证闭环已归档」这一交付前提在 git 史上不可见；推送前工作区非 clean 亦违反原子交付纪律。
- 建议：将验证报告提交入库（路径/命名偏差可随提交说明登记，或按 impl_plan 落点归位至 docs/reviews/）。

### Suggestion（CONSIDER）

**S-1 docs/roles/format-v2-implementer.md L74 含旧文法示例（`--reply-to 1` 与 `--from`/`--to`/位置正文）**
- 该文件为 v0.5 Format Renewal 时代的历史角色剧本，裁决前即已同样过时（含更早已删除的 `--to`），属历史归档而非现行 agent 教学面。建议与 design.md 历史论证文本同口径处置（历史归档、冲突以 spec 为准）并在台账登记一句即可，不必回改。

## 核查通过项（无发现）

1. **写侧撤销的 breaking 披露充分**：CHANGELOG Removed 段明确标注 Breaking、给出逐字迁移路径（正文 `@#N`/`@name` 直书）、声明读侧过滤器保留；实测 send/edit 传入两 flag 均 exit 2 + `fix:` 迁移教学 + canonical example + **零写入**（bdd S-SEND-22/23/S-EDIT-10 口径逐字命中）。
2. **agent 消费面一次走通**：SKILL.md/根 README 的快速上手与 Commands 示例已全部换正文直书形态；按文档示例逐条实测（send 建线程 -> `--message "@#1 On it, @alice"` 回复 -> read 过滤 -> edit）全部一次通过；撤销声明与 advisory 语义说明在场。全仓 grep 教学残留仅剩 W-2/S-1 两处（均已分级）。
3. **read 过滤器保留与文档一致**：post.rs read 臂代码零改动；实测 `post read --reply-to 1` / `--mention alice` 均命中；cli_integration diff 中读侧用例（S-READ-04/06/07 等）无任何 `-` 行。
4. **信封协议只增不改**：`advisory` 仅 contacts add/update 两写点、写入成功后追加；实测三形态文案与 spec §3.6 冻结逐字一致（does not exist / is not readable / is not a valid profile file），Default 与 `--json` 同名 key（JSON 实测 `{"advisory":...}`），触发/不触发、exit 0 恒常均验证；无其他信封字段变动。
5. **冻结面完整**：`repos/paperwork-core/**` 与 `ops_tests.rs` diff 为空（字节级零改动防线成立）；char_tests 重冻登记完整——O3 提交信息登记 4 项变更 + 7 项新增 label 及替换关系，与 tdd §9.3 五类面口径对应；spec/bdd/tdd 对负向清单口径一致（S-SHORT-02 枚举收窄为「post read 两项」、不写死总数、分项枚举，三文档互洽）；flag_inventory_matches_spec 已翻转为 send 侧负向断言且与 bdd 枚举一致；VALUE_TAKING_FLAGS 按 spec §5 第 5 条保留 `--reply-to`/`--mention`（main.rs L300~L301 实证）。
6. **版本纪律合规**：两 crate 均 0.5.0；tag 清单止于 v0.5.0 无新增；CHANGELOG 仅 `[Unreleased]` 增量、既有发布段（含 0.5.0 段对糖衣 flag 的历史记载 L70/L248）不回改，符合「发布段不回改、不新增版本段」纪律。
7. **推送状态合理**：master ahead origin/master 7 提交（d920271 任务 #35 文档提交 + O1~O5 补记共 6 提交），与 impl_plan「提交与推送节奏由编排层统一安排」一致，无越权推送。
8. **worktree/分支**：`wip/v0.5-perfection-snapshot-2026-08-15` 检出在 wt-v05perfection worktree，属另一 agent 在途职责——按指示仅登记事实，不评审不处置。

## 维度结论

**影响面维度：合格（附条件）。** 代码与对外协议层无回归破坏：撤销面的 breaking 已充分披露且迁移教学可操作，读侧/信封/退出码/冻结面全部按契约保留或只增，426 测试全绿复验。残余风险集中在**文档披露一致性**（W-1/W-2）与**仓库卫生/归档**（W-3），均不影响运行时行为。

**可推送/可交付建议**：W-1、W-2、W-3 三项闭合（或经编排层裁定豁免并登记）后可推送；S-1 登记即可。推送本身不引入任何对外契约风险。
