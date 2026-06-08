---
id: semantic-jet-parity-oracle-src-channels
summary: Semantic coverage for "projects/jet/parity/oracle/src/channels"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/oracle/src/channels

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/parity/oracle/src/channels"
  source_group: "projects/jet/parity/oracle/src/channels"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/oracle/src/channels/ime.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ImeChannel"
            kind: "struct"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "capture"
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
          domain: "projects/jet/parity/oracle/src/channels"
      - path: "projects/jet/parity/oracle/src/channels/pixel.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "PixelChannel"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "capture"
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
          domain: "projects/jet/parity/oracle/src/channels"
      - path: "projects/jet/parity/oracle/src/channels/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "a11y"
            kind: "module"
            public: true
          - name: "focus"
            kind: "module"
            public: true
          - name: "ime"
            kind: "module"
            public: true
          - name: "pixel"
            kind: "module"
            public: true
          - name: "pointer"
            kind: "module"
            public: true
          - name: "ChannelArtifact"
            kind: "enum"
            public: true
          - name: "ChannelError"
            kind: "enum"
            public: true
          - name: "ChannelCtx"
            kind: "struct"
            public: true
          - name: "DeterministicPrng"
            kind: "struct"
            public: true
          - name: "from_fixture_name"
            kind: "function"
            public: true
          - name: "next_u64"
            kind: "function"
            public: true
          - name: "next_point"
            kind: "function"
            public: true
          - name: "fnv1a64"
            kind: "function"
            public: true
          - name: "PointerHit"
            kind: "struct"
            public: true
          - name: "FocusEntry"
            kind: "struct"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/oracle/src/channels"
      - path: "projects/jet/parity/oracle/src/channels/focus.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "FocusChannel"
            kind: "struct"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "capture"
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
          domain: "projects/jet/parity/oracle/src/channels"
      - path: "projects/jet/parity/oracle/src/channels/pointer.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "PointerChannel"
            kind: "struct"
            public: true
          - name: "POINTER_SAMPLES"
            kind: "constant"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "capture"
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
          domain: "projects/jet/parity/oracle/src/channels"
      - path: "projects/jet/parity/oracle/src/channels/a11y.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "A11yChannel"
            kind: "struct"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "capture"
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
          domain: "projects/jet/parity/oracle/src/channels"
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
  - path: "projects/jet/parity/oracle/src/channels/ime.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/oracle/src/channels/pixel.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/oracle/src/channels/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/oracle/src/channels/focus.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/oracle/src/channels/pointer.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/oracle/src/channels/a11y.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
