---
id: semantic-jet-wasm-src-react
summary: Semantic coverage for "projects/jet/wasm/src/react"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm/src/react

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm/src/react"
  source_group: "projects/jet/wasm/src/react"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/wasm/src/react/webgpu_app.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "JetWebGpuApp"
            kind: "struct"
            public: true
          - name: "destroy"
            kind: "function"
            public: true
          - name: "status"
            kind: "function"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "repaint"
            kind: "function"
            public: false
          - name: "read_text_glyph_count"
            kind: "function"
            public: false
          - name: "install_click_listener"
            kind: "function"
            public: false
          - name: "call_render_frame"
            kind: "function"
            public: false
          - name: "call_method0"
            kind: "function"
            public: false
          - name: "STATUS_KEY"
            kind: "constant"
            public: false
          - name: "init_status"
            kind: "function"
            public: false
          - name: "record_frame"
            kind: "function"
            public: false
          - name: "record_error"
            kind: "function"
            public: false
          - name: "set_status_str"
            kind: "function"
            public: false
          - name: "set_status_num"
            kind: "function"
            public: false
          - name: "set_status_value"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/react"
      - path: "projects/jet/wasm/src/react/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "canvas_app"
            kind: "module"
            public: true
          - name: "dom_app"
            kind: "module"
            public: true
          - name: "webgpu_app"
            kind: "module"
            public: true
          - name: "Fiber"
            kind: "struct"
            public: true
          - name: "FiberId"
            kind: "struct"
            public: true
          - name: "HookSlot"
            kind: "enum"
            public: true
          - name: "MemoDepHash"
            kind: "type"
            public: true
          - name: "Runtime"
            kind: "struct"
            public: false
          - name: "new_fiber"
            kind: "function"
            public: false
          - name: "fiber_mut"
            kind: "function"
            public: false
          - name: "begin_render"
            kind: "function"
            public: false
          - name: "end_render"
            kind: "function"
            public: false
          - name: "with_current_fiber"
            kind: "function"
            public: false
          - name: "use_state"
            kind: "function"
            public: true
          - name: "StateSetter"
            kind: "struct"
            public: true
          - name: "clone"
            kind: "function"
            public: false
          - name: "set"
            kind: "function"
            public: true
          - name: "use_effect_once"
            kind: "function"
            public: true
          - name: "DispatchHandle"
            kind: "struct"
            public: true
          - name: "clone"
            kind: "function"
            public: false
          - name: "dispatch"
            kind: "function"
            public: true
          - name: "use_reducer"
            kind: "function"
            public: true
          - name: "RefHandle"
            kind: "struct"
            public: true
          - name: "clone"
            kind: "function"
            public: false
          - name: "current"
            kind: "function"
            public: true
          - name: "set"
            kind: "function"
            public: true
          - name: "with_mut"
            kind: "function"
            public: true
          - name: "use_ref"
            kind: "function"
            public: true
          - name: "use_memo"
            kind: "function"
            public: true
          - name: "use_callback"
            kind: "function"
            public: true
          - name: "hash_dep"
            kind: "function"
            public: true
          - name: "mount"
            kind: "function"
            public: true
          - name: "MountHandle"
            kind: "struct"
            public: true
          - name: "snapshot"
            kind: "function"
            public: true
          - name: "flush"
            kind: "function"
            public: true
          - name: "mark_root_dirty"
            kind: "function"
            public: true
          - name: "DebugFiberMeta"
            kind: "struct"
            public: true
          - name: "debug_snapshot_fibers"
            kind: "function"
            public: true
          - name: "DebugHookSummary"
            kind: "struct"
            public: true
          - name: "debug_snapshot_hooks"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/react"
      - path: "projects/jet/wasm/src/react/dom_app.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "run"
            kind: "function"
            public: true
          - name: "install_flush_loop"
            kind: "function"
            public: false
          - name: "render_current"
            kind: "function"
            public: false
          - name: "build_node"
            kind: "function"
            public: false
          - name: "dom_tag"
            kind: "function"
            public: false
          - name: "apply_props"
            kind: "function"
            public: false
          - name: "install_events"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/react"
      - path: "projects/jet/wasm/src/react/canvas_app.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "run"
            kind: "function"
            public: true
          - name: "repaint"
            kind: "function"
            public: false
          - name: "install_click_listener"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/wasm/src/react"
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
  - path: "projects/jet/wasm/src/react/webgpu_app.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/react/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/react/dom_app.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/wasm/src/react/canvas_app.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
