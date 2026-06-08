---
id: semantic-jet-wasm-src-renderer-layout
summary: Semantic coverage for "projects/jet/wasm/src/renderer/layout"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm/src/renderer/layout

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm/src/renderer/layout"
  source_group: "projects/jet/wasm/src/renderer/layout"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/wasm/src/renderer/layout/dirty.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "propagate_ancestors"
            kind: "function"
            public: true
          - name: "closure_with_ancestors"
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
          domain: "projects/jet/wasm/src/renderer/layout"
      - path: "projects/jet/wasm/src/renderer/layout/style_parser.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "ParseError"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: false
          - name: "SILENT_IGNORE"
            kind: "constant"
            public: false
          - name: "ERROR_PROPERTIES"
            kind: "constant"
            public: false
          - name: "ERROR_PROPERTY_VALUES"
            kind: "constant"
            public: false
          - name: "parse_style"
            kind: "function"
            public: true
          - name: "normalize_key"
            kind: "function"
            public: false
          - name: "parse_display"
            kind: "function"
            public: false
          - name: "parse_flex_direction"
            kind: "function"
            public: false
          - name: "parse_justify_content"
            kind: "function"
            public: false
          - name: "parse_align_items"
            kind: "function"
            public: false
          - name: "parse_dimension"
            kind: "function"
            public: false
          - name: "assign_dim"
            kind: "function"
            public: false
          - name: "parse_rect4_shorthand"
            kind: "function"
            public: false
          - name: "assign_rect4"
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
          domain: "projects/jet/wasm/src/renderer/layout"
      - path: "projects/jet/wasm/src/renderer/layout/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "dirty"
            kind: "module"
            public: true
          - name: "style_parser"
            kind: "module"
            public: true
          - name: "LayoutNodeId"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "DisplayKind"
            kind: "enum"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "FlexDirection"
            kind: "enum"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "JustifyContent"
            kind: "enum"
            public: true
          - name: "AlignItems"
            kind: "enum"
            public: true
          - name: "Dimension"
            kind: "enum"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "Rect4"
            kind: "struct"
            public: true
          - name: "LayoutStyle"
            kind: "struct"
            public: true
          - name: "LayoutNode"
            kind: "struct"
            public: true
          - name: "Viewport"
            kind: "struct"
            public: true
          - name: "Rect"
            kind: "struct"
            public: true
          - name: "LaidOutNode"
            kind: "struct"
            public: true
          - name: "LayoutTree"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "upsert"
            kind: "function"
            public: true
          - name: "set_root"
            kind: "function"
            public: true
          - name: "mark_dirty"
            kind: "function"
            public: true
          - name: "dirty_nodes"
            kind: "function"
            public: true
          - name: "last_viewport"
            kind: "function"
            public: true
          - name: "parent_map"
            kind: "function"
            public: true
          - name: "layout"
            kind: "function"
            public: true
          - name: "walk_extract"
            kind: "function"
            public: false
          - name: "layout_style_to_taffy"
            kind: "function"
            public: false
          - name: "dimension_to_taffy"
            kind: "function"
            public: false
          - name: "dimension_to_lp"
            kind: "function"
            public: false
          - name: "dimension_to_lpa"
            kind: "function"
            public: false
          - name: "rect4_to_taffy_lp"
            kind: "function"
            public: false
          - name: "rect4_to_taffy_lpa"
            kind: "function"
            public: false
          - name: "leaf"
            kind: "function"
            public: true
          - name: "parent"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/renderer/layout"
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
  - path: "projects/jet/wasm/src/renderer/layout/dirty.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/renderer/layout/style_parser.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/renderer/layout/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
