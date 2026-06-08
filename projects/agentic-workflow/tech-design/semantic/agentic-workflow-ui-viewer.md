---
id: semantic-agentic-workflow-ui-viewer
summary: Semantic coverage for "projects/agentic-workflow/src/ui/viewer"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "This semantic TD covers AW core/client model source behavior and shared workflow domain primitives."
---

# Semantic TD: agentic-workflow/ui/viewer

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/ui/viewer"
  source_group: "projects/agentic-workflow/src/ui/viewer"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/ui/viewer/ipc.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model"]
        symbols:
          - name: "IpcError"
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
          domain: "projects/agentic-workflow/src/ui/viewer"
      - path: "projects/agentic-workflow/src/ui/viewer/render.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "SAFE_TAGS"
            kind: "constant"
            public: false
          - name: "SAFE_ATTRS"
            kind: "constant"
            public: false
          - name: "STRIP_CONTENT_TAGS"
            kind: "constant"
            public: false
          - name: "sanitize_html"
            kind: "function"
            public: false
          - name: "sanitize_tag"
            kind: "function"
            public: false
          - name: "parse_attributes"
            kind: "function"
            public: false
          - name: "slugify"
            kind: "function"
            public: true
          - name: "render_markdown_to_html"
            kind: "function"
            public: true
          - name: "wrap_latex_expressions"
            kind: "function"
            public: false
          - name: "render_yaml_to_html"
            kind: "function"
            public: true
          - name: "html_escape"
            kind: "function"
            public: false
          - name: "render_not_found_html"
            kind: "function"
            public: true
          - name: "wrap_in_document"
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
          domain: "projects/agentic-workflow/src/ui/viewer"
      - path: "projects/agentic-workflow/src/ui/viewer/manager.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ViewerManager"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "new_project_manager"
            kind: "function"
            public: true
          - name: "change_exists"
            kind: "function"
            public: true
          - name: "change_dir"
            kind: "function"
            public: true
          - name: "load_file"
            kind: "function"
            public: true
          - name: "load_annotations"
            kind: "function"
            public: true
          - name: "save_annotations"
            kind: "function"
            public: true
          - name: "annotations_path"
            kind: "function"
            public: true
          - name: "validate_filename"
            kind: "function"
            public: false
          - name: "list_files"
            kind: "function"
            public: true
          - name: "generate_project_tree"
            kind: "function"
            public: true
          - name: "scan_directory"
            kind: "function"
            public: false
          - name: "update_phase"
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
          domain: "projects/agentic-workflow/src/ui/viewer"
      - path: "projects/agentic-workflow/src/ui/viewer/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "AppState"
            kind: "struct"
            public: false
          - name: "start_viewer"
            kind: "function"
            public: true
          - name: "run_server"
            kind: "function"
            public: false
          - name: "find_available_port"
            kind: "function"
            public: false
          - name: "serve_index"
            kind: "function"
            public: false
          - name: "serve_assets"
            kind: "function"
            public: false
          - name: "InfoResponse"
            kind: "struct"
            public: false
          - name: "api_info"
            kind: "function"
            public: false
          - name: "api_list_files"
            kind: "function"
            public: false
          - name: "api_load_file"
            kind: "function"
            public: false
          - name: "SaveAnnotationRequest"
            kind: "struct"
            public: false
          - name: "api_save_annotation"
            kind: "function"
            public: false
          - name: "api_resolve_annotation"
            kind: "function"
            public: false
          - name: "api_approve_review"
            kind: "function"
            public: false
          - name: "api_request_changes"
            kind: "function"
            public: false
          - name: "api_submit_comments"
            kind: "function"
            public: false
          - name: "api_close_window"
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
          domain: "projects/agentic-workflow/src/ui/viewer"
      - path: "projects/agentic-workflow/src/ui/viewer/api.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ChangeResponse"
            kind: "struct"
            public: true
          - name: "ChangeSummaryResponse"
            kind: "struct"
            public: true
          - name: "IssueResponse"
            kind: "struct"
            public: true
          - name: "IssueSummaryResponse"
            kind: "struct"
            public: true
          - name: "LineageGraphResponse"
            kind: "struct"
            public: true
          - name: "ProjectInfoResponse"
            kind: "struct"
            public: true
          - name: "TechDesignResponse"
            kind: "struct"
            public: true
          - name: "TechDesignSummaryResponse"
            kind: "struct"
            public: true
          - name: "project_root"
            kind: "function"
            public: false
          - name: "issue_status_str"
            kind: "function"
            public: false
          - name: "priority_from_labels"
            kind: "function"
            public: false
          - name: "title_from_filename"
            kind: "function"
            public: false
          - name: "crate_from_relative_path"
            kind: "function"
            public: false
          - name: "api_list_issues"
            kind: "function"
            public: true
          - name: "api_get_issue"
            kind: "function"
            public: true
          - name: "api_list_tech_designs"
            kind: "function"
            public: true
          - name: "api_get_tech_design"
            kind: "function"
            public: true
          - name: "api_list_changes"
            kind: "function"
            public: true
          - name: "api_get_change"
            kind: "function"
            public: true
          - name: "api_get_lineage"
            kind: "function"
            public: true
          - name: "api_project_info"
            kind: "function"
            public: true
          - name: "collect_tech_design_files"
            kind: "function"
            public: false
          - name: "resolve_tech_design_file"
            kind: "function"
            public: false
          - name: "collect_md_files"
            kind: "function"
            public: false
          - name: "collect_md_files_recursive"
            kind: "function"
            public: false
          - name: "file_modified_iso"
            kind: "function"
            public: false
          - name: "load_change_state"
            kind: "function"
            public: false
          - name: "collect_spec_ids"
            kind: "function"
            public: false
          - name: "map_phase_to_frontend"
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
          domain: "projects/agentic-workflow/src/ui/viewer"
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
  - path: "projects/agentic-workflow/src/ui/viewer/ipc.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/viewer/render.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/viewer/manager.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/viewer/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/viewer/api.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
