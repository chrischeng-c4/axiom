---
id: semantic-agentic-workflow-generate-gen-python
summary: Semantic coverage for "projects/agentic-workflow/src/generate/gen/python"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/generate/gen/python

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/generate/gen/python"
  source_group: "projects/agentic-workflow/src/generate/gen/python"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/generate/gen/python/types.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "PythonBackendSpec"
            kind: "struct"
            public: true
          - name: "RouterIr"
            kind: "struct"
            public: true
          - name: "RouteRecord"
            kind: "struct"
            public: true
          - name: "HttpMethod"
            kind: "enum"
            public: true
          - name: "decorator"
            kind: "function"
            public: true
          - name: "PydanticModelIr"
            kind: "struct"
            public: true
          - name: "PydanticField"
            kind: "struct"
            public: true
          - name: "ImportIr"
            kind: "struct"
            public: true
          - name: "PythonModuleIr"
            kind: "struct"
            public: true
          - name: "PythonModuleItemIr"
            kind: "enum"
            public: true
          - name: "PythonClassItemIr"
            kind: "enum"
            public: true
          - name: "EmittedPythonFile"
            kind: "struct"
            public: true
          - name: "PythonBodyKind"
            kind: "enum"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/gen/python"
      - path: "projects/agentic-workflow/src/generate/gen/python/pydantic_model.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "emit_pydantic_model"
            kind: "function"
            public: true
          - name: "emit_pydantic_module"
            kind: "function"
            public: true
          - name: "snake_case"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/gen/python"
      - path: "projects/agentic-workflow/src/generate/gen/python/determinism.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "normalize"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/gen/python"
      - path: "projects/agentic-workflow/src/generate/gen/python/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "ast"
            kind: "module"
            public: true
          - name: "determinism"
            kind: "module"
            public: true
          - name: "lower"
            kind: "module"
            public: true
          - name: "module"
            kind: "module"
            public: true
          - name: "pydantic_model"
            kind: "module"
            public: true
          - name: "router"
            kind: "module"
            public: true
          - name: "types"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/gen/python"
      - path: "projects/agentic-workflow/src/generate/gen/python/router.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "emit_router"
            kind: "function"
            public: true
          - name: "route_stmt"
            kind: "function"
            public: false
          - name: "router_imports"
            kind: "function"
            public: false
          - name: "import_key"
            kind: "function"
            public: false
          - name: "collect_models"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/gen/python"
      - path: "projects/agentic-workflow/src/generate/gen/python/module.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "emit_python_module"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate/gen/python"
      - path: "projects/agentic-workflow/src/generate/gen/python/ast.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "PythonModuleAst"
            kind: "struct"
            public: true
          - name: "PythonImportStmt"
            kind: "struct"
            public: true
          - name: "PythonClassDef"
            kind: "struct"
            public: true
          - name: "PythonModuleStmt"
            kind: "enum"
            public: true
          - name: "PythonClassStmt"
            kind: "enum"
            public: true
          - name: "PythonFunctionDef"
            kind: "struct"
            public: true
          - name: "PythonFunctionStmt"
            kind: "enum"
            public: true
          - name: "pydantic_module_ast"
            kind: "function"
            public: true
          - name: "pydantic_model_ast"
            kind: "function"
            public: true
          - name: "python_module_ir_ast"
            kind: "function"
            public: true
          - name: "python_module_ir_from_source"
            kind: "function"
            public: true
          - name: "import_ast"
            kind: "function"
            public: false
          - name: "module_item_stmt"
            kind: "function"
            public: false
          - name: "class_item_stmt"
            kind: "function"
            public: false
          - name: "function_def"
            kind: "function"
            public: false
          - name: "field_stmt"
            kind: "function"
            public: false
          - name: "render_python_module"
            kind: "function"
            public: true
          - name: "render_import"
            kind: "function"
            public: false
          - name: "import_kind"
            kind: "function"
            public: false
          - name: "render_module_stmt"
            kind: "function"
            public: false
          - name: "render_class"
            kind: "function"
            public: false
          - name: "render_function"
            kind: "function"
            public: false
          - name: "render_function_with_indent"
            kind: "function"
            public: false
          - name: "render_raw_lines"
            kind: "function"
            public: false
          - name: "strip_score_handwrite_envelope"
            kind: "function"
            public: false
          - name: "strip_comment_lead"
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
          domain: "projects/agentic-workflow/src/generate/gen/python"
      - path: "projects/agentic-workflow/src/generate/gen/python/lower.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "PythonBackendSpecYaml"
            kind: "struct"
            public: false
          - name: "PythonRouterYaml"
            kind: "struct"
            public: false
          - name: "PythonRouteYaml"
            kind: "struct"
            public: false
          - name: "PythonPydanticModelYaml"
            kind: "struct"
            public: false
          - name: "PythonPydanticFieldYaml"
            kind: "struct"
            public: false
          - name: "PythonImportYaml"
            kind: "struct"
            public: false
          - name: "default_pydantic_base"
            kind: "function"
            public: false
          - name: "lower_backend_spec_yaml"
            kind: "function"
            public: true
          - name: "lower_backend_spec_value"
            kind: "function"
            public: true
          - name: "lower_backend_spec"
            kind: "function"
            public: false
          - name: "default_spec_id"
            kind: "function"
            public: false
          - name: "trim_trailing_newlines"
            kind: "function"
            public: false
          - name: "parse_http_method"
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
          domain: "projects/agentic-workflow/src/generate/gen/python"
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
  - path: "projects/agentic-workflow/src/generate/gen/python/types.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/gen/python/pydantic_model.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/gen/python/determinism.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/gen/python/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/gen/python/router.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/gen/python/module.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/gen/python/ast.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/gen/python/lower.rs"
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
