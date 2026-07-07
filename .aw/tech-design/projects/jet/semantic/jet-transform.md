---
id: semantic-jet-transform
summary: Semantic coverage for "projects/jet/src/transform"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/transform

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/transform"
  source_group: "projects/jet/src/transform"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/transform/jsx.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "transform_jsx"
            kind: "function"
            public: true
          - name: "transform_node"
            kind: "function"
            public: false
          - name: "transform_jsx_element"
            kind: "function"
            public: false
          - name: "transform_jsx_fragment"
            kind: "function"
            public: false
          - name: "extract_tag_name"
            kind: "function"
            public: false
          - name: "extract_tag_from_opening"
            kind: "function"
            public: false
          - name: "extract_props"
            kind: "function"
            public: false
          - name: "extract_props_from_opening"
            kind: "function"
            public: false
          - name: "extract_jsx_attribute"
            kind: "function"
            public: false
          - name: "extract_jsx_expression_value"
            kind: "function"
            public: false
          - name: "extract_children"
            kind: "function"
            public: false
          - name: "transform_to_create_element"
            kind: "function"
            public: false
          - name: "transform_to_jsx_runtime"
            kind: "function"
            public: false
          - name: "generate_source_map"
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
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/typescript.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "transform_typescript"
            kind: "function"
            public: true
          - name: "remove_types"
            kind: "function"
            public: false
          - name: "visit_node"
            kind: "function"
            public: false
          - name: "child_has_only_type"
            kind: "function"
            public: false
          - name: "compile_enum"
            kind: "function"
            public: true
          - name: "emit_satisfies_expression"
            kind: "function"
            public: false
          - name: "emit_as_expression"
            kind: "function"
            public: false
          - name: "generate_source_map"
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
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/type_strip.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "is_type_only_export"
            kind: "function"
            public: true
          - name: "is_type_only_import"
            kind: "function"
            public: true
          - name: "has_inline_type_specifiers"
            kind: "function"
            public: true
          - name: "transform_import_with_inline_types"
            kind: "function"
            public: true
          - name: "transform_satisfies_expression"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "css"
            kind: "module"
            public: true
          - name: "incremental"
            kind: "module"
            public: true
          - name: "jsx"
            kind: "module"
            public: true
          - name: "modules"
            kind: "module"
            public: true
          - name: "react_refresh"
            kind: "module"
            public: true
          - name: "transform_tsx"
            kind: "module"
            public: true
          - name: "type_strip"
            kind: "module"
            public: true
          - name: "typescript"
            kind: "module"
            public: true
          - name: "Transformer"
            kind: "struct"
            public: true
          - name: "TransformOptions"
            kind: "struct"
            public: true
          - name: "TypeScriptTarget"
            kind: "enum"
            public: true
          - name: "TransformResult"
            kind: "struct"
            public: true
          - name: "TRANSFORM_JS_NO_EXTENSION_FALLBACK"
            kind: "constant"
            public: true
          - name: "format_transform_js_no_extension_warn"
            kind: "function"
            public: true
          - name: "format_transform_js_non_utf8_extension_warn"
            kind: "function"
            public: true
          - name: "coerce_transform_js_extension_or_warn"
            kind: "function"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "transform_js"
            kind: "function"
            public: true
          - name: "transform_js_with_context"
            kind: "function"
            public: true
          - name: "transform_css"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3809_transform_js_extension_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/incremental.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "CacheKey"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "CachedEntry"
            kind: "struct"
            public: true
          - name: "RebuildMetrics"
            kind: "struct"
            public: true
          - name: "log_line"
            kind: "function"
            public: true
          - name: "json_string"
            kind: "function"
            public: false
          - name: "Ext"
            kind: "enum"
            public: false
          - name: "from_path"
            kind: "function"
            public: false
          - name: "IncrementalTransformer"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "transform_incremental"
            kind: "function"
            public: true
          - name: "transform_for_path"
            kind: "function"
            public: true
          - name: "invalidate"
            kind: "function"
            public: true
          - name: "metrics_snapshot"
            kind: "function"
            public: true
          - name: "find_prior_tree_for"
            kind: "function"
            public: false
          - name: "transform_tree"
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
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/modules.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "ModuleMapping"
            kind: "enum"
            public: true
          - name: "transform_modules"
            kind: "function"
            public: true
          - name: "transform_modules_with_dir"
            kind: "function"
            public: true
          - name: "transform_node"
            kind: "function"
            public: false
          - name: "transform_import"
            kind: "function"
            public: false
          - name: "transform_export"
            kind: "function"
            public: false
          - name: "extract_export_source"
            kind: "function"
            public: false
          - name: "is_dynamic_import"
            kind: "function"
            public: false
          - name: "transform_dynamic_import"
            kind: "function"
            public: false
          - name: "resolve_module_path"
            kind: "function"
            public: false
          - name: "is_require_call"
            kind: "function"
            public: false
          - name: "transform_require_call"
            kind: "function"
            public: false
          - name: "extract_export_value"
            kind: "function"
            public: false
          - name: "extract_declaration_names"
            kind: "function"
            public: false
          - name: "parse_export_clause"
            kind: "function"
            public: false
          - name: "parse_export_specifier"
            kind: "function"
            public: false
          - name: "ImportSpec"
            kind: "enum"
            public: false
          - name: "parse_import_clause"
            kind: "function"
            public: false
          - name: "parse_namespace_import"
            kind: "function"
            public: false
          - name: "parse_named_imports"
            kind: "function"
            public: false
          - name: "parse_import_specifier"
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
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/transform_tsx_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "test_tsx_simple_component"
            kind: "function"
            public: false
          - name: "test_tsx_nested_jsx"
            kind: "function"
            public: false
          - name: "test_tsx_function_call_with_jsx"
            kind: "function"
            public: false
          - name: "test_tsx_with_type_annotations"
            kind: "function"
            public: false
          - name: "test_tsx_self_closing_with_props"
            kind: "function"
            public: false
          - name: "t17_strip_export_type_statement"
            kind: "function"
            public: false
          - name: "t18_strip_import_type_statement"
            kind: "function"
            public: false
          - name: "t19_strip_inline_type_import_specifier"
            kind: "function"
            public: false
          - name: "t20_remove_empty_type_only_import"
            kind: "function"
            public: false
          - name: "t20b_strip_arrow_function_type_predicate"
            kind: "function"
            public: false
          - name: "t21_strip_multiline_interface_short"
            kind: "function"
            public: false
          - name: "t22_strip_multiline_interface_long"
            kind: "function"
            public: false
          - name: "t23_strip_standalone_interface"
            kind: "function"
            public: false
          - name: "t24_strip_declare_function"
            kind: "function"
            public: false
          - name: "t25_strip_declare_module"
            kind: "function"
            public: false
          - name: "t26_strip_declare_const"
            kind: "function"
            public: false
          - name: "t27_strip_declare_global"
            kind: "function"
            public: false
          - name: "t28_strip_satisfies_expression"
            kind: "function"
            public: false
          - name: "t28b_strip_satisfies_multiline_generic"
            kind: "function"
            public: false
          - name: "t29_preserve_type_as_identifier"
            kind: "function"
            public: false
          - name: "t30_strip_as_const_expression"
            kind: "function"
            public: false
          - name: "t31_strip_type_alias"
            kind: "function"
            public: false
          - name: "t32_mixed_import_value_and_type"
            kind: "function"
            public: false
          - name: "t1_import_meta_hot_injected_in_dev_mode"
            kind: "function"
            public: false
          - name: "t2_import_meta_hot_not_injected_in_prod"
            kind: "function"
            public: false
          - name: "t10_component_declaration_gets_refresh_reg"
            kind: "function"
            public: false
          - name: "t11_arrow_component_gets_refresh_reg"
            kind: "function"
            public: false
          - name: "t12_hook_usage_gets_refresh_sig"
            kind: "function"
            public: false
          - name: "t13_non_component_function_skipped"
            kind: "function"
            public: false
          - name: "t14_react_memo_wrapped_component"
            kind: "function"
            public: false
          - name: "t15_preamble_and_footer_injected"
            kind: "function"
            public: false
          - name: "no_fast_refresh_for_non_jsx_file"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/css.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "transform_css"
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
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/react_refresh.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "inject_react_fast_refresh"
            kind: "function"
            public: true
          - name: "ReactComponent"
            kind: "struct"
            public: false
          - name: "detect_react_components"
            kind: "function"
            public: false
          - name: "collect_components"
            kind: "function"
            public: false
          - name: "check_function_component"
            kind: "function"
            public: false
          - name: "check_variable_component"
            kind: "function"
            public: false
          - name: "get_function_name"
            kind: "function"
            public: false
          - name: "get_declarator_name"
            kind: "function"
            public: false
          - name: "function_body_has_jsx"
            kind: "function"
            public: false
          - name: "subtree_has_jsx"
            kind: "function"
            public: false
          - name: "detect_hooks_usage"
            kind: "function"
            public: false
          - name: "collect_hook_calls"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/transform"
      - path: "projects/jet/src/transform/transform_tsx.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "normalize_jsx_text"
            kind: "function"
            public: true
          - name: "transform_tsx"
            kind: "function"
            public: true
          - name: "has_jsx"
            kind: "function"
            public: false
          - name: "transform_node"
            kind: "function"
            public: true
          - name: "should_skip_node"
            kind: "function"
            public: false
          - name: "is_as_expression"
            kind: "function"
            public: false
          - name: "transform_as_expression"
            kind: "function"
            public: false
          - name: "transform_jsx_element"
            kind: "function"
            public: false
          - name: "transform_jsx_fragment"
            kind: "function"
            public: false
          - name: "extract_tag_name"
            kind: "function"
            public: false
          - name: "extract_tag_from_opening"
            kind: "function"
            public: false
          - name: "extract_props"
            kind: "function"
            public: false
          - name: "extract_props_from_opening"
            kind: "function"
            public: false
          - name: "extract_single_prop"
            kind: "function"
            public: false
          - name: "is_spread_expression"
            kind: "function"
            public: false
          - name: "extract_spread_expression"
            kind: "function"
            public: false
          - name: "extract_jsx_expression"
            kind: "function"
            public: false
          - name: "extract_children"
            kind: "function"
            public: false
          - name: "transform_to_jsx_runtime"
            kind: "function"
            public: false
          - name: "transform_to_create_element"
            kind: "function"
            public: false
          - name: "extract_jsx_pragma"
            kind: "function"
            public: false
          - name: "extract_jsx_frag_pragma"
            kind: "function"
            public: false
          - name: "generate_source_map"
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
          domain: "projects/jet/src/transform"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/transform/jsx.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/typescript.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/type_strip.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/incremental.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/modules.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/transform_tsx_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/css.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/react_refresh.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/transform/transform_tsx.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-transform.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
