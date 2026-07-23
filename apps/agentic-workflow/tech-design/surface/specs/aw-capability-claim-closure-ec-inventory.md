---
id: aw-capability-claim-closure-ec-inventory
summary: Map verified Agentic Workflow README work-root claims to required EC cases.
capability_refs:
  - id: capability-control-plane
    role: primary
    gap: capability-readiness-reporting
    claim: capability-readiness-reporting
    coverage: full
    rationale: "Claim closure requires every verified README work-root claim to have a production EC case."
---

# AW Capability Claim Closure EC Inventory

Agentic Workflow self-health hard-gates capability contracts and EC claim
closure. This inventory turns the existing verified README work-root gates into
project-local EC cases so claim closure can evaluate concrete case IDs rather
than only free-form gate prose.

## Claim Closure EC Cases
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: aw-core-client-core-concept-model-and-invariants
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: core-concept-model-and-invariants
    command: cargo test -p agentic-workflow --test cli_tests fixture_loop_test::fixture_loop_drives_wi_run_to_workflow_complete -- --exact --nocapture
    assertions:
      - "from an admitted CB-generated child, the real compiled goal runner follows emitted CB commands, closes the child, rolls up its epic and capability, and terminates with completion.workflow_complete=true"
  - id: aw-core-client-core-concept-model-ec-first-phase-table
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: core-concept-model-and-invariants
    command: cargo test -p agentic-workflow --lib cli::run::tests::python_artifact_goal_routing_uses_one_ec_first_phase_table -- --exact --nocapture
    assertions:
      - "the Python Spec lifecycle has one explicit EC review, TD behavior/security, CB generation/check, and terminal all-dimension EC routing table with no phase gaps"
  - id: aw-core-client-core-concept-model-phase-less-admission
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: core-concept-model-and-invariants
    command: cargo test -p agentic-workflow --lib cli::run::tests::phase_less_project_wi_enters_ec_before_td -- --exact --nocapture
    assertions:
      - "a phase-less project WorkItem enters EC authoring before any TD authoring command"
  - id: aw-core-client-core-concept-model-remote-ledger-admission
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: core-concept-model-and-invariants
    command: cargo test -p agentic-workflow --lib cli::run::tests::remote_wi_admission_seeds_the_local_ec_lifecycle_ledger -- --exact --nocapture
    assertions:
      - "admitting a remote WorkItem seeds the local EC-first lifecycle ledger before dispatching artifact work"
  - id: aw-core-client-workitem-artifact-admission-gate
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: workitem-artifact-admission-gate
    command: cargo test -p agentic-workflow --test cli_tests inplace_mode_test::workitem_artifact_admission_gate_real_cli_positive_and_negative -- --exact --nocapture
    assertions:
      - "the real compiled CB generator rejects an unsupported or unadmitted artifact before issue, Git, or source mutation and accepts a valid admitted WorkItem with exact generated ownership"
  - id: aw-core-client-agent-first-cli-product-model
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: agent-first-cli-product-model
    command: cargo test -p agentic-workflow --lib agent_first_product_contracts_reject_removed_architecture -- --nocapture
    assertions:
      - binary orientation, README, capability contract, and canonical product TDs share one agent-first CLI model and reject removed architecture prose
  - id: aw-core-client-agent-orientation-surface
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: agent-orientation-surface
    command: cargo test -p agentic-workflow --lib llm_outline_uses_cli_std_and_standard_commands -- --nocapture
    assertions:
      - agent-facing llm outline lists the registered command surface
  - id: aw-core-client-prompt-vocabulary-and-grammar
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: prompt-vocabulary-and-grammar
    command: cargo test -p agentic-workflow --lib cli::llm::tests::prompt_topic_public_renderer_pins_closed_language -- --exact --nocapture
    assertions:
      - "the registered public Markdown and JSON renderers expose identical prompt content with the exact closed vocabulary, seven ASCII operators, complete EC-first Python Spec transition table, sole workflow-authority boundary, and no Unicode operator lookalikes"
  - id: aw-core-client-typed-prompt-ir-and-envelope-projection
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: typed-prompt-ir-and-envelope-projection
    command: cargo test -p agentic-workflow --lib cli::run::tests::workflow_envelope_serializes_typed_prompt_contract_from_same_ir -- --exact --nocapture
    assertions:
      - "a production WorkflowEnvelope pins every typed prompt JSON field and its rendered agent_prompt from the same decoded IR"
      - "an invalid typed contract makes WorkflowEnvelope serialization fail instead of falling back to prose"
  - id: aw-core-client-lifecycle-prompt-stage-conformance
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: lifecycle-prompt-migration-and-conformance
    command: cargo test -p agentic-workflow --lib cli::run::tests::python_artifact_prompt_contracts_preserve_stage_owner_and_gate -- --exact --nocapture
    assertions:
      - "every Python EC, TD, and CB phase-table row, including EC review and change close, projects exact writable and read-only scopes, verifier predicate, terminal level, and lifecycle guard"
      - "a frontend CB transition projects the complete concrete artifact-quality guard id set"
  - id: aw-core-client-lifecycle-prompt-blocker-conformance
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: lifecycle-prompt-migration-and-conformance
    command: cargo test -p agentic-workflow --lib cli::run::tests::prompt_contract_routes_invalid_oracle_and_typed_blockers -- --exact --nocapture
    assertions:
      - "invalid oracle state routes to EC repair and decision, approval, environment, red-gate, and missing-evidence blockers remain typed with exact resume"
  - id: aw-core-client-lifecycle-prompt-rollup-conformance
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: lifecycle-prompt-migration-and-conformance
    command: cargo test -p agentic-workflow --lib cli::run::tests::prompt_contract_distinguishes_child_parked_and_root_terminal -- --exact --nocapture
    assertions:
      - "child dispatch, parked backlog work, and root terminal completion are distinct prompt states"
  - id: aw-core-client-workitem-loop-state-model
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: workitem-loop-state-model
    command: cargo test -p agentic-workflow --lib loop_state_round_trips -- --nocapture
    assertions:
      - work-item loop state serializes and parses losslessly
  - id: workflow-root-runner-cli-workflow-chain
    capability_id: workflow-root-runner
    claim_id: cli-workflow-chain
    command: cargo test -p agentic-workflow --lib emit_registry_entries_are_all_chain_valid -- --nocapture
    assertions:
      - CLI workflow chain emit sites resolve through the real clap tree for capability and work-item roots
  - id: workflow-root-runner-root-envelope-completion-contract
    capability_id: workflow-root-runner
    claim_id: root-envelope-completion-contract
    command: cargo test -p agentic-workflow --lib create_wi_blocks_on_pending_epicize_artifact -- --nocapture
    assertions:
      - root envelope blocks on pending epicize artifacts before creating WIs
  - id: workflow-root-runner-parent-rollup-routing
    capability_id: workflow-root-runner
    claim_id: parent-rollup-routing
    command: cargo test -p agentic-workflow --lib closed_change_outputs_parent_inspection -- --nocapture
    assertions:
      - closed change roots route agents to parent inspection
  - id: capability-control-plane-markdown-capability-schema
    capability_id: capability-control-plane
    claim_id: markdown-capability-schema
    command: cargo test -p agentic-workflow --lib markdown_capability_tables -- --nocapture
    assertions:
      - canonical field-style capability contracts parse from Markdown
  - id: capability-control-plane-capability-readiness-reporting
    capability_id: capability-control-plane
    claim_id: capability-readiness-reporting
    command: cargo test -p agentic-workflow --lib fixture_reference_can_verify_required_claim -- --nocapture
    assertions:
      - required claims can be verified by fixture references
  - id: capability-control-plane-capability-project-sweep
    capability_id: capability-control-plane
    claim_id: capability-project-sweep
    command: cargo test -p agentic-workflow --lib capability_sweep -- --nocapture
    assertions:
      - capability sweep groups project readiness and next actions
  - id: capability-control-plane-missing-readme-initialization
    capability_id: capability-control-plane
    claim_id: missing-readme-initialization
    command: cargo test -p agentic-workflow --lib capability_init -- --nocapture
    assertions:
      - capability init renders a canonical README shell
  - id: work-item-planning-epic-to-change-atomization
    capability_id: work-item-planning
    claim_id: epic-to-change-atomization
    command: cargo test -p agentic-workflow --lib prioritize_lanes_put_bounded_bug_in_ready_now -- --nocapture
    assertions:
      - prioritization routes bounded changes into the ready lane
  - id: td-cb-lifecycle-automation-crrr-removal-linear-lifecycle
    capability_id: td-cb-lifecycle-automation
    claim_id: crrr-removal-linear-lifecycle
    command: cargo test -p agentic-workflow --lib td_created_dispatches_to_gen -- --nocapture
    assertions:
      - TD created phase dispatches directly to generation in the linear lifecycle
  - id: td-cb-lifecycle-automation-self-ec-fixture-loop-gate
    capability_id: td-cb-lifecycle-automation
    claim_id: self-ec-fixture-loop-gate
    command: cargo test -p agentic-workflow --test cli_tests fixture_loop_test::fixture_loop_required_ec_refuses_red_then_records_green_terminal_completion -- --exact --nocapture
    assertions:
      - "a configured required EC case refuses the unchanged CB-filled WorkItem while red without phase or close mutation, then permits terminal close only when green and records the consulted case in the success envelope"
  - id: td-cb-lifecycle-automation-remove-td-merge-command
    capability_id: td-cb-lifecycle-automation
    claim_id: remove-td-merge-command
    command: cargo test -p agentic-workflow --test cli_tests legacy_cli_removal_test::test_td_merge_subcommand_is_removed -- --exact --nocapture
    assertions:
      - "the retired `aw td merge` command is absent from the Clap tree and parsing it returns the literal unrecognized-subcommand failure (#914, refs #851)"
  - id: td-cb-lifecycle-automation-chain-liveness-proof
    capability_id: td-cb-lifecycle-automation
    claim_id: chain-liveness-proof
    command: cargo test -p agentic-workflow --test cli_tests chain_liveness_test::chain_liveness_claim_never_lands_on_deadlock_phase -- --exact --nocapture
    assertions:
      - "the exact driven chain reaches a terminal action within its bounded hop budget without landing on a deadlock phase (#914, refs #921)"
  - id: td-cb-lifecycle-automation-chain-liveness-retry
    capability_id: td-cb-lifecycle-automation
    claim_id: chain-liveness-proof
    command: cargo test -p agentic-workflow --test cli_tests chain_liveness_test::chain_liveness_code_check_retry_recovers_stranded_terminal_within_tick_budget -- --exact --nocapture
    assertions:
      - "a stranded terminal retry emits the exact `aw cb check <slug>` command, remains within its tick budget, and preserves the lifecycle state until successful completion"
  - id: td-cb-lifecycle-automation-hand-written-implementation-evidence-gate
    capability_id: td-cb-lifecycle-automation
    claim_id: hand-written-implementation-evidence-gate
    command: "cargo test -p agentic-workflow --test cli_tests td_no_merge_test::test_code_check_refuses_unchanged_hand_written_modify_paths -- --exact --nocapture"
    assertions:
      - "terminal CB check refuses a hand-written modify path with zero committed implementation diff since its Td-Init baseline (#1382)"
  - id: td-cb-lifecycle-automation-hand-written-partial-evidence-gate
    capability_id: td-cb-lifecycle-automation
    claim_id: hand-written-implementation-evidence-gate
    command: "cargo test -p agentic-workflow --test cli_tests td_no_merge_test::test_code_check_refuses_partial_hand_written_lifecycle_diff -- --exact --nocapture"
    assertions:
      - "terminal CB check refuses partial evidence when any declared hand-written create or modify target still has no committed implementation diff"
  - id: td-cb-lifecycle-automation-hand-written-complete-evidence-gate
    capability_id: td-cb-lifecycle-automation
    claim_id: hand-written-implementation-evidence-gate
    command: "cargo test -p agentic-workflow --test cli_tests td_no_merge_test::test_code_check_accepts_complete_hand_written_lifecycle_diff -- --exact --nocapture"
    assertions:
      - "terminal CB check accepts complete evidence only after every declared hand-written create and modify target has a committed implementation diff"
  - id: td-cb-lifecycle-automation-td-surface-convergence-ec-gated-terminal-check-unification-verb-lifecycle-policy-fixture-loop-self-ec
    capability_id: td-cb-lifecycle-automation
    claim_id: td-surface-convergence-ec-gated-terminal-check-unification-verb-lifecycle-policy-fixture-loop-self-ec
    command: cargo test -p agentic-workflow --test cli_tests fixture_loop_test::fixture_loop_goal_converges_through_cb_to_required_ec_red_green_terminal -- --exact --nocapture
    assertions:
      - "the public goal runner follows CB fill and check, stops at terminal required EC while red, resumes the same CB-filled WorkItem when green, records the consulted case, and closes at the unified terminal check"
  - id: existing-project-standardization-shared-service-kit-substrate
    capability_id: existing-project-standardization
    claim_id: shared-service-kit-substrate
    command: cargo test -p server-tcp tests::serve_accepts_closure_handler_without_async_trait_boxing -- --exact --nocapture
    assertions:
      - "the shared TCP accept loop binds a real listener, admits a connection, invokes the closure handler, and completes without an async-trait box (#1241)"
  - id: existing-project-standardization-shared-service-kit-drain
    capability_id: existing-project-standardization
    claim_id: shared-service-kit-substrate
    command: cargo test -p server-lifecycle --test drain_prestart receiverless_drain_persists_for_late_subscriber -- --exact --nocapture
    assertions:
      - "a drain transition published before subscription remains durable and is observed by a late server-plane subscriber"
  - id: existing-project-standardization-shared-service-kit-connection-budget
    capability_id: existing-project-standardization
    claim_id: shared-service-kit-substrate
    command: cargo test -p server-tcp tests::connection_budget_releases_after_handler_finishes -- --exact --nocapture
    assertions:
      - "connection admission consumes the configured budget and releases the permit after the handler finishes"
  - id: existing-project-standardization-shared-service-kit-http1-h2c-options
    capability_id: existing-project-standardization
    claim_id: shared-service-kit-substrate
    command: cargo test -p server-http tests::serves_http1_and_h2c_on_one_listener_with_tunable_options -- --exact --nocapture
    assertions:
      - "the shared HTTP runtime serves HTTP/1.1 and h2c on one real listener while accepting explicit HTTP/2 stream and drain options"
  - id: existing-project-standardization-shared-service-kit-service-http-delegation
    capability_id: existing-project-standardization
    claim_id: shared-service-kit-substrate
    command: cargo test -p service-http transport::delegation_tests::serve_delegates_listener_to_shared_http_runtime -- --exact --nocapture
    assertions:
      - "the service-http policy shell delegates listener ownership and request dispatch to server-http while preserving the service router response"
  - id: project-local-td-and-ec-gates-project-local-td-root-resolver
    capability_id: project-local-td-and-ec-gates
    claim_id: project-local-td-root-resolver
    command: cargo test -p agentic-workflow --lib falls_back_to_project_tech_design -- --nocapture
    assertions:
      - project-local TD root resolution falls back to the project tech-design directory
  - id: project-local-td-and-ec-gates-td-lock-and-external-contract-target-resolution
    capability_id: project-local-td-and-ec-gates
    claim_id: td-lock-and-external-contract-target-resolution
    command: cargo test -p agentic-workflow --lib ec_context_defaults_td_root_to_project_tech_design -- --nocapture
    assertions:
      - EC context defaults TD roots to the project tech-design directory
  - id: project-local-td-and-ec-gates-cb-generation-and-standardize-scan-defaults
    capability_id: project-local-td-and-ec-gates
    claim_id: cb-generation-and-standardize-scan-defaults
    command: cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design -- --nocapture
    assertions:
      - CB force regeneration defaults to project-local tech-design roots
  - id: project-local-td-and-ec-gates-project-dirty-scope-protection
    capability_id: project-local-td-and-ec-gates
    claim_id: project-dirty-scope-protection
    command: cargo test -p agentic-workflow --lib semantic_coverage_excludes_aw_ec_generated_wrappers -- --nocapture
    assertions:
      - semantic coverage excludes generated EC wrappers from dirty source scope
  - id: project-local-td-and-ec-gates-ec-evidence-documentation
    capability_id: project-local-td-and-ec-gates
    claim_id: ec-evidence-documentation
    command: cargo test -p agentic-workflow --lib ec_doc -- --nocapture
    assertions:
      - EC documentation generation and drift checks are covered
  - id: project-local-td-and-ec-gates-ec-external-contract-source
    capability_id: project-local-td-and-ec-gates
    claim_id: ec-external-contract-source
    command: cargo test -p agentic-workflow --lib ec_draft_fill_markdown_drives_inventory -- --nocapture
    assertions:
      - EC draft and fill Markdown drives inventory generation
  - id: project-local-td-and-ec-gates-ec-tool-binding-dispatch
    capability_id: project-local-td-and-ec-gates
    claim_id: ec-tool-binding-dispatch
    command: cargo test -p agentic-workflow --lib ec_binding_command -- --nocapture
    assertions:
      - EC tool binding commands resolve the configured runner dispatch
  - id: manual-evidence-artifacts-generated-manual-ec-evidence-schema
    capability_id: manual-evidence-artifacts
    claim_id: generated-manual-ec-evidence-schema
    command: cargo test -p agentic-workflow --lib ec_generated_manual_artifact -- --nocapture
    assertions:
      - generated manual EC evidence metadata validates
  - id: manual-evidence-artifacts-manual-runner-output-convention
    capability_id: manual-evidence-artifacts
    claim_id: manual-runner-output-convention
    command: cargo test -p agentic-workflow --lib ec_doc_gen_writes_manual_from_inventory -- --nocapture
    assertions:
      - EC doc generation writes the manual from inventory
  - id: existing-project-standardization-brownfield-takeover-surface
    capability_id: existing-project-standardization
    claim_id: brownfield-takeover-surface
    command: cargo test -p agentic-workflow --test cli_tests standardize_subcommands_registered -- --nocapture
    assertions:
      - standardize command surface is registered for brownfield takeover
  - id: existing-project-standardization-managed-and-semantic-production-gates
    capability_id: existing-project-standardization
    claim_id: managed-and-semantic-production-gates
    command: cargo test -p agentic-workflow --lib semantic_coverage_prioritizes_missing_td_before_generator_gap -- --nocapture
    assertions:
      - semantic coverage prioritizes missing TD before generator gaps
  - id: existing-project-standardization-traceability-closure-gate
    capability_id: existing-project-standardization
    claim_id: traceability-closure-gate
    command: cargo test -p agentic-workflow --lib traceability -- --nocapture
    assertions:
      - traceability tests cover command, TD, source, and CB closure behavior
  - id: existing-project-standardization-cb-and-cold-verification-gates
    capability_id: existing-project-standardization
    claim_id: cb-and-cold-verification-gates
    command: cargo test -p agentic-workflow --lib cb_gen_cold_rebuild_targets_include_codegen_changes -- --nocapture
    assertions:
      - CB cold rebuild targets include codegen changes
```
