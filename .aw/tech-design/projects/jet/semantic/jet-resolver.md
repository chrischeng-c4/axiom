---
id: semantic-jet-resolver
summary: Semantic coverage for "projects/jet/src/resolver"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/resolver

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/resolver"
  source_group: "projects/jet/src/resolver"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/resolver/alias.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "AliasResolver"
            kind: "struct"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "to_resolve_aliases"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "TsConfig"
            kind: "struct"
            public: false
          - name: "CompilerOptions"
            kind: "struct"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "load_tsconfig_paths"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/resolver"
      - path: "projects/jet/src/resolver/package.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "PackageJson"
            kind: "struct"
            public: true
          - name: "read_package_json"
            kind: "function"
            public: true
          - name: "get_package_main"
            kind: "function"
            public: true
          - name: "resolve_exports"
            kind: "function"
            public: true
          - name: "resolve_export_value"
            kind: "function"
            public: false
          - name: "match_export_pattern"
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
          domain: "projects/jet/src/resolver"
      - path: "projects/jet/src/resolver/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "alias"
            kind: "module"
            public: true
          - name: "package"
            kind: "module"
            public: true
          - name: "ModuleResolver"
            kind: "struct"
            public: true
          - name: "ResolveOptions"
            kind: "struct"
            public: true
          - name: "ResolvedModule"
            kind: "struct"
            public: true
          - name: "ResolveKind"
            kind: "enum"
            public: true
          - name: "parse_package_specifier"
            kind: "function"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "resolve"
            kind: "function"
            public: true
          - name: "detect_kind"
            kind: "function"
            public: false
          - name: "is_alias"
            kind: "function"
            public: false
          - name: "is_external"
            kind: "function"
            public: false
          - name: "resolve_relative"
            kind: "function"
            public: false
          - name: "resolve_absolute"
            kind: "function"
            public: false
          - name: "resolve_package"
            kind: "function"
            public: false
          - name: "resolve_package_dir"
            kind: "function"
            public: false
          - name: "resolve_alias"
            kind: "function"
            public: false
          - name: "try_extensions"
            kind: "function"
            public: false
          - name: "default"
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
          domain: "projects/jet/src/resolver"
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
  - path: "projects/jet/src/resolver/alias.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/resolver/package.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/resolver/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-resolver.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
