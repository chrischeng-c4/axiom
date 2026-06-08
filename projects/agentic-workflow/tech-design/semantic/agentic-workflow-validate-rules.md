---
id: semantic-agentic-workflow-validate-rules
summary: Semantic coverage for "projects/agentic-workflow/src/validate/rules"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/validate/rules

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/validate/rules"
  source_group: "projects/agentic-workflow/src/validate/rules"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/validate/rules/r7c_duplicate_section.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "DuplicateSectionRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/mermaid_plus.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r7f_field_near_match.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "FieldNearMatchRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r6a_loose_root_file.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "LooseRootFileRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "locate_in_crate_spec_root"
            kind: "function"
            public: true
          - name: "is_codegen_fixture_spec"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r3e_impl_mode_misuse.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "ALLOWED"
            kind: "constant"
            public: false
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "ImplModeMisuseRule"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r7b_format_priority_violation.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "REQUIRED"
            kind: "constant"
            public: false
          - name: "PROSE_ONLY"
            kind: "constant"
            public: false
          - name: "FormatPriorityViolationRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r3c_orphan_binding.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "check_schema"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "OrphanBindingRule"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r7e_schema_conflict.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "SchemaConflictRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r3g_rust_type_consistency.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "RustTypeConsistencyRule"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "r3a_double_option"
            kind: "module"
            public: true
          - name: "r3b_nullable_required"
            kind: "module"
            public: true
          - name: "r3c_orphan_binding"
            kind: "module"
            public: true
          - name: "r3d_lowercase_enum"
            kind: "module"
            public: true
          - name: "r3e_impl_mode_misuse"
            kind: "module"
            public: true
          - name: "r3f_codegen_ready"
            kind: "module"
            public: true
          - name: "r3g_rust_type_consistency"
            kind: "module"
            public: true
          - name: "r6a_loose_root_file"
            kind: "module"
            public: true
          - name: "r6b_unexpected_subdir"
            kind: "module"
            public: true
          - name: "r7a_missing_section_annotation"
            kind: "module"
            public: true
          - name: "r7b_format_priority_violation"
            kind: "module"
            public: true
          - name: "r7c_duplicate_section"
            kind: "module"
            public: true
          - name: "r7d_orphan_requirement"
            kind: "module"
            public: true
          - name: "r7e_schema_conflict"
            kind: "module"
            public: true
          - name: "r7f_field_near_match"
            kind: "module"
            public: true
          - name: "section_format"
            kind: "module"
            public: true
          - name: "all_rules"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r7a_missing_section_annotation.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "MissingSectionAnnotationRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r7d_orphan_requirement.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "OrphanRequirementRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r6b_unexpected_subdir.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "ALLOWED_TOP_DIRS"
            kind: "constant"
            public: false
          - name: "UnexpectedSubdirRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/section_format.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "DEFAULT_LOOKAHEAD"
            kind: "constant"
            public: true
          - name: "PLACEHOLDER_MARKER"
            kind: "constant"
            public: true
          - name: "SectionFormatRule"
            kind: "struct"
            public: true
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "check_section_format"
            kind: "function"
            public: true
          - name: "find_annotation_line"
            kind: "function"
            public: false
          - name: "has_placeholder"
            kind: "function"
            public: false
          - name: "find_fence_in_window"
            kind: "function"
            public: false
          - name: "fence_lang_matches"
            kind: "function"
            public: false
          - name: "mermaid_starts_with_frontmatter"
            kind: "function"
            public: false
          - name: "UiBudgetConfig"
            kind: "struct"
            public: false
          - name: "UiBudgetProject"
            kind: "struct"
            public: false
          - name: "UiBudgetWorkspace"
            kind: "struct"
            public: false
          - name: "UiBudgetProfile"
            kind: "struct"
            public: false
          - name: "UiTaskMetric"
            kind: "struct"
            public: false
          - name: "UI_BUDGET_SECTION_TYPES"
            kind: "constant"
            public: false
          - name: "check_ui_complexity_budgets"
            kind: "function"
            public: false
          - name: "find_score_project_root"
            kind: "function"
            public: false
          - name: "find_ui_workspace"
            kind: "function"
            public: false
          - name: "extract_ui_tasks"
            kind: "function"
            public: false
          - name: "collect_tasks_from_value"
            kind: "function"
            public: false
          - name: "collect_tasks_from_sequence"
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
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r3a_double_option.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "extract_rust_type_value"
            kind: "function"
            public: false
          - name: "contains_double_option"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "DoubleOptionRule"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r3f_codegen_ready.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "has_signature"
            kind: "function"
            public: false
          - name: "split_frontmatter"
            kind: "function"
            public: false
          - name: "extract_sections"
            kind: "function"
            public: false
          - name: "parse_section_type"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "CodegenReadyRule"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r3b_nullable_required.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "check_schema"
            kind: "function"
            public: false
          - name: "is_option_type"
            kind: "function"
            public: false
          - name: "YamlBlock"
            kind: "struct"
            public: true
          - name: "extract_yaml_blocks"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "NullableRequiredRule"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
      - path: "projects/agentic-workflow/src/validate/rules/r3d_lowercase_enum.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "id"
            kind: "function"
            public: false
          - name: "check"
            kind: "function"
            public: false
          - name: "check_schema"
            kind: "function"
            public: false
          - name: "flag_if_not_pascal"
            kind: "function"
            public: false
          - name: "is_pascal_case"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "LowercaseEnumRule"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/validate/rules"
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
  - path: "projects/agentic-workflow/src/validate/rules/r7c_duplicate_section.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/mermaid_plus.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r7f_field_near_match.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r6a_loose_root_file.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r3e_impl_mode_misuse.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r7b_format_priority_violation.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r3c_orphan_binding.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r7e_schema_conflict.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r3g_rust_type_consistency.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r7a_missing_section_annotation.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r7d_orphan_requirement.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r6b_unexpected_subdir.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/section_format.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r3a_double_option.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r3f_codegen_ready.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r3b_nullable_required.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/validate/rules/r3d_lowercase_enum.rs"
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
