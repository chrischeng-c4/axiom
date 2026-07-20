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
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: agent-first-cli-product-model
    claim: agent-first-cli-product-model
    coverage: full
    rationale: "The CLI semantic domain owns binary product orientation and the regression that keeps README, CAPABILITIES, and canonical product TDs aligned on the single agent-first CLI model."
  - id: existing-project-standardization
    role: primary
    gap: force-regeneration-project-root-llms-projection
    claim: force-regeneration-project-root-llms-projection
    coverage: full
    rationale: "The public force-regeneration command uses the shared TD-first runner and project_root_llms emitter after source application, so one canonical CODEGEN context replaces generic fallback output while HANDWRITE siblings remain untouched."
  - id: "aw-core-client-model-workitem-first-artifact-lifecycle"
    role: primary
    gap: "aw-epic-project-label-dispatch"
    claim: "aw-epic-project-label-dispatch"
    coverage: full
    rationale: "The CLI semantic domain owns run.rs project-label resolution, epic atomize dispatch, and the unresolved-label HITL envelope."
  - id: workflow-root-runner
    role: primary
    gap: self-hosting-root-runner-policy
    claim: self-hosting-root-runner-policy
    coverage: full
    rationale: "Runner admission rejects Agentic Workflow's own project, capability, and WI roots before mutation, while self-health exposes the sanctioned direct-commit gate partition."
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
  - id: project-local-td-and-ec-gates
    role: primary
    gap: stale-ec-inventory-verifier-fail-closed-routing
    claim: stale-ec-inventory-verifier-fail-closed-routing
    coverage: full
    rationale: "The EC verifier refuses stale or unreviewed executable inventories before command execution and persists independent review/regeneration as the bounded WI's runnable next action."
  - id: project-local-td-and-ec-gates
    role: primary
    gap: ec-needs-revision-existing-source-routing
    claim: ec-needs-revision-existing-source-routing
    coverage: full
    rationale: "Needs-revision review evidence can route only to an existing canonical EC Markdown source, including the legacy TD source fallback, so every emitted ec fill command is executable."
  - id: project-local-td-and-ec-gates
    role: primary
    gap: ec-gen-post-implementation-phase-aware-handoff
    claim: ec-gen-post-implementation-phase-aware-handoff
    coverage: full
    rationale: "EC regeneration preserves the fresh EC-first handoff to TD authoring but routes a cb_filled WI to execution of the newly current verifier instead of rewinding to an invalid td create command."
  - id: project-local-td-and-ec-gates
    role: primary
    gap: root-owned-ec-production-required-verdict
    claim: root-owned-ec-production-required-verdict
    coverage: full
    rationale: "WI and capability roots record their EC verdict from the production-required case set used by terminal code-check, while unscoped manual verification can still exercise advisory cases."
  - id: project-local-td-and-ec-gates
    role: primary
    gap: project-label-producer-td-routing
    claim: project-label-producer-td-routing
    coverage: full
    rationale: "The WI --project producer receives path-canonical app/lib labels from registered rows before the default TD resolver accepts them; a raw retired project label remains invalid at the TD boundary."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-apply-section-lookup-parity
    claim: td-apply-section-lookup-parity
    coverage: full
    rationale: "The CLI semantic domain owns generic TD payload normalization, typed single-section merge boundaries, and pre-write applicability validation in src/cli/td.rs."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-merged-candidate-in-memory-validation
    claim: td-merged-candidate-in-memory-validation
    coverage: full
    rationale: "The CLI semantic domain explicitly selects candidate-backed full-registry validation before section writes and file-backed validation for completed on-disk specs."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: default-td-target-plan-queue
    claim: default-td-target-plan-queue
    coverage: full
    rationale: "Fresh applicability and contract authoring queues initialize and projection-lock an editable Changes target plan between Logic and Unit Test, preserving explicit custom queues and supplying aw td gen with authoritative targets."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: rebased-td-lifecycle-recovery
    claim: rebased-td-lifecycle-recovery
    coverage: full
    rationale: "TD create shares CB's exact reachable Td-Init lookup, safely resets stale authoring state and projection after rewritten history, and preserves valid resume plus fail-closed implementation evidence."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: committed-td-skeleton-lifecycle
    claim: committed-td-skeleton-lifecycle
    coverage: full
    rationale: "TD create admits only its sole exact untracked known-empty skeleton, preserves that candidate through activation and rebased reset/provision, and stages the canonical skeleton in exactly one queue-start commit."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: generated-td-lock-commit-handoff
    claim: generated-td-lock-commit-handoff
    coverage: full
    rationale: "TD lock preflights lexical and canonical repository containment before any write, then fresh writes and legacy uncommitted-lock recovery create one lock-path-only lifecycle commit while preserving unrelated index and worktree state; read-only lock modes never commit."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: ambiguous-multi-target-generation-preflight
    claim: ambiguous-multi-target-generation-preflight
    coverage: full
    rationale: "The CLI semantic domain owns read-only exact-spec preparation before TD lifecycle mutation, stable plan-error envelopes, and prepared-byte revalidation in src/cli/td.rs."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: exact-generated-unit-target-ownership
    claim: exact-generated-unit-target-ownership
    coverage: full
    rationale: "The CLI admission surface emits structured invalid-ownership and unsupported-owned-unit envelopes before lifecycle mutation, including stable IDs, targets, remediation, and HITL state."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: terminal-touched-codegen-drift-gate
    claim: terminal-touched-codegen-drift-gate
    coverage: full
    rationale: "The CLI semantic domain resolves accepted Td-Init-scoped CODEGEN claims, invokes the shared deterministic audit before EC or mutation, and routes drift through phase-safe exact-target regeneration."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: terminal-ec-process-liveness
    claim: terminal-ec-process-liveness
    coverage: full
    rationale: "The CLI semantic domain owns bounded terminal EC process-group cleanup, cross-process single-flight, typed failure results, and exact code-check retry envelopes in src/cli/ec.rs and src/cli/cb.rs."
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
          - name: "EC_COMMAND_TIMEOUT_ENV"
            kind: "constant"
            public: false
          - name: "DEFAULT_EC_COMMAND_TIMEOUT_SECS"
            kind: "constant"
            public: false
          - name: "EC_PROCESS_TERM_GRACE"
            kind: "constant"
            public: false
          - name: "EC_PROCESS_KILL_GRACE"
            kind: "constant"
            public: false
          - name: "EC_OUTPUT_CLOSE_GRACE"
            kind: "constant"
            public: false
          - name: "TERMINAL_EC_GATE_PATHS"
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
          - name: "EcVerifyFailureKind"
            kind: "enum"
            public: true
          - name: "EcCommandOutput"
            kind: "struct"
            public: false
          - name: "EcCommandTimeoutError"
            kind: "struct"
            public: false
          - name: "TerminalEcGateLock"
            kind: "struct"
            public: false
          - name: "TerminalEcGateSession"
            kind: "struct"
            public: true
          - name: "TerminalEcGateAcquisition"
            kind: "enum"
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
          - name: "evaluate"
            kind: "function"
            public: true
          - name: "acquire_terminal_ec_gate"
            kind: "function"
            public: true
          - name: "terminal_ec_gate_blocked_summary"
            kind: "function"
            public: false
          - name: "try_acquire_terminal_ec_gate_lock"
            kind: "function"
            public: false
          - name: "release_terminal_ec_gate_path"
            kind: "function"
            public: false
          - name: "terminal_ec_gate_lock_path"
            kind: "function"
            public: false
          - name: "run_ec_verify_command_with_timeout"
            kind: "function"
            public: false
          - name: "ec_command_timeout"
            kind: "function"
            public: false
          - name: "run_ec_command_with_timeout"
            kind: "function"
            public: false
          - name: "spawn_ec_output_reader"
            kind: "function"
            public: false
          - name: "join_ec_output_reader_until"
            kind: "function"
            public: false
          - name: "configure_ec_command_process_group"
            kind: "function"
            public: false
          - name: "terminate_ec_command"
            kind: "function"
            public: false
          - name: "terminate_residual_ec_process_group"
            kind: "function"
            public: false
          - name: "signal_ec_process_group"
            kind: "function"
            public: false
          - name: "ec_process_group_is_alive"
            kind: "function"
            public: false
          - name: "reap_ec_child_in_background"
            kind: "function"
            public: false
          - name: "ec_verify_bounds_a_wrapper_after_its_child_exits"
            kind: "function"
            public: false
          - name: "ec_verify_kills_surviving_group_member_after_leader_exits_on_sigterm"
            kind: "function"
            public: false
          - name: "ec_verify_rejects_natural_leader_success_with_live_descendant"
            kind: "function"
            public: false
          - name: "terminal_ec_gate_rejects_a_duplicate_inflight_inventory"
            kind: "function"
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
          - name: "terminal_ec_failure_envelope"
            kind: "function"
            public: false
          - name: "terminal_ec_test_barrier"
            kind: "function"
            public: false
          - name: "terminal_ec_test_barrier_after_initial_issue_read"
            kind: "function"
            public: false
          - name: "terminal_ec_test_barrier_after_phase_update"
            kind: "function"
            public: false
          - name: "run_check_lifecycle_terminal"
            kind: "function"
            public: false
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
          - name: "TdInitReachability"
            kind: "enum"
            public: false
          - name: "reachable_td_init_from_head"
            kind: "function"
            public: false
          - name: "committed_paths_since_td_init"
            kind: "function"
            public: false
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
          - name: "run_terminal_codegen_repair"
            kind: "function"
            public: false
          - name: "terminal_touched_codegen_claims"
            kind: "function"
            public: false
          - name: "terminal_touched_codegen_findings"
            kind: "function"
            public: false
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
          - name: "resolve_project_label"
            kind: "function"
            public: true
          - name: "build_create_label_vec"
            kind: "function"
            public: true
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
          - name: "SELF_HOSTING_POLICY_MODE"
            kind: "constant"
            public: true
          - name: "SELF_HOSTING_HARD_GATES"
            kind: "constant"
            public: false
          - name: "SELF_HOSTING_ADVISORY_AXES"
            kind: "constant"
            public: false
          - name: "is_self_hosting_project"
            kind: "function"
            public: true
          - name: "self_hosting_hard_gates"
            kind: "function"
            public: true
          - name: "self_hosting_advisory_axes"
            kind: "function"
            public: true
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
          - name: "SelfHostingPolicyEnvelope"
            kind: "struct"
            public: false
          - name: "self_hosting_policy_envelope"
            kind: "function"
            public: false
          - name: "emit_self_hosting_policy_error"
            kind: "function"
            public: true
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
          - name: "wi_envelope"
            kind: "function"
            public: false
          - name: "open_epic_envelope"
            kind: "function"
            public: false
          - name: "project_from_labels"
            kind: "function"
            public: false
          - name: "issue_is_self_hosting"
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
          - name: "is_active_td_authoring_section_type"
            kind: "function"
            public: false
          - name: "suggested_td_authoring_section_types"
            kind: "function"
            public: false
          - name: "td_section_queue"
            kind: "function"
            public: false
          - name: "td_section_queue_for_content"
            kind: "function"
            public: false
          - name: "td_section_queue_for_spec"
            kind: "function"
            public: false
          - name: "project_label_for_issue"
            kind: "function"
            public: false
          - name: "default_spec_path_for_issue_in_project"
            kind: "function"
            public: true
          - name: "td_section_payload_template"
            kind: "function"
            public: false
          - name: "td_json_payload_schema_hint"
            kind: "function"
            public: false
          - name: "complete_section_apply"
            kind: "function"
            public: false
          - name: "td_error"
            kind: "function"
            public: false
          - name: "PreparedTdGeneration"
            kind: "struct"
            public: false
          - name: "shell_quote_td_arg"
            kind: "function"
            public: false
          - name: "print_generation_plan_error"
            kind: "function"
            public: false
          - name: "prepare_td_generation_before_lifecycle"
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
          - name: "is_recoverable_td_authoring_phase"
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
          - name: "activate_td_workspace_with_recoverable_skeleton"
            kind: "function"
            public: false
          - name: "canonical_issue_path_for_slug"
            kind: "function"
            public: false
          - name: "issue_path_arg"
            kind: "function"
            public: false
          - name: "ensure_clean_or_only_dirty_paths"
            kind: "function"
            public: false
          - name: "checkout_has_only_exact_untracked_path"
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
          - name: "reset_unreachable_td_init"
            kind: "function"
            public: false
          - name: "bootstrap_td_issue"
            kind: "function"
            public: false
          - name: "run_create_brief"
            kind: "function"
            public: false
          - name: "td_spec_skeleton"
            kind: "function"
            public: false
          - name: "known_empty_td_spec_skeletons"
            kind: "function"
            public: false
          - name: "is_known_empty_td_spec_skeleton"
            kind: "function"
            public: false
          - name: "recoverable_untracked_td_skeleton"
            kind: "function"
            public: false
          - name: "canonicalize_recoverable_td_skeleton"
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
          - name: "normalize_generic_td_section_payload"
            kind: "function"
            public: false
          - name: "TdPayloadTopLevelShape"
            kind: "struct"
            public: false
          - name: "td_payload_top_level_shape"
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
          - name: "TdLockWriteAction"
            kind: "enum"
            public: false
          - name: "TdLockWriteResult"
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
          - name: "write_project_td_lock_at_root"
            kind: "function"
            public: false
          - name: "write_project_td_lock_file_at_root"
            kind: "function"
            public: false
          - name: "commit_td_lock_update"
            kind: "function"
            public: false
          - name: "preflight_repo_relative_td_lock_path"
            kind: "function"
            public: false
          - name: "git_diff_has_changes"
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
          - name: "AGY_CONFIG_KEY"
            kind: "constant"
            public: false
          - name: "AGY_MATCHER"
            kind: "constant"
            public: false
          - name: "LOCAL_BYPASS_GIT_REL"
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
          - name: "GuardBypassArgs"
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
          - name: "includes_agy"
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
          - name: "run_bypass"
            kind: "function"
            public: false
          - name: "run_resume"
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
          - name: "agy_guard_command"
            kind: "function"
            public: false
          - name: "agy_hooks_path"
            kind: "function"
            public: false
          - name: "install_agy_guard_hook"
            kind: "function"
            public: false
          - name: "remove_agy_guard_hook"
            kind: "function"
            public: false
          - name: "upsert_agy_guard_hook_at"
            kind: "function"
            public: false
          - name: "remove_agy_guard_hook_at"
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
          - name: "agy_allow_output"
            kind: "function"
            public: false
          - name: "agy_deny_output"
            kind: "function"
            public: false
          - name: "tracked_guard_paths"
            kind: "function"
            public: false
          - name: "ensure_tracked_guard_paths_clean"
            kind: "function"
            public: false
          - name: "commit_guard_policy"
            kind: "function"
            public: false
          - name: "local_bypass_path"
            kind: "function"
            public: false
          - name: "local_bypass_is_active"
            kind: "function"
            public: false
          - name: "guard_handler_is_active"
            kind: "function"
            public: false
          - name: "hook_file_has_guard_handler"
            kind: "function"
            public: false
          - name: "agy_hook_has_guard_handler"
            kind: "function"
            public: false
          - name: "decide_active_pretool_payload"
            kind: "function"
            public: false
          - name: "decide_pretool_payload"
            kind: "function"
            public: false
          - name: "sanction_reason_for"
            kind: "function"
            public: false
          - name: "extract_target_paths"
            kind: "function"
            public: false
          - name: "agy_command_line"
            kind: "function"
            public: false
          - name: "parse_agy_direct_mutation_targets"
            kind: "function"
            public: false
          - name: "shell_path_operands"
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
          - name: "strip_project_prefix"
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
      #1469: `verify_ec_context` gained a `required_only: bool` execution-time
      filter parameter. When `true`, a `required_for_production: false` case
      is never executed — it still gets a `status: "skipped"` entry in
      `EcVerifyCommandResult.results` (`stderr_tail: "skipped (advisory)"`)
      so the demotion stays auditable — and `command_count`/`passed_count`/
      `failed_count` (and therefore `clean`) only count executed entries.
      `TerminalEcGateSession::evaluate` (the #858 per-close terminal EC gate)
      calls `verify_ec_context(&ctx, true)`; `EcVerifyArgs` gained a
      `required_only` flag (`aw ec verify --required-only`) that threads the
      same `true` into `run_verify`, while the bare `aw ec verify` default
      stays `verify_ec_context(&ctx, false)` (unchanged, runs everything).
      Tool-manifest commands have no `required_for_production` concept and
      always run regardless of the filter. See this file's `cb.rs` entry
      below for the terminal-gate envelope's `cases` list rendering of
      skipped entries. #1579 assigns each EC command its own process group and
      bounds it with `AW_EC_COMMAND_TIMEOUT_SECS`; the 30-minute default keeps
      legitimate long Cargo/VAT evaluations viable while tests and operators
      can select a shorter deadline. On timeout, AW sends TERM to the group,
      preserves the full grace even when the leader exits first, KILLs any
      surviving descendants, probes ESRCH safely, bounds leader reaping and
      output-pipe joins. A normal leader exit with live same-group descendants
      is cleaned but rejected as `RunnerError`; it can never preserve an exit-0
      false green. Results classify command failure, timeout, runner error, and
      single-flight separately. `acquire_terminal_ec_gate` returns a lease
      before evaluation; `cb.rs` re-reads the WI under that lease, skips EC when
      a stale fresh phase has already become `td_merged`, and keeps the lease
      through the full terminal transition. Thus both overlapping callers and
      late-acquiring stale readers execute one VAT/Cargo inventory.
      #1829: `aw ec review` now accepts independent agent-backed evidence
      alongside human evidence, gated by the project's `ec_review_backing`
      policy (`human` default | `agent` | `either`, resolved from
      `Project::ec_review_backing` via `resolve_ec_project_context` into a
      new `EcProjectContext.review_backing` field, normalized by
      `normalize_review_backing`/checked by `ec_review_backing_allows`).
      `run_gen` now writes a durable `ec-author.json` record
      (`EcAuthorRecord`, best-effort, never blocks `ec gen`) via
      `write_ec_author_record`, capturing `current_actor_identity()`
      (`AW_ACTOR_ID`/`AW_AGENT_ID` env, else `unknown-actor`) for the
      finalized `generated_from_td_digest`. Independence is enforced by
      `ec_reviewer_is_independent`: an agent `reviewed_by` identity matching
      the recorded author for the same digest is rejected in both
      `validate_ec_review_payload` (submission time) and
      `ec_review_record_findings` (terminal-gate re-check), regardless of
      policy; a legacy/missing author record is treated as independent
      (backward compatible). `validate_ec_review_payload` also rejects
      `reviewer_kind: "agent"` outright when the policy doesn't allow it, and
      rejects any `reviewer_kind` other than `human`/`agent`.
      `EcReviewSummary` gained `backing: Option<String>` (the accepted
      record's `reviewer_kind`) and `agent_review_prompt: Option<String>`.
      When the policy allows agent backing and no current accepted/
      needs-revision record exists, `run_review` now emits a non-blocking
      `status: "pending_agent_review"` envelope (`requires_hitl: false`)
      carrying `agent_review_prompt` — the structured inspection-checklist
      prompt from `agent_ec_review_prompt` covering the six #1504 dimensions
      (capability_claim_coverage, required_dimensions, assertions_specific,
      oracle_independent, loopholes_checked, false_green_risk_checked) plus
      the independence requirement — instead of the blocking human HITL
      question; the host dispatches its own subagent reviewer and submits
      the verdict back through the existing `--evidence-file` mechanism
      (`reviewer_kind: "agent"`), which resumes the same validation path. An
      agent `needs_revision` verdict routes to bounded `aw ec fill` exactly
      like the human path (same `ec_fill_command_for_target` remediation). A
      human `--evidence-file` submission (`reviewer_kind: "human"`) always
      remains valid regardless of policy, so a post-completion human audit
      can reopen (`needs_revision`) an EC whose current accepted record is
      agent-backed. New pub helper `project_ec_review_backing(project) ->
      Result<String>` resolves a project's policy for `aw health`/reporting
      (see this file's `project.rs`/`cb.rs` entries) without duplicating
      `resolve_ec_project_context`. Default behavior (`ec_review_backing`
      absent from `aw.toml`) is unchanged: `review_backing` normalizes to
      `"human"`, so `reviewer_kind: "agent"` evidence is rejected exactly as
      before #1829.
      #1828: EC review can now be deferred to a post-completion batch
      review instead of blocking the loop, gated by the project's
      `ec_review_mode` policy (`blocking` default | `deferred`, resolved
      from `Project::ec_review_mode` via `resolve_ec_project_context` into
      a new `EcProjectContext.review_mode` field, normalized by
      `normalize_review_mode`/checked by `ec_review_mode_is_deferred`).
      `ec_review_gate_findings` (the original single-purpose gate-findings
      function) is now composed from `ec_review_outstanding_findings`
      (all missing/stale/rejected-review findings, unchanged from
      pre-#1828), `ec_review_outstanding_is_deferrable` (true only when the
      outstanding set is exactly the "review hasn't happened yet" case —
      never an explicit `needs_revision` decision or a structural EC
      content finding from `ec_semantic_review_findings`, per R6), and
      `ec_review_deferred_pending_findings` (the deferrable subset,
      surfaced as a queue entry rather than a gate finding). In `deferred`
      mode with a deferrable outstanding review, `ec_review_gate_findings`
      returns empty — so `run_verify`/`verify_ec_context` and the terminal
      gate no longer block — while `verify_ec_context` inserts a pseudo
      `EcVerifyCommandResult` with `status: "deferred"` for the review case
      (excluded from `executed_count`/`passed_count`/`failed_count`, so it
      never flips `clean` to `false`) so the pending review stays visible
      in verify output. `run_review` gains a matching `deferred_pending_human`
      branch: when no explicit decision exists and the outstanding findings
      are deferrable under `deferred` mode, it returns an `EcReviewSummary`
      with `status: "deferred_pending_human"`, `requires_hitl: false`, and
      a `next: Option<String>` command string identical in form to the
      blocking-mode HITL resume command (`aw ec review --project <p>
      --evidence-file <path>`), so a human can finalize post-hoc with the
      same digest-bound `--evidence-file` submission — `run_review`'s
      acceptance/`needs_revision` paths are otherwise unchanged (R4). An
      explicit `needs_revision` decision, or any structural
      `ec_semantic_review_findings` content finding, still blocks in
      `deferred` mode exactly as in `blocking` mode (R6: deferred mode
      changes *when* review must happen, never *what* content is
      acceptable, nor *who* may review — same-agent self-review stays
      unaccepted). New pub helpers `project_ec_review_mode(project) ->
      Result<String>` and `project_pending_ec_review(project) ->
      Result<Option<String>>` resolve a project's policy and its current
      deferred-pending queue entry (if any) for `aw health`/reporting (see
      this file's `project.rs`/`cb.rs` entries) without duplicating
      `resolve_ec_project_context`. Default behavior (`ec_review_mode`
      absent from `aw.toml`) is unchanged: `review_mode` normalizes to
      `"blocking"`, so a missing/stale review still blocks exactly as
      before #1828.
      #1859: the unconfigured/mistyped-key `ec_review_backing` default flips
      from `"human"` to `"either"` (agent-first) — `resolve_ec_project_context`
      now falls back to `EC_REVIEW_BACKING_EITHER` instead of
      `EC_REVIEW_BACKING_HUMAN` when `Project.ec_review_backing` is absent,
      and `normalize_review_backing` gained an explicit `"human"` match arm
      (preserving that literal exactly) while its catch-all for any other
      unrecognized value also becomes `EC_REVIEW_BACKING_EITHER`. No routing
      logic changed: `run_review`'s existing `ec_review_backing_allows(&ctx
      .review_backing, "agent")` check (already gating the non-blocking
      `pending_agent_review` envelope vs. the blocking human HITL) picks up
      the new default automatically, so an unconfigured project now emits
      `pending_agent_review` where it previously emitted a blocking HITL
      question; explicit `ec_review_backing = "human"` continues to
      reproduce the pre-#1859 blocking-HITL/agent-evidence-rejected behavior
      byte-for-byte (R2/AC2). All other invariants (author-identity
      independence, digest binding, `needs_revision` routing, human-audit
      reopen of an agent-accepted EC, `ec_review_mode = "deferred"`
      semantics) are unchanged (R3).
      #2084: `run_verify` now calls the same structural inventory check used by
      `aw ec check` before executing any generated command. A stale TD/EC
      source digest fails closed and, when
      `--wi` is present, persists `aw ec review --project <project> --wi <wi>`
      as the root-owned next action. `verify_ec_context` repeats this guard at
      the shared terminal-gate boundary so code-check cannot execute stale
      inventory either. A semantic-review pseudo-failure is likewise routed
      to review rather than recorded as a generic red result whose default
      `aw td gen` adaptation is unrunnable for cb_filled HANDWRITE-only WIs.
      Regressions use stale and unreviewed `touch` commands to prove neither
      sentinel is ever created and the latter is classified for review
      routing.
      #2122: `needs_revision` review evidence now names an existing Markdown
      source before AW accepts it. `ec_fill_command_for_target` and
      `run_fill` accept canonical `external-contracts/**` sources plus an
      existing `tech-design/**` fallback for projects whose inventory is
      still TD-derived; arbitrary and missing paths remain rejected. The
      independent-review prompt states the same boundary, preventing a
      reviewer from inventing an external-contract path that would make the
      emitted `aw ec fill` command fail immediately.
      #2124: `run_gen --wi` now resolves the owning WI phase before it writes
      the durable post-generation action. A fresh EC-first root still hands
      off to `aw td create`, while `cb_filled` (including retired post-fill
      aliases normalized by `td_phase::normalize`) routes to `aw ec verify`
      against the newly current inventory. This prevents an accepted
      review/regeneration cycle from rewinding a completed implementation to
      a `td create` command that the phase guard must reject.
      #2125: `ec_verify_command`, shared by WI roots, capability roots, and
      the post-review `ec gen` handoff, now emits `--required-only`. The
      root-owned loop verdict therefore executes the same production-required
      case set as terminal code-check and records advisory cases as skipped
      instead of turning a documented non-production stability NO-GO into TD
      adaptation. The public unscoped `aw ec verify` default is unchanged and
      still runs all configured cases when an operator explicitly wants the
      advisory suite.
      #1799: `ensure_ec_review_payload` no longer rewrites an existing
      payload file when its `version`/`project`/`source_digest` mismatch the
      current manifest — it now only scaffolds the blank pending template
      when nothing exists yet at the target path, so a populated payload
      (any decision, checklist, or digest) is never silently clobbered back
      to pending before `run_review` evaluates it. A stale digest is still
      caught and reported by `validate_ec_review_payload`'s existing
      "EC review payload is stale; rerun `aw ec review`" error, so staleness
      stays visible instead of being masked by a silent reset.
      `run_review` also stops calling `ensure_ec_review_payload` at all when
      `--evidence-file` names caller-supplied evidence: that path is read
      as-is via `read_ec_review_record` and is never auto-initialized or
      rewritten, so a populated `--evidence-file` payload is guaranteed to
      be evaluated exactly as submitted.
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
      #1469: the terminal session success-envelope branch (issue #858's
      per-close EC gate, `run_check_lifecycle_terminal`) renders each
      `EcVerifyCommandResult` whose `status` is `"skipped"` as
      `"<case_id> (skipped (advisory))"` in the `ec_gate.cases` list instead
      of the bare case id, so a `required_for_production: false` case the
      gate's `verify_ec_context(&ctx, true)` filter skipped stays auditable
      in the envelope; `ec_gate.commands_consulted` is unchanged
      (`summary.command_count`, which `ec.rs` already restricts to executed
      cases). #1579 includes each failed result's stderr tail, emits distinct
      `terminal_ec_failure`, `terminal_ec_timeout`,
      `terminal_ec_runner_error`, or `terminal_ec_single_flight` error kinds,
      and always returns the exact runnable retry
      `next.command = aw td code-check <slug>`. The refusal happens before
      issue phase, close state, or terminal commit mutation. The function now
      acquires the EC lease before evaluation, re-reads the WI under the lease,
      routes a stale `td_merged` observation through terminal retry without EC,
      and holds the lease until remote closure, landing, terminal commit, and
      workflow unlock finish. Configured-inventory retry entries acquire that
      same lease while continuing to skip EC, so they cannot race an owner's
      post-phase terminal steps. Narrowly named bounded debug-only barriers
      expose the post-initial-read/pre-acquire and post-phase-update seams for
      deterministic process coverage. See `ec.rs` above for runner and lease.
      Issue #1602 extracts the exact HEAD-reachable slug plus Td-Init lookup
      into `TdInitReachability`. The hand-written implementation gate retains
      its prior distinction: no slug history stays legacy-compatible, while
      same-slug lifecycle history without an exact Td-Init remains a hard
      verification error rather than becoming implementation evidence.
      Issue #1635 reuses that exact baseline to intersect committed paths with
      current CODEGEN rows from the WI's accepted TDs. The fresh terminal path
      sends each exact target/spec-section claim through `audit_file` before
      EC or mutation, ignores reports owned by other specs or sections, and
      emits `aw td gen <slug>` on drift. In a terminal phase that command
      preflights and regenerates only the selected target-file scopes without
      project-wide post-passes, commits those paths, preserves WI phase, and
      returns the exact code-check retry. Issue #1591 routes the public
      force-regeneration entry point through the same shared spec runner used
      by replay and cold verification. Source application still honors
      hand-written skips, then `project_root_llms` targets are deterministically
      replaced by the TD-first project context emitter before formatting and
      lifecycle commit; a real Git fixture proves the output is one canonical
      CODEGEN document with no generic TODO and preserves a HANDWRITE sibling
      byte-for-byte.
      #1828: the terminal EC-gate success envelope's `ec_gate.cases`
      rendering gains a third label form alongside the bare case id and
      `"(skipped (advisory))"`: a result whose `status` is `"deferred"`
      (this project's `ec_review_mode = "deferred"` and a pending human
      review, from `ec.rs`'s `verify_ec_context` pseudo-result) renders as
      `"<case_id> (deferred (pending human review))"`, so the terminal gate
      commits/closes without blocking while keeping the pending review
      visible in the same envelope. See `ec.rs` above for the deferred-mode
      gate/queue implementation.
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
      Issue #1519 keeps `resolve_project_label` as the WI `--project` producer
      boundary: registry rows canonicalize the retired `project:` family before
      `build_create_label_vec` persists an issue label, so current commands emit
      only path-correct `app:` or `lib:` identities using the registered row
      name. The regression includes a project-local stale-label override of a
      label-free root row.
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
  - path: "apps/agentic-workflow/src/cli/run.rs"
    action: modify
    section: schema
    description: |
      Issue #1518 extends the centralized project-label resolver with the
      canonical `project:<name>` prefix while preserving `app:` and `lib:`.
      Open epic routing now uses `open_epic_envelope`: a resolved identity
      emits an exact `aw wi atomize --project <name>` command, while missing,
      empty, or whitespace-only identity labels return a blocked/HITL
      envelope with `aw wi show <id>` remediation instead of the retired
      `PROJECT` placeholder. Focused tests cover the historical #1511 pgpool
      label shape, app/lib compatibility, invalid values, and real-CLI chain
      parsing.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/run.rs"
    action: modify
    section: schema
    description: |
      Issue #1501 adds a single self-hosting admission policy. Root capability
      and work-item runners reject Agentic Workflow identity before a lifecycle
      tick, return a terminal aw.cli.v1 policy envelope with no next command or
      invoke, and route self rollup to read-only health claims instead of a
      project root runner.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1501 rejects the project-scoped capability runner before its
      report or next-action loop can mutate local planning or tracker state.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/chain.rs"
    action: modify
    section: schema
    description: |
      Issue #1518 registers `run.rs:open_epic_envelope` in EMIT_REGISTRY with
      `aw wi atomize --project pgpool`, so the exact epic handoff is parsed
      against the real clap tree by the chain-conformance suite.
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
      Issue #1562 accepts a body-only generic section payload by restoring the
      requested typed H2/annotation wrapper before the existing exact
      single-section merge. A fence-aware boundary rejects empty or placeholder
      bodies, unclosed or wrong-language fences, mismatched annotations, broken
      wrappers, and multiple top-level H2 sections before any spec write. A
      complete matching wrapper, including a custom heading, is preserved; the
      existing RequireThrough(candidate) validation and dirty-spec allowance
      remain unchanged. Issue #1586 sends the normalized merged candidate
      through the full shared registry before the write boundary, while
      completed specs keep the file-backed registry path. A valid signature /
      loop LogicSpec can replace stale plain Mermaid without inheriting the
      old file's finding; an invalid candidate leaves spec and payload intact.
      Issue #1598 makes the fresh default queue `logic`, `changes`, then
      `unit-test`: Changes is the
      explicit target plan required before codegen infers implementation paths,
      while an already non-empty custom queue keeps its declared membership and
      order. The initialized generic JSON payload carries an editable YAML
      `changes[]` skeleton without narrowing legacy/custom Changes fields. Each
      Changes transition is exposed through the issue projection lock in both
      passes, including the first contract section after applicability unlock;
      the real CLI regression applies both target plans, passes `aw td check`,
      writes the fixture TD IR lock, and proves `aw td gen` creates the named
      new target instead of falling back to impossible new-path inference.
      Issue #1633 makes the complete generation plan a caller-side admission
      gate. The CLI reads exact spec bytes without issue hydration or checkout,
      including an existing TD branch through Git objects, invokes the shared
      read-only exact spec-ref inference and Schema/CLI ownership predicate,
      and emits one structured stdout error carrying stable kind, section,
      sorted targets, remediation, shell-safe next command, and incomplete
      completion. One inferred existing target remains compatible; none is
      unavailable and multiple are ambiguous. Only an admitted plan may enter
      lifecycle mutation, and its bytes are compared exactly again after
      activation before execution.
      Issue #1634 extends that same admission boundary with stable
      `schema:<name>` and `cli:<name>` unit IDs. Canonical Changes
      `generates:` lists must form an exhaustive, unique partition before
      lifecycle mutation; invalid ownership emits a structured remediation
      envelope, while an owned unit without a generator emits a typed HITL
      generator-gap envelope. The executor repeats the predicate and passes
      only the current target's typed IR partition to Schema or CLI codegen.
      Issue #1602 activates an existing TD branch before inspecting reachable
      history. An exact slug plus Td-Init resumes normally; missing or
      same-slug-without-init history clears the stale authoring phase, branch,
      projection, and lock labels, records an unreachable-td-init Td-Reset,
      then reuses normal provisioning for a fresh baseline and Logic queue.
      Post-gen and terminal retry phases are excluded from recovery, and
      ordinary WI phase `created` remains a fresh provisioning entry.
      Issue #1580 makes the CLI-owned empty TD skeleton part of lifecycle
      history. Recovery requires whole-tree porcelain-v1 bytes to be exactly
      one untracked target and its regular-file bytes to match a finite set of
      historical empty skeletons. Tracked, staged, renamed, authored, symlink,
      or sibling-dirty state remains immutable and fails the ordinary clean
      gate. The admitted candidate is revalidated across branch activation and
      #1602 reset/provision, canonicalized through the serde-YAML id renderer,
      only after the refreshed issue passes the `td_inited` phase guard, then
      staged by the fresh queue-start commit. An already locked old run receives
      one spec-only recovery queue-start commit; reruns are clean and
      history-idempotent. Reachable `td_created`, post-gen, and terminal phases
      never mutate or commit the candidate.
      Issue #1519 adds a producer-to-resolver regression that sends stale
      registered library and app labels through `resolve_project_label`, the
      create label vector, and `default_spec_path_for_issue_in_project`.
      Libraries resolve under `libs/<name>/tech-design`, apps retain their
      existing root, and a raw `project:` issue label still fails loudly.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/project.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
      #1828: `ProjectEcGateReport` gains `review_mode: Option<String>` and
      `pending_review: Option<String>` fields (populated by
      `project_health_ec_axis` from `ec.rs`'s `project_ec_review_mode`/
      `project_pending_ec_review`, and echoed in `project_ec_gate_summary`'s
      JSON), surfacing the deferred-mode pending-review queue in `aw health`
      reporting (AC3). `apply_ec_to_report` now delegates the
      advisory-vs-blocker classification of an outstanding pending review to
      a new `apply_pending_ec_review_classification` helper: when
      `review_mode == "deferred"`, a pending review is always surfaced as a
      finding but never added to `report.blockers` (advisory, R5); in
      `blocking` mode (default) it stays a hard blocker under
      `--verify-ec`, byte-for-byte the pre-#1828 behavior.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/td_lock.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
      Issue #1587 makes the mutating lock writer a root-aware transaction. A
      fresh or stale lock is serialized, staged with a literal exact path, and
      committed with `git commit --only`; a semantically clean lock left by an
      older CLI is recovered through the same path. Git diff exit 0, 1, and
      error statuses remain distinct, the configured lock must resolve to a
      regular file inside the repository, and the command fails closed when a
      changed lock cannot be committed. Unrelated staged, unstaged, and
      untracked paths retain their exact state. An already committed lock is a
      no-op, while `--check` and `--show` return before any commit path. Before
      snapshot or write, lexical path components and the canonical TD parent
      must both remain real directories inside the repository; an existing
      lock must be a non-symlink regular file at that exact canonical leaf.
      External TD-directory and lock-leaf symlink regressions prove external
      bytes, HEAD, and repository status remain unchanged on rejection.
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
      semantic domain's evidence list. Issue #1496 adds the `model` topic with
      the canonical agent-first CLI definition and a compile-time contract
      regression that rejects removed product architecture from active
      README, CAPABILITIES, and product-model TD prose.
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
  - path: "apps/agentic-workflow/src/cli/guard.rs"
    action: modify
    section: schema
    description: |
      Issue #1429 (closes #1269): `aw guard pretool` now consults the #1428
      sanctioned-path resolver (`guard_sanction::is_sanctioned`) before its
      deny decision, allowing edits to TD `impl_mode: hand-written` paths
      declared by an active WI at an eligible phase. `decide_pretool_payload`
      became async and gained a new `sanction_reason_for` helper (resolves a
      guarded target's project-root-relative sanction reason); `GuardScope`
      gained a `strip_project_prefix` method to translate repo-root-relative
      targets into the project-root-relative keys the resolver uses. Symbol
      list updated for both new functions.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/guard.rs"
    action: modify
    section: schema
    description: |
      Issue #1802: persistent `aw guard on/off` toggles now commit only their
      changed tracked Codex/Claude policy files and reject pre-existing policy
      drift. `aw guard bypass`/`resume` provide an expiring worktree-local
      exception instead of disabling policy. Added the AGY adapter, preserving
      its user-global hooks, returning its required explicit allow/deny JSON,
      and recognizing only direct target-bearing shell mutations so CAP remains
      responsible for broad destructive-command control. Symbol inventory and
      focused guard coverage now include the new policy, bypass, and AGY paths.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1847: doc-stored WI ids on capability work roots become
      optional-deprecated, derived provenance instead of a readiness
      precondition or parse requirement -- the reference direction is
      one-way (WI -> durable doc), never the reverse. The private
      `MarkdownWorkRootIndices.wi` column index becomes `Option<usize>`
      (`markdown_work_root_indices` no longer requires a `WI`/`WorkItem`
      header to exist at all); `upsert_work_root_wi_cell` (the `set-wi-ref`
      writer) becomes a no-op for a matched row that has no WI column instead
      of panicking on an out-of-bounds index. R4: the four
      `validate_work_root_kind/impl/verification/maturity` cell validators
      change from `Result<()>` (`anyhow::bail!`, propagated through
      `parse_markdown_capability_block`/`parse_markdown_table_capability_sections`
      via `?` and surfaced as a hard `parse_capability_document` error) to
      `Option<String>` advisory findings folded into the existing
      `CapabilityDocument.findings` bucket with `aw capability check
      --project <project>` remediation text -- a single malformed legacy cell
      (e.g. the historical `Kind bug` #1801 row) can no longer break
      `aw capability report`/`run`/`check` for the whole document; the row's
      other fields still parse and render. R2: `capability_wi_evidence` now
      additionally calls a new `wi_derived_capability_evidence` helper (after
      resolving all doc-stored `active_wi` refs, tracked via a
      `seen_references: BTreeSet<String>` dedup set) that scans every
      candidate issue's body for a `## Capability Alignment` section (new
      `wi_body_capability_alignment_ids`, bounded to the next `\n## ` heading
      or EOF, backtick-quoted ids extracted via the existing
      `extract_backtick_values` + `slugify`) and, when a WI declares the
      capability or one of its gap ids there, appends it as additional
      `CapabilityWiEvidence` -- additive and read-only, never required for a
      work-root row to be valid, and never gating readiness. Readiness
      (`capability_claim_reports`/`capability_gap_status_from_table`) was
      already computed purely from verification gates/fixtures and
      Implementation/Verification cells, not from the WI column, confirming
      R1 structurally: a work-root row (or an entire table) with no WI
      column at all is fully valid and its claims verify/fail solely from
      their own gates and fixtures. `CAPABILITIES.md`'s pre-existing `Shell
      text-source-unit cold replay | bug | #1801` row (the reproduction case
      for this issue) had its Kind cell corrected to `change` as migration
      proof (AC4); the row still parses/renders under the new advisory path
      regardless. New focused tests:
      `markdown_work_root_table_degrades_invalid_enums_to_advisory_findings`
      (renamed from `_rejects_invalid_enums`),
      `work_root_table_without_wi_column_parses_and_computes_readiness`,
      `zero_wi_ref_readiness_spans_verified_red_gate_and_unmapped_claims`,
      `capability_wi_evidence_resolves_from_wi_body_without_doc_stored_ref`,
      and `fictitious_doc_wi_ref_degrades_to_advisory_evidence` (the
      jet-#818/#819-class fictitious-ref scenario, AC3). Deferred out of this
      change: R3 (rewriting `aw capability run` scheduling to query the WI
      tracker directly instead of doc rows), full R5 (retiring/repurposing
      `aw capability set-wi-ref` to write WI-side state), and R6 (CLAUDE.md/
      AGENTS.md self-hosting-gate prose) -- R6's prose files and R5's WI-body
      writer both route through `src/cli/issues.rs`, which is off-limits to
      this change per a concurrent in-flight edit.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1847 (R3, second slice): `choose_next_action`'s open-gap
      scheduling branch now resolves tracker-side first. A new
      `tracker_open_evidence` lookup finds an OPEN `CapabilityWiEvidence`
      entry for the gap's id inside `item.wi_evidence` -- which already
      covers both doc-stored `active_wi` refs AND R2's WI-body derived
      provenance (`wi_derived_capability_evidence`) -- and, when found, wins
      over the doc-stored `gap.active_wi` cell for both the epic-vs-bounded
      branch and the `lifecycle_action_for_work_item` dispatch (`active_ref`
      prefers `tracker_open_evidence.reference`, falling back to
      `gap.active_wi` only when the tracker has no open match). This closes
      the R3 gap: previously a gap whose doc row carried no WI cell but was
      already served by an open WI (declared only in that WI's own body)
      fell through to `CreateWi`/`aw wi plan`, silently proposing duplicate
      work. The doc-stored ref remains the fallback input for both the
      epic-rollup path and the advisory `ReconcileWiRefs` degradation when a
      doc ref is stale (points at a closed/unknown/nonexistent issue) and no
      other open tracker evidence exists for the gap -- so the jet-class
      fictitious-ref and #1801-class malformed-row failure modes still
      degrade to advisory action, never a hard error. New focused tests on
      `choose_next_action`: `next_action_schedules_tracker_open_wi_found_via_body_ref_without_doc_ref`
      (open WI found only via body ref -> scheduled, not re-created),
      `next_action_routes_red_gate_with_no_tracker_wi_to_creation` (no doc
      ref, no tracker WI -> `CreateWi`), and
      `next_action_degrades_stale_doc_ref_with_no_open_tracker_wi_to_advisory`
      (stale doc ref, no open tracker WI -> `ReconcileWiRefs` advisory, not
      an error).
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1847 (R5, second slice): `aw capability set-wi-ref` is
      repurposed to write the WI side instead of the doc side. `set_capability_wi_ref`
      becomes `async` (dispatched via `.await` from `run()`); its primary
      effect is now, for each `--wi` id, resolving the issue through
      `crate::issues::{resolve_default_backend, make_backend}` and, when the
      backend is writable and the issue resolves, calling the new
      `ensure_wi_body_capability_alignment_ref` helper (idempotent: inserts
      `Capability: \`<id>\`` / `Capability Gap: \`<id>\`` lines into the WI
      body's own `## Capability Alignment` section -- creating the section
      if absent, leaving the rest of the body untouched, no-op if both ids
      already declared) and persisting via `IssueBackend::update` with an
      `IssuePatch { body: Some(updated), .. }`. The legacy README/CAPABILITIES
      work-root `WI` cell rewrite (`upsert_capability_wi_ref_in_readme`) is
      kept only as a deprecated best-effort compatibility path: a row with no
      WI column, or any doc-side mismatch, now surfaces as a
      `doc_cell_warning` in the JSON payload instead of a hard `bail!` --
      doc-stored WI refs are advisory, not the contract, so this verb must
      not fail the (now-primary) WI-side write over a doc-side row shape it
      no longer requires. Payload gains `wi_side_updated`/`wi_side_warnings`/
      `doc_cell_updated`/`doc_cell_warning`/`notice` fields; the `notice`
      text names the doc-cell path as deprecated in every invocation's
      output. New `normalize_wi_ref_ids` (bare-digit variant of
      `normalize_wi_ref_values`, for `IssueBackend::get`/`update` keys, which
      take numeric id or slug, not the `#<n>` display form). No `src/cli/
      issues.rs` (the `aw wi` CLI file, off-limits to this change) or
      `VERB_LIFECYCLE_REGISTRY` edit was needed: the write goes through
      `crate::issues::{IssueBackend, IssuePatch}` (the backend engine module,
      already imported here), and `capability.set-wi-ref` keeps its existing
      `Core`/`mutates_lifecycle: true` classification since it still mutates
      tracked lifecycle state, now on the WI side. New focused tests:
      `ensure_wi_body_capability_alignment_ref_creates_section_when_absent`,
      `ensure_wi_body_capability_alignment_ref_adds_missing_id_to_existing_section`,
      and `ensure_wi_body_capability_alignment_ref_is_a_noop_when_both_ids_already_present`.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1848 (R1-R3): the default `cap_path` resolution (no explicit
      `[[projects]].cap_path`/`--cap-path`) flips from `<project>/README.md`
      to `<project>/CAPABILITIES.md` in `capability_path_from_row`; an
      explicit `cap_path` (aw.toml or `--cap-path`) still wins unchanged.
      New `readme_capability_migration_source` detects the degradation case:
      when the resolved default CAPABILITIES.md does not exist yet and the
      project's README still carries canonical or legacy capability
      structure, that README is a migration source, not a hard read
      failure. `build_capability_report_inner` calls it right after
      `resolve_capability_path` (default-path only) and, when a source is
      found, short-circuits to the new `capability_map_readme_resident_report`
      helper, which returns a `status: "blocked"` report whose `next_action`
      is the new `CapabilityActionKind::LocationMigrationRequired` variant
      naming `aw capability migrate --project <name>` -- never a parse
      error. `migrate_capability_format` branches on this same detection: a
      README-resident default project routes to the new
      `apply_readme_capability_relocation_tick` (reads+parses the README via
      `parse_capability_document_repairing_previous_migration`, renders a
      fresh CAPABILITIES.md body via the new
      `render_relocated_capability_document` -- `## Brief` / `## Capabilities`
      intro / `### Capability Index` + per-capability sections, reusing
      `render_capability_registry` for the body -- writes it, then rewrites
      the README via the new `render_readme_capability_migration_residue`,
      which strips migrated capability sections
      (`strip_migrated_capability_sources` + the existing
      `CAPABILITY_MIGRATION_INSERT_MARKER` split/consume pattern) and leaves
      a `## Capability Contract` pointer to CAPABILITIES.md in their place);
      any other project keeps the existing in-place
      `apply_capability_format_migration_tick` YAML/legacy-to-canonical
      path. `render_empty_capability_readme` (the `aw capability init`
      shell) gains a `readme_resident: bool` parameter: the new default
      (CAPABILITIES.md-resident) shape is `## Brief` / `## Capabilities` /
      `### Capability Index` only; the README-resident shape (selected only
      for an explicit `--cap-path README.md` override) keeps the prior
      `## Contributing` + `## Capability Contract` pointer scaffold.
      `init_capability_readme` derives `readme_resident` from the resolved
      `cap_path` file name. New focused tests:
      `capability_path_from_row_defaults_to_capabilities_md`,
      `capability_path_from_row_explicit_cap_path_wins`,
      `resolve_capability_path_override_argument_wins_over_default`,
      `readme_capability_migration_source_detects_readme_resident_structure`,
      `readme_capability_migration_source_none_when_capabilities_md_exists`,
      `readme_capability_migration_source_none_when_readme_has_no_capability_structure`,
      `capability_init_renders_capabilities_md_resident_shell_by_default`,
      and `capability_init_renders_readme_resident_shell_for_explicit_readme_override`.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/run.rs"
    action: modify
    section: schema
    description: |
      Issue #1848: the workflow envelope dispatcher's exhaustive match on
      `CapabilityActionKind` gains the new `LocationMigrationRequired`
      variant (#1848), routed identically to the existing
      `FormatMigrationRequired` arm (dispatch to
      `aw capability run --project <name> --non-interactive`).
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/run.rs"
    action: modify
    section: schema
    description: |
      Issue #1899: `wi_run_command`, `capability_run_command`, and
      `project_capability_rollup_command` -- the shared envelope
      command-string builders every `next`/`resume_command`/`agent_prompt`
      site funnels through -- now emit `aw goal wi <id>`, `aw goal
      capability <cap-id> --project <p>`, and `aw goal capability --project
      <p> --non-interactive --max-ticks 1` respectively, in place of the
      previous `aw wi run <id>` / `aw capability run [<cap-id>] --project
      <p>` forms. `run_wi_root`/`run_capability_root` themselves are
      unchanged (`aw goal wi`/`aw goal capability` in `goal.rs` call them
      directly); only the emitted command strings moved. Adds
      `emit_retired_verb_redirect(retired, root_kind, root_id, replacement,
      print)`, a shared helper the retired `aw capability run` clap leaf now
      calls: it prints a `status: "error"`, `action: "retired_verb"`
      `aw.cli.v1` envelope naming the exact `aw goal` replacement command in
      `next.command`, then returns an error, so the old verb stays
      registered (never a bare clap "unknown subcommand") but never
      re-enters the run engine. `aw wi run <id>` was left unretired by this
      slice pending its clap-parsing home `src/cli/issues.rs` becoming
      in-scope; the remaining #1899 slice retires it too (see the
      `src/cli/issues.rs` entry below).
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/issues.rs"
    action: modify
    section: schema
    description: |
      Issue #1899 R3 (remaining scope after the above slice): `aw wi run
      <id>`'s clap leaf (`IssuesCommand::Run` / `run_wi_run`) is retired the
      same way `aw capability run` already was -- it computes the `aw goal
      wi <id>` replacement and calls the shared
      `run::emit_retired_verb_redirect` helper instead of calling straight
      into `run_wi_root`. `chain.rs`'s `VERB_LIFECYCLE_REGISTRY` reclassifies
      `wi.run` from `Core`/mutating to `Migration`/non-mutating with a
      populated `sunset_criterion`, mirroring `capability.run`'s entry.
      `normalize_legacy_wi_capability_run_command` (added in the prior
      slice) already rewrites a persisted `aw wi run <id>` `next_action` to
      `aw goal wi <id>` on read, so in-flight pre-flip workflows resume
      correctly with no additional chain.rs change needed.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1899: `CapabilityCommand::Run` no longer runs the
      project-wide/single-capability tick loop directly -- it computes the
      `aw goal capability ...` replacement command from the same
      project/capability-id inputs and calls the shared
      `run::emit_retired_verb_redirect` helper. `run_capability_tick`
      becomes `pub(crate)` so `goal.rs`'s `run_goal_capability` (the new
      `aw goal capability --project <p>` project-wide rollup path) can call
      it directly; the redundant `run_capability_root_tick` thin wrapper
      (superseded by `goal.rs` calling `crate::cli::run::run_capability_root`
      directly) is deleted. Every literal `aw capability run ...` string
      previously embedded in agent-facing prose (capability report
      remediation hints, HITL `resume_command` builders, the
      `--non-interactive` bail message) is replaced with the equivalent `aw
      goal capability ...` form.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/run.rs"
    action: add
    section: schema
    description: |
      Issue #1899 R7: `aw goal backlog --project <p>` (`goal.rs`'s
      `run_goal_backlog` thin delegate calls straight into this). New
      `BacklogState` (workspace-scoped, `goal_state_path(root,
      "backlog-<project>")`, id `backlog-<project>`) records the `parked`
      set: WI id -> park reason, so a HITL/hard-blocked WI is not re-probed
      every tick. `list_open_project_issues` resolves the project's
      canonical tracker label, lists every open issue via the configured
      `IssueBackend`, and sorts by the shared
      `crate::cli::issues::priority_rank` (now `pub(crate)`), then numeric
      id, then slug, for deterministic drain order.
      `probe_wi_root_envelope` builds the same `ResolvedRunRoot::Wi`
      envelope `aw goal wi <id>` would emit, with progress streaming
      disabled (a silent probe, not a real tick). `run_backlog_root`:
      short-circuits self-hosting projects via
      `emit_self_hosting_policy_error`; drops parked entries whose WI
      closed since the last drain; walks the priority-ordered open list,
      skipping already-parked ids, probing each candidate, parking (never
      emitting) any that reports `requires_hitl`/`action == "blocked"` with
      the envelope's reason text, and selecting the first
      still-not-terminal, still-not-blocked candidate; persists the updated
      `BacklogState`; and emits one `WorkflowEnvelope` (`kind: "backlog"`,
      `id: <project>`) per invocation -- `action: "dispatch"` with
      `next.command` set to the selected WI's `aw goal wi <id>` (the exact
      string `wi_run_command` builds, per the new `EMIT_REGISTRY` entry in
      `chain.rs`) when a candidate was selected, or a terminal `action:
      "done"` / `completion.workflow_complete: true` envelope reporting the
      full parked set (id + reason) in `agent_prompt` when every open WI is
      either closed or parked. One bounded CLI invocation performs at most
      one probe-and-select tick (never a self-looping multi-WI executor);
      the host's normal per-WI loop drives the selected WI to its own
      terminal via the emitted `aw goal wi <id>`, and the next `aw goal
      backlog` invocation then advances the drain.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/run.rs"
    action: modify
    section: unit-test
    description: |
      #1899 assertion repair: `self_hosting_policy_envelope_is_terminal_
      and_has_no_root_retry` previously banned the substring `aw goal
      capability` anywhere in the serialized policy envelope, which the
      post-retirement policy prose legitimately names. The gate is now
      structural: a recursive `has_command_key` walk asserts no `command`
      key exists anywhere in the envelope JSON, so no executable root
      retry can be handed back while prose may reference the goal forms.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/llm.rs"
    action: modify
    section: unit-test
    description: |
      #1899/#1816 audience-partition alignment: `agent_first_product_
      contracts_reject_removed_architecture` no longer requires the
      canonical product-responsibility phrases in README (human-first
      doc; the canonical phrasing lives in CAPABILITIES.md and the TD
      contracts). README stays in the removed-semantics negative loop so
      it can never re-advertise retired architecture.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #1800: `lifecycle_action_for_work_item` (the routing table both
      `aw goal capability`'s bounded-tick rollup and its per-capability
      root share, via `choose_next_action`/`capability_envelope`) trusted
      an active WI's cached `CapabilityWiEvidence.phase` even when the
      tracker had already closed that same issue, so a stale local phase
      (e.g. `cb_genned` left over from before close) could still drive
      `aw td fill <id>` against a WI the backend now reports 404 for. A new
      early check in `lifecycle_action_for_work_item` excludes any evidence
      whose `state == "closed"` from every phase/`expected_command`
      dispatch branch and routes to the existing `ReconcileWiRefs` (`aw wi
      plan --project <p>`) remediation instead, with a reason naming the
      closed-issue exclusion so the loop never re-emits an unexecutable
      command for a WI that is actually closed.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/issues.rs"
    action: modify
    section: schema
    description: |
      Issue #2187: `aw wi plan` now emits a digest-bound independent review
      payload and `aw wi plan-review` validates policy, author/reviewer
      separation, checklist, findings, and exact plan/manifest digest before
      idempotently publishing bounded claim WIs. Default/either policy is
      agent-first; explicit human-only policy remains blocking and
      `needs_revision` publishes nothing. Closed or missing doc-stored WI refs
      remain advisory reconciliation evidence and do not suppress a bounded
      replacement candidate for an otherwise unverified claim.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/capability.rs"
    action: modify
    section: schema
    description: |
      Issue #2187: capability roots surface pending plan evidence as an agent
      review action rather than unconditional HITL, and published claim WIs
      route through the EC-first TD/codegen lifecycle until claim evidence
      closes.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/run.rs"
    action: modify
    section: schema
    description: |
      Issue #2187: capability-plan review maps to the workflow envelope's
      `agent_review` action.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/chain.rs"
    action: modify
    section: schema
    description: |
      Issue #2187: classify `wi.plan-review` as a mutating core lifecycle leaf
      and register its emitted evidence-consumer command for chain validation.
    impl_mode: hand-written
  - path: "apps/agentic-workflow/src/cli/llm.rs"
    action: modify
    section: schema
    description: |
      Issue #2187: the WI orientation documents agent-first capability-plan
      review, explicit human-only opt-in, and accepted versus needs_revision
      publication behavior.
    impl_mode: hand-written
```
