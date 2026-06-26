---
id: semantic-jet-wasm-src
summary: Semantic coverage for "projects/jet/wasm/src"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm/src"
  source_group: "projects/jet/wasm/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/wasm/src/host.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "fetch"
            kind: "function"
            public: true
          - name: "fetch_with_init"
            kind: "function"
            public: true
          - name: "fetch_json"
            kind: "function"
            public: true
          - name: "fetch_json_with_init"
            kind: "function"
            public: true
          - name: "fetch_text"
            kind: "function"
            public: true
          - name: "fetch_text_with_init"
            kind: "function"
            public: true
          - name: "fetch_response"
            kind: "function"
            public: false
          - name: "console_log"
            kind: "function"
            public: true
          - name: "console_warn"
            kind: "function"
            public: true
          - name: "console_error"
            kind: "function"
            public: true
          - name: "console_info"
            kind: "function"
            public: true
          - name: "local_storage_get_item"
            kind: "function"
            public: true
          - name: "local_storage_set_item"
            kind: "function"
            public: true
          - name: "local_storage_remove_item"
            kind: "function"
            public: true
          - name: "local_storage_clear"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src"
      - path: "projects/jet/wasm/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "renderer"
            kind: "module"
            public: true
          - name: "manifest"
            kind: "module"
            public: true
          - name: "host"
            kind: "module"
            public: true
          - name: "text"
            kind: "module"
            public: true
          - name: "react"
            kind: "module"
            public: true
          - name: "debug"
            kind: "module"
            public: true
          - name: "Element"
            kind: "enum"
            public: true
          - name: "intrinsic"
            kind: "function"
            public: true
          - name: "text"
            kind: "function"
            public: true
          - name: "from_number"
            kind: "function"
            public: true
          - name: "Component"
            kind: "struct"
            public: true
          - name: "ComponentFn"
            kind: "type"
            public: true
          - name: "Props"
            kind: "struct"
            public: true
          - name: "Callback"
            kind: "struct"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "call"
            kind: "function"
            public: true
          - name: "find_on_click"
            kind: "function"
            public: true
          - name: "text_content"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src"
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
  - path: "projects/jet/wasm/src/host.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-src.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
