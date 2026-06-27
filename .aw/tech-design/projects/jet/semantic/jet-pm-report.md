---
id: semantic-jet-pm-report
summary: Semantic coverage for "projects/jet/src/pm_report"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/pm_report

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/pm_report"
  source_group: "projects/jet/src/pm_report"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/pm_report/metadata.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "PM_REPORT_METADATA_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "UNAVAILABLE_LABEL"
            kind: "constant"
            public: true
          - name: "PmReportMetadata"
            kind: "struct"
            public: true
          - name: "EnvDisplayPair"
            kind: "struct"
            public: true
          - name: "AdapterContext"
            kind: "struct"
            public: true
          - name: "from_manifest"
            kind: "function"
            public: true
          - name: "present_or_unavailable"
            kind: "function"
            public: false
          - name: "command_label"
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
          domain: "projects/jet/src/pm_report"
      - path: "projects/jet/src/pm_report/redaction.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "PM_REDACTION_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "REDACTION_PLACEHOLDER"
            kind: "constant"
            public: true
          - name: "RedactionScope"
            kind: "enum"
            public: true
          - name: "RedactionPolicy"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "shareable_baseline"
            kind: "function"
            public: true
          - name: "passthrough"
            kind: "function"
            public: true
          - name: "redact_header"
            kind: "function"
            public: true
          - name: "redact_url"
            kind: "function"
            public: true
          - name: "redact_env"
            kind: "function"
            public: true
          - name: "should_redact_body_key"
            kind: "function"
            public: true
          - name: "apply"
            kind: "function"
            public: true
          - name: "RedactablePayload"
            kind: "struct"
            public: true
          - name: "empty"
            kind: "function"
            public: true
          - name: "RedactionReport"
            kind: "struct"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pm_report"
      - path: "projects/jet/src/pm_report/ia.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "PM_REPORT_IA_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "PmReportSection"
            kind: "enum"
            public: true
          - name: "slug"
            kind: "function"
            public: true
          - name: "PmReportControls"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "PmReportRoute"
            kind: "struct"
            public: true
          - name: "for_section"
            kind: "function"
            public: true
          - name: "for_case"
            kind: "function"
            public: true
          - name: "for_step"
            kind: "function"
            public: true
          - name: "path"
            kind: "function"
            public: true
          - name: "PmReportFieldMapping"
            kind: "struct"
            public: true
          - name: "PmReportIa"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "default_sections"
            kind: "function"
            public: true
          - name: "default_field_mappings"
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
          domain: "projects/jet/src/pm_report"
      - path: "projects/jet/src/pm_report/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "deep_links"
            kind: "module"
            public: true
          - name: "ia"
            kind: "module"
            public: true
          - name: "loader"
            kind: "module"
            public: true
          - name: "metadata"
            kind: "module"
            public: true
          - name: "nav"
            kind: "module"
            public: true
          - name: "redaction"
            kind: "module"
            public: true
          - name: "states"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pm_report"
      - path: "projects/jet/src/pm_report/states.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "PM_REPORT_STATES_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ReportContentSummary"
            kind: "struct"
            public: true
          - name: "ReportState"
            kind: "enum"
            public: true
          - name: "is_renderable_report"
            kind: "function"
            public: true
          - name: "is_error"
            kind: "function"
            public: true
          - name: "classify"
            kind: "function"
            public: true
          - name: "from_error"
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
          domain: "projects/jet/src/pm_report"
      - path: "projects/jet/src/pm_report/deep_links.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "PM_REPORT_DEEP_LINK_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "DeepLink"
            kind: "enum"
            public: true
          - name: "DeepLinkParseError"
            kind: "enum"
            public: true
          - name: "DeepLinkOutcome"
            kind: "enum"
            public: true
          - name: "DeepLinkNotFound"
            kind: "enum"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "parse_fragment"
            kind: "function"
            public: true
          - name: "cursor_to_fragment"
            kind: "function"
            public: true
          - name: "apply"
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
          domain: "projects/jet/src/pm_report"
      - path: "projects/jet/src/pm_report/loader.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "PM_LOADER_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "StaticReportLoadError"
            kind: "enum"
            public: true
          - name: "ResolvedArtifact"
            kind: "struct"
            public: true
          - name: "StaticReportBundle"
            kind: "struct"
            public: true
          - name: "load_from_dir"
            kind: "function"
            public: true
          - name: "from_manifest"
            kind: "function"
            public: true
          - name: "artifact"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "format_pm_report_loader_non_utf8_artifact_warn"
            kind: "function"
            public: true
          - name: "resolve_artifact_path_or_warn"
            kind: "function"
            public: true
          - name: "resolve_artifact"
            kind: "function"
            public: false
          - name: "extract_schema_tag"
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
          domain: "projects/jet/src/pm_report"
      - path: "projects/jet/src/pm_report/nav.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "PM_REPORT_NAV_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "CaseNavInfo"
            kind: "struct"
            public: true
          - name: "StepNavInfo"
            kind: "struct"
            public: true
          - name: "NavPage"
            kind: "enum"
            public: true
          - name: "section"
            kind: "function"
            public: false
          - name: "PmReportCursor"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "NavRejection"
            kind: "enum"
            public: true
          - name: "NavEvent"
            kind: "enum"
            public: true
          - name: "select_case"
            kind: "function"
            public: true
          - name: "select_step"
            kind: "function"
            public: true
          - name: "open_failure"
            kind: "function"
            public: true
          - name: "open_artifacts"
            kind: "function"
            public: true
          - name: "return_to_summary"
            kind: "function"
            public: true
          - name: "active_section"
            kind: "function"
            public: true
          - name: "artifact_state"
            kind: "function"
            public: true
          - name: "ArtifactPanelState"
            kind: "enum"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pm_report"
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
  - path: "projects/jet/src/pm_report/metadata.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pm_report/redaction.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pm_report/ia.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pm_report/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pm_report/states.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pm_report/deep_links.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pm_report/loader.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pm_report/nav.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-pm-report.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
