---
id: semantic-guard-guard-cli-src-bin
summary: Semantic coverage for "projects/guard/guard-cli/src/bin"
fill_sections: [schema, unit-test, changes]
---

# Semantic TD: guard/guard-cli/src/bin

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "guard/guard-cli/src/bin"
  source_group: "projects/guard/guard-cli/src/bin"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/guard/guard-cli/src/bin/guard.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/guard/guard-cli/src/bin"
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: unit-test
coverage_kind: semantic
strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
evidence:
  source_tests: []
---
requirementDiagram

element UT_SOURCE_TESTS {
  type: "TestEvidence"
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/guard/guard-cli/src/bin/guard.rs"
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
