---
id: semantic-jet-wasm-src-manifest
summary: Semantic coverage for "projects/jet/wasm/src/manifest"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm/src/manifest

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm/src/manifest"
  source_group: "projects/jet/wasm/src/manifest"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/wasm/src/manifest/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "defaults"
            kind: "module"
            public: true
          - name: "parser"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/manifest"
      - path: "projects/jet/wasm/src/manifest/defaults.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "named"
            kind: "function"
            public: false
          - name: "default_export"
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
          domain: "projects/jet/wasm/src/manifest"
      - path: "projects/jet/wasm/src/manifest/parser.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ParsedManifest"
            kind: "struct"
            public: true
          - name: "ModuleEntry"
            kind: "struct"
            public: true
          - name: "ExportEntry"
            kind: "struct"
            public: true
          - name: "ExportKind"
            kind: "enum"
            public: true
          - name: "JetImpl"
            kind: "enum"
            public: true
          - name: "ManifestError"
            kind: "struct"
            public: true
          - name: "ManifestErrorCode"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "parse_manifest"
            kind: "function"
            public: true
          - name: "parse_manifest_text"
            kind: "function"
            public: true
          - name: "parse_module_name"
            kind: "function"
            public: false
          - name: "take_string_literal"
            kind: "function"
            public: false
          - name: "collect_module_body"
            kind: "function"
            public: false
          - name: "parse_module_body"
            kind: "function"
            public: false
          - name: "parse_jet_impl"
            kind: "function"
            public: false
          - name: "take_identifier"
            kind: "function"
            public: false
          - name: "strip_trailing_semicolon"
            kind: "function"
            public: false
          - name: "truncate"
            kind: "function"
            public: false
          - name: "ancestor_chain"
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
          domain: "projects/jet/wasm/src/manifest"
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
  - path: "projects/jet/wasm/src/manifest/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/manifest/defaults.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/manifest/parser.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
