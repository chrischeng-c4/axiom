# Agentic Workflow Capabilities

## Brief

Machine-readable capability contract for Agentic Workflow.

## Capabilities

Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| AW Agent-First CLI Model | #1496 | implemented | verified | smoke | ready | verified; one coding-agent CLI owns next-action guidance, artifact skeletons, strict validation/phases, codegen, WorkItem-first admission, and evidence-backed rollup |
| Workflow Root Runner | - | implemented | verified | smoke | ready | verified; CLI workflow chain and root-to-child rollup contract |
| Capability Control Plane | - | implemented | verified | smoke | ready | verified; CAPABILITIES.md capability map, `aw capability`, and verification summaries |
| Work Item Planning | - | implemented | verified | smoke | ready | verified; epic/change split and bounded planning artifacts |
| TD/CB Lifecycle Automation | - | implemented | verified | smoke | ready | verified; WI to TD to code-check terminal workflow |
| Project-Local TD and EC Gates | #13 | implemented | verified | smoke | ready | verified; TD roots default to `<project.path>/tech-design`, EC contracts default to `<project.path>/external-contracts`, and generated tests/tool configs stay project-local |
| Manual Evidence Artifacts | #57 | implemented | verified | smoke | ready | verified; generated product manuals are tracked as EC evidence artifacts with runner commands and optional media |
| Existing Project Standardization | - | implemented | verified | smoke | ready | verified; takeover readiness, managed/semantic/traceability gates, and generator gap requests |

### AW Agent-First CLI Model

ID: aw-core-client-model-workitem-first-artifact-lifecycle
Type: DeveloperTool
Surfaces:
- CLI: `aw wi` + `aw ec` + `aw td` + `aw wi run`/`aw capability run` - the single agent-first project-iteration surface.
EC Dimensions:
- behavior: WorkItem-first artifact admission, agent-first CLI ownership, strict validation/phase transitions, codegen, and evidence-backed rollup semantics.
Root WI: #1496
Status: verified
Required Verification: smoke
Promise:
Agentic Workflow (`aw`) is an agent-first project-iteration CLI for coding agents. It owns next-action guidance, durable artifact skeletons, strict format and phase validation, and code generation, with WorkItem-first admission and evidence-backed rollup.
Gate Inventory:
- apps/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md; apps/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md; apps/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Core concept model and invariants | change | #3894 | implemented | verified | smoke | apps/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md |
| WorkItem artifact admission gate | change | #3895 | implemented | verified | smoke | apps/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md |
| Agent-first CLI product model | change | #1496 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib agent_first_product_contracts_reject_removed_architecture -- --nocapture`; apps/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md |
| Repo View command and desktop product removal | change | #1502 | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests legacy_cli_removal_test -- --nocapture`; apps/agentic-workflow/tech-design/surface/specs/aw-view-removal.md |
| Cross-checkout chat command and shared-channel removal | change | #1503 | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests legacy_cli_removal_test -- --nocapture`; `cargo test -p agentic-workflow --lib test_install_skills_prunes_aw_chat_listen -- --nocapture`; apps/agentic-workflow/tech-design/surface/specs/aw-chat-removal.md |
| Agent orientation surface | change | #178 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib llm_outline_uses_cli_std_and_standard_commands`; apps/agentic-workflow/tech-design/logic/aw-llm-offline-agent-orientation-command.md |
| WorkItem loop-state model | change | #189 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib loop_state_round_trips`; apps/agentic-workflow/tech-design/logic/workitem-loop-state-model-additive-foundation.md |
| AW epic project label dispatch | change | #1518 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib epic_project_label_dispatch_ -- --nocapture`; apps/agentic-workflow/tech-design/semantic/aw-epic-project-label-dispatch.md |

### Workflow Root Runner

ID: workflow-root-runner
Type: DeveloperTool
Surfaces:
- CLI: `aw wi run <id>` / `aw capability run [<cap-id>] --project <p>` - root-scoped WI, capability, and project workflow runners for coding agents.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --lib emit_registry_entries_are_all_chain_valid` - root parsing and JSON envelope contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
`aw wi run`/`aw capability run` emit a CLI workflow chain from project, capability, epic, or change roots and keep rolling work upward until the project root is complete or blocked.
Gate Inventory:
- apps/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CLI workflow chain | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib emit_registry_entries_are_all_chain_valid` |
| Root envelope completion contract | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib create_wi_blocks_on_pending_epicize_artifact` |
| Parent rollup routing | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib closed_change_outputs_parent_inspection` |
| Runtime Envelope Backward Compatibility | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib envelope_profile -- --nocapture`; apps/agentic-workflow/tech-design/specs/3903.md |

### Capability Control Plane

ID: capability-control-plane
Type: DeveloperTool
Surfaces:
- CLI: `aw capability` - report, next, draft, migrate, check, init, sweep, and contract field setters.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --lib markdown_capability_tables` - Markdown capability-document contract parsing, migration, and readiness reporting.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Project capability documents can describe capabilities as readable Markdown headings and tables while detailed proof lives in validation inventories and external contracts.
Gate Inventory:
- apps/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Markdown capability schema | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib markdown_capability_tables` |
| Capability readiness reporting | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib fixture_reference_can_verify_required_claim` |
| Capability project sweep | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib capability_sweep`; human sweep queue output reviewed through aw capability sweep |
| Missing README initialization | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib capability_init`; README shell init behavior only, no runtime project mutation gate |

### Work Item Planning

ID: work-item-planning
Type: DeveloperTool
Surfaces:
- CLI: `aw wi` - inventory, validation drafting, epicization, atomization, prioritization, and issue updates.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --lib epicize_artifact_includes_markdown_capability_roots` - capability-to-WI planning projection.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Capability information can be projected into epic roots, and epic roots can be atomized into bounded change WIs for agent-sized execution.
Gate Inventory:
- apps/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Capability to epic planning | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib epicize_artifact_includes_markdown_capability_roots` |
| Epic to change atomization | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib prioritize_lanes_put_bounded_bug_in_ready_now` |
| Wi Create Help Smoke | change | - | implemented | verified | smoke | `./target/debug/aw wi create --help`; apps/agentic-workflow/tech-design/specs/3909.md |
| Wi Create Remote Flag Tests | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib wi_create_remote -- --nocapture`; apps/agentic-workflow/tech-design/specs/3909.md |
| Wi Remove Agent Estimate Unit Command | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib wi_remove_agent_estimate -- --nocapture`; apps/agentic-workflow/tech-design/specs/3910.md |
| WI close remote rehydration | change | #1551 | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests wi_close_remote_ -- --nocapture`; apps/agentic-workflow/tech-design/semantic/wi-close-remote-rehydration.md; #1583 is a duplicate field reproducer |
| Linear WI authoring without CRRR | change | #1504 | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests legacy_cli_removal_test -- --nocapture`; `cargo test -p agentic-workflow --lib runtime::session -- --nocapture`; apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md; WI and draft authoring terminate at validation, with generic review/arbitration/runtime roles removed and older tracker review fields retained only for compatibility |

### TD/CB Lifecycle Automation

ID: td-cb-lifecycle-automation
Type: DeveloperTool
Surfaces:
- CLI: `aw td` - tech-design lifecycle plus inherited code-artifact lifecycle commands.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --lib td_branch_activation_only_uses_main` - TD/CB lifecycle command dispatch and phase rules.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Atomic change WIs can move through TD authoring, code generation, handwrite fill, and code-check with CLI-emitted next steps. The lifecycle is linear (no review/revise ceremony, no merge step); code-check is the terminal step, and the gate that authorizes it is EC, not review.
Gate Inventory:
- apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| TD lifecycle dispatch | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib td_branch_activation_only_uses_main` |
| CB lifecycle dispatch | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_gen_force_regen_verify_parses_without_slug` |
| CRRR removal (linear lifecycle) | change | #191 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib td_created_dispatches_to_gen`; apps/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md |
| Remove TD merge command | change | #914 | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests test_td_merge_subcommand_is_removed`; apps/agentic-workflow/tech-design/surface/validate/tests/td_no_merge_test.md; refs #851 |
| Chain liveness proof | change | #914 | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests chain_liveness`; apps/agentic-workflow/tech-design/surface/validate/tests/chain_liveness_test.md; refs #921 |
| Self-EC fixture-loop gate | change | #1280 | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests fixture_loop -- --nocapture`; apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_self_ec_fixture_loop_gate.rs; apps/agentic-workflow/tech-design/surface/specs/aw-capability-claim-closure-ec-inventory.md; refs #1279, #1280 |
| Hand-written implementation evidence gate | change | #1382 | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests td_no_merge_test:: -- --nocapture`; every hand-written create/modify path must have a committed diff since its exact Td-Init baseline before terminal code-check can close |
| Large explicit-source fillback and lossless exact gen-source | change | #1506 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib explicit_ -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_explicit_large_file_emits_single_dispatch_and_terminal_gen_source_json -- --nocapture`; repo-built `aw td create --from-source <file>` then emitted `aw td gen-source ... --dry-run` on a 315,692-byte fixture |
| Numeric TD skeleton IDs | change | #1521 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib initialize_td_spec_skeleton -- --nocapture`; apps/agentic-workflow/tech-design/logic/numeric-td-skeleton-id.md |
| Committed TD skeleton lifecycle | change | #1580 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib td_skeleton_recovery -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_commits_fresh_numeric_skeleton_once -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_recovers_reachable_locked_legacy_skeleton_once -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_rebased_lifecycle_reprovisions_untracked_legacy_skeleton -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_rejects_authored_tracked_staged_and_sibling_skeleton_states -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_post_gen_and_terminal_phases_reject_untracked_skeleton -- --nocapture`; apps/agentic-workflow/tech-design/logic/numeric-td-skeleton-id.md; apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md; apps/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md; only the sole exact untracked known-empty TD skeleton may cross authoring activation/reset/provision, and queue-start owns its canonical bytes exactly once |
| TD default section queue preservation | change | #1556 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib merge_spec_section_preserves_ -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_replay_does_not_clobber_authored_logic_section -- --nocapture`; apps/agentic-workflow/tech-design/logic/td-default-section-queue-preservation.md |
| TD apply section lookup parity | change | #1562 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib normalize_generic_td_section_payload -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test -- --nocapture`; apps/agentic-workflow/tech-design/semantic/aw-td-apply-section-lookup-parity.md |
| TD merged candidate in memory validation | change | #1586 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib merged_td_candidate_validation -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_apply_validates_merged_candidate_in_memory_before_write -- --nocapture`; apps/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-runner-rs.md; apps/agentic-workflow/tech-design/surface/interfaces/src/td.md; apps/agentic-workflow/tech-design/surface/validate/tests/inplace_mode_test.md; apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md; apps/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md; merged section candidates run the complete shared registry before write while completed specs remain file-backed |
| Generated TD lock commit handoff | change | #1587 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib td_lock_commit_ -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_default_changes_queue_applies_both_passes_then_gen_uses_explicit_target -- --nocapture`; apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md; apps/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md; apps/agentic-workflow/tech-design/surface/validate/tests/inplace_mode_test.md; the mutating TD-lock command preflights exact in-repository path containment before any write, rejects external directory or lock symlinks without mutation, commits only the configured lock path, preserves unrelated index/worktree state, recovers legacy uncommitted locks once, keeps check/show read-only, and leaves the generated lock clean for the emitted generation command |
| Terminal EC process liveness | change | #1579 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_verify_ -- --nocapture`; `cargo test -p agentic-workflow --lib terminal_ec_gate_rejects_a_duplicate_inflight_inventory -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_code_check_bounds_no_child_ec_wrapper_and_preserves_phase -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_code_check_cross_process_single_flight_prevents_duplicate_ec_launch -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_code_check_fast_green_stale_reader_rechecks_phase_before_ec -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_code_check_retry_contends_while_terminal_transition_holds_lease -- --nocapture`; apps/agentic-workflow/tech-design/semantic/aw-terminal-vat-ec-process-lifecycle.md |
| Default TD target-plan queue | change | #1598 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib target_plan -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_default_changes_queue_applies_both_passes_then_gen_uses_explicit_target -- --nocapture`; apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md; fresh TD applicability and contract passes dispatch an editable Logic -> Changes -> Unit Test queue whose explicit target plan reaches code generation |
| Rebased TD lifecycle recovery | change | #1602 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib rebased_td_lifecycle -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_rebased_lifecycle -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_create_on_project_branch_stays_on_current_branch -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_code_check_refuses_unchanged_hand_written_modify_paths -- --nocapture`; apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md; apps/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md; active authoring state without a reachable exact Td-Init is reset and safely re-provisioned, while a reachable baseline resumes unchanged and code-check evidence remains fail-closed |
| Canonical app/lib TD fill scope reconciliation | bug | #1638 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib canonical_td_changes_path_queues_app_marker`; `cargo test -p agentic-workflow --lib whole_worktree_walk_includes_apps_and_libs`; canonical `apps/<name>` and `libs/<name>` Changes paths queue their HANDWRITE markers and zero-marker re-entry records the normal filled lifecycle phase |
| TD generation project-root precedence | bug | #1705 | implemented | verified | smoke | `cargo test -p agentic-workflow td_gen_prefers_project_default_over_foreign_legacy_spec --lib -- --nocapture`; a hydrated project-labelled issue selects its configured project TD root before foreign legacy `.aw/tech-design` discovery |
| Scoped TD fill marker completion | bug | #1717 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_fill_apply_scopes_remaining_markers_to_active_changes -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_apply_marker_replaces_block -- --nocapture`; post-apply marker queues remain bounded to the active TD Changes paths, so foreign HANDWRITE gaps cannot block this work item's code-check |
| Ambiguous multi-target generation preflight | change | #1633 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib generation_plan -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_gen_ambiguous_schema_plan_fails_before_any_lifecycle_mutation -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests td_gen_no_changes_single_inferred_schema_target_remains_compatible -- --nocapture`; apps/agentic-workflow/tech-design/semantic/td-generation-target-ownership.md; Schema/CLI whole-section plans require exactly one explicit or read-only inferred CODEGEN destination before issue hydration, branch activation, source/lifecycle/tracker/index/HEAD mutation, emit typed remediation for zero or sorted ambiguous targets, preserve single, inferred-single, and mixed CODEGEN/HANDWRITE plans, and leave canonical multi-target `generates:` ownership to #1634 |
| Terminal touched CODEGEN drift gate | change | #1635 | implemented | verified | smoke | `cargo test -p agentic-workflow touched_codegen_claims_select_changed_accepted_codegen_only -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_code_check_terminal_touched_codegen_red_repair_green_unrelated_and_retry -- --nocapture`; apps/agentic-workflow/tech-design/semantic/td-code-check-touched-codegen-drift.md; numeric/slug terminal code-check resolves accepted CODEGEN claims changed since the exact Td-Init baseline, reuses path-mode deterministic block comparison before EC or mutation, ignores unrelated drift, and emits phase-safe exact-target `aw td gen <slug>` repair |
| Quality Primitive Metadata Contract Test | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib quality_primitives -- --nocapture`; apps/agentic-workflow/tech-design/specs/3905.md |
| Missing Source Review Fails | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib source_reference_missing_required_source -- --nocapture`; apps/agentic-workflow/tech-design/specs/3907.md |
| Api Contract Source Passes | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib source_reference_api_contract_source_backed -- --nocapture`; apps/agentic-workflow/tech-design/specs/3907.md |
| Placeholder Completeness Unit Gate | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib completeness_placeholder -- --nocapture`; apps/agentic-workflow/tech-design/specs/3908.md |
| td surface convergence (EC-gated terminal, check unification, verb lifecycle policy, fixture-loop self-EC) | epic | #1270 | implemented | passing | smoke | children #1272-#1281 + #858 (rescoped); gates: chain conformance per removal, fixture-loop e2e (#1279), self-EC inventory (#1280) |

### Project-Local TD and EC Gates

ID: project-local-td-and-ec-gates
Type: DeveloperTool
Surfaces:
- CLI: `aw ec` + `aw td check` - project-local external-contract, generated gate, and TD validation commands.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --lib ec_draft_fill_markdown_drives_inventory` - EC markdown source, aw.toml inventory, and generated tool manifest contract.
- behavior: `cargo test -p agentic-workflow --lib ec_verify_rejects_zero_test_false_green -- --nocapture` - EC verification rejects cargo-test false greens that run zero tests and keeps precise cargo target selectors when known.
- stability: `cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design` - project-local TD root resolution and dirty-scope protection.
Root WI: #13
Status: verified
Required Verification: smoke
Promise:
AW-managed projects keep their README, external contracts, tech designs, source, tests, and generated tool configs under the project tree by default: `td_path` is only an override, EC contracts live under `<project.path>/external-contracts`, and the generated EC inventory lives in the project `aw.toml` AW-EC block.
Gate Inventory:
- `cargo test -p agentic-workflow --lib falls_back_to_project_tech_design`; `cargo test -p agentic-workflow --lib ec_context_defaults_td_root_to_project_tech_design`; `cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design`; `cargo test -p agentic-workflow --lib semantic_coverage_excludes_aw_ec_generated_wrappers`; `cargo test -p agentic-workflow --lib ec_doc`; `cargo test -p agentic-workflow --lib ec_verify_rejects_zero_test_false_green -- --nocapture`; `aw td check apps/agentic-workflow/tech-design/core/specs/td-root-resolver.md`; `aw td check apps/agentic-workflow/tech-design/core/interfaces/services/project_registry.md`; `aw td check apps/agentic-workflow/tech-design/surface/interfaces/src/cb.md`; `aw td check apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Project-local TD root resolver | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib falls_back_to_project_tech_design`; `aw td check apps/agentic-workflow/tech-design/core/specs/td-root-resolver.md` |
| Project label producer TD routing | change | #1519 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib project_label_canonicalization -- --nocapture`; `cargo test -p agentic-workflow --lib default_td_spec_path_errors_loudly_on_unrecognized_project_label -- --nocapture`; `cargo test -p agentic-workflow --lib explicit_diff_ -- --nocapture`; `cargo test -p agentic-workflow --lib rest_patch_args_distinguishes_omitted_from_explicit_empty_labels -- --nocapture`; apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md; apps/agentic-workflow/tech-design/core/specs/td-root-resolver.md; apps/agentic-workflow/tech-design/core/logic/issues-backend.md; registered library rows produce canonical lib labels that reach their project-local TD root, explicit tracker update removals retire stale project labels without deleting unrelated labels (including the final GitHub label), app routing is preserved, and raw retired project labels remain rejected |
| TD lock and external-contract target resolution | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_context_defaults_td_root_to_project_tech_design`; `aw td check apps/agentic-workflow/tech-design/core/interfaces/services/project_registry.md` |
| CB generation and standardize scan defaults | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design`; `aw td check apps/agentic-workflow/tech-design/surface/interfaces/src/cb.md` |
| Project dirty-scope protection | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib semantic_coverage_excludes_aw_ec_generated_wrappers`; `aw td check apps/agentic-workflow/tech-design/surface/interfaces/src/standardize.md` |
| EC evidence documentation | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_doc` |
| EC external-contract source | change | #13 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_draft_fill_markdown_drives_inventory`; aw ec draft/fill authors project-local external-contract markdown and aw ec gen writes the project aw.toml EC inventory plus generated tests and rig/meter/guard/vat tool configs; arena is retained as a legacy compatibility import |
| EC tool binding dispatch | change | #13 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_binding_command`; `cargo test -p agentic-workflow --lib resolve_ec_command_dispatches_bound_category`; apps/agentic-workflow/tech-design/config/ec-tool-binding-config-ec-category-verify-ec-dispatch-with-manif.md; apps/agentic-workflow/tech-design/logic/aw-ec-add-vat-binding-command-support.md |
| EC false-green guard | change | #694 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_verify_rejects_zero_test_false_green -- --nocapture`; apps/agentic-workflow/tech-design/surface/specs/aw-ec-zero-test-false-green.md |
| EC-only independent semantic approval | change | #1504 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_review_ -- --nocapture`; `cargo test -p agentic-workflow --lib terminal_ec_missing_semantic_review_routes_to_hitl -- --nocapture`; apps/agentic-workflow/tech-design/surface/specs/aw-ec-only-semantic-approval.md; production EC generation and verification require current digest-bound human acceptance after dimension, claim, assertion, oracle-independence, loophole, and false-green inspection |

### Manual Evidence Artifacts

ID: manual-evidence-artifacts
Type: DeveloperTool
Surfaces:
- CLI: `aw ec doc` - generated, checked, or previewed EC-derived product documentation evidence.
EC Dimensions: behavior: `cargo test -p agentic-workflow --lib ec_doc_gen_writes_manual_from_inventory` - generated manual artifact schema and output convention
Root WI: #57
Status: verified
Required Verification: smoke
Promise:
AW treats generated product manuals as first-class EC evidence artifacts. A manual artifact records its project-local output path, the runner command that produces it, and optional screenshots, highlights, or step metadata without requiring every manual to use a visual overlay recorder.
Gate Inventory:
- apps/agentic-workflow/src/tools/common_change_spec.rs; apps/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md; /Users/chris.cheng/projects/ai-studio/docs/user-manual

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Generated manual EC evidence schema | change | #57 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_generated_manual_artifact` |
| Manual runner output convention | change | #57 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_doc_gen_writes_manual_from_inventory`; apps/agentic-workflow/src/tools/common_change_spec.rs |

### Existing Project Standardization

ID: existing-project-standardization
Type: DeveloperTool
Surfaces:
- CLI: `aw health` (takeover-audit axis) + `aw td audit-record` - brownfield takeover guidance and readiness rollup (#1278, epic #1270 R7: `aw standardize` namespace retired, reporting folded into `aw health`, `audit record` rehomed to `aw td`).
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --test cli_tests standardize_subcommands_registered` - takeover command surface and readiness reporting.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Existing projects can be adopted one bounded tick at a time: capability readiness stays in `aw capability`, takeover runs through managed/semantic/traceability, and generator gaps route back into normal WI/TD/CB work.
Gate Inventory:
- apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Brownfield takeover surface | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests standardize_subcommands_registered` |
| Managed and semantic production gates | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib semantic_coverage_prioritizes_missing_td_before_generator_gap` |
| Authoritative source-snapshot projection | change | #1548 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib legacy_source_snapshot -- --nocapture`; `cargo test -p agentic-workflow --test cli_tests test_gen_source_projects_legacy_snapshot_and_runs_generated_test -- --nocapture`; apps/agentic-workflow/tech-design/semantic/td-gen-source-source-snapshot-projection.md |
| Traceability closure gate | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib traceability` covers command, TD, source, and CB closure |
| CB and cold verification gates | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_gen_cold_rebuild_targets_include_codegen_changes` |
| Force-regeneration project-root llms projection | change | #1591 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_gen_force_regen_public_path_emits_td_first_project_root_llms -- --nocapture`; `cargo test -p agentic-workflow --lib cb_gen_project_root_llms -- --nocapture`; `cargo test -p agentic-workflow --lib cb_gen_force_regen_specs_do_not_format_handwritten_skips -- --nocapture`; apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md; the public force-regeneration path shares the TD-first project-root llms emitter used by replay/cold verification, replaces generic placeholders with one canonical CODEGEN document, and preserves HANDWRITE siblings byte-for-byte |
| Shared service kit substrate | change | #1241 | implemented | verified | smoke | `cargo test -p server-core -p tcp-server -p http-server -p h2c -p service-http`; apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md |
| Regenerability maturity loop (optional) | epic | - | out_of_scope | none | none | - |
| Authoritative Fixture Blocks On Regenerability Gap | change | - | implemented | verified | smoke | `bash apps/agentic-workflow/tests/fixtures/regenerability_authority/assert_authoritative_blocker.sh`; apps/agentic-workflow/tech-design/specs/3901.md |
| External Fixture Reports Advisory Gap | change | - | implemented | verified | smoke | `bash apps/agentic-workflow/tests/fixtures/regenerability_authority/assert_external_advisory.sh`; apps/agentic-workflow/tech-design/specs/3901.md |
| Project Health No Regression | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib project_health -- --nocapture`; apps/agentic-workflow/tech-design/specs/3903.md |
| Artifact Preflight Health Rollup | change | - | implemented | verified | smoke | `./target/debug/aw health --project agentic-workflow | tail -n 1 | grep -q axes`; apps/agentic-workflow/tech-design/specs/3904.md |
| Standardize Audit First Contract Test | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib standardize_audit -- --nocapture`; apps/agentic-workflow/tech-design/specs/3906.md |
| Aw Health Default Full Verification Smoke | change | - | implemented | verified | smoke | `./target/debug/aw health --project agentic-workflow | tail -n 1 | grep -q payload_path`; apps/agentic-workflow/tech-design/validate/health-defaults-to-streaming-full-verification.md |
