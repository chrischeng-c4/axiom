---
id: semantic-lumen-projects-lumen
summary: Semantic coverage for "projects/lumen"
capability_refs:
  - id: "search"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen`."
fill_sections: [schema, unit-test, changes]
---

# Semantic TD: lumen/projects/lumen

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "lumen/projects/lumen"
  source_group: "projects/lumen"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/build.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen"
      - path: "projects/lumen/install.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen"
      - path: "projects/lumen/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen"
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
  - path: "projects/lumen/build.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:#4158>"
  - path: "projects/lumen/install.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:#4158>"
  - path: "projects/lumen/llms.txt"
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
