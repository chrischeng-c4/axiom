---
id: semantic-jet-tests
summary: Semantic coverage for "projects/jet/tests"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/tests"
  source_group: "projects/jet/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tests/install_dev_build_browser_lifecycle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "jet_cli_lifecycle_matrix_uses_shutdown_commands_and_persists_failure_artifacts"
            kind: "function"
            public: false
          - name: "run_lifecycle_matrix"
            kind: "function"
            public: false
          - name: "run_dev_browser_flow"
            kind: "function"
            public: false
          - name: "wait_for_dev_session"
            kind: "function"
            public: false
          - name: "shutdown_dev"
            kind: "function"
            public: false
          - name: "write_lifecycle_react_fixture"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/mui_visual_regression.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "mui_visual_fixture_renders_on_react_dom_and_jet_wasm"
            kind: "function"
            public: false
          - name: "run_dom_side"
            kind: "function"
            public: false
          - name: "run_wasm_side"
            kind: "function"
            public: false
          - name: "write_visual_mismatch_artifacts"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/build/production_build_regression.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "production_build_regression_fixture_boots_in_browser"
            kind: "function"
            public: false
          - name: "StaticDistServer"
            kind: "struct"
            public: false
          - name: "serve_static_dist_request"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/web_server_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "python_available"
            kind: "function"
            public: false
          - name: "free_port"
            kind: "function"
            public: false
          - name: "w_parse_full_toml"
            kind: "function"
            public: false
          - name: "w_parse_minimal_toml_uses_defaults"
            kind: "function"
            public: false
          - name: "w_boot_with_tcp_probe"
            kind: "function"
            public: false
          - name: "w_boot_with_http_probe"
            kind: "function"
            public: false
          - name: "w_boot_fails_fast_on_bad_command"
            kind: "function"
            public: false
          - name: "w_boot_times_out"
            kind: "function"
            public: false
          - name: "w_reuse_existing_skips_spawn"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_boolean_literal_state.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "FIXTURE"
            kind: "constant"
            public: false
          - name: "boolean_literal_initializer_yields_bool_turbofish"
            kind: "function"
            public: false
          - name: "numeric_literal_initializer_preserves_i64_default"
            kind: "function"
            public: false
          - name: "string_literal_initializer_yields_string_turbofish"
            kind: "function"
            public: false
          - name: "boolean_state_drives_conditional_render"
            kind: "function"
            public: false
          - name: "boolean_state_setter_lowers_unary_not"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_controlled_input.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "CONTROLLED_INPUT_TSX"
            kind: "constant"
            public: false
          - name: "controlled_input_lowers_value_placeholder_and_on_change"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/jet_test_api_compat.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "fixture_root"
            kind: "function"
            public: false
          - name: "copy_dir_recursive"
            kind: "function"
            public: false
          - name: "run_fixture"
            kind: "function"
            public: false
          - name: "api_compat_corpus_passes_under_jet_test"
            kind: "function"
            public: false
          - name: "api_compat_corpus_reports_name_each_guarded_behavior"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_effect_fetch.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "use_effect_fetch_lowers_to_wasm_host_bridge"
            kind: "function"
            public: false
          - name: "use_effect_non_empty_deps_fail_loudly"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/list_render_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "count_spans"
            kind: "function"
            public: false
          - name: "list_render_demo_grows_children_on_click"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/nested_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "intrinsic_depth"
            kind: "function"
            public: false
          - name: "nested_demo_preserves_tree_depth_and_hit_tests_through_layers"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/big_list_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "span_texts"
            kind: "function"
            public: false
          - name: "big_list_demo_renders_100_spans_in_order"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_counter.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "COUNTER_TSX"
            kind: "constant"
            public: false
          - name: "counter_transpiles_without_error"
            kind: "function"
            public: false
          - name: "generated_has_props_struct"
            kind: "function"
            public: false
          - name: "generated_has_render_fn"
            kind: "function"
            public: false
          - name: "generated_calls_use_state_with_i64"
            kind: "function"
            public: false
          - name: "generated_emits_jsx_as_element_intrinsic"
            kind: "function"
            public: false
          - name: "generated_emits_text_and_interpolation"
            kind: "function"
            public: false
          - name: "generated_has_factory_fn"
            kind: "function"
            public: false
          - name: "out_of_subset_fails_loudly"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/react_dom_oracle_conformance.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "browser_options"
            kind: "function"
            public: false
          - name: "workspace_root"
            kind: "function"
            public: false
          - name: "page_add_init_script_runs_before_navigation"
            kind: "function"
            public: false
          - name: "canvas_spy_records_draw_calls_before_page_script_runs"
            kind: "function"
            public: false
          - name: "counter_demo_matches_react_dom_oracle_initial_and_after_click"
            kind: "function"
            public: false
          - name: "jet_element_tree_normalizes_to_react_host_tree_shape"
            kind: "function"
            public: false
          - name: "paint_ops_map_to_canvas_method_sequence"
            kind: "function"
            public: false
          - name: "counter_demo_exposes_normalized_jet_tree_for_react_oracle"
            kind: "function"
            public: false
          - name: "react_dom_oracle_prerequisites_are_explicitly_gated"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/pm_report_acceptance.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "pm_fixture_bundle"
            kind: "function"
            public: false
          - name: "setup_source_with_screenshot"
            kind: "function"
            public: false
          - name: "pm_report_hides_pause_next_replay_dev_controls"
            kind: "function"
            public: false
          - name: "pm_report_preserves_failure_screenshot_assertion_and_step_context"
            kind: "function"
            public: false
          - name: "pm_report_works_from_static_files_without_runner_process"
            kind: "function"
            public: false
          - name: "pm_report_carries_no_open_control_protocol"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/fixture_timeout_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "hanging_fixture_terminates_with_timeout"
            kind: "function"
            public: false
          - name: "fixture_finishing_within_timeout_still_passes"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/parity_oracle_reexport.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "run_fixture_symbol_resolves"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/pkg-mgmt/workspace_protocol.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "write_file"
            kind: "function"
            public: false
          - name: "pkg_json"
            kind: "function"
            public: false
          - name: "test_pnpm_workspace_yaml_discovery"
            kind: "function"
            public: false
          - name: "test_jet_workspace_yaml_priority"
            kind: "function"
            public: false
          - name: "test_catalog_resolution"
            kind: "function"
            public: false
          - name: "test_workspace_mode_jet_detected_for_pnpm_yaml"
            kind: "function"
            public: false
          - name: "make_two_package_workspace"
            kind: "function"
            public: false
          - name: "test_workspace_star_symlink"
            kind: "function"
            public: false
          - name: "test_workspace_caret_resolution"
            kind: "function"
            public: false
          - name: "test_recursive_workspace_install"
            kind: "function"
            public: false
          - name: "test_no_registry_call_for_workspace_dep"
            kind: "function"
            public: false
          - name: "test_lockfile_workspace_fields"
            kind: "function"
            public: false
          - name: "test_idempotent_symlink_creation"
            kind: "function"
            public: false
          - name: "test_workspace_protocol_resolution_variants"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/browser_cli_smoke.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "free_port"
            kind: "function"
            public: false
          - name: "browser_cli_drives_debug_bridge_end_to_end"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/fixture_lifecycle_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "run_spec"
            kind: "function"
            public: false
          - name: "l1_per_test_setup_and_teardown_run_for_each_test"
            kind: "function"
            public: false
          - name: "l2_per_test_fixture_state_does_not_leak"
            kind: "function"
            public: false
          - name: "l3_teardown_runs_in_reverse_order"
            kind: "function"
            public: false
          - name: "l4_cleanup_failure_becomes_test_failure"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/usememo_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "tree_contains_text"
            kind: "function"
            public: false
          - name: "usememo_demo_tracks_state_across_click"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/storage_state_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "run_spec_str"
            kind: "function"
            public: false
          - name: "skip"
            kind: "function"
            public: false
          - name: "s_t1_add_cookies_roundtrip"
            kind: "function"
            public: false
          - name: "s_t2_clear_cookies"
            kind: "function"
            public: false
          - name: "s_t3_storage_state_shape"
            kind: "function"
            public: false
          - name: "s_t4_set_storage_state"
            kind: "function"
            public: false
          - name: "s_t5_file_roundtrip"
            kind: "function"
            public: false
          - name: "s_t6_unknown_context_error"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/self_closing_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "intrinsic_with_id"
            kind: "function"
            public: false
          - name: "self_closing_demo_renders_void_elements_as_leaf_intrinsics"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/html_reporter_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "make_report"
            kind: "function"
            public: false
          - name: "make_report_failed"
            kind: "function"
            public: false
          - name: "test_reporter_emits_index_html"
            kind: "function"
            public: false
          - name: "test_aggregate_stats_rendered"
            kind: "function"
            public: false
          - name: "test_test_row_contains_required_fields"
            kind: "function"
            public: false
          - name: "test_parser_parses_ndjson"
            kind: "function"
            public: false
          - name: "test_reporter_flag_parses"
            kind: "function"
            public: false
          - name: "test_merge_dedupes_by_test_id"
            kind: "function"
            public: false
          - name: "test_merge_shard_info_aggregated"
            kind: "function"
            public: false
          - name: "test_deterministic_output"
            kind: "function"
            public: false
          - name: "test_trace_link_reference"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/page_api_parity.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "run_spec_str"
            kind: "function"
            public: false
          - name: "parity_test_module_compiles"
            kind: "function"
            public: false
          - name: "page_request_title_serializes"
            kind: "function"
            public: false
          - name: "page_request_set_viewport_size_serializes"
            kind: "function"
            public: false
          - name: "page_request_screenshot_serializes"
            kind: "function"
            public: false
          - name: "page_response_screenshot_result_serializes"
            kind: "function"
            public: false
          - name: "page_request_navigation_variants_serialize"
            kind: "function"
            public: false
          - name: "page_request_keyboard_variants_serialize"
            kind: "function"
            public: false
          - name: "page_request_mouse_event_serializes"
            kind: "function"
            public: false
          - name: "page_request_set_content_serializes"
            kind: "function"
            public: false
          - name: "page_request_content_serializes"
            kind: "function"
            public: false
          - name: "page_request_bounding_box_serializes"
            kind: "function"
            public: false
          - name: "page_response_bounding_box_result_serializes"
            kind: "function"
            public: false
          - name: "page_request_hover_serializes"
            kind: "function"
            public: false
          - name: "page_request_locator_press_serializes"
            kind: "function"
            public: false
          - name: "page_request_event_subscription_variants_serialize"
            kind: "function"
            public: false
          - name: "parse_page_request_new_variants"
            kind: "function"
            public: false
          - name: "test_t1_page_title"
            kind: "function"
            public: false
          - name: "test_t2_set_viewport_size"
            kind: "function"
            public: false
          - name: "test_t3_wait_for_timeout"
            kind: "function"
            public: false
          - name: "test_t4_screenshot"
            kind: "function"
            public: false
          - name: "test_t5_page_on_console_api_surface"
            kind: "function"
            public: false
          - name: "test_t6_page_on_pageerror_api_surface"
            kind: "function"
            public: false
          - name: "test_t7_go_back"
            kind: "function"
            public: false
          - name: "test_t8_reload"
            kind: "function"
            public: false
          - name: "test_t9_keyboard_press"
            kind: "function"
            public: false
          - name: "test_t10_keyboard_type"
            kind: "function"
            public: false
          - name: "test_t11_mouse_click"
            kind: "function"
            public: false
          - name: "test_t12_set_content"
            kind: "function"
            public: false
          - name: "test_t13_content"
            kind: "function"
            public: false
          - name: "test_t14_bounding_box"
            kind: "function"
            public: false
          - name: "test_t15_is_visible_is_hidden"
            kind: "function"
            public: false
          - name: "test_t16_is_enabled"
            kind: "function"
            public: false
          - name: "test_t17_hover"
            kind: "function"
            public: false
          - name: "test_t18_locator_press"
            kind: "function"
            public: false
          - name: "test_t19_select_option"
            kind: "function"
            public: false
          - name: "test_t20_count"
            kind: "function"
            public: false
          - name: "test_t21_nth"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/trace_viewer.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "tempdir"
            kind: "function"
            public: false
          - name: "sample_zip"
            kind: "function"
            public: false
          - name: "test_http_server_binds_loopback"
            kind: "function"
            public: false
          - name: "test_trace_json_endpoint_matches_manifest"
            kind: "function"
            public: false
          - name: "test_asset_endpoint_returns_bytes"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/items_list_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "items_list_demo_element_tree_matches_snapshot"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/task-runner/nx_support.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "create_file"
            kind: "function"
            public: false
          - name: "write_file"
            kind: "function"
            public: false
          - name: "make_nx_graph"
            kind: "function"
            public: false
          - name: "test_detect_nx_workspace_when_nx_json_present"
            kind: "function"
            public: false
          - name: "test_detect_jet_workspace_when_package_json_has_workspaces"
            kind: "function"
            public: false
          - name: "test_detect_single_when_no_workspace_config"
            kind: "function"
            public: false
          - name: "test_detect_nx_takes_priority_over_package_json_workspaces"
            kind: "function"
            public: false
          - name: "test_detect_jet_workspace_from_yaml"
            kind: "function"
            public: false
          - name: "test_detect_returns_error_for_malformed_nx_json"
            kind: "function"
            public: false
          - name: "test_nx_manager_stores_root_path"
            kind: "function"
            public: false
          - name: "test_nx_manager_returns_none_without_nx_json"
            kind: "function"
            public: false
          - name: "test_graph_topological_sort_respects_dependencies"
            kind: "function"
            public: false
          - name: "test_graph_includes_all_projects_in_sort"
            kind: "function"
            public: false
          - name: "test_graph_project_names_returns_sorted_list"
            kind: "function"
            public: false
          - name: "test_graph_json_roundtrip"
            kind: "function"
            public: false
          - name: "test_workspace_discovery_reads_project_json_files_directly"
            kind: "function"
            public: false
          - name: "test_workspace_discovery_resolves_workspace_protocol_deps"
            kind: "function"
            public: false
          - name: "test_workspace_discovery_ignores_node_modules"
            kind: "function"
            public: false
          - name: "test_workspace_discovery_topological_order"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/locator_js_api.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "run_spec_str"
            kind: "function"
            public: false
          - name: "skip_if_no_browser"
            kind: "function"
            public: false
          - name: "test_t1_sub_locator_css_concat"
            kind: "function"
            public: false
          - name: "test_t2_sub_locator_pseudo_scope"
            kind: "function"
            public: false
          - name: "test_t3_filter_has_text_click"
            kind: "function"
            public: false
          - name: "test_t4_filter_regex"
            kind: "function"
            public: false
          - name: "test_t5_auto_wait_late_mount"
            kind: "function"
            public: false
          - name: "test_t6_auto_wait_timeout_hidden"
            kind: "function"
            public: false
          - name: "test_t7_stability_static"
            kind: "function"
            public: false
          - name: "test_t8_nth_click_indexed"
            kind: "function"
            public: false
          - name: "test_t9_nth_reads_indexed"
            kind: "function"
            public: false
          - name: "test_t10_chained_fill"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/trace_capture.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "tempdir"
            kind: "function"
            public: false
          - name: "test_trace_buffer_append_flush"
            kind: "function"
            public: false
          - name: "test_trace_zip_roundtrip"
            kind: "function"
            public: false
          - name: "test_retain_on_failure_discard_passing"
            kind: "function"
            public: false
          - name: "test_retain_on_failure_write_failing"
            kind: "function"
            public: false
          - name: "test_trace_off_no_cdp_calls"
            kind: "function"
            public: false
          - name: "test_trace_path_in_test_results_json"
            kind: "function"
            public: false
          - name: "test_all_event_types_captured"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/large_int_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "START"
            kind: "constant"
            public: false
          - name: "large_int_demo_survives_near_max_increments"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/jet_test_dogfood.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "dogfood_src_dir"
            kind: "function"
            public: false
          - name: "unit_and_integration_dogfood_specs_pass"
            kind: "function"
            public: false
          - name: "failure_fixture_produces_structured_result_data"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/matchers_state_value_a11y.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "run_spec_str"
            kind: "function"
            public: false
          - name: "skip"
            kind: "function"
            public: false
          - name: "test_m1a_to_be_checked_pass"
            kind: "function"
            public: false
          - name: "test_m1b_to_be_checked_timeout"
            kind: "function"
            public: false
          - name: "test_m2_m3_disabled_enabled"
            kind: "function"
            public: false
          - name: "test_m4_focused"
            kind: "function"
            public: false
          - name: "test_m5_css"
            kind: "function"
            public: false
          - name: "test_m6_accessible_name"
            kind: "function"
            public: false
          - name: "test_m7_role"
            kind: "function"
            public: false
          - name: "test_m8_match_object"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/classname_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "classname_demo_surfaces_class_name_in_element_tree"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/to_have_screenshot_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "run_spec_in_dir"
            kind: "function"
            public: false
          - name: "skip"
            kind: "function"
            public: false
          - name: "ts1_first_run_writes_second_matches"
            kind: "function"
            public: false
          - name: "ts2_named_baseline"
            kind: "function"
            public: false
          - name: "ts3_locator_rejected"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/playwright_compat_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "test_playwright_flag_delegates_to_playwright_runner"
            kind: "function"
            public: false
          - name: "test_playwright_test_import_routed_to_subprocess"
            kind: "function"
            public: false
          - name: "test_playwright_test_import_double_quotes"
            kind: "function"
            public: false
          - name: "test_non_playwright_import_not_detected"
            kind: "function"
            public: false
          - name: "test_deprecation_warning_printed_on_stderr"
            kind: "function"
            public: false
          - name: "test_suppress_warning_env_var"
            kind: "function"
            public: false
          - name: "test_suppress_warning_non_one_value_does_not_suppress"
            kind: "function"
            public: false
          - name: "test_reporter_flag_conflict_exits_2"
            kind: "function"
            public: false
          - name: "test_trace_flag_conflict_exits_2"
            kind: "function"
            public: false
          - name: "test_workers_flag_conflict_exits_2"
            kind: "function"
            public: false
          - name: "test_shard_flag_conflict_exits_2"
            kind: "function"
            public: false
          - name: "test_report_dir_flag_conflict_exits_2"
            kind: "function"
            public: false
          - name: "test_no_native_flag_conflict_ok"
            kind: "function"
            public: false
          - name: "test_native_runner_unaffected_without_playwright_flag"
            kind: "function"
            public: false
          - name: "test_migration_guide_exists_and_complete"
            kind: "function"
            public: false
          - name: "test_e2e_playwright_fixture_spec_exit_0"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/test_runner_smoke.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "runs_basic_spec_and_reports_pass_fail_skip"
            kind: "function"
            public: false
          - name: "unit_test_surface_hooks_fixtures_and_matchers"
            kind: "function"
            public: false
          - name: "jet_test_contract_introspection_and_tripwires"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/build/tree_shaking.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "snapshot_dir"
            kind: "function"
            public: false
          - name: "snapshot_eq"
            kind: "function"
            public: false
          - name: "canonicalize"
            kind: "function"
            public: false
          - name: "sort_keys"
            kind: "function"
            public: false
          - name: "result_to_value"
            kind: "function"
            public: false
          - name: "fixture"
            kind: "function"
            public: false
          - name: "unused_named_exports_are_stripped"
            kind: "function"
            public: false
          - name: "side_effect_free_module_is_eliminated"
            kind: "function"
            public: false
          - name: "side_effect_full_module_is_preserved"
            kind: "function"
            public: false
          - name: "reexport_chain_partial_usage_baseline"
            kind: "function"
            public: false
          - name: "reexport_star_marks_all_leaf_exports_used"
            kind: "function"
            public: false
          - name: "reexport_renamed_threads_to_original_leaf_name"
            kind: "function"
            public: false
          - name: "dynamic_import_retained_baseline"
            kind: "function"
            public: false
          - name: "dynamic_import_retained_await"
            kind: "function"
            public: false
          - name: "class_unused_methods_documented_baseline"
            kind: "function"
            public: false
          - name: "mixed_esm_cjs_interop_baseline"
            kind: "function"
            public: false
          - name: "conditional_exports_pruning_browser_condition"
            kind: "function"
            public: false
          - name: "post_shake_size"
            kind: "function"
            public: false
          - name: "mini_react_e2e_baseline"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/string_state_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "string_state_demo_surfaces_string_type_and_value"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/wasm_dev_smoke.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "free_port"
            kind: "function"
            public: false
          - name: "dev_server_serves_wasm_bundle"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/no_state_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "no_state_demo_runs_with_zero_hooks"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/toggle_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "toggle_demo_flips_bool_and_conditionally_renders_span"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/e2e_playwright_residue.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "e2e_playwright_residue_absent"
            kind: "function"
            public: false
          - name: "walk"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_ast_probe.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "COUNTER_TSX"
            kind: "constant"
            public: false
          - name: "print_tree"
            kind: "function"
            public: false
          - name: "dump_counter_ast"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/worker_pool_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "make_spec"
            kind: "function"
            public: false
          - name: "make_specs_on_disk"
            kind: "function"
            public: false
          - name: "test_workers_bounds_concurrency"
            kind: "function"
            public: false
          - name: "test_workers_one_is_serial"
            kind: "function"
            public: false
          - name: "test_shard_partition_selects_ith_bucket"
            kind: "function"
            public: false
          - name: "test_shard_partition_stable_across_runs"
            kind: "function"
            public: false
          - name: "test_shard_partition_covers_all_specs"
            kind: "function"
            public: false
          - name: "test_crashed_worker_surfaces_errored"
            kind: "function"
            public: false
          - name: "test_trace_filename_includes_shard_tag"
            kind: "function"
            public: false
          - name: "test_ndjson_contains_shard_fields"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_imports.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "react_imports_do_not_block_transpile"
            kind: "function"
            public: false
          - name: "type_only_and_css_imports_do_not_block_transpile"
            kind: "function"
            public: false
          - name: "runtime_imports_fail_with_wasm_diagnostic"
            kind: "function"
            public: false
          - name: "local_runtime_imports_fail_before_silent_drop"
            kind: "function"
            public: false
          - name: "compat_lowering_maps_mui_imports_to_wasm_intrinsics"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/auto_artifacts_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "RUNNER_HARD_TIMEOUT"
            kind: "constant"
            public: false
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "run_spec"
            kind: "function"
            public: false
          - name: "skip"
            kind: "function"
            public: false
          - name: "aa1_failing_test_produces_artifact"
            kind: "function"
            public: false
          - name: "aa2_disabled_produces_empty"
            kind: "function"
            public: false
          - name: "aa3_multi_page_capture"
            kind: "function"
            public: false
          - name: "aa4_passing_test_no_artifacts"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/route_intercept_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "run_spec"
            kind: "function"
            public: false
          - name: "skip"
            kind: "function"
            public: false
          - name: "ri1_fetch_glob_mock"
            kind: "function"
            public: false
          - name: "ri2_fetch_regex_mock"
            kind: "function"
            public: false
          - name: "ri3_unmatched_fetch_passthrough"
            kind: "function"
            public: false
          - name: "ri4_fetch_abort_rejects"
            kind: "function"
            public: false
          - name: "ri5_ri6_unroute_and_unroute_all"
            kind: "function"
            public: false
          - name: "ri7_xhr_mock"
            kind: "function"
            public: false
          - name: "ri8_xhr_abort_onerror"
            kind: "function"
            public: false
          - name: "ri9_first_match_wins"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/text_snapshot_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "run_spec_with"
            kind: "function"
            public: false
          - name: "t1_first_run_writes_baseline_and_passes"
            kind: "function"
            public: false
          - name: "t2_matching_baseline_passes"
            kind: "function"
            public: false
          - name: "t3_mismatch_fails_with_diff_and_preserves_baseline"
            kind: "function"
            public: false
          - name: "t4_update_snapshots_overwrites_baseline_and_passes"
            kind: "function"
            public: false
          - name: "t5_object_snapshot_uses_stable_key_order"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_toggle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "TOGGLE_TSX"
            kind: "constant"
            public: false
          - name: "toggle_transpiles_without_error"
            kind: "function"
            public: false
          - name: "bool_prop_yields_bool_turbofish"
            kind: "function"
            public: false
          - name: "unary_not_in_setter_call"
            kind: "function"
            public: false
          - name: "nested_jsx_produces_nested_intrinsics"
            kind: "function"
            public: false
          - name: "self_closing_emits_empty_children"
            kind: "function"
            public: false
          - name: "conditional_render_uses_if_else"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/browser_install.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "install_chromium_downloads_and_is_launchable"
            kind: "function"
            public: false
          - name: "find_chrome_prefers_cache"
            kind: "function"
            public: false
          - name: "unsupported_platform_returns_clear_error"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/page_fixture_auto_inject.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "glob_first"
            kind: "function"
            public: false
          - name: "run_spec_str"
            kind: "function"
            public: false
          - name: "test_runner_config_default_for_root_constructs"
            kind: "function"
            public: false
          - name: "test_page_fixture_auto_injected_into_test_body"
            kind: "function"
            public: false
          - name: "test_page_auto_closed_after_test"
            kind: "function"
            public: false
          - name: "test_page_auto_closed_on_test_failure"
            kind: "function"
            public: false
          - name: "test_browser_shared_across_tests_in_worker"
            kind: "function"
            public: false
          - name: "test_baseurl_resolution_relative_path"
            kind: "function"
            public: false
          - name: "test_user_extend_page_overrides_default"
            kind: "function"
            public: false
          - name: "test_user_fixture_receives_cdp_page_as_dependency"
            kind: "function"
            public: false
          - name: "test_no_page_no_injection"
            kind: "function"
            public: false
          - name: "test_cdp_launch_failure_error_message"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/product_step_timeline.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "cue_step"
            kind: "function"
            public: false
          - name: "cue_artifact_studio_bundle"
            kind: "function"
            public: false
          - name: "dogfood_case_uses_named_product_steps"
            kind: "function"
            public: false
          - name: "evidence_carries_ordered_step_records"
            kind: "function"
            public: false
          - name: "open_mode_timeline_consumes_same_step_records_as_run_mode"
            kind: "function"
            public: false
          - name: "step_events_carry_start_end_duration_and_failure_context"
            kind: "function"
            public: false
          - name: "kind"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/pm_report_static_smoke.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "fixture_with_artifacts"
            kind: "function"
            public: false
          - name: "setup_source_root"
            kind: "function"
            public: false
          - name: "static_report_directory_is_self_contained"
            kind: "function"
            public: false
          - name: "static_report_html_embeds_no_runner_endpoints"
            kind: "function"
            public: false
          - name: "static_report_renders_failure_artifacts_from_relative_paths"
            kind: "function"
            public: false
          - name: "static_report_can_load_without_running_jet_services"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/cue_artifact_studio_dogfood.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "fixture_spec_path"
            kind: "function"
            public: false
          - name: "cue_summary_with"
            kind: "function"
            public: false
          - name: "report"
            kind: "function"
            public: false
          - name: "assert_bundle_shape"
            kind: "function"
            public: false
          - name: "fixture_is_present_and_self_contained"
            kind: "function"
            public: false
          - name: "run_mode_evidence_carries_product_steps_for_cue_flow"
            kind: "function"
            public: false
          - name: "open_mode_evidence_inspects_same_flow"
            kind: "function"
            public: false
          - name: "failure_path_carries_assertion_context"
            kind: "function"
            public: false
          - name: "fixture_path_resolves_relative_to_jet_crate"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/playwright_compat_shim_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "run_spec"
            kind: "function"
            public: false
          - name: "pc1_named_imports"
            kind: "function"
            public: false
          - name: "pc2_browser_namespace"
            kind: "function"
            public: false
          - name: "pc3_default_namespace"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_i18n_probe.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "TSX"
            kind: "constant"
            public: false
          - name: "print_tree"
            kind: "function"
            public: false
          - name: "dump_i18n_ast"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/tsx_to_rust_i18n_copy_constants.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "FIXTURE"
            kind: "constant"
            public: false
          - name: "top_level_string_const_passes_through"
            kind: "function"
            public: false
          - name: "top_level_object_const_passes_through"
            kind: "function"
            public: false
          - name: "in_component_string_const_lowers_to_let"
            kind: "function"
            public: false
          - name: "member_access_in_jsx_resolves"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/wasm_build_end_to_end.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "counter_demo_builds_and_renders_on_canvas"
            kind: "function"
            public: false
          - name: "webgpu_renderer_build_selects_webgpu_scaffold"
            kind: "function"
            public: false
          - name: "wasm_build_bundles_css_side_effect_imports"
            kind: "function"
            public: false
          - name: "wasm_build_compat_lowers_mui_runtime_imports"
            kind: "function"
            public: false
          - name: "use_effect_fetch_reaches_host_api_from_wasm"
            kind: "function"
            public: false
          - name: "cue_artifact_studio_dom_wasm_loads_api_and_posts"
            kind: "function"
            public: false
          - name: "webgpu_renderer_reports_runtime_status_when_available"
            kind: "function"
            public: false
          - name: "write_css_import_fixture"
            kind: "function"
            public: false
          - name: "write_mui_compat_fixture"
            kind: "function"
            public: false
          - name: "write_webgpu_fixture"
            kind: "function"
            public: false
          - name: "spawn_static_server"
            kind: "function"
            public: false
          - name: "ApiStaticState"
            kind: "struct"
            public: false
          - name: "spawn_static_server_with_api"
            kind: "function"
            public: false
          - name: "CueWasmState"
            kind: "struct"
            public: false
          - name: "spawn_cue_wasm_server"
            kind: "function"
            public: false
          - name: "handle_cue_api_projects"
            kind: "function"
            public: false
          - name: "handle_cue_post_project"
            kind: "function"
            public: false
          - name: "handle_cue_post_message"
            kind: "function"
            public: false
          - name: "handle_cue_wasm_index"
            kind: "function"
            public: false
          - name: "handle_cue_wasm_static"
            kind: "function"
            public: false
          - name: "cue_projects_json"
            kind: "function"
            public: false
          - name: "wait_for_body_text"
            kind: "function"
            public: false
          - name: "wait_for_counter"
            kind: "function"
            public: false
          - name: "handle_api_projects"
            kind: "function"
            public: false
          - name: "handle_api_index"
            kind: "function"
            public: false
          - name: "handle_api_static"
            kind: "function"
            public: false
          - name: "handle_index"
            kind: "function"
            public: false
          - name: "handle_static"
            kind: "function"
            public: false
          - name: "serve_file"
            kind: "function"
            public: false
          - name: "not_found"
            kind: "function"
            public: false
          - name: "content_type_for"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser-bridge/browser_context.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "chromium_available"
            kind: "function"
            public: false
          - name: "new_context_request_serializes"
            kind: "function"
            public: false
          - name: "close_context_request_serializes"
            kind: "function"
            public: false
          - name: "context_new_page_request_serializes"
            kind: "function"
            public: false
          - name: "context_result_response_serializes"
            kind: "function"
            public: false
          - name: "context_variants_round_trip_serde"
            kind: "function"
            public: false
          - name: "browser_launch_exposes_default_and_new_context"
            kind: "function"
            public: false
          - name: "pages_carry_context_id_only_for_user_contexts"
            kind: "function"
            public: false
          - name: "two_contexts_are_isolated_by_target_listing"
            kind: "function"
            public: false
          - name: "closed_context_rejects_new_page"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/build/bundler_monorepo.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "write_file"
            kind: "function"
            public: false
          - name: "test_bundler_circular_dependency_completes"
            kind: "function"
            public: false
          - name: "test_bundler_three_module_cycle_completes"
            kind: "function"
            public: false
          - name: "test_bundler_resolves_monorepo_workspace_root_package"
            kind: "function"
            public: false
          - name: "test_bundler_project_node_modules_takes_priority"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/test-runner/fixture_di_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "node_available"
            kind: "function"
            public: false
          - name: "run_spec"
            kind: "function"
            public: false
          - name: "fd1_advanced_depends_on_flat"
            kind: "function"
            public: false
          - name: "fd2_two_deep_chain"
            kind: "function"
            public: false
          - name: "fd3_shared_dep_resolved_once"
            kind: "function"
            public: false
          - name: "fd4_cycle_detected"
            kind: "function"
            public: false
          - name: "fd5_undefined_dep"
            kind: "function"
            public: false
          - name: "fd6_flat_still_works"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/multi_handler_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "hook_i64"
            kind: "function"
            public: false
          - name: "multi_handler_demo_isolates_state_per_button"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm/unicode_debug.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "common"
            kind: "module"
            public: false
          - name: "GREETING"
            kind: "constant"
            public: false
          - name: "unicode_demo_roundtrips_cjk_emoji_and_cyrillic"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/build/incremental_rebuild_bench.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method", "test_case"]
        symbols:
          - name: "MODULE_COUNT"
            kind: "constant"
            public: false
          - name: "R6_CEILING_MS"
            kind: "constant"
            public: false
          - name: "js_lang"
            kind: "function"
            public: false
          - name: "build_500_module_graph"
            kind: "function"
            public: false
          - name: "u_basename"
            kind: "function"
            public: false
          - name: "prime_cache"
            kind: "function"
            public: false
          - name: "cold_full_graph_baseline"
            kind: "function"
            public: false
          - name: "single_leaf_change_warm_cache_under_100ms"
            kind: "function"
            public: false
          - name: "barrel_cascade_under_2x_leaf_baseline"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/jet/tests/test-runner/web_server_tests.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_boolean_literal_state.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_controlled_input.rs"
      - path: "projects/jet/tests/test-runner/jet_test_api_compat.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_effect_fetch.rs"
      - path: "projects/jet/tests/wasm/list_render_debug.rs"
      - path: "projects/jet/tests/wasm/nested_debug.rs"
      - path: "projects/jet/tests/wasm/big_list_debug.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_counter.rs"
      - path: "projects/jet/tests/wasm/react_dom_oracle_conformance.rs"
      - path: "projects/jet/tests/browser-bridge/pm_report_acceptance.rs"
      - path: "projects/jet/tests/test-runner/fixture_timeout_tests.rs"
      - path: "projects/jet/tests/wasm/parity_oracle_reexport.rs"
      - path: "projects/jet/tests/pkg-mgmt/workspace_protocol.rs"
      - path: "projects/jet/tests/browser-bridge/browser_cli_smoke.rs"
      - path: "projects/jet/tests/test-runner/fixture_lifecycle_tests.rs"
      - path: "projects/jet/tests/wasm/usememo_debug.rs"
      - path: "projects/jet/tests/browser-bridge/storage_state_tests.rs"
      - path: "projects/jet/tests/wasm/self_closing_debug.rs"
      - path: "projects/jet/tests/test-runner/html_reporter_tests.rs"
      - path: "projects/jet/tests/browser-bridge/page_api_parity.rs"
      - path: "projects/jet/tests/browser-bridge/trace_viewer.rs"
      - path: "projects/jet/tests/wasm/items_list_debug.rs"
      - path: "projects/jet/tests/task-runner/nx_support.rs"
      - path: "projects/jet/tests/browser-bridge/locator_js_api.rs"
      - path: "projects/jet/tests/browser-bridge/trace_capture.rs"
      - path: "projects/jet/tests/wasm/large_int_debug.rs"
      - path: "projects/jet/tests/test-runner/jet_test_dogfood.rs"
      - path: "projects/jet/tests/browser-bridge/matchers_state_value_a11y.rs"
      - path: "projects/jet/tests/wasm/classname_debug.rs"
      - path: "projects/jet/tests/browser-bridge/to_have_screenshot_tests.rs"
      - path: "projects/jet/tests/browser-bridge/playwright_compat_tests.rs"
      - path: "projects/jet/tests/test-runner/test_runner_smoke.rs"
      - path: "projects/jet/tests/build/tree_shaking.rs"
      - path: "projects/jet/tests/wasm/string_state_debug.rs"
      - path: "projects/jet/tests/wasm/wasm_dev_smoke.rs"
      - path: "projects/jet/tests/wasm/no_state_debug.rs"
      - path: "projects/jet/tests/wasm/toggle_debug.rs"
      - path: "projects/jet/tests/browser-bridge/e2e_playwright_residue.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_ast_probe.rs"
      - path: "projects/jet/tests/test-runner/worker_pool_tests.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_imports.rs"
      - path: "projects/jet/tests/browser-bridge/auto_artifacts_tests.rs"
      - path: "projects/jet/tests/browser-bridge/route_intercept_tests.rs"
      - path: "projects/jet/tests/test-runner/text_snapshot_tests.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_toggle.rs"
      - path: "projects/jet/tests/browser-bridge/browser_install.rs"
      - path: "projects/jet/tests/browser-bridge/page_fixture_auto_inject.rs"
      - path: "projects/jet/tests/browser-bridge/product_step_timeline.rs"
      - path: "projects/jet/tests/browser-bridge/pm_report_static_smoke.rs"
      - path: "projects/jet/tests/browser-bridge/cue_artifact_studio_dogfood.rs"
      - path: "projects/jet/tests/browser-bridge/playwright_compat_shim_tests.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_i18n_probe.rs"
      - path: "projects/jet/tests/wasm/tsx_to_rust_i18n_copy_constants.rs"
      - path: "projects/jet/tests/wasm/wasm_build_end_to_end.rs"
      - path: "projects/jet/tests/browser-bridge/browser_context.rs"
      - path: "projects/jet/tests/build/bundler_monorepo.rs"
      - path: "projects/jet/tests/test-runner/fixture_di_tests.rs"
      - path: "projects/jet/tests/wasm/multi_handler_debug.rs"
      - path: "projects/jet/tests/wasm/unicode_debug.rs"
      - path: "projects/jet/tests/build/incremental_rebuild_bench.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/tests/test-runner/web_server_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_boolean_literal_state.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_controlled_input.rs"
    action: modify
    section: schema
    description: |
      Controlled input TSX lowering behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/jet_test_api_compat.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_effect_fetch.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/list_render_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/nested_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/big_list_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_counter.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/react_dom_oracle_conformance.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/pm_report_acceptance.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/fixture_timeout_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/parity_oracle_reexport.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/pkg-mgmt/workspace_protocol.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/browser_cli_smoke.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/fixture_lifecycle_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/usememo_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/storage_state_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/self_closing_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/html_reporter_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/page_api_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/trace_viewer.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/items_list_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/task-runner/nx_support.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/locator_js_api.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/trace_capture.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/large_int_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/jet_test_dogfood.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/matchers_state_value_a11y.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/classname_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/to_have_screenshot_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/playwright_compat_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/test_runner_smoke.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/tree_shaking.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/string_state_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/wasm_dev_smoke.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/no_state_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/toggle_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/e2e_playwright_residue.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_ast_probe.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/worker_pool_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_imports.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/auto_artifacts_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/route_intercept_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/text_snapshot_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_toggle.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/browser_install.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/page_fixture_auto_inject.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/product_step_timeline.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/pm_report_static_smoke.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/cue_artifact_studio_dogfood.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/playwright_compat_shim_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_i18n_probe.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/tsx_to_rust_i18n_copy_constants.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/wasm_build_end_to_end.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser-bridge/browser_context.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/bundler_monorepo.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/test-runner/fixture_di_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/multi_handler_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/wasm/unicode_debug.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/incremental_rebuild_bench.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-tests.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
