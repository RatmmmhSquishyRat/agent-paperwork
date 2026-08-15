# 深审 A：文法矩阵与输出协议一致性实测报告

- 日期：2026-08-15
- 基线：master @ 3829fd9（v0.6 具名文法 + contacts CRUD + 写路径锁统一，288 测试全绿）
- 被测物：target/release/paperwork.exe（cargo build --release；paperwork 0.5.0——spec §7.4 本轮不 bump 版本，一致）
- 权威基准：docs/ssot/specs/cli-grammar-v0.6/{spec.md,bdd.md,tdd.md,impl_plan.md}；冻结基线 docs/ssot/specs/cli-ux-redesign/（v0.5）
- 实测沙盒：A:\.audit（主工作区根下 subst 映射的临时目录，repos/ 之外；未改任何源代码、未做 git 提交）

## 结论摘要

117 个矩阵判定点（正向 50 / 负向与边界 41 / 无短形式负向 26）全部与 spec/bdd 一致，**行为面零偏差**。发现 2 项文档级措辞问题（A-01/A-02，均「低」）与 2 项观察项（A-03/A-04）。阻塞 0，重要 0。

## 1. 命令 × flag 正向矩阵（happy path，全部 exit 0）

### 1.1 post 组

| # | 调用 / flag | 期望（spec/bdd） | 实测摘要 | 判定 |
|---|---|---|---|---|
| P01 | post send t1 --author alice --message "Hello" | 自动建线程；ok post.send #1 -> t1.post.md；seq/path/sender（S-SEND-01/03） | 逐字一致；t1.post.md 落盘 | PASS |
| P02 | -a alice -m "short form" | 短形式与全称逐字等价（S-SEND-02/S-SHORT-01） | ok post.send #3，sender: alice | PASS |
| P03 | --reply-to 2 | implicit-mention: alice（S-SEND-04）；正文首行 @#2 @alice 糖衣注入（S-SEND-20） | 字段 implicit-mention: alice；落盘正文首行 @#2 @alice | PASS |
| P04 | --mention carol | @carol token 注入正文（S-SEND-20） | 正文首行 @carol，空行后原正文 | PASS |
| P05 | 既有线程附 --title | 静默忽略，exit 0，标题不变（S-SEND-17） | exit 0；summary title 仍为 t1 | PASS |
| P06 | 新线程 + --title "Daily Standup" | preamble 仅 H1，无属性行（S-SEND-21） | 文件以 # Daily Standup 起始，直接接 ## #1 | PASS |
| P07 | --stdin | 正文逐字 = stdin（S-SEND-08） | 落盘 multi-line body 一致 | PASS |
| P08 | --message "-fix flag text" | flag 值直传无需 --（S-SEND-10，allow_hyphen_values） | 逐字写入 | PASS |
| P09 | --message "--json" | 值不触发 JSON 模式（VALUE_TAKING_FLAGS 职责） | 默认信封 ok post.send，非 JSON | PASS |
| P10 | read 默认 | showing + window 恒显（S-READ-01） | showing: 10/10；window: #1-#10 | PASS |
| P11 | read --from 2 --to 3 | seq 范围（S-READ-02） | showing: 2/2；window: #2-#3 | PASS |
| P12 | read --mention carol | mention 过滤 | window: #5-#5，行内 mentions:carol | PASS |
| P13 | read --reply-to 2 | reply 过滤 | window: #4-#4，行内 reply:#2 mentions:alice | PASS |
| P14 | read --limit 2 | 尾窗（v0.5 S-READ-02 冻结） | showing: 2/10；window: #9-#10 | PASS |
| P15 | read --mention zed（零命中） | showing 0/0、无 window 字段（v0.5 bdd S-READ-06 冻结口径） | showing: 0/0，无 window；注：v0.6 bdd 措辞有误，见 A-01 | PASS |
| P16 | summary | title/participants/messages/last.*（S-SUM-01） | 全部字段齐备，participants: alice, bob | PASS |
| P17 | edit --author bob --seq 1 --message "edited" | ok post.edit #1（S-EDIT-01） | 一致 | PASS |
| P18 | edit 非本人消息 | not-allowed exit 1；message 含 sent by 'bob', not 'alice'（S-EDIT-07） | 逐字一致；example 为 v0.6 形态 | PASS |
| P19 | edit --stdin | 正文逐字 = stdin（S-EDIT-09） | 一致 | PASS |

### 1.2 profile 组

| # | 调用 / flag | 期望 | 实测摘要 | 判定 |
|---|---|---|---|---|
| P20 | profile create alice --name alice --model gpt-4o --description "..." | ok profile.create；name 字段（S-PROF-01） | 一致 | PASS |
| P21 | create --scope-read/--scope-write/--scope-owns | 三 scope 写入 | show 显示 scope.read/write/owns | PASS |
| P22 | profile show | ok 信封 + 字段（S-PROF-05） | name/model/description 齐备 | PASS |
| P23 | profile edit --model --description | changed: model, description | 一致 | PASS |
| P24 | profile list <DIR> | ok profile.list N profiles | 3 profiles，逐条 name (model) | PASS |
| P25 | 重复 create | already-exists exit 1（S-PROF-04） | 一致，fix 引导 profile edit | PASS |

### 1.3 brief 组

| # | 调用 / flag | 期望 | 实测摘要 | 判定 |
|---|---|---|---|---|
| P26 | brief create --title --owner | ok brief.create（S-BRIEF-01） | 一致 | PASS |
| P27 | brief add --entry --regex --note | ok brief.add <entry> -> <brief>；hash 快照（S-BRIEF-02） | 一致，--full 可见 hash/regex/note | PASS |
| P28 | brief read 默认 TOC | 首行 ok brief.read N entries（conclusion 全量条目数口径） | ok brief.read 2 entries | PASS |
| P29 | brief read --full | 条目 path/hash/regex/note | 一致 | PASS |
| P30 | brief read --entry-title f1.txt | 仅该条目 --full 档字段；conclusion 仍为全量 N entries（S-BRIEF-07） | 首行 ok brief.read 2 entries，仅 f1 详情 | PASS |
| P31 | --json + --entry-title | entries 仅含命中条目且含 path/hash/regex/note，不受 --full 门控（Daniel m-4） | 一致 | PASS |
| P32 | --full --entry-title 组合 | 等价单条目详情，无冲突（S-BRIEF-09） | 一致 | PASS |
| P33 | --json 默认档 vs --full 档 | 默认档 entries 含 hash/path/title；--full 增 regex/note（冻结字段面） | 一致 | PASS |
| P34 | brief remove --entry-title f2.txt | ok brief.remove；removed 字段 | 一致 | PASS |
| P35 | brief verify / verify --base-dir | N/M fresh 三态（S-BRIEF-06） | 两形态均 1/1 fresh | PASS |

### 1.4 contacts 组

| # | 调用 / flag | 期望 | 实测摘要 | 判定 |
|---|---|---|---|---|
| P36 | contacts create --title / 缺省 | title Core Team / 默认 Contacts（S-CONTACTS-01） | 两形态一致 | PASS |
| P37 | contacts add --profile | ok contacts.add <profile> -> <contacts>；contacts/profile 字段（S-CONTACTS-02） | 一致 | PASS |
| P38 | 已存在条目再 add | exit 0 幂等 no-op（S-CONTACTS-11） | 一致 | PASS |
| P39 | contacts read | 富化 <path>: <name> (<description>)（S-CONTACTS-05） | 一致 | PASS |
| P40 | contacts update --profile OLD --new-profile NEW | 首行 ok contacts.update <OLD> -> <NEW>；updated 箭头串逐字；顺序保留；label 依 R11 重派生（S-CONTACTS-08） | updated: alice.profile.md -> carol.profile.md；文件 [carol](carol.profile.md) 原位替换，bob 位置不变 | PASS |
| P41 | update 到不存在 NEW | exit 0 静默写入；label 回退文件名主干（S-CONTACTS-14 钉住静默面） | updated: bob.profile.md -> dave；落盘 [dave](dave) | PASS |
| P42 | contacts remove --profile | 首行 ok contacts.remove <profile> -> <contacts>；contacts/removed 字段（S-CONTACTS-06） | 一致 | PASS |
| P43 | remove 最后一条目 | 文件 = title H1 + 空行；validate 合法；再 remove 同键 not-found（S-CONTACTS-12） | 三项全中 | PASS |
| P44 | 特殊字符路径 add/update/remove 往返 | 键 = 未转义原串命中；新路径 angle-bracket 序列化（S-CONTACTS-13） | 三步全命中，validate 合法 | PASS |

### 1.5 validate 与横切

| # | 调用 / flag | 期望 | 实测摘要 | 判定 |
|---|---|---|---|---|
| P45 | validate 后缀推断 / --type 覆盖 | exit 0（S-VAL-01/02） | 两形态 ok validate | PASS |
| P46 | --json ok 信封（各命令） | status:"ok" + command + conclusion + 命令特有 key（S-OUT-01；新字段 contacts/removed/updated 只增） | post.send/read/edit、profile.show、brief.read、contacts.update/remove 全符合 | PASS |
| P47 | -q（全局） | 隐 ok 首行、字段与 body 保留（冻结） | 一致 | PASS |
| P48 | --plain | 原始字节输出 | 直接输出文件原内容 | PASS |
| P49 | 隐藏别名 po/c/v | 可用且不出现于 help 命令列表（S-ALIAS-01/S-SHORT-02） | 全 exit 0；help 无别名 | PASS |
| P50 | ASCII 输出契约 | usage 与运行时错误 stderr 全 ASCII（S-OUT-05） | 5 类错误信封字节级抽查 non-ascii=0 | PASS |

## 2. 无短形式负向清单（26 项，bdd S-SHORT-02）

逐项以首字母类短形式探针实测，期望一律 usage exit 2；实测 26/26 全为 exit 2（error usage: unexpected argument），且零文件写入（探针产物 z* 文件检查为空）。

| # | flag | 探针 | 实测 | 判定 |
|---|---|---|---|---|
| N01 | --seq | post edit ... -s 1 | exit 2 | PASS |
| N02 | --stdin | post send ... -s | exit 2 | PASS |
| N03 | --title | brief create ... -t T | exit 2 | PASS |
| N04 | --to | post read ... -t 3 | exit 2 | PASS |
| N05 | --from | post read ... -f 1 | exit 2 | PASS |
| N06 | --entry | brief add ... -e f1.txt | exit 2 | PASS |
| N07 | --entry-title | brief remove ... -e f1.txt | exit 2 | PASS |
| N08 | --profile | contacts add ... -p x | exit 2 | PASS |
| N09 | --new-profile | contacts update ... -n x | exit 2 | PASS |
| N10 | --name | profile create ... -n zed | exit 2 | PASS |
| N11 | --model | profile create ... -m gpt | exit 2 | PASS |
| N12 | --description | profile create ... -d D | exit 2 | PASS |
| N13 | --owner | brief create ... -o alice | exit 2 | PASS |
| N14 | --note | brief add ... -n note | exit 2 | PASS |
| N15 | --regex | brief add ... -r re | exit 2 | PASS |
| N16 | --scope-read | profile create ... -s src | exit 2 | PASS |
| N17 | --scope-write | profile edit ... -w docs | exit 2 | PASS |
| N18 | --scope-owns | profile create ... -o repo | exit 2 | PASS |
| N19 | --full | brief read ... -f | exit 2 | PASS |
| N20 | --limit | post read ... -l 5 | exit 2 | PASS |
| N21 | --base-dir | brief verify ... -b . | exit 2 | PASS |
| N22 | --type | validate ... -t post | exit 2 | PASS |
| N23 | --json | paperwork -j ... | exit 2 | PASS |
| N24 | --plain | paperwork -p ... | exit 2 | PASS |
| N25 | --reply-to | post send ... -r 1 | exit 2 | PASS |
| N26 | --mention（read） | post read ... -m alice | exit 2（S-READ-04） | PASS |

正向短形式集合核验：实测仅 {-a, -m, -q} 三项可用（P02/P47），与 spec §4 F3 收窄裁定精确一致；且 -m 仅在 post send/edit 存在（N11 探针证明 profile create 中 -m 被拒），逐命令作用域正确。

## 3. 负向与边界矩阵（usage exit 2 / 运行时 exit 1）

| # | 形态 | 期望（spec/bdd） | 实测摘要 | 判定 |
|---|---|---|---|---|
| U01 | send 缺 --author | usage exit 2；example=send 规范示例（S-SEND-05） | 一致；example 逐字=spec §5.2 钉住串 | PASS |
| U02 | send 缺正文通道 | usage exit 2（required_unless_present，F2） | 一致，单一 --message 形态示例（F5） | PASS |
| U03 | --message+--stdin 同给 | usage exit 2（conflicts，层级提升 spec §5.1） | 一致 | PASS |
| U04 | v0.5 位置文法 send x alice "Hi" | 多余位置参数 usage（S-SEND-12） | unexpected argument 'alice' | PASS |
| U05 | send 缺 PATH | usage（S-SEND-19） | required <PATH> | PASS |
| U06 | send --from alice | 未知 flag + 旧文法教学（S-SEND-13） | 一致 | PASS |
| U07 | 裸 -fix token | fix 引导 --message 形态（S-SEND-11） | fix 含 --message "-fix flag text" 示范 | PASS |
| U08 | --seq abc | u64 类型 usage（S-EDIT-06） | invalid digit；example 含 --seq 合法形态 | PASS |
| U09/U10 | read --from alice / --to bob | u64 类型防线（S-READ-03/08） | 均 usage exit 2，example 示范 seq 范围 | PASS |
| U11 | read --author alice | fix 点名 --mention 替代（S-READ-09） | 一致 | PASS |
| U12 | profile create 缺 --name / v0.5 位置 | usage；example 含 --name（S-PROF-02/03） | 一致 | PASS |
| U13 | brief create/add/remove 缺必填 | example 分别含 --title/--entry/--entry-title（S-BRIEF-03） | 三条全中 | PASS |
| U14 | brief add v0.5 位置文法 | usage（S-BRIEF-04） | 一致 | PASS |
| U15 | contacts add 缺 --profile / v0.5 位置 | usage（S-CONTACTS-03/04） | 一致 | PASS |
| U16 | contacts remove 缺 --profile | example 逐字=钉住串（S-CONTACTS-10） | paperwork contacts remove team.contacts.md --profile alice.profile.md 逐字 | PASS |
| U17 | contacts update 缺全部 / 缺 --new-profile | example 逐字=钉住串（两形态同一示例） | paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md 逐字 | PASS |
| U18 | 未知后缀 validate | format exit 1（S-VAL-03） | fix 引导 --type | PASS |
| U19 | --type bogus | usage exit 2（S-VAL-05） | possible values 枚举 | PASS |
| U20 | garbage.post.md | format exit 1（S-VAL-04） | 行为一致；example 措辞见 A-02 | PASS |
| U21 | --json --plain 同给 | JSON 错误信封 category usage exit_code 2（S-OUT-06） | 一致 | PASS |
| U22 | --json 下 usage 错误 | 单行 JSON，command/example/exit_code:2（S-OUT-03） | 一致，argv 扫描感知 | PASS |
| U23 | 顶层缺子命令 | exit 2；组集合枚举（v0.5 冻结） | 枚举精确={profile,post,brief,contacts,validate} | PASS |
| U24 | --message "   " 空正文 | validation exit 1（S-SEND-09） | message body is empty；零写入 | PASS |
| U25 | --author "   " | validation exit 1（S-SEND-18） | sender name (--author) is empty | PASS |
| U26 | read 不存在线程 | not-found exit 1；fix/example 引导 send 建线程（S-READ-05） | 一致，v0.6 形态 example | PASS |
| U27 | brief read --entry-title 无匹配 | not-found；fix 引导 brief read（S-BRIEF-08） | 一致 | PASS |
| U28 | --entry-title "" / 全空白 | validation；message/fix/example 逐字钉住（S-BRIEF-10） | entry title (--entry-title) is empty / provide a non-empty --entry-title value / example 逐字，三串全中 | PASS |
| U29 | brief remove 未命中 | not-found exit 1 | 一致 | PASS |
| U30 | update OLD 未命中 | not-found + 键口径教学句逐字（S-CONTACTS-09） | the key is the profile path as stored in the contacts file, not the label 逐字 | PASS |
| U31 | update NEW 已存在 | already-exists；fix 引导先 remove | 一致 | PASS |
| U32 | remove 未命中 / label-as-key | not-found + 教学句（S-CONTACTS-07 含 And 段） | 两形态一致 | PASS |
| U33 | add --profile "" | validation 逐字；零写入；先于存在性判定（S-CONTACTS-15） | profile path (--profile) is empty；文件 hash 前后不变；缺文件亦落 validation | PASS |
| U34 | update --new-profile 空白 | validation 逐字；零写入 | new profile path (--new-profile) is empty；hash 不变 | PASS |
| U35 | paperwork init | 组集合冻结无 init（spec §7.5） | unrecognized subcommand，exit 2；见 A-04 | PASS |
| U36 | JSON 运行时错误信封 | status/category/command/exit_code:1（S-OUT-02） | 一致（not-found 探针） | PASS |

注：N26 与 read -m 负向为同一判定点，未重复计数；edit 三重护栏（P18）计入正向。

## 4. 输出协议逐字段对照（--json / 默认信封）

| 命令 | 实测 JSON key 集 | spec/bdd 口径 | 判定 |
|---|---|---|---|
| post.send | status,command,conclusion,path,sender,seq | S-OUT-01 既有 key 只增不改不删 | PASS |
| post.read | +messages[](body/mentions/reply_to/sender/seq/timestamp),showing,window | 冻结字段面 | PASS |
| post.edit | status,command,conclusion,path,seq | 冻结 | PASS |
| profile.show | status,command,conclusion,name,model,description | 冻结 | PASS |
| brief.read | status,command,conclusion,title,owner,entries[]；默认档含 hash/path/title，--full 增 regex/note，--entry-title 命中条目直出全字段 | §3.5 字段面口径（Daniel m-4） | PASS |
| contacts.update | status,command=contacts.update,conclusion,contacts,updated | updated 值=alice.profile.md -> carol.profile.md（<OLD> -> <NEW> 单空格三段，spec §3.6 Ryan m-4 逐字） | PASS |
| contacts.remove | status,command=contacts.remove,conclusion,contacts,removed | spec §7.5 新增字段 | PASS |
| 错误信封（运行时） | status:"error",category,command,message,fix,example,exit_code:1 | S-OUT-02 | PASS |
| 错误信封（usage+--json） | 同上，exit_code:2，example 为 v0.6 静态规范示例 | S-OUT-03 | PASS |
| 默认档 updated 箭头 | updated: alice.profile.md -> carol.profile.md；ok 首行 ok contacts.update <OLD> -> <NEW> 同构 | §3.6 两载体同一信息 | PASS |
| --quiet | 隐 ok 首行，字段区保留 | 冻结 | PASS |

注：任务单提到的 command_id 在实际实现与 spec 中的 key 名为 command（spec §7.1「command 标识」），实测与 spec 一致。

## 5. help 面与钉住文本对照（spec L225 附近 §5.2）

| 项 | 期望 | 实测 | 判定 |
|---|---|---|---|
| paperwork help / --help | exit 0；Commands 精确 {profile,post,brief,contacts,validate}(+help)；隐藏别名不出现（S-SHORT-02） | 一致 | PASS |
| Grammar 行 | paperwork [global flags] <group> <verb> <PATH> --required-flag ... [--optional-flag ...]（必填出方括号，Pete N6） | 逐字一致 | PASS |
| contacts --help 动词集 | 精确 {create,add,remove,update,read}，无 edit（Daniel M-3 新建断言面） | 一致，且带 contacts has no edit verb 注记 | PASS |
| post --help 动词集 | {send,read,summary,edit}，无 create（format-v2 删除） | 一致 | PASS |
| send 规范示例（usage 信封） | paperwork post send standup.post.md --author alice --message "Hello"（spec §5.2 逐字） | 逐字一致（U01/U02/U03 三形态） | PASS |
| contacts remove 规范示例 | paperwork contacts remove team.contacts.md --profile alice.profile.md（§5.2 逐字钉住） | usage 信封与 contacts remove --help Examples 两处逐字一致 | PASS |
| contacts update 规范示例 | paperwork contacts update team.contacts.md --profile alice.profile.md --new-profile carol.profile.md（§5.2 逐字钉住） | usage 信封两形态逐字一致 | PASS |
| not-found example 形态 | remove/update 未命中均为 paperwork contacts read <PATH>（PATH 取用户所给） | 一致（team/solo 两路径实测） | PASS |
| --help/-V 穿透 | exit 0 不进信封（S-OUT-04 冻结） | 各层 --help 与 -V 均 exit 0 | PASS |

## 6. VALUE_TAKING_FLAGS 一致性（main.rs L281-307）

常量 25 项 = spec §2/§4 全表全部 23 个带值长 flag（--message --author --seq --reply-to --mention --title --name --model --description --entry --entry-title --profile --new-profile --from --to --type --limit --note --regex --base-dir --scope-read --scope-write --scope-owns）+ 短形式 -a/-m；开关型 --stdin/--full/--json/--plain/-q 正确不在列。职责探针 P09（--message "--json" 不误触 JSON 模式）通过。判定：与 spec flag 表完全一致。

## 7. 发现清单

| 编号 | 严重度 | 内容 | 证据 |
|---|---|---|---|
| A-01 | 低 | 文档自相矛盾：v0.6 bdd S-READ-06 断言 showing: 0/4 且注「total 为过滤前全量口径」，但其引用的冻结基线 v0.5 bdd S-READ-06/07 口径为 total=过滤后、limit 前（零命中应为 0/0）。实测为 0/0，实现与真实冻结基线一致，实现无偏差；v0.6 bdd 该句为转写错误 | 命令 paperwork post read t1.post.md --mention zed 输出 showing: 0/0；docs/ssot/specs/cli-ux-redesign/bdd.md L169-179 |
| A-02 | 低 | example 措辞微差：validate 异型文件 format 错误的 example 实测为 paperwork post send myfile --author alice --message "hello"，bdd S-VAL-04 钉住 myfile.post.md（带后缀）。该示例仍可执行（ensure_suffix 自动补后缀建线程），行为面无偏差 | repos/paperwork-cli/src/cmd/validate.rs L104；实测 validate garbage.post.md 输出 |
| A-03 | 观察 | validate --type bogus 的 usage 信封 fix 复用通用 base 文案（提 --author/--message for post send/edit），教学焦点与 validate 场景不完全贴合；spec 未钉住该文案，不构成偏差，可作后续打磨候选 | 实测 validate t1.post.md --type bogus 输出 |
| A-04 | 观察 | 任务单提及的 init 命令与 post notify 动词在 spec 与实现中均不存在（组集合冻结 {profile,post,brief,contacts,validate}，post 动词集 {send,read,summary,edit}）；实测 paperwork init 落 unrecognized subcommand exit 2，符合冻结契约。属任务单范围澄清，非产品偏差 | paperwork help Commands 列表；init 探针输出 |

## 8. 结论

- 一致项：117 个矩阵判定点全部通过（正向 50：P01-P50；负向与边界 41：U01-U36 按形态计；无短形式负向 26：N01-N26）。spec §2 全部 17 个命令签名、全部带值/开关 flag、--json/--plain/-q 三档、usage/运行时两档信封、三条逐字钉住的 canonical_example、updated 箭头串、空键/空值守栏逐字文案、ASCII 契约，均与实现一致。
- 行为偏差：0（阻塞 0，重要 0）。
- 文档级发现：A-01/A-02（均低，建议下轮文档勘误）；观察项 A-03/A-04。
- 覆盖面统计：5 组 17 签名 100% 正向覆盖；26 项无短形式 flag 100% 负向覆盖；短形式白名单 {-a,-m,-q} 正向等价 + 逐命令作用域核验；退出码 0/1/2 全覆盖；七类错误 category 实测触达六类（usage/validation/format/not-found/already-exists/not-allowed），io 类（锁获取失败）未现场触发，依赖既有 288 测试与 code review 覆盖（S-LOCK-03 口径）。
- 未覆盖声明：多进程并发锁实测（S-LOCK-01/02）未在本审计执行（需并发 harness，以既有集成测试全绿为据）；--help 全文逐行 ASCII 仅抽查未逐行穷举。

