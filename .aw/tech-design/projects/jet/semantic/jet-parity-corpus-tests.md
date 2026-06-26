---
id: semantic-jet-parity-corpus-tests
summary: Semantic coverage for "projects/jet/parity/corpus/tests"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/corpus/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/parity/corpus/tests"
  source_group: "projects/jet/parity/corpus/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/corpus/tests/corpus.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "workspace_root"
            kind: "function"
            public: false
          - name: "corpus_root"
            kind: "function"
            public: false
          - name: "manifest_path"
            kind: "function"
            public: false
          - name: "copy_corpus_into"
            kind: "function"
            public: false
          - name: "copy_dir_recursive"
            kind: "function"
            public: false
          - name: "t1_manifest_parse_minimal_corpus"
            kind: "function"
            public: false
          - name: "t2_manifest_rejects_unknown_channel"
            kind: "function"
            public: false
          - name: "t3_manifest_rejects_malformed_id"
            kind: "function"
            public: false
          - name: "t4_hash_jsx_is_deterministic"
            kind: "function"
            public: false
          - name: "t5_hash_jsx_matches_known_vector"
            kind: "function"
            public: false
          - name: "t6_verify_clean_corpus"
            kind: "function"
            public: false
          - name: "t7_verify_detects_one_byte_jsx_edit"
            kind: "function"
            public: false
          - name: "t8_cli_list_show_verify_smoke"
            kind: "function"
            public: false
          - name: "observation_channel_kebab_roundtrip"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/parity/corpus/tests"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/jet/parity/corpus/tests/corpus.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/corpus/tests/corpus.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-parity-corpus-tests.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
