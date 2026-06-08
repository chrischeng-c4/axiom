---
id: semantic-agentic-workflow-generate-diagrams-flowchart-plus
summary: Semantic coverage for "projects/agentic-workflow/src/generate/diagrams/flowchart_plus"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/generate/diagrams/flowchart_plus

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/generate/diagrams/flowchart_plus"
  source_group: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "FlowchartSeverity"
            kind: "enum"
            public: true
          - name: "FlowchartValidationResult"
            kind: "struct"
            public: true
          - name: "FlowchartValidationError"
            kind: "struct"
            public: true
          - name: "FlowchartValidator"
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
          - name: "new"
            kind: "function"
            public: true
          - name: "strict"
            kind: "function"
            public: true
          - name: "validate"
            kind: "function"
            public: true
          - name: "validate_semantics"
            kind: "function"
            public: false
          - name: "check_unreachable_nodes"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus/mod.rs"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus/schema.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "DbOperation"
            kind: "enum"
            public: true
          - name: "EdgeDef"
            kind: "struct"
            public: true
          - name: "EdgeStyle"
            kind: "enum"
            public: true
          - name: "FlowDirection"
            kind: "enum"
            public: true
          - name: "FlowchartDef"
            kind: "struct"
            public: true
          - name: "HttpMethod"
            kind: "enum"
            public: true
          - name: "NodeDef"
            kind: "struct"
            public: true
          - name: "NodeShape"
            kind: "enum"
            public: true
          - name: "PrimitiveKind"
            kind: "enum"
            public: true
          - name: "SemanticType"
            kind: "enum"
            public: true
          - name: "SubgraphDef"
            kind: "struct"
            public: true
          - name: "is_default_direction"
            kind: "function"
            public: false
          - name: "is_default_shape"
            kind: "function"
            public: false
          - name: "is_default_edge_style"
            kind: "function"
            public: false
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
          domain: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus/generator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "FlowchartPlusOutput"
            kind: "struct"
            public: true
          - name: "FlowchartPlusGenerator"
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
          - name: "reorder_nodes"
            kind: "function"
            public: false
          - name: "topo_sort_node_keys"
            kind: "function"
            public: false
          - name: "generate_mermaid"
            kind: "function"
            public: true
          - name: "format_node"
            kind: "function"
            public: false
          - name: "format_edge"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus"
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
  - path: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus/schema.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/flowchart_plus/generator.rs"
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
