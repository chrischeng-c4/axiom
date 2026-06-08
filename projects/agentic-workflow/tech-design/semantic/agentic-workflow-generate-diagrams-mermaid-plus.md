---
id: semantic-agentic-workflow-generate-diagrams-mermaid-plus
summary: Semantic coverage for "projects/agentic-workflow/src/generate/diagrams/mermaid_plus"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/generate/diagrams/mermaid_plus

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/generate/diagrams/mermaid_plus"
  source_group: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/validator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Severity"
            kind: "enum"
            public: true
          - name: "ValidationResult"
            kind: "struct"
            public: true
          - name: "ValidationError"
            kind: "struct"
            public: true
          - name: "StateMachineValidator"
            kind: "struct"
            public: true
          - name: "ok"
            kind: "function"
            public: true
          - name: "with_error"
            kind: "function"
            public: true
          - name: "with_warning"
            kind: "function"
            public: true
          - name: "StateInfo"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "strict"
            kind: "function"
            public: true
          - name: "validate"
            kind: "function"
            public: true
          - name: "collect_states"
            kind: "function"
            public: false
          - name: "validate_transition"
            kind: "function"
            public: false
          - name: "validate_target"
            kind: "function"
            public: false
          - name: "find_reachable"
            kind: "function"
            public: false
          - name: "get_targets"
            kind: "function"
            public: false
          - name: "validate_guard_refs"
            kind: "function"
            public: false
          - name: "validate_action_refs"
            kind: "function"
            public: false
          - name: "default"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/migrate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "MIGRATE_TOOL_VERSION"
            kind: "constant"
            public: true
          - name: "PAYLOAD_DIR"
            kind: "constant"
            public: true
          - name: "MigrationOptions"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "MigrateState"
            kind: "enum"
            public: true
          - name: "DiagramKind"
            kind: "enum"
            public: true
          - name: "MigrationEnvelope"
            kind: "struct"
            public: true
          - name: "run_migration"
            kind: "function"
            public: true
          - name: "enumerate_envelopes"
            kind: "function"
            public: true
          - name: "apply_block_payload"
            kind: "function"
            public: true
          - name: "format_block_id"
            kind: "function"
            public: false
          - name: "parse_open_line"
            kind: "function"
            public: false
          - name: "resolve_payload_path"
            kind: "function"
            public: false
          - name: "relativize"
            kind: "function"
            public: false
          - name: "detect_diagram_kind"
            kind: "function"
            public: true
          - name: "render_body_from_frontmatter"
            kind: "function"
            public: false
          - name: "mermaid_equivalent"
            kind: "function"
            public: true
          - name: "canonical_form"
            kind: "function"
            public: false
          - name: "render_state_diagram_local"
            kind: "function"
            public: false
          - name: "render_flowchart_local"
            kind: "function"
            public: false
          - name: "render_sequence_diagram_local"
            kind: "function"
            public: false
          - name: "render_requirement_diagram_local"
            kind: "function"
            public: false
          - name: "LegacyOrPlusBlock"
            kind: "struct"
            public: false
          - name: "enumerate_mermaid_blocks"
            kind: "function"
            public: false
          - name: "build_converted_block"
            kind: "function"
            public: false
          - name: "splice_blocks"
            kind: "function"
            public: false
          - name: "atomic_write"
            kind: "function"
            public: false
          - name: "now_iso8601"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "generator"
            kind: "module"
            public: false
          - name: "migrate"
            kind: "module"
            public: true
          - name: "schema"
            kind: "module"
            public: false
          - name: "validator"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "StateType"
            kind: "enum"
            public: true
          - name: "TransitionInput"
            kind: "enum"
            public: true
          - name: "ActionRef"
            kind: "enum"
            public: true
          - name: "StateMachineDef"
            kind: "struct"
            public: true
          - name: "StateNodeDef"
            kind: "struct"
            public: true
          - name: "TransitionDetail"
            kind: "struct"
            public: true
          - name: "GuardDef"
            kind: "struct"
            public: true
          - name: "ActionDef"
            kind: "struct"
            public: true
          - name: "to_vec"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "BlockMigrationResult"
            kind: "struct"
            public: true
          - name: "BlockMigrationStatus"
            kind: "enum"
            public: true
          - name: "MigrationAudit"
            kind: "struct"
            public: true
          - name: "MigrationMode"
            kind: "enum"
            public: true
          - name: "MigrationReport"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus"
      - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/generator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "MermaidPlusOutput"
            kind: "struct"
            public: true
          - name: "MermaidPlusGenerator"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "generate"
            kind: "function"
            public: true
          - name: "generate_frontmatter"
            kind: "function"
            public: false
          - name: "generate_mermaid"
            kind: "function"
            public: true
          - name: "generate_states"
            kind: "function"
            public: false
          - name: "generate_single_state"
            kind: "function"
            public: false
          - name: "generate_transition"
            kind: "function"
            public: false
          - name: "generate_detailed_transition"
            kind: "function"
            public: false
          - name: "default"
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
          domain: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus"
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
  - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/validator.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/migrate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diagrams/mermaid_plus/generator.rs"
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
