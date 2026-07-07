---
id: semantic-rig-scenario
summary: Semantic coverage for "projects/rig/src/scenario"
capability_refs:
  - id: "scenario-engine"
    role: primary
    claim: "record-contract-check-and-json-report"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/rig/src/scenario`."
fill_sections: [schema, changes]
---

# Semantic TD: rig/scenario

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "rig/scenario"
  source_group: "projects/rig/src/scenario"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/rig/src/scenario/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "case"
            kind: "module"
            public: true
          - name: "interp"
            kind: "module"
            public: true
          - name: "load"
            kind: "module"
            public: true
          - name: "record"
            kind: "module"
            public: true
          - name: "step"
            kind: "module"
            public: true
          - name: "VatNeeds"
            kind: "struct"
            public: true
          - name: "Limits"
            kind: "struct"
            public: true
          - name: "default_timeout_secs"
            kind: "function"
            public: false
          - name: "default"
            kind: "function"
            public: false
          - name: "Scenario"
            kind: "struct"
            public: true
          - name: "parse_scenario"
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
          domain: "projects/rig/src/scenario"
      - path: "projects/rig/src/scenario/interp.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "VarStore"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "seed"
            kind: "function"
            public: true
          - name: "set"
            kind: "function"
            public: true
          - name: "get"
            kind: "function"
            public: true
          - name: "get_f64"
            kind: "function"
            public: true
          - name: "interpolate"
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
          domain: "projects/rig/src/scenario"
      - path: "projects/rig/src/scenario/load.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model"]
        symbols:
          - name: "ACHIEVED_QPS_HONESTY_RATIO"
            kind: "constant"
            public: true
          - name: "LoadProfile"
            kind: "struct"
            public: true
          - name: "LOAD_METRICS"
            kind: "constant"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/rig/src/scenario"
      - path: "projects/rig/src/scenario/case.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "default_required"
            kind: "function"
            public: false
          - name: "default_n"
            kind: "function"
            public: false
          - name: "default_metric"
            kind: "function"
            public: false
          - name: "CaseRecord"
            kind: "struct"
            public: true
          - name: "Prepare"
            kind: "struct"
            public: true
          - name: "Exercise"
            kind: "struct"
            public: true
          - name: "QuerySpec"
            kind: "struct"
            public: true
          - name: "LoadSpec"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "Clean"
            kind: "struct"
            public: true
          - name: "TestCase"
            kind: "struct"
            public: true
          - name: "case_id"
            kind: "function"
            public: true
          - name: "is_load"
            kind: "function"
            public: true
          - name: "parse_case"
            kind: "function"
            public: true
          - name: "VALID_DIMENSIONS"
            kind: "constant"
            public: false
          - name: "is_compare_predicate"
            kind: "function"
            public: false
          - name: "lint_case"
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
          domain: "projects/rig/src/scenario"
      - path: "projects/rig/src/scenario/record.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ScenarioKind"
            kind: "enum"
            public: true
          - name: "ExpectedOutcome"
            kind: "enum"
            public: true
          - name: "Record"
            kind: "struct"
            public: true
          - name: "default_required"
            kind: "function"
            public: false
          - name: "LintViolation"
            kind: "struct"
            public: true
          - name: "lint_record"
            kind: "function"
            public: true
          - name: "scenario_id"
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
          domain: "projects/rig/src/scenario"
      - path: "projects/rig/src/scenario/step.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "HttpExpect"
            kind: "struct"
            public: true
          - name: "status_ok"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "default_status"
            kind: "function"
            public: false
          - name: "default_timeout_ms"
            kind: "function"
            public: false
          - name: "HttpRequest"
            kind: "struct"
            public: true
          - name: "HttpStep"
            kind: "struct"
            public: true
          - name: "SampleStep"
            kind: "struct"
            public: true
          - name: "AssertStep"
            kind: "struct"
            public: true
          - name: "WaitUntilStep"
            kind: "struct"
            public: true
          - name: "default_interval_ms"
            kind: "function"
            public: false
          - name: "MeasureRssStep"
            kind: "struct"
            public: true
          - name: "ExecStep"
            kind: "struct"
            public: true
          - name: "default_exec_timeout"
            kind: "function"
            public: false
          - name: "Step"
            kind: "enum"
            public: true
          - name: "name"
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
          domain: "projects/rig/src/scenario"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/rig/src/scenario/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/scenario/interp.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/scenario/load.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/scenario/case.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/scenario/record.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/rig/src/scenario/step.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
