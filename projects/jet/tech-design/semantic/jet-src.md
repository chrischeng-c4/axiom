---
id: semantic-jet-src
summary: Semantic coverage for "projects/jet/src"
capability_refs:
  - id: "rust-native-frontend-toolchain"
    role: primary
    claim: "production-replacement-readiness"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/jet/src`."
fill_sections: [schema, changes]
---

# Semantic TD: jet/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/src"
  source_group: "projects/jet/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/frontend.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "FrontendSources"
            kind: "struct"
            public: true
          - name: "LocalComponentImport"
            kind: "struct"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "load_html_template"
            kind: "function"
            public: true
          - name: "default_index_html"
            kind: "function"
            public: true
          - name: "extract_css_side_effect_imports"
            kind: "function"
            public: true
          - name: "extract_local_component_imports"
            kind: "function"
            public: true
          - name: "resolve_local_import_path"
            kind: "function"
            public: true
          - name: "contains_function_component"
            kind: "function"
            public: true
          - name: "parse_tsx_root"
            kind: "function"
            public: false
          - name: "function_name"
            kind: "function"
            public: false
          - name: "parse_default_import_name"
            kind: "function"
            public: false
          - name: "parse_named_import_names"
            kind: "function"
            public: false
          - name: "render_js_index_html"
            kind: "function"
            public: true
          - name: "render_wasm_index_html"
            kind: "function"
            public: true
          - name: "ensure_canvas_mount"
            kind: "function"
            public: false
          - name: "ensure_dom_mount"
            kind: "function"
            public: false
          - name: "replace_first_module_script_or_inject"
            kind: "function"
            public: false
          - name: "replace_module_entry_script"
            kind: "function"
            public: false
          - name: "module_src_matches_entry"
            kind: "function"
            public: false
          - name: "inject_stylesheet_links"
            kind: "function"
            public: false
          - name: "inject_before_head_end"
            kind: "function"
            public: false
          - name: "inject_before_body_end"
            kind: "function"
            public: false
          - name: "remove_title_tags"
            kind: "function"
            public: false
          - name: "is_style_specifier"
            kind: "function"
            public: false
          - name: "first_child_of_kind"
            kind: "function"
            public: false
          - name: "node_text"
            kind: "function"
            public: false
          - name: "strip_quotes"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/rerun_manifest.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "RERUN_MANIFEST_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "RerunManifest"
            kind: "struct"
            public: true
          - name: "RerunEntry"
            kind: "struct"
            public: true
          - name: "MISSING_FAILURE_METADATA_SENTINEL"
            kind: "constant"
            public: true
          - name: "format_rerun_manifest_missing_failure_warn"
            kind: "function"
            public: true
          - name: "rerun_entry_from_case_or_warn"
            kind: "function"
            public: true
          - name: "from_envelope"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "write_manifest"
            kind: "function"
            public: true
          - name: "is_failure_outcome"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/ci_summary.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "render_summary"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/build_target.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "BuildTarget"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "target_feature"
            kind: "function"
            public: true
          - name: "produces_wasm"
            kind: "function"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "BuildTargetError"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "parse"
            kind: "function"
            public: true
          - name: "resolve"
            kind: "function"
            public: true
          - name: "FlagSnapshot"
            kind: "struct"
            public: true
          - name: "validate_combination"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "agent"
            kind: "module"
            public: true
          - name: "asset"
            kind: "module"
            public: true
          - name: "browser"
            kind: "module"
            public: true
          - name: "browser_cli"
            kind: "module"
            public: true
          - name: "build_clean"
            kind: "module"
            public: true
          - name: "build_target"
            kind: "module"
            public: true
          - name: "bundler"
            kind: "module"
            public: true
          - name: "cdp_driver"
            kind: "module"
            public: true
          - name: "ci_summary"
            kind: "module"
            public: true
          - name: "cli"
            kind: "module"
            public: true
          - name: "codegen"
            kind: "module"
            public: true
          - name: "css"
            kind: "module"
            public: true
          - name: "dev_server"
            kind: "module"
            public: true
          - name: "e2e"
            kind: "module"
            public: true
          - name: "evidence"
            kind: "module"
            public: true
          - name: "frontend"
            kind: "module"
            public: true
          - name: "pkg_manager"
            kind: "module"
            public: true
          - name: "pm_report"
            kind: "module"
            public: true
          - name: "report_package"
            kind: "module"
            public: true
          - name: "reporter"
            kind: "module"
            public: true
          - name: "rerun_manifest"
            kind: "module"
            public: true
          - name: "resolver"
            kind: "module"
            public: true
          - name: "result_envelope"
            kind: "module"
            public: true
          - name: "runner"
            kind: "module"
            public: true
          - name: "standard_cli"
            kind: "module"
            public: true
          - name: "stories"
            kind: "module"
            public: true
          - name: "task_runner"
            kind: "module"
            public: true
          - name: "test_runner"
            kind: "module"
            public: true
          - name: "trace"
            kind: "module"
            public: true
          - name: "transform"
            kind: "module"
            public: true
          - name: "tsx_to_rust"
            kind: "module"
            public: true
          - name: "wasm_build"
            kind: "module"
            public: true
          - name: "wasm_dev"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/standard_cli.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "TOOL"
            kind: "constant"
            public: false
          - name: "TOPICS"
            kind: "constant"
            public: false
          - name: "llm_command"
            kind: "function"
            public: true
          - name: "upgrade_command"
            kind: "function"
            public: true
          - name: "issue_command"
            kind: "function"
            public: true
          - name: "run_llm"
            kind: "function"
            public: true
          - name: "run_upgrade"
            kind: "function"
            public: true
          - name: "run_issue"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/build_clean.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "CleanRejection"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "assess_clean"
            kind: "function"
            public: true
          - name: "empty_out_dir"
            kind: "function"
            public: true
          - name: "empty_out_dir_with_cwd"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/main.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/report_package.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "StaticReportPackage"
            kind: "struct"
            public: true
          - name: "CopiedArtifact"
            kind: "struct"
            public: true
          - name: "package_static_report"
            kind: "function"
            public: true
          - name: "rewrite_artifacts"
            kind: "function"
            public: false
          - name: "resolve_source_path"
            kind: "function"
            public: false
          - name: "format_artifact_basename_fallback_warn"
            kind: "function"
            public: true
          - name: "pick_unique_basename"
            kind: "function"
            public: false
          - name: "package_from_file"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3741_artifact_basename_fallback_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
      - path: "projects/jet/src/result_envelope.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method", "test_case"]
        symbols:
          - name: "SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ResultMode"
            kind: "enum"
            public: true
          - name: "ResultSummary"
            kind: "struct"
            public: true
          - name: "ResultSourceLocation"
            kind: "struct"
            public: true
          - name: "ResultFailure"
            kind: "struct"
            public: true
          - name: "ResultArtifact"
            kind: "struct"
            public: true
          - name: "FailureArtifactRef"
            kind: "struct"
            public: true
          - name: "resolve_all"
            kind: "function"
            public: true
          - name: "is_missing"
            kind: "function"
            public: true
          - name: "golden_triplet"
            kind: "function"
            public: true
          - name: "ResultRetryAttempt"
            kind: "struct"
            public: true
          - name: "ResultCase"
            kind: "struct"
            public: true
          - name: "WorkerScheduling"
            kind: "struct"
            public: true
          - name: "from_config"
            kind: "function"
            public: true
          - name: "ResultEnvelope"
            kind: "struct"
            public: true
          - name: "from_test_summary"
            kind: "function"
            public: true
          - name: "from_test_summary_with_config"
            kind: "function"
            public: true
          - name: "with_worker_scheduling"
            kind: "function"
            public: true
          - name: "slowest_cases"
            kind: "function"
            public: true
          - name: "from_e2e_bundle"
            kind: "function"
            public: true
          - name: "e2e_case_rerun_hint"
            kind: "function"
            public: true
          - name: "outcome_str"
            kind: "function"
            public: false
          - name: "test_report_to_case"
            kind: "function"
            public: false
          - name: "EmitterOptions"
            kind: "struct"
            public: true
          - name: "EmittedArtifacts"
            kind: "struct"
            public: true
          - name: "emit_artifacts"
            kind: "function"
            public: true
          - name: "EMPTY_FAILURE_MESSAGE_SENTINEL"
            kind: "constant"
            public: true
          - name: "format_result_envelope_empty_failure_message_warn"
            kind: "function"
            public: true
          - name: "first_failure_message_line_or_warn"
            kind: "function"
            public: true
          - name: "format_failures_text"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/src"
      - path: "projects/jet/src/cli.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "command"
            kind: "function"
            public: true
          - name: "serve_command"
            kind: "function"
            public: false
          - name: "browser_bridge_command"
            kind: "function"
            public: false
          - name: "execute"
            kind: "function"
            public: true
          - name: "execute_async"
            kind: "function"
            public: false
          - name: "run_nx_build"
            kind: "function"
            public: false
          - name: "run_library_build"
            kind: "function"
            public: false
          - name: "resolve_store_path_from_home"
            kind: "function"
            public: true
          - name: "format_store_home_err"
            kind: "function"
            public: true
          - name: "format_unknown_store_subcommand_err"
            kind: "function"
            public: true
          - name: "absolutize_report_index"
            kind: "function"
            public: true
          - name: "format_report_view_abs_err"
            kind: "function"
            public: true
          - name: "format_cli_flag_parse_warn"
            kind: "function"
            public: true
          - name: "parse_cli_numeric_flag"
            kind: "function"
            public: true
          - name: "merge_dev_proxy_rules"
            kind: "function"
            public: false
          - name: "parse_dev_proxy_rule"
            kind: "function"
            public: false
          - name: "handle_serve_command"
            kind: "function"
            public: false
          - name: "launch_detached_serve"
            kind: "function"
            public: false
          - name: "shutdown_dev_server"
            kind: "function"
            public: false
          - name: "print_shutdown_result"
            kind: "function"
            public: false
          - name: "read_project_type_is_lib"
            kind: "function"
            public: false
          - name: "package_script_value_kind"
            kind: "function"
            public: true
          - name: "format_cli_run_non_string_script_value_warn"
            kind: "function"
            public: true
          - name: "format_non_string_script_value_sentinel"
            kind: "function"
            public: true
          - name: "coerce_script_command_or_warn"
            kind: "function"
            public: true
          - name: "format_cli_build_sourcemap_unknown_warn"
            kind: "function"
            public: true
          - name: "coerce_sourcemap_mode_or_warn"
            kind: "function"
            public: true
          - name: "format_cli_build_sourcemap_non_utf8_entry_warn"
            kind: "function"
            public: true
          - name: "coerce_sourcemap_entry_path_or_warn"
            kind: "function"
            public: true
          - name: "list_scripts"
            kind: "function"
            public: false
          - name: "handle_run"
            kind: "function"
            public: false
          - name: "find_entry_point"
            kind: "function"
            public: false
          - name: "content_hash_prefix"
            kind: "function"
            public: false
          - name: "write_bundle_assets"
            kind: "function"
            public: false
          - name: "append_css_side_effect_assets"
            kind: "function"
            public: false
          - name: "resolve_css_side_effect_import_path"
            kind: "function"
            public: false
          - name: "copy_public_dir"
            kind: "function"
            public: false
          - name: "copy_public_dir_contents"
            kind: "function"
            public: false
          - name: "emit_build_index_html"
            kind: "function"
            public: false
          - name: "render_build_index_html"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/frontend.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/rerun_manifest.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/ci_summary.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/build_target.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/standard_cli.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-src-standard-cli-rs>"
  - path: "projects/jet/src/build_clean.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/main.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/report_package.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/result_envelope.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/cli.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
