# Managed File Format v2 测试清单（TDD）

> **文档性质**：实现前测试清单（Normative）。逐条列出需新增/重写/删除的测试，每条标注对应 bdd.md 场景编号与目标文件/函数。
>
> **编号约定**：`T-FM-*`（format/mod.rs 内联单测）、`T-FT-*`（format/thread.rs）、`T-FP-*`（format/profile.rs）、`T-FB-*`（format/manifest.rs）、`T-FC-*`（format/contacts.rs）、`T-OPS-*`（`repos/paperwork-core/tests/ops_tests.rs`）、`T-CLI-*`（`repos/paperwork-cli/tests/cli_integration.rs`）、`T-CI-*`（`.github/workflows/ci.yml` smoke）。
>
> **覆盖义务**：bdd.md 全部 79 个场景必须出现在下表的"BDD 场景"列中；实现完成时以本表为验收核对单。
>
> **2026-08-09 owner 追裁（D1–D3）联动**：改写 T-FT-01/02/03/09/12/13/19/20、T-OPS-07/12/14/16/26、T-CLI-06/09/11、T-CI-01/02；新增 T-FT-23/24/25（派生类测试）；T-FM-02 断言示例改用 profile/brief 键。

---

## §1 删除清单（旧格式测试，全部移除或改写；现状清点按真实条目名）

| 位置 | 删除对象 |
| --- | --- |
| `format/mod.rs` tests | 现有 14 个内联测试：删除 `test_extract_bullet_key`、`test_parse_message_header`、`test_is_boundary_line`、`test_find_message_boundaries_basic`、`test_find_message_boundaries_lone_hr`、`test_find_message_boundaries_fence_aware`、`test_parse_scope_globs`、`test_serialize_scope_globs`、`test_scope_roundtrip`（9 个）；重写 `test_validate_markdown_valid`、`test_validate_markdown_unclosed_three_fence`、`test_validate_markdown_unclosed_four_fence`、`test_validate_markdown_nested_fences`（4 个，按 T-FM-05）；保留 `test_normalize_crlf`（T-FM-01） |
| `format/thread.rs` tests | 全部现有测试（22 个：`test_parse_single_message`、`test_parse_multi_message`、`test_parse_message_with_reply`、`test_parse_message_no_reply`、`test_parse_message_multiline_body`、`test_parse_message_body_with_hr`、`test_parse_message_body_with_h3`、`test_serialize_message_roundtrip`、`test_serialize_message_with_reply_roundtrip`、`test_serialize_message_broadcast`、`test_parse_empty_thread`、`test_seq_monotonicity_valid`、`test_seq_gap_detection`、`test_seq_wrong_start`、`test_parse_crlf_thread`、`test_parse_unicode_message`、`test_parse_multi_recipient`、`test_serialize_thread_roundtrip`、`test_body_with_bold_text`、`test_empty_body`、`test_body_with_triple_backtick_fence`、`test_parse_message_with_mentions`）——旧 `·` 头/`---` 边界字面量全部作废，按 §2.2 重写 |
| `format/profile.rs` tests | 全部现有测试（8 个：`test_parse_profile_basic`、`test_parse_profile_empty_scope`、`test_parse_profile_multi_glob`、`test_serialize_profile_roundtrip`、`test_parse_profile_invalid_no_h1`、`test_parse_profile_missing_model`、`test_parse_profile_crlf`、`test_parse_profile_unicode`）——按 §2.3 重写 |
| `format/manifest.rs` tests | 全部现有测试（10 个）——按 §2.4 重写 |
| `format/contacts.rs` tests | 全部现有测试（6 个）——按 §2.5 重写 |
| `tests/ops_tests.rs` | 全部现有 35 个用例（profile 6、thread 9、thread_edit 4、brief 10、contacts 5、e2e 1）的旧格式字面量与断言改写（函数名保留者见 §3） |
| `tests/cli_integration.rs` | 现有 20 个用例：`post_create_send_read` 等涉及 `post create` 的用例改写（见 §4）；现有 `validate_post_file`、`validate_unknown_suffix` 改写为新 VAL 语义；无 validate 垃圾/空文件用例（新增） |

## §2 format 层内联单测（`repos/paperwork-core/src/format/`）

### 2.1 mod.rs（共享工具）

| 编号 | 测试函数 | 断言要点 | BDD 场景 | spec 章节 |
| --- | --- | --- | --- | --- |
| T-FM-01 | `test_normalize_crlf`（保留） | `\r\n`/`\r` → `\n` | PROF-08、POST-12、BRIEF-10 | §3.1 |
| T-FM-02 | `test_extract_attribute`（重写） | 小写键 `- model: x` 命中；大写键 `- Model: x` 不命中；`- owner: alice` 命中；`- created: 2026-...Z` 命中；非属性行返回 None（post 不再使用属性行，D2；该文法仅服务 profile/brief） | PROF-07、BRIEF-01 | §3.2 |
| T-FM-03 | `test_fence_scan`（新增） | N 反引号 + 任意 info string 开栏；≥N 纯反引号行关栏；`<N` 不关栏；fence 内 `## #N ...` 不参与结构扫描；**≤3 空格前导识别为围栏行、≥4 空格不识别（缩进代码块）；`~~~` 不翻转围栏状态** | POST-05、POST-06、POST-24 | §3.3 |
| T-FM-04 | `test_compute_fence_length`（新增） | 无 backtick → 3；最长串 k=3/4/5/6 → 4/5/6/7 | POST-06 | §3.4 |
| T-FM-05 | `test_validate_markdown_dynamic`（重写） | 任意长度未闭合围栏报告行号；嵌套（长包短）合法；闭合判定按长度规则 | POST-11、VAL-03 | §3.3、§8 |

### 2.2 thread.rs

| 编号 | 测试函数 | 断言要点 | BDD 场景 | spec 章节 |
| --- | --- | --- | --- | --- |
| T-FT-01 | `test_parse_full_thread` | preamble（仅 H1，无 participants 行，D1）+ 消息（无属性区，正文含 @mention/@#N，D2）解析 | POST-01 | §5.2–5.4 |
| T-FT-02 | `test_parse_broadcast` | 无 @ 的普通消息：派生 mentions 空、reply-to None；序列化仅头 + ```md 围栏 | POST-02 | §5.4 |
| T-FT-03 | `test_derive_mentions` | 顺序去重；排除 sender 自提及；`@#N` token 不计入 mentions | POST-03 | §5.4 |
| T-FT-04 | `test_parse_bad_timestamp` | 非法时间戳 → Parse（fix 指向 RFC 3339） | POST-04 | §3.5、§9.2 |
| T-FT-05 | `test_fence_fake_header` | fence 内 `## #99 ...` 不切分消息 | POST-05 | §3.3、§5.3 |
| T-FT-06 | `test_dynamic_fence_roundtrip` | k=3/4/5/6 四个 body：序列化开启行 4/5/6/7 反引号，roundtrip **规范化相等**；另含"关闭行比开启行长"的解析接受用例 | POST-06 | §3.4 |
| T-FT-07 | `test_sender_not_boundary` | sender 含空格/括号的 H2 均不匹配头正则（`[^\s()]+`，R1），归 preamble，消息数 0 | POST-07 | §5.3 |
| T-FT-08 | `test_parse_empty` | 空串/纯空白 → 缺省 meta + 0 消息 | POST-08 | §5.2 |
| T-FT-09 | `test_parse_preamble_only` | 仅 H1（可含被忽略散文）→ meta 正常、0 消息 | POST-09 | §5.2 |
| T-FT-10 | `test_parse_crlf` | CRLF 与 LF 结果一致 | POST-12 | §3.1 |
| T-FT-11 | `test_parse_unicode` | Unicode body/sender 原值保留 | POST-13 | §3.6 |
| T-FT-12 | `test_serialize_thread_roundtrip` | meta + 消息组合 roundtrip（body 规范化相等）；输出无 `---`/`·`/`—`/`- participants:`/任何消息属性行；围栏 info 均为 `md`；含空 body 用例 | POST-14 | §5.9 |
| T-FT-13 | `test_preamble_variants` | H1 后散文被忽略；额外 H2 与历史 `- participants:` 同形行归 preamble 忽略；无 H1 | POST-15 | §5.2 |
| T-FT-14 | `test_seq_monotonicity`（重写保留） | 连续通过；gap 报 "gap"；首值非 1 报 "expected 1" | POST-10、VAL-02 | §8 |
| T-FT-15 | `test_preamble_closed_fence_then_header` | preamble 内已闭合围栏块之后的消息头正常识别 | POST-21 | §3.3、§5.2 |
| T-FT-16 | `test_preamble_unclosed_fence` | preamble 未闭合围栏 → 其后头均非边界，0 消息 | POST-22 | §3.3、§5.2 |
| T-FT-17 | `test_body_normalization` | body 首尾空白行去除、`\n` 连接；规范化相等语义 | POST-23 | §5.4 |
| T-FT-18 | `test_fence_indent_policy` | ≤3 空格围栏行识别、≥4 空格不识别（body 为空宽容） | POST-24 | §3.3 |
| T-FT-19 | `test_body_fence_info_md_markdown` | `md`、`markdown`、无 info 三种围栏均接受为正文（前缀匹配宽容，C2/D3）；写入侧统一输出 `md` | POST-25 | §5.4 |
| T-FT-20 | `test_multi_fence_and_legacy_attr_ignored` | 双围栏取首个、其余忽略；头与围栏间历史属性同形行（`- reply-to: #1`）不具属性语义、忽略 | POST-26 | §5.4 |
| T-FT-21 | `test_header_trailing_garbage` | `(ts) (备注)` 尾部垃圾 → 时间戳 Parse，整文件不可读 | POST-28 | §3.5、§5.3 |
| T-FT-22 | `test_header_whitespace_lenient` | 字段间多空格/行尾尾随空白的头仍解析（R9）；序列化输出规范单空格 | POST-01 补充 | §5.3、§5.9 |
| T-FT-23 | `test_derive_reply_to` | `@#1` → Some(1)；多个 `@#N` 取首个其余忽略；`@#999` 目标不存在仍派生（不校验，宽容） | POST-33 | §5.4 |
| T-FT-24 | `test_at_token_lenient` | 孤立 `@`、`@)` 等非合法 token → 不派生任何结果且不报错 | POST-34 | §5.4 |
| T-FT-25 | `test_derivation_not_persisted` | serialize_message 输出仅头 + ```md 围栏，无任何派生行/字段；派生仅在读取路径 | POST-35 | §5.4、§5.9 |

### 2.3 profile.rs

| 编号 | 测试函数 | 断言要点 | BDD 场景 | spec 章节 |
| --- | --- | --- | --- | --- |
| T-FP-01 | `test_parse_minimal` | H1 + `- model:` 即合法 | PROF-01 | §4.2 |
| T-FP-02 | `test_parse_description_scope_lines` | 散文 description + **Scope 属性行列表**解析（R3） | PROF-02 | §4.1/4.2 |
| T-FP-03 | `test_parse_multi_row_permission` | 同 permission 多行聚合、保序（键可重复） | PROF-03 | §4.2 |
| T-FP-04 | `test_serialize_empty_scope_omitted` | 空 scope → 无 `## Scope`、无 scope 属性行、无 `—` | PROF-04 | §4.3 |
| T-FP-05 | `test_parse_missing_h1` | Parse 错误文案 | PROF-05 | §4.4 |
| T-FP-06 | `test_parse_missing_model` | Parse 错误文案（小写键） | PROF-06 | §4.4 |
| T-FP-07 | `test_parse_lenient` | 未知属性/节/permission（`- admin:`）忽略 | PROF-07 | §3.6 |
| T-FP-08 | `test_parse_crlf` | CRLF 归一 | PROF-08 | §3.1 |
| T-FP-09 | `test_parse_unicode` | Unicode 字段 | PROF-09 | §3.6 |
| T-FP-10 | `test_roundtrip` | 全字段相等 | PROF-10 | §4.3 |
| T-FP-11 | `test_description_bullet_attribution` | description 内 bullet 同形行按属性行识别并忽略，不入 description | PROF-11 | §3.2 |

### 2.4 manifest.rs（brief）

| 编号 | 测试函数 | 断言要点 | BDD 场景 | spec 章节 |
| --- | --- | --- | --- | --- |
| T-FB-01 | `test_parse_entry_full` | path/hash/regex/note 裸文本解析 | BRIEF-01 | §6.2 |
| T-FB-02 | `test_no_regex_omitted` | 解析 regex None；序列化不输出该行与 `—` | BRIEF-02 | §6.2/6.3 |
| T-FB-03 | `test_fenced_regex` | ` ```regex ` 围栏含换行/反引号；序列化复杂模式用围栏 | BRIEF-03 | §6.2 |
| T-FB-04 | `test_hash_full_hex` | 64 位小写 hex 全量保留 | BRIEF-04 | §6.2 |
| T-FB-05 | `test_groups_derived` | 命名捕获组派生；序列化不落盘 groups | BRIEF-05 | §6.2 |
| T-FB-06 | `test_missing_required` | 缺 owner/created/H1 三种坏例 | BRIEF-06 | §6.5 |
| T-FB-07 | `test_prose_note` | 多行散文 note；序列化无 `>` 前缀 | BRIEF-07 | §6.2/6.3 |
| T-FB-08 | `test_parse_crlf_unicode` | CRLF 与 Unicode | BRIEF-10 | §3.1/3.6 |
| T-FB-09 | `test_roundtrip` | 全字段相等；无 `## Entries`/`—`/大写键 | BRIEF-11 | §6.3 |
| T-FB-10 | `test_extract_regex_groups`（保留） | 捕获组提取逻辑不变 | BRIEF-05 | §6.2 |
| T-FB-11 | `test_entry_attribute_zone_boundary` | 属性区至首个非属性非空行（空行不终止）；note 内同形行归 note 不覆盖属性 | BRIEF-12 | §3.2、§6.2 |

### 2.5 contacts.rs

| 编号 | 测试函数 | 断言要点 | BDD 场景 | spec 章节 |
| --- | --- | --- | --- | --- |
| T-FC-01 | `test_parse_links` | 裸形式链接解析（label + path） | CONT-01 | §7.2 |
| T-FC-02 | `test_parse_angle_bracket` | `<path>` 形式剥离尖括号保留空格 | CONT-02 | §7.2 |
| T-FC-03 | `test_serialize_escaping` | 含空格/tab/括号/`<`/`>` → 尖括号形式（`<`/`>` 转义 `\<`/`\>`）；否则裸形式 | CONT-03、CONT-04、CONT-08 | §7.3 |
| T-FC-04 | `test_roundtrip_windows_path` | Windows 带空格路径 roundtrip | CONT-03、CONT-04 | §7.3 |
| T-FC-05 | `test_missing_title` | Parse 错误文案 | CONT-05 | §7.4 |
| T-FC-06 | `test_bare_path_ignored` | 旧裸路径 bullet 忽略 | CONT-06 | §7.2 |
| T-FC-07 | `test_unicode` | Unicode 标题/路径 | CONT-07 | §3.6 |
| T-FC-08 | `test_unescape_and_title` | 解析反转义 `\]`（label）与 `\<`/`\>`（destination）；`[label](path "title")` 忽略 title 提取 destination | CONT-08 | §7.2、§7.3 |

## §3 ops 层测试（`repos/paperwork-core/tests/ops_tests.rs`）

现有 35 个用例全部改写为新格式断言；函数名保留，新增用例如下。

| 编号 | 测试函数 | 断言要点 | BDD 场景 | spec 章节 |
| --- | --- | --- | --- | --- |
| T-OPS-01 | `create_profile_writes_file`（改写） | 落盘字面量含 `# <name>`、`- model:`，空 description/scope 不出现 | PROF-04、PROF-10 | §4.3 |
| T-OPS-02 | `create_profile_creates_parent_dirs`（改写） | 行为不变 | PROF-01 | §4 |
| T-OPS-03 | `create_profile_rejects_overwrite`（改写） | AlreadyExists 不变 | PROF-05 | §9.1 |
| T-OPS-04 | `show_profile_reads_file`（改写） | 新格式 fixture | PROF-02 | §4 |
| T-OPS-05 | `show_profile_not_found`（保留断言） | NotFound 不变 | — | §9.1 |
| T-OPS-06 | `edit_profile_updates_fields`（改写） | 编辑后落盘为属性行列表 Scope（R3） | PROF-02、PROF-10 | §4.3 |
| T-OPS-07 | `thread_send_creates_file_and_returns_seq`（改写） | 首写含 preamble（仅 H1，D1）；返回 seq == 1（无 system 消息）；`thread_send` 新签名不含 to/mentions/reply_to 参数（D2） | POST-19、CONC-02 | §5.7 |
| T-OPS-08 | `thread_send_creates_parent_dirs`（改写） | 行为不变 | POST-19 | §5.7 |
| T-OPS-09 | `thread_send_increments_seq`（改写） | seq 1..N 连续，头文法为 `## #N sender (time)` | CONC-01 | §5.3/5.5 |
| T-OPS-10 | `thread_read_range_subset`（改写） | 新格式 fixture 上范围过滤不变 | POST-01 | §5 |
| T-OPS-11 | `thread_read_not_found`（保留断言） | NotFound 不变 | — | §9.1 |
| T-OPS-12 | `thread_summary_correct`（改写） | count/last/snippet 不变；participants 改由消息 sender 集合派生（首次出现顺序去重，D1） | POST-01 | §5.4 |
| T-OPS-13 | `thread_summary_empty_for_missing_file`（保留） | 行为不变 | POST-08 | §5 |
| T-OPS-14 | `thread_send_body_references`（改写自 `thread_send_with_reply_to`） | body 含 `@#1`/`@bob` 文本原样落盘于 ```md 围栏内；读取侧派生 reply-to/mentions 正确；无任何属性行落盘（D2） | POST-03、POST-33、POST-35 | §5.4 |
| T-OPS-15 | `concurrent_thread_send_safety`（改写） | 2N 条、seq == 1..=2N、body 无损 | CONC-01 | §5.8 |
| T-OPS-16 | `thread_meta_reads_preamble`（新增） | `thread_meta` 仅返回 title（participants 字段已删，D1）；文件缺失返回缺省 meta 不报错 | POST-01、POST-19 | §5.2 |
| T-OPS-17 | `thread_send_rejects_invalid_sender`（新增） | 空格/括号/换行/空串 → Validation，文件不写入 | POST-17 | §5.6 |
| T-OPS-18 | `thread_send_rejects_oversized`（新增） | > 64KB → MessageTooLarge，文件不增长 | POST-18 | §5.8 |
| T-OPS-19 | `thread_edit_preserves_preamble_verbatim`（新增） | 重写后首个消息头之前字节区间**逐字节保留**（含手写 description/额外 H2）；仅目标 body 变化 | POST-16、POST-29 | §5.7 |
| T-OPS-20 | `thread_edit_constraints`（新增） | 非本人/非本人最新/非末条 → NotAllowed 三种 | POST-16 | §5.8 |
| T-OPS-21 | `concurrent_first_write_single_preamble`（新增） | 两线程首写竞争：preamble 恰一次、seq {1,2} | CONC-02 | §5.7 |
| T-OPS-22 | `brief_add_entry_hash_full`（新增） | 落盘 hash 为完整 64 位 hex | BRIEF-04 | §6.2 |
| T-OPS-23 | `brief_verify_three_states`（新增） | Fresh/Shifted/Stale 三态 | BRIEF-08 | §6.4 |
| T-OPS-24 | `brief_verify_newline_sensitive`（新增） | 仅换行变化 → Shifted（文档化预期行为） | BRIEF-09 | §6.4 |
| T-OPS-25 | `contacts_add_link_roundtrip`（新增） | 落盘为链接形式；label 取目标 profile H1（读取失败回退主干）；带空格路径走尖括号转义 | CONT-03、CONT-04、CONT-08 | §7.3 |
| T-OPS-26 | `thread_send_on_preamble_only_file`（新增） | preamble-only 存量文件：seq == 1、原 preamble 不重写、`--title` 忽略 | POST-27 | §5.7 |
| T-OPS-27 | `thread_edit_rejects_oversized`（新增） | edit 新 body 序列化 >64KB → MessageTooLarge，文件不变 | POST-30 | §5.8 |
| T-OPS-28 | `tail_scan_buffer_boundaries`（新增） | 尾扫三例：缓冲起点落在行中间（丢弃不完整行）、起点恰在行首（不丢）、read_start == 0 首行即头（不丢，seq 不重复） | POST-32 | §5.5 |
| T-OPS-29 | `tail_scan_fence_aware_fake_header`（新增） | 缓冲区内已闭合围栏中的伪造头被跳过，下次 send seq 正确；残留限制行为固化 | CONC-03 | §5.5 |
| T-OPS-30 | `first_write_crash_zero_byte_recovery`（新增） | 遗留 0 字节文件 → 下一 send 锁内补写 preamble，preamble 恰一次 | CONC-04 | §5.7 |
| T-OPS-31 | `thread_send_repairs_missing_trailing_newline` / `thread_send_keeps_well_formed_file_untouched`（新增，终审 review F1） | 无尾换行文件追加：锁内探测末字节，非 `\n` 时 payload 前补 `\n`，新头独立成行、全部消息读回无损；正常文件不注入额外空行 | POST-36 | §5.7/§5.9 |

## §4 CLI 集成测试（`repos/paperwork-cli/tests/cli_integration.rs`）

现有用例字面量全部改写；涉及 `post create` 的用例改为 `post send` 首写语义。

| 编号 | 测试函数 | 断言要点 | BDD 场景 |
| --- | --- | --- | --- |
| T-CLI-01 | `profile_create_and_show`（改写） | show 输出新格式字段 | PROF-01/02 |
| T-CLI-02 | `profile_create_json`（改写） | JSON 信封字段不变 | PROF-01 |
| T-CLI-03 | `profile_create_duplicate_fails`（改写） | error 信封不变 | PROF-05 |
| T-CLI-04 | `profile_edit`（改写） | 属性行列表 Scope 编辑（R3） | PROF-03 |
| T-CLI-05 | `profile_list`（改写） | 字面量更新 | PROF-01 |
| T-CLI-06 | `post_send_read`（改写自 `post_create_send_read`） | 首条 send 建文件 + preamble（仅 H1，D1）；read 从 #1 起 | POST-19 |
| T-CLI-07 | `post_send_stdin`（改写） | 行为不变 | POST-19 |
| T-CLI-08 | `post_send_empty_body_rejected`（保留断言） | error validation（既有行为保留，无新增 BDD 场景；CI smoke 同步保留该负例） | — |
| T-CLI-09 | `post_send_removed_flags`（改写自 `post_send_to_flag`） | `--to charlie` 与 `--participants alice,bob` 均报未知 flag（exit ≠ 0）；`--title` 保留可用（D1/D2） | POST-19 |
| T-CLI-10 | `post_edit`（改写） | 编辑后 preamble 原文保留 | POST-16 |
| T-CLI-11 | `post_summary`（改写） | title 直读 preamble；participants 由消息 sender 集合派生（无字符串切分，D1） | POST-19 |
| T-CLI-12 | `post_create_removed`（新增） | `post create` 报未知子命令（exit ≠ 0） | POST-20 |
| T-CLI-13 | `brief_create_add_read`（改写） | 新格式字面量（小写键、无 `—`） | BRIEF-01/04 |
| T-CLI-14 | `brief_remove`（改写） | 行为不变 | BRIEF-02 |
| T-CLI-15 | `brief_verify`（改写） | 三态输出 | BRIEF-08 |
| T-CLI-16 | `contacts_create_add_read`（改写） | 输出含链接形式 `[label](path)`；label 为目标 profile H1 | CONT-01、CONT-03 |
| T-CLI-17 | `validate_ok`（改写自现有 `validate_post_file`） | 四格式合法文件全 ok | VAL-01、VAL-05 |
| T-CLI-18 | `validate_seq_gap`（新增） | gap 文件 → error 信封 **category == "validation"**（直出 Validation，R10） | VAL-02 |
| T-CLI-19 | `validate_unclosed_fence`（新增） | 断 fence → error 信封（category format） | VAL-03 |
| T-CLI-20 | `validate_garbage`（新增；现状无对应用例） | 垃圾/旧格式零消息 → error，fix 指向新文法 | VAL-04 |
| T-CLI-21 | `validate_unknown_suffix`（改写自现有同名用例） | 未知后缀 → error | VAL-06 |
| T-CLI-22 | `post_read_plain_no_preamble`（新增） | read --plain 子集输出无 preamble；再解析 title 空宽容 | POST-31 |
| T-CLI-23 | `validate_empty_file`（新增） | 空 .post.md → error（Parse，行为变更） | VAL-07 |
| T-CLI-24 | `validate_suspected_header_warning`（新增） | 疑似头行 → validate 仍 ok 且含 warning + fix | VAL-08 |
| T-CLI-25 | `post_send_appends_to_file_missing_trailing_newline` / `post_edit_missing_body_example_shows_edit_form`（新增，终审 review F1/F3） | 无尾换行文件追加端到端：send 成功、读回 2 条消息无黏连；edit 缺 body 错误 example 为 edit 形态（不再错指 send） | POST-36 |

## §5 CI smoke（`.github/workflows/ci.yml`）

| 编号 | 改动点 | 断言要点 | BDD 场景 |
| --- | --- | --- | --- |
| T-CI-01 | unix smoke 脚本改写 | 删除 `post create` 行；首写改 `post send standup --title "Test" --from alice "Hello world"`（无 --participants，D1）；`--reply-to 2` 改 `--reply-to 1`（OQ-4 默认：糖衣 flag 保留，正文 token 注入）；删除 `--to` 断言（flag 已删，D2），改断言落盘围栏 info 为 `md`（D3）；`validate standup.post.md` 保留；新增 seq gap 负例（手工写 gap 文件 → 期望 error，category validation） | POST-19、POST-20、POST-25、VAL-01/02 |
| T-CI-02 | windows smoke 脚本同步改写 | 与 T-CI-01 语义一致（Select-String 形式） | 同上 |

## §6 覆盖核对表（BDD → TDD 反查）

| BDD 场景 | 覆盖条目 |
| --- | --- |
| PROF-01..10 | T-FP-01..10；T-OPS-01..06；T-CLI-01..05 |
| PROF-11 | T-FP-11 |
| POST-01..15 | T-FT-01..14、T-FT-22；T-FM-03..05；T-OPS-07..16 |
| POST-16 | T-OPS-19、T-OPS-20；T-CLI-10 |
| POST-17 | T-OPS-17 |
| POST-18 | T-OPS-18 |
| POST-19 | T-OPS-07/08/16；T-CLI-06/07/09/11；T-CI-01/02 |
| POST-20 | T-CLI-12；T-CI-01/02 |
| POST-21 | T-FT-15 |
| POST-22 | T-FT-16 |
| POST-23 | T-FT-17 |
| POST-24 | T-FM-03；T-FT-18 |
| POST-25 | T-FT-19；T-CI-01/02 |
| POST-26 | T-FT-20 |
| POST-27 | T-OPS-26 |
| POST-28 | T-FT-21 |
| POST-29 | T-OPS-19 |
| POST-30 | T-OPS-27 |
| POST-31 | T-CLI-22 |
| POST-32 | T-OPS-28 |
| POST-33 | T-FT-23；T-OPS-14 |
| POST-34 | T-FT-24 |
| POST-35 | T-FT-25；T-OPS-14 |
| POST-36 | T-OPS-31；T-CLI-25 |
| BRIEF-01..07、10、11 | T-FB-01..10 |
| BRIEF-08 | T-OPS-23；T-CLI-15 |
| BRIEF-09 | T-OPS-24 |
| BRIEF-12 | T-FB-11 |
| CONT-01..07 | T-FC-01..07；T-OPS-25；T-CLI-16 |
| CONT-08 | T-FC-03、T-FC-08；T-OPS-25 |
| CONC-01 | T-OPS-09、T-OPS-15 |
| CONC-02 | T-OPS-21 |
| CONC-03 | T-OPS-29 |
| CONC-04 | T-OPS-30 |
| VAL-01..06 | T-CLI-17..21；T-FM-05；T-CI-01/02 |
| VAL-07 | T-CLI-23 |
| VAL-08 | T-CLI-24 |

**验收门槛**（与 impl_plan.md 一致）：`cargo test --workspace` 全绿 + `cargo clippy --all-targets -- -D warnings` 零告警；CI 三平台 smoke 通过。
