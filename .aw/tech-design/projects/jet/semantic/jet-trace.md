---
id: semantic-jet-trace
summary: Semantic coverage for "projects/jet/src/trace"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/trace

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/trace"
  source_group: "projects/jet/src/trace"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/trace/manifest.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "MANIFEST_VERSION"
            kind: "constant"
            public: true
          - name: "TraceManifest"
            kind: "struct"
            public: true
          - name: "TraceOutcome"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "TraceEvent"
            kind: "enum"
            public: true
          - name: "ActionStepEvent"
            kind: "struct"
            public: true
          - name: "ActionKind"
            kind: "enum"
            public: true
          - name: "ConsoleEvent"
            kind: "struct"
            public: true
          - name: "ConsoleLevel"
            kind: "enum"
            public: true
          - name: "NetworkEvent"
            kind: "struct"
            public: true
          - name: "ScreenshotEvent"
            kind: "struct"
            public: true
          - name: "encode_ndjson"
            kind: "function"
            public: true
          - name: "decode_ndjson"
            kind: "function"
            public: true
          - name: "TraceManifestHeader"
            kind: "struct"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/trace"
      - path: "projects/jet/src/trace/server.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "VIEWER_HTML"
            kind: "constant"
            public: false
          - name: "VIEWER_JS"
            kind: "constant"
            public: false
          - name: "VIEWER_CSS"
            kind: "constant"
            public: false
          - name: "build_viewer_html"
            kind: "function"
            public: false
          - name: "ViewerState"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "build_router"
            kind: "function"
            public: true
          - name: "handle_root"
            kind: "function"
            public: false
          - name: "handle_trace_json"
            kind: "function"
            public: false
          - name: "handle_asset"
            kind: "function"
            public: false
          - name: "handle_not_found"
            kind: "function"
            public: false
          - name: "content_type_for_asset_id"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/trace"
      - path: "projects/jet/src/trace/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "archive"
            kind: "module"
            public: true
          - name: "buffer"
            kind: "module"
            public: true
          - name: "manifest"
            kind: "module"
            public: true
          - name: "server"
            kind: "module"
            public: true
          - name: "view"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/trace"
      - path: "projects/jet/src/trace/buffer.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "TraceMode"
            kind: "enum"
            public: true
          - name: "from_str"
            kind: "function"
            public: true
          - name: "is_active"
            kind: "function"
            public: true
          - name: "TraceBuffer"
            kind: "struct"
            public: true
          - name: "safe_trace_now_ms"
            kind: "function"
            public: true
          - name: "format_safe_trace_now_ms_warn"
            kind: "function"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "elapsed_ms"
            kind: "function"
            public: false
          - name: "asset_id"
            kind: "function"
            public: false
          - name: "append_action_step"
            kind: "function"
            public: true
          - name: "append_console"
            kind: "function"
            public: true
          - name: "append_network"
            kind: "function"
            public: true
          - name: "append_screenshot"
            kind: "function"
            public: true
          - name: "flush"
            kind: "function"
            public: true
          - name: "commit_trace"
            kind: "function"
            public: true
          - name: "commit_trace_with_shard"
            kind: "function"
            public: true
          - name: "TRACE_SHARD_FALLBACK_STEM"
            kind: "constant"
            public: true
          - name: "format_trace_shard_no_stem_warn"
            kind: "function"
            public: true
          - name: "format_trace_shard_non_utf8_stem_warn"
            kind: "function"
            public: true
          - name: "format_trace_shard_no_parent_warn"
            kind: "function"
            public: true
          - name: "derive_shard_trace_stem_or_warn"
            kind: "function"
            public: true
          - name: "derive_shard_trace_dir_or_warn"
            kind: "function"
            public: true
          - name: "gh3673_safe_trace_now_ms_tests"
            kind: "module"
            public: false
          - name: "gh3795_shard_trace_naming_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/trace"
      - path: "projects/jet/src/trace/archive.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "TraceAsset"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "write_trace_zip"
            kind: "function"
            public: true
          - name: "read_manifest_from_zip"
            kind: "function"
            public: true
          - name: "read_asset_from_zip"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/trace"
      - path: "projects/jet/src/trace/view.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "run"
            kind: "function"
            public: true
          - name: "show"
            kind: "function"
            public: true
          - name: "extract"
            kind: "function"
            public: true
          - name: "shutdown_signal"
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
          domain: "projects/jet/src/trace"
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
  - path: "projects/jet/src/trace/manifest.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/trace/server.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/trace/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/trace/buffer.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/trace/archive.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/trace/view.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-trace.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
