---
id: semantic-agentic-workflow-generate-patterns
summary: Semantic coverage for "projects/agentic-workflow/src/generate/patterns"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/generate/patterns

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/generate/patterns"
  source_group: "projects/agentic-workflow/src/generate/patterns"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/generate/patterns/registry.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "pattern_registry"
            kind: "function"
            public: true
          - name: "PATTERN_REGISTRY"
            kind: "constant"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/patterns"
      - path: "projects/agentic-workflow/src/generate/patterns/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "registry"
            kind: "module"
            public: true
          - name: "resolver"
            kind: "module"
            public: true
          - name: "PatternNode"
            kind: "struct"
            public: true
          - name: "PatternSlot"
            kind: "struct"
            public: true
          - name: "SlotContent"
            kind: "struct"
            public: true
          - name: "UxPattern"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/patterns"
      - path: "projects/agentic-workflow/src/generate/patterns/resolver.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "resolve_pattern"
            kind: "function"
            public: true
          - name: "expand_pattern"
            kind: "function"
            public: true
          - name: "expand_node"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/patterns"
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
  - path: "projects/agentic-workflow/src/generate/patterns/registry.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/patterns/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/patterns/resolver.rs"
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
