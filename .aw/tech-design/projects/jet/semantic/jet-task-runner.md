---
id: semantic-jet-task-runner
summary: Semantic coverage for "projects/jet/src/task_runner"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/task_runner

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/task_runner"
  source_group: "projects/jet/src/task_runner"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/task_runner/cache.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "TaskCacheEntry"
            kind: "struct"
            public: true
          - name: "TaskCache"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "compute_hash"
            kind: "function"
            public: true
          - name: "lookup"
            kind: "function"
            public: true
          - name: "store"
            kind: "function"
            public: true
          - name: "copy_outputs_to_cache"
            kind: "function"
            public: false
          - name: "restore_outputs"
            kind: "function"
            public: true
          - name: "collect_output_files"
            kind: "function"
            public: false
          - name: "format_task_cache_non_utf8_err"
            kind: "function"
            public: true
          - name: "format_output_escape_err"
            kind: "function"
            public: true
          - name: "format_restore_prefix_err"
            kind: "function"
            public: true
          - name: "chrono_now"
            kind: "function"
            public: false
          - name: "safe_cache_now"
            kind: "function"
            public: true
          - name: "format_safe_cache_now_warn"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3644_safe_cache_now_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/task_runner"
      - path: "projects/jet/src/task_runner/graph.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "TaskGraph"
            kind: "struct"
            public: true
          - name: "from_config"
            kind: "function"
            public: true
          - name: "execution_order"
            kind: "function"
            public: true
          - name: "detect_cycles"
            kind: "function"
            public: false
          - name: "dfs_cycle"
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
          domain: "projects/jet/src/task_runner"
      - path: "projects/jet/src/task_runner/config.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "JetConfig"
            kind: "struct"
            public: true
          - name: "TestConfig"
            kind: "struct"
            public: true
          - name: "WebServerConfig"
            kind: "struct"
            public: true
          - name: "default_web_server_timeout"
            kind: "function"
            public: false
          - name: "ResolveConfig"
            kind: "struct"
            public: true
          - name: "DevConfig"
            kind: "struct"
            public: true
          - name: "JetBuildConfig"
            kind: "struct"
            public: true
          - name: "TaskDef"
            kind: "struct"
            public: true
          - name: "default_true"
            kind: "function"
            public: false
          - name: "JET_TOP_LEVEL_KEYS"
            kind: "constant"
            public: false
          - name: "resolve_conditions"
            kind: "function"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "classify_jet_toml_error"
            kind: "function"
            public: false
          - name: "extract_unknown_field"
            kind: "function"
            public: false
          - name: "line_for_byte_offset"
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
          domain: "projects/jet/src/task_runner"
      - path: "projects/jet/src/task_runner/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "cache"
            kind: "module"
            public: true
          - name: "config"
            kind: "module"
            public: true
          - name: "graph"
            kind: "module"
            public: true
          - name: "hash"
            kind: "module"
            public: true
          - name: "TaskResult"
            kind: "struct"
            public: true
          - name: "TaskStatus"
            kind: "enum"
            public: true
          - name: "TaskRunner"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "has_task"
            kind: "function"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "execute_task"
            kind: "function"
            public: false
          - name: "print_summary"
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
          domain: "projects/jet/src/task_runner"
      - path: "projects/jet/src/task_runner/hash.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "EnvHashTriple"
            kind: "struct"
            public: false
          - name: "format_env_var_lookup_warn"
            kind: "function"
            public: true
          - name: "os_string_bytes"
            kind: "function"
            public: false
          - name: "os_string_bytes_non_unix_lossy_or_warn"
            kind: "function"
            public: true
          - name: "format_env_var_lookup_non_unix_lossy_warn"
            kind: "function"
            public: true
          - name: "compute_task_hash"
            kind: "function"
            public: true
          - name: "collect_input_files"
            kind: "function"
            public: false
          - name: "format_task_hash_non_utf8_warn"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3753_non_utf8_path_warn_tests"
            kind: "module"
            public: false
          - name: "gh3807_non_unix_lossy_env_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/task_runner"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

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
  - path: "projects/jet/src/task_runner/cache.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/task_runner/graph.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/task_runner/config.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/task_runner/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/task_runner/hash.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-task-runner.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
