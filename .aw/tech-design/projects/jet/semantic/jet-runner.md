---
id: semantic-jet-runner
summary: Semantic coverage for "projects/jet/src/runner"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/runner

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/runner"
  source_group: "projects/jet/src/runner"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/runner/source_map.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "append_inline_source_map"
            kind: "function"
            public: true
          - name: "generate_identity_source_map"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/runner"
      - path: "projects/jet/src/runner/env.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "build_env"
            kind: "function"
            public: true
          - name: "safe_prepend_bin_to_path"
            kind: "function"
            public: true
          - name: "format_safe_prepend_bin_to_path_warn"
            kind: "function"
            public: true
          - name: "safe_node_env"
            kind: "function"
            public: true
          - name: "format_safe_node_env_warn"
            kind: "function"
            public: true
          - name: "scan_env_files"
            kind: "function"
            public: true
          - name: "import_meta_env_defines"
            kind: "function"
            public: true
          - name: "parse_env_file"
            kind: "function"
            public: false
          - name: "EnvLineSkipReason"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "format_env_line_skipped_warn"
            kind: "function"
            public: true
          - name: "format_env_define_warn"
            kind: "function"
            public: true
          - name: "unquote"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3717_env_line_skipped_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/runner"
      - path: "projects/jet/src/runner/jit.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "format_jit_watch_no_filename_err"
            kind: "function"
            public: true
          - name: "format_jit_stem_err"
            kind: "function"
            public: true
          - name: "safe_jit_stem"
            kind: "function"
            public: true
          - name: "JitEngine"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "execute"
            kind: "function"
            public: true
          - name: "execute_watch"
            kind: "function"
            public: true
          - name: "transform_file"
            kind: "function"
            public: false
          - name: "source_map"
            kind: "module"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3600_watch_filename_tests"
            kind: "module"
            public: false
          - name: "gh3614_safe_jit_stem_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/runner"
      - path: "projects/jet/src/runner/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "env"
            kind: "module"
            public: true
          - name: "jit"
            kind: "module"
            public: true
          - name: "source_map"
            kind: "module"
            public: true
          - name: "watcher"
            kind: "module"
            public: true
          - name: "ScriptRunner"
            kind: "struct"
            public: true
          - name: "PkgScripts"
            kind: "struct"
            public: false
          - name: "RunResult"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "load_pkg_scripts"
            kind: "function"
            public: false
          - name: "run_script"
            kind: "function"
            public: true
          - name: "run_file"
            kind: "function"
            public: true
          - name: "exec_command"
            kind: "function"
            public: true
          - name: "resolve_bin_path"
            kind: "function"
            public: false
          - name: "exec_shell"
            kind: "function"
            public: false
          - name: "exec_node"
            kind: "function"
            public: false
          - name: "has_script"
            kind: "function"
            public: true
          - name: "is_file"
            kind: "function"
            public: true
          - name: "RUN_FILE_NO_EXTENSION_FALLBACK"
            kind: "constant"
            public: true
          - name: "format_runner_run_file_no_extension_warn"
            kind: "function"
            public: true
          - name: "format_runner_run_file_non_utf8_extension_warn"
            kind: "function"
            public: true
          - name: "coerce_run_file_extension_or_warn"
            kind: "function"
            public: true
          - name: "exit_code"
            kind: "function"
            public: false
          - name: "safe_runner_exit_code"
            kind: "function"
            public: true
          - name: "format_safe_runner_exit_code_warn"
            kind: "function"
            public: true
          - name: "format_lifecycle_short_circuit_info"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3691_safe_runner_exit_code_tests"
            kind: "module"
            public: false
          - name: "gh3723_lifecycle_short_circuit_tests"
            kind: "module"
            public: false
          - name: "gh3723_run_script_lifecycle_tests"
            kind: "module"
            public: false
          - name: "gh3801_run_file_extension_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/runner"
      - path: "projects/jet/src/runner/watcher.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "DebouncedWatcher"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "wait_for_change"
            kind: "function"
            public: true
          - name: "is_source_file"
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
          domain: "projects/jet/src/runner"
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
  - path: "projects/jet/src/runner/source_map.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/runner/env.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/runner/jit.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/runner/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/runner/watcher.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-runner.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
