---
id: semantic-agentic-workflow-td-ast
summary: Semantic coverage for "projects/agentic-workflow/src/td_ast"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD lifecycle source behavior and TD AST processing."
---

# Semantic TD: agentic-workflow/td_ast

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/td_ast"
  source_group: "projects/agentic-workflow/src/td_ast"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/td_ast/ir.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "DbEntity"
            kind: "struct"
            public: true
          - name: "DbField"
            kind: "struct"
            public: true
          - name: "DbModelFrontmatter"
            kind: "struct"
            public: true
          - name: "DbModelIR"
            kind: "struct"
            public: true
          - name: "DbRelationship"
            kind: "struct"
            public: true
          - name: "DependencyFrontmatter"
            kind: "struct"
            public: true
          - name: "IREdge"
            kind: "struct"
            public: true
          - name: "IRFamily"
            kind: "struct"
            public: true
          - name: "IRNode"
            kind: "struct"
            public: true
          - name: "InteractionActor"
            kind: "struct"
            public: true
          - name: "InteractionFrontmatter"
            kind: "struct"
            public: true
          - name: "InteractionIR"
            kind: "struct"
            public: true
          - name: "InteractionMessage"
            kind: "struct"
            public: true
          - name: "LogicEdge"
            kind: "struct"
            public: true
          - name: "LogicFrontmatter"
            kind: "struct"
            public: true
          - name: "LogicGraphIR"
            kind: "struct"
            public: true
          - name: "LogicNode"
            kind: "struct"
            public: true
          - name: "LowerDiagnostic"
            kind: "struct"
            public: true
          - name: "MermaidPlusBlockTyped"
            kind: "struct"
            public: true
          - name: "MermaidPlusFrontmatter"
            kind: "struct"
            public: true
          - name: "MindmapFrontmatter"
            kind: "struct"
            public: true
          - name: "MindmapNode"
            kind: "struct"
            public: true
          - name: "ParseDiagnostic"
            kind: "struct"
            public: true
          - name: "RequirementItem"
            kind: "struct"
            public: true
          - name: "RequirementSetIR"
            kind: "struct"
            public: true
          - name: "RequirementsFrontmatter"
            kind: "struct"
            public: true
          - name: "ScenarioItem"
            kind: "struct"
            public: true
          - name: "ScenarioSetIR"
            kind: "struct"
            public: true
          - name: "ScenariosFrontmatter"
            kind: "struct"
            public: true
          - name: "SourceSpan"
            kind: "struct"
            public: true
          - name: "StateEdge"
            kind: "struct"
            public: true
          - name: "StateMachineFrontmatter"
            kind: "struct"
            public: true
          - name: "StateMachineIR"
            kind: "struct"
            public: true
          - name: "StateNode"
            kind: "struct"
            public: true
          - name: "TestItem"
            kind: "struct"
            public: true
          - name: "TestPlanFrontmatter"
            kind: "struct"
            public: true
          - name: "TestPlanIR"
            kind: "struct"
            public: true
          - name: "enter"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/types.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "TDAst"
            kind: "struct"
            public: true
          - name: "TDSection"
            kind: "struct"
            public: true
          - name: "TdParseError"
            kind: "struct"
            public: true
          - name: "TypedBody"
            kind: "enum"
            public: true
          - name: "default_parse_kind"
            kind: "function"
            public: false
          - name: "MermaidPlusPayload"
            kind: "struct"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "SectionKind"
            kind: "enum"
            public: true
          - name: "for_section_type"
            kind: "function"
            public: true
          - name: "DbEntity"
            kind: "struct"
            public: true
          - name: "DbField"
            kind: "struct"
            public: true
          - name: "DbModelFrontmatter"
            kind: "struct"
            public: true
          - name: "DbModelIR"
            kind: "struct"
            public: true
          - name: "DbRelationship"
            kind: "struct"
            public: true
          - name: "DependencyFrontmatter"
            kind: "struct"
            public: true
          - name: "IREdge"
            kind: "struct"
            public: true
          - name: "IRFamily"
            kind: "struct"
            public: true
          - name: "IRNode"
            kind: "struct"
            public: true
          - name: "InteractionActor"
            kind: "struct"
            public: true
          - name: "InteractionFrontmatter"
            kind: "struct"
            public: true
          - name: "InteractionIR"
            kind: "struct"
            public: true
          - name: "InteractionMessage"
            kind: "struct"
            public: true
          - name: "LogicEdge"
            kind: "struct"
            public: true
          - name: "LogicFrontmatter"
            kind: "struct"
            public: true
          - name: "LogicGraphIR"
            kind: "struct"
            public: true
          - name: "LogicNode"
            kind: "struct"
            public: true
          - name: "LowerDiagnostic"
            kind: "struct"
            public: true
          - name: "MermaidPlusBlockTyped"
            kind: "struct"
            public: true
          - name: "MermaidPlusFrontmatter"
            kind: "struct"
            public: true
          - name: "MindmapFrontmatter"
            kind: "struct"
            public: true
          - name: "MindmapNode"
            kind: "struct"
            public: true
          - name: "ParseDiagnostic"
            kind: "struct"
            public: true
          - name: "RequirementItem"
            kind: "struct"
            public: true
          - name: "RequirementSetIR"
            kind: "struct"
            public: true
          - name: "RequirementsFrontmatter"
            kind: "struct"
            public: true
          - name: "ScenarioItem"
            kind: "struct"
            public: true
          - name: "ScenarioSetIR"
            kind: "struct"
            public: true
          - name: "ScenariosFrontmatter"
            kind: "struct"
            public: true
          - name: "SourceSpan"
            kind: "struct"
            public: true
          - name: "StateEdge"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/mermaid_plus.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "DbEntity"
            kind: "struct"
            public: true
          - name: "DbField"
            kind: "struct"
            public: true
          - name: "DbModelFrontmatter"
            kind: "struct"
            public: true
          - name: "DbModelIR"
            kind: "struct"
            public: true
          - name: "DbRelationship"
            kind: "struct"
            public: true
          - name: "DependencyFrontmatter"
            kind: "struct"
            public: true
          - name: "IREdge"
            kind: "struct"
            public: true
          - name: "IRFamily"
            kind: "struct"
            public: true
          - name: "IRNode"
            kind: "struct"
            public: true
          - name: "InteractionActor"
            kind: "struct"
            public: true
          - name: "InteractionFrontmatter"
            kind: "struct"
            public: true
          - name: "InteractionIR"
            kind: "struct"
            public: true
          - name: "InteractionMessage"
            kind: "struct"
            public: true
          - name: "LogicEdge"
            kind: "struct"
            public: true
          - name: "LogicFrontmatter"
            kind: "struct"
            public: true
          - name: "LogicGraphIR"
            kind: "struct"
            public: true
          - name: "LogicNode"
            kind: "struct"
            public: true
          - name: "LowerDiagnostic"
            kind: "struct"
            public: true
          - name: "MermaidPlusBlockTyped"
            kind: "struct"
            public: true
          - name: "MermaidPlusFrontmatter"
            kind: "struct"
            public: true
          - name: "MindmapFrontmatter"
            kind: "struct"
            public: true
          - name: "MindmapNode"
            kind: "struct"
            public: true
          - name: "ParseDiagnostic"
            kind: "struct"
            public: true
          - name: "RequirementItem"
            kind: "struct"
            public: true
          - name: "RequirementSetIR"
            kind: "struct"
            public: true
          - name: "RequirementsFrontmatter"
            kind: "struct"
            public: true
          - name: "ScenarioItem"
            kind: "struct"
            public: true
          - name: "ScenarioSetIR"
            kind: "struct"
            public: true
          - name: "ScenariosFrontmatter"
            kind: "struct"
            public: true
          - name: "SourceSpan"
            kind: "struct"
            public: true
          - name: "StateEdge"
            kind: "struct"
            public: true
          - name: "StateMachineFrontmatter"
            kind: "struct"
            public: true
          - name: "StateMachineIR"
            kind: "struct"
            public: true
          - name: "StateNode"
            kind: "struct"
            public: true
          - name: "TestItem"
            kind: "struct"
            public: true
          - name: "TestPlanFrontmatter"
            kind: "struct"
            public: true
          - name: "TestPlanIR"
            kind: "struct"
            public: true
          - name: "enter"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/query.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "RefKind"
            kind: "enum"
            public: true
          - name: "Ref"
            kind: "struct"
            public: true
          - name: "TypeDef"
            kind: "struct"
            public: true
          - name: "is_type_declaring"
            kind: "function"
            public: false
          - name: "is_ref_bearing"
            kind: "function"
            public: false
          - name: "ref_kind_for"
            kind: "function"
            public: false
          - name: "find_section_by_type"
            kind: "function"
            public: true
          - name: "find_section_by_id"
            kind: "function"
            public: true
          - name: "resolve_type"
            kind: "function"
            public: true
          - name: "all_references_to"
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
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "entities"
            kind: "module"
            public: true
          - name: "parse"
            kind: "module"
            public: true
          - name: "types"
            kind: "module"
            public: true
          - name: "query"
            kind: "module"
            public: true
          - name: "validate"
            kind: "module"
            public: true
          - name: "payloads"
            kind: "module"
            public: true
          - name: "anti_patterns"
            kind: "module"
            public: true
          - name: "mermaid_plus"
            kind: "module"
            public: true
          - name: "enter"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/anti_patterns.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "check_content_anti_patterns"
            kind: "function"
            public: true
          - name: "check_filesystem_anti_patterns"
            kind: "function"
            public: true
          - name: "ap_001_placeholder_leftover"
            kind: "function"
            public: false
          - name: "slice_section_body"
            kind: "function"
            public: false
          - name: "ap_004_non_existent_spec_ref"
            kind: "function"
            public: false
          - name: "extract_spec_paths"
            kind: "function"
            public: false
          - name: "ap_008_body_equals_title"
            kind: "function"
            public: false
          - name: "body_equals_title_heuristic"
            kind: "function"
            public: false
          - name: "ap_009_non_existent_replaces_symbol"
            kind: "function"
            public: false
          - name: "ChangeEntryView"
            kind: "struct"
            public: false
          - name: "iter_changes_entries"
            kind: "function"
            public: false
          - name: "file_declares_symbol"
            kind: "function"
            public: false
          - name: "ap_010_changes_path_not_found"
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
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/parse.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "parse_error"
            kind: "function"
            public: false
          - name: "parse_td"
            kind: "function"
            public: true
          - name: "parse_td_str"
            kind: "function"
            public: true
          - name: "split_frontmatter"
            kind: "function"
            public: false
          - name: "extract_section_body"
            kind: "function"
            public: false
          - name: "first_code_fence"
            kind: "function"
            public: false
          - name: "code_fence_open"
            kind: "function"
            public: false
          - name: "code_fence_closes"
            kind: "function"
            public: false
          - name: "is_placeholder_block"
            kind: "function"
            public: false
          - name: "parse_typed_body"
            kind: "function"
            public: false
          - name: "compute_hash"
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
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/entities.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "EntityRef"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "entities"
            kind: "function"
            public: false
          - name: "mermaid_entities"
            kind: "function"
            public: false
          - name: "json_schema_entities"
            kind: "function"
            public: false
          - name: "openrpc_entities"
            kind: "function"
            public: false
          - name: "openapi_entities"
            kind: "function"
            public: false
          - name: "asyncapi_entities"
            kind: "function"
            public: false
          - name: "cli_entities"
            kind: "function"
            public: false
          - name: "config_entities"
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
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/payloads.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "JsonSchemaPayload"
            kind: "struct"
            public: true
          - name: "OpenRpcPayload"
            kind: "struct"
            public: true
          - name: "OpenApiPayload"
            kind: "struct"
            public: true
          - name: "AsyncApiPayload"
            kind: "struct"
            public: true
          - name: "CliManifestPayload"
            kind: "struct"
            public: true
          - name: "ConfigManifestPayload"
            kind: "struct"
            public: true
          - name: "PayloadTypeDef"
            kind: "struct"
            public: true
          - name: "RpcMethod"
            kind: "struct"
            public: true
          - name: "RpcParam"
            kind: "struct"
            public: true
          - name: "OpenApiPathItem"
            kind: "struct"
            public: true
          - name: "OpenApiOperation"
            kind: "struct"
            public: true
          - name: "AsyncApiChannel"
            kind: "struct"
            public: true
          - name: "CliCommandDef"
            kind: "struct"
            public: true
          - name: "CliArgDef"
            kind: "struct"
            public: true
          - name: "ConfigKeyDef"
            kind: "struct"
            public: true
          - name: "TdParseErrorKind"
            kind: "enum"
            public: true
          - name: "value_is_empty_mapping"
            kind: "function"
            public: false
          - name: "from_yaml_str"
            kind: "function"
            public: true
          - name: "from_yaml_str"
            kind: "function"
            public: true
          - name: "from_yaml_str"
            kind: "function"
            public: true
          - name: "extract_components_schemas"
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
          domain: "projects/agentic-workflow/src/td_ast"
      - path: "projects/agentic-workflow/src/td_ast/validate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "TdErrorCode"
            kind: "enum"
            public: true
          - name: "TdError"
            kind: "struct"
            public: true
          - name: "is_type_declaring"
            kind: "function"
            public: false
          - name: "is_ref_bearing"
            kind: "function"
            public: false
          - name: "validate_td"
            kind: "function"
            public: true
          - name: "validate_td_full"
            kind: "function"
            public: true
          - name: "check_duplicate_entities"
            kind: "function"
            public: false
          - name: "check_undefined_type_refs"
            kind: "function"
            public: false
          - name: "is_titlecase_ident"
            kind: "function"
            public: false
          - name: "check_orphan_changes_targets"
            kind: "function"
            public: false
          - name: "check_missing_required_sections"
            kind: "function"
            public: false
          - name: "section_type_from_str"
            kind: "function"
            public: false
          - name: "check_missing_schema_for_logic"
            kind: "function"
            public: false
          - name: "find_changes_section"
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
          domain: "projects/agentic-workflow/src/td_ast"
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
  - path: "projects/agentic-workflow/src/td_ast/ir.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/types.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/mermaid_plus.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/query.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/anti_patterns.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/parse.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/entities.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/payloads.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/td_ast/validate.rs"
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
