---
id: semantic-agentic-workflow-generate-generators-tests
summary: Semantic coverage for "projects/agentic-workflow/src/generate/generators/tests"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: cb-and-cold-verification-gates
    claim: cb-and-cold-verification-gates
    coverage: full
    rationale: "This semantic TD covers verification source behavior used by existing-project standardization gates."
---

# Semantic TD: agentic-workflow/generate/generators/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/generate/generators/tests"
  source_group: "projects/agentic-workflow/src/generate/generators/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/generate/generators/tests/cli_subcommand_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/src/generate/generators/tests"
      - path: "projects/agentic-workflow/src/generate/generators/tests/module_facade_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/src/generate/generators/tests"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/agentic-workflow/src/generate/generators/tests/cli_subcommand_test.rs"
      - path: "projects/agentic-workflow/src/generate/generators/tests/module_facade_test.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/agentic-workflow/src/generate/generators/tests/cli_subcommand_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/generators/tests/module_facade_test.rs"
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
