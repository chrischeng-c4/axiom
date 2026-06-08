---
id: semantic-jet-wasm-dev
summary: Semantic coverage for "projects/jet/src/wasm_dev"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm_dev

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm_dev"
  source_group: "projects/jet/src/wasm_dev"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/wasm_dev/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "DevOptions"
            kind: "struct"
            public: true
          - name: "serve"
            kind: "function"
            public: true
          - name: "build_router"
            kind: "function"
            public: false
          - name: "handle_index"
            kind: "function"
            public: false
          - name: "handle_static"
            kind: "function"
            public: false
          - name: "should_spa_fallback"
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
          - name: "spawn_watcher"
            kind: "function"
            public: false
          - name: "format_wasm_watch_error"
            kind: "function"
            public: false
          - name: "should_trigger_rebuild"
            kind: "function"
            public: false
          - name: "debounce_and_rebuild"
            kind: "function"
            public: false
          - name: "shutdown_signal"
            kind: "function"
            public: false
          - name: "format_wasm_dev_ctrl_c_warn"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3730_wasm_dev_ctrl_c_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/wasm_dev"
```

## Tests
<!-- type: tests lang: yaml -->

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
  - path: "projects/jet/src/wasm_dev/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-dev.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
