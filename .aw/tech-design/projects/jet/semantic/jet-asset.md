---
id: semantic-jet-asset
summary: Semantic coverage for "projects/jet/src/asset"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/asset

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/asset"
  source_group: "projects/jet/src/asset"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/asset/types.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model"]
        symbols:
          - name: "AssetOptions"
            kind: "struct"
            public: true
          - name: "ProcessedAsset"
            kind: "struct"
            public: true
          - name: "AssetType"
            kind: "enum"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/asset"
      - path: "projects/jet/src/asset/image_processor.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "MIN_OPTIMIZE_SIZE"
            kind: "constant"
            public: false
          - name: "DEFAULT_JPEG_QUALITY"
            kind: "constant"
            public: false
          - name: "format_image_step_err"
            kind: "function"
            public: true
          - name: "format_image_filename_err"
            kind: "function"
            public: true
          - name: "optimize_image"
            kind: "function"
            public: true
          - name: "optimize_jpeg"
            kind: "function"
            public: false
          - name: "optimize_png"
            kind: "function"
            public: false
          - name: "format_svg_non_utf8_warn"
            kind: "function"
            public: true
          - name: "optimize_svg"
            kind: "function"
            public: false
          - name: "collapse_svg_whitespace"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3621_image_extensionless_no_trailing_dot_tests"
            kind: "module"
            public: false
          - name: "gh3734_svg_non_utf8_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/asset"
      - path: "projects/jet/src/asset/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "image_processor"
            kind: "module"
            public: true
          - name: "types"
            kind: "module"
            public: true
          - name: "format_asset_hashed_filename_err"
            kind: "function"
            public: true
          - name: "safe_asset_filename"
            kind: "function"
            public: true
          - name: "format_asset_filename_err"
            kind: "function"
            public: true
          - name: "lowercase_extension_or_warn"
            kind: "function"
            public: true
          - name: "format_asset_non_utf8_extension_warn"
            kind: "function"
            public: true
          - name: "format_asset_hashed_filename_non_utf8_stem_warn"
            kind: "function"
            public: true
          - name: "format_asset_hashed_filename_non_utf8_ext_warn"
            kind: "function"
            public: true
          - name: "coerce_hashed_filename_stem_or_warn"
            kind: "function"
            public: true
          - name: "coerce_hashed_filename_ext_or_warn"
            kind: "function"
            public: true
          - name: "AssetProcessor"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "process"
            kind: "function"
            public: true
          - name: "detect_type"
            kind: "function"
            public: false
          - name: "process_image"
            kind: "function"
            public: false
          - name: "process_font"
            kind: "function"
            public: false
          - name: "process_generic"
            kind: "function"
            public: false
          - name: "compute_hash"
            kind: "function"
            public: false
          - name: "create_hashed_filename"
            kind: "function"
            public: false
          - name: "default"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3618_create_hashed_filename_tests"
            kind: "module"
            public: false
          - name: "gh3634_safe_asset_filename_tests"
            kind: "module"
            public: false
          - name: "gh3774_non_utf8_extension_warn_tests"
            kind: "module"
            public: false
          - name: "gh3805_hashed_filename_non_utf8_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/asset"
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
  - path: "projects/jet/src/asset/types.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/asset/image_processor.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/asset/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-asset.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
