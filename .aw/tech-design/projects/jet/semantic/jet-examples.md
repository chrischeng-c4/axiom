---
id: semantic-jet-examples
summary: Semantic coverage for "projects/jet/examples"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/examples

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/examples"
  source_group: "projects/jet/examples"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/examples/full_pipeline.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "find_example_root"
            kind: "function"
            public: false
          - name: "resolve_options"
            kind: "function"
            public: false
          - name: "transform_options"
            kind: "function"
            public: false
          - name: "demo_resolver"
            kind: "function"
            public: false
          - name: "demo_transformer"
            kind: "function"
            public: false
          - name: "demo_bundler"
            kind: "function"
            public: false
          - name: "demo_dev_server_config"
            kind: "function"
            public: false
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/examples"
      - path: "projects/jet/examples/dev_server.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "find_example_root"
            kind: "function"
            public: false
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/examples"
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
  - path: "projects/jet/examples/full_pipeline.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/examples/dev_server.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-examples.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
