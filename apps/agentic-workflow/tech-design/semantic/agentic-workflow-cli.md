---
id: semantic-agentic-workflow-cli
summary: Semantic coverage for "apps/agentic-workflow/src/cli"
capability_refs:
  - id: "aw-core-client-model-workitem-first-artifact-lifecycle"
    role: primary
    gap: "core-concept-model-and-invariants"
    claim: "core-concept-model-and-invariants"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `apps/agentic-workflow/src/cli`."
  - id: project-local-td-and-ec-gates
    role: primary
    gap: ec-evidence-documentation
    claim: ec-evidence-documentation
    coverage: partial
    rationale: "The CLI semantic domain covers `aw ec doc` rendering, check, preview, and EC evidence documentation behavior in src/cli/ec.rs."
  - id: project-local-td-and-ec-gates
    role: primary
    gap: ec-external-contract-source
    claim: ec-external-contract-source
    coverage: partial
    rationale: "The CLI semantic domain covers `aw ec draft/fill/gen` project-local external-contract markdown and generated aw.toml EC inventory behavior in src/cli/ec.rs."
fill_sections: [schema, changes]
---

# Semantic TD: agentic-workflow/cli

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/cli"
  source_group: "apps/agentic-workflow/src/cli"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/agentic-workflow/src/cli/hook.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "HookArgs"
            kind: "struct"
            public: true
          - name: "HookEvent"
            kind: "enum"
            public: true
          - name: "PretooluseKind"
            kind: "enum"
            public: true
          - name: "PosttooluseKind"
            kind: "enum"
            public: true
          - name: "Decision"
            kind: "enum"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "run_workflow_guard"
            kind: "function"
            public: false
          - name: "run_workflow_apply"
            kind: "function"
            public: false
          - name: "read_json_payload"
            kind: "function"
            public: false
          - name: "workflow_hook_decision"
            kind: "function"
            public: false
          - name: "run_write_scope_guarded"
            kind: "function"
            public: false
          - name: "panic_message"
            kind: "function"
            public: false
          - name: "emit_and_exit"
            kind: "function"
            public: false
          - name: "write_scope"
            kind: "module"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/proposal.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "ProposalCommands"
            kind: "enum"
            public: true
          - name: "run"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/shell_env.rs"
        language: "rust"
        ownership_state: "unmanaged"
        generator_primitives: ["service_method"]
        symbols:
          - name: "apply_default_shell_env"
            kind: "function"
            public: true
          - name: "default_path"
            kind: "function"
            public: false
          - name: "default_home"
            kind: "function"
            public: false
          - name: "default_home_from"
            kind: "function"
            public: false
          - name: "default_path_for"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/sdd.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/fillback.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "run"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/cb_fill.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "HandwriteMarkerEntry"
            kind: "struct"
            public: true
          - name: "enumerate_worktree_markers"
            kind: "function"
            public: true
          - name: "marker_body_is_unfilled"
            kind: "function"
            public: false
          - name: "count_worktree_handwrite_markers"
            kind: "function"
            public: true
          - name: "cb_marker_payload_path"
            kind: "function"
            public: false
          - name: "cb_fill_apply_command"
            kind: "function"
            public: false
          - name: "td_code_check_command"
            kind: "function"
            public: false
          - name: "marker_payload_template"
            kind: "function"
            public: false
          - name: "initialize_marker_payload"
            kind: "function"
            public: false
          - name: "next_for_marker"
            kind: "function"
            public: false
          - name: "next_for_td_code_check"
            kind: "function"
            public: false
          - name: "print_compact_json"
            kind: "function"
            public: false
          - name: "BeginEndMarker"
            kind: "struct"
            public: false
          - name: "HANDWRITE_BEGIN_TOKEN"
            kind: "constant"
            public: false
          - name: "HANDWRITE_END_TOKEN"
            kind: "constant"
            public: false
          - name: "parse_handwrite_begin_end"
            kind: "function"
            public: false
          - name: "strip_lead"
            kind: "function"
            public: false
          - name: "extract_xml_attr"
            kind: "function"
            public: false
          - name: "slugify_short"
            kind: "function"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "run_brief"
            kind: "function"
            public: false
          - name: "resolve_active_spec_path"
            kind: "function"
            public: false
          - name: "derive_spec_path_from_implements"
            kind: "function"
            public: false
          - name: "extract_change_paths_from_spec"
            kind: "function"
            public: true
          - name: "append_change_paths_from_yaml"
            kind: "function"
            public: false
          - name: "filter_markers_to_change_paths"
            kind: "function"
            public: true
          - name: "scope_markers_for_change_paths"
            kind: "function"
            public: true
          - name: "path_matches"
            kind: "function"
            public: false
          - name: "normalize_rel_path"
            kind: "function"
            public: false
          - name: "run_apply"
            kind: "function"
            public: false
          - name: "replace_block_body"
            kind: "function"
            public: false
          - name: "replace_block_body_for_path"
            kind: "function"
            public: false
          - name: "should_preserve_handwrite_markers"
            kind: "function"
            public: false
          - name: "replace_block_and_markers"
            kind: "function"
            public: false
          - name: "resolve_base_branch"
            kind: "function"
            public: false
          - name: "branch_changed_files"
            kind: "function"
            public: true
          - name: "run_cb_check_gate"
            kind: "function"
            public: false
          - name: "should_stage_lifecycle_path"
            kind: "function"
            public: false
          - name: "stage_and_commit_cb_fill"
            kind: "function"
            public: false
          - name: "stage_and_commit_cb_marker"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/ec.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "EC_MANIFEST_VERSION"
            kind: "constant"
            public: false
          - name: "LEGACY_EC_MANIFEST_FILE"
            kind: "constant"
            public: false
          - name: "EC_DOC_REL"
            kind: "constant"
            public: false
          - name: "EC_SOURCE_REL"
            kind: "constant"
            public: false
          - name: "PROJECT_AW_REL"
            kind: "constant"
            public: false
          - name: "EC_AW_BEGIN_MARKER"
            kind: "constant"
            public: false
          - name: "EC_AW_END_MARKER"
            kind: "constant"
            public: false
          - name: "EC_BEGIN_MARKER"
            kind: "constant"
            public: false
          - name: "EC_END_MARKER"
            kind: "constant"
            public: false
          - name: "EC_TOOL_BEGIN_MARKER"
            kind: "constant"
            public: false
          - name: "EC_TOOL_END_MARKER"
            kind: "constant"
            public: false
          - name: "EC_DOC_BEGIN_MARKER"
            kind: "constant"
            public: false
          - name: "EC_DOC_END_MARKER"
            kind: "constant"
            public: false
          - name: "EC_CATEGORIES"
            kind: "constant"
            public: false
          - name: "ec_categories"
            kind: "function"
            public: true
          - name: "EcArgs"
            kind: "struct"
            public: true
          - name: "EcCommand"
            kind: "enum"
            public: true
          - name: "EcDraftArgs"
            kind: "struct"
            public: true
          - name: "EcFillArgs"
            kind: "struct"
            public: true
          - name: "EcGenArgs"
            kind: "struct"
            public: true
          - name: "EcCheckArgs"
            kind: "struct"
            public: true
          - name: "EcVerifyArgs"
            kind: "struct"
            public: true
          - name: "EcDocArgs"
            kind: "struct"
            public: true
          - name: "EcDocCommand"
            kind: "enum"
            public: true
          - name: "EcDocGenArgs"
            kind: "struct"
            public: true
          - name: "EcDocCheckArgs"
            kind: "struct"
            public: true
          - name: "EcDocPreviewArgs"
            kind: "struct"
            public: true
          - name: "EcManifest"
            kind: "struct"
            public: true
          - name: "EcManifestCase"
            kind: "struct"
            public: true
          - name: "EcEvidenceArtifact"
            kind: "struct"
            public: true
          - name: "EcEvaluator"
            kind: "struct"
            public: true
          - name: "EcToolManifest"
            kind: "struct"
            public: true
          - name: "EcCheckSummary"
            kind: "struct"
            public: true
          - name: "EcDocCheckSummary"
            kind: "struct"
            public: true
          - name: "EcDocPreviewSummary"
            kind: "struct"
            public: true
          - name: "EcVerifySummary"
            kind: "struct"
            public: true
          - name: "EcVerifyCommandResult"
            kind: "struct"
            public: true
          - name: "EcProjectContext"
            kind: "struct"
            public: true
          - name: "E2eYaml"
            kind: "struct"
            public: false
          - name: "E2eYamlCase"
            kind: "struct"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/tasks.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "TasksCommands"
            kind: "enum"
            public: true
          - name: "run"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/merge_target.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "resolve_merge_target"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/cb.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "AW_EC_BEGIN_MARKER"
            kind: "constant"
            public: false
          - name: "CbArgs"
            kind: "struct"
            public: true
          - name: "CbCommand"
            kind: "enum"
            public: true
          - name: "CbFillArgs"
            kind: "struct"
            public: true
          - name: "CbClaimArgs"
            kind: "struct"
            public: true
          - name: "CbGenArgs"
            kind: "struct"
            public: true
          - name: "CbCheckArgs"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "CbGenSourceArgs"
            kind: "struct"
            public: true
          - name: "run_gen_source"
            kind: "function"
            public: false
          - name: "run_gen"
            kind: "function"
            public: true
          - name: "run_force_regen"
            kind: "function"
            public: false
          - name: "run_force_regen_verify"
            kind: "function"
            public: false
          - name: "run_force_regen_verify_cold"
            kind: "function"
            public: false
          - name: "force_regen_verify_cold_summary_at"
            kind: "function"
            public: false
          - name: "CbCodegenOriginClass"
            kind: "enum"
            public: false
          - name: "codegen_origin_for_cold_targets"
            kind: "function"
            public: false
          - name: "classify_codegen_origin_spec"
            kind: "function"
            public: false
          - name: "source_section_has_type_marker"
            kind: "function"
            public: false
          - name: "CbVerifySummary"
            kind: "struct"
            public: true
          - name: "CbCodegenOriginSummary"
            kind: "struct"
            public: true
          - name: "CbColdVerifySummary"
            kind: "struct"
            public: true
          - name: "percent_of"
            kind: "function"
            public: false
          - name: "project_force_regen_verify_summary"
            kind: "function"
            public: true
          - name: "cb_verify_summary_from_report"
            kind: "function"
            public: false
          - name: "project_force_regen_cold_verify_summary"
            kind: "function"
            public: true
          - name: "project_force_regen_cold_verify_workspaces"
            kind: "function"
            public: true
          - name: "run_force_regen_specs"
            kind: "function"
            public: false
          - name: "write_project_root_llms_targets"
            kind: "function"
            public: false
          - name: "extract_project_root_llms_target_paths"
            kind: "function"
            public: false
          - name: "format_rust_files"
            kind: "function"
            public: false
          - name: "commit_force_regen"
            kind: "function"
            public: false
          - name: "ForceRegenScope"
            kind: "struct"
            public: false
          - name: "resolve_project_force_regen_scope"
            kind: "function"
            public: false
          - name: "CbGenConfig"
            kind: "struct"
            public: false
          - name: "CbGenProject"
            kind: "struct"
            public: false
          - name: "matches"
            kind: "function"
            public: false
          - name: "CbGenWorkspace"
            kind: "struct"
            public: false
          - name: "project_source_roots"
            kind: "function"
            public: false
          - name: "workspace_source_roots"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/chat_members.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ChannelMessage"
            kind: "struct"
            public: true
          - name: "MessageFrontmatter"
            kind: "struct"
            public: true
          - name: "Member"
            kind: "struct"
            public: true
          - name: "MembersFile"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "resolve_identity"
            kind: "function"
            public: true
          - name: "git_toplevel_from"
            kind: "function"
            public: false
          - name: "git_branch_from"
            kind: "function"
            public: false
          - name: "detect_team_identity"
            kind: "function"
            public: true
          - name: "detect_git_branch"
            kind: "function"
            public: true
          - name: "detect_git_toplevel"
            kind: "function"
            public: true
          - name: "lookup_member_name_by_branch"
            kind: "function"
            public: true
          - name: "read_config_team_name"
            kind: "function"
            public: true
          - name: "is_old_pipe_format"
            kind: "function"
            public: true
          - name: "parse_pipe_line"
            kind: "function"
            public: true
          - name: "parse_pipe_format"
            kind: "function"
            public: true
          - name: "serialize_message_block"
            kind: "function"
            public: true
          - name: "rewrite_channel_as_frontmatter"
            kind: "function"
            public: true
          - name: "parse_channel_markdown"
            kind: "function"
            public: true
          - name: "looks_like_jsonl"
            kind: "function"
            public: false
          - name: "parse_channel_jsonl"
            kind: "function"
            public: true
          - name: "serialize_message_jsonl"
            kind: "function"
            public: true
          - name: "parse_frontmatter_blocks"
            kind: "function"
            public: true
          - name: "read_members_file"
            kind: "function"
            public: true
          - name: "write_members_file"
            kind: "function"
            public: true
          - name: "run_members_register"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/capability_type.rs"
        language: "rust"
        ownership_state: "unmanaged"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "CAPABILITY_TYPES_REL"
            kind: "constant"
            public: true
          - name: "CapabilityType"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "from_cli_str"
            kind: "function"
            public: true
          - name: "required_ec_dimensions"
            kind: "function"
            public: true
          - name: "category_is_required_for_type"
            kind: "function"
            public: true
          - name: "CapabilityTypesFile"
            kind: "struct"
            public: false
          - name: "capability_types_path"
            kind: "function"
            public: true
          - name: "load_capability_types_from_readme"
            kind: "function"
            public: true
          - name: "explicit_capability_types_from_readme"
            kind: "function"
            public: false
          - name: "explicit_capability_type_from_block"
            kind: "function"
            public: false
          - name: "split_markdown_field"
            kind: "function"
            public: false
          - name: "is_markdown_heading"
            kind: "function"
            public: false
          - name: "next_markdown_heading"
            kind: "function"
            public: false
          - name: "parse_markdown_table_at"
            kind: "function"
            public: false
          - name: "parse_markdown_table_row"
            kind: "function"
            public: false
          - name: "is_markdown_separator_row"
            kind: "function"
            public: false
          - name: "table_cell"
            kind: "function"
            public: false
          - name: "find_table_column"
            kind: "function"
            public: false
          - name: "normalize_key"
            kind: "function"
            public: false
          - name: "load_capability_types"
            kind: "function"
            public: true
          - name: "upsert_capability_type"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/issues.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "IssuesArgs"
            kind: "struct"
            public: true
          - name: "IssuesCommand"
            kind: "enum"
            public: true
          - name: "DraftArgs"
            kind: "struct"
            public: true
          - name: "DraftCommand"
            kind: "enum"
            public: true
          - name: "DraftInitArgs"
            kind: "struct"
            public: true
          - name: "DraftFillArgs"
            kind: "struct"
            public: true
          - name: "DraftValidateArgs"
            kind: "struct"
            public: true
          - name: "DraftReviewArgs"
            kind: "struct"
            public: true
          - name: "ListArgs"
            kind: "struct"
            public: true
          - name: "ShowArgs"
            kind: "struct"
            public: true
          - name: "CreateArgs"
            kind: "struct"
            public: true
          - name: "PriorityFilter"
            kind: "enum"
            public: true
          - name: "as_label_suffix"
            kind: "function"
            public: true
          - name: "UpdateArgs"
            kind: "struct"
            public: true
          - name: "CloseArgs"
            kind: "struct"
            public: true
          - name: "FindArgs"
            kind: "struct"
            public: true
          - name: "PlanArgs"
            kind: "struct"
            public: true
          - name: "EpicizeArgs"
            kind: "struct"
            public: true
          - name: "AtomizeArgs"
            kind: "struct"
            public: true
          - name: "PrioritizeArgs"
            kind: "struct"
            public: true
          - name: "EnrichArgs"
            kind: "struct"
            public: true
          - name: "ValidateArgs"
            kind: "struct"
            public: true
          - name: "FillSectionArgs"
            kind: "struct"
            public: true
          - name: "ArbitrateArgs"
            kind: "struct"
            public: true
          - name: "ReviewArgs"
            kind: "struct"
            public: true
          - name: "BackendKind"
            kind: "enum"
            public: true
          - name: "StateFilter"
            kind: "enum"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "TypeFilter"
            kind: "enum"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "emit_create_envelope_error"
            kind: "function"
            public: false
          - name: "emit_json_error"
            kind: "function"
            public: false
          - name: "emit_validation_error"
            kind: "function"
            public: false
          - name: "read_body_file"
            kind: "function"
            public: false
          - name: "default_structured_issue_body"
            kind: "function"
            public: false
          - name: "body_from_inputs"
            kind: "function"
            public: false
          - name: "draft_body_from_inputs"
            kind: "function"
            public: false
          - name: "normalize_initial_draft_body"
            kind: "function"
            public: false
          - name: "replace_h2_content"
            kind: "function"
            public: false
          - name: "normalize_known_draft_sections"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/conf.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ConfArgs"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "run_at_root"
            kind: "function"
            public: false
          - name: "run_drift_check"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/update.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "CURRENT_VERSION"
            kind: "constant"
            public: false
          - name: "REPO"
            kind: "constant"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "get_latest_version"
            kind: "function"
            public: false
          - name: "is_newer"
            kind: "function"
            public: true
          - name: "update_binary"
            kind: "function"
            public: false
          - name: "detect_platform"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/remote_push.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "maybe_push_remote"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/standardize_audit.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "AUDIT_DIR"
            kind: "constant"
            public: false
          - name: "PreservationSurfaceKind"
            kind: "enum"
            public: true
          - name: "PreservationSurface"
            kind: "struct"
            public: true
          - name: "ModernizationRisk"
            kind: "enum"
            public: true
          - name: "SafeModernizationLever"
            kind: "struct"
            public: true
          - name: "PreservationAudit"
            kind: "struct"
            public: true
          - name: "StandardizeAuditDecision"
            kind: "struct"
            public: true
          - name: "audit_path"
            kind: "function"
            public: true
          - name: "evaluate_audit_decision"
            kind: "function"
            public: true
          - name: "fixture_audit"
            kind: "function"
            public: true
          - name: "is_quality_changing_action"
            kind: "function"
            public: false
          - name: "preservation_surface_names"
            kind: "function"
            public: false
          - name: "sanitize_project_key"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/check_alignment.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "run"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/run.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "GOAL_INLINE_LIMIT_BYTES"
            kind: "constant"
            public: false
          - name: "RunArgs"
            kind: "struct"
            public: true
          - name: "ResolvedRunRoot"
            kind: "enum"
            public: false
          - name: "command"
            kind: "function"
            public: false
          - name: "WorkflowNode"
            kind: "struct"
            public: false
          - name: "WorkflowNext"
            kind: "struct"
            public: false
          - name: "WorkflowInvoke"
            kind: "struct"
            public: false
          - name: "WorkflowCompletion"
            kind: "struct"
            public: false
          - name: "WorkflowPersistence"
            kind: "struct"
            public: false
          - name: "WorkflowEnvelope"
            kind: "struct"
            public: false
          - name: "serialize"
            kind: "function"
            public: false
          - name: "SerializableWorkflowCompletion"
            kind: "struct"
            public: false
          - name: "SerializableWorkflowNext"
            kind: "struct"
            public: false
          - name: "CanonicalWorkflowCompletion"
            kind: "struct"
            public: false
          - name: "CanonicalWorkflowNext"
            kind: "struct"
            public: false
          - name: "workflow_status"
            kind: "function"
            public: false
          - name: "serializable_next"
            kind: "function"
            public: false
          - name: "canonical_completion"
            kind: "function"
            public: false
          - name: "canonical_next_owned"
            kind: "function"
            public: false
          - name: "canonical_next_kind"
            kind: "function"
            public: false
          - name: "WorkflowGoalEnvelope"
            kind: "struct"
            public: false
          - name: "RunPrintOptions"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "run_resolved_root"
            kind: "function"
            public: false
          - name: "wi_run_command"
            kind: "function"
            public: true
          - name: "run_wi_root"
            kind: "function"
            public: true
          - name: "capability_run_command"
            kind: "function"
            public: true
          - name: "run_capability_root"
            kind: "function"
            public: true
          - name: "project_capability_rollup_command"
            kind: "function"
            public: true
          - name: "print_run_deprecation_notice"
            kind: "function"
            public: false
          - name: "resolve_run_root"
            kind: "function"
            public: false
          - name: "resolve_explicit_root"
            kind: "function"
            public: false
          - name: "resolve_capability_root_parts"
            kind: "function"
            public: false
          - name: "capability_root_command"
            kind: "function"
            public: false
          - name: "infer_current_project"
            kind: "function"
            public: false
          - name: "canonical_project_name_or_self"
            kind: "function"
            public: false
          - name: "RunProgressSink"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
          - name: "emit"
            kind: "function"
            public: false
          - name: "heartbeat"
            kind: "function"
            public: false
          - name: "RunProgressHeartbeat"
            kind: "struct"
            public: false
          - name: "drop"
            kind: "function"
            public: false
          - name: "emit_run_progress_event"
            kind: "function"
            public: false
          - name: "await_with_progress"
            kind: "function"
            public: false
          - name: "workflow_goal_envelope"
            kind: "function"
            public: false
          - name: "workflow_goal_prompt"
            kind: "function"
            public: false
          - name: "workflow_goal_payload_path"
            kind: "function"
            public: false
          - name: "write_goal_payload"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/production.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ProductionStatus"
            kind: "enum"
            public: true
          - name: "ProductionCapabilityReadiness"
            kind: "struct"
            public: true
          - name: "ProductionReadinessReport"
            kind: "struct"
            public: true
          - name: "ProductionCapabilityInput"
            kind: "struct"
            public: true
          - name: "inputs_from_sections"
            kind: "function"
            public: true
          - name: "inputs_from_report_items"
            kind: "function"
            public: true
          - name: "evaluate_release_scope"
            kind: "function"
            public: true
          - name: "evaluate_capability_scope"
            kind: "function"
            public: true
          - name: "evaluate_release_scope_with_regenerability"
            kind: "function"
            public: true
          - name: "evaluate_capability_scope_with_regenerability"
            kind: "function"
            public: true
          - name: "evaluate_scope"
            kind: "function"
            public: false
          - name: "visit_scope"
            kind: "function"
            public: false
          - name: "dependency_closure_for"
            kind: "function"
            public: false
          - name: "capability_ready"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/workflow_guard.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "LOCK_LABEL"
            kind: "constant"
            public: true
          - name: "TD_LOCK_LABEL"
            kind: "constant"
            public: true
          - name: "CB_LOCK_LABEL"
            kind: "constant"
            public: true
          - name: "STATE_START"
            kind: "constant"
            public: false
          - name: "STATE_END"
            kind: "constant"
            public: false
          - name: "TransitionLock"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "with_expected_payload"
            kind: "function"
            public: true
          - name: "with_phase_from"
            kind: "function"
            public: true
          - name: "with_active_phase"
            kind: "function"
            public: true
          - name: "with_active_branch"
            kind: "function"
            public: true
          - name: "with_current_section"
            kind: "function"
            public: true
          - name: "with_remaining_sections"
            kind: "function"
            public: true
          - name: "with_dirty_paths"
            kind: "function"
            public: true
          - name: "WorkflowProjection"
            kind: "struct"
            public: true
          - name: "from_lock"
            kind: "function"
            public: false
          - name: "IssueLockView"
            kind: "struct"
            public: true
          - name: "from_issue"
            kind: "function"
            public: false
          - name: "parse_projection"
            kind: "function"
            public: true
          - name: "upsert_projection"
            kind: "function"
            public: true
          - name: "unlock_projection_for_closed_issue"
            kind: "function"
            public: true
          - name: "create_issue_lock"
            kind: "function"
            public: true
          - name: "complete_issue_lock"
            kind: "function"
            public: true
          - name: "record_issue_blocker"
            kind: "function"
            public: true
          - name: "guard_issue_mutation"
            kind: "function"
            public: true
          - name: "issue_locks"
            kind: "function"
            public: true
          - name: "hook_pretooluse_workflow_guard"
            kind: "function"
            public: true
          - name: "hook_posttooluse_workflow_apply"
            kind: "function"
            public: true
          - name: "HookDecision"
            kind: "enum"
            public: true
          - name: "lock_owner_from_labels"
            kind: "function"
            public: false
          - name: "owner_label"
            kind: "function"
            public: false
          - name: "owner_labels"
            kind: "function"
            public: false
          - name: "maybe_push_issue"
            kind: "function"
            public: false
          - name: "payload_file_path"
            kind: "function"
            public: false
          - name: "path_to_rel"
            kind: "function"
            public: false
          - name: "path_allowed_by_lock"
            kind: "function"
            public: false
          - name: "is_score_workflow_mutation"
            kind: "function"
            public: false
          - name: "command_matches"
            kind: "function"
            public: false
          - name: "normalize_command"
            kind: "function"
            public: false
          - name: "normalize_rel"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/commands.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "Commands"
            kind: "enum"
            public: true
          - name: "run_command"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/migrate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "migrate_config"
            kind: "function"
            public: true
          - name: "version_lt"
            kind: "function"
            public: false
          - name: "migrate_envfile_support"
            kind: "function"
            public: false
          - name: "insert_before"
            kind: "function"
            public: false
          - name: "insert_provider_envfile"
            kind: "function"
            public: false
          - name: "migrate_project_section"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/slug_workspace.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ActiveWorkspace"
            kind: "struct"
            public: true
          - name: "enter_workspace_for_verb"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "capability"
            kind: "module"
            public: true
          - name: "capability_type"
            kind: "module"
            public: true
          - name: "cb"
            kind: "module"
            public: true
          - name: "cb_arbitrate"
            kind: "module"
            public: true
          - name: "cb_fill"
            kind: "module"
            public: true
          - name: "cb_review"
            kind: "module"
            public: true
          - name: "cb_revise"
            kind: "module"
            public: true
          - name: "chat"
            kind: "module"
            public: true
          - name: "check_alignment"
            kind: "module"
            public: true
          - name: "commands"
            kind: "module"
            public: true
          - name: "ec"
            kind: "module"
            public: true
          - name: "fillback"
            kind: "module"
            public: true
          - name: "generator"
            kind: "module"
            public: true
          - name: "hook"
            kind: "module"
            public: true
          - name: "init"
            kind: "module"
            public: true
          - name: "issues"
            kind: "module"
            public: true
          - name: "production"
            kind: "module"
            public: true
          - name: "project"
            kind: "module"
            public: true
          - name: "regenerability_policy"
            kind: "module"
            public: true
          - name: "remote_push"
            kind: "module"
            public: true
          - name: "run"
            kind: "module"
            public: true
          - name: "shell_env"
            kind: "module"
            public: true
          - name: "slug_workspace"
            kind: "module"
            public: true
          - name: "standardize"
            kind: "module"
            public: true
          - name: "sync"
            kind: "module"
            public: true
          - name: "td"
            kind: "module"
            public: true
          - name: "td_check_section_type"
            kind: "module"
            public: true
          - name: "td_lock"
            kind: "module"
            public: true
          - name: "td_migrate"
            kind: "module"
            public: true
          - name: "update"
            kind: "module"
            public: true
          - name: "validate_spec_structure"
            kind: "module"
            public: true
          - name: "workflow_guard"
            kind: "module"
            public: true
          - name: "migrate"
            kind: "module"
            public: true
          - name: "merge_target"
            kind: "module"
            public: true
          - name: "LEGACY_SCORE_WORKSPACE_DIR"
            kind: "constant"
            public: false
          - name: "legacy_score_workspace_error"
            kind: "function"
            public: false
          - name: "find_project_root"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/td.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "TdArgs"
            kind: "struct"
            public: true
          - name: "TdCommand"
            kind: "enum"
            public: true
          - name: "TdClaimArgs"
            kind: "struct"
            public: true
          - name: "AstArgs"
            kind: "struct"
            public: true
          - name: "CreateArgs"
            kind: "struct"
            public: true
          - name: "ValidateArgs"
            kind: "struct"
            public: true
          - name: "ReviewArgs"
            kind: "struct"
            public: true
          - name: "ReviseArgs"
            kind: "struct"
            public: true
          - name: "GenCodeArgs"
            kind: "struct"
            public: true
          - name: "CheckArgs"
            kind: "struct"
            public: true
          - name: "CodeCheckArgs"
            kind: "struct"
            public: true
          - name: "PromoteArgs"
            kind: "struct"
            public: true
          - name: "ArbitrateArgs"
            kind: "struct"
            public: true
          - name: "AuditGroupBy"
            kind: "enum"
            public: true
          - name: "AuditArgs"
            kind: "struct"
            public: true
          - name: "TdEnvelope"
            kind: "enum"
            public: false
          - name: "Invoke"
            kind: "struct"
            public: false
          - name: "print_envelope"
            kind: "function"
            public: false
          - name: "print_json_value"
            kind: "function"
            public: false
          - name: "next_dispatch"
            kind: "function"
            public: false
          - name: "next_none"
            kind: "function"
            public: false
          - name: "td_error"
            kind: "function"
            public: false
          - name: "td_workspace_path"
            kind: "function"
            public: true
          - name: "workflow_slug_for_issue"
            kind: "function"
            public: false
          - name: "should_use_td_branch"
            kind: "function"
            public: false
          - name: "td_branch_name"
            kind: "function"
            public: false
          - name: "activate_td_workspace_for_lifecycle"
            kind: "function"
            public: false
          - name: "td_activate_inplace_if_present"
            kind: "function"
            public: true
          - name: "td_activate_inplace_allowing_dirty_lifecycle_paths"
            kind: "function"
            public: true
          - name: "td_activate_inplace_allowing_dirty_spec_path"
            kind: "function"
            public: true
          - name: "canonical_issue_path_for_slug"
            kind: "function"
            public: false
          - name: "issue_path_arg"
            kind: "function"
            public: false
          - name: "ensure_clean_or_only_dirty_paths"
            kind: "function"
            public: false
          - name: "porcelain_path"
            kind: "function"
            public: false
          - name: "normalize_checkout_rel_path"
            kind: "function"
            public: false
          - name: "provision_td_workspace"
            kind: "function"
            public: false
          - name: "bootstrap_td_issue"
            kind: "function"
            public: false
          - name: "discover_worktree_spec"
            kind: "function"
            public: true
          - name: "commit_lifecycle"
            kind: "function"
            public: false
          - name: "stage_lifecycle_paths"
            kind: "function"
            public: false
          - name: "should_stage_lifecycle_path"
            kind: "function"
            public: false
          - name: "run_promote"
            kind: "function"
            public: false
          - name: "run_promote_at"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/project.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "ProjectHealthArgs"
            kind: "struct"
            public: true
          - name: "ProjectHealthSection"
            kind: "enum"
            public: true
          - name: "ProjectHealthReport"
            kind: "struct"
            public: true
          - name: "CbOwnershipSummary"
            kind: "struct"
            public: true
          - name: "CapabilityHealthReport"
            kind: "struct"
            public: true
          - name: "ready_fixture"
            kind: "function"
            public: false
          - name: "blocked"
            kind: "function"
            public: false
          - name: "RegenerabilityAuthorityReport"
            kind: "struct"
            public: true
          - name: "ProjectHealthStatus"
            kind: "enum"
            public: true
          - name: "ProjectTestGateReport"
            kind: "struct"
            public: true
          - name: "ProjectTestGateStatus"
            kind: "enum"
            public: true
          - name: "ProjectTestCommandReport"
            kind: "struct"
            public: true
          - name: "ProjectTestCommandStatus"
            kind: "enum"
            public: true
          - name: "ProjectEcGateReport"
            kind: "struct"
            public: true
          - name: "ProjectEcGateStatus"
            kind: "enum"
            public: true
          - name: "ProjectEcCommandReport"
            kind: "struct"
            public: true
          - name: "ProjectClaimClosureReport"
            kind: "struct"
            public: true
          - name: "ProjectClaimClosureItem"
            kind: "struct"
            public: true
          - name: "ProjectClaimClosureStatus"
            kind: "enum"
            public: true
          - name: "not_evaluated"
            kind: "function"
            public: true
          - name: "from_blocker"
            kind: "function"
            public: false
          - name: "not_evaluated"
            kind: "function"
            public: true
          - name: "from_check"
            kind: "function"
            public: false
          - name: "build_health_report"
            kind: "function"
            public: true
          - name: "build_health_report_with_options"
            kind: "function"
            public: true
          - name: "build_health_report_with_options_internal"
            kind: "function"
            public: false
          - name: "build_health_report_with_test_gates"
            kind: "function"
            public: true
          - name: "build_health_report_with_test_gates_and_capability_verified"
            kind: "function"
            public: true
          - name: "build_health_report_with_test_gates_and_capability_verified_internal"
            kind: "function"
            public: false
          - name: "resolve_health_project_name"
            kind: "function"
            public: false
          - name: "cb_verify_not_evaluated"
            kind: "function"
            public: false
          - name: "apply_scoped_production_readiness"
            kind: "function"
            public: false
          - name: "from_components"
            kind: "function"
            public: true
          - name: "from_components_with_traceability"
            kind: "function"
            public: true
          - name: "apply_preflight_gate_report"
            kind: "function"
            public: true
          - name: "refresh_takeover_readiness"
            kind: "function"
            public: false
          - name: "regenerability_authority_report"
            kind: "function"
            public: false
          - name: "regenerability_gap_count"
            kind: "function"
            public: false
          - name: "HealthProgressSink"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/td_lock.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "TdLockArgs"
            kind: "struct"
            public: true
          - name: "TdLockStatus"
            kind: "struct"
            public: true
          - name: "ready_fixture"
            kind: "function"
            public: true
          - name: "TdLockState"
            kind: "enum"
            public: true
          - name: "TdLockFile"
            kind: "struct"
            public: false
          - name: "TdLockEntry"
            kind: "struct"
            public: false
          - name: "TdLockTarget"
            kind: "struct"
            public: false
          - name: "TdLockConfig"
            kind: "struct"
            public: false
          - name: "TdLockProject"
            kind: "struct"
            public: false
          - name: "matches"
            kind: "function"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "check_project_td_lock"
            kind: "function"
            public: true
          - name: "write_project_td_lock"
            kind: "function"
            public: false
          - name: "check_project_td_lock_at_root"
            kind: "function"
            public: false
          - name: "status_from_parts"
            kind: "function"
            public: false
          - name: "stale_message"
            kind: "function"
            public: false
          - name: "print_status"
            kind: "function"
            public: false
          - name: "resolve_td_lock_target"
            kind: "function"
            public: false
          - name: "repo_relative_display"
            kind: "function"
            public: false
          - name: "TdSnapshot"
            kind: "struct"
            public: false
          - name: "snapshot_td_root"
            kind: "function"
            public: false
          - name: "collect_td_files"
            kind: "function"
            public: false
          - name: "digest_bytes"
            kind: "function"
            public: false
          - name: "root_digest"
            kind: "function"
            public: false
          - name: "diff_entries"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/validate_proposal.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ValidationSummary"
            kind: "struct"
            public: true
          - name: "is_valid"
            kind: "function"
            public: true
          - name: "is_valid_strict"
            kind: "function"
            public: true
          - name: "has_warnings"
            kind: "function"
            public: true
          - name: "to_json_output"
            kind: "function"
            public: true
          - name: "ErrorAccumulator"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
          - name: "process_result"
            kind: "function"
            public: false
          - name: "process_error"
            kind: "function"
            public: false
          - name: "process_errors_slice"
            kind: "function"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "validate_proposal"
            kind: "function"
            public: true
          - name: "print_error"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/init.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "SDD_VERSION"
            kind: "constant"
            public: false
          - name: "SKILL_CODEX_REVIEW"
            kind: "constant"
            public: false
          - name: "SKILL_GEMINI_EXPLORE_SPECS"
            kind: "constant"
            public: false
          - name: "SKILL_GEMINI_EXPLORE_CODEBASE"
            kind: "constant"
            public: false
          - name: "SKILL_MERGE"
            kind: "constant"
            public: false
          - name: "SKILL_CAPABILITY"
            kind: "constant"
            public: false
          - name: "SKILL_WI"
            kind: "constant"
            public: false
          - name: "SKILL_BUILD_DEBUG"
            kind: "constant"
            public: false
          - name: "SKILL_RELEASE_PATCH"
            kind: "constant"
            public: false
          - name: "SKILL_MAMBA_TEST_COVERAGE"
            kind: "constant"
            public: false
          - name: "SKILL_TD_CREATE"
            kind: "constant"
            public: false
          - name: "SKILL_CB_FILL"
            kind: "constant"
            public: false
          - name: "SKILL_CB_CLAIM"
            kind: "constant"
            public: false
          - name: "SKILL_STANDARDIZE"
            kind: "constant"
            public: false
          - name: "SKILL_BUILD_RELEASE"
            kind: "constant"
            public: false
          - name: "SKILL_CHAT_LISTEN"
            kind: "constant"
            public: false
          - name: "SKILL_HEALTH"
            kind: "constant"
            public: false
          - name: "SCRIPT_BUILD_RELEASE"
            kind: "constant"
            public: false
          - name: "SCRIPT_BUILD_DEBUG"
            kind: "constant"
            public: false
          - name: "SCRIPT_RELEASE_PATCH"
            kind: "constant"
            public: false
          - name: "SCRIPT_MAMBA_TEST_COVERAGE"
            kind: "constant"
            public: false
          - name: "SETTINGS_JSON_TEMPLATE"
            kind: "constant"
            public: false
          - name: "CLAUDE_TEMPLATE"
            kind: "constant"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "NewArgs"
            kind: "struct"
            public: true
          - name: "run_new"
            kind: "function"
            public: true
          - name: "NewProjectOutcome"
            kind: "struct"
            public: false
          - name: "run_new_with_current_dir"
            kind: "function"
            public: false
          - name: "resolve_new_target"
            kind: "function"
            public: false
          - name: "prepare_new_target"
            kind: "function"
            public: false
          - name: "is_directory_empty"
            kind: "function"
            public: false
          - name: "run_at_project_root"
            kind: "function"
            public: false
          - name: "Platform"
            kind: "enum"
            public: false
          - name: "AuthMethod"
            kind: "enum"
            public: false
          - name: "PlatformTomlUpdate"
            kind: "enum"
            public: false
          - name: "determine_platform_update"
            kind: "function"
            public: false
          - name: "determine_platform"
            kind: "function"
            public: false
          - name: "replace_toml_section"
            kind: "function"
            public: false
          - name: "apply_platform_update"
            kind: "function"
            public: false
          - name: "refresh_existing_config_content"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/regenerability_policy.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "RegenerabilityAuthority"
            kind: "enum"
            public: true
          - name: "RegenerabilityPolicy"
            kind: "struct"
            public: true
          - name: "required_for_production"
            kind: "function"
            public: true
          - name: "ConfigFile"
            kind: "struct"
            public: false
          - name: "ProjectRow"
            kind: "struct"
            public: false
          - name: "ProjectRegenerabilityConfig"
            kind: "struct"
            public: false
          - name: "resolve_regenerability_policy"
            kind: "function"
            public: true
          - name: "resolve_regenerability_policy_at"
            kind: "function"
            public: true
          - name: "default_regenerability_policy"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/td_migrate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "MigrateMermaidArgs"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "commit_mermaid_migration"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/td_check_section_type.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "REGISTRY_PROJECT"
            kind: "constant"
            public: false
          - name: "REGISTRY_PROJECT_REL_PATH"
            kind: "constant"
            public: false
          - name: "SEED_DEPRECATED"
            kind: "constant"
            public: false
          - name: "CheckArgs"
            kind: "struct"
            public: true
          - name: "FindingKind"
            kind: "enum"
            public: true
          - name: "Finding"
            kind: "struct"
            public: true
          - name: "Report"
            kind: "struct"
            public: true
          - name: "Registry"
            kind: "struct"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "print_human"
            kind: "function"
            public: false
          - name: "kind_label"
            kind: "function"
            public: false
          - name: "load_registry"
            kind: "function"
            public: false
          - name: "extract_yaml_block"
            kind: "function"
            public: false
          - name: "H2Section"
            kind: "struct"
            public: false
          - name: "scan_spec"
            kind: "function"
            public: false
          - name: "parse_annotation"
            kind: "function"
            public: false
          - name: "classify_section"
            kind: "function"
            public: false
          - name: "collect_specs"
            kind: "function"
            public: false
          - name: "walk"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/validate_spec_structure.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "ALLOWED_ROOT_FILES"
            kind: "constant"
            public: false
          - name: "ALLOWED_TOP_DIRS"
            kind: "constant"
            public: false
          - name: "Violation"
            kind: "struct"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "run_all"
            kind: "function"
            public: true
          - name: "discover_spec_roots"
            kind: "function"
            public: false
          - name: "validate_root"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/standardize.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "standardize_audit"
            kind: "module"
            public: false
          - name: "SOURCE_EXTS"
            kind: "constant"
            public: false
          - name: "PROJECT_CONTEXT_ARTIFACTS"
            kind: "constant"
            public: false
          - name: "RUST_BINARY_ARTIFACTS"
            kind: "constant"
            public: false
          - name: "EXCLUDED_DIRS"
            kind: "constant"
            public: false
          - name: "DELETED_COMMAND_PATHS"
            kind: "constant"
            public: false
          - name: "AW_EC_BEGIN_MARKER"
            kind: "constant"
            public: false
          - name: "TraceabilityCli"
            kind: "struct"
            public: false
          - name: "StandardizeArgs"
            kind: "struct"
            public: true
          - name: "StandardizeCommand"
            kind: "enum"
            public: true
          - name: "StandardizeAuditArgs"
            kind: "struct"
            public: true
          - name: "StandardizeAuditCommand"
            kind: "enum"
            public: true
          - name: "StandardizeAuditCheckArgs"
            kind: "struct"
            public: true
          - name: "StandardizeAuditRecordArgs"
            kind: "struct"
            public: true
          - name: "StandardizeStageArgs"
            kind: "struct"
            public: true
          - name: "StandardizeStageCommand"
            kind: "enum"
            public: true
          - name: "StandardizeReportArgs"
            kind: "struct"
            public: true
          - name: "StandardizeNextArgs"
            kind: "struct"
            public: true
          - name: "StandardizeRunArgs"
            kind: "struct"
            public: true
          - name: "StandardizeTraceabilityArgs"
            kind: "struct"
            public: true
          - name: "StandardizeTraceabilityCommand"
            kind: "enum"
            public: true
          - name: "StandardizeTraceabilityReportArgs"
            kind: "struct"
            public: true
          - name: "StandardizeTraceabilityRunArgs"
            kind: "struct"
            public: true
          - name: "StandardizationCoverage"
            kind: "struct"
            public: true
          - name: "MarkerCounts"
            kind: "struct"
            public: true
          - name: "CodegenCoverage"
            kind: "struct"
            public: true
          - name: "RegenerabilityCoverage"
            kind: "struct"
            public: true
          - name: "SemanticCoverage"
            kind: "struct"
            public: true
          - name: "TraceabilityCoverage"
            kind: "struct"
            public: true
          - name: "ready_fixture"
            kind: "function"
            public: true
          - name: "CommandTraceabilityCoverage"
            kind: "struct"
            public: true
          - name: "ready_fixture"
            kind: "function"
            public: true
          - name: "TraceabilityBlocker"
            kind: "struct"
            public: true
          - name: "TraceabilityBlockerKind"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "StackMigrationCoverage"
            kind: "struct"
            public: true
          - name: "ProjectHealthStandardizeCoverage"
            kind: "struct"
            public: true
          - name: "WorkspaceStackMigration"
            kind: "struct"
            public: true
          - name: "DependencyPolicyFinding"
            kind: "struct"
            public: true
          - name: "DeploymentFacetFinding"
            kind: "struct"
            public: true
          - name: "PromoteOutcome"
            kind: "struct"
            public: false
          - name: "resolve_promote_target"
            kind: "function"
            public: false
          - name: "promote_handwrite_marker_to_codegen"
            kind: "function"
            public: false
          - name: "gap_issue_title"
            kind: "function"
            public: false
          - name: "gap_issue_create_args"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/chat.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "chat_members"
            kind: "module"
            public: true
          - name: "CHANNEL_PATH"
            kind: "constant"
            public: false
          - name: "MEMBERS_PATH"
            kind: "constant"
            public: false
          - name: "ChatArgs"
            kind: "struct"
            public: true
          - name: "ChatCommand"
            kind: "enum"
            public: true
          - name: "PostArgs"
            kind: "struct"
            public: true
          - name: "ListArgs"
            kind: "struct"
            public: true
          - name: "ReadArgs"
            kind: "struct"
            public: true
          - name: "MembersArgs"
            kind: "struct"
            public: true
          - name: "ListenArgs"
            kind: "struct"
            public: true
          - name: "AgentLastSeen"
            kind: "struct"
            public: true
          - name: "ListenState"
            kind: "type"
            public: true
          - name: "OutputFormat"
            kind: "enum"
            public: false
          - name: "run_chat"
            kind: "function"
            public: true
          - name: "detect_output_format"
            kind: "function"
            public: false
          - name: "parse_channel"
            kind: "function"
            public: false
          - name: "format_terse"
            kind: "function"
            public: false
          - name: "format_listen"
            kind: "function"
            public: false
          - name: "format_human"
            kind: "function"
            public: false
          - name: "render"
            kind: "function"
            public: false
          - name: "run_post"
            kind: "function"
            public: false
          - name: "run_list"
            kind: "function"
            public: false
          - name: "run_read"
            kind: "function"
            public: false
          - name: "run_members"
            kind: "function"
            public: false
          - name: "run_members_list"
            kind: "function"
            public: false
          - name: "TailGuard"
            kind: "struct"
            public: false
          - name: "drop"
            kind: "function"
            public: false
          - name: "run_listen"
            kind: "function"
            public: false
          - name: "should_emit"
            kind: "function"
            public: false
          - name: "thread_root_of"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/generator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "GeneratorArgs"
            kind: "struct"
            public: true
          - name: "GeneratorCommand"
            kind: "enum"
            public: true
          - name: "GeneratorCheckArgs"
            kind: "struct"
            public: true
          - name: "GeneratorRequestArgs"
            kind: "struct"
            public: true
          - name: "GeneratorGap"
            kind: "struct"
            public: true
          - name: "GeneratorHealthSummary"
            kind: "struct"
            public: true
          - name: "GeneratorNextAction"
            kind: "struct"
            public: true
          - name: "GeneratorCheckReport"
            kind: "struct"
            public: true
          - name: "GeneratorRequestReport"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "run_check"
            kind: "function"
            public: false
          - name: "run_request"
            kind: "function"
            public: false
          - name: "generator_health_report"
            kind: "function"
            public: false
          - name: "build_check_report"
            kind: "function"
            public: false
          - name: "build_request_report"
            kind: "function"
            public: false
          - name: "generator_gaps"
            kind: "function"
            public: false
          - name: "takeover_blockers"
            kind: "function"
            public: false
          - name: "from"
            kind: "function"
            public: false
          - name: "generator_request_payload_path"
            kind: "function"
            public: false
          - name: "write_request_payload"
            kind: "function"
            public: false
          - name: "print_json"
            kind: "function"
            public: false
          - name: "slug_for_path"
            kind: "function"
            public: false
          - name: "shell_quote"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/capability.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "CAPABILITY_MIGRATION_INSERT_MARKER"
            kind: "constant"
            public: false
          - name: "CAPABILITY_GATE_TIMEOUT_ENV"
            kind: "constant"
            public: false
          - name: "DEFAULT_CAPABILITY_GATE_TIMEOUT_SECS"
            kind: "constant"
            public: false
          - name: "CapabilityArgs"
            kind: "struct"
            public: true
          - name: "CapabilityCommand"
            kind: "enum"
            public: true
          - name: "CapabilityReportArgs"
            kind: "struct"
            public: true
          - name: "CapabilityNextArgs"
            kind: "struct"
            public: true
          - name: "CapabilityDraftArgs"
            kind: "struct"
            public: true
          - name: "CapabilityApplyDraftArgs"
            kind: "struct"
            public: true
          - name: "CapabilityRunArgs"
            kind: "struct"
            public: true
          - name: "CapabilityMigrateArgs"
            kind: "struct"
            public: true
          - name: "CapabilityCheckArgs"
            kind: "struct"
            public: true
          - name: "CapabilityInitArgs"
            kind: "struct"
            public: true
          - name: "CapabilitySweepArgs"
            kind: "struct"
            public: true
          - name: "CapabilitySetTypeArgs"
            kind: "struct"
            public: true
          - name: "CapabilitySetStatusArgs"
            kind: "struct"
            public: true
          - name: "CapabilitySetSurfaceArgs"
            kind: "struct"
            public: true
          - name: "CapabilitySetEcDimensionArgs"
            kind: "struct"
            public: true
          - name: "CapabilitySetWiRefArgs"
            kind: "struct"
            public: true
          - name: "CapabilityStatus"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "CapabilityGapStatus"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "CapabilityRefRole"
            kind: "enum"
            public: true
          - name: "CapabilityCoverage"
            kind: "enum"
            public: true
          - name: "CapabilityMaturity"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "CapabilityGap"
            kind: "struct"
            public: true
          - name: "CapabilityIndexSummary"
            kind: "struct"
            public: true
          - name: "CapabilityWorkRoot"
            kind: "struct"
            public: true
          - name: "CapabilityVerification"
            kind: "struct"
            public: true
          - name: "CapabilityClaimGate"
            kind: "struct"
            public: true
          - name: "default_required_for_verified"
            kind: "function"
            public: false
          - name: "is_false"
            kind: "function"
            public: false
          - name: "CapabilityClaim"
            kind: "struct"
            public: true
          - name: "CapabilityVerificationContract"
            kind: "struct"
            public: true
          - name: "CapabilitySurface"
            kind: "struct"
            public: true
          - name: "CapabilityEcDimensionKind"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "CapabilityEfficiencyBackfillSlot"
            kind: "struct"
            public: true
          - name: "CapabilityEcDimension"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/chain.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "ChainBlockerKind"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "ChainBlocker"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: false
          - name: "fmt"
            kind: "function"
            public: false
          - name: "ChainRequiredPositional"
            kind: "struct"
            public: false
          - name: "CHAIN_REQUIRED_POSITIONALS"
            kind: "constant"
            public: false
          - name: "validate_aw_command_string"
            kind: "function"
            public: true
          - name: "check_chain_required_positionals"
            kind: "function"
            public: false
          - name: "descend_subcommand"
            kind: "function"
            public: false
          - name: "EmitSite"
            kind: "struct"
            public: false
          - name: "EMIT_REGISTRY"
            kind: "constant"
            public: false
          - name: "LegacyNextActionRule"
            kind: "struct"
            public: false
          - name: "LEGACY_NEXT_ACTION_RULES"
            kind: "constant"
            public: false
          - name: "VatRunnerEntry"
            kind: "struct"
            public: false
          - name: "VatRunnersFile"
            kind: "struct"
            public: false
          - name: "VatRunnerInvocation"
            kind: "struct"
            public: false
          - name: "parse_vat_runner_invocation"
            kind: "function"
            public: false
          - name: "check_ec_vat_runner_binding"
            kind: "function"
            public: true
          - name: "normalize_legacy_aw_run_command"
            kind: "function"
            public: false
          - name: "normalize_legacy_next_action"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/guard.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "CODEX_HOOKS_REL"
            kind: "constant"
            public: false
          - name: "CLAUDE_SETTINGS_REL"
            kind: "constant"
            public: false
          - name: "CODEX_MATCHER"
            kind: "constant"
            public: false
          - name: "CLAUDE_MATCHER"
            kind: "constant"
            public: false
          - name: "GuardArgs"
            kind: "struct"
            public: true
          - name: "GuardCommand"
            kind: "enum"
            public: true
          - name: "GuardToggleArgs"
            kind: "struct"
            public: true
          - name: "GuardPretoolArgs"
            kind: "struct"
            public: true
          - name: "GuardAgent"
            kind: "enum"
            public: true
          - name: "includes_codex"
            kind: "function"
            public: false
          - name: "includes_claude"
            kind: "function"
            public: false
          - name: "GuardHookChange"
            kind: "struct"
            public: false
          - name: "GuardDecision"
            kind: "enum"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "run_on"
            kind: "function"
            public: false
          - name: "run_off"
            kind: "function"
            public: false
          - name: "run_pretool"
            kind: "function"
            public: false
          - name: "emit_toggle_summary"
            kind: "function"
            public: false
          - name: "install_guard_hooks"
            kind: "function"
            public: false
          - name: "remove_guard_hooks"
            kind: "function"
            public: false
          - name: "guard_command"
            kind: "function"
            public: false
          - name: "upsert_hook_file"
            kind: "function"
            public: false
          - name: "remove_hook_from_file"
            kind: "function"
            public: false
          - name: "command_project"
            kind: "function"
            public: false
          - name: "read_json_or_empty_object"
            kind: "function"
            public: false
          - name: "pretty_json"
            kind: "function"
            public: false
          - name: "write_json_if_changed"
            kind: "function"
            public: false
          - name: "aw_guard_handler"
            kind: "function"
            public: false
          - name: "append_pretool_handler"
            kind: "function"
            public: false
          - name: "ensure_object"
            kind: "function"
            public: false
          - name: "ensure_child_object"
            kind: "function"
            public: false
          - name: "ensure_child_array"
            kind: "function"
            public: false
          - name: "remove_aw_guard_handlers"
            kind: "function"
            public: false
          - name: "is_aw_guard_handler"
            kind: "function"
            public: false
          - name: "decide_pretool_payload"
            kind: "function"
            public: false
          - name: "extract_target_paths"
            kind: "function"
            public: false
          - name: "parse_apply_patch_targets"
            kind: "function"
            public: false
          - name: "GuardScope"
            kind: "struct"
            public: false
          - name: "for_project"
            kind: "function"
            public: false
          - name: "contains"
            kind: "function"
            public: false
          - name: "guard_prefixes_from_row"
            kind: "function"
            public: false
          - name: "target_to_repo_rel"
            kind: "function"
            public: false
          - name: "resolve_existing_prefix"
            kind: "function"
            public: false
          - name: "lexical_normalize"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/llm.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "LlmTopic"
            kind: "enum"
            public: true
          - name: "LlmFormat"
            kind: "enum"
            public: true
          - name: "LlmArgs"
            kind: "struct"
            public: true
          - name: "TOPICS"
            kind: "constant"
            public: false
          - name: "run"
            kind: "function"
            public: true
          - name: "cli_std_format"
            kind: "function"
            public: false
          - name: "topic_name"
            kind: "function"
            public: false
          - name: "registered_verbs"
            kind: "function"
            public: false
          - name: "CAPABILITY_MD"
            kind: "constant"
            public: false
          - name: "TD_MD"
            kind: "constant"
            public: false
          - name: "EC_MD"
            kind: "constant"
            public: false
          - name: "WI_MD"
            kind: "constant"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/loop_state.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "LOOP_START"
            kind: "constant"
            public: false
          - name: "LOOP_END"
            kind: "constant"
            public: false
          - name: "LastResult"
            kind: "enum"
            public: true
          - name: "LoopStatus"
            kind: "enum"
            public: true
          - name: "Iteration"
            kind: "struct"
            public: true
          - name: "LoopState"
            kind: "struct"
            public: true
          - name: "parse_loop_state"
            kind: "function"
            public: true
          - name: "upsert_loop_state"
            kind: "function"
            public: true
          - name: "apply_verification"
            kind: "function"
            public: true
          - name: "decide_next_action"
            kind: "function"
            public: true
          - name: "record_verification"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/standard_cli.rs"
        language: "rust"
        ownership_state: "unmanaged"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "TOOL"
            kind: "constant"
            public: false
          - name: "ISSUE_TOOL"
            kind: "constant"
            public: false
          - name: "UpgradeArgs"
            kind: "struct"
            public: true
          - name: "ReportIssueArgs"
            kind: "struct"
            public: true
          - name: "IssueArgs"
            kind: "struct"
            public: true
          - name: "IssueCommand"
            kind: "enum"
            public: true
          - name: "IssueSearchArgs"
            kind: "struct"
            public: true
          - name: "IssueViewArgs"
            kind: "struct"
            public: true
          - name: "IssueCreateArgs"
            kind: "struct"
            public: true
          - name: "run_upgrade"
            kind: "function"
            public: true
          - name: "run_report_issue"
            kind: "function"
            public: true
          - name: "run_issue"
            kind: "function"
            public: true
          - name: "report_issue_labels"
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
          domain: "apps/agentic-workflow/src/cli"
      - path: "apps/agentic-workflow/src/cli/view.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "ViewArgs"
            kind: "struct"
            public: true
          - name: "ViewLayout"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "toggled_view_layout"
            kind: "function"
            public: true
          - name: "layout_toggle_button_label"
            kind: "function"
            public: true
          - name: "RepoViewSnapshot"
            kind: "struct"
            public: true
          - name: "RepoViewRepo"
            kind: "struct"
            public: true
          - name: "RepoCatalogItem"
            kind: "struct"
            public: true
          - name: "UserRepoRegistry"
            kind: "struct"
            public: false
          - name: "UserRepoRegistryEntry"
            kind: "struct"
            public: false
          - name: "TerminalSnapshot"
            kind: "struct"
            public: true
          - name: "RepoViewItemSnapshot"
            kind: "struct"
            public: true
          - name: "ProjectViewProject"
            kind: "struct"
            public: true
          - name: "ProjectCatalogItem"
            kind: "struct"
            public: true
          - name: "ReadmeSnapshot"
            kind: "struct"
            public: true
          - name: "CapabilitySnapshot"
            kind: "struct"
            public: true
          - name: "CapabilitySnapshotItem"
            kind: "struct"
            public: true
          - name: "EcSnapshot"
            kind: "struct"
            public: true
          - name: "EcCaseSnapshot"
            kind: "struct"
            public: true
          - name: "TdSnapshot"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "build_repo_view_snapshot"
            kind: "function"
            public: true
          - name: "build_repo_view_snapshot_with_repo_registry_path"
            kind: "function"
            public: false
          - name: "build_repo_view_item_snapshot"
            kind: "function"
            public: false
          - name: "empty_repo_view_item_snapshot"
            kind: "function"
            public: false
          - name: "select_catalog_item"
            kind: "function"
            public: false
          - name: "project_view_project"
            kind: "function"
            public: false
          - name: "project_catalog"
            kind: "function"
            public: false
          - name: "project_kind"
            kind: "function"
            public: false
          - name: "user_repo_registry_path"
            kind: "function"
            public: false
          - name: "load_or_update_repo_catalog"
            kind: "function"
            public: false
          - name: "read_user_repo_registry"
            kind: "function"
            public: false
          - name: "write_user_repo_registry"
            kind: "function"
            public: false
          - name: "upsert_user_repo_entry"
            kind: "function"
            public: false
          - name: "repo_catalog_item_from_entry"
            kind: "function"
            public: false
          - name: "repo_counts_for_path"
            kind: "function"
            public: false
          - name: "canonical_repo_path"
            kind: "function"
            public: false
          - name: "load_ec_snapshot"
            kind: "function"
            public: false
          - name: "empty_ec_snapshot"
            kind: "function"
            public: false
          - name: "td_snapshot"
            kind: "function"
            public: false
          - name: "capability_snapshot"
            kind: "function"
            public: false
          - name: "build_terminal_snapshot"
            kind: "function"
            public: false
          - name: "build_surface_snapshot"
            kind: "function"
            public: false
          - name: "RepoSurfaceProps"
            kind: "struct"
            public: false
          - name: "render_repo_surface"
            kind: "function"
            public: false
          - name: "build_surface_element"
            kind: "function"
            public: false
          - name: "headless_contract_check"
            kind: "function"
            public: true
          - name: "selected_item"
            kind: "function"
            public: false
          - name: "extract_h1"
            kind: "function"
            public: false
          - name: "extract_brief"
            kind: "function"
            public: false
          - name: "relative_to"
            kind: "function"
            public: false
          - name: "APP_SCREENSHOT_WIDTH"
            kind: "constant"
            public: true
          - name: "APP_SCREENSHOT_HEIGHT"
            kind: "constant"
            public: true
          - name: "APP_SCREENSHOT_FONT_CANDIDATES"
            kind: "constant"
            public: false
          - name: "render_app_screenshot_image"
            kind: "function"
            public: true
          - name: "render_app_screenshot_image_at_scale"
            kind: "function"
            public: true
          - name: "render_app_screenshot_image_at_scale_with_options"
            kind: "function"
            public: false
          - name: "AppScreenshotPaintOptions"
            kind: "struct"
            public: false
          - name: "full"
            kind: "function"
            public: false
          - name: "native_backdrop"
            kind: "function"
            public: false
          - name: "normalized_screenshot_scale"
            kind: "function"
            public: false
          - name: "scaled_screenshot_dimension"
            kind: "function"
            public: false
          - name: "load_app_screenshot_font"
            kind: "function"
            public: false
          - name: "ScaledScreenshotPainter"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
          - name: "px"
            kind: "function"
            public: false
          - name: "fill_rect"
            kind: "function"
            public: false
          - name: "draw_text_line"
            kind: "function"
            public: false
          - name: "measure_text_width"
            kind: "function"
            public: false
          - name: "FontdueScreenshotPainter"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
          - name: "into_image"
            kind: "function"
            public: false
          - name: "fill_rect"
            kind: "function"
            public: false
          - name: "draw_text_line"
            kind: "function"
            public: false
          - name: "measure_text_width"
            kind: "function"
            public: false
          - name: "macos_screenshot"
            kind: "module"
            public: false
          - name: "render_app_screenshot_png"
            kind: "function"
            public: true
          - name: "render_app_screenshot_png_at_scale"
            kind: "function"
            public: true
          - name: "render_native_app_backdrop_png_at_scale"
            kind: "function"
            public: true
          - name: "render_app_screenshot"
            kind: "function"
            public: true
          - name: "build_desktop_app_bundle"
            kind: "function"
            public: true
          - name: "build_macos_app_bundle"
            kind: "function"
            public: false
          - name: "macos_app_info_plist"
            kind: "function"
            public: false
          - name: "shell_quote"
            kind: "function"
            public: false
          - name: "paint_repo_view_screenshot"
            kind: "function"
            public: false
          - name: "paint_layout_toggle_button"
            kind: "function"
            public: false
          - name: "DetailPaintDensity"
            kind: "struct"
            public: false
          - name: "standard"
            kind: "function"
            public: false
          - name: "compact"
            kind: "function"
            public: false
          - name: "paint_detail_panel"
            kind: "function"
            public: false
          - name: "paint_catalog"
            kind: "function"
            public: false
          - name: "paint_project_selector"
            kind: "function"
            public: false
          - name: "paint_terminal"
            kind: "function"
            public: false
          - name: "paint_detail"
            kind: "function"
            public: false
          - name: "paint_stat_card"
            kind: "function"
            public: false
          - name: "paint_list_panel"
            kind: "function"
            public: false
          - name: "draw_wrapped_text"
            kind: "function"
            public: false
          - name: "truncate_for_width"
            kind: "function"
            public: false
          - name: "estimate_fontdue_text_width"
            kind: "function"
            public: false
          - name: "draw_fontdue_text_line"
            kind: "function"
            public: false
          - name: "fill_rect_pixels"
            kind: "function"
            public: false
          - name: "blend_pixel"
            kind: "function"
            public: false
          - name: "rgba"
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
          domain: "apps/agentic-workflow/src/cli"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/agentic-workflow/src/cli/hook.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/proposal.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/shell_env.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/sdd.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/fillback.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/cb_fill.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/ec.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
      #921 tier 1b (epic #914 slice G): `EcProjectContext` gained an
      `ec_bindings: BTreeMap<String, EcBinding>` field (populated in
      `resolve_ec_project_context` from the already-loaded `Project.ec` map,
      no new project-loading logic), and `EcCheckSummary` gained an
      `ec_binding_warnings: Vec<String>` field (never affects `clean`).
      `check_manifest_against_expected` now calls
      `chain::check_ec_vat_runner_binding` once per `ec.*` binding before its
      final sort/dedup pass, folding any blocker into the existing
      `findings` (so a misspelled vat.toml runner id blocks `clean` like any
      other finding) and collecting warn-only findings (e.g. an
      as-yet-unbuilt runner binary) into `ec_binding_warnings`.
      `run_check`'s non-JSON branch prints each warning to stderr regardless
      of `clean`. See `chain.md#changes` for the validator implementation.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/tasks.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/merge_target.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/cb.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/chat_members.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability_type.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/issues.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/conf.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/update.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/remote_push.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/standardize_audit.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/check_alignment.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/run.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
      #860 cleanup (epic #914 dead-code pass, orphans called out in #918/#920
      follow-up comments): removed the `ResolvedRunRoot::Project` variant and
      its match arms (the `aw run --project` root-rollup path that
      `aw capability run --project` now owns via `capability.rs`'s own
      `run_capability_tick`), and deleted the dead
      `RunProjectConfig::canonical_project_name` method plus its direct test.
      Deleted 5 fully-dead functions with zero callers anywhere
      (`project_envelope`,
      `project_done_or_dirty_envelope_with_capability_report`,
      `project_production_blocked_from_health_report`,
      `project_backlog_envelope`, `completion_missing_from_capability_action`).
      The remaining lower-level helpers under that superseded tree
      (`project_completion`, `project_done_envelope`,
      `persistence_blocked_envelope`, `project_repo_side_dirty_paths_at`,
      `project_repo_side_scopes`, `commit_project_persistence_if_approved`,
      `project_persistence_request_path`, `stable_project_root_hash`,
      `write_project_persistence_request`, `load_run_project_config`,
      `scope_strings`, `ProjectPersistenceRequest`, `RunProjectConfig`,
      `RunProjectRow`, `project_ready_wi_envelope`,
      `project_atomize_backlog_envelope`,
      `project_prioritize_blocked_envelope`) lost their only production
      caller but retain direct `#[test]` or test-twin
      (`project_done_or_dirty_envelope_with_health`/
      `project_production_blocked_envelope`) coverage, so they are now
      `#[cfg(test)]`-gated rather than deleted; the `wi_cli`/`Deserialize`/
      `DefaultHasher`/`Hash`/`Hasher` imports they depend on are
      `#[cfg(test)]`-gated to match. Zero-warning plain `cargo build`.
      #1268: `wi_envelope`'s non-epic, non-loop-state branch unconditionally
      dispatched `aw td create <id>` regardless of the WI's actual tracker
      phase, so `aw wi run` re-emitted `td create` for a WI already past
      `td_created` — a command `aw td create` then rejects
      ("expected td_inited"). Added `wi_change_lifecycle_step(&Issue)`,
      which routes off `issue.phase` (already normalized on read by the
      issue backend, see `crate::issues::types::td_phase::normalize`)
      through the same phase table `capability::lifecycle_action_for_work_item`
      uses for `aw capability run` (#916): `td_created` -> `aw td gen`,
      `cb_genned` -> `aw td fill`, `cb_filled` / `td_merged` (resumable
      retry) -> `aw td code-check`; `None`/`td_inited`/unrecognized phases
      keep the original `aw td create` catch-all. Also corrected the
      catch-all's reason string, which still said "WI -> TD -> CB -> TD
      merge lifecycle" after the merge step was removed (#842-#860), to
      "WI -> TD -> CB -> code-check lifecycle".
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/production.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/workflow_guard.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/commands.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/migrate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/slug_workspace.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/td.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/project.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/td_lock.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/validate_proposal.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/cb_revise.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/init.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/regenerability_policy.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/td_migrate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/td_check_section_type.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/validate_spec_structure.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/cb_arbitrate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/standardize.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/chat.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/generator.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/cb_review.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
      Issue #819: added `aw capability set-wi-ref --project <p> --capability
      <cap-id> --claim <claim-id> --wi <n>` (repeatable `--wi`) to the set-*
      contract-field-setter family (alongside set-type/set-status/
      set-surface/set-ec-dimension). Claim ids are the README work-root
      table's row ids (`slugify(work_root)`, the same id space as
      `CapabilityGap.id`/`CapabilityClaim.id`), so the verb resolves
      `--claim` against a capability's work-root rows and rewrites that
      row's `WI` cell -- not a separate claim-level ref surface, since none
      exists. `--wi` accepts `#<n>` or bare `<n>`, normalized to the table's
      existing `#<n>` format; multiple `--wi` values join with `, ` for a
      claim row that tracks more than one reference. Unknown capability/claim
      ids fail closed via `resolve_capability_and_claim_ids` with the valid id
      list from a fresh `parse_capability_document` scan; a successful edit is
      re-parsed and the row's `wi` field reconfirmed before the file is
      written, so the verb never persists a table it cannot itself read back.
      `choose_next_action`'s `reconcile_wi_refs` branch
      (`lifecycle_action_for_work_item`'s unresolved-issue-evidence path) now
      also names this verb in its `reason` text, so `aw wi plan` is no longer
      the sole remediation surfaced for a stale Active WI reference (project
      jet's `wasm-multi-target-readiness` claim, #818).
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1077 (traits slice 1/3): `baseline_caps_for_trait` becomes a
      thin lookup over the new `crate::cli::doc_mirror::TRAITS` const table
      (home of the archetype-anchored `TraitDef` registry) for the three
      traits with a settled CONTRIBUTING.md doc home (`http2_api`,
      `kubernetes_native`, `primary_replicas`), falling back to the
      remaining known traits' baseline caps in the new private helper
      `other_known_trait_baseline_caps` (`cli_facing`,
      `competitive_replacement`, `long_running`, `network_exposed`,
      `agent_facing`, `stateful_storage`) -- behavior-preserving, same
      signature, same call sites. `known_capability_profile_traits()` and
      `required_baseline_caps_for_traits` are unchanged (kept as the
      existing static 9-trait list so trait-iteration order used by other
      tests does not shift); a new unit test
      `known_traits_include_every_doc_mirror_trait_def` asserts the two
      registries can never drift apart (every `doc_mirror::TRAITS` entry is
      in `known_capability_profile_traits()` and its baseline caps match).
      `cargo test -p agentic-workflow --lib cli::capability::` stays fully
      green (144 tests) -- AC1.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1078 (traits slice 2/3): deleted the private
      `other_known_trait_baseline_caps` fallback added by #1077 -- the six
      general traits it covered (`cli_facing`, `competitive_replacement`,
      `long_running`, `network_exposed`, `agent_facing`, `stateful_storage`)
      are now first-class `doc_mirror::TRAITS` entries with
      `contributing_anchor: None`, so `baseline_caps_for_trait` becomes a
      pure lookup over that one registry (no fallback branch left).
      `required_baseline_caps_for_traits` now calls the new
      `doc_mirror::expand_capability_profile_traits(traits)` first (expanding
      the `service` umbrella into its members, deduped via `BTreeSet`) before
      deriving caps, so declaring `service` derives the full baseline set of
      its members and umbrella+member double-declaration does not duplicate.
      `known_capability_profile_traits()` grows from 9 to 14 ids (the four
      new anchored traits `standard_endpoints`/`ec_gated`/`cli_std`/
      `chainable_output`, plus the `service` umbrella itself, added to the
      existing 9). `baseline_capability_title` gained match arms for the
      four new anchored traits' baseline cap ids
      (`standard-operational-endpoints`, `ec-gates-configured`,
      `cli-standard-surface`, `chainable-output-conformance`).
      `render_trait_baseline_caps_cell` gained an umbrella branch: when
      `trait_id` matches a `doc_mirror::TRAIT_EXPANSIONS` entry, it renders
      `"expands: {members} -> caps: {derived}"` (derived via a nested call to
      `required_baseline_caps_for_traits` on the member list) instead of a
      plain baseline-cap lookup, so the umbrella's expansion is visible in
      profile/draft rendering, not just in the derived cap list. The
      drift-guard test `known_traits_include_every_doc_mirror_trait_def`
      gained a second loop asserting every `TRAIT_EXPANSIONS` id is known and
      every member id is a real `doc_mirror::TRAITS` id. Two new fixture
      tests, `project_declaring_service_umbrella_derives_full_deduped_
      baseline_set` and `project_declaring_service_umbrella_and_a_member_
      does_not_duplicate`, cover AC1 directly: a project declaring
      `traits = ["service"]` derives the full deduped 6-member baseline set,
      and declaring `["service", "http2_api", "primary_replicas"]` alongside
      it adds only `primary-replicas` on top with no duplicate
      `http2-api-list` entry.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/cb_revise.rs"
    action: delete
    section: schema
    description: |
      Issue #848: removed the `source_units` evidence block for this file
      (with cb_arbitrate.rs and cb_review.rs below) -- all three files were
      deleted from `src/cli/` by the #860 terminal-lifecycle cleanup and no
      longer exist on disk, so their evidence entries were stale/false. The
      prior entries for these three paths (above, kept as history) predate
      that deletion.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/cb_arbitrate.rs"
    action: delete
    section: schema
    description: |
      Issue #848: see the cb_revise.rs entry immediately above -- same
      removal, same reason.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/cb_review.rs"
    action: delete
    section: schema
    description: |
      Issue #848: see the cb_revise.rs entry above -- same removal, same
      reason.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/chain.rs"
    action: add
    section: schema
    description: |
      Issue #848: added the `source_units` evidence block for chain.rs
      (added by #915's emit-site next-action registry and legacy-command
      normalization), which had never been captured in this semantic
      domain's evidence list.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/guard.rs"
    action: add
    section: schema
    description: |
      Issue #848: added the `source_units` evidence block for guard.rs
      (the live `aw guard` hook-installation/pre-tool-policy surface),
      missing from this semantic domain's evidence list.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/llm.rs"
    action: add
    section: schema
    description: |
      Issue #848: added the `source_units` evidence block for llm.rs (the
      live `aw llm` offline agent-orientation surface), missing from this
      semantic domain's evidence list.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/loop_state.rs"
    action: add
    section: schema
    description: |
      Issue #848: added the `source_units` evidence block for loop_state.rs
      (the WorkItem loop-state model), missing from this semantic domain's
      evidence list.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/standard_cli.rs"
    action: add
    section: schema
    description: |
      Issue #848: added the `source_units` evidence block for
      standard_cli.rs (the shared `llm`/`upgrade`/`issue` CLI-convention
      surface), missing from this semantic domain's evidence list.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/view.rs"
    action: add
    section: schema
    description: |
      Issue #848: added the `source_units` evidence block for view.rs (the
      read-only repo visual-reader snapshot and desktop app surface),
      missing from this semantic domain's evidence list.
    impl_mode: hand-written
```
