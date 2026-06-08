---
id: semantic-agentic-workflow-generate-diagrams-erd-plus
summary: Semantic coverage for "projects/agentic-workflow/src/generate/diagrams/erd_plus"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/generate/diagrams/erd_plus

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/generate/diagrams/erd_plus"
  source_group: "projects/agentic-workflow/src/generate/diagrams/erd_plus"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ERDSeverity"
            kind: "enum"
            public: true
          - name: "ERDValidationError"
            kind: "struct"
            public: true
          - name: "ERDValidationResult"
            kind: "struct"
            public: true
          - name: "ok"
            kind: "function"
            public: true
          - name: "with_error"
            kind: "function"
            public: true
          - name: "with_warning"
            kind: "function"
            public: true
          - name: "ERDValidator"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "strict"
            kind: "function"
            public: true
          - name: "validate"
            kind: "function"
            public: true
          - name: "default"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/erd_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/erd_plus/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "generator"
            kind: "module"
            public: false
          - name: "schema"
            kind: "module"
            public: false
          - name: "validator"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/diagrams/erd_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/erd_plus/schema.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Cardinality"
            kind: "enum"
            public: true
          - name: "ERDAttributeDef"
            kind: "struct"
            public: true
          - name: "ERDDef"
            kind: "struct"
            public: true
          - name: "ERDRelationshipDef"
            kind: "struct"
            public: true
          - name: "EntityDef"
            kind: "struct"
            public: true
          - name: "KeyType"
            kind: "enum"
            public: true
          - name: "is_false"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/erd_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/erd_plus/generator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ERDPlusOutput"
            kind: "struct"
            public: true
          - name: "ERDPlusGenerator"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "generate"
            kind: "function"
            public: true
          - name: "generate_frontmatter"
            kind: "function"
            public: false
          - name: "reorder_entities"
            kind: "function"
            public: false
          - name: "topo_sort_entity_keys"
            kind: "function"
            public: false
          - name: "generate_mermaid"
            kind: "function"
            public: true
          - name: "cardinality_to_symbols"
            kind: "function"
            public: false
          - name: "default"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/erd_plus"
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
  - path: "projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/erd_plus/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/erd_plus/schema.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/erd_plus/generator.rs"
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
