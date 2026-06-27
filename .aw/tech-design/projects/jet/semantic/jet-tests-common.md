---
id: semantic-jet-tests-common
summary: Semantic coverage for "projects/jet/tests/common"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tests/common

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/tests/common"
  source_group: "projects/jet/tests/common"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tests/common/canvas_spy.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "init_script"
            kind: "function"
            public: true
          - name: "captured_calls_expr"
            kind: "function"
            public: true
          - name: "clear_expr"
            kind: "function"
            public: true
          - name: "canonical_canvas_methods"
            kind: "function"
            public: true
          - name: "expected_canvas_methods_from_paint_ops"
            kind: "function"
            public: true
          - name: "method_summary"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests/common"
      - path: "projects/jet/tests/common/react_oracle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "react_node_modules_root"
            kind: "function"
            public: false
          - name: "react_dom_umd_paths"
            kind: "function"
            public: true
          - name: "react_dom_available"
            kind: "function"
            public: true
          - name: "should_skip_react_oracle_env"
            kind: "function"
            public: true
          - name: "dom_serializer_expr"
            kind: "function"
            public: true
          - name: "counter_fixture_html"
            kind: "function"
            public: true
          - name: "file_url"
            kind: "function"
            public: false
          - name: "normalize_jet_element_tree"
            kind: "function"
            public: true
          - name: "normalize_jet_node"
            kind: "function"
            public: false
          - name: "normalize_children"
            kind: "function"
            public: false
          - name: "push_child"
            kind: "function"
            public: false
          - name: "normalize_jet_attrs"
            kind: "function"
            public: false
          - name: "normalize_text"
            kind: "function"
            public: false
          - name: "finalize_text_nodes"
            kind: "function"
            public: false
          - name: "diff_message"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests/common"
      - path: "projects/jet/tests/common/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method", "test_case"]
        symbols:
          - name: "canvas_spy"
            kind: "module"
            public: true
          - name: "react_oracle"
            kind: "module"
            public: true
          - name: "snapshot"
            kind: "module"
            public: true
          - name: "WASM_E2E_READY_ATTEMPTS"
            kind: "constant"
            public: true
          - name: "WASM_E2E_READY_INTERVAL"
            kind: "constant"
            public: true
          - name: "node_available"
            kind: "function"
            public: true
          - name: "python_available"
            kind: "function"
            public: true
          - name: "wasm_pack_available"
            kind: "function"
            public: true
          - name: "chromium_available"
            kind: "function"
            public: true
          - name: "skip_or_fail"
            kind: "function"
            public: true
          - name: "should_skip_env"
            kind: "function"
            public: true
          - name: "should_skip_wasm_pack_env"
            kind: "function"
            public: true
          - name: "should_skip_full_wasm_e2e_env"
            kind: "function"
            public: true
          - name: "wait_for_http_ready"
            kind: "function"
            public: true
          - name: "free_port"
            kind: "function"
            public: false
          - name: "JetTestApp"
            kind: "struct"
            public: true
          - name: "launch"
            kind: "function"
            public: true
          - name: "launch_with_init_scripts"
            kind: "function"
            public: true
          - name: "click_canvas"
            kind: "function"
            public: true
          - name: "shutdown"
            kind: "function"
            public: true
          - name: "element_tree"
            kind: "function"
            public: true
          - name: "layout_tree"
            kind: "function"
            public: true
          - name: "paint_ops"
            kind: "function"
            public: true
          - name: "pick_at"
            kind: "function"
            public: true
          - name: "fiber_tree"
            kind: "function"
            public: true
          - name: "hook_values"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests/common"
      - path: "projects/jet/tests/common/snapshot.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "snapshot_dir"
            kind: "function"
            public: false
          - name: "snapshot_eq_impl"
            kind: "function"
            public: true
          - name: "canonicalize"
            kind: "function"
            public: false
          - name: "sort_keys"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests/common"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/jet/tests/common/canvas_spy.rs"
      - path: "projects/jet/tests/common/react_oracle.rs"
      - path: "projects/jet/tests/common/mod.rs"
      - path: "projects/jet/tests/common/snapshot.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/tests/common/canvas_spy.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/common/react_oracle.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/common/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/common/snapshot.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-tests-common.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
