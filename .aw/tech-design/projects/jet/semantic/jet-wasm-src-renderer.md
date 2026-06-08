---
id: semantic-jet-wasm-src-renderer
summary: Semantic coverage for "projects/jet/wasm/src/renderer"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm/src/renderer

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm/src/renderer"
  source_group: "projects/jet/wasm/src/renderer"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/wasm/src/renderer/webgpu.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "CELL_F32_STRIDE"
            kind: "constant"
            public: true
          - name: "WebGpuFramePlan"
            kind: "struct"
            public: true
          - name: "is_complete"
            kind: "function"
            public: true
          - name: "packed_f32_len"
            kind: "function"
            public: true
          - name: "write_packed_f32"
            kind: "function"
            public: true
          - name: "to_packed_f32"
            kind: "function"
            public: true
          - name: "to_float32_array"
            kind: "function"
            public: true
          - name: "to_js_text_runs"
            kind: "function"
            public: true
          - name: "WebGpuTextRun"
            kind: "struct"
            public: true
          - name: "to_js_object"
            kind: "function"
            public: true
          - name: "WebGpuUnsupportedOp"
            kind: "enum"
            public: true
          - name: "WebGpuBackend"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "plan"
            kind: "function"
            public: true
          - name: "last_frame"
            kind: "function"
            public: true
          - name: "backend_description"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "execute"
            kind: "function"
            public: false
          - name: "fill_rect_to_cell"
            kind: "function"
            public: false
          - name: "lower_stroke_rect"
            kind: "function"
            public: false
          - name: "color_to_f32"
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
          domain: "projects/jet/wasm/src/renderer"
      - path: "projects/jet/wasm/src/renderer/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "layout"
            kind: "module"
            public: true
          - name: "canvas"
            kind: "module"
            public: true
          - name: "webgpu"
            kind: "module"
            public: true
          - name: "Rect"
            kind: "struct"
            public: true
          - name: "top_left"
            kind: "function"
            public: true
          - name: "with_y"
            kind: "function"
            public: true
          - name: "with_height"
            kind: "function"
            public: true
          - name: "Point"
            kind: "struct"
            public: true
          - name: "Color"
            kind: "struct"
            public: true
          - name: "rgb"
            kind: "function"
            public: true
          - name: "FontSpec"
            kind: "struct"
            public: true
          - name: "BorderSpec"
            kind: "struct"
            public: true
          - name: "PaintOp"
            kind: "enum"
            public: true
          - name: "Viewport"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "Theme"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "LayoutTree"
            kind: "struct"
            public: true
          - name: "LaidOutNode"
            kind: "struct"
            public: true
          - name: "LaidOutKind"
            kind: "enum"
            public: true
          - name: "hit_test_on_click"
            kind: "function"
            public: true
          - name: "rect_contains"
            kind: "function"
            public: false
          - name: "layout"
            kind: "function"
            public: true
          - name: "Cursor"
            kind: "struct"
            public: false
          - name: "recurse"
            kind: "function"
            public: false
          - name: "measure_intrinsic"
            kind: "function"
            public: false
          - name: "measure_block_height"
            kind: "function"
            public: false
          - name: "default_height_for_tag"
            kind: "function"
            public: false
          - name: "is_block_container"
            kind: "function"
            public: false
          - name: "paint"
            kind: "function"
            public: true
          - name: "paint_intrinsic"
            kind: "function"
            public: false
          - name: "Renderer"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "render"
            kind: "function"
            public: true
          - name: "NoopBackend"
            kind: "struct"
            public: true
          - name: "execute"
            kind: "function"
            public: false
          - name: "RecordingBackend"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "execute"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/renderer"
      - path: "projects/jet/wasm/src/renderer/canvas.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "CanvasBackend"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "execute"
            kind: "function"
            public: false
          - name: "css_color"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/renderer"
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
  - path: "projects/jet/wasm/src/renderer/webgpu.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/renderer/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/renderer/canvas.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
