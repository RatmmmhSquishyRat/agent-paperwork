# 文法与命令面矩阵级深审报告（Round 2）

- 任务：#42 深审——文法与命令面（三深审之一）
- 日期：2026-08-15
- 基线：master @ 46b1f47（工作区干净，仅他审计未跟踪报告）
- 二进制：cargo build --release --locked 成功（paperwork 0.5.0）
- 权威依据：docs/ssot/specs/cli-grammar-v0.6/spec.md（含裁决后修订）、同目录 bdd.md、SKILL.md

## 方法

- TEMP 夹具：C:\Users\15480\AppData\Local\Temp\pw-audit-g2（只读仓库，夹具全在 TEMP）
- 探针法：PowerShell ProcessStartInfo.ArgumentList 精确传参，UTF-8 双通道捕获 stdout/stderr/exit code，规避 cp936 误码
- 探针量：212 个探针 / 13 批脚本（s1~s13），全量记录 probes.log（1699 行）；证据锚点 = 探针标签 + probes.log 行号
- 回归：cargo test --release --locked --workspace 全绿 444（7+33+148+16+4+102+12+33+18+71，另 doc-tests 0）

## 结论速览

- 行为偏差：0。全部矩阵格 PASS。
- 观察项：2（G2-O1 / G2-O2，均不违反任何钉住面，见 §8）。
- round-1（audit-grammar-matrix-2026-08-15.md）A-01~A-04 延续登记见 §8。

## 1. 审计面一：命令面穷举矩阵（spec §2 全表 19 签名）

判定口径：正向 = ok 信封结构/字段/退出码符合 spec；负向 = category/退出码/fix/example 符合 spec。证据均在 probes.log。

### 1.1 profile 组（4 签名）

| 矩阵格 | 探针（行号） | 判定 |
|---|---|---|
| create --name 必填 + 可选 --model/--description/--scope-* | PROF-01(L1) ok；PROF-12 scope 多值；PROF-07 edit --model+--scope-read | PASS |
| create 缺 --name -> usage 2 | PROF-03 | PASS |
| create v0.5 位置 NAME -> usage 2 | PROF-04 | PASS |
| create 重复 -> already-exists 1 | PROF-02 | PASS |
| show / ensure_suffix 裸名 / 缺失 | PROF-05 / PROF-06 / PROF-11 not-found 1 | PASS |
| edit 无 flag no-op | PROF-08 exit 0 | PASS |
| list 目录 / 缺目录 | PROF-09 ok / PROF-10 not-found 1 | PASS |

### 1.2 post send（§3.1）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| 基本形态 canonical | SEND-01(L85) ok post.send #1 | PASS |
| 短形式 -a -m 等价 | SEND-02(L93) / SHORT-01(L1412) | PASS |
| 缺 --author -> usage 2 | SEND-03 | PASS |
| 缺正文通道 -> usage 2 | SEND-04 | PASS |
| --message + --stdin 互斥 -> usage 2 | SEND-05 | PASS |
| 仅 --stdin | SEND-06 | PASS |
| 空正文 -> validation 1 | SEND-07 | PASS |
| dash 开头正文经 --message | SEND-08 | PASS |
| equals 形态 --flag=v（含 flag-like 值） | SEND-14 / SEND-15 / SEND-18 | PASS |
| --title 首写生效 / 既有线程静默忽略 | SEND-16 / SEND-17（§3.1 F6 登记一致） | PASS |
| 空 author -> validation 1 | SEND-12(L166) 逐字文案 | PASS |
| 缺 PATH -> usage 2 | SEND-13 | PASS |
| v0.5 位置 NAME BODY / v0.4 --from -> usage 2 教学 | SEND-10(L152) / SEND-11(L159) | PASS |
| 裸 -fix token -> usage 教学指向 --message | SEND-09(L145) | PASS |

### 1.3 post edit（§3.2）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| 正常编辑（自己的/最新/最后一条） | EDIT-01 | PASS |
| 短形式 -a -m | EDIT-SHORT | PASS |
| 缺 --author / 缺 --seq / 缺正文通道 -> usage 2 | EDIT-02 / EDIT-03 / EDIT-04 | PASS |
| --message + --stdin 互斥 -> usage 2 | EDIT-05 | PASS |
| --seq 非数字 -> usage 2 | EDIT-06 | PASS |
| 三重护栏违规 -> not-allowed 1 | EDIT-07（非本人）；ivy_g4 三连（§6） | PASS |
| v0.5 位置参数 -> usage 2 | EDIT-08 | PASS |
| --stdin 编辑 | EDIT-09(L276) | PASS |

### 1.4 post read / summary（§3.3）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| 全量 read（showing + window） | READ-01(L290) | PASS |
| --from / --to 窗口 | READ-02 / EQ-02 / RDO-01(--from 0) | PASS |
| --from/--to 身份值误用 -> usage 2 | READ-03 / READ-08 | PASS |
| --limit N / 非数字 | READ-10 / READ-11(L373) | PASS |
| 缺文件 -> not-found 1 | READ-05 | PASS |
| summary 字段面 / 缺文件 | EDGE-11(L1583) / EDGE-12(L1594) | PASS |

### 1.5 brief 组（5 签名）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| create --title 必填 / 缺失 usage 2 | BRIEF-01(L501) / BRIEF-04 | PASS |
| add --entry 必填 / 缺失 usage 2 / 第二条目 | BRIEF-03 / BRIEF-05 | PASS |
| add 条目文件不存在 -> io 1 | BRIEF-18(L631) | PASS |
| remove --entry-title 必填 / basename 命中 / 全路径键未命中 | BRIEF-16 / BRIEF-17 not-found 1 / BRIEF-06 | PASS |
| v0.5 位置 ENTRY -> usage 2 | BRIEF-07 | PASS |
| read TOC / --full | BRIEF-08 / BRIEF-09 | PASS |
| read --entry-title 命中 / 未命中 not-found / 与 --full 组合 | BRIEF-10 / BRIEF-11 / BRIEF-14 | PASS |
| read --entry-title 空值/全空白 -> validation 1（F1 护栏） | BRIEF-12 / BRIEF-13 逐字文案 | PASS |
| verify [--base-dir] | BRIEF-15(L609) fresh | PASS |

### 1.6 contacts 组（5 签名）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| create [--title] / 默认 title | CON-01(L638) / CON-02 | PASS |
| add --profile 必填 / 缺失 usage 2 canonical | CON-10(L705) / CON-11 v0.5 位置 usage 2 | PASS |
| add 幂等再入 / 正常 add | CON-04 / CON-03（无 advisory） | PASS |
| add 空键 -> validation 1 逐字 | CON-09(L698) | PASS |
| read 富化输出 | CON-12 / UPD-12 | PASS |
| remove 命中 / 未命中 not-found + 键教学 / label 当键 | CON-13 / CON-14 / CON-15 / UPD-02(L779) | PASS |
| remove 缺 --profile -> usage 2 canonical 逐字 | CON-16(L751) | PASS |
| update 命中重派生 label / 缺 flag canonical | UPD-01(L772) / CON-17(L758) / CON-18 | PASS |
| update OLD 未命中 / NEW 已存在 / OLD==NEW | UPD-02 / UPD-03(L786) / UPD-04 | PASS |
| update NEW 不存在静默成功 + advisory | UPD-05(L800) | PASS |
| update 空键两侧 -> validation 1 逐字 | UPD-06(L808) / UPD-07 | PASS |
| 特殊字符键 roundtrip（空格/括号） | SPEC-01(L874) / SPEC-02 / SPEC-03 | PASS |
| remove 末条（solo）/ 空文件 validate / 再删 | SOLO-01(L855) / SOLO-02 / SOLO-03 | PASS |

### 1.7 validate（§3.7）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| 后缀推断 / --type 覆盖 / equals 形态 | VAL-01(L904) / VAL-02 / VAL-06 | PASS |
| 未知后缀无 --type -> format 1 | VAL-03 | PASS |
| --type bogus -> usage 2（ValueEnum） | VAL-04 | PASS |
| S-VAL-06 钉住形态（profile 文件按 post 校验） | VAL-08(L1048) format 1 | PASS |
| 交叉异型（contacts 当 brief / brief 当 contacts） | VAL-09 / VAL-10 format 1 | PASS |
| 缺文件 -> io 1 | VAL-07(L938) | PASS |
| --type contacts 作用于 post 文件 | VAL-05(L928) exit 0——见 §8 G2-O1 | PASS* |

### 1.8 --help / -V / 别名各级

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| 顶层 --help / -h | HELP-01(L1176) / HELP-02 exit 0 不进信封 | PASS |
| 五组组级 --help（含动词清单） | HELP-03/04/06/07/08 | PASS |
| 动词级 help（post send --help） | HELP-05 | PASS |
| -V / --version | HELP-09(L1358) / HELP-10 | PASS |
| 单字母别名 po/p/b/c/v | ALIAS-01(L1368)~ALIAS-05 等价行为 | PASS |

## 2. 审计面二：裁决面复核（2026-08-15 owner 四项裁决）

### 2.1 写侧 --reply-to / --mention 撤销（全写命令）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| send --reply-to -> usage 2 + @#N token 迁移教学 | RUL-12(L487) fix 逐字含 "removed from write commands (owner ruling 2026-08-15)" 与 `--message "@#2 Sure"` 教学 | PASS |
| send --mention -> usage 2 + @name token 教学 | RUL-13(L494) 同构，教学示例 `--message "@carol ping"` | PASS |
| edit --reply-to -> usage 2（example 用 edit canonical） | EDIT-10(L283) | PASS |
| JSON 面 usage 信封（argv 扫描）stdout exit_code 2 | OUT-14(L1035) | PASS |

### 2.2 读侧过滤器冻结保留（post read 全形态）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| --mention 命中 / --reply-to 命中 | RUL-04(L407) / RUL-03(L397) showing 1/1 + window | PASS |
| equals 形态 --mention= / --from= | EQ-01(L1633) / EQ-02 | PASS |
| 零命中宽容（0/0 无 window，exit 0） | EDGE-04(L1541) / EDGE-05 | PASS |
| --reply-to 0 过滤 | RDO-02(L1675) | PASS |
| read 的 --mention/--reply-to 无短形式（-m 拒绝） | READ-04(L333) usage 2 | PASS |
| read 未知 --author -> 教学指向过滤器 | READ-09(L354) | PASS |

### 2.3 正文直书 @ 语义派生一致性（RUL-01~11，读回 RUL-09 汇总核验）

| 语义形态 | 探针 | 派生结果（probes.log L450-468 读回实证） | 判定 |
|---|---|---|---|
| @#N 派生 reply | RUL-02(L388) | #2 reply:#1 | PASS |
| @name 派生 mentions | RUL-02 | #2 mentions:carol | PASS |
| 全角 ＠ 不派生 | RUL-05(L417) | #3 无任何派生注记，原文保留 | PASS |
| @#0（seq 不存在）派生 reply:#0、静默不报错 | RUL-06(L425) | #4 reply:#0，exit 0 | PASS |
| 重复 token 去重 | RUL-07(L433) | #5 mentions:carol（单值） | PASS |
| 混合形态（@#1 @carol @bob，自发者排除） | RUL-08(L441) | #6 sender carol：reply:#1 mentions:bob（@carol 自发排除） | PASS |
| implicit-mention：reply 对象自动派生 | RUL-02 | #2 implicit-mention:alice | PASS |
| implicit-mention 自身边界（回复自己不出字段） | RUL-10(L470) | 无字段 | PASS |
| implicit-mention 显式优先边界 | RUL-11(L478) | implicit-mention:bob | PASS |
| 过滤器与派生结果一致（读侧闭环） | RUL-03/RUL-04 | 同一消息被两过滤器命中 | PASS |

### 2.4 contacts advisory 非阻塞校验（三文案 + JSON 形态）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| 文案一 does not exist（逐字） | CON-06(L674) / CON-05(L666) advisory: destination '<P>' does not exist | PASS |
| 文案二 is not readable（逐字） | CON-08(L690)（目录路径） | PASS |
| 文案三 is not a valid profile file（逐字） | CON-07(L682)（坏 profile 夹具） | PASS |
| 非阻塞：写入完成 + exit 0 + ok 信封携带 advisory 字段 | CON-05~08 全部 exit 0；UPD-05 update 同构 | PASS |
| 有效目标无 advisory | CON-03 | PASS |
| update 面 advisory（NEW 不存在静默成功） | UPD-05(L800) | PASS |
| JSON 形态 advisory key（add / update） | UPD-09(L827) / JSON-01(L894) 字母序 BTreeMap | PASS |
| 空键护栏（add --profile / update 双侧） | CON-09 / UPD-06 / UPD-07 validation 1，message/fix 逐字与 spec §3.6 F1 一致 | PASS |

## 3. 审计面三：输出协议面（Default / --json / --quiet / --plain 四形态）

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| --json 覆盖每组（post/profile/brief/contacts） | OUT-01(L945) send / OUT-04 read / OUT-06 profile list / OUT-08 brief verify / OUT-10 contacts read；单行紧凑 JSON、stdout | PASS |
| JSON key 序 = BTreeMap 字母序（冻结构造路径） | OUT-14(L1035) / UPD-08(L822) / UPD-09 逐键核验 | PASS |
| --quiet 隐首行保字段 | OUT-02(L950)（seq/path/sender 保留，ok 首行隐去）/ OUT-07 brief / OUT-11 contacts | PASS |
| --plain 原始内容 | OUT-03(L957) post read 文件格式字节 / OUT-05 profile show / OUT-09 contacts read | PASS |
| --plain post summary 空输出 exit 0 | EDGE-13(L1601)——冻结行为，见 §8 G2-O2 | PASS* |
| --json + --plain 互斥 -> usage 2 | OUT-12(L1025) | PASS |
| --quiet 不改运行时错误 stderr / JSON 错误 stdout | OUT-13(L1030) / QUIET-ERR(L1692) exit 1 不变 | PASS |
| usage 错误的 --json argv 扫描（解析前判定） | OUT-14 stdout JSON exit_code:2 | PASS |
| --message 值为字面量 "--json" 不误触 JSON 模式 | OUT-15(L1040) Default 信封 | PASS |
| 信封结构：ok <command> <conclusion> + 字段区 + --- body；error <category>: + fix + example | 全探针逐字核验（SEND-01/RUL-09/CON-14 等） | PASS |
| 退出码分层 0 / 1 / 2 | 212 探针全量分布核验 | PASS |

### 3.1 七类 category 现场触达（含 round-1 缺口补齐）

| category | 触发探针 | 判定 |
|---|---|---|
| usage（exit 2） | SEND-03 / NEG 系列 / VTF 系列 | PASS |
| validation（exit 1） | SEND-07 / SEND-12 / CON-09 / BRIEF-12 | PASS |
| format（exit 1） | VAL-03 / VAL-08 / EDGE-09 / EDGE-10 | PASS |
| io（exit 1） | VAL-07(L938) / BRIEF-18(L631)——round-1 未现场触发，本轮补齐 | PASS |
| not-found（exit 1） | READ-05 / CON-14 / UPD-02 / BRIEF-11 | PASS |
| already-exists（exit 1） | PROF-02 / UPD-03 | PASS |
| not-allowed（exit 1） | EDIT-07 | PASS |

## 4. 审计面四：负向清单与边界

| 矩阵格 | 探针 | 判定 |
|---|---|---|
| 未知 flag / 未知动词 / 未知组 | NEG-01 / NEG-02 / NEG-03 usage 2 | PASS |
| 顶层无子命令 / 组级无动词（post、contacts） | NEG-04 / NEG-05 / NEG-06 usage 2 | PASS |
| 多余位置参数（send 双额外） | NEG-07 usage 2 | PASS |
| 空字符串 PATH（send / read） | NEG-08(L1118) / NEG-09 usage 2 | PASS |
| --message= / --author= 空 equals -> validation 1 | NEG-10 / NEG-11 | PASS |
| --seq= 空 equals -> usage 2（类型层） | NEG-12 | PASS |
| -- 分隔符形态 | NEG-13(L1153) | PASS |
| 全局 -q 短形式 / 未知全局 flag | NEG-14 / NEG-15 | PASS |
| author 含空格/括号/制表符 -> validation 1（单 token 校验） | EDGE-01/02/03 | PASS |
| 异型文件写读双向 -> format 1 | EDGE-09 / EDGE-10 | PASS |

## 5. 审计面五：VALUE_TAKING_FLAGS / 短形式 / canonical_example 逐项比对

### 5.1 VALUE_TAKING_FLAGS（main.rs L294-320，25 项）

常量 25 项 = spec §2/§4 全表 23 个带值长 flag（--message --author --seq --reply-to --mention --title --name --model --description --entry --entry-title --profile --new-profile --from --to --type --limit --note --regex --base-dir --scope-read --scope-write --scope-owns）+ 短形式 -a/-m；开关型 --stdin/--full/--json/--plain/-q 不在列。--reply-to/--mention 按裁决口径保留在列（写侧已撤销但读侧合法，且撤销路径在 clap 之前需要值跳过）。

| 值跳过实证探针 | 结果 | 判定 |
|---|---|---|
| VTF-01(L1605) post read x --mention "--json" | Default usage 信封走 stderr，未误入 JSON 模式 | PASS |
| VTF-02 read --reply-to "--json" | 同上 | PASS |
| VTF-03 contacts add --profile "--json" | 同上 | PASS |
| VTF-04 brief add --entry "--json" | 同上 | PASS |
| OUT-15 --message "--json" | Default ok 信封 | PASS |

### 5.2 短形式白名单 {-a, -m, -q}（spec §4，F3 收窄裁定）

- 正向：-a/-m 等价（SEND-02 / SHORT-01(L1412) / EDIT-SHORT）、-q 全局（NEG-14 / SHORT-02）。
- 负向：13 个非白名单短字母（-s -l -n -t -e -p -r -d -o -f -w -b -c）逐一作用于 post read，全部 usage 2（NSHORT 系列，probes.log L1429 起）。
- 作用域：read 的 --mention 拒绝 -m（READ-04），避免 -m 双义，与 §4 钉住一致。
- 测试钉住：short_form_whitelist_is_exact（cli_integration.rs）绿。判定：与 spec §4 完全一致。PASS

### 5.3 canonical_example（main.rs canonical_example()，F2 静态规范形态）

| 命令 | 实测 example 行（探针） | 与 spec/bdd 钉住形态 | 判定 |
|---|---|---|---|
| post send | paperwork post send standup.post.md --author alice --message "Hello"（SEND-09/10/11 L150/157/164） | spec §5 第 2 条逐字；canonical_send_example_matches_spec_52 绿 | PASS |
| post edit | paperwork post edit standup.post.md --author alice --seq 3 --message "corrected body"（EDIT-10 L288） | bdd S-EDIT 钉住 | PASS |
| post read | paperwork post read standup.post.md --from 5 --to 20（READ-11 L378） | spec §5 | PASS |
| contacts add | paperwork contacts add team.contacts.md --profile agents/alice.profile.md（CON-10 L710） | spec §5 | PASS |
| contacts remove | paperwork contacts remove team.contacts.md --profile alice.profile.md（CON-16 L756） | spec §3.6 Ryan m-2 逐字钉住 | PASS |
| contacts update | paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md（CON-17 L763） | spec §3.6 逐字钉住 | PASS |
| brief add | paperwork brief add onboarding.brief.md --entry src/main.rs --regex "fn main"（VTF-04 L1631） | spec §5 | PASS |

全部 example 为静态规范形态，不携带用户原参数值（F2），且纯 ASCII。PASS

## 6. 审计面六：Ivy G1–G5 回填批核验（只读测试源码 + 绿态）

tests/ivy_gap_tests.rs（775 行，16 测试全绿；wip 5bfb061 Plan-C 选择性回填，全部调用点改写为 v0.6 具名文法）：

| 缺口 | 测试 | spec 对应 | 判定 |
|---|---|---|---|
| G1 VAL-04 v0.4 遗留 post（### #N 头族）默认信封逐字 | ivy_g1_validate_v04_legacy_post_default_envelope | S-VAL-04；example 与 §5 canonical 一致（ivy L95 逐字 paperwork post send myfile --author alice --message "hello"） | PASS |
| G2 profile/brief/contacts 异型夹具全信封（4 测试，含 brief 部分迁移残留双变体） | ivy_g2_*（L106-222） | format category 契约 + legacy 迁移教学；contacts legacy 全信封补齐 cli_integration 的子串断言面 | PASS |
| G3 validate --json 错误信封结构（字节级） | ivy_g3_validate_json_error_envelope_structure（L233） | §5 信封继承：status/category/command/message/fix/example/exit_code=1，BTreeMap 字母序，stderr 空 | PASS |
| G4 post edit 三重拒绝 CLI 面 + 拒绝后字节不变（3 测试） | ivy_g4_*（L282-450） | §3.2 三重护栏；补 char_tests 未断言的拒绝字节稳定性 | PASS |
| G5 宽容面与模式组合（8 测试） | ivy_g5_*（L460-774） | 见下细分 | PASS |

G5 细分：过滤器零命中空信封 Default+JSON 双过滤器（S-READ-06 真基线 0/0 口径）；summary 缺文件 not-found（Kim M1 对称护栏，wip 宽容态已撤销）；--quiet/--plain 错误面字节不变；CRLF roundtrip；Unicode（小明/中文标题）Default+JSON roundtrip；注入防线（--title/--model 字面换行拒绝且零写入）；并发首 send（CONC-02 CLI 面，preamble 恰一次、seq {1,2}）。

与 spec 对应关系核验结论：G1–G5 全部场景均可追溯到 v0.6 spec/bdd 条款或既有冻结判例，无孤立回填；v0.6 文法改写无残留 v0.5 位置参数调用（全文仅具名 flag + PATH 位置参数）。PASS

## 7. 测试套件状态与裁决批测试覆盖

cargo test --release --locked --workspace：444 全绿（7 main 单元 + 33 char_tests + 148 cli_integration + 16 ivy_gap + 4 t6 + 102/12/33/18/71 其余 crate）。

裁决批（任务 #36/#37 实施面）测试存在于 cli_integration.rs 并全绿：
- 写侧撤销：post_send_revoked_reply_to_flag_usage_rejected / post_send_revoked_mention_flag_usage_rejected / post_edit_revoked_flags_usage_rejected / revoked_flag_usage_envelopes_are_pure_ascii
- advisory：contacts_add_destination_advisory_nonblocking / contacts_add_valid_destination_no_advisory / contacts_update_destination_advisory_nonblocking
- 短形式与 canonical：short_form_whitelist_is_exact / canonical_send_example_matches_spec_52 / contacts_group_help_lists_verbs / help_short_flags
- JSON 值误触防线：message_value_literal_json_does_not_trigger_json_mode
- brief 选择性详情：brief_read_entry_title_selective_details / brief_read_entry_title_miss_is_not_found / brief_read_entry_title_combines_with_full
- contacts CRUD：contacts_remove_success / contacts_remove_miss_and_label_as_key_are_not_found / contacts_update_success / contacts_update_error_paths / contacts_remove_update_missing_flags_are_usage / contacts_remove_positional_misuse_is_usage / contacts_remove_last_entry_shape / contacts_special_char_path_roundtrip / contacts_update_nonexistent_new_is_silent_success
- 护栏与并发：empty_key_values_are_refused_as_validation / validate_rejects_legacy_contacts / validate_empty_contacts_ok / contacts_add_rejects_legacy_file / multiprocess_concurrent_contacts_brief_add_no_lost_entries / profile_edit_concurrent_disjoint_fields_union

矩阵格与测试覆盖交叉核验：未发现测试未覆盖的命令面格子——19 签名的正向/必填缺失/位置误用/短形态面均有 cli_integration 或 char_tests/ivy_gap 对应钉住；本轮探针为独立第二证据源。

## 8. 偏差清单（G2-xx 分级）

行为偏差（阻塞级）：0。行为偏差（重要级）：0。观察项：2。

| 编号 | 分级 | 内容 | 证据 |
|---|---|---|---|
| G2-O1 | 观察 | validate --type 交叉反向宽容：validate standup.post.md --type contacts 得 exit 0（post 内容按 contacts 解析未报错）。S-VAL-06 钉住面为反向形态（profile 文件 + --type post -> format 1，VAL-08 实测 PASS）；本方向无任何 spec/bdd 钉住或行为承诺，不构成偏差。建议：后续文档修订时登记该宽容口径或补钉住 | probes.log VAL-05(L928) / VAL-08(L1048) |
| G2-O2 | 观察 | --plain post summary 输出为空且 exit 0：机制为 Plain 档 emit_ok 静默（output.rs L79-81）且 summary 无 plain handler；char_tests L1837 post_summary_plain_stdout="" 已字节级钉住，属冻结基线行为，非偏差。若 owner 认为 plain summary 应有降级输出，属未来工作项而非本轮缺陷 | probes.log EDGE-13(L1601)；tests/char_tests.rs |

round-1 发现延续（无变化，不重复编号）：
- A-01（低）v0.6 bdd S-READ-06 转写口径错误（showing 0/4 vs 真基线 0/0）——实测仍 0/0，与真冻结基线一致（EDGE-04 复证）。
- A-02（低）validate 异型 example 措辞 myfile 无后缀 vs bdd S-VAL-04 钉住 myfile.post.md——本轮 ivy_g1 已按实现形态逐字重冻（ivy L95），文档侧与实现侧仍存一词之差。
- A-03（观察）usage fix base 文案泛化（提 --author/--message for post send/edit）复现于 contacts/brief/read 场景（CON-10 L709、READ-11 L377、VTF-03 L1623）——spec 未钉住该句，教学焦点打磨候选，维持观察。
- A-04（观察）init/notify 不存在属任务单范围澄清，本轮复证 NEG-03 冻结组集合未被突破。

## 9. 覆盖率统计与未覆盖声明

- 签名覆盖：spec §2 全表 19/19 = 100%（含任务单未点名的 profile list 与 brief verify）。
- 探针总量：212（PROF 12 / SEND 18 / EDIT 11 / READ 9 / BRIEF 18 / CON 18 / UPD 12 / SOLO 3 / SPEC 3 / JSON 2 / VAL 10 / OUT 15 / NEG 15 / HELP 10 / ALIAS 5 / SHORT 2 / NSHORT 13 / EDGE 13 / VTF 4 / EQ 3 / RDO 2 / RUL 13 / QUIET 1）。
- 裁决面：四项裁决 100% 实测（写侧撤销 3 命令面 + JSON 面；读侧过滤器 6 形态；@ 语义 10 形态；advisory 三文案 + JSON + 空键护栏）。
- 输出协议：四形态 x 五组抽样全覆盖；七类 category 全部现场触达（io 类本轮补齐 round-1 缺口）；退出码 0/1/2 全覆盖。
- 未覆盖声明：--help 全文逐行 ASCII 仅抽查（char_tests all_help_output_is_pure_ascii / ascii_output_contract_guard 已钉住全量面）；io 类锁竞争现场压测不在本审计面（任务 #43 职责），以 ivy_g5_concurrent_first_send_cli_contention 与 multiprocess 测试绿态为据。

## 10. 结论

实现与 v0.6 spec（含 2026-08-15 裁决后修订）在命令面、裁决面、输出协议面、负向边界、VTF/短形式/canonical 细节上全矩阵一致；行为偏差 0，观察项 2 项均不触碰钉住面；444 测试全绿且裁决批测试覆盖完备。文法与命令面维度达到可发布质量。

（完。证据库：C:\Users\15480\AppData\Local\Temp\pw-audit-g2\probes.log，1699 行）
