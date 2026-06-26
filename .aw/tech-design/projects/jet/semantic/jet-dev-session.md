---
id: semantic-jet-dev-session
summary: Semantic coverage for "projects/jet/src/dev_session.rs"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: persistent dev-session state supports the aggregate production replacement capability by letting browser-backed DOM/WASM lifecycle tests shut down Jet dev servers through the Jet CLI instead of process killing."
---

# Semantic TD: jet/dev_session

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/dev_session"
  source_group: "projects/jet/src/dev_session.rs"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/dev_session.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "DevSessionMode"
            kind: "enum"
            public: true
          - name: "DevSession"
            kind: "struct"
            public: true
          - name: "session_path"
            kind: "function"
            public: true
          - name: "shutdown_request_path"
            kind: "function"
            public: true
          - name: "write"
            kind: "function"
            public: true
          - name: "read"
            kind: "function"
            public: true
          - name: "clear"
            kind: "function"
            public: true
          - name: "request_shutdown"
            kind: "function"
            public: true
          - name: "shutdown_requested"
            kind: "function"
            public: true
          - name: "clear_shutdown_request"
            kind: "function"
            public: true
          - name: "now_unix"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/dev_session.rs"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: verify lifecycle state through the CLI matrix test that drives DOM and WASM dev shutdown
  evidence:
    source_tests:
      - path: "projects/jet/tests/install_dev_build_browser_lifecycle.rs"
        command: "cargo test -p jet --test install_dev_build_browser_lifecycle"
        covers:
          - "dev session file is written after DOM and WASM dev servers bind"
          - "jet dev shutdown writes a request consumed by the running server"
          - "session and shutdown-request files are cleared after graceful shutdown"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/dev_session.rs"
    action: add
    section: schema
    impl_mode: hand-written
    description: |
      Persistent dev-server lifecycle session state and shutdown request helpers.
  - path: "projects/jet/tests/install_dev_build_browser_lifecycle.rs"
    action: add
    section: e2e-test
    impl_mode: hand-written
    description: |
      CLI lifecycle matrix evidence for install, build, dev, browser, screenshot, and graceful shutdown flows.
```
