---
id: semantic-jet-parity-oracle-src
summary: Semantic coverage for "projects/jet/parity/oracle/src"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/oracle/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/parity/oracle/src"
  source_group: "projects/jet/parity/oracle/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/oracle/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "artifacts"
            kind: "module"
            public: true
          - name: "channels"
            kind: "module"
            public: true
          - name: "manifest"
            kind: "module"
            public: true
          - name: "runner"
            kind: "module"
            public: true
          - name: "run_fixture"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/oracle/src"
      - path: "projects/jet/parity/oracle/src/manifest.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "FixtureManifest"
            kind: "struct"
            public: true
          - name: "default_tab_count"
            kind: "function"
            public: false
          - name: "ManifestError"
            kind: "enum"
            public: true
          - name: "from_file"
            kind: "function"
            public: true
          - name: "from_source"
            kind: "function"
            public: true
          - name: "validate"
            kind: "function"
            public: true
          - name: "is_kebab_case"
            kind: "function"
            public: false
          - name: "extract_frontmatter"
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
          domain: "projects/jet/parity/oracle/src"
      - path: "projects/jet/parity/oracle/src/runner.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "RunnerConfig"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "BrowserKind"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "MatrixEntry"
            kind: "struct"
            public: true
          - name: "dpr_label"
            kind: "function"
            public: true
          - name: "RunnerError"
            kind: "enum"
            public: true
          - name: "PageHost"
            kind: "struct"
            public: true
          - name: "PlaywrightBrowserSession"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "browser_kind"
            kind: "function"
            public: false
          - name: "page"
            kind: "function"
            public: false
          - name: "page_mut"
            kind: "function"
            public: false
          - name: "launch"
            kind: "function"
            public: false
          - name: "navigate"
            kind: "function"
            public: false
          - name: "await_mount"
            kind: "function"
            public: false
          - name: "close"
            kind: "function"
            public: false
          - name: "screenshot"
            kind: "function"
            public: false
          - name: "ax_full_tree"
            kind: "function"
            public: false
          - name: "capture_focus_trace"
            kind: "function"
            public: false
          - name: "capture_pointer_hits"
            kind: "function"
            public: false
          - name: "capture_ime_trace"
            kind: "function"
            public: false
          - name: "StubBrowserSession"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "browser_kind"
            kind: "function"
            public: false
          - name: "page"
            kind: "function"
            public: false
          - name: "page_mut"
            kind: "function"
            public: false
          - name: "launch"
            kind: "function"
            public: false
          - name: "navigate"
            kind: "function"
            public: false
          - name: "await_mount"
            kind: "function"
            public: false
          - name: "close"
            kind: "function"
            public: false
          - name: "screenshot"
            kind: "function"
            public: false
          - name: "ax_full_tree"
            kind: "function"
            public: false
          - name: "capture_focus_trace"
            kind: "function"
            public: false
          - name: "capture_pointer_hits"
            kind: "function"
            public: false
          - name: "capture_ime_trace"
            kind: "function"
            public: false
          - name: "Runner"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/parity/oracle/src"
      - path: "projects/jet/parity/oracle/src/artifacts.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ArtifactBundle"
            kind: "struct"
            public: true
          - name: "ArtifactError"
            kind: "enum"
            public: true
          - name: "ArtifactWriter"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "root_dir"
            kind: "function"
            public: true
          - name: "write_bytes"
            kind: "function"
            public: true
          - name: "write_json"
            kind: "function"
            public: true
          - name: "write_png"
            kind: "function"
            public: true
          - name: "sha256s"
            kind: "function"
            public: true
          - name: "into_sha256s"
            kind: "function"
            public: true
          - name: "to_deterministic_json"
            kind: "function"
            public: true
          - name: "canonicalize"
            kind: "function"
            public: false
          - name: "write_canonical"
            kind: "function"
            public: false
          - name: "reencode_png_stripped"
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
          domain: "projects/jet/parity/oracle/src"
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
  - path: "projects/jet/parity/oracle/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/oracle/src/manifest.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/oracle/src/runner.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/parity/oracle/src/artifacts.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
