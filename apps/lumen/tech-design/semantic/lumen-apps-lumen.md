---
id: semantic-lumen-projects-lumen
summary: Semantic coverage for "apps/lumen"
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `apps/lumen`."
fill_sections: [schema, unit-test, changes]
---

# Semantic TD: lumen/apps/lumen

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "lumen/apps/lumen"
  source_group: "apps/lumen"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/lumen/build.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
          - name: "short_sha"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/lumen"
      - path: "apps/lumen/build.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "apps/lumen"
      - path: "apps/lumen/install.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "apps/lumen"
      - path: "apps/lumen/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "source"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "apps/lumen"
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
  - path: "apps/lumen/build.rs"
    action: modify
    section: schema
    description: |
      Build-time provenance stamping for /version is covered by this root-level semantic TD.
    impl_mode: hand-written
  - path: "apps/lumen/build.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD;
      concrete replay ownership lives in semantic-lumen-build-script.
    impl_mode: hand-written
  - path: "apps/lumen/install.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:#4158>"
  - path: "apps/lumen/llms.txt"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."
```
