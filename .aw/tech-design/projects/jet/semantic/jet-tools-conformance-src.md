---
id: semantic-jet-tools-conformance-src
summary: Semantic coverage for "projects/jet/tools/conformance/src"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tools/conformance/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/tools/conformance/src"
  source_group: "projects/jet/tools/conformance/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tools/conformance/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "DEFAULT_MANIFEST"
            kind: "constant"
            public: false
          - name: "DEFAULT_SCHEMA"
            kind: "constant"
            public: false
          - name: "Manifest"
            kind: "struct"
            public: false
          - name: "Entry"
            kind: "struct"
            public: false
          - name: "Status"
            kind: "enum"
            public: false
          - name: "CheckConformanceManifestModule"
            kind: "struct"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "command"
            kind: "function"
            public: false
          - name: "execute"
            kind: "function"
            public: false
          - name: "run"
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
          domain: "projects/jet/tools/conformance/src"
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
  - path: "projects/jet/tools/conformance/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-tools-conformance-src.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
