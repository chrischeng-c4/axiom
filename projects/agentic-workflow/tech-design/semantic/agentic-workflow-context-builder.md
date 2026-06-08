---
id: semantic-agentic-workflow-context-builder
summary: Semantic coverage for "projects/agentic-workflow/src/context_builder"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/context_builder

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/context_builder"
  source_group: "projects/agentic-workflow/src/context_builder"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/context_builder/types.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ContextEntry"
            kind: "struct"
            public: true
          - name: "ContextReason"
            kind: "enum"
            public: true
          - name: "ContextRequest"
            kind: "struct"
            public: true
          - name: "ContextResponse"
            kind: "struct"
            public: true
          - name: "ContextStats"
            kind: "struct"
            public: true
          - name: "ContextTarget"
            kind: "struct"
            public: true
          - name: "parse"
            kind: "function"
            public: true
          - name: "default_depth"
            kind: "function"
            public: false
          - name: "empty"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/context_builder"
      - path: "projects/agentic-workflow/src/context_builder/traversal.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "forward_traverse"
            kind: "function"
            public: true
          - name: "backward_traverse"
            kind: "function"
            public: true
          - name: "depth_to_score"
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
          domain: "projects/agentic-workflow/src/context_builder"
      - path: "projects/agentic-workflow/src/context_builder/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "test_detection"
            kind: "module"
            public: true
          - name: "traversal"
            kind: "module"
            public: true
          - name: "types"
            kind: "module"
            public: true
          - name: "ContextBuilder"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "with_type_context"
            kind: "function"
            public: true
          - name: "with_project_files"
            kind: "function"
            public: true
          - name: "build_context"
            kind: "function"
            public: true
          - name: "resolve_symbol"
            kind: "function"
            public: false
          - name: "collect_type_signatures"
            kind: "function"
            public: false
          - name: "merge_entries"
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
          domain: "projects/agentic-workflow/src/context_builder"
      - path: "projects/agentic-workflow/src/context_builder/test_detection.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "enum_model", "service_method", "test_case"]
        symbols:
          - name: "TEST_FILE_SCORE"
            kind: "constant"
            public: false
          - name: "TestLanguage"
            kind: "enum"
            public: true
          - name: "from_path"
            kind: "function"
            public: true
          - name: "detect_test_files"
            kind: "function"
            public: true
          - name: "is_test_file_for"
            kind: "function"
            public: false
          - name: "is_python_test"
            kind: "function"
            public: false
          - name: "is_typescript_test"
            kind: "function"
            public: false
          - name: "is_rust_test"
            kind: "function"
            public: false
          - name: "is_go_test"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/agentic-workflow/src/context_builder"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/agentic-workflow/src/context_builder/test_detection.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/agentic-workflow/src/context_builder/types.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/context_builder/traversal.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/context_builder/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/context_builder/test_detection.rs"
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
