---
id: semantic-agentic-workflow-tests-cli-tests
summary: Semantic coverage for "apps/agentic-workflow/tests/cli/tests"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "CLI tests cover TD/CB lifecycle dispatch, including CB claim and lifecycle command behavior."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-apply-section-lookup-parity
    claim: td-apply-section-lookup-parity
    coverage: full
    rationale: "The in-place real-CLI fixture proves body-only Logic normalization, malformed non-mutation, and sequential applicability Logic to structured Unit Test dispatch."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: default-td-target-plan-queue
    claim: default-td-target-plan-queue
    coverage: full
    rationale: "The in-place real-CLI fixture applies fresh Logic, Changes, and Unit Test payloads in both passes, checks every projection lock, validates and locks the final TD, then proves aw td gen creates its explicit new target."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: rebased-td-lifecycle-recovery
    claim: rebased-td-lifecycle-recovery
    coverage: full
    rationale: "Real CLI fixtures prove stale rewritten TD history is re-provisioned with a fresh projection, reachable exact Td-Init resumes without reset, and ordinary phase created still provisions."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: terminal-ec-process-liveness
    claim: terminal-ec-process-liveness
    coverage: full
    rationale: "Real CLI regressions prove bounded no-child wrapper cleanup, pre-mutation timeout refusal, and cross-process same-WI single-flight with one EC launch."
---

# Semantic TD: agentic-workflow/tests/cli/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/tests/cli/tests"
  source_group: "apps/agentic-workflow/tests/cli/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/agentic-workflow/tests/cli/tests/td_check_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "test_case"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "test_td_check_registered"
            kind: "function"
            public: false
          - name: "test_td_validate_subcommand_removed_from_check_test_surface"
            kind: "function"
            public: false
          - name: "test_td_check_slug_mode_and_path_mode_share_rule_registry"
            kind: "function"
            public: false
          - name: "test_td_check_path_mode_smoke"
            kind: "function"
            public: false
          - name: "test_td_check_accepts_operations_section_types"
            kind: "function"
            public: false
          - name: "test_td_check_unresolvable_target_errors"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/td_no_merge_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "test_five_step_transaction"
            kind: "function"
            public: false
          - name: "test_rollback_on_close_fail"
            kind: "function"
            public: false
          - name: "test_rollback_on_phase_fail"
            kind: "function"
            public: false
          - name: "test_rollback_on_commit_fail"
            kind: "function"
            public: false
          - name: "test_rollback_on_prune_fail"
            kind: "function"
            public: false
          - name: "test_idempotent_all_complete"
            kind: "function"
            public: false
          - name: "test_idempotent_issue_still_open"
            kind: "function"
            public: false
          - name: "test_idempotent_phase_not_advanced"
            kind: "function"
            public: false
          - name: "test_idempotent_trailer_missing"
            kind: "function"
            public: false
          - name: "test_partial_state_worktree_pruned"
            kind: "function"
            public: false
          - name: "test_done_only_after_all"
            kind: "function"
            public: false
          - name: "test_integration_idempotent_merge"
            kind: "function"
            public: false
          - name: "test_td_merged_constant_value"
            kind: "function"
            public: false
          - name: "wait_for_1579_path"
            kind: "function"
            public: false
          - name: "wait_for_1579_process_exit"
            kind: "function"
            public: false
          - name: "test_code_check_refuses_configured_red_ec_gate"
            kind: "function"
            public: false
          - name: "test_code_check_bounds_no_child_ec_wrapper_and_preserves_phase"
            kind: "function"
            public: false
          - name: "test_code_check_cross_process_single_flight_prevents_duplicate_ec_launch"
            kind: "function"
            public: false
          - name: "test_code_check_fast_green_stale_reader_rechecks_phase_before_ec"
            kind: "function"
            public: false
          - name: "test_code_check_retry_contends_while_terminal_transition_holds_lease"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/recovery_flow_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "test_case"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "test_recovery_verbs_present"
            kind: "function"
            public: false
          - name: "flow_b1_e2e_init_and_sync"
            kind: "function"
            public: false
          - name: "flow_b2_e2e_td_claim_from_path"
            kind: "function"
            public: false
          - name: "flow_b3_e2e_cb_then_td_claim"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/standardize_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "test_case"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "standardize_subcommands_registered"
            kind: "function"
            public: false
          - name: "standardize_top_level_and_audit_forms_fail_to_parse"
            kind: "function"
            public: false
          - name: "audit_record_is_rehomed_under_td"
            kind: "function"
            public: false
          - name: "audit_record_project_option_propagates"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_namespace_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "test_case"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "test_cb_gen_registered"
            kind: "function"
            public: false
          - name: "test_cb_check_registered"
            kind: "function"
            public: false
          - name: "test_cb_gen_phase_advance"
            kind: "function"
            public: false
          - name: "test_cb_gen_trailer"
            kind: "function"
            public: false
          - name: "test_cb_gen_envelope"
            kind: "function"
            public: false
          - name: "test_cb_check_group_by"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/root_doc_mirror_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "repo_root"
            kind: "function"
            public: false
          - name: "strip_codex_only_blocks"
            kind: "function"
            public: false
          - name: "agents_md_is_claude_md_plus_codex_whitelist"
            kind: "function"
            public: false
      - path: "apps/agentic-workflow/tests/cli/tests/root_trait_coverage_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "repo_root"
            kind: "function"
            public: false
          - name: "service_archetype_section_range"
            kind: "function"
            public: false
          - name: "every_trait_contributing_anchor_resolves_to_a_real_heading"
            kind: "function"
            public: false
          - name: "every_service_archetype_h3_is_either_trait_anchored_or_marked_policy_only"
            kind: "function"
            public: false
      - path: "apps/agentic-workflow/tests/cli/tests/root_doc_allowlist_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "repo_root"
            kind: "function"
            public: false
          - name: "repo_root_md_files_match_allowlist"
            kind: "function"
            public: false
      - path: "apps/agentic-workflow/tests/cli/tests/chain_liveness_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "skip_unless_binaries"
            kind: "function"
            public: false
          - name: "init_seed_repo"
            kind: "function"
            public: false
          - name: "write_demo_changes_spec"
            kind: "function"
            public: false
          - name: "seed_open_issue_at_phase"
            kind: "function"
            public: false
          - name: "seed_stranded_terminal_issue"
            kind: "function"
            public: false
          - name: "count_cb_code_check_trailer_commits"
            kind: "function"
            public: false
          - name: "chain_liveness_claim_never_lands_on_deadlock_phase"
            kind: "function"
            public: false
          - name: "chain_liveness_code_check_terminates_within_tick_budget"
            kind: "function"
            public: false
          - name: "chain_liveness_code_check_retry_recovers_stranded_terminal_within_tick_budget"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/td_dirty_gate_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "skip_unless_ready"
            kind: "function"
            public: false
          - name: "run"
            kind: "function"
            public: false
          - name: "bootstrap_repo"
            kind: "function"
            public: false
          - name: "write_fixture"
            kind: "function"
            public: false
          - name: "append_needs_revision_review"
            kind: "function"
            public: false
          - name: "touch_dirty_issue"
            kind: "function"
            public: false
          - name: "dirty_gate_accepts_spec_and_issue"
            kind: "function"
            public: false
          - name: "dirty_gate_rejects_third_dirty"
            kind: "function"
            public: false
          - name: "dirty_gate_accepts_spec_only"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/inplace_mode_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "skip_unless_ready"
            kind: "function"
            public: false
          - name: "bootstrap_repo"
            kind: "function"
            public: false
          - name: "current_branch"
            kind: "function"
            public: false
          - name: "branch_exists"
            kind: "function"
            public: false
          - name: "git_status"
            kind: "function"
            public: false
          - name: "commit_all_with_message"
            kind: "function"
            public: false
          - name: "git_log_messages"
            kind: "function"
            public: false
          - name: "write_td_review_fixture"
            kind: "function"
            public: false
          - name: "append_needs_revision_review"
            kind: "function"
            public: false
          - name: "inplace_td_init_switches_branch_no_worktree_dir"
            kind: "function"
            public: false
          - name: "td_create_on_project_branch_stays_on_current_branch"
            kind: "function"
            public: false
          - name: "td_create_rebased_lifecycle_reprovisions_unreachable_exact_td_init"
            kind: "function"
            public: false
          - name: "td_create_rebased_lifecycle_preserves_reachable_exact_td_init"
            kind: "function"
            public: false
          - name: "td_create_numeric_id_uses_tracker_id_branch_with_legacy_cache_file"
            kind: "function"
            public: false
          - name: "inplace_verb_bails_without_init"
            kind: "function"
            public: false
          - name: "td_review_apply_accepts_dirty_spec_on_td_branch"
            kind: "function"
            public: false
          - name: "td_review_apply_rejects_unrelated_dirty_file"
            kind: "function"
            public: false
          - name: "wi_validate_accepts_apply_dirty_issue_file_on_issue_branch"
            kind: "function"
            public: false
          - name: "td_dispatch_envelope"
            kind: "function"
            public: false
          - name: "run_td_section_apply"
            kind: "function"
            public: false
          - name: "dispatched_payload_path"
            kind: "function"
            public: false
          - name: "assert_td_projection"
            kind: "function"
            public: false
          - name: "td_1598_logic_payload"
            kind: "function"
            public: false
          - name: "td_1598_changes_payload"
            kind: "function"
            public: false
          - name: "td_1598_unit_test_payload"
            kind: "function"
            public: false
          - name: "td_1598_changes_skeleton_body"
            kind: "function"
            public: false
          - name: "td_create_default_changes_queue_applies_both_passes_then_gen_uses_explicit_target"
            kind: "function"
            public: false
          - name: "td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
          - name: "run_init"
            kind: "function"
            public: false
          - name: "aw_start_block"
            kind: "function"
            public: false
          - name: "combined_output"
            kind: "function"
            public: false
          - name: "fresh_init_creates_both_root_docs_and_check_is_clean"
            kind: "function"
            public: false
          - name: "fresh_init_agents_md_block_matches_doc_mirror_projection_of_claude_md_block"
            kind: "function"
            public: false
          - name: "init_check_detects_tamper_without_writing_and_init_restores_it"
            kind: "function"
            public: false
          - name: "init_emits_chainable_next_step"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_review_revise_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "smoke_phase_constants_compile"
            kind: "function"
            public: false
          - name: "smoke_lifecycle_trailers_compile"
            kind: "function"
            public: false
          - name: "smoke_is_terminal_code_checkable_includes_new_phases"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/merge_target_branch.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "init_git_repo"
            kind: "function"
            public: false
          - name: "write_score_config"
            kind: "function"
            public: false
          - name: "head_sha"
            kind: "function"
            public: false
          - name: "detach_head"
            kind: "function"
            public: false
          - name: "case_a_feature_branch_detected"
            kind: "function"
            public: false
          - name: "case_b_main_branch_detected"
            kind: "function"
            public: false
          - name: "case_c_target_branch_override_wins"
            kind: "function"
            public: false
          - name: "case_d_detached_head_uses_config_default_branch"
            kind: "function"
            public: false
          - name: "case_detached_head_no_config_returns_error"
            kind: "function"
            public: false
          - name: "case_e_full_cli_regression"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/td_claim_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "test_case"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "test_td_claim_registered"
            kind: "function"
            public: false
          - name: "test_td_claim_from_path_flag"
            kind: "function"
            public: false
          - name: "test_td_claim_force_rebase_flag"
            kind: "function"
            public: false
          - name: "test_td_claim_trailer_const"
            kind: "function"
            public: false
          - name: "test_td_claim_phase_target"
            kind: "function"
            public: false
          - name: "test_td_claim_e2e_phase_advance"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/hook_pretooluse_write_scope.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method", "test_case"]
        symbols:
          - name: "AW_BIN"
            kind: "constant"
            public: false
          - name: "skip_unless_git"
            kind: "function"
            public: false
          - name: "bootstrap_repo"
            kind: "function"
            public: false
          - name: "checkout_branch"
            kind: "function"
            public: false
          - name: "run"
            kind: "function"
            public: false
          - name: "write_config"
            kind: "function"
            public: false
          - name: "jet_config"
            kind: "function"
            public: false
          - name: "HookOutcome"
            kind: "struct"
            public: false
          - name: "invoke_hook"
            kind: "function"
            public: false
          - name: "invoke_hook_with_payload"
            kind: "function"
            public: false
          - name: "tp5_main_branch_unscoped"
            kind: "function"
            public: false
          - name: "tp6_tracking_branches_unscoped"
            kind: "function"
            public: false
          - name: "tp7_no_matching_project_entry_blocks"
            kind: "function"
            public: false
          - name: "tp8_malformed_config_fails_open"
            kind: "function"
            public: false
          - name: "tp9_detached_head_fails_open"
            kind: "function"
            public: false
          - name: "tp10_cwd_outside_repo_fails_open"
            kind: "function"
            public: false
          - name: "project_jet_in_scope_path_allows"
            kind: "function"
            public: false
          - name: "project_jet_out_of_scope_blocks"
            kind: "function"
            public: false
          - name: "missing_file_path_allows"
            kind: "function"
            public: false
          - name: "non_json_stdin_allows"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/project_health_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "managed"
            kind: "function"
            public: false
          - name: "regenerable"
            kind: "function"
            public: false
          - name: "semantic"
            kind: "function"
            public: false
          - name: "cb_summary"
            kind: "function"
            public: false
          - name: "cold_summary"
            kind: "function"
            public: false
          - name: "stack_migration"
            kind: "function"
            public: false
          - name: "stack_migration_with_dependency_blocker"
            kind: "function"
            public: false
          - name: "clean_project_health_json_fields_are_healthy"
            kind: "function"
            public: false
          - name: "blocked_project_health_collects_governance_blockers"
            kind: "function"
            public: false
          - name: "regenerability_gaps_are_advisory_when_production_gates_clean"
            kind: "function"
            public: false
          - name: "semantic_review_required_is_reported_without_blocking_project_health"
            kind: "function"
            public: false
          - name: "dependency_policy_blockers_block_even_when_stack_migration_is_normalized"
            kind: "function"
            public: false
          - name: "cold_rebuild_failures_block_project_health"
            kind: "function"
            public: false
          - name: "project_health_takeover_audit_axis_is_not_applicable_on_fresh_project"
            kind: "function"
            public: false
          - name: "project_health_takeover_audit_axis_reports_recorded_state"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/td_dispatch_chain_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "td_dispatch_chain_round_trip_placeholder"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_review_to_merge_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "test_cb_reviewed_merge_succeeds"
            kind: "function"
            public: false
          - name: "test_cb_genned_still_accepted"
            kind: "function"
            public: false
          - name: "test_cb_filled_still_accepted"
            kind: "function"
            public: false
          - name: "test_td_reviewed_still_accepted"
            kind: "function"
            public: false
          - name: "test_td_merged_still_accepted"
            kind: "function"
            public: false
          - name: "test_unknown_phase_rejected"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/in_place_lifecycle_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "skip_unless_git"
            kind: "function"
            public: false
          - name: "bootstrap_repo"
            kind: "function"
            public: false
          - name: "current_branch"
            kind: "function"
            public: false
          - name: "in_place_activation_switches_branch_and_skips_worktrees_dir"
            kind: "function"
            public: false
          - name: "in_place_cross_namespace_chain"
            kind: "function"
            public: false
          - name: "in_place_reentry_is_idempotent"
            kind: "function"
            public: false
          - name: "in_place_refuses_dirty_tree"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "test_case"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "legacy_top_level_commands_are_removed"
            kind: "function"
            public: false
          - name: "workflow_protocol_commands_remain_registered"
            kind: "function"
            public: false
          - name: "deprecated_capability_alias_is_rejected_by_parser"
            kind: "function"
            public: false
          - name: "deleted_top_level_commands_fail_as_unknown_commands"
            kind: "function"
            public: false
          - name: "active_docs_and_templates_do_not_reference_deleted_commands"
            kind: "function"
            public: false
          - name: "deprecated_td_aliases_are_removed"
            kind: "function"
            public: false
          - name: "canonical_cb_commands_remain_registered"
            kind: "function"
            public: false
          - name: "test_td_validate_subcommand_is_removed"
            kind: "function"
            public: false
          - name: "test_td_validate_parse_fails"
            kind: "function"
            public: false
          - name: "public_aggregation_points_remain_registered"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/phase_migration_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "test_phase_reader_accepts_legacy"
            kind: "function"
            public: false
          - name: "test_phase_writer_emits_canonical"
            kind: "function"
            public: false
          - name: "test_trailer_reader_accepts_legacy"
            kind: "function"
            public: false
          - name: "test_trailer_writer_emits_canonical"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_fill_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "test_case"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "marker"
            kind: "function"
            public: false
          - name: "handwrite_begin"
            kind: "function"
            public: false
          - name: "handwrite_end"
            kind: "function"
            public: false
          - name: "test_cb_fill_registered"
            kind: "function"
            public: false
          - name: "test_cb_fill_apply_flag"
            kind: "function"
            public: false
          - name: "test_cb_fill_spec_path_flag"
            kind: "function"
            public: false
          - name: "test_issue_phase_cb_filled_variant"
            kind: "function"
            public: false
          - name: "test_cb_arbitrate_constants"
            kind: "function"
            public: false
          - name: "test_cb_arbitrate_registered"
            kind: "function"
            public: false
          - name: "test_lifecycle_trailer_cb_fill_variant"
            kind: "function"
            public: false
          - name: "test_td_code_check_accepts_cb_filled"
            kind: "function"
            public: false
          - name: "test_brief_mode_envelope_shape"
            kind: "function"
            public: false
          - name: "test_brief_mode_marker_list_present"
            kind: "function"
            public: false
          - name: "test_brief_mode_agent_address"
            kind: "function"
            public: false
          - name: "test_apply_marker_enumerates_block"
            kind: "function"
            public: false
          - name: "test_apply_marker_no_adjacent_disturbance"
            kind: "function"
            public: false
          - name: "test_zero_marker_fastpath_no_markers"
            kind: "function"
            public: false
          - name: "test_count_matches_enumeration"
            kind: "function"
            public: false
          - name: "test_extract_change_paths_supports_changes_and_files"
            kind: "function"
            public: false
          - name: "test_scope_filters_to_changed_source_paths"
            kind: "function"
            public: false
          - name: "test_scope_zero_marker_for_spec_only_change"
            kind: "function"
            public: false
          - name: "test_scope_missing_spec_uses_legacy_all_markers"
            kind: "function"
            public: false
          - name: "test_collision_enumerate_returns_both_entries"
            kind: "function"
            public: false
          - name: "test_apply_marker_replaces_block"
            kind: "function"
            public: false
          - name: "test_cb_fill_trailer_committed"
            kind: "function"
            public: false
          - name: "test_cb_filled_phase_written"
            kind: "function"
            public: false
          - name: "test_cb_check_gate_rejection"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_claim_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "test_case"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "test_cb_claim_registered"
            kind: "function"
            public: false
          - name: "test_cb_claim_init_flag"
            kind: "function"
            public: false
          - name: "test_cb_claim_issue_stub_flag"
            kind: "function"
            public: false
          - name: "test_cb_claim_trailer_const"
            kind: "function"
            public: false
          - name: "test_cb_claim_fillback_invoked_e2e"
            kind: "function"
            public: false
          - name: "test_cb_claim_non_interactive_flag_registered"
            kind: "function"
            public: false
          - name: "test_cb_claim_non_interactive_writes_spec"
            kind: "function"
            public: false
          - name: "test_gen_source_projects_legacy_snapshot_and_runs_generated_test"
            kind: "function"
            public: false
          - name: "count_md_recursive"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
      - path: "apps/agentic-workflow/tests/cli/tests/wi_close_remote_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "test_case"]
        symbols:
          - name: "GhFixtureState"
            kind: "struct"
            public: false
          - name: "gh_fixture"
            kind: "function"
            public: false
          - name: "start_gh_fixture"
            kind: "function"
            public: false
          - name: "write_project"
            kind: "function"
            public: false
          - name: "write_gh_adapter"
            kind: "function"
            public: false
          - name: "run_aw"
            kind: "function"
            public: false
          - name: "wi_close_remote_numeric_rehydrates_reason_and_closes_once"
            kind: "function"
            public: false
          - name: "wi_close_missing_remote_reports_backend_repo_and_recovery_command"
            kind: "function"
            public: false
          - name: "wi_close_local_issue_behavior_is_preserved"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "apps/agentic-workflow/tests/cli/tests"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "apps/agentic-workflow/tests/cli/tests/td_check_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/td_no_merge_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/recovery_flow_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/standardize_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_namespace_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/root_doc_mirror_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/root_trait_coverage_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/root_doc_allowlist_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/chain_liveness_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/td_dirty_gate_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/inplace_mode_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_review_revise_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/merge_target_branch.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/td_claim_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/hook_pretooluse_write_scope.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/project_health_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/td_dispatch_chain_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_review_to_merge_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/in_place_lifecycle_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/phase_migration_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_fill_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/cb_claim_test.rs"
      - path: "apps/agentic-workflow/tests/cli/tests/wi_close_remote_test.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/agentic-workflow/tests/cli/tests/td_check_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/td_no_merge_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
      Issue #1579 adds real-binary regressions for a wrapper whose external
      child has already exited, proving timeout remains bounded, the wrapper
      is gone, the WI stays open in its pre-terminal phase, no terminal commit
      lands, and the envelope returns `terminal_ec_timeout` with exact
      `aw td code-check <slug>` retry guidance. A separate two-process test
      launches the same slug concurrently and proves the filesystem lock
      rejects the second caller with `terminal_ec_single_flight` while the EC
      launch marker records exactly one execution. The debug-only barrier test
      pauses a process after its initial `cb_filled` read, lets another process
      finish a fast-green terminal transition, then proves the stale reader
      re-reads `td_merged` after lease acquisition, skips EC, and leaves both
      the launch count and terminal commit count at one. A post-phase-update
      barrier separately proves that a caller beginning in retry phase still
      contends on the owner's lease and cannot race landing or terminal commit.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/recovery_flow_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/standardize_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/cb_namespace_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/root_doc_mirror_test.rs"
    action: create
    section: schema
    description: |
      Root doc mirror contract (meta-doc sheet 1): AGENTS.md must equal
      CLAUDE.md plus the fixed Codex-only whitelist (title swap, Codex
      Operational Rules section, slash-command translation paragraph);
      drift outside the whitelist fails the suite.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/root_trait_coverage_test.rs"
    action: create
    section: schema
    description: |
      Issue #1077 (traits slice 1/3): bidirectional coverage between
      `doc_mirror::TRAITS` and CONTRIBUTING.md's "## Service archetype..."
      section. `every_trait_contributing_anchor_resolves_to_a_real_heading`
      proves every `TraitDef.contributing_anchor` matches a real heading in
      CONTRIBUTING.md. `every_service_archetype_h3_is_either_trait_anchored_or_marked_policy_only`
      proves every H3 within the archetype section is either a trait anchor
      or carries a literal `policy-only` marker line in its own body.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/root_doc_allowlist_test.rs"
    action: create
    section: schema
    description: |
      Repo-root doc allowlist contract (meta-doc sheet 2, doc consolidation
      wave 2): the repo root carries exactly README.md, CONTRIBUTING.md,
      CLAUDE.md, and AGENTS.md; any other visible root-level
      `.md` file fails the suite naming the unexpected file and pointing at
      the CONTRIBUTING.md meta-doc content contract. Dotfiles (tool
      prompt/cache files) are out of scope.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/chain_liveness_test.rs"
    action: create
    section: schema
    description: |
      #921 tier 2 (epic #914 slice G): a bounded-tick ("livelock = failure")
      chain-liveness proof for the internal-lifecycle-verb layer (`aw td
      claim`, `aw td code-check` via `LocalBackend`, matching
      `td_no_merge_test.rs`'s established pattern), driving the real `aw`
      binary. Three tests: `chain_liveness_claim_never_lands_on_deadlock_phase`
      asserts a repeated `aw td claim --from-path` / `--force-rebase` loop
      never reports the `td_reviewed` phase (#843's fixed claim-deadlock) and
      converges on `td_created` within budget;
      `chain_liveness_code_check_terminates_within_tick_budget` asserts a clean
      `cb_filled` WI reaches `"action":"done"` in a single tick (code-check is
      the lifecycle's terminal step);
      `chain_liveness_code_check_retry_recovers_stranded_terminal_within_tick_budget`
      seeds a WI exactly where #846's partial-terminal-failure left one
      (phase already `td_merged`, no `Cb-CodeCheck` trailer commit landed yet)
      and asserts the retry converges within budget, lands exactly one
      trailer commit, and a second bounded-tick pass is an immediate,
      idempotent no-op (no duplicate commit). #842 (the `td-<slug>` branch
      never landing) already has dedicated coverage in
      `td_no_merge_test.rs::test_code_check_lands_td_slug_branch_onto_main`
      and is cross-referenced rather than duplicated. AC2's full
      `completion.workflow_complete=true` proof via `aw wi run` is
      deliberately out of scope here: that field lives on the root-driven
      runner envelope (`cli/run.rs`), and `aw wi`'s public verbs require a
      configured `github`/`gitlab` `issue_platform`/`repo_platform` in
      `.aw/config.toml` (`issues::resolve_default_backend` rejects `local`
      there by design), so it cannot be driven offline against a from-scratch
      sandbox repo — the internal-verb layer proven here is where #842/#843/
      #846 actually manifested and is where a regression would reappear.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/td_dirty_gate_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/inplace_mode_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
      Issue #1562 adds a deterministic real-CLI fixture with a canonical
      capability reference and an already-valid Logic plus Unit Test TD. It
      proves read-only `aw td check` reports zero findings, missing and
      malformed generic payloads leave the spec byte-identical, body-only
      Logic restores exactly one typed wrapper, and the initialized queue
      advances through applicability Unit Test before contract Logic.
      Issue #1598 adds the fresh default Logic -> Changes -> Unit Test real CLI
      lifecycle in both applicability and contract passes. It parses each
      initialized Changes JSON body as an editable target-plan skeleton,
      applies concrete Logic and Unit Test implementation edges, asserts every
      current section and payload path in the issue projection (including the
      first contract Logic lock), runs the final read-only checker, writes and
      commits the fixture TD IR lock, and proves `aw td gen` creates a new
      explicitly named target with source ownership and contract block refs.
      Issue #1602 adds a rewritten-history fixture whose old exact Td-Init is
      unreachable while a later same-slug lifecycle commit and stale lock
      survive. Real `aw td create` emits one recovery reset and one fresh init,
      replaces the old projection with the applicability Logic command and
      payload, and preserves existing spec/source bytes. Its negative fixture
      proves a reachable exact init emits neither reset nor second init; the
      project-branch fixture explicitly covers fresh WI phase `created`.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/cb_review_revise_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/merge_target_branch.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/td_claim_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/hook_pretooluse_write_scope.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/project_health_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/td_dispatch_chain_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/cb_review_to_merge_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/in_place_lifecycle_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/phase_migration_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/cb_fill_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/cb_claim_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
      Issue #1548 adds a real-CLI projection fixture that changes source bytes,
      runs exactly one uniquely named generated test, proves idempotence and
      sibling isolation, and rejects an unmatched existing target.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/tests/cli/tests/wi_close_remote_test.rs"
    action: create
    section: schema
    description: |
      Issue #1551 adds isolated real-CLI coverage for configured-backend
      numeric close rehydration, explicit repository selection, exactly-once
      close/reason mutation, actionable missing diagnostics, and preserved
      local close behavior. Issue #1583 is duplicate reproduction evidence.
    impl_mode: hand-written
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
