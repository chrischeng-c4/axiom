---
id: semantic-agentic-workflow-generate
summary: Semantic coverage for "projects/agentic-workflow/src/generate"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/generate

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/generate"
  source_group: "projects/agentic-workflow/src/generate"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/generate/handwrite.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "CoverageReport"
            kind: "struct"
            public: true
          - name: "HandwriteEntry"
            kind: "struct"
            public: true
          - name: "HandwriteMarker"
            kind: "struct"
            public: true
          - name: "HandwriteParseError"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/types.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "AbstractType"
            kind: "enum"
            public: true
          - name: "default_int_size"
            kind: "function"
            public: false
          - name: "RustConfig"
            kind: "struct"
            public: true
          - name: "RustTypeTranslator"
            kind: "struct"
            public: true
          - name: "translate"
            kind: "function"
            public: false
          - name: "parse_abstract_type"
            kind: "function"
            public: true
          - name: "strip_wrapper"
            kind: "function"
            public: false
          - name: "split_map_params"
            kind: "function"
            public: false
          - name: "default_rust_derives"
            kind: "function"
            public: false
          - name: "default_serde_rename"
            kind: "function"
            public: false
          - name: "default_visibility"
            kind: "function"
            public: false
          - name: "default"
            kind: "function"
            public: false
          - name: "has_serde_derives"
            kind: "function"
            public: true
          - name: "merge_overrides"
            kind: "function"
            public: true
          - name: "derive_attr"
            kind: "function"
            public: true
          - name: "serde_rename_attr"
            kind: "function"
            public: true
          - name: "vis_prefix"
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
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/handwrite_scaffold.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "ScaffoldOutcome"
            kind: "enum"
            public: true
          - name: "scaffold_handwrite"
            kind: "function"
            public: true
          - name: "PENDING_TRACKER"
            kind: "constant"
            public: true
          - name: "derive_gap"
            kind: "function"
            public: false
          - name: "derive_tracker"
            kind: "function"
            public: false
          - name: "derive_reason"
            kind: "function"
            public: false
          - name: "line_matches_anchor"
            kind: "function"
            public: false
          - name: "has_surrounding_marker"
            kind: "function"
            public: false
          - name: "find_block_end"
            kind: "function"
            public: false
          - name: "escape_attr"
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
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/frontmatter.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "MermaidPlusBlock"
            kind: "struct"
            public: true
          - name: "extract_mermaid_plus_blocks"
            kind: "function"
            public: true
          - name: "parse_section_type_annotation"
            kind: "function"
            public: true
          - name: "parse_mermaid_block"
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
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/audit.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "ReportKind"
            kind: "enum"
            public: true
          - name: "BlockReport"
            kind: "struct"
            public: true
          - name: "MarkerGap"
            kind: "struct"
            public: true
          - name: "UncoveredItem"
            kind: "struct"
            public: true
          - name: "audit_file"
            kind: "function"
            public: true
          - name: "audit_block"
            kind: "function"
            public: false
          - name: "generated_block_content"
            kind: "function"
            public: false
          - name: "is_rust_source_path"
            kind: "function"
            public: false
          - name: "is_handwritten_change_entry"
            kind: "function"
            public: false
          - name: "split_spec_ref"
            kind: "function"
            public: false
          - name: "regenerate_for_block"
            kind: "function"
            public: false
          - name: "normalize"
            kind: "function"
            public: false
          - name: "rustfmt_normalized_eq"
            kind: "function"
            public: false
          - name: "rustfmt_snippet"
            kind: "function"
            public: false
          - name: "summarize_diff"
            kind: "function"
            public: false
          - name: "audit_markers"
            kind: "function"
            public: true
          - name: "has_skip_file_directive"
            kind: "function"
            public: false
          - name: "parse_spec_managed"
            kind: "function"
            public: false
          - name: "looks_like_top_level_item"
            kind: "function"
            public: false
          - name: "has_spec_marker_above"
            kind: "function"
            public: false
          - name: "SpecFileIndex"
            kind: "type"
            public: true
          - name: "build_spec_file_index"
            kind: "function"
            public: true
          - name: "walk_md_recursive"
            kind: "function"
            public: false
          - name: "audit_uncovered"
            kind: "function"
            public: true
          - name: "handwrite_ranges_for_uncovered"
            kind: "function"
            public: false
          - name: "lenient_handwrite_ranges"
            kind: "function"
            public: false
          - name: "skip_literal_line"
            kind: "function"
            public: false
          - name: "opens_regular_string_continuation"
            kind: "function"
            public: false
          - name: "regular_string_continuation_closes"
            kind: "function"
            public: false
          - name: "UnifiedReport"
            kind: "enum"
            public: true
          - name: "is_clean"
            kind: "function"
            public: true
          - name: "file"
            kind: "function"
            public: true
          - name: "status"
            kind: "function"
            public: true
          - name: "gap"
            kind: "function"
            public: true
          - name: "audit_file_unified"
            kind: "function"
            public: true
          - name: "HandwriteParseFailure"
            kind: "enum"
            public: true
          - name: "into_struct_error"
            kind: "function"
            public: false
          - name: "parse_handwrite_markers"
            kind: "function"
            public: true
          - name: "detect_unclosed_raw_string"
            kind: "function"
            public: false
          - name: "strip_comment_lead"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/render.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "BlockRenderResult"
            kind: "struct"
            public: true
          - name: "RenderReport"
            kind: "struct"
            public: true
          - name: "run_render"
            kind: "function"
            public: true
          - name: "regenerate_body"
            kind: "function"
            public: false
          - name: "render_state_diagram"
            kind: "function"
            public: false
          - name: "render_sequence_diagram"
            kind: "function"
            public: false
          - name: "render_flowchart"
            kind: "function"
            public: false
          - name: "render_requirement_diagram"
            kind: "function"
            public: false
          - name: "rebuild_spec_content"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "diagrams"
            kind: "module"
            public: true
          - name: "engine"
            kind: "module"
            public: true
          - name: "generators"
            kind: "module"
            public: true
          - name: "mcp"
            kind: "module"
            public: true
          - name: "patterns"
            kind: "module"
            public: true
          - name: "schema"
            kind: "module"
            public: true
          - name: "spec_ir"
            kind: "module"
            public: true
          - name: "specs"
            kind: "module"
            public: true
          - name: "validator"
            kind: "module"
            public: true
          - name: "apply"
            kind: "module"
            public: true
          - name: "audit"
            kind: "module"
            public: true
          - name: "diff"
            kind: "module"
            public: true
          - name: "frontmatter"
            kind: "module"
            public: true
          - name: "gen"
            kind: "module"
            public: true
          - name: "marker"
            kind: "module"
            public: true
          - name: "render"
            kind: "module"
            public: true
          - name: "types"
            kind: "module"
            public: true
          - name: "from_td_ast"
            kind: "module"
            public: true
          - name: "handwrite"
            kind: "module"
            public: true
          - name: "handwrite_scaffold"
            kind: "module"
            public: true
          - name: "handwrite_scaffold_test"
            kind: "module"
            public: false
          - name: "Result"
            kind: "type"
            public: true
          - name: "GenerateError"
            kind: "enum"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "from"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/apply.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "FileApplyResult"
            kind: "struct"
            public: true
          - name: "ApplyReport"
            kind: "struct"
            public: true
          - name: "total_blocks_updated"
            kind: "function"
            public: true
          - name: "files_created"
            kind: "function"
            public: true
          - name: "run_apply"
            kind: "function"
            public: true
          - name: "run_apply_scoped"
            kind: "function"
            public: true
          - name: "run_apply_scoped_sections"
            kind: "function"
            public: true
          - name: "run_apply_scoped_targets"
            kind: "function"
            public: true
          - name: "run_apply_worktree"
            kind: "function"
            public: true
          - name: "run_apply_inner"
            kind: "function"
            public: false
          - name: "is_whole_file_codegen_section"
            kind: "function"
            public: false
          - name: "is_whole_file_codegen_content"
            kind: "function"
            public: false
          - name: "is_whole_file_handwrite_content"
            kind: "function"
            public: false
          - name: "auto_wire_readme_symbols"
            kind: "function"
            public: true
          - name: "write_readme_symbols"
            kind: "function"
            public: false
          - name: "insert_readme_symbols_block"
            kind: "function"
            public: false
          - name: "auto_wire_mamba_lib"
            kind: "function"
            public: true
          - name: "configured_readme_symbol_paths"
            kind: "function"
            public: false
          - name: "exports_mamba_register"
            kind: "function"
            public: false
          - name: "extract_existing_modules"
            kind: "function"
            public: false
          - name: "insert_register_body_block"
            kind: "function"
            public: false
          - name: "scaffold_handwrite_file"
            kind: "function"
            public: false
          - name: "handwrite_attr_value"
            kind: "function"
            public: false
          - name: "short_path_hash"
            kind: "function"
            public: false
          - name: "ChangeEntry"
            kind: "struct"
            public: true
          - name: "ImplMode"
            kind: "enum"
            public: true
          - name: "from_str"
            kind: "function"
            public: false
          - name: "extract_change_entries_count"
            kind: "function"
            public: true
          - name: "missing_implementation_paths"
            kind: "function"
            public: true
          - name: "is_all_hand_written"
            kind: "function"
            public: true
          - name: "extract_section_anchors"
            kind: "function"
            public: true
          - name: "parse_fn_name_from_signature"
            kind: "function"
            public: false
          - name: "collect_schema_anchors"
            kind: "function"
            public: false
          - name: "collect_cli_anchors"
            kind: "function"
            public: false
          - name: "section_yaml_block"
            kind: "function"
            public: false
          - name: "should_emit_section_to_entry"
            kind: "function"
            public: true
          - name: "extract_change_entries"
            kind: "function"
            public: true
          - name: "MODULE_PREAMBLE_REPLACE"
            kind: "constant"
            public: false
          - name: "MODULE_TRAILER_REPLACE"
            kind: "constant"
            public: false
          - name: "HANDWRITE_GAP_REPLACE_PREFIX"
            kind: "constant"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/from_td_ast.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "DispatchCtx"
            kind: "struct"
            public: true
          - name: "DispatchOutcome"
            kind: "struct"
            public: true
          - name: "DispatchReport"
            kind: "struct"
            public: true
          - name: "DispatchStatus"
            kind: "enum"
            public: true
          - name: "dispatch_from_tdast"
            kind: "function"
            public: true
          - name: "classify_section"
            kind: "function"
            public: false
          - name: "collect_orphan_changes_paths"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "enter"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/diff.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "DiffClass"
            kind: "enum"
            public: true
          - name: "DiffReport"
            kind: "struct"
            public: true
          - name: "FileDiff"
            kind: "struct"
            public: true
          - name: "overall_drift_pct"
            kind: "function"
            public: true
          - name: "has_drift"
            kind: "function"
            public: true
          - name: "run_diff"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "extract_change_paths"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/generate"
      - path: "projects/agentic-workflow/src/generate/marker.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Lang"
            kind: "enum"
            public: true
          - name: "line_comment"
            kind: "function"
            public: true
          - name: "line_comment_end"
            kind: "function"
            public: true
          - name: "is_codegen_begin"
            kind: "function"
            public: false
          - name: "is_codegen_end"
            kind: "function"
            public: false
          - name: "CodegenBlock"
            kind: "struct"
            public: true
          - name: "parse_codegen_blocks"
            kind: "function"
            public: true
          - name: "rust_raw_string_line_mask"
            kind: "function"
            public: true
          - name: "rust_marker_fixture_line_mask"
            kind: "function"
            public: false
          - name: "rust_string_line_mask"
            kind: "function"
            public: false
          - name: "rust_char_literal_end_at"
            kind: "function"
            public: false
          - name: "raw_string_start_at"
            kind: "function"
            public: false
          - name: "raw_string_end_at"
            kind: "function"
            public: false
          - name: "parse_spec_managed_comment"
            kind: "function"
            public: false
          - name: "replace_codegen_block"
            kind: "function"
            public: true
          - name: "insert_codegen_block"
            kind: "function"
            public: true
          - name: "emit_spec_ref"
            kind: "function"
            public: true
          - name: "MarkerEntry"
            kind: "struct"
            public: true
          - name: "collect_spec_refs"
            kind: "function"
            public: true
          - name: "group_markers"
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
          domain: "projects/agentic-workflow/src/generate"
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
  - path: "projects/agentic-workflow/src/generate/handwrite.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/types.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/handwrite_scaffold.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/frontmatter.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/audit.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/render.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/apply.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/from_td_ast.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/diff.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/generate/marker.rs"
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
