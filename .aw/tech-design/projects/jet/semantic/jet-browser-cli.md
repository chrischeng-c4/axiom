---
id: semantic-jet-browser-cli
summary: Semantic coverage for "projects/jet/src/browser_cli"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/browser_cli

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/browser_cli"
  source_group: "projects/jet/src/browser_cli"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/browser_cli/session.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "Session"
            kind: "struct"
            public: true
          - name: "session_path"
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
          - name: "now_unix"
            kind: "function"
            public: true
          - name: "safe_session_now_unix"
            kind: "function"
            public: true
          - name: "format_safe_session_now_unix_warn"
            kind: "function"
            public: true
          - name: "read_live"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3677_safe_session_now_unix_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser_cli"
      - path: "projects/jet/src/browser_cli/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "pretty"
            kind: "module"
            public: true
          - name: "session"
            kind: "module"
            public: true
          - name: "safe_chrome_path_override"
            kind: "function"
            public: true
          - name: "format_safe_chrome_path_warn"
            kind: "function"
            public: true
          - name: "attach"
            kind: "function"
            public: true
          - name: "assert_debug_bridge"
            kind: "function"
            public: false
          - name: "expr"
            kind: "function"
            public: false
          - name: "prepare_session"
            kind: "function"
            public: true
          - name: "prepare_session_with_init_scripts"
            kind: "function"
            public: true
          - name: "launch"
            kind: "function"
            public: true
          - name: "format_browser_cli_ctrl_c_warn"
            kind: "function"
            public: true
          - name: "tree"
            kind: "function"
            public: true
          - name: "hooks"
            kind: "function"
            public: true
          - name: "pick"
            kind: "function"
            public: true
          - name: "highlight"
            kind: "function"
            public: true
          - name: "frame"
            kind: "function"
            public: true
          - name: "screenshot"
            kind: "function"
            public: true
          - name: "eval"
            kind: "function"
            public: true
          - name: "tsx"
            kind: "function"
            public: true
          - name: "gh3612_safe_chrome_path_tests"
            kind: "module"
            public: false
          - name: "gh3732_browser_cli_ctrl_c_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser_cli"
      - path: "projects/jet/src/browser_cli/pretty.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "element_tree"
            kind: "function"
            public: true
          - name: "render_element"
            kind: "function"
            public: false
          - name: "layout_tree"
            kind: "function"
            public: true
          - name: "fiber_tree"
            kind: "function"
            public: true
          - name: "hook_values"
            kind: "function"
            public: true
          - name: "paint_ops"
            kind: "function"
            public: true
          - name: "render_rect"
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
          domain: "projects/jet/src/browser_cli"
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
  - path: "projects/jet/src/browser_cli/session.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser_cli/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser_cli/pretty.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-browser-cli.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
