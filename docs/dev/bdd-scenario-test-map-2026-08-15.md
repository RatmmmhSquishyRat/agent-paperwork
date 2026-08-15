# BDD 场景 → 测试差分表（v0.6 文法轮收口）

- 日期：2026-08-15
- 批次：v0.5-perfection P-8（T7 测试缺口闭合核对）
- 对账基准一：`docs/ssot/specs/cli-grammar-v0.6/bdd.md`（v0.6 CLI 行为场景，S-* 独立编号）
- 对账基准二：`docs/dev/format-v2/bdd.md`（format-v2 格式层 79 场景）
- 测试口径：master HEAD（P-7 收口后）workspace 全量 = **397 测试**（含本表随附新增的 S-EDIT-08 用例 1 个）
- 冻结声明：v0.5 bdd 中被本文引用为「冻结回归」的场景（并发 seq 无间隙、--json/--plain/-q 三档、--help/-V 穿透、别名、ensure_suffix 三级解析）行为冻结，映射列给出 v0.6 文法下的现行回归测试

约定：测试名不带文件前缀；`cli:` = paperwork-cli/tests，`core:` = paperwork-core/tests，`lib:` = paperwork-core/src 内单元测试。「声明面」表示该场景的验收以代码审查/盘点断言为准（场景文本自身如此规定），不强制集成测试模拟。

---

## 一、v0.6 bdd.md 场景映射

### 1. post send（S-SEND-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-SEND-01 | v0.6 文法成功发送 | cli: char_post_send_modes, post_send_read |
| S-SEND-02 | 短形式与全称等价 | cli: short_forms_equivalent_to_long_flags, char_post_send_modes |
| S-SEND-03 | 线程不存在自动创建 | cli: path_send_creates_suffixed_landing；core: thread_send_creates_file_and_returns_seq, thread_send_creates_parent_dirs |
| S-SEND-04 | reply-to 隐式 mention（冻结） | cli: implicit_mention_triggered_on_reply, implicit_mention_not_triggered_boundaries, implicit_mention_persisted_to_file, char_post_send_reply_to_implicit_mention |
| S-SEND-05 | 缺 --author（usage） | cli: send_missing_author_is_usage |
| S-SEND-06 | 缺正文通道（usage） | cli: send_missing_message_no_stdin_is_usage, usage_missing_body_post_send |
| S-SEND-07 | --message 与 --stdin 同给（usage） | cli: send_message_and_stdin_conflict_is_usage |
| S-SEND-08 | 仅 --stdin 成功 | cli: post_send_stdin, char_post_send_stdin_body |
| S-SEND-09 | 空正文拒绝（validation） | cli: post_send_empty_body_rejected |
| S-SEND-10 | --message 值以 `-` 开头直传 | cli: dash_body_direct_via_message_send_and_edit |
| S-SEND-11 | 裸 `-` token 教学（usage） | cli: bare_dash_token_teaches_message_flag, usage_fix_dash_teaching_only_for_dash_tokens |
| S-SEND-12 | v0.5 位置文法落 usage（迁移教学） | cli: usage_extra_positional_send, send_exact_two_extra_positionals_is_usage |
| S-SEND-13 | v0.4 `--from` 落 usage（迁移链） | cli: usage_old_grammar_send_from |
| S-SEND-14 | 并发 send seq 无间隙（冻结） | cli: multiprocess_concurrent_send_no_lost_messages；core: concurrent_thread_send_safety |
| S-SEND-15 | NAME/BODY 混淆面消亡 | cli: name_body_confusion_single_string |
| S-SEND-17 | 既有线程 `--title` 静默忽略（冻结） | cli: post_send_title_ignored_on_existing_thread |
| S-SEND-18 | --author 空值拒绝（validation） | cli: send_empty_author_is_validation |
| S-SEND-19 | 缺 PATH（usage） | cli: send_missing_path_is_usage |
| S-SEND-20 | --reply-to/--mention 糖衣 token 注入 | cli: post_send_reply_to_injects_body_tokens, post_send_mention_injects_body_tokens, post_send_reply_token_dedup, post_send_mention_trims_whitespace_and_trailing_comma；core: thread_send_body_text_refs_derived |
| S-SEND-21 | 首次 send preamble 仅 H1 | cli: char_post_send_seq_increments_and_title_flag；core: thread_send_on_preamble_only_file |

注：S-SEND-16 已随 owner 追裁 D1/D2 删除（baseline 勘误），无编号缺口需覆盖。

### 2. post edit（S-EDIT-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-EDIT-01 | v0.6 文法成功编辑 | cli: post_edit, char_post_edit_modes；core: thread_edit_own_message |
| S-EDIT-02 | 缺 --author（usage） | cli: edit_missing_author_is_usage |
| S-EDIT-03 | 缺 --seq（usage） | cli: edit_missing_seq_is_usage |
| S-EDIT-04 | 缺正文通道（usage） | cli: edit_missing_message_is_usage |
| S-EDIT-05 | --message 与 --stdin 同给（usage） | cli: edit_message_and_stdin_conflict_is_usage |
| S-EDIT-06 | SEQ 非数字（usage） | cli: usage_seq_not_numeric |
| S-EDIT-07 | 三重护栏（not-allowed，冻结） | cli: edit_triple_guardrail_cli；core: thread_edit_rejects_other_sender, thread_edit_constraints |
| S-EDIT-08 | v0.5 位置文法调用（usage） | cli: edit_v05_grammar_positional_is_usage（**本轮新增**，差分核对发现的缺口） |
| S-EDIT-09 | 仅 --stdin 成功 | cli: edit_stdin_only_succeeds |

### 3. post read / summary（S-READ-* / S-SUM-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-READ-01 | 窗口字段恒显（冻结） | cli: read_showing_window_small_thread, read_showing_window_over_limit, quiet_read_keeps_showing_and_window, char_post_read_modes |
| S-READ-02 | seq 范围过滤（冻结） | core: thread_read_range_subset；cli: char_post_read_filters_window |
| S-READ-03 | --from 传身份值（usage） | cli: read_from_identity_value_is_usage, usage_identity_flag_post_edit_from |
| S-READ-04 | --mention 无短形式 | cli: read_mention_has_no_short_form |
| S-READ-05 | 文件不存在（冻结） | cli: read_missing_thread_stays_not_found；core: thread_read_not_found |
| S-READ-06 | 零命中 total 口径与空 window | cli: read_mention_filter_zero_hits_on_nonempty_thread, read_empty_thread_showing_zero_no_window |
| S-READ-07 | 过滤+limit 的 total 口径 | cli: read_filter_then_limit_total_semantics |
| S-READ-08 | read --to 身份值（usage） | cli: read_to_identity_value_is_usage |
| S-READ-09 | read --author 习惯迁移（usage） | cli: read_unknown_author_flag_teaches_mention, read_unknown_long_flag_lists_read_filters |
| S-SUM-01 | summary 字段集（title/participants/…） | cli: post_summary, char_post_summary_modes；core: thread_summary_correct, thread_summary_empty_for_missing_file |

### 4. profile（S-PROF-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-PROF-01 | create v0.6 文法成功 | cli: profile_create_and_show, char_profile_create_modes；core: create_profile_writes_file |
| S-PROF-02 | create 缺 --name（usage） | cli: profile_create_missing_name_usage |
| S-PROF-03 | v0.5 位置文法（usage） | cli: usage_v05_grammar_profile_create_positional |
| S-PROF-04 | 重复 create（冻结） | cli: profile_create_duplicate_fails, char_profile_create_scope_and_duplicate；core: create_profile_rejects_overwrite |
| S-PROF-05 | show/edit/list 不变 | cli: char_profile_show_modes, char_profile_show_errors, char_profile_edit_modes, char_profile_list_modes；core: show_profile_reads_file, show_profile_not_found, edit_profile_updates_fields |

### 5. brief（S-BRIEF-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-BRIEF-01 | create v0.6 文法 | cli: brief_create_add_read, char_brief_create_modes；core: brief_create_writes_file |
| S-BRIEF-02 | add v0.6 文法 | cli: brief_create_add_read, char_brief_add_modes；core: brief_add_entry_hash_full |
| S-BRIEF-03 | 缺必填 flag（usage） | cli: brief_missing_required_flags_are_usage |
| S-BRIEF-04 | v0.5 位置文法（usage） | cli: usage_v05_grammar_brief_add_positional |
| S-BRIEF-05 | remove 与 basename 推导（冻结） | cli: brief_remove, brief_add_remove_basename_mapping；core: brief_remove_entry, brief_remove_entry_not_found |
| S-BRIEF-06 | read/verify 三态不变 | cli: brief_verify, char_brief_verify_modes, char_brief_read_modes；core: brief_verify_three_states |
| S-BRIEF-07 | read --entry-title 选择性详情 | cli: brief_read_entry_title_selective_details |
| S-BRIEF-08 | read --entry-title 无匹配（not-found） | cli: brief_read_entry_title_miss_is_not_found |
| S-BRIEF-09 | --entry-title 与 --full 组合 | cli: brief_read_entry_title_combines_with_full |
| S-BRIEF-10 | read --entry-title 空值守栏 | cli: empty_key_values_are_refused_as_validation（含 entry-title 逐字断言） |

### 6. contacts（S-CONTACTS-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-CONTACTS-01 | create 不变 | cli: contacts_create_add_read, char_contacts_create_modes；core: contacts_create_writes_file |
| S-CONTACTS-02 | add v0.6 文法 | cli: contacts_create_add_read, char_contacts_add_modes；core: contacts_add_and_read |
| S-CONTACTS-03 | add 缺 --profile（usage） | cli: contacts_add_missing_profile_is_usage |
| S-CONTACTS-04 | v0.5 位置文法（usage） | cli: usage_v05_grammar_contacts_add_positional |
| S-CONTACTS-05 | read 富化不变 | cli: t6_contacts_read_enriches_relative_to_contacts_dir, t6_contacts_read_json_enriches_relative_to_contacts_dir, t6_contacts_read_unresolvable_entry_stays_unreadable, char_contacts_read_modes |
| S-CONTACTS-06 | remove 成功 | cli: contacts_remove_success；core: contacts_remove_hit_preserves_title_and_order |
| S-CONTACTS-07 | remove 未命中 + label-as-key（not-found） | cli: contacts_remove_miss_and_label_as_key_are_not_found；core: contacts_remove_miss_is_not_found_and_file_unchanged |
| S-CONTACTS-08 | update 成功（label 重派生 + 顺序保留） | cli: contacts_update_success；core: contacts_update_hit_redrives_label_and_keeps_order |
| S-CONTACTS-09 | update 错误路径（not-found/already-exists） | cli: contacts_update_error_paths；core: contacts_update_old_miss_is_not_found_and_file_unchanged, contacts_update_new_already_exists_is_already_exists, contacts_update_old_equals_new_follows_judgment_order |
| S-CONTACTS-10 | remove/update 缺必填 flag（usage，example 逐字） | cli: contacts_remove_update_missing_flags_are_usage |
| S-CONTACTS-11 | 旧文法位置参数（usage）+ add 幂等回归 | cli: contacts_remove_positional_misuse_is_usage；core: contacts_add_idempotent_after_lock_migration, contacts_add_idempotent, idempotent_add_keeps_bytes_and_mtime_stable |
| S-CONTACTS-12 | remove 最后一条目后的文件形态 | cli: contacts_remove_last_entry_shape；core: contacts_remove_last_entry_matches_create_initial_shape |
| S-CONTACTS-13 | 特殊字符路径 remove/update 往返 | cli: contacts_special_char_path_roundtrip；core: remove_update_roundtrip_with_special_character_paths |
| S-CONTACTS-14 | update 到不存在 NEW 的静默成功面（声明面） | cli: contacts_update_nonexistent_new_is_silent_success；core: contacts_update_nonexistent_new_is_silent_success, contacts_update_label_fallback_to_stem_when_new_unreadable |
| S-CONTACTS-15 | add/update 空键护栏 | cli: empty_key_values_are_refused_as_validation, contacts_crud_error_envelopes_are_ascii；core: contacts_add_empty_profile_is_validation_error, contacts_update_empty_keys_are_validation_errors |

### 7. validate（S-VAL-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-VAL-01 | 按后缀推断成功 | cli: validate_post_file |
| S-VAL-02 | --type 覆盖后缀 | cli: validate_type_overrides_suffix |
| S-VAL-03 | 未知后缀（format） | cli: validate_unknown_suffix_no_type_format_error, validate_unknown_suffix |
| S-VAL-04 | 垃圾内容示例换 v0.6 文法 | cli: validate_garbage |
| S-VAL-05 | --type 非法值（usage） | cli: validate_type_bogus_is_usage |
| S-VAL-06 | --type 与后缀交叉（format） | cli: validate_type_mismatch_format_error |

### 8. 横切：路径解析（沿用 v0.5 S-PATH-01~08，示例换 v0.6 文法）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-PATH-01 | 原路径优先 | cli: path_original_file_wins |
| S-PATH-02 | 补后缀 | cli: path_suffix_fallback |
| S-PATH-03 | create 补后缀 | cli: t6_profile_create_scoped_refuses_existing_file（create 侧后缀落点面）+ path_suffix_fallback |
| S-PATH-04 | 两者皆无 not-found | cli: path_both_missing_not_found_names_suffixed_path |
| S-PATH-05 | x.md 与 x.post.md 并存用 x.md | cli: path_both_exist_original_wins |
| S-PATH-06 | send 自动创建落点 | cli: path_send_creates_suffixed_landing |
| S-PATH-07 | 异型文件 format 不改道 | cli: path_stage1_foreign_file_format_error_no_reroute |
| S-PATH-08 | 目录不命中 | cli: path_directory_never_matches_stage1 |

### 9. 横切：输出模式与 ASCII 契约（S-OUT-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-OUT-01 | --json 成功（冻结） | cli: profile_create_json；char_tests 全部 json gold 面（post_send_json_stdout 等） |
| S-OUT-02 | --json 运行时错误（冻结） | cli: json_runtime_error_has_command_field, json_error_on_stdout |
| S-OUT-03 | --json usage 错误 | cli: json_usage_error_on_stdout |
| S-OUT-04 | -q/--plain/--help/-V/缺子命令（冻结） | cli: quiet_suppresses_status_line, quiet_read_keeps_showing_and_window, plain_read_outputs_file_format, help_and_version_pass_through_exit_0, missing_subcommand_message_shape, top_level_parse_failure_command_usage |
| S-OUT-05 | ASCII 输出契约（冻结防线） | cli: ascii_output_contract_guard, contacts_crud_error_envelopes_are_ascii, all_help_output_is_pure_ascii；char_tests 逐 gold 面 CR/ASCII 断言 |
| S-OUT-06 | --json 与 --plain 同给（usage） | cli: json_plain_conflict_is_usage |

### 10. 横切：别名（S-ALIAS-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-ALIAS-01 | po 隐藏别名与既有别名 | cli: po_hidden_alias_equivalent_to_post, single_letter_aliases_work |

### 11. 横切：短形式全表（S-SHORT-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-SHORT-01 | 短形式与全称等价（-a/-m/-q） | cli: short_forms_equivalent_to_long_flags |
| S-SHORT-02 | 命名政策白名单断言（additive） | cli: naming_policy_whitelist, flag_inventory_matches_spec, short_form_whitelist_is_exact, post_group_help_lists_verbs, contacts_group_help_lists_verbs |

### 12. 横切：写路径锁（S-LOCK-*）

| 场景 | 摘要 | 覆盖测试 |
|---|---|---|
| S-LOCK-01 | 多进程并发写不丢失（contacts/brief） | cli: multiprocess_concurrent_contacts_brief_add_no_lost_entries；core: multithread_concurrent_add_remove_loses_no_entries, concurrent_contacts_add_no_lost_updates |
| S-LOCK-02 | profile edit 并发串行化（字段并集） | cli: profile_edit_concurrent_disjoint_fields_union |
| S-LOCK-03 | fast fail 无降级防线 | **声明面**：场景文本规定以 code review + 锁调用点位盘点断言（六写路径 contacts add/remove/update、brief add/remove、profile edit 均为 lock_exclusive 后无降级分支；thread 写路径回归由 S-SEND-14 用例承载）。锁失败 io 信封措辞由 core: test_io_ctx_envelope 与 lib: ops::lock::tests 族钉住 |

---

## 二、format-v2 79 场景映射

### 1. profile（PROF-01~11）

| 场景 | 覆盖测试 |
|---|---|
| PROF-01 最小合法 profile | lib: test_parse_minimal；cli: char_profile_roundtrip_p1_minimal |
| PROF-02 description 散段 + scope 列表 | lib: test_parse_description_scope_lines；cli: char_profile_roundtrip_p2_full_scope |
| PROF-03 同一 permission 多 glob | lib: test_parse_multi_row_permission |
| PROF-04 scope 省略 = 空 | lib: test_serialize_empty_scope_omitted；cli: char_profile_roundtrip_p3_description_no_scope |
| PROF-05 缺 H1 拒绝 | lib: test_parse_missing_h1 |
| PROF-06 缺 model 拒绝 | lib: test_parse_missing_model |
| PROF-07 前向兼容未知字段 | lib: test_parse_lenient |
| PROF-08 CRLF 归一 | lib: test_parse_crlf, test_normalize_crlf, test_normalize_single_pass_equivalence |
| PROF-09 Unicode | lib: test_parse_unicode |
| PROF-10 序列化 roundtrip | lib: test_roundtrip；cli: char_profile_roundtrip_p1/p2/p3 |
| PROF-11 description 散段与 bullet 共存 | lib: test_description_bullet_attribution, test_contains_bare_bullet |

### 2. post / thread（POST-01~36）

| 场景 | 覆盖测试 |
|---|---|
| POST-01 正常路径（preamble 仅 H1 + 消息） | lib: test_parse_full_thread；cli: char_thread_roundtrip_t1_multi_message_refs_dynamic_fence |
| POST-02 广播 = 无 @ 的普通消息 | core: thread_send_body_text_refs_derived；lib: test_derive_bare_at_tokens |
| POST-03 @mention 派生 | lib: test_derive_mentions_order_and_dedup, test_derive_mentions_self_exclusion |
| POST-04 非法时间戳拒绝 | lib: test_parse_bad_timestamp |
| POST-05 fence 内伪消息头不是边界 | lib: test_fence_fake_header；core: tail_scan_fence_aware_fake_header, thread_edit_new8_fake_headers_inside_body_fences |
| POST-06 动态 fence（3~6 反引号） | lib: test_dynamic_fence_roundtrip, test_compute_fence_length；cli: char_thread_roundtrip_t3_fence_growth |
| POST-07 sender 含空白/括号不构成边界 | lib: test_sender_not_boundary；cli: post_send_bad_author 面（char gold） |
| POST-08 空文件 | lib: test_parse_empty；core: thread_summary_empty_for_missing_file |
| POST-09 preamble-only（0 消息） | lib: test_parse_preamble_only；core: thread_send_on_preamble_only_file, thread_send_preamble_only_file_ok |
| POST-10 seq gap | lib: test_seq_monotonicity；cli: validate_seq_gap |
| POST-11 无 fence / 宽容解析 | lib: test_parse_lenient, test_body_fence_info_lenient |
| POST-12 CRLF | lib: test_parse_crlf；core: thread_edit_new8_crlf_file_fallback |
| POST-13 Unicode 正文 | lib: test_parse_crlf_unicode；cli: char_thread_roundtrip_t2_unicode_multiline |
| POST-14 序列化/解析 roundtrip | lib: test_serialize_thread_roundtrip；cli: char_thread_roundtrip_t1/t2 |
| POST-15 preamble 标题 | lib: test_preamble_variants |
| POST-16 edit 保持 preamble 原样 | core: thread_edit_preserves_preamble_verbatim |
| POST-17 写路径 sender 字符校验 | lib: test_validate_sender；core: thread_send_rejects_invalid_sender |
| POST-18 body > 64KB 拒绝 | core: thread_send_rejects_oversized, post_send_oversized_body_after_injection |
| POST-19 send 不改动既有消息 | core: thread_send_keeps_well_formed_file_untouched, thread_send_repairs_missing_trailing_newline；cli: post_send_appends_to_file_missing_trailing_newline |
| POST-20 post create 已删除 | cli: post_create_removed_is_usage |
| POST-21 preamble 闭合 fence 后消息头识别 | lib: test_preamble_closed_fence_then_header |
| POST-22 preamble 未闭合 fence 吞后续头 | lib: test_preamble_unclosed_fence；core: thread_send_allows_legacy_shaped_line_inside_preamble_fence |
| POST-23 body 尾部空白规范化 | lib: test_body_normalization, test_serialize_empty_body |
| POST-24 fence 闭合匹配规则 | lib: test_compute_fence_length, test_fence_scan, test_fence_indent_policy |
| POST-25 body fence info md/markdown 双容 | lib: test_body_fence_info_lenient |
| POST-26 一消息多 fence 取首个 | lib: test_multi_fence_first_wins |
| POST-27 preamble-only 文件 + send | core: thread_send_on_preamble_only_file, thread_send_preamble_only_file_ok |
| POST-28 消息头尾部垃圾 | lib: test_header_trailing_garbage, test_header_whitespace_lenient |
| POST-29 edit 保持 preamble（伪头/单 CR） | core: thread_edit_preserves_preamble_pseudo_headers, thread_edit_preserves_preamble_lone_cr, thread_edit_preserves_preamble_fence_close_lone_cr |
| POST-30 edit body 64KB 拒绝 | core: thread_edit_rejects_oversized |
| POST-31 read --plain 原始文件形态 | cli: plain_read_outputs_file_format, post_read_plain_no_preamble |
| POST-32 尾扫有界截断边界 | core: tail_scan_buffer_boundaries；cli: tail_scan_fence_parity_limitation_pinned |
| POST-33 @#N reply-to token | cli: post_send_reply_to_injects_body_tokens；lib: test_derive_reply_to_first_wins |
| POST-34 裸 @ 不是合法 token | lib: test_derive_bare_at_tokens |
| POST-35 mentions 派生全流程 | lib: test_derive_mentions_order_and_dedup, test_derive_mentions_self_exclusion；core: thread_send_body_text_refs_derived |
| POST-36 缺尾换行文件追加修复 | core: thread_send_repairs_missing_trailing_newline；cli: post_send_appends_to_file_missing_trailing_newline |

### 3. brief（BRIEF-01~12）

| 场景 | 覆盖测试 |
|---|---|
| BRIEF-01 正常路径（条目四字段） | lib: test_parse_entry_full；cli: char_brief_roundtrip_b1_inline_regex_note |
| BRIEF-02 regex 省略 | lib: test_no_regex_omitted |
| BRIEF-03 fenced regex | lib: test_fenced_regex；cli: char_brief_roundtrip_b2_fenced_regex_multi_entry |
| BRIEF-04 hash 全量 SHA-256 | core: brief_add_entry_hash_full；lib: test_hash_full_hex, test_hash_file_matches_hash_bytes |
| BRIEF-05 groups 派生 | lib: test_groups_derived, test_extract_regex_groups |
| BRIEF-06 缺必填属性拒绝 | lib: test_missing_required, test_missing_title |
| BRIEF-07 note 散段 | lib: test_prose_note |
| BRIEF-08 verify 三态 | core: brief_verify_three_states；cli: brief_verify, char_brief_verify_modes |
| BRIEF-09 hash 确定性 | lib: test_hash_deterministic, test_hash_bytes_hello, test_hash_bytes_empty；core: brief_verify_newline_sensitive |
| BRIEF-10 CRLF 与 Unicode | lib: test_parse_crlf, test_unicode |
| BRIEF-11 roundtrip | lib: test_roundtrip；cli: char_brief_roundtrip_b1/b2 |
| BRIEF-12 条目属性区终止边界 | lib: test_entry_attribute_zone_boundary, test_h2_inside_fence_not_entry；cli: char_brief_roundtrip_b3_attribute_zone_boundary |

### 4. contacts（CONT-01~08）

| 场景 | 覆盖测试 |
|---|---|
| CONT-01 正常路径（链接条目） | lib: test_parse_links；cli: char_contacts_roundtrip_c1_plain_links |
| CONT-02 尖括号转义形态 | lib: test_parse_angle_bracket, test_serialize_escaping |
| CONT-03 Windows 带空格路径 | lib: test_roundtrip_windows_path；cli: char_contacts_roundtrip_c3_windows_path |
| CONT-04 roundtrip 含特殊路径 | lib: test_roundtrip_backslash_escaping；cli: char_contacts_roundtrip_c2_escapes |
| CONT-05 缺 H1 拒绝 | lib: test_parse_missing_h1, test_missing_title |
| CONT-06 裸路径 bullet 识别 | lib: test_bare_path_ignored, test_contains_bare_bullet；cli: validate_rejects_legacy_contacts, contacts_add_rejects_legacy_file；core: contacts_add_rejects_legacy_bare_bullets, contacts_add_allows_fenced_bare_bullet_example |
| CONT-07 Unicode | lib: test_unicode, test_parse_crlf_unicode |
| CONT-08 反斜杠转义 roundtrip | lib: test_roundtrip_backslash_escaping, test_unescape_and_title |

### 5. 并发（CONC-01~04）

| 场景 | 覆盖测试 |
|---|---|
| CONC-01 并发写者互斥追加 | core: concurrent_thread_send_safety；cli: multiprocess_concurrent_send_no_lost_messages |
| CONC-02 首写 preamble 单一 | core: concurrent_first_write_single_preamble |
| CONC-03 尾扫 fence 感知（并发写者持锁） | core: tail_scan_fence_aware_fake_header, find_message_sender_ignores_fake_headers_inside_body_fences |
| CONC-04 零字节文件恢复 | core: first_write_crash_zero_byte_recovery |

### 6. validate 面（VAL-01~08）

| 场景 | 覆盖测试 |
|---|---|
| VAL-01 validate post 全通过 | cli: validate_post_file；lib: test_validate_markdown_dynamic |
| VAL-02 seq gap 拒绝 | cli: validate_seq_gap；lib: test_seq_monotonicity, test_seq_monotonicity_overflow_safe |
| VAL-03 未闭合 fence 拒绝 | cli: validate_unclosed_fence |
| VAL-04 空消息/垃圾文件拒绝 | cli: validate_garbage |
| VAL-05 指定类型 validate | cli: validate_type_overrides_suffix |
| VAL-06 未知后缀拒绝 | cli: validate_unknown_suffix_no_type_format_error |
| VAL-07 空文件 validate 拒绝 | cli: validate_empty_file |
| VAL-08 可疑消息头格式 warning | cli: validate_suspected_header_warning, validate_suspected_header_multi_space_warning |

---

## 三、差分结论

1. **无未映射场景**：v0.6 bdd.md 全部 S-* 场景与 format-v2 全部 79 场景均有测试映射或显式声明面落点（S-LOCK-03、S-CONTACTS-14 按场景文本自身的「code review / 声明面」口径）。
2. **本轮闭合的缺口**：S-EDIT-08（v0.5 edit 位置文法 → usage）原无专属测试，本轮新增 `edit_v05_grammar_positional_is_usage`（cli_integration）。
3. **G1–G5 超集核对**：v0.6 轮 tdd 1b/4 用例（混淆面翻转、canonical example 钉住）与 contacts CRUD 批次（任务 #19/#34）已覆盖 G1–G5 断言面；本表第一节逐条对账即核对产物，未见残留缺口。
4. **冻结面确认**：S-SEND-14（并发 seq）、S-OUT-01~04（输出模式）、S-PATH-01~08（ensure_suffix 三级解析）、S-ALIAS-01（别名）在 v0.6 文法下全部有现行回归测试承载，行为冻结声明与测试面一致。
