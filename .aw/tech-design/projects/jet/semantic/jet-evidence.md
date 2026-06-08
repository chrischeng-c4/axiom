---
id: semantic-jet-evidence
summary: Semantic coverage for "projects/jet/src/evidence"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/evidence

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/evidence"
  source_group: "projects/jet/src/evidence"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/evidence/bundle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "BUNDLE_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "BUNDLE_SCHEMA_CURRENT_MAJOR"
            kind: "constant"
            public: true
          - name: "MANIFEST_FILE_NAME"
            kind: "constant"
            public: true
          - name: "SchemaCompat"
            kind: "enum"
            public: true
          - name: "classify_schema_version"
            kind: "function"
            public: true
          - name: "validate_schema_version"
            kind: "function"
            public: true
          - name: "BundleCommand"
            kind: "enum"
            public: true
          - name: "BundleEnvironment"
            kind: "struct"
            public: true
          - name: "BundleArtifact"
            kind: "struct"
            public: true
          - name: "BundleManifest"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "BundleHandle"
            kind: "struct"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "write"
            kind: "function"
            public: true
          - name: "root"
            kind: "function"
            public: true
          - name: "manifest"
            kind: "function"
            public: true
          - name: "resolve"
            kind: "function"
            public: true
          - name: "ensure_relative_inside_root"
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
          domain: "projects/jet/src/evidence"
      - path: "projects/jet/src/evidence/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "bundle"
            kind: "module"
            public: true
          - name: "writer"
            kind: "module"
            public: true
          - name: "read_bundle_from_file"
            kind: "function"
            public: true
          - name: "read_events_from_jsonl"
            kind: "function"
            public: true
          - name: "events_for"
            kind: "function"
            public: true
          - name: "ReportAdapter"
            kind: "struct"
            public: true
          - name: "ArtifactAvailability"
            kind: "enum"
            public: true
          - name: "from_bundle"
            kind: "function"
            public: true
          - name: "from_file"
            kind: "function"
            public: true
          - name: "summary"
            kind: "function"
            public: true
          - name: "cases"
            kind: "function"
            public: true
          - name: "failures"
            kind: "function"
            public: true
          - name: "artifacts"
            kind: "function"
            public: true
          - name: "events"
            kind: "function"
            public: true
          - name: "bundle"
            kind: "function"
            public: true
          - name: "resolve_artifact"
            kind: "function"
            public: true
          - name: "EVIDENCE_ROOT_NO_PARENT_FALLBACK"
            kind: "constant"
            public: true
          - name: "format_evidence_no_parent_warn"
            kind: "function"
            public: true
          - name: "resolve_evidence_root_or_warn"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3797_evidence_root_no_parent_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/evidence"
      - path: "projects/jet/src/evidence/writer.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method", "test_case"]
        symbols:
          - name: "EVENTS_FILE_NAME"
            kind: "constant"
            public: true
          - name: "EvidenceEvent"
            kind: "enum"
            public: true
          - name: "EvidenceWriter"
            kind: "struct"
            public: true
          - name: "open"
            kind: "function"
            public: true
          - name: "root"
            kind: "function"
            public: true
          - name: "manifest"
            kind: "function"
            public: true
          - name: "append"
            kind: "function"
            public: false
          - name: "run_started"
            kind: "function"
            public: true
          - name: "test_result"
            kind: "function"
            public: true
          - name: "case_result"
            kind: "function"
            public: true
          - name: "test_retry"
            kind: "function"
            public: true
          - name: "case_retry"
            kind: "function"
            public: true
          - name: "register_artifact"
            kind: "function"
            public: true
          - name: "run_finished"
            kind: "function"
            public: true
          - name: "finalize"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/src/evidence"
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
  - path: "projects/jet/src/evidence/bundle.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/evidence/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/evidence/writer.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-evidence.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
