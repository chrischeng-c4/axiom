---
id: semantic-jet-parity-gate-src
summary: Semantic coverage for "projects/jet/parity/gate/src"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/gate/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/parity/gate/src"
  source_group: "projects/jet/parity/gate/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/gate/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "cli"
            kind: "module"
            public: true
          - name: "gate"
            kind: "module"
            public: true
          - name: "init"
            kind: "module"
            public: true
          - name: "manifest"
            kind: "module"
            public: true
          - name: "result"
            kind: "module"
            public: true
          - name: "waivers"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/gate/src"
      - path: "projects/jet/parity/gate/src/manifest.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "CANONICAL_CHANNELS"
            kind: "constant"
            public: true
          - name: "Channel"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "GateError"
            kind: "enum"
            public: true
          - name: "Tolerance"
            kind: "struct"
            public: true
          - name: "AdapterSelector"
            kind: "struct"
            public: true
          - name: "GatingManifest"
            kind: "struct"
            public: true
          - name: "parse"
            kind: "function"
            public: true
          - name: "validate"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/gate/src"
      - path: "projects/jet/parity/gate/src/waivers.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "Waiver"
            kind: "struct"
            public: true
          - name: "Waivers"
            kind: "struct"
            public: true
          - name: "parse"
            kind: "function"
            public: true
          - name: "applies_to"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/gate/src"
      - path: "projects/jet/parity/gate/src/main.rs"
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
          domain: "projects/jet/parity/gate/src"
      - path: "projects/jet/parity/gate/src/init.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "MANIFEST_TEMPLATE"
            kind: "constant"
            public: false
          - name: "WAIVERS_TEMPLATE"
            kind: "constant"
            public: false
          - name: "DOCS_TEMPLATE"
            kind: "constant"
            public: false
          - name: "InitReport"
            kind: "struct"
            public: true
          - name: "run_init"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/gate/src"
      - path: "projects/jet/parity/gate/src/gate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "EXIT_PASS"
            kind: "constant"
            public: true
          - name: "EXIT_BLOCKING_FAIL"
            kind: "constant"
            public: true
          - name: "EXIT_SOFT_FAIL"
            kind: "constant"
            public: true
          - name: "EXIT_SKIPPED"
            kind: "constant"
            public: true
          - name: "GateReport"
            kind: "struct"
            public: true
          - name: "run_gate"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/gate/src"
      - path: "projects/jet/parity/gate/src/result.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Status"
            kind: "enum"
            public: true
          - name: "DiffKind"
            kind: "enum"
            public: true
          - name: "ChannelResult"
            kind: "struct"
            public: true
          - name: "parse_dir"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/gate/src"
      - path: "projects/jet/parity/gate/src/cli.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "DEFAULT_MANIFEST"
            kind: "constant"
            public: false
          - name: "DEFAULT_WAIVERS"
            kind: "constant"
            public: false
          - name: "DEFAULT_RESULTS_DIR"
            kind: "constant"
            public: false
          - name: "DEFAULT_INIT_DIR"
            kind: "constant"
            public: false
          - name: "Cli"
            kind: "struct"
            public: true
          - name: "Command"
            kind: "enum"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "dispatch"
            kind: "function"
            public: false
          - name: "print_report"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/gate/src"
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
  - path: "projects/jet/parity/gate/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/gate/src/manifest.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/gate/src/waivers.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/gate/src/main.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/gate/src/init.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/gate/src/gate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/gate/src/result.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/gate/src/cli.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
