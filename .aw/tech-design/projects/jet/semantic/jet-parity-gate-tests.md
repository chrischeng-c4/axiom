---
id: semantic-jet-parity-gate-tests
summary: Semantic coverage for "projects/jet/parity/gate/tests"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/gate/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/parity/gate/tests"
  source_group: "projects/jet/parity/gate/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/gate/tests/gate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "repo_root"
            kind: "function"
            public: false
          - name: "default_manifest_path"
            kind: "function"
            public: false
          - name: "default_waivers_path"
            kind: "function"
            public: false
          - name: "sample_result"
            kind: "function"
            public: false
          - name: "t1_parses_default_manifest"
            kind: "function"
            public: false
          - name: "t2_rejects_unknown_channel"
            kind: "function"
            public: false
          - name: "t3_rejects_pixel_delta_out_of_range"
            kind: "function"
            public: false
          - name: "t4_empty_waivers_parses"
            kind: "function"
            public: false
          - name: "t5_expired_waiver_does_not_apply"
            kind: "function"
            public: false
          - name: "manifest_with"
            kind: "function"
            public: false
          - name: "t6_all_pass_returns_zero"
            kind: "function"
            public: false
          - name: "t7_blocking_fail_returns_one"
            kind: "function"
            public: false
          - name: "t8_soft_fail_returns_two"
            kind: "function"
            public: false
          - name: "t9_no_results_returns_skipped"
            kind: "function"
            public: false
          - name: "t10_init_scaffold"
            kind: "function"
            public: false
          - name: "parse_dir_round_trip"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/parity/gate/tests"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/jet/parity/gate/tests/gate.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/gate/tests/gate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-parity-gate-tests.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
