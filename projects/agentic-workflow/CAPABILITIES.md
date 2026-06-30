# Agentic Workflow Capabilities

## Brief

Machine-readable capability contract for Agentic Workflow. The project overview lives in [README.md](README.md).

## Capabilities

Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| AW Core Client Model | #3894 | implemented | verified | smoke | ready | verified; shared AW Core nouns, WorkItem-first artifact admission, and client boundaries |
| Workflow Root Runner | - | implemented | verified | smoke | ready | verified; CLI workflow chain and root-to-child rollup contract |
| Capability Control Plane | - | implemented | verified | smoke | ready | verified; CAPABILITIES.md capability map, `aw capability`, and verification summaries |
| Work Item Planning | - | implemented | verified | smoke | ready | verified; epic/change split and bounded planning artifacts |
| TD/CB Lifecycle Automation | - | implemented | verified | smoke | ready | verified; WI to TD to CB to merge workflow |
| Project-Local TD and EC Gates | #13 | implemented | verified | smoke | ready | verified; TD roots default to `<project.path>/tech-design`, EC contracts default to `<project.path>/external-contracts`, and generated tests/tool configs stay project-local |
| Manual Evidence Artifacts | #57 | implemented | verified | smoke | ready | verified; generated product manuals are tracked as EC evidence artifacts with runner commands and optional media |
| Repo View Desktop App | - | implemented | verified | smoke | ready | verified; exposes `aw view`, `aw view --layout left-right|top-bottom`, `aw view --snapshot`, `aw view --check`, `aw view --screenshot <png>`, and `aw view --app <app>` for the native repo reader, in-app layout toggle, EC snapshot, quick headless contract check, app-level visual debug capture, and macOS app bundle |
| Existing Project Standardization | - | implemented | verified | smoke | ready | verified; takeover readiness, managed/semantic/traceability gates, and generator gap requests |

### AW Core Client Model

ID: aw-core-client-model-workitem-first-artifact-lifecycle
Type: DeveloperTool
Surfaces:
- CLI: `aw wi` + `aw td` + `aw run` - standalone AW Core client entrypoints over the shared workflow protocol.
EC Dimensions:
- behavior: shared WorkItem-first artifact admission, client boundary, and rollup semantics from the AW Core TD set.
Root WI: #3894
Status: verified
Required Verification: smoke
Promise:
AW Core defines the client-independent workflow and domain protocol shared by `aw CLI`, Cue, and future clients, with WorkItem-first artifact admission and evidence-backed rollup.
Gate Inventory:
- projects/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md; projects/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md; projects/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Core concept model and invariants | change | #3894 | implemented | verified | smoke | projects/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md |
| WorkItem artifact admission gate | change | #3895 | implemented | verified | smoke | projects/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md |
| Client boundary model | change | #3896 | implemented | verified | smoke | projects/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md |
| Agent orientation surface | change | #178 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib llm_outline_uses_cli_std_and_standard_commands`; projects/agentic-workflow/tech-design/logic/aw-llm-offline-agent-orientation-command.md |
| WorkItem loop-state model | change | #189 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib loop_state_round_trips`; projects/agentic-workflow/tech-design/logic/workitem-loop-state-model-additive-foundation.md |

### Workflow Root Runner

ID: workflow-root-runner
Type: DeveloperTool
Surfaces:
- CLI: `aw run` - root-scoped project, capability, and WI workflow runner for coding agents.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --lib root_parser_accepts_capability_and_wi_roots` - root parsing and JSON envelope contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
`aw run` emits a CLI workflow chain from project, capability, epic, or change roots and keeps rolling work upward until the project root is complete or blocked.
Gate Inventory:
- projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CLI workflow chain | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib root_parser_accepts_capability_and_wi_roots` |
| Root envelope completion contract | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib create_wi_blocks_on_pending_epicize_artifact` |
| Parent rollup routing | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib closed_change_outputs_parent_inspection` |

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
- projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

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
- projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Capability to epic planning | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib epicize_artifact_includes_markdown_capability_roots` |
| Epic to change atomization | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib prioritize_lanes_put_bounded_bug_in_ready_now` |

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
Atomic change WIs can move through TD authoring, code generation, handwrite fill, and merge with CLI-emitted next steps. The lifecycle is linear (no review/revise ceremony); the gate that authorizes merge is EC, not review.
Gate Inventory:
- projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| TD lifecycle dispatch | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib td_branch_activation_only_uses_main` |
| CB lifecycle dispatch | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_gen_force_regen_verify_parses_without_slug` |
| CRRR removal (linear lifecycle) | change | #191 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib td_created_dispatches_to_gen`; projects/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md |

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
- `cargo test -p agentic-workflow --lib falls_back_to_project_tech_design`; `cargo test -p agentic-workflow --lib ec_context_defaults_td_root_to_project_tech_design`; `cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design`; `cargo test -p agentic-workflow --lib semantic_coverage_excludes_aw_ec_generated_wrappers`; `cargo test -p agentic-workflow --lib ec_doc`; `cargo test -p agentic-workflow --lib ec_verify_rejects_zero_test_false_green -- --nocapture`; `aw td check projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md`; `aw td check projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/standardize.md`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Project-local TD root resolver | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib falls_back_to_project_tech_design`; `aw td check projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md` |
| TD lock and external-contract target resolution | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_context_defaults_td_root_to_project_tech_design`; `aw td check projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md` |
| CB generation and standardize scan defaults | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md` |
| Project dirty-scope protection | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib semantic_coverage_excludes_aw_ec_generated_wrappers`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/standardize.md` |
| EC evidence documentation | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_doc` |
| EC external-contract source | change | #13 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_draft_fill_markdown_drives_inventory`; aw ec draft/fill authors project-local external-contract markdown and aw ec gen writes the project aw.toml EC inventory plus generated tests and rig/meter/guard/vat tool configs; arena is retained as a legacy compatibility import |
| EC tool binding dispatch | change | #13 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_binding_command`; `cargo test -p agentic-workflow --lib resolve_ec_command_dispatches_bound_category`; projects/agentic-workflow/tech-design/config/ec-tool-binding-config-ec-category-verify-ec-dispatch-with-manif.md; projects/agentic-workflow/tech-design/logic/aw-ec-add-vat-binding-command-support.md |
| EC false-green guard | change | #694 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_verify_rejects_zero_test_false_green -- --nocapture`; projects/agentic-workflow/tech-design/surface/specs/aw-ec-zero-test-false-green.md |

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
- projects/agentic-workflow/src/tools/common_change_spec.rs; projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md; /Users/chris.cheng/projects/ai-studio/docs/user-manual

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Generated manual EC evidence schema | change | #57 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_generated_manual_artifact` |
| Manual runner output convention | change | #57 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_doc_gen_writes_manual_from_inventory`; projects/agentic-workflow/src/tools/common_change_spec.rs |

### Repo View Desktop App

ID: repo-view-desktop-app
Type: DeveloperTool
Surfaces:
- CLI: `aw view` + `aw view --layout left-right|top-bottom` + `aw view --snapshot` + `aw view --check` + `aw view --screenshot <png>` + `aw view --app <app>` - native desktop repo reader with fixed project list, in-app terminal/detail layout toggle, stable JSON/surface snapshot, quick headless contract check, app-level visual debug capture, and macOS app bundle generation for agents and EC gates.
EC Dimensions:
- behavior: `./target/debug/aw view --snapshot` - repo catalog, terminal status, focused README brief, capability map detail, EC inventory, TD summary, and renderer-neutral surface snapshot are present.
- behavior: `./target/debug/aw view --check` - headless contract check contains the terminal pane, repo catalog, semantic layout toggle, selected README brief, capability map, EC, and TD detail panes.
- behavior: `./target/debug/aw view --screenshot /private/tmp/aw-view-app.png` - app-level PNG capture is rendered from the same surface tree without a browser or desktop screen capture.
- behavior: `./target/debug/aw view --layout top-bottom --screenshot /private/tmp/aw-view-app-top-bottom.png` - repo list stays fixed while the terminal/detail region can switch to top-bottom layout with a visible toggle control.
- behavior: `./target/debug/aw view --app /private/tmp/AWRepoView.app` - native macOS app bundle is produced and launches the repo-built desktop view.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
AW presents the repository as a visual-reader model: repo navigation is the root, selected repo items expose README brief text, configured capability-map detail, EC contract status, and TD relation summary, and the desktop artifact has stable semantic surface IDs so tests do not depend on a toolkit-private tree. The repo list stays fixed on the left; the right-side workspace can place terminal/status and current EC/capability detail either left-right or top-bottom, defaulting to left-right. A native toggle button switches that workspace layout in the running desktop app.
Gate Inventory:
- `cargo test -p agentic-workflow --lib view_repo_snapshot -- --nocapture`
- `./target/debug/aw view --snapshot`
- `./target/debug/aw view --check`
- `./target/debug/aw view --screenshot /private/tmp/aw-view-app.png`
- `./target/debug/aw view --layout top-bottom --screenshot /private/tmp/aw-view-app-top-bottom.png`
- `./target/debug/aw view --app /private/tmp/AWRepoView.app`
- projects/agentic-workflow/tech-design/surface/specs/aw-repo-view-desktop-app.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Repo desktop reader | change | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib view_repo_snapshot -- --nocapture`; `./target/debug/aw view --snapshot`; `./target/debug/aw view --check`; `./target/debug/aw view --screenshot /private/tmp/aw-view-app.png`; `./target/debug/aw view --layout top-bottom --screenshot /private/tmp/aw-view-app-top-bottom.png`; `./target/debug/aw view --app /private/tmp/AWRepoView.app` |

### Existing Project Standardization

ID: existing-project-standardization
Type: DeveloperTool
Surfaces:
- CLI: `aw standardize` + `aw health` - brownfield takeover guidance and readiness rollup.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --test cli_tests standardize_subcommands_registered` - takeover command surface and readiness reporting.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Existing projects can be adopted one bounded tick at a time: capability readiness stays in `aw capability`, takeover runs through managed/semantic/traceability, and generator gaps route back into normal WI/TD/CB work.
Gate Inventory:
- projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Brownfield takeover surface | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --test cli_tests standardize_subcommands_registered` |
| Managed and semantic production gates | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib semantic_coverage_prioritizes_missing_td_before_generator_gap` |
| Traceability closure gate | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib traceability` covers command, TD, source, and CB closure |
| CB and cold verification gates | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_gen_cold_rebuild_targets_include_codegen_changes` |
| Regenerability maturity loop (optional) | epic | - | out_of_scope | none | none | - |
