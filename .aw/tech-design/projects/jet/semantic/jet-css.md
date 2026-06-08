---
id: semantic-jet-css
summary: Semantic coverage for "projects/jet/src/css"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/css

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/css"
  source_group: "projects/jet/src/css"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/css/import_resolver.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "resolve_imports"
            kind: "function"
            public: true
          - name: "resolve_source"
            kind: "function"
            public: true
          - name: "resolve_file"
            kind: "function"
            public: false
          - name: "process_source"
            kind: "function"
            public: false
          - name: "resolve_import_path"
            kind: "function"
            public: false
          - name: "extract_import_path"
            kind: "function"
            public: false
          - name: "strip_quotes"
            kind: "function"
            public: false
          - name: "is_remote"
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
          domain: "projects/jet/src/css"
      - path: "projects/jet/src/css/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "directives"
            kind: "module"
            public: true
          - name: "import_resolver"
            kind: "module"
            public: true
          - name: "output"
            kind: "module"
            public: true
          - name: "plugins"
            kind: "module"
            public: true
          - name: "tailwind"
            kind: "module"
            public: true
          - name: "CssPipeline"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "process"
            kind: "function"
            public: true
          - name: "process_source"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "apply_lightningcss"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css"
      - path: "projects/jet/src/css/output.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "CssOutput"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "sha256_prefix"
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
          domain: "projects/jet/src/css"
      - path: "projects/jet/src/css/directives.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "has_tailwind_directives"
            kind: "function"
            public: true
          - name: "has_apply_directives"
            kind: "function"
            public: true
          - name: "inject_tailwind_layers"
            kind: "function"
            public: true
          - name: "expand_apply"
            kind: "function"
            public: true
          - name: "LayerOutput"
            kind: "struct"
            public: true
          - name: "process_layer_directives"
            kind: "function"
            public: true
          - name: "process_directives"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "find_matching_close_brace"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css"
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
  - path: "projects/jet/src/css/import_resolver.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/output.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/directives.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-css.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
