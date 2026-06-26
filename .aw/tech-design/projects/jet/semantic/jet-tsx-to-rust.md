---
id: semantic-jet-tsx-to-rust
summary: Semantic coverage for "projects/jet/src/tsx_to_rust"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tsx_to_rust

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/tsx_to_rust"
  source_group: "projects/jet/src/tsx_to_rust"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/tsx_to_rust/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "emit"
            kind: "module"
            public: true
          - name: "transpile"
            kind: "function"
            public: true
          - name: "TranspileResult"
            kind: "struct"
            public: true
          - name: "ImportAction"
            kind: "enum"
            public: true
          - name: "transpile_with_source"
            kind: "function"
            public: true
          - name: "transpile_compat_with_source"
            kind: "function"
            public: true
          - name: "extract_function_source"
            kind: "function"
            public: false
          - name: "handle_import_statement"
            kind: "function"
            public: false
          - name: "find_function_declaration"
            kind: "function"
            public: false
          - name: "collect_compat_import_aliases"
            kind: "function"
            public: false
          - name: "imported_local_names"
            kind: "function"
            public: false
          - name: "mui_component_lowering"
            kind: "function"
            public: false
          - name: "is_style_side_effect_import"
            kind: "function"
            public: false
          - name: "has_only_inline_type_specifiers"
            kind: "function"
            public: false
          - name: "strip_quotes"
            kind: "function"
            public: false
          - name: "node_text"
            kind: "function"
            public: true
          - name: "first_child_of_kind"
            kind: "function"
            public: true
          - name: "format_pos"
            kind: "function"
            public: true
          - name: "reject"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/tsx_to_rust"
      - path: "projects/jet/src/tsx_to_rust/emit.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "Emitter"
            kind: "struct"
            public: true
          - name: "PositionMap"
            kind: "struct"
            public: true
          - name: "ComponentPos"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "push_line"
            kind: "function"
            public: true
          - name: "push_block"
            kind: "function"
            public: true
          - name: "push_tsx_annot"
            kind: "function"
            public: true
          - name: "indent"
            kind: "function"
            public: true
          - name: "dedent"
            kind: "function"
            public: true
          - name: "finish"
            kind: "function"
            public: true
          - name: "take_position_map"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "prelude"
            kind: "function"
            public: true
          - name: "props_interface"
            kind: "function"
            public: true
          - name: "ts_type_to_rust"
            kind: "function"
            public: false
          - name: "function_component"
            kind: "function"
            public: true
          - name: "compat_function_component"
            kind: "function"
            public: true
          - name: "compat_artifact_studio_component"
            kind: "function"
            public: true
          - name: "find_top_level_return_expr"
            kind: "function"
            public: false
          - name: "parse_destructured_props"
            kind: "function"
            public: false
          - name: "emit_hook_binding"
            kind: "function"
            public: false
          - name: "emit_effect_call"
            kind: "function"
            public: false
          - name: "emit_effect_body"
            kind: "function"
            public: false
          - name: "emit_effect_call_expression"
            kind: "function"
            public: false
          - name: "top_level_const"
            kind: "function"
            public: true
          - name: "try_lower_in_component_copy_const"
            kind: "function"
            public: false
          - name: "string_literal_text"
            kind: "function"
            public: false
          - name: "parse_string_object_literal"
            kind: "function"
            public: false
          - name: "emit_use_state_binding"
            kind: "function"
            public: false
          - name: "emit_use_memo_binding"
            kind: "function"
            public: false
          - name: "is_copy_primitive"
            kind: "function"
            public: false
          - name: "transpile_expr"
            kind: "function"
            public: false
          - name: "emit_jsx_expr"
            kind: "function"
            public: false
          - name: "emit_jsx_element"
            kind: "function"
            public: false
          - name: "emit_jsx_self_closing"
            kind: "function"
            public: false
          - name: "render_props_literal"
            kind: "function"
            public: false
          - name: "compat_jsx_expr"
            kind: "function"
            public: false
          - name: "compat_jsx_element"
            kind: "function"
            public: false
          - name: "compat_jsx_self_closing"
            kind: "function"
            public: false
          - name: "compat_jsx_fragment"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/tsx_to_rust"
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
  - path: "projects/jet/src/tsx_to_rust/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/tsx_to_rust/emit.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
