---
id: semantic-jet-wasm-src-debug
summary: Semantic coverage for "projects/jet/wasm/src/debug"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm/src/debug

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm/src/debug"
  source_group: "projects/jet/wasm/src/debug"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/wasm/src/debug/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "DebugBridgeState"
            kind: "struct"
            public: true
          - name: "RepaintTrigger"
            kind: "type"
            public: true
          - name: "JetDebug"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "element_tree"
            kind: "function"
            public: true
          - name: "layout_tree"
            kind: "function"
            public: true
          - name: "paint_ops"
            kind: "function"
            public: true
          - name: "fiber_tree"
            kind: "function"
            public: true
          - name: "hook_values"
            kind: "function"
            public: true
          - name: "pick_at"
            kind: "function"
            public: true
          - name: "highlight"
            kind: "function"
            public: true
          - name: "force_rerender"
            kind: "function"
            public: true
          - name: "DebugElement"
            kind: "enum"
            public: false
          - name: "from_element"
            kind: "function"
            public: false
          - name: "DebugProps"
            kind: "struct"
            public: false
          - name: "DebugLayoutTree"
            kind: "struct"
            public: false
          - name: "from_layout"
            kind: "function"
            public: false
          - name: "DebugLaidOutNode"
            kind: "struct"
            public: false
          - name: "from_node"
            kind: "function"
            public: false
          - name: "DebugLaidOutKind"
            kind: "enum"
            public: false
          - name: "DebugRect"
            kind: "struct"
            public: false
          - name: "from_rect"
            kind: "function"
            public: false
          - name: "DebugPoint"
            kind: "struct"
            public: false
          - name: "from_point"
            kind: "function"
            public: false
          - name: "DebugColor"
            kind: "struct"
            public: false
          - name: "from_color"
            kind: "function"
            public: false
          - name: "DebugFontSpec"
            kind: "struct"
            public: false
          - name: "from_font"
            kind: "function"
            public: false
          - name: "DebugPaintOp"
            kind: "enum"
            public: false
          - name: "from_op"
            kind: "function"
            public: false
          - name: "DebugFiber"
            kind: "struct"
            public: false
          - name: "DebugHook"
            kind: "struct"
            public: false
          - name: "DebugPickResult"
            kind: "struct"
            public: false
          - name: "contains"
            kind: "function"
            public: false
          - name: "to_js"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/debug"
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
  - path: "projects/jet/wasm/src/debug/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
