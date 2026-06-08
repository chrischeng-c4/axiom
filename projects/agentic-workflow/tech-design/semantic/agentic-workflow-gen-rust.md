---
id: semantic-agentic-workflow-gen-rust
summary: Semantic coverage for "projects/agentic-workflow/src/gen/rust"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/gen/rust

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/gen/rust"
  source_group: "projects/agentic-workflow/src/gen/rust"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/gen/rust/rpc_api.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "RpcApiGenOutput"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/gen/rust"
      - path: "projects/agentic-workflow/src/gen/rust/db_model.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "DbModelGenOutput"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/gen/rust"
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
  - path: "projects/agentic-workflow/src/gen/rust/rpc_api.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/gen/rust/db_model.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
