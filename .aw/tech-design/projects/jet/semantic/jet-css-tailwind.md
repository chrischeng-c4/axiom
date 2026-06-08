---
id: semantic-jet-css-tailwind
summary: Semantic coverage for "projects/jet/src/css/tailwind"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/css/tailwind

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/css/tailwind"
  source_group: "projects/jet/src/css/tailwind"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/css/tailwind/variants.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "BREAKPOINTS"
            kind: "constant"
            public: false
          - name: "ParsedClass"
            kind: "struct"
            public: true
          - name: "parse"
            kind: "function"
            public: true
          - name: "wrap_with_variants"
            kind: "function"
            public: true
          - name: "breakpoint_min_width"
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
          domain: "projects/jet/src/css/tailwind"
      - path: "projects/jet/src/css/tailwind/config.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "DarkMode"
            kind: "enum"
            public: true
          - name: "ThemeConfig"
            kind: "struct"
            public: true
          - name: "TailwindConfig"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "load"
            kind: "function"
            public: true
          - name: "from_js"
            kind: "function"
            public: true
          - name: "from_toml"
            kind: "function"
            public: true
          - name: "parse_js_config"
            kind: "function"
            public: false
          - name: "parse_yaml_config"
            kind: "function"
            public: false
          - name: "strip_js_comments"
            kind: "function"
            public: false
          - name: "extract_exports_body"
            kind: "function"
            public: false
          - name: "extract_object_field"
            kind: "function"
            public: false
          - name: "extract_array_field"
            kind: "function"
            public: false
          - name: "extract_string_field"
            kind: "function"
            public: false
          - name: "extract_string_array"
            kind: "function"
            public: false
          - name: "extract_string_map"
            kind: "function"
            public: false
          - name: "extract_plugins"
            kind: "function"
            public: false
          - name: "find_matching_brace"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "find_matching_bracket"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css/tailwind"
      - path: "projects/jet/src/css/tailwind/scanner.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ContentScanner"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "scan"
            kind: "function"
            public: true
          - name: "build_glob_set"
            kind: "function"
            public: false
          - name: "extract_classes"
            kind: "function"
            public: true
          - name: "extract_from_class_attributes"
            kind: "function"
            public: false
          - name: "extract_from_clsx_cn"
            kind: "function"
            public: false
          - name: "extract_bare_tokens"
            kind: "function"
            public: false
          - name: "collect_class_tokens"
            kind: "function"
            public: false
          - name: "looks_like_tailwind_class"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "find_closing_quote"
            kind: "function"
            public: false
          - name: "gh3310_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css/tailwind"
      - path: "projects/jet/src/css/tailwind/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "config"
            kind: "module"
            public: true
          - name: "preflight"
            kind: "module"
            public: true
          - name: "scanner"
            kind: "module"
            public: true
          - name: "utilities"
            kind: "module"
            public: true
          - name: "variants"
            kind: "module"
            public: true
          - name: "TailwindLayers"
            kind: "struct"
            public: true
          - name: "concat"
            kind: "function"
            public: true
          - name: "TailwindEmitter"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "scan"
            kind: "function"
            public: true
          - name: "generate"
            kind: "function"
            public: true
          - name: "build_base"
            kind: "function"
            public: false
          - name: "build_components"
            kind: "function"
            public: false
          - name: "build_utilities"
            kind: "function"
            public: false
          - name: "emit_class"
            kind: "function"
            public: false
          - name: "resolve_declarations"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "escape_selector"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css/tailwind"
      - path: "projects/jet/src/css/tailwind/preflight.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface"]
        symbols:
          - name: "PREFLIGHT"
            kind: "constant"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/css/tailwind"
      - path: "projects/jet/src/css/tailwind/utilities.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "class_to_css"
            kind: "function"
            public: true
          - name: "apply_to_declarations"
            kind: "function"
            public: true
          - name: "build_exact_table"
            kind: "function"
            public: false
          - name: "generate_utility"
            kind: "function"
            public: false
          - name: "generate_spacing"
            kind: "function"
            public: false
          - name: "parse_spacing_value"
            kind: "function"
            public: false
          - name: "generate_sizing"
            kind: "function"
            public: false
          - name: "generate_grid"
            kind: "function"
            public: false
          - name: "generate_color_utility"
            kind: "function"
            public: false
          - name: "resolve_color"
            kind: "function"
            public: true
          - name: "tailwind_color_rgb"
            kind: "function"
            public: false
          - name: "generate_arbitrary"
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
          domain: "projects/jet/src/css/tailwind"
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
  - path: "projects/jet/src/css/tailwind/variants.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/tailwind/config.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/tailwind/scanner.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/tailwind/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/tailwind/preflight.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/css/tailwind/utilities.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-css-tailwind.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
