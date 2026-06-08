---
id: semantic-jet-parity-oracle-tests
summary: Semantic coverage for "projects/jet/parity/oracle/tests"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/oracle/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/parity/oracle/tests"
  source_group: "projects/jet/parity/oracle/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/oracle/tests/runner_smoke.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "fixture_path"
            kind: "function"
            public: false
          - name: "make_config"
            kind: "function"
            public: false
          - name: "run_with_stub"
            kind: "function"
            public: false
          - name: "test_runner_emits_five_artifacts"
            kind: "function"
            public: false
          - name: "test_non_ime_fixture_writes_empty_ime_json"
            kind: "function"
            public: false
          - name: "test_ime_fixture_captures_composition"
            kind: "function"
            public: false
          - name: "test_pixel_artifact_naming"
            kind: "function"
            public: false
          - name: "test_a11y_artifact_is_verbatim_axtree"
            kind: "function"
            public: false
          - name: "test_focus_trace_length_and_shape"
            kind: "function"
            public: false
          - name: "test_pointer_hitmap_seeded_1000"
            kind: "function"
            public: false
          - name: "test_byte_equivalent_replay"
            kind: "function"
            public: false
          - name: "test_cdp_session_attached"
            kind: "function"
            public: false
          - name: "test_mount_sentinel_timeout"
            kind: "function"
            public: false
          - name: "test_per_fixture_budget_under_8s"
            kind: "function"
            public: false
          - name: "test_runner_live_chromium_smoke"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/parity/oracle/tests"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/jet/parity/oracle/tests/runner_smoke.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/oracle/tests/runner_smoke.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-parity-oracle-tests.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
