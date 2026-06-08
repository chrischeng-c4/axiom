---
id: semantic-agentic-workflow-validator
summary: Semantic coverage for "projects/agentic-workflow/src/validator"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/validator

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/validator"
  source_group: "projects/agentic-workflow/src/validator"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/validator/format.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "SpecFormatValidator"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "validate_with_type"
            kind: "function"
            public: true
          - name: "validate"
            kind: "function"
            public: true
          - name: "validate_requirement_heading"
            kind: "function"
            public: false
          - name: "validate_scenario_heading"
            kind: "function"
            public: false
          - name: "validate_required_headings"
            kind: "function"
            public: false
          - name: "validate_requirement_completeness"
            kind: "function"
            public: false
          - name: "ValidationState"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
          - name: "enter_heading"
            kind: "function"
            public: false
          - name: "exit_heading"
            kind: "function"
            public: false
          - name: "add_heading"
            kind: "function"
            public: false
          - name: "find_line_number"
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
          domain: "projects/agentic-workflow/src/validator"
      - path: "projects/agentic-workflow/src/validator/consistency.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ConsistencyValidator"
            kind: "struct"
            public: true
          - name: "SpecRef"
            kind: "struct"
            public: false
          - name: "parse"
            kind: "function"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "validate_all"
            kind: "function"
            public: true
          - name: "validate_task_spec_refs"
            kind: "function"
            public: true
          - name: "anchor_exists"
            kind: "function"
            public: false
          - name: "validate_proposal_spec_alignment"
            kind: "function"
            public: true
          - name: "validate_task_dependencies"
            kind: "function"
            public: true
          - name: "detect_cycle"
            kind: "function"
            public: false
          - name: "validate_spec_hierarchy"
            kind: "function"
            public: true
          - name: "is_reference_error"
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
          domain: "projects/agentic-workflow/src/validator"
      - path: "projects/agentic-workflow/src/validator/semantic.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "SemanticValidator"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "validate"
            kind: "function"
            public: true
          - name: "check_spec_type_completeness"
            kind: "function"
            public: false
          - name: "extract_frontmatter_field"
            kind: "function"
            public: false
          - name: "contains_diagram_of_type"
            kind: "function"
            public: false
          - name: "contains_api_spec"
            kind: "function"
            public: false
          - name: "extract_requirements"
            kind: "function"
            public: false
          - name: "check_duplicate_requirements"
            kind: "function"
            public: false
          - name: "check_cross_references"
            kind: "function"
            public: false
          - name: "check_requirement_completeness"
            kind: "function"
            public: false
          - name: "validate_batch"
            kind: "function"
            public: true
          - name: "check_cross_file_duplicates"
            kind: "function"
            public: false
          - name: "RequirementInfo"
            kind: "struct"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validator"
      - path: "projects/agentic-workflow/src/validator/challenge.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "ChallengeValidator"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validator"
      - path: "projects/agentic-workflow/src/validator/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "challenge"
            kind: "module"
            public: true
          - name: "consistency"
            kind: "module"
            public: true
          - name: "fix"
            kind: "module"
            public: true
          - name: "format"
            kind: "module"
            public: true
          - name: "schema"
            kind: "module"
            public: true
          - name: "semantic"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validator"
      - path: "projects/agentic-workflow/src/validator/schema.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "DocumentType"
            kind: "enum"
            public: true
          - name: "SchemaValidator"
            kind: "struct"
            public: true
          - name: "schema_filename"
            kind: "function"
            public: true
          - name: "from_filename"
            kind: "function"
            public: true
          - name: "from_type_field"
            kind: "function"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "get_validator"
            kind: "function"
            public: false
          - name: "validate_file"
            kind: "function"
            public: true
          - name: "validate_content"
            kind: "function"
            public: true
          - name: "detect_document_type"
            kind: "function"
            public: false
          - name: "validate_required_fields"
            kind: "function"
            public: true
          - name: "validate_frontmatter_schema"
            kind: "function"
            public: true
          - name: "validate_frontmatter_content"
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
          domain: "projects/agentic-workflow/src/validator"
      - path: "projects/agentic-workflow/src/validator/fix.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "AutoFixer"
            kind: "struct"
            public: true
          - name: "FixResult"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "fix_errors"
            kind: "function"
            public: true
          - name: "apply_fix"
            kind: "function"
            public: false
          - name: "has_valid_acceptance_criteria"
            kind: "function"
            public: false
          - name: "fix_missing_heading"
            kind: "function"
            public: false
          - name: "fix_missing_when_then"
            kind: "function"
            public: false
          - name: "fix_missing_scenario"
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
          domain: "projects/agentic-workflow/src/validator"
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
  - path: "projects/agentic-workflow/src/validator/format.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validator/consistency.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validator/semantic.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validator/challenge.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validator/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validator/schema.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validator/fix.rs"
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
