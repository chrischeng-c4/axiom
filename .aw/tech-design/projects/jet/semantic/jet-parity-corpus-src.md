---
id: semantic-jet-parity-corpus-src
summary: Semantic coverage for "projects/jet/parity/corpus/src"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/corpus/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/parity/corpus/src"
  source_group: "projects/jet/parity/corpus/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/corpus/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "cli"
            kind: "module"
            public: true
          - name: "hash"
            kind: "module"
            public: true
          - name: "manifest"
            kind: "module"
            public: true
          - name: "verify"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/corpus/src"
      - path: "projects/jet/parity/corpus/src/manifest.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ObservationChannel"
            kind: "enum"
            public: true
          - name: "as_kebab"
            kind: "function"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "FixtureEntry"
            kind: "struct"
            public: true
          - name: "FixtureManifest"
            kind: "struct"
            public: true
          - name: "RawManifest"
            kind: "struct"
            public: false
          - name: "CorpusError"
            kind: "enum"
            public: true
          - name: "fixture_id_regex"
            kind: "function"
            public: false
          - name: "parse_manifest"
            kind: "function"
            public: true
          - name: "from_kebab"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/corpus/src"
      - path: "projects/jet/parity/corpus/src/verify.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "FixtureStatus"
            kind: "enum"
            public: true
          - name: "VerifyEntry"
            kind: "struct"
            public: true
          - name: "DriftedFixture"
            kind: "struct"
            public: true
          - name: "VerifyReport"
            kind: "struct"
            public: true
          - name: "verify"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/corpus/src"
      - path: "projects/jet/parity/corpus/src/main.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/corpus/src"
      - path: "projects/jet/parity/corpus/src/hash.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "hash_jsx_file"
            kind: "function"
            public: true
          - name: "hash_bytes"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/corpus/src"
      - path: "projects/jet/parity/corpus/src/cli.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "DEFAULT_MANIFEST"
            kind: "constant"
            public: false
          - name: "FixturesCli"
            kind: "struct"
            public: true
          - name: "FixturesCmd"
            kind: "enum"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "dispatch"
            kind: "function"
            public: true
          - name: "io_err"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/corpus/src"
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
  - path: "projects/jet/parity/corpus/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/corpus/src/manifest.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/corpus/src/verify.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/corpus/src/main.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/corpus/src/hash.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/corpus/src/cli.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
