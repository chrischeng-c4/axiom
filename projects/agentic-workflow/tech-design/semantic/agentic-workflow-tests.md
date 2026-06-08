---
id: semantic-agentic-workflow-tests
summary: Semantic coverage for "projects/agentic-workflow/tests"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: cb-and-cold-verification-gates
    claim: cb-and-cold-verification-gates
    coverage: full
    rationale: "This semantic TD covers verification source behavior used by existing-project standardization gates."
---

# Semantic TD: agentic-workflow/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/tests"
  source_group: "projects/agentic-workflow/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/tests/issues_remote_round_trip.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "gh_authenticated"
            kind: "function"
            public: false
          - name: "sandbox_repo"
            kind: "function"
            public: false
          - name: "fixture_issue"
            kind: "function"
            public: false
          - name: "github_backend_round_trips_crrr_state_via_labels"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/execution_modes_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "project_root"
            kind: "function"
            public: false
          - name: "test_legacy_agent_pretooluse_hooks_are_not_required"
            kind: "function"
            public: false
          - name: "test_mainthread_hooks_are_not_required"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/validate_all_snapshot.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "write"
            kind: "function"
            public: false
          - name: "VALID_PROSE"
            kind: "constant"
            public: false
          - name: "VALID_STRUCTURAL"
            kind: "constant"
            public: false
          - name: "VALID_MERMAID_PLUS"
            kind: "constant"
            public: false
          - name: "INVALID_STRUCTURAL"
            kind: "constant"
            public: false
          - name: "INVALID_MERMAID"
            kind: "constant"
            public: false
          - name: "validate_all_snapshot_matches_expected_violations"
            kind: "function"
            public: false
          - name: "validate_all_emits_zero_findings_on_clean_input"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/sync_check_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "make_score_root"
            kind: "function"
            public: false
          - name: "write_config"
            kind: "function"
            public: false
          - name: "check_targets_config_toml"
            kind: "function"
            public: false
          - name: "check_no_drift_when_up_to_date"
            kind: "function"
            public: false
          - name: "check_does_not_modify_config_toml"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/cli_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "cb_claim_test"
            kind: "module"
            public: false
          - name: "cb_fill_test"
            kind: "module"
            public: false
          - name: "cb_namespace_test"
            kind: "module"
            public: false
          - name: "cb_review_revise_test"
            kind: "module"
            public: false
          - name: "cb_review_to_merge_test"
            kind: "module"
            public: false
          - name: "hook_pretooluse_write_scope"
            kind: "module"
            public: false
          - name: "in_place_lifecycle_test"
            kind: "module"
            public: false
          - name: "inplace_mode_test"
            kind: "module"
            public: false
          - name: "legacy_cli_removal_test"
            kind: "module"
            public: false
          - name: "merge_target_branch"
            kind: "module"
            public: false
          - name: "phase_migration_test"
            kind: "module"
            public: false
          - name: "project_health_test"
            kind: "module"
            public: false
          - name: "recovery_flow_test"
            kind: "module"
            public: false
          - name: "standardize_test"
            kind: "module"
            public: false
          - name: "td_check_test"
            kind: "module"
            public: false
          - name: "td_claim_test"
            kind: "module"
            public: false
          - name: "td_dirty_gate_test"
            kind: "module"
            public: false
          - name: "td_dispatch_chain_test"
            kind: "module"
            public: false
          - name: "td_merge_atomic_test"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/codegen_full_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "project_root"
            kind: "function"
            public: false
          - name: "walk_md_files"
            kind: "function"
            public: false
          - name: "has_changes_section"
            kind: "function"
            public: false
          - name: "try_apply"
            kind: "function"
            public: false
          - name: "test_all_specs_no_panic"
            kind: "function"
            public: false
          - name: "test_gentest_produces_all_files"
            kind: "function"
            public: false
          - name: "test_task_state_machine_roundtrip"
            kind: "function"
            public: false
          - name: "dump_task_state_machine_to_tmp"
            kind: "function"
            public: false
          - name: "test_minimal_frontmatter_skips_gracefully"
            kind: "function"
            public: false
          - name: "test_rpc_api_and_config_dispatchers"
            kind: "function"
            public: false
          - name: "test_unsupported_language_errors_loud"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/project_discovery_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "rule_e_package_name"
            kind: "function"
            public: false
          - name: "test_cmd_relative_path_python"
            kind: "function"
            public: false
          - name: "test_cmd_relative_path_typescript"
            kind: "function"
            public: false
          - name: "rule_a_be_test_cmd_is_relative"
            kind: "function"
            public: false
          - name: "rule_e_fallback_to_dir_basename_when_no_package_name"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/spec_alignment_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "spec_alignment_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/sdd_viewer_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "sdd_viewer_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/python_backend_emitter.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "SPEC_ID"
            kind: "constant"
            public: false
          - name: "orders_router"
            kind: "function"
            public: false
          - name: "order_create"
            kind: "function"
            public: false
          - name: "order_response"
            kind: "function"
            public: false
          - name: "router_emit_is_byte_equivalent"
            kind: "function"
            public: false
          - name: "pydantic_model_emit_is_byte_equivalent"
            kind: "function"
            public: false
          - name: "pydantic_model_emits_defaults_and_optional_fields"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/github_backend_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "env_lock"
            kind: "function"
            public: false
          - name: "write_mock_gh"
            kind: "function"
            public: false
          - name: "with_token"
            kind: "function"
            public: false
          - name: "without_token"
            kind: "function"
            public: false
          - name: "create_happy_path"
            kind: "function"
            public: false
          - name: "auth_missing_returns_error"
            kind: "function"
            public: false
          - name: "list_open_issues_maps_state"
            kind: "function"
            public: false
          - name: "read_by_id_returns_body"
            kind: "function"
            public: false
          - name: "update_returns_unsupported_in_slice_1"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/project_registry_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "make_score_root"
            kind: "function"
            public: false
          - name: "make_project"
            kind: "function"
            public: false
          - name: "write_config"
            kind: "function"
            public: false
          - name: "marker_upsert_first_run"
            kind: "function"
            public: false
          - name: "marker_upsert_round_trip"
            kind: "function"
            public: false
          - name: "load_projects_reads_workspace_codegen_profile"
            kind: "function"
            public: false
          - name: "migration_deletes_projects_toml"
            kind: "function"
            public: false
          - name: "check_drift_references_config_toml_in_output"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/gitlab_backend_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "env_lock"
            kind: "function"
            public: false
          - name: "write_mock_glab"
            kind: "function"
            public: false
          - name: "create_happy_path"
            kind: "function"
            public: false
          - name: "auth_missing_returns_error"
            kind: "function"
            public: false
          - name: "list_open_issues_maps_state_and_labels"
            kind: "function"
            public: false
          - name: "read_by_id_returns_body"
            kind: "function"
            public: false
          - name: "update_returns_unsupported_in_slice_1"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/lens_dissolution_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "project_root"
            kind: "function"
            public: false
          - name: "sdd_src"
            kind: "function"
            public: false
          - name: "r2_no_top_level_types_dir"
            kind: "function"
            public: false
          - name: "r4_no_pub_mod_lens_in_lib"
            kind: "function"
            public: false
          - name: "r5_no_residual_lens_imports"
            kind: "function"
            public: false
          - name: "nf3_lens_directory_deleted"
            kind: "function"
            public: false
          - name: "r6_old_lens_spec_directory_deleted"
            kind: "function"
            public: false
          - name: "r6_sdd_logic_specs_contain_migrated_lens_specs"
            kind: "function"
            public: false
          - name: "nf1_core_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_diagnostic_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_syntax_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_lint_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_lens_error_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_checker_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_watch_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_spec_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_gen_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_server_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_semantic_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_refactoring_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_format_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_graph_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_schemas_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_search_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_storage_accessible"
            kind: "function"
            public: false
          - name: "r2_type_inference_types_accessible"
            kind: "function"
            public: false
          - name: "nf1_multiparser_detects_python"
            kind: "function"
            public: false
          - name: "nf1_multiparser_detects_typescript"
            kind: "function"
            public: false
          - name: "nf1_multiparser_detects_rust"
            kind: "function"
            public: false
          - name: "nf1_lint_config_default_languages"
            kind: "function"
            public: false
          - name: "nf1_lint_config_excludes_defaults"
            kind: "function"
            public: false
          - name: "nf1_checker_registry_has_python"
            kind: "function"
            public: false
          - name: "nf1_checker_registry_has_typescript"
            kind: "function"
            public: false
          - name: "nf1_checker_registry_has_rust"
            kind: "function"
            public: false
          - name: "nf1_argus_config_default"
            kind: "function"
            public: false
          - name: "nf1_diagnostic_severity_ordering"
            kind: "function"
            public: false
          - name: "nf1_watch_config_default"
            kind: "function"
            public: false
          - name: "create_python_fixtures"
            kind: "function"
            public: false
          - name: "create_clean_fixtures"
            kind: "function"
            public: false
          - name: "create_polyglot_fixtures"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/sdd_coverage_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/python_backend_emitter_real.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "SPEC_ID"
            kind: "constant"
            public: false
          - name: "SPEC_REF"
            kind: "constant"
            public: false
          - name: "workspace_root"
            kind: "function"
            public: false
          - name: "target_path"
            kind: "function"
            public: false
          - name: "video_qc_spec"
            kind: "function"
            public: false
          - name: "extract_codegen_body"
            kind: "function"
            public: false
          - name: "video_qc_api_models_is_byte_equivalent"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/spec_alignment_phase2_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "spec_alignment_phase2_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/stage_2b_golden_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "stage_2b_golden_tests_pending"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
      - path: "projects/agentic-workflow/tests/from_td_ast_dispatch.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "fixture_spec"
            kind: "function"
            public: false
          - name: "dispatch_classifies_each_section"
            kind: "function"
            public: false
          - name: "empty_spec_yields_empty_report"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/tests"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/agentic-workflow/tests/issues_remote_round_trip.rs"
      - path: "projects/agentic-workflow/tests/execution_modes_test.rs"
      - path: "projects/agentic-workflow/tests/validate_all_snapshot.rs"
      - path: "projects/agentic-workflow/tests/sync_check_test.rs"
      - path: "projects/agentic-workflow/tests/cli_tests.rs"
      - path: "projects/agentic-workflow/tests/codegen_full_test.rs"
      - path: "projects/agentic-workflow/tests/project_discovery_test.rs"
      - path: "projects/agentic-workflow/tests/spec_alignment_tests.rs"
      - path: "projects/agentic-workflow/tests/sdd_viewer_test.rs"
      - path: "projects/agentic-workflow/tests/python_backend_emitter.rs"
      - path: "projects/agentic-workflow/tests/github_backend_tests.rs"
      - path: "projects/agentic-workflow/tests/project_registry_test.rs"
      - path: "projects/agentic-workflow/tests/gitlab_backend_tests.rs"
      - path: "projects/agentic-workflow/tests/lens_dissolution_test.rs"
      - path: "projects/agentic-workflow/tests/sdd_coverage_test.rs"
      - path: "projects/agentic-workflow/tests/python_backend_emitter_real.rs"
      - path: "projects/agentic-workflow/tests/spec_alignment_phase2_tests.rs"
      - path: "projects/agentic-workflow/tests/stage_2b_golden_tests.rs"
      - path: "projects/agentic-workflow/tests/from_td_ast_dispatch.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/agentic-workflow/tests/issues_remote_round_trip.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/execution_modes_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/validate_all_snapshot.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/sync_check_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/cli_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/codegen_full_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/project_discovery_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/spec_alignment_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/sdd_viewer_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/python_backend_emitter.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/github_backend_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/project_registry_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/gitlab_backend_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/lens_dissolution_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/sdd_coverage_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/python_backend_emitter_real.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/spec_alignment_phase2_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/stage_2b_golden_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/tests/from_td_ast_dispatch.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
