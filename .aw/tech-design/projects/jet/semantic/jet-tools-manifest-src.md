---
id: semantic-jet-tools-manifest-src
summary: Semantic coverage for "projects/jet/tools/manifest/src"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tools/manifest/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/tools/manifest/src"
  source_group: "projects/jet/tools/manifest/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tools/manifest/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "format_safe_manifest_canonicalize_err"
            kind: "function"
            public: true
          - name: "safe_manifest_canonicalize"
            kind: "function"
            public: true
          - name: "ParseManifestModule"
            kind: "struct"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "command"
            kind: "function"
            public: false
          - name: "execute"
            kind: "function"
            public: false
          - name: "render_yaml"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3699_safe_manifest_canonicalize_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/tools/manifest/src"
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
  - path: "projects/jet/tools/manifest/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-tools-manifest-src.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
