---
id: semantic-jet-tauri-shell-src
summary: Semantic coverage for "projects/jet/tauri-shell/src"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tauri-shell/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/tauri-shell/src"
  source_group: "projects/jet/tauri-shell/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tauri-shell/src/bridge.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "BridgeFuture"
            kind: "type"
            public: true
          - name: "BridgeError"
            kind: "enum"
            public: true
          - name: "RpcRequest"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "RpcResponse"
            kind: "struct"
            public: true
          - name: "ok"
            kind: "function"
            public: true
          - name: "err"
            kind: "function"
            public: true
          - name: "RPC_METHOD_NOT_FOUND"
            kind: "constant"
            public: true
          - name: "RPC_INVALID_PARAMS"
            kind: "constant"
            public: true
          - name: "RPC_INTERNAL"
            kind: "constant"
            public: true
          - name: "RpcError"
            kind: "struct"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "dispatch_envelope"
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
          domain: "projects/jet/tauri-shell/src"
      - path: "projects/jet/tauri-shell/src/packager.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "FileCopy"
            kind: "struct"
            public: true
          - name: "PlannedCommand"
            kind: "struct"
            public: true
          - name: "HostOs"
            kind: "enum"
            public: true
          - name: "current"
            kind: "function"
            public: true
          - name: "dist_subdir"
            kind: "function"
            public: true
          - name: "PackagePlanError"
            kind: "enum"
            public: true
          - name: "PackagePlan"
            kind: "struct"
            public: true
          - name: "for_host"
            kind: "function"
            public: true
          - name: "for_current_host"
            kind: "function"
            public: true
          - name: "source_paths"
            kind: "function"
            public: true
          - name: "dest_paths"
            kind: "function"
            public: true
          - name: "plan_package"
            kind: "function"
            public: true
          - name: "PackageError"
            kind: "enum"
            public: true
          - name: "CopyReport"
            kind: "struct"
            public: true
          - name: "execute_copies"
            kind: "function"
            public: true
          - name: "CommandReport"
            kind: "struct"
            public: true
          - name: "execute_command"
            kind: "function"
            public: true
          - name: "format_dry_run"
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
          domain: "projects/jet/tauri-shell/src"
      - path: "projects/jet/tauri-shell/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "bridge"
            kind: "module"
            public: true
          - name: "lifecycle"
            kind: "module"
            public: true
          - name: "packager"
            kind: "module"
            public: true
          - name: "SUPPORTED_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "BundleManifest"
            kind: "struct"
            public: true
          - name: "Artifact"
            kind: "struct"
            public: true
          - name: "Build"
            kind: "struct"
            public: true
          - name: "Source"
            kind: "struct"
            public: true
          - name: "ManifestError"
            kind: "enum"
            public: true
          - name: "from_artifact_dir"
            kind: "function"
            public: true
          - name: "from_path"
            kind: "function"
            public: true
          - name: "validate_for_desktop"
            kind: "function"
            public: true
          - name: "WindowConfig"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "with_title"
            kind: "function"
            public: true
          - name: "with_size"
            kind: "function"
            public: true
          - name: "locked"
            kind: "function"
            public: true
          - name: "TauriShell"
            kind: "struct"
            public: true
          - name: "from_artifact_dir"
            kind: "function"
            public: true
          - name: "with_window"
            kind: "function"
            public: true
          - name: "with_lifecycle_bus"
            kind: "function"
            public: true
          - name: "lifecycle"
            kind: "function"
            public: true
          - name: "manifest"
            kind: "function"
            public: true
          - name: "artifact_dir"
            kind: "function"
            public: true
          - name: "window"
            kind: "function"
            public: true
          - name: "plan_package"
            kind: "function"
            public: true
          - name: "entry_html_path"
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
          domain: "projects/jet/tauri-shell/src"
      - path: "projects/jet/tauri-shell/src/lifecycle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "LifecycleEvent"
            kind: "enum"
            public: true
          - name: "WindowId"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "main"
            kind: "function"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "LifecycleBus"
            kind: "struct"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "subscribe"
            kind: "function"
            public: true
          - name: "listener_count"
            kind: "function"
            public: true
          - name: "publish"
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
          domain: "projects/jet/tauri-shell/src"
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
  - path: "projects/jet/tauri-shell/src/bridge.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tauri-shell/src/packager.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tauri-shell/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tauri-shell/src/lifecycle.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
