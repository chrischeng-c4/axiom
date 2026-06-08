---
id: semantic-jet-reporter
summary: Semantic coverage for "projects/jet/src/reporter"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/reporter

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/reporter"
  source_group: "projects/jet/src/reporter"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/reporter/html.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "relativize_report_file"
            kind: "function"
            public: true
          - name: "format_relativize_outside_cwd_warn"
            kind: "function"
            public: true
          - name: "format_relativize_cwd_err_warn"
            kind: "function"
            public: true
          - name: "lossy_with_non_utf8_warn"
            kind: "function"
            public: true
          - name: "format_html_report_non_utf8_warn"
            kind: "function"
            public: true
          - name: "HTML_TEMPLATE"
            kind: "constant"
            public: false
          - name: "REPORT_JS"
            kind: "constant"
            public: true
          - name: "REPORT_CSS"
            kind: "constant"
            public: true
          - name: "TestRow"
            kind: "struct"
            public: true
          - name: "HtmlReporter"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "emit"
            kind: "function"
            public: true
          - name: "finalize"
            kind: "function"
            public: true
          - name: "into_rows"
            kind: "function"
            public: true
          - name: "render_html"
            kind: "function"
            public: true
          - name: "build_stats_html"
            kind: "function"
            public: false
          - name: "build_shard_html"
            kind: "function"
            public: false
          - name: "build_report_data_json"
            kind: "function"
            public: false
          - name: "build_rows_html"
            kind: "function"
            public: false
          - name: "build_single_row"
            kind: "function"
            public: false
          - name: "outcome_to_status"
            kind: "function"
            public: false
          - name: "stable_id"
            kind: "function"
            public: false
          - name: "esc_html"
            kind: "function"
            public: false
          - name: "esc_attr"
            kind: "function"
            public: false
          - name: "json_string"
            kind: "function"
            public: false
          - name: "render_from_rows"
            kind: "function"
            public: true
          - name: "read_rows_from_dir"
            kind: "function"
            public: true
          - name: "finalize_with_sidecar"
            kind: "function"
            public: true
          - name: "row_to_ndjson_line"
            kind: "function"
            public: true
          - name: "trace_button_tests"
            kind: "module"
            public: false
          - name: "gh3602_relativize_tests"
            kind: "module"
            public: false
          - name: "gh3776_non_utf8_path_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/reporter"
      - path: "projects/jet/src/reporter/merge.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "merge_reports"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/reporter"
      - path: "projects/jet/src/reporter/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "html"
            kind: "module"
            public: true
          - name: "merge"
            kind: "module"
            public: true
          - name: "parser"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/reporter"
      - path: "projects/jet/src/reporter/parser.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "parse_ndjson"
            kind: "function"
            public: true
          - name: "parse_ndjson_to_rows"
            kind: "function"
            public: true
          - name: "json_value_kind"
            kind: "function"
            public: false
          - name: "format_reporter_parser_string_field_warn"
            kind: "function"
            public: true
          - name: "format_reporter_parser_u64_field_warn"
            kind: "function"
            public: true
          - name: "format_reporter_parser_optional_string_field_warn"
            kind: "function"
            public: true
          - name: "string_field_or_warn"
            kind: "function"
            public: false
          - name: "u64_field_or_warn"
            kind: "function"
            public: false
          - name: "optional_string_field_or_warn"
            kind: "function"
            public: false
          - name: "gh3314_tests"
            kind: "module"
            public: false
          - name: "gh3763_wrong_shape_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/reporter"
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
  - path: "projects/jet/src/reporter/html.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/reporter/merge.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/reporter/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/reporter/parser.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-reporter.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
