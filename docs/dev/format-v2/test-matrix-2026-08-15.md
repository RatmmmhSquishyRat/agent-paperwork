# 79 BDD 场景 × 测试矩阵差分表（2026-08-15）

> **文档性质**：T7（Ivy 测试缺口 G1–G5 超集闭合）交付物之一。逐场景列出 bdd.md 全部 79 个场景的覆盖测试名（既有 + 本轮新增），并给出差分结论。
>
> **实测基准**：本表中全部既有测试名取自当前 worktree（分支 `wip/v0.5-perfection-snapshot-2026-08-15`）实测 `cargo test --workspace -- --list`（304 项基线）；本轮新增 16 项全部位于 `repos/paperwork-cli/tests/ivy_gap_tests.rs`（前缀 `ivy_`，以 ⭐ 标注）。闭合后 workspace 共 320 项。
>
> **测试文件对照**：core 单元 = `paperwork-core/src/**` 内联单测；`ops_tests` / `guard_tests` / `char_tests(roundtrip)` = `paperwork-core/tests/*`；`cli_integration` / `char_tests` / `t6_cli_tests` / `ivy_gap_tests` = `paperwork-cli/tests/*`。

---

## 1. profile（PROF-01..11）

| 场景 | 覆盖测试（实测名） | 本轮差分 |
| --- | --- | --- |
| PROF-01 | `format::profile::tests::test_parse_minimal`、`create_profile_writes_file`、`create_profile_full_single_shot_with_scopes`、`profile_create_and_show`、`profile_create_json`、`char_profile_roundtrip_p1_minimal` | 无空档 |
| PROF-02 | `format::profile::tests::test_parse_description_scope_lines`、`show_profile_reads_file`、`profile_create_and_show`、`edit_profile_updates_fields`、`char_profile_roundtrip_p2_full_scope` | 无空档 |
| PROF-03 | `format::profile::tests::test_parse_multi_row_permission`、`profile_edit` | 无空档 |
| PROF-04 | `format::profile::tests::test_serialize_empty_scope_omitted`、`create_profile_writes_file`、`char_profile_roundtrip_p1_minimal`、`char_profile_roundtrip_p3_description_no_scope` | 无空档 |
| PROF-05 | `format::profile::tests::test_parse_missing_h1`、`create_profile_rejects_overwrite`、`profile_create_duplicate_fails`、`t6_profile_create_scoped_refuses_existing_file`、`concurrent_create_profile_exactly_one_wins` | 无空档 |
| PROF-06 | `format::profile::tests::test_parse_missing_model`；⭐ `ivy_g2_validate_profile_missing_model_envelope`（CLI validate 面坏例信封）；⭐ `ivy_g3_validate_json_error_envelope_structure`（--json 信封结构） | **本轮闭合 CLI 面**（G2/G3） |
| PROF-07 | `format::profile::tests::test_parse_lenient`、`format::tests::test_extract_attribute` | 无空档 |
| PROF-08 | `format::profile::tests::test_parse_crlf`、`format::tests::test_normalize_crlf` | 无空档 |
| PROF-09 | `format::profile::tests::test_parse_unicode` | 无空档 |
| PROF-10 | `format::profile::tests::test_roundtrip`、`char_profile_roundtrip_p1_minimal`、`char_profile_roundtrip_p2_full_scope`、`char_profile_roundtrip_p3_description_no_scope` | 无空档 |
| PROF-11 | `format::profile::tests::test_description_bullet_attribution`、`profile_description_rejects_dangerous_key_line` | 无空档 |

## 2. post / thread（POST-01..36）

| 场景 | 覆盖测试（实测名） | 本轮差分 |
| --- | --- | --- |
| POST-01 | `format::thread::tests::test_parse_full_thread`、`post_send_read`、`char_post_read_default`、`thread_read_range_subset`、`e2e_profile_post_workflow` | 无空档 |
| POST-02 | `format::thread::tests::test_serialize_no_attribute_lines`、`char_thread_roundtrip_t1_multi_message_refs_dynamic_fence` | 无空档 |
| POST-03 | `format::thread::tests::test_derive_mentions_order_and_dedup`、`test_derive_mentions_self_exclusion`、`thread_send_body_text_refs_derived`、`post_send_mention_injects_body_tokens`、`char_post_send_mention_reply_token_injection`；⭐ `ivy_g5_read_filters_no_match_empty_envelope`（派生过滤否定面） | **本轮补否定面**（G5） |
| POST-04 | `format::thread::tests::test_parse_bad_timestamp` | 无空档 |
| POST-05 | `format::thread::tests::test_fence_fake_header`、`find_message_sender_ignores_fake_headers_inside_body_fences`、`thread_edit_new8_fake_headers_inside_body_fences`、`tail_scan_fence_aware_fake_header` | 无空档 |
| POST-06 | `format::thread::tests::test_dynamic_fence_roundtrip`、`format::tests::test_compute_fence_length`、`char_thread_roundtrip_t3_fence_growth` | 无空档 |
| POST-07 | `format::thread::tests::test_sender_not_boundary`、`thread_send_rejects_invalid_sender` | 无空档 |
| POST-08 | `format::thread::tests::test_parse_empty`；⭐ `ivy_g5_summary_missing_file_lenient_empty_summary`（缺文件宽容空 summary 的 CLI 面钉住） | **本轮补宽容路径 CLI 面**（G5） |
| POST-09 | `format::thread::tests::test_parse_preamble_only`、`thread_send_preamble_only_file_ok` | 无空档 |
| POST-10 | `format::thread::tests::test_seq_monotonicity`、`validate_seq_gap` | 无空档 |
| POST-11 | `format::tests::test_validate_markdown_dynamic`、`validate_unclosed_fence` | 无空档 |
| POST-12 | `format::thread::tests::test_parse_crlf`；⭐ `ivy_g5_crlf_post_send_read_roundtrip`（CRLF 文件 read/send/validate CLI 往返） | **本轮闭合 CLI 面**（G5） |
| POST-13 | `format::thread::tests::test_parse_unicode`、`char_thread_roundtrip_t2_unicode_multiline`；⭐ `ivy_g5_unicode_send_read_json_roundtrip`（中文正文/署名 send→read→JSON 往返） | **本轮闭合 CLI 面**（G5） |
| POST-14 | `format::thread::tests::test_serialize_thread_roundtrip`、`char_thread_roundtrip_t1_multi_message_refs_dynamic_fence`、`char_thread_roundtrip_t2_unicode_multiline`、`char_thread_roundtrip_t3_fence_growth` | 无空档 |
| POST-15 | `format::thread::tests::test_preamble_variants`、`test_preamble_participants_line_ignored` | 无空档 |
| POST-16 | `thread_edit_preserves_preamble_verbatim`、`thread_edit_constraints`、`post_edit`、`char_post_edit_success_rewrites_file`、`char_post_edit_not_owned_rejected`、`char_post_edit_not_most_recent_rejected`、`char_post_edit_not_final_rejected`；⭐ `ivy_g4_edit_not_owned_cli_envelope_and_bytes_unchanged`、⭐ `ivy_g4_edit_not_most_recent_cli_envelope_and_bytes_unchanged`、⭐ `ivy_g4_edit_not_final_cli_envelope_and_bytes_unchanged`（三拒绝 CLI 信封类别/措辞 + 拒绝后文件字节不变） | **本轮补字节不变断言**（G4；char_tests 钉字节信封但未断言拒绝后文件不变） |
| POST-17 | `format::thread::tests::test_validate_sender`、`thread_send_rejects_invalid_sender`；⭐ `ivy_g5_injection_guards_cli_literal_newline_refused`（--title 字面换行经 CLI 拒绝） | **本轮补 CLI 注入护栏面**（G5） |
| POST-18 | `thread_send_rejects_oversized`、`post_send_oversized_body_after_injection` | 无空档 |
| POST-19 | `post_send_read`、`post_send_stdin`、`post_send_to_and_participants_flags_removed`、`char_post_send_default_json_quiet_envelopes`、`char_post_send_seq_increments_and_title_flag`、`char_post_send_stdin_body`、`thread_send_creates_file_and_returns_seq`、`thread_send_creates_parent_dirs`、`e2e_profile_post_workflow` | 无空档 |
| POST-20 | `post_create_removed` | 无空档 |
| POST-21 | `format::thread::tests::test_preamble_closed_fence_then_header` | 无空档 |
| POST-22 | `format::thread::tests::test_preamble_unclosed_fence`、`format::tests::test_scanners_unclosed_fence_swallows_tail` | 无空档 |
| POST-23 | `format::thread::tests::test_body_normalization` | 无空档 |
| POST-24 | `format::thread::tests::test_fence_indent_policy`、`format::tests::test_scanners_indent_stance` | 无空档 |
| POST-25 | `format::thread::tests::test_body_fence_info_lenient`、`post_send_read`（写入侧 ` ```md ` 断言） | 无空档 |
| POST-26 | `format::thread::tests::test_multi_fence_first_wins`、`test_message_attribute_lines_ignored` | 无空档 |
| POST-27 | `thread_send_on_preamble_only_file` | 无空档 |
| POST-28 | `format::thread::tests::test_header_trailing_garbage` | 无空档 |
| POST-29 | `thread_edit_preserves_preamble_verbatim`、`thread_edit_preserves_preamble_pseudo_headers`、`thread_edit_preserves_preamble_lone_cr`、`thread_edit_preserves_preamble_fence_close_lone_cr` | 无空档 |
| POST-30 | `thread_edit_rejects_oversized` | 无空档 |
| POST-31 | `post_read_plain_no_preamble`、`char_post_read_plain_and_quiet` | 无空档 |
| POST-32 | `tail_scan_buffer_boundaries` | 无空档 |
| POST-33 | `format::thread::tests::test_derive_reply_to_first_wins`、`thread_send_body_text_refs_derived`、`post_send_reply_to_injects_body_tokens`、`char_post_send_mention_reply_token_injection`；⭐ `ivy_g5_read_filters_no_match_empty_envelope`（--reply-to 过滤否定面） | **本轮补否定面**（G5） |
| POST-34 | `format::thread::tests::test_derive_bare_at_tokens` | 无空档 |
| POST-35 | `format::thread::tests::test_serialize_no_attribute_lines`、`thread_send_body_text_refs_derived` | 无空档 |
| POST-36 | `thread_send_repairs_missing_trailing_newline`、`thread_send_keeps_well_formed_file_untouched`、`post_send_appends_to_file_missing_trailing_newline`、`post_edit_missing_body_example_shows_edit_form` | 无空档 |

## 3. brief（BRIEF-01..12）

| 场景 | 覆盖测试（实测名） | 本轮差分 |
| --- | --- | --- |
| BRIEF-01 | `format::manifest::tests::test_parse_entry_full`、`brief_create_add_read`、`char_brief_create_default_envelope_and_file`、`char_brief_roundtrip_b1_inline_regex_note` | 无空档 |
| BRIEF-02 | `format::manifest::tests::test_no_regex_omitted`、`brief_remove` | 无空档 |
| BRIEF-03 | `format::manifest::tests::test_fenced_regex`、`char_brief_roundtrip_b2_fenced_regex_multi_entry` | 无空档 |
| BRIEF-04 | `format::manifest::tests::test_hash_full_hex`、`brief_add_entry_hash_full` | 无空档 |
| BRIEF-05 | `format::manifest::tests::test_groups_derived`、`test_extract_regex_groups` | 无空档 |
| BRIEF-06 | `format::manifest::tests::test_missing_required`、`brief_create_no_owner`；⭐ `ivy_g2_validate_brief_missing_owner_and_created_envelopes`（缺 owner / 缺 created 双坏例 CLI 信封）；⭐ `ivy_g3_validate_json_error_envelope_structure` | **本轮闭合 CLI 面**（G2/G3） |
| BRIEF-07 | `format::manifest::tests::test_prose_note`、`char_brief_roundtrip_b1_inline_regex_note`、`brief_note_later_attribute_line_stays_legal` | 无空档 |
| BRIEF-08 | `brief_verify_three_states`、`brief_verify`、`char_brief_verify_default_and_json`、`brief_verify_semantic_drift_still_reports_states`、`brief_verify_missing_target_stays_stale_per_spec`、`brief_verify_real_io_failure_is_not_swallowed_as_stale` | 无空档 |
| BRIEF-09 | `brief_verify_newline_sensitive` | 无空档 |
| BRIEF-10 | `format::manifest::tests::test_parse_crlf_unicode` | 无空档 |
| BRIEF-11 | `format::manifest::tests::test_roundtrip`、`char_brief_roundtrip_b1_inline_regex_note`、`char_brief_roundtrip_b2_fenced_regex_multi_entry`、`char_brief_roundtrip_b3_attribute_zone_boundary` | 无空档 |
| BRIEF-12 | `format::manifest::tests::test_entry_attribute_zone_boundary`、`char_brief_roundtrip_b3_attribute_zone_boundary`；⭐ `ivy_g2_validate_brief_partial_migration_residue_rejected`（T2 守卫 CLI 面负例：`## Entries` 与 H3 双变体） | **本轮闭合 CLI 负例面**（G2） |

## 4. contacts（CONT-01..08）

| 场景 | 覆盖测试（实测名） | 本轮差分 |
| --- | --- | --- |
| CONT-01 | `format::contacts::tests::test_parse_links`、`contacts_create_add_read`、`contacts_add_and_read`、`char_contacts_create_add_read_default`、`t6_contacts_read_enriches_relative_to_contacts_dir` | 无空档 |
| CONT-02 | `format::contacts::tests::test_parse_angle_bracket` | 无空档 |
| CONT-03 | `format::contacts::tests::test_roundtrip_windows_path`、`test_serialize_escaping`、`contacts_add_link_roundtrip`、`char_contacts_roundtrip_c3_windows_path` | 无空档 |
| CONT-04 | `format::contacts::tests::test_serialize_escaping`、`contacts_add_link_roundtrip`、`char_contacts_roundtrip_c2_escapes` | 无空档 |
| CONT-05 | `format::contacts::tests::test_missing_title` | 无空档 |
| CONT-06 | `format::contacts::tests::test_bare_path_ignored`、`test_contains_bare_bullet`、`contacts_add_rejects_legacy_bare_bullets`、`validate_rejects_legacy_contacts`、`char_validate_legacy_contacts_format_error`、`contacts_add_rejects_legacy_file`、`char_contacts_add_legacy_bare_bullet_guard`；⭐ `ivy_g2_validate_contacts_legacy_full_envelope`（default 模式完整信封 shape：error/fix/example 三行钉死，与 M2 的 --json 面互补） | **本轮补完整信封 shape**（G2） |
| CONT-07 | `format::contacts::tests::test_unicode` | 无空档 |
| CONT-08 | `format::contacts::tests::test_roundtrip_backslash_escaping`、`test_unescape_and_title`、`contacts_add_link_roundtrip`、`char_contacts_roundtrip_c2_escapes` | 无空档 |

## 5. 并发（CONC-01..04）

| 场景 | 覆盖测试（实测名） | 本轮差分 |
| --- | --- | --- |
| CONC-01 | `concurrent_thread_send_safety`、`thread_send_increments_seq` | 无空档 |
| CONC-02 | `concurrent_first_write_single_preamble`；⭐ `ivy_g5_concurrent_first_send_cli_contention`（两进程 CLI 面首写竞争：preamble 恰一次、seq {1,2}、双双成功） | **本轮闭合 CLI 面**（超集项） |
| CONC-03 | `tail_scan_fence_aware_fake_header`、`tail_scan_fence_parity_limitation_pinned` | 无空档 |
| CONC-04 | `first_write_crash_zero_byte_recovery` | 无空档 |

## 6. validate 语义（VAL-01..08）

| 场景 | 覆盖测试（实测名） | 本轮差分 |
| --- | --- | --- |
| VAL-01 | `validate_ok`、`char_validate_four_formats_ok` | 无空档 |
| VAL-02 | `validate_seq_gap`、`format::thread::tests::test_seq_monotonicity` | 无空档 |
| VAL-03 | `validate_unclosed_fence`、`format::tests::test_validate_markdown_dynamic` | 无空档 |
| VAL-04 | `validate_garbage`（垃圾文本面）；⭐ `ivy_g1_validate_v04_legacy_post_default_envelope`（v0.4 `### #N` 头族形态的 default 信封逐字节钉死 + 退出码 1）；写入侧伴生：`thread_send_rejects_legacy_v04_thread`、`char_post_send_legacy_thread_write_guard` | **本轮闭合 v0.4 形态坏例**（G1；原仅垃圾文本面） |
| VAL-05 | `validate_ok`（正例三格式）；⭐ `ivy_g2_validate_profile_missing_model_envelope`、⭐ `ivy_g2_validate_brief_missing_owner_and_created_envelopes`、⭐ `ivy_g2_validate_contacts_legacy_full_envelope`、⭐ `ivy_g2_validate_brief_partial_migration_residue_rejected`（坏例 CLI 信封）；⭐ `ivy_g3_validate_json_error_envelope_structure`（--json 信封结构） | **本轮闭合坏例 CLI 面**（G2/G3；原仅正例） |
| VAL-06 | `validate_unknown_suffix` | 无空档 |
| VAL-07 | `validate_empty_file` | 无空档 |
| VAL-08 | `validate_suspected_header_warning`、`validate_suspected_header_multi_space_warning`、`char_validate_suspected_header_warning_default_and_json` | 无空档 |

---

## 7. 场景外覆盖面（附录，非 79 场景编号但已被测试钉住）

| 覆盖面 | 覆盖测试（实测名） |
| --- | --- |
| 全局 flag / 信封协议（quiet、json/plain 冲突、错误退出码、JSON 错误走 stdout） | `quiet_suppresses_status_line`、`error_exit_code_1`、`json_error_on_stdout`、`json_and_plain_conflict`、`char_output_err_json_shape`、`char_json_plain_conflict_is_clap_exit_2`、`char_post_send_default_json_quiet_envelopes`；⭐ `ivy_g5_quiet_error_envelope_unchanged`（错误路径 --quiet 组合：stderr 信封不变、stdout 空、退出码 1） |
| 注入护栏（NEW-1，单行字段拒换行；写入侧校验面） | `thread_send_rejects_multiline_preamble_title`、`create_profile_rejects_multiline_name_and_model`、`edit_profile_rejects_multiline_model_and_bad_description`、`brief_create_rejects_multiline_title_and_owner`、`brief_add_entry_rejects_multiline_entry_path`、`contacts_create_rejects_multiline_title`、`contacts_add_rejects_multiline_profile_path`、`format::tests::test_check_single_line`；⭐ `ivy_g5_injection_guards_cli_literal_newline_refused`（CLI 参数注入字面换行的 CLI 面回归：--title 与 --model 双例 + 零落盘） |
| 散文/首行可表性守卫（M1） | `brief_description_rejects_dangerous_key_line`、`profile_description_rejects_dangerous_key_line`、`brief_add_entry_rejects_attribute_shaped_note`、`brief_add_entry_rejects_regex_fence_note`、`char_brief_add_attribute_shaped_note_is_clap_rejected`、`char_brief_add_note_guard_regex_fence_first_line`、`format::tests::test_prose_representation_issue` |
| brief 部分迁移守卫（T2/Sam-S1 ops 面） | `legacy_brief_residue_rejected_at_parse`、`legacy_brief_write_ops_refuse_and_leave_bytes_unchanged`、`format::manifest::tests::test_t4_legacy_residue_differential_corpus`（CLI 面负例见 BRIEF-12 行 ⭐） |
| create 竞争（ops 面） | `concurrent_brief_create_exactly_one_wins`、`concurrent_contacts_create_exactly_one_wins`、`concurrent_create_profile_exactly_one_wins`、`create_ops_repeat_rejected_with_existing_envelope`（CLI 面见 CONC-02 行 ⭐） |
| 路径解析 / 后缀 / UTF-8 | `resolve_contact_path_two_levels`、`t6_contacts_read_unresolvable_entry_stays_unreadable`、`t6_contacts_read_json_enriches_relative_to_contacts_dir`、`cmd::tests::ensure_suffix_*`、`cmd::tests::invalid_utf8::*` |
| mention/reply-to 旗标层校验（MJ-2/n3/n4） | `post_send_mention_rejects_malformed_values`、`post_send_mention_trims_whitespace_and_trailing_comma`、`post_send_reply_to_zero_rejected`、`post_send_reply_token_dedup`、`char_post_send_reply_to_zero_rejected`、`char_post_send_reply_to_missing_seq_envelope_unchanged` |
| 锁 / 尾扫差分语料 | `ops::lock::tests::*`、`ops::thread_scan::tests::*`、`format::contacts::tests::test_t4_*`、`format::thread::tests::test_t4_*`、`format::tests::test_scanners_*` |

---

## 8. 差分结论

1. **79 场景逐一核对：零空档。** 每个场景至少有一个实测存活的测试覆盖；本轮新闭合/增强的场景面共 12 处（PROF-06、POST-03、POST-08、POST-12、POST-13、POST-16、POST-17、POST-33、BRIEF-06、BRIEF-12、CONT-06、CONC-02、VAL-04、VAL-05 中标 ⭐ 的新增面，均属"既有场景的 CLI 坏例/否定面/宽容路径补强"，非新场景）。
2. **G1–G5 台账闭合映射**：G1→VAL-04（`ivy_g1_*`）；G2→PROF-06/BRIEF-06/BRIEF-12/CONT-06/VAL-05（`ivy_g2_*` 四测）；G3→VAL-05 --json 面（`ivy_g3_*`）；G4→POST-16（`ivy_g4_*` 三测，新增拒绝后字节不变断言）；G5→POST-03/08/12/13/17/33 + 信封协议附录面（`ivy_g5_*` 七测，含超集项 CONC-02 CLI 面）。
3. **既有基线未受扰动**：char_tests 38 项黄金快照零 diff（全绿）；cli_integration 41、t6_cli_tests 4、core 215（单元 109 + ops 71 + guard 23 + roundtrip 12，其中 char_tests roundtrip 12 归属 paperwork-core；Ultra Review F7 订正：原计数 209 为验算错误，109+71+23+12=215）、CLI 内联单测 6（paperwork-cli/src 内联）原样存活；验算 38+41+4+16+215+6=320 ✓。新增 16 项纯新增文件 `ivy_gap_tests.rs`，无任何既有源码/测试被修改。（后续 Ultra Review 回流批在此 320 基线上纯新增测试，不影响本表矩阵口径。**实测终态订正（2026-08-15）**：回流批纯新增 +6（来自 F1/F2 守卫测试：core 单元 109→111 +2、ops_tests 71→74 +3、CLI 内联单测 6→7 +1），故 core 220（单元 111 + ops 74 + guard 23 + roundtrip 12）、CLI 内联单测 7；验算 38+41+4+16+220+7=326 ✓，workspace 测试总量终态为 **326** 项。逐目标分项：cli 单元 7 + cli char_tests 38 + cli_integration 41 + ivy_gap_tests 16 + t6_cli_tests 4 + core 单元 111 + core char_tests(roundtrip) 12 + guard_tests 23 + ops_tests 74 = 326。）
