---
id: semantic-agentic-workflow-ui
summary: Semantic coverage for "projects/agentic-workflow/src/ui"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "This semantic TD covers AW core/client model source behavior and shared workflow domain primitives."
---

# Semantic TD: agentic-workflow/ui

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/ui"
  source_group: "projects/agentic-workflow/src/ui"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/ui/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "colors"
            kind: "module"
            public: true
          - name: "progress"
            kind: "module"
            public: true
          - name: "tables"
            kind: "module"
            public: true
          - name: "viewer"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/ui"
      - path: "projects/agentic-workflow/src/ui/colors.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "ColorScheme"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/ui"
      - path: "projects/agentic-workflow/src/ui/progress.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "ProgressBar"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/ui"
      - path: "projects/agentic-workflow/src/ui/tables.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "Table"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/ui"
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
  - path: "projects/agentic-workflow/src/ui/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/colors.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/progress.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/tables.rs"
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
