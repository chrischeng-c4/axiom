---
id: semantic-jet-css-plugins
summary: Semantic coverage for "projects/jet/src/css/plugins"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/css/plugins

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/css/plugins"
  source_group: "projects/jet/src/css/plugins"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/css/plugins/animate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "emit"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "AnimationDef"
            kind: "struct"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css/plugins"
      - path: "projects/jet/src/css/plugins/typography.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "emit"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "PROSE_BASE"
            kind: "constant"
            public: false
          - name: "PROSE_SM"
            kind: "constant"
            public: false
          - name: "PROSE_BASE_SIZE"
            kind: "constant"
            public: false
          - name: "PROSE_LG"
            kind: "constant"
            public: false
          - name: "PROSE_XL"
            kind: "constant"
            public: false
          - name: "PROSE_2XL"
            kind: "constant"
            public: false
          - name: "PROSE_INVERT"
            kind: "constant"
            public: false
          - name: "PROSE_DARK_INVERT"
            kind: "constant"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css/plugins"
      - path: "projects/jet/src/css/plugins/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "animate"
            kind: "module"
            public: true
          - name: "typography"
            kind: "module"
            public: true
          - name: "AnimateEmitter"
            kind: "struct"
            public: false
          - name: "TypographyEmitter"
            kind: "struct"
            public: false
          - name: "name"
            kind: "function"
            public: false
          - name: "emit"
            kind: "function"
            public: false
          - name: "name"
            kind: "function"
            public: false
          - name: "emit"
            kind: "function"
            public: false
          - name: "emit_plugins"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css/plugins"
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
  - path: "projects/jet/src/css/plugins/animate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/plugins/typography.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/plugins/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-css-plugins.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
