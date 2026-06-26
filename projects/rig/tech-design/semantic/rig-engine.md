---
id: semantic-rig-engine
summary: Semantic coverage for "projects/rig/src/engine"
capability_refs:
  - id: "scenario-engine"
    role: primary
    claim: "record-contract-check-and-json-report"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/rig/src/engine`."
fill_sections: [schema, changes]
---

# Semantic TD: rig/engine

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "rig/engine"
  source_group: "projects/rig/src/engine"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/rig/src/engine/timeout.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "WaitOutcome"
            kind: "enum"
            public: true
          - name: "TimeoutPolicy"
            kind: "struct"
            public: true
          - name: "from_env"
            kind: "function"
            public: true
          - name: "fixed"
            kind: "function"
            public: true
          - name: "with_poll_interval"
            kind: "function"
            public: true
          - name: "timeout"
            kind: "function"
            public: true
          - name: "wait_with_timeout"
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
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/assert.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "evaluate"
            kind: "function"
            public: true
          - name: "operand"
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
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/exec.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ExecOutcome"
            kind: "struct"
            public: true
          - name: "execute"
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
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/transport.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "HttpTransport"
            kind: "struct"
            public: true
          - name: "connect"
            kind: "function"
            public: false
          - name: "HttpWorker"
            kind: "struct"
            public: false
          - name: "execute"
            kind: "function"
            public: false
          - name: "PostgresTransport"
            kind: "struct"
            public: true
          - name: "connect"
            kind: "function"
            public: false
          - name: "PgWorker"
            kind: "struct"
            public: false
          - name: "execute"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/rss.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "execute"
            kind: "function"
            public: true
          - name: "resolve_pid"
            kind: "function"
            public: false
          - name: "rss_kb_of"
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
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "assert"
            kind: "module"
            public: true
          - name: "case"
            kind: "module"
            public: true
          - name: "exec"
            kind: "module"
            public: true
          - name: "http"
            kind: "module"
            public: true
          - name: "loadgen"
            kind: "module"
            public: true
          - name: "rss"
            kind: "module"
            public: true
          - name: "sample"
            kind: "module"
            public: true
          - name: "timeout"
            kind: "module"
            public: true
          - name: "transport"
            kind: "module"
            public: true
          - name: "ScenarioRun"
            kind: "struct"
            public: true
          - name: "PhaseRun"
            kind: "struct"
            public: true
          - name: "run_phase"
            kind: "function"
            public: true
          - name: "run_scenario"
            kind: "function"
            public: true
          - name: "StepResult"
            kind: "enum"
            public: false
          - name: "run_step"
            kind: "function"
            public: false
          - name: "subject_name"
            kind: "function"
            public: false
          - name: "expr_snapshot"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/http.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "HttpOutcome"
            kind: "struct"
            public: true
          - name: "execute"
            kind: "function"
            public: true
          - name: "capture_value"
            kind: "function"
            public: true
          - name: "json_path"
            kind: "function"
            public: true
          - name: "check_predicate"
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
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/case.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "Mode"
            kind: "enum"
            public: true
          - name: "CaseResult"
            kind: "enum"
            public: true
          - name: "run_case"
            kind: "function"
            public: true
          - name: "exercise_http_step"
            kind: "function"
            public: false
          - name: "build_load_transport"
            kind: "function"
            public: false
          - name: "run_query_once"
            kind: "function"
            public: false
          - name: "run_query_once"
            kind: "function"
            public: false
          - name: "query_finding"
            kind: "function"
            public: false
          - name: "run_clean"
            kind: "function"
            public: false
          - name: "finish_prepare_failure"
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
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/loadgen.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "LoadStats"
            kind: "struct"
            public: true
          - name: "get"
            kind: "function"
            public: true
          - name: "Schedule"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "run_transport"
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
          domain: "projects/rig/src/engine"
      - path: "projects/rig/src/engine/sample.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "SampleStats"
            kind: "struct"
            public: true
          - name: "fold"
            kind: "function"
            public: true
          - name: "get"
            kind: "function"
            public: true
          - name: "percentile"
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
          domain: "projects/rig/src/engine"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/rig/src/engine/timeout.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/assert.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/exec.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/transport.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/rss.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/http.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/case.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/loadgen.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/engine/sample.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
