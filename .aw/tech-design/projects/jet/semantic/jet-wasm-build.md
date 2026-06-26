---
id: semantic-jet-wasm-build
summary: Semantic coverage for "projects/jet/src/wasm_build"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/wasm_build

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/wasm_build"
  source_group: "projects/jet/src/wasm_build"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/wasm_build/config.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "WasmConfig"
            kind: "struct"
            public: true
          - name: "WasmRenderer"
            kind: "enum"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "WASM_SECTION_KEYS"
            kind: "constant"
            public: false
          - name: "TOP_LEVEL_KEYS"
            kind: "constant"
            public: false
          - name: "DeprecatedKeyEntry"
            kind: "struct"
            public: false
          - name: "DEPRECATED_KEYS"
            kind: "constant"
            public: false
          - name: "DeprecatedKeyWarning"
            kind: "struct"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "RootPropValue"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "ConfigFile"
            kind: "struct"
            public: false
          - name: "ConfigSpan"
            kind: "struct"
            public: true
          - name: "ConfigError"
            kind: "enum"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "load_typed"
            kind: "function"
            public: true
          - name: "parse_str"
            kind: "function"
            public: true
          - name: "parse_str_with_warnings"
            kind: "function"
            public: true
          - name: "classify_toml_error"
            kind: "function"
            public: false
          - name: "extract_unknown_field"
            kind: "function"
            public: false
          - name: "candidate_keys_for"
            kind: "function"
            public: false
          - name: "parse_expected_list"
            kind: "function"
            public: false
          - name: "span_for_byte_range"
            kind: "function"
            public: false
          - name: "nearest_candidate"
            kind: "function"
            public: true
          - name: "levenshtein"
            kind: "function"
            public: false
          - name: "apply_deprecated_remap"
            kind: "function"
            public: false
          - name: "find_key_span"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "legacy_anyhow_message"
            kind: "function"
            public: false
          - name: "_typecheck_context_compat"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/wasm_build"
      - path: "projects/jet/src/wasm_build/manifest.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "Manifest"
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
          - name: "ManifestInputs"
            kind: "struct"
            public: true
          - name: "build"
            kind: "function"
            public: true
          - name: "profile_target_for"
            kind: "function"
            public: false
          - name: "package_for"
            kind: "function"
            public: false
          - name: "artifact_for"
            kind: "function"
            public: false
          - name: "rustc_target_for"
            kind: "function"
            public: false
          - name: "hex_lower"
            kind: "function"
            public: false
          - name: "write"
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
          domain: "projects/jet/src/wasm_build"
      - path: "projects/jet/src/wasm_build/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "config"
            kind: "module"
            public: true
          - name: "lint"
            kind: "module"
            public: true
          - name: "manifest"
            kind: "module"
            public: true
          - name: "schema"
            kind: "module"
            public: true
          - name: "Profile"
            kind: "enum"
            public: true
          - name: "build"
            kind: "function"
            public: true
          - name: "build_with_profile"
            kind: "function"
            public: true
          - name: "ComponentSource"
            kind: "struct"
            public: false
          - name: "resolve_component_source"
            kind: "function"
            public: false
          - name: "copy_style_import_groups"
            kind: "function"
            public: false
          - name: "write_target_manifest"
            kind: "function"
            public: false
          - name: "manifest_cargo_features"
            kind: "function"
            public: false
          - name: "emit_wasm_entry"
            kind: "function"
            public: false
          - name: "snake"
            kind: "function"
            public: false
          - name: "scaffold_cargo_project"
            kind: "function"
            public: false
          - name: "workspace_root"
            kind: "function"
            public: false
          - name: "workspace_root_from"
            kind: "function"
            public: false
          - name: "compute_rustup_bin"
            kind: "function"
            public: true
          - name: "compose_wasm_pack_path"
            kind: "function"
            public: true
          - name: "format_wasm_pack_env_warn"
            kind: "function"
            public: true
          - name: "run_wasm_pack"
            kind: "function"
            public: false
          - name: "copy_pkg_outputs"
            kind: "function"
            public: false
          - name: "boot_js"
            kind: "function"
            public: false
          - name: "host_bridge_js"
            kind: "function"
            public: false
          - name: "index_html"
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
          domain: "projects/jet/src/wasm_build"
      - path: "projects/jet/src/wasm_build/lint.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "LintOutcome"
            kind: "enum"
            public: true
          - name: "to_exit_code"
            kind: "function"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "lint_path"
            kind: "function"
            public: true
          - name: "LintReport"
            kind: "struct"
            public: true
          - name: "outcome"
            kind: "function"
            public: true
          - name: "print_report"
            kind: "function"
            public: false
          - name: "format_human"
            kind: "function"
            public: true
          - name: "format_json"
            kind: "function"
            public: true
          - name: "error_to_json"
            kind: "function"
            public: false
          - name: "warning_to_json"
            kind: "function"
            public: false
          - name: "span_json"
            kind: "function"
            public: false
          - name: "json_string"
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
          domain: "projects/jet/src/wasm_build"
      - path: "projects/jet/src/wasm_build/schema.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "SchemaOutcome"
            kind: "enum"
            public: true
          - name: "to_exit_code"
            kind: "function"
            public: true
          - name: "SCHEMA_REL_PATH"
            kind: "constant"
            public: true
          - name: "build_schema"
            kind: "function"
            public: true
          - name: "render"
            kind: "function"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "check_against_disk"
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
          domain: "projects/jet/src/wasm_build"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

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
  - path: "projects/jet/src/wasm_build/config.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/wasm_build/manifest.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/wasm_build/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/wasm_build/lint.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/wasm_build/schema.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-wasm-build.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
