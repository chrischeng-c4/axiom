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
    command: ./target/debug/aw td check projects/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md
    assertions:
      - core client model TD validates as the source claim for AW core invariants
  - id: aw-core-client-workitem-artifact-admission-gate
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: workitem-artifact-admission-gate
    command: ./target/debug/aw td check projects/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md
    assertions:
      - workitem artifact gate TD validates as the admission gate claim
  - id: aw-core-client-client-boundary-model
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: client-boundary-model
    command: ./target/debug/aw td check projects/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md
    assertions:
      - client boundary TD validates as the shared client boundary claim
  - id: aw-core-client-agent-orientation-surface
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: agent-orientation-surface
    command: cargo test -p agentic-workflow --lib llm_outline_uses_cli_std_and_standard_commands -- --nocapture
    assertions:
      - agent-facing llm outline lists the registered command surface
  - id: aw-core-client-workitem-loop-state-model
    capability_id: aw-core-client-model-workitem-first-artifact-lifecycle
    claim_id: workitem-loop-state-model
    command: cargo test -p agentic-workflow --lib loop_state_round_trips -- --nocapture
    assertions:
      - work-item loop state serializes and parses losslessly
  - id: workflow-root-runner-cli-workflow-chain
    capability_id: workflow-root-runner
    claim_id: cli-workflow-chain
    command: cargo test -p agentic-workflow --lib root_parser_accepts_capability_and_wi_roots -- --nocapture
    assertions:
      - aw run root parser accepts capability and work-item roots
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
