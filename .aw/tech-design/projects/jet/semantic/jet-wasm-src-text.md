---
id: semantic-jet-wasm-src-text
summary: Semantic coverage for "projects/jet/wasm/src/text"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm/src/text

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm/src/text"
  source_group: "projects/jet/wasm/src/text"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/wasm/src/text/cache.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ShapeCacheKey"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "ShapeCache"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "get"
            kind: "function"
            public: true
          - name: "insert"
            kind: "function"
            public: true
          - name: "len"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "clear"
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
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/shaped.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ShapedGlyph"
            kind: "struct"
            public: true
          - name: "ShapedRun"
            kind: "struct"
            public: true
          - name: "empty"
            kind: "function"
            public: true
          - name: "line_height"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/line_break.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "LineBreakKind"
            kind: "enum"
            public: true
          - name: "LineBreakOpportunity"
            kind: "struct"
            public: true
          - name: "LineBreaker"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "classify"
            kind: "function"
            public: false
          - name: "next"
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
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/paint_bridge.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "DrawGlyph"
            kind: "struct"
            public: true
          - name: "GlyphPaintOp"
            kind: "enum"
            public: true
          - name: "Origin"
            kind: "struct"
            public: true
          - name: "emit_draw_glyphs"
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
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/shaper.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "shape_text"
            kind: "function"
            public: true
          - name: "measure_text"
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
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/bidi.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Direction"
            kind: "enum"
            public: true
          - name: "BidiRun"
            kind: "struct"
            public: true
          - name: "BidiResolver"
            kind: "struct"
            public: true
          - name: "resolve"
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
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "bidi"
            kind: "module"
            public: true
          - name: "cache"
            kind: "module"
            public: true
          - name: "font_face"
            kind: "module"
            public: true
          - name: "line_break"
            kind: "module"
            public: true
          - name: "paint_bridge"
            kind: "module"
            public: true
          - name: "paragraph"
            kind: "module"
            public: true
          - name: "script_run"
            kind: "module"
            public: true
          - name: "shaped"
            kind: "module"
            public: true
          - name: "shaper"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/script_run.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ScriptRun"
            kind: "struct"
            public: true
          - name: "script_runs"
            kind: "function"
            public: true
          - name: "script_full_name"
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
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/paragraph.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "Paragraph"
            kind: "struct"
            public: true
          - name: "empty"
            kind: "function"
            public: true
          - name: "shape_paragraph"
            kind: "function"
            public: true
          - name: "intersections"
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
          domain: "projects/jet/wasm/src/text"
      - path: "projects/jet/wasm/src/text/font_face.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "FontError"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "FontFace"
            kind: "struct"
            public: true
          - name: "from_bytes"
            kind: "function"
            public: true
          - name: "from_bytes_with_index"
            kind: "function"
            public: true
          - name: "face"
            kind: "function"
            public: true
          - name: "ascent_at"
            kind: "function"
            public: true
          - name: "descent_at"
            kind: "function"
            public: true
          - name: "scale"
            kind: "function"
            public: false
          - name: "bytes"
            kind: "function"
            public: true
          - name: "eq"
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
          domain: "projects/jet/wasm/src/text"
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
  - path: "projects/jet/wasm/src/text/cache.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/shaped.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/line_break.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/paint_bridge.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/shaper.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/bidi.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/script_run.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/paragraph.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/text/font_face.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
