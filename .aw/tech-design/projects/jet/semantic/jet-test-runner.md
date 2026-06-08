---
id: semantic-jet-test-runner
summary: Semantic coverage for "projects/jet/src/test_runner"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/test_runner

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/test_runner"
  source_group: "projects/jet/src/test_runner"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/test_runner/coverage.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "CoverageMetric"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "pct"
            kind: "function"
            public: true
          - name: "CoverageSummary"
            kind: "struct"
            public: true
          - name: "evaluate"
            kind: "function"
            public: true
          - name: "check"
            kind: "function"
            public: false
          - name: "CoverageThresholds"
            kind: "struct"
            public: true
          - name: "CoverageMetricKind"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "CoverageThresholdFailure"
            kind: "struct"
            public: true
          - name: "display_line"
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
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/list_manifest.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "LIST_MANIFEST_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ListedTest"
            kind: "struct"
            public: true
          - name: "TestListManifest"
            kind: "struct"
            public: true
          - name: "from_discovery"
            kind: "function"
            public: true
          - name: "from_config"
            kind: "function"
            public: true
          - name: "to_json"
            kind: "function"
            public: true
          - name: "listed_test"
            kind: "function"
            public: false
          - name: "format_list_manifest_non_utf8_warn"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3755_non_utf8_id_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/web_server.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "WebServerHandle"
            kind: "struct"
            public: true
          - name: "reused"
            kind: "function"
            public: false
          - name: "drop"
            kind: "function"
            public: false
          - name: "boot"
            kind: "function"
            public: true
          - name: "probe_ready"
            kind: "function"
            public: false
          - name: "probe_tcp"
            kind: "function"
            public: false
          - name: "probe_http"
            kind: "function"
            public: false
          - name: "parse_http_url"
            kind: "function"
            public: false
          - name: "short_label"
            kind: "function"
            public: false
          - name: "spawn_log_pump"
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
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/reporter.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "Outcome"
            kind: "enum"
            public: true
          - name: "SourceLocation"
            kind: "struct"
            public: true
          - name: "TestError"
            kind: "struct"
            public: true
          - name: "parse_from_stack"
            kind: "function"
            public: true
          - name: "parse_path_line_col"
            kind: "function"
            public: false
          - name: "TestReport"
            kind: "struct"
            public: true
          - name: "Summary"
            kind: "struct"
            public: true
          - name: "source_location_tests"
            kind: "module"
            public: false
          - name: "default"
            kind: "function"
            public: false
          - name: "rerun_hint"
            kind: "function"
            public: true
          - name: "MultiReporter"
            kind: "struct"
            public: true
          - name: "ReporterState"
            kind: "struct"
            public: false
          - name: "from_config"
            kind: "function"
            public: true
          - name: "on_start"
            kind: "function"
            public: true
          - name: "on_event"
            kind: "function"
            public: true
          - name: "on_finish"
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
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/config.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "RunnerConfig"
            kind: "struct"
            public: true
          - name: "LiveE2eConfig"
            kind: "struct"
            public: true
          - name: "TestEnvironment"
            kind: "enum"
            public: true
          - name: "parse"
            kind: "function"
            public: true
          - name: "ensure_supported"
            kind: "function"
            public: true
          - name: "Reporter"
            kind: "enum"
            public: true
          - name: "parse_list"
            kind: "function"
            public: true
          - name: "default_for_root"
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
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/worker_pool.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "safe_shard_key"
            kind: "function"
            public: true
          - name: "format_safe_shard_key_err"
            kind: "function"
            public: true
          - name: "format_safe_shard_key_non_utf8_err"
            kind: "function"
            public: true
          - name: "ShardSpec"
            kind: "type"
            public: true
          - name: "parse_shard"
            kind: "function"
            public: true
          - name: "partition_shard"
            kind: "function"
            public: true
          - name: "path_hash_u64"
            kind: "function"
            public: true
          - name: "WorkerPool"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "run_serial"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3616_safe_shard_key_tests"
            kind: "module"
            public: false
          - name: "gh3757_safe_shard_key_non_utf8_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/discovery.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "safe_relative_path"
            kind: "function"
            public: true
          - name: "format_safe_relative_path_warn"
            kind: "function"
            public: true
          - name: "RESOLVED_DISCOVERY_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "DiscoveryConfigError"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: false
          - name: "to_json"
            kind: "function"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "ResolvedSpec"
            kind: "struct"
            public: true
          - name: "ResolvedDiscovery"
            kind: "struct"
            public: true
          - name: "environment_str"
            kind: "function"
            public: false
          - name: "resolve_discovery"
            kind: "function"
            public: true
          - name: "SpecFile"
            kind: "struct"
            public: true
          - name: "scan"
            kind: "function"
            public: true
          - name: "pick_focused_specs"
            kind: "function"
            public: true
          - name: "build_globset"
            kind: "function"
            public: false
          - name: "is_hidden"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3623_safe_relative_path_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/worker.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method", "test_case"]
        symbols:
          - name: "WORKER_RUNTIME"
            kind: "constant"
            public: false
          - name: "PAGE_SHIM"
            kind: "constant"
            public: false
          - name: "MATCHERS_SHIM"
            kind: "constant"
            public: false
          - name: "run_spec"
            kind: "function"
            public: true
          - name: "LiveCheckpointState"
            kind: "struct"
            public: false
          - name: "LivePageStep"
            kind: "struct"
            public: false
          - name: "title"
            kind: "function"
            public: false
          - name: "append_worker_live_event"
            kind: "function"
            public: false
          - name: "append_page_live_event"
            kind: "function"
            public: false
          - name: "live_step_from_page_request"
            kind: "function"
            public: false
          - name: "page_request_kind"
            kind: "function"
            public: false
          - name: "highlight_selector_for_live_e2e"
            kind: "function"
            public: false
          - name: "wait_for_live_checkpoint"
            kind: "function"
            public: false
          - name: "LiveControlFile"
            kind: "struct"
            public: false
          - name: "read_live_control"
            kind: "function"
            public: false
          - name: "append_jsonl"
            kind: "function"
            public: false
          - name: "test_title_for_live"
            kind: "function"
            public: false
          - name: "now_ms"
            kind: "function"
            public: false
          - name: "format_safe_worker_now_ms_warn"
            kind: "function"
            public: true
          - name: "safe_worker_now_ms"
            kind: "function"
            public: true
          - name: "page_req_id_str"
            kind: "function"
            public: false
          - name: "ensure_browser"
            kind: "function"
            public: false
          - name: "handle_expect_request"
            kind: "function"
            public: true
          - name: "load_or_write_snapshot"
            kind: "function"
            public: true
          - name: "load_or_write_text_snapshot"
            kind: "function"
            public: true
          - name: "spec_slug_for"
            kind: "function"
            public: true
          - name: "wire_trace_mode_to_buffer_mode"
            kind: "function"
            public: true
          - name: "test_outcome_to_trace_outcome"
            kind: "function"
            public: true
          - name: "commit_trace_for_report"
            kind: "function"
            public: true
          - name: "test_id_slug"
            kind: "function"
            public: false
          - name: "write_response"
            kind: "function"
            public: true
          - name: "format_worker_stdin_write_warn"
            kind: "function"
            public: true
          - name: "transform_spec"
            kind: "function"
            public: false
          - name: "build_boot"
            kind: "function"
            public: false
          - name: "path_to_file_url"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3685_safe_worker_now_ms_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/expect.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "CORE_MATCHERS"
            kind: "constant"
            public: true
          - name: "format_diff"
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
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "config"
            kind: "module"
            public: true
          - name: "coverage"
            kind: "module"
            public: true
          - name: "discovery"
            kind: "module"
            public: true
          - name: "expect"
            kind: "module"
            public: true
          - name: "list_manifest"
            kind: "module"
            public: true
          - name: "reporter"
            kind: "module"
            public: true
          - name: "web_server"
            kind: "module"
            public: true
          - name: "wire"
            kind: "module"
            public: true
          - name: "worker"
            kind: "module"
            public: true
          - name: "worker_pool"
            kind: "module"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "run_at"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/test_runner"
      - path: "projects/jet/src/test_runner/wire.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "WireTraceMode"
            kind: "enum"
            public: true
          - name: "from_str"
            kind: "function"
            public: true
          - name: "is_active"
            kind: "function"
            public: true
          - name: "WireTraceEvent"
            kind: "enum"
            public: true
          - name: "WireRequest"
            kind: "enum"
            public: true
          - name: "WireResponse"
            kind: "enum"
            public: true
          - name: "MatcherDiff"
            kind: "struct"
            public: true
          - name: "parse_request"
            kind: "function"
            public: true
          - name: "WorkerEvent"
            kind: "enum"
            public: true
          - name: "TestDescriptor"
            kind: "struct"
            public: true
          - name: "TestOutcome"
            kind: "enum"
            public: true
          - name: "ConsoleStream"
            kind: "enum"
            public: true
          - name: "TestError"
            kind: "struct"
            public: true
          - name: "parse_line"
            kind: "function"
            public: true
          - name: "is_unknown_variant_error"
            kind: "function"
            public: true
          - name: "value_has_kind"
            kind: "function"
            public: false
          - name: "format_worker_event_shape_warn"
            kind: "function"
            public: true
          - name: "format_wire_request_shape_warn"
            kind: "function"
            public: true
          - name: "truncate_preview"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3759_wire_shape_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/test_runner"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests:
      - path: "projects/jet/src/test_runner/coverage.rs"
      - path: "projects/jet/src/test_runner/list_manifest.rs"
      - path: "projects/jet/src/test_runner/web_server.rs"
      - path: "projects/jet/src/test_runner/reporter.rs"
      - path: "projects/jet/src/test_runner/config.rs"
      - path: "projects/jet/src/test_runner/worker_pool.rs"
      - path: "projects/jet/src/test_runner/discovery.rs"
      - path: "projects/jet/src/test_runner/worker.rs"
      - path: "projects/jet/src/test_runner/expect.rs"
      - path: "projects/jet/src/test_runner/mod.rs"
      - path: "projects/jet/src/test_runner/wire.rs"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/test_runner/coverage.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/list_manifest.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/web_server.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/reporter.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/config.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/worker_pool.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/discovery.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/worker.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/expect.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/test_runner/wire.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-test-runner.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
