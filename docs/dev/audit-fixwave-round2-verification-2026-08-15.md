# 审计修复波 Round2 复验报告（2026-08-15，任务 #46）

- 验证对象：全量审核补齐修复波 F1~F4（docs/dev/audit-fixwave-round2-execution-log-2026-08-15.md），master @ 04024a8。
- 基线：本地 master = origin/master = `04024a8897d3803c44646321387f3441294bedf6`（fetch 后逐字核对），验证开始时工作区干净；四个修复提交 54beff3 / 0ffd9d2 / 0dc23f0 / 04024a8 在案。
- 纪律：只运行与取证，未改任何源代码/测试（本报告落盘除外）；未触碰 wip 分支。

## 总判定表

| # | 检查项 | 判定 |
|---|--------|------|
| 1 | 冷重建回归（build/test/clippy/fmt/docs gate） | **FAIL（docs gate）** |
| 2 | 修复项实测（R2-01 / B-x 测试 / smoke.ps1 / FR-1 / B-8 注记） | PASS |
| 3 | 冻结面（信封/category/退出码/ops_tests/黄金快照） | PASS |
| 4 | 文档面抽查（台账十六节/spec README/roles/format-v2 OQ-4） | PASS |
| 5 | 卫生核验（分支清理/_wip_stage/wip 保全） | PASS |
| 6 | 版本纪律 | PASS |
| 7 | 线上 CI run 状态 | **FAIL（run 31883911085，test×3 failure）** |

**最终结论：不放行。** 451 测试全绿属实，但线上 CI（run 31883911085）在 test×3 的 Docs step 全数失败，根因为 F1 修复波 R2-01 引入的 rustdoc 缺陷（error.rs:11 公有常量文档链接到 pub(crate) 私项），且该缺陷已离线复现。修复波执行日志的终局门禁清单漏跑 docs gate（ci.yml Docs step：`RUSTDOCFLAGS=-D warnings cargo doc`），属门禁覆盖缺口。其余各项全部通过；修复该单一缺陷后预期即可放行。

## 1. 冷重建回归

| 门禁 | 结果 |
|------|------|
| `cargo clean` | 移除 5203 文件 / 1.3GiB |
| `cargo build --release --locked` | exit 0 |
| `cargo test --workspace --locked` | **451 全绿**：7 + 33 + 154 + 16 + 4 + 103 + 12 + 33 + 18 + 71 + 0，与执行日志分布逐位一致（cli_integration 148→154、core unit 102→103 为 F1/F2 增量） |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| docs gate（`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`） | **FAIL（exit 101）**，详见下 |

docs gate 失败证据（`cargo clean -p paperwork-core` 后强制复跑，与线上日志逐字同源）：

```
error: public documentation for `FILE_NOT_UTF8_FIX` links to private item `PaperworkError::io_ctx_file_read`
  --> repos/paperwork-core/src/error.rs:11:28
   = note: `-D rustdoc::private-intra-doc-links` implied by `-D warnings`
error: could not document `paperwork-core`
```

取证诚实性登记：本验证早期一轮 docs gate 曾得 exit 0——系 target/doc 增量指纹缓存导致 paperwork-core 未重新 rustdoc；`cargo clean -p paperwork-core` 强制重文档化后稳定复现 exit 101，与线上失败一致。以复现结果为准。

根因定位：error.rs L9-14 为 F1（54beff3）新增的公有常量 `FILE_NOT_UTF8_FIX`，其文档注释 L11 使用 intra-doc link `[`PaperworkError::io_ctx_file_read`]` 指向同批新增的 `pub(crate)` 函数——rustdoc 的 private-intra-doc-links lint 在 -D warnings 下升级为硬错误。修复方向（供后续修复任务，不在本验证范围内动手）：将链接改为代码格式反引号引用或去掉链接形式，不改任何行为面。

过程发现（登记）：修复波执行日志「终局门禁」节仅列 test / clippy / fmt 三项，未包含 ci.yml test job 的 Docs step——门禁清单与 ci.yml 不对齐是该缺陷漏网的系统性原因，建议把「docs gate」固化为修复波门禁必填项（与此前任务 #39 的 CI 内嵌脚本教训同类）。

## 2. 修复项实测 — PASS

### R2-01（release 二进制现场实测）

| 用例 | 实测 | 判定 |
|------|------|------|
| UTF-16 LE 文件 post read（default） | exit 1，`error io: ... stream did not contain valid UTF-8`，fix 逐字 = `the file is not valid UTF-8; check that the file is UTF-8 encoded (binary and UTF-16 files are not supported)` | PASS |
| 同文件 `--json` 档 | `"category":"io","exit_code":1` + 同 fix，信封结构不变 | PASS |
| validate / post summary / contacts read 同文件 | 同 io 信封同文案（validate 带 example 面） | PASS |
| 二进制文件 post read | 同路径同文案 | PASS |
| 对照组：合法 UTF-8 文件 read/validate | exit 0 正常，不受影响 | PASS |

category 维持 io、exit code 维持 1、信封结构未变——「只增不改」口径成立（行为面）。

### B-x 盲区测试（源码级断言强度核验，cli_integration.rs L5326-5589）

| 测试 | 断言强度核验 | 判定 |
|------|--------------|------|
| B-1 `bom_prefixed_thread_is_tolerated_on_read_and_validate` | 真实前置 BOM 字节（EF BB BF）写盘；read 断言消息计数+正文内容；validate code 0 | 非弱 |
| B-2 `utf16_file_read_fast_fails_with_encoding_pointing_fix` | UTF-16 LE+BOM 构造；code 1 + `error io:` + 新文案逐字断言（default/validate/JSON category=io 三面）+ **字节级零写入断言**（`assert_eq!` 全文件） | 非弱 |
| B-2 伴生 `binary_file_read_fast_fails_with_encoding_pointing_fix` | 0xFF/NUL/非法 UTF-8 序列构造；code 1 + io + 编码文案断言 | 非弱 |
| B-5 `reserved_device_names_are_sealed_by_suffix_normalization` | CON/NUL 双名循环；30s timeout 防设备挂起；send 落 `<NAME>.post.md` 普通文件存在断言 + 裸路径读回断言；validate 裸名 unknown file type code 1 | 非弱 |
| B-6 `large_thread_2500_messages_send_read_roundtrip` | 2500 消息真实构造；showing 20/2500、validate 全文、send seq 2501、head #1 与 tail #2501 双向读回断言 | 非弱 |
| B-8 `h1_leniency_missing_and_duplicate_h1_read_cleanly` | 缺 H1 与双 H1 双夹具；read 消息计数 + validate code 0 双面 | 非弱 |

实测：6 项全部包含在 154 个 cli_integration 通过项中（451 全绿覆盖）。

### _e2e/smoke.ps1 复跑 + FR-1 复核

- `_e2e/smoke.ps1` 逐字复跑：**exit 0 全断言命中**；L38 修复面现场确认——正文直书 `@#1 Tests merged. cc @alice`，read 回显 `reply:#1 mentions:alice` 派生正确。
- FR-1 全仓 grep（`--reply-to`/`--mention`）复核：
  - `.github/workflows/ci.yml`：仅 2 处裁决说明注释（L82/L184），零写侧调用。
  - `SKILL.md` / 根 `README.md`：命中均为读侧过滤器示例与撤销声明文本（豁免面，甄别正确）。
  - `_e2e/smoke.ps1`：仅 L38 裁决注释 1 处；`repos/paperwork-cli/README.md`：零命中。
  - 结论：写侧糖标志教学面零残留，与执行日志声明一致。

### B-8 三处注记一致性

- spec.md §3.3（L131）与 §3.7（L195）、design.md §8（L221）、bdd.md S-READ-10（L255-259）三处注记均在位，口径逐条一致：读侧/validate 侧 H1 非强制、写侧首写仍写 H1、读写不对称为刻意保留、钉住现行行为不改行为；互相引用关系闭合（spec↔bdd↔design↔测试函数名）。

## 3. 冻结面 — PASS

| 冻结项 | 证据 | 判定 |
|--------|------|------|
| ops_tests 71 项零改动 | `git diff 46b1f47..HEAD -- repos/paperwork-core/tests/ops_tests.rs` 为空；71 全绿 | PASS |
| 信封/category/退出码 | error.rs diff 仅新增常量与 pub(crate) 构造函数，category 映射行（`=> "..."`）零改动；R2-01 实测 category=io / exit 1 不变 | PASS |
| 黄金快照 | char_tests.rs 整文件 diff 为空（未重冻）；内嵌黄金表仍恰 150 条 | PASS |
| 变更面范围 | 46b1f47..HEAD 代码变更仅 error.rs（+64）、ops/* 调用点迁移（每文件 2~6 行）、cli post.rs/validate.rs 内联引用、cli_integration +269 行新测试；无越界 | PASS |

## 4. 文档面抽查 — PASS

| 项 | 证据 | 判定 |
|----|------|------|
| 台账第十六节零开放终态 | open-items-ledger L482-516：LED-06/07/08 闭合销账表（含 INV 编号与闭合证据）、LED-18~23 登记、「登记总数 23、仍开放项为零」声明在位；非闭合保留项（LED-15/16/19 及 LED-18/20~23）与各项实际状态一致 | PASS |
| spec README 勾选 | cli-grammar-v0.6/README.md L64：`[x] owner 四项裁决落盘…`（含提交链与 444 承载声明） | PASS |
| roles 归档标注 | 两份 implementer role 头部均有「历史归档声明（2026-08-15，任务 #45 修复波 F3 / S2-04）」+ 糖标志撤销冲突指针（以 cli-grammar-v0.6 spec 与 owner-rulings 为准） | PASS |
| format-v2 OQ-4 指针 | §5.7 与 OQ-4 均带「2026-08-15 裁决指针注记（任务 #45 F3 / S2-03）」：写侧糖标志全面移除、正文 token 直书、权威口径指向 cli-grammar-v0.6 spec | PASS |

## 5. 卫生核验 — PASS

| 项 | 证据 | 判定 |
|----|------|------|
| cli-grammar-v0.6 分支清理 | `git branch -a` 仅 master + wip/v0.5-perfection-snapshot-2026-08-15（本地与 origin 双侧），无 cli-grammar-v0.6 | PASS |
| _wip_stage 删除 | `Test-Path _wip_stage` = False | PASS |
| wip 分支保全 | 本地与 origin tip 均 `9d63d3b715b332bc84217a1b05f59ad9670cf8e6`，与执行日志登记一致，零触碰 | PASS |
| worktree | `git worktree list` 仅主工作区（master @ 04024a8），无 wt-v05perfection 接触面 | PASS |

## 6. 版本纪律 — PASS

- 双 crate `version = "0.5.0"` 未 bump。
- `git ls-remote --tags origin`：仅 v0.2.0 / v0.3.0 / v0.4.0 / v0.5.0，无新 tag。
- CHANGELOG 无新发布段（本轮为缺陷修复波，依口径不入发布段）。

## 7. 线上 CI run — FAIL

- run **31883911085**（CI，push/master，commit 04024a8）：status=completed，**conclusion=failure**。
- 逐 job：fmt=success；**test (ubuntu/macos/windows)=failure ×3**；smoke=skipped（needs: test 未满足）。
- 三平台失败日志同因（`--log-failed` 取证）：均死于 Docs step 的 `error: public documentation for FILE_NOT_UTF8_FIX links to private item PaperworkError::io_ctx_file_read`（error.rs:11），exit code 101。
- 与声称差异：修复波执行日志未声称「线上全绿」，但其终局门禁缺 docs gate 导致该缺陷未在线下拦截；线上红灯即该缺口的直接后果。

## 不放行理由与修复建议

1. **唯一阻塞项**：error.rs L11 intra-doc link 指向私项，docs gate 线下复现 + 线上三平台同失败。修复面极小（文档注释一行：链接改反引号代码形式或移除链接），不改任何行为面，预计修复后 test×3 转绿、smoke 恢复执行。
2. **过程建议**（登记不改码）：修复波终局门禁清单应与 ci.yml job/step 对齐（test/clippy/fmt/**docs gate** 四项缺一不可）；与此前任务 #39 CI 内嵌脚本教训同类，建议并入 workflow-and-todo 验证门禁条目。
3. 其余五大项（修复项实测、冻结面、文档面、卫生、版本纪律）全部 PASS，451 全绿独立复现——修复波主体工作质量成立，仅差 docs gate 单点闭环。
