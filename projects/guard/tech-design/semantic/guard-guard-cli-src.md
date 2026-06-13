---
id: semantic-guard-guard-cli-src
summary: Semantic coverage for "projects/guard/guard-cli/src"
fill_sections: [schema, unit-test, changes]
---

# Semantic TD: guard/guard-cli/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "guard/guard-cli/src"
  source_group: "projects/guard/guard-cli/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/guard/guard-cli/src/dispatch.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "GuardCommand"
            kind: "struct"
            public: true
          - name: "OutputOpts"
            kind: "struct"
            public: true
          - name: "Verb"
            kind: "enum"
            public: true
          - name: "ScanArgs"
            kind: "struct"
            public: true
          - name: "dispatch"
            kind: "function"
            public: true
          - name: "print_report"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/guard/guard-cli/src"
      - path: "projects/guard/guard-cli/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "dispatch"
            kind: "module"
            public: true
          - name: "GuardCli"
            kind: "struct"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "command"
            kind: "function"
            public: false
          - name: "execute"
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
          domain: "projects/guard/guard-cli/src"
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
  - path: "projects/guard/guard-cli/src/dispatch.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/guard/guard-cli/src/lib.rs"
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
