# Agentic Workflow

## Brief

Workflow protocol and CLI chain for capability-driven project takeover,
work-item planning, TD/CB lifecycle execution, and production-readiness rollup.

## Overview

Agentic Workflow (`aw`) coordinates bounded project work through a CLI workflow
chain from product capabilities down through work items, tech designs, code
artifacts, verification gates, and parent-root rollup.

AW Core is the client-independent model underneath `aw CLI`, Cue, and future
clients. Its canonical nouns are Project, Capability, WorkItem, Artifact, Gate,
Evidence, HITL, Rollup, and Client; CLI commands and Cue product workflows are
client adapters over those shared semantics. See
`projects/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md`.
Client boundary: `aw CLI` is the standalone repo-local developer and
coding-agent client; Cue is the enterprise team collaboration web
frontend/backend client. Both sit over AW Core, and Cue is not an `aw CLI`
wrapper. See
`projects/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md`.
Core artifact admission is WorkItem-first: no artifact is created before an
accepted WorkItem, and the requested artifact type must be allowed by that
WorkItem's target artifact route. See
`projects/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md`.

Milestone persistence is enforced by the CLI workflow state, not by agent memory
or a separate agent-called commit command. Mutating TD, CB, and standardization
verbs create scoped lifecycle commits when they change repo-owned artifacts, and
`aw run` blocks project completion with structured persistence details when
configured repo scopes still have uncommitted lifecycle changes.

`aw run` is the canonical root runner for coding agents. Omit `--root` to run
the current project root, or pass `--root capability:<id>` / `--root wi:<id>` to
scope the loop. Every JSON envelope uses `schema_version: aw.cli.v1`, exposes
`completion.workflow_complete` and `completion.requires_hitl`, and carries the
only command the agent should run in `next.command`. Long project-root
evaluation emits bounded JSONL `event=progress` records before the final
envelope, and project aliases are normalized to the configured canonical project
name. If `completion.requires_hitl=true`, the envelope must include
`hitl_question` so the agent can ask the user before doing unattended repair
work. Agents must re-run the same root command after each child command
completes and stop only when the envelope reports workflow completion, HITL,
blocker, or error.

## Lifecycle Surface

AW uses canonical agent-facing command names for the main lifecycle:

| CLI | Long name | Role |
|---|---|---|
| `aw capability` | Product capabilities | Define the project capability tree, claims, maturity, release scope, and required external contracts. |
| `aw ec` | External Contracts | Define behavior, efficiency, security, and stability contracts; generate tests and tool configs. |
| `aw td` | Tech Design + code artifacts | Describe implementation design and own generated-code verbs: `gen`, `gen-source`, `fill`, `code-check`, and `code-claim`. TD output is a candidate implementation that iterates until EC and health gates pass. |
| `aw health` | Project health | Aggregate capabilities, EC, TD, CB, tests, claim closure, locks, and blocker status. |

The canonical flow for greenfield projects is:

```text
aw capability report/next/migrate/check -> aw ec draft/fill -> aw ec gen -> aw td create/gen/fill/code-check/merge -> aw health
```

Greenfield starts by defining capabilities and required external contracts. EC
contracts may begin red: `aw ec gen` materializes the tests, runner stubs, and
tool manifests first, then TD/CB/code work drives those contracts green.

The canonical flow for brownfield projects is:

```text
aw capability check -> aw ec check/gen -> aw td claim/code-claim/gen/fill -> aw health
```

Brownfield starts by adding capabilities around existing behavior, then
externalizes the missing behavior/efficiency/security/stability contracts before
TD and CB claim the source. Missing EC is a normal adoption gap until production
readiness is requested.

Capability claim closure is the deterministic production link between capabilities,
EC, TD, and generated artifacts. Agents make the semantic judgment, but they must
write it down as explicit metadata: capability claims name `claim_id`, EC cases
name the same `capability_id` and `claim_id`, and TD frontmatter names the same
claim in a primary `capability_refs` entry. `aw health --project <name> claims`
checks those typed edges, EC command results, and existing artifact health; it
does not infer semantic coverage from prose. Production-required EC cases may
not remain `unmapped`.

EC source of truth lives under `projects/<name>/external-contracts/` as
markdown, using the same section authoring pattern as TD:

```text
external-contracts/
  behavior/
  efficiency/
  security/
  stability/
```

Use `aw ec draft --project <name> <id>` to create an EC markdown skeleton and
`aw ec fill --project <name> <path> --section <type> --body-file <file>` to fill
typed sections such as `e2e-test` and `tool-contract`. `aw ec gen --project
<name>` reads `external-contracts/**/*.md` and generates project-local tests
and integrated tool configuration for `rig`, `meter`, `guard`, and `vat`, while
retaining `arena` only as a legacy compatibility import. It does not write
generated state into `aw.toml`; `aw.toml` remains the project root config.
Legacy TD `e2e-test` and `tool-contract` sections remain a compatibility import
source only when `external-contracts/` has no contracts.

TD section types and EC contracts are artifact-producing inputs. A typed TD
section must drive source, tests, config, manifests, deployment artifacts, or a
verification/tool artifact; EC contracts must generate tests or native tool
configs. Pure product explanation, semantic notes, and non-generating evidence
belong in README capability text or ordinary docs, not in TD/EC typed sections.


## Capabilities

Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| AW Core Client Model | #3894 | implemented | verified | smoke | ready | verified; shared AW Core nouns, WorkItem-first artifact admission, and client boundaries |
| Workflow Root Runner | - | implemented | verified | smoke | ready | verified; CLI workflow chain and root-to-child rollup contract |
| Capability Control Plane | - | implemented | verified | smoke | ready | verified; README capability map, `aw capability`, and verification summaries |
| Work Item Planning | - | implemented | verified | smoke | ready | verified; epic/change split and bounded planning artifacts |
| TD/CB Lifecycle Automation | - | implemented | verified | smoke | ready | verified; WI to TD to CB to merge workflow |
| Project-Local TD and EC Gates | #13 | implemented | verified | smoke | ready | verified; TD roots default to `<project.path>/tech-design`, EC contracts default to `<project.path>/external-contracts`, and generated tests/tool configs stay project-local |
| Manual Evidence Artifacts | #57 | implemented | verified | smoke | ready | verified; generated product manuals are tracked as EC evidence artifacts with runner commands and optional media |
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
| Agent orientation surface | change | #178 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib llm_outline_lists_registered_verbs`; projects/agentic-workflow/tech-design/logic/aw-llm-offline-agent-orientation-command.md |
| WorkItem loop-state model | change | #189 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib loop_state_round_trips`; projects/agentic-workflow/tech-design/logic/workitem-loop-state-model-additive-foundation.md |

### Workflow Root Runner

ID: workflow-root-runner
Type: DeveloperTool
Surfaces:
- CLI: `aw run` - root-scoped project, capability, and WI workflow runner for coding agents.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow root_parser_accepts_capability_and_wi_roots` - root parsing and JSON envelope contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
`aw run` emits a CLI workflow chain from README, capability, epic, or change roots and keeps rolling work upward until the project root is complete or blocked.
Gate Inventory:
- projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CLI workflow chain | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow root_parser_accepts_capability_and_wi_roots` |
| Root envelope completion contract | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow create_wi_blocks_on_pending_epicize_artifact` |
| Parent rollup routing | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow closed_change_outputs_parent_inspection` |

### Capability Control Plane

ID: capability-control-plane
Type: DeveloperTool
Surfaces:
- CLI: `aw capability` - report, next, draft, migrate, check, init, sweep, and contract field setters.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow markdown_capability_tables` - Markdown README contract parsing, migration, and readiness reporting.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Project READMEs can describe capabilities as readable Markdown headings and tables while detailed proof lives in validation inventories and external contracts.
Gate Inventory:
- projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Markdown capability schema | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow markdown_capability_tables` |
| Capability readiness reporting | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow fixture_reference_can_verify_required_claim` |
| Capability project sweep | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib capability_sweep`; human sweep queue output reviewed through aw capability sweep |
| Missing README initialization | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib capability_init`; README shell init behavior only, no runtime project mutation gate |

### Work Item Planning

ID: work-item-planning
Type: DeveloperTool
Surfaces:
- CLI: `aw wi` - inventory, validation drafting, epicization, atomization, prioritization, and issue updates.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow epicize_artifact_includes_markdown_capability_roots` - capability-to-WI planning projection.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Capability information can be projected into epic roots, and epic roots can be atomized into bounded change WIs for agent-sized execution.
Gate Inventory:
- projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Capability to epic planning | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow epicize_artifact_includes_markdown_capability_roots` |
| Epic to change atomization | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow prioritize_lanes_put_bounded_bug_in_ready_now` |

### TD/CB Lifecycle Automation

ID: td-cb-lifecycle-automation
Type: DeveloperTool
Surfaces:
- CLI: `aw td` - tech-design lifecycle plus inherited code-artifact lifecycle commands.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow td_branch_activation_only_uses_main` - TD/CB lifecycle command dispatch and phase rules.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Atomic change WIs can move through TD authoring, review, code generation, handwrite fill, review, and merge with CLI-emitted next steps.
Gate Inventory:
- projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| TD lifecycle dispatch | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow td_branch_activation_only_uses_main` |
| CB lifecycle dispatch | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow cb_gen_force_regen_verify_parses_without_slug` |

### Project-Local TD and EC Gates

ID: project-local-td-and-ec-gates
Type: DeveloperTool
Surfaces:
- CLI: `aw ec` + `aw td check` - project-local external-contract, generated gate, and TD validation commands.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow --lib ec_draft_fill_markdown_drives_inventory` - EC markdown source, aw.toml inventory, and generated tool manifest contract.
- stability: `cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design` - project-local TD root resolution and dirty-scope protection.
Root WI: #13
Status: verified
Required Verification: smoke
Promise:
AW-managed projects keep their README, external contracts, tech designs, source, tests, and generated tool configs under the project tree by default: `td_path` is only an override, EC contracts live under `<project.path>/external-contracts`, and the generated EC inventory lives in the project `aw.toml` AW-EC block.
Gate Inventory:
- `cargo test -p agentic-workflow --lib falls_back_to_project_tech_design`; `cargo test -p agentic-workflow --lib ec_context_defaults_td_root_to_project_tech_design`; `cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design`; `cargo test -p agentic-workflow --lib semantic_coverage_excludes_aw_ec_generated_wrappers`; `cargo test -p agentic-workflow --lib ec_doc`; `aw td check projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md`; `aw td check projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/standardize.md`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Project-local TD root resolver | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib falls_back_to_project_tech_design`; `aw td check projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md` |
| TD lock and external-contract target resolution | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_context_defaults_td_root_to_project_tech_design`; `aw td check projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md` |
| CB generation and standardize scan defaults | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md` |
| Project dirty-scope protection | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib semantic_coverage_excludes_aw_ec_generated_wrappers`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/standardize.md` |
| EC evidence documentation | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_doc` |
| EC external-contract source | change | #13 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_draft_fill_markdown_drives_inventory`; aw ec draft/fill authors project-local external-contract markdown and aw ec gen writes the project `aw.toml` EC inventory plus generated tests and rig/meter/guard/vat tool configs; arena is retained as a legacy compatibility import |
| EC tool binding dispatch | change | #13 | implemented | verified | smoke | `cargo test -p agentic-workflow --lib ec_binding_command`; `cargo test -p agentic-workflow --lib resolve_ec_command_dispatches_bound_category`; projects/agentic-workflow/tech-design/config/ec-tool-binding-config-ec-category-verify-ec-dispatch-with-manif.md; projects/agentic-workflow/tech-design/logic/aw-ec-add-vat-binding-command-support.md |

### Manual Evidence Artifacts

ID: manual-evidence-artifacts
Type: DeveloperTool
Surfaces:
- CLI: `aw ec doc` - generated, checked, or previewed EC-derived product documentation evidence.
EC Dimensions: behavior: `cargo test -p agentic-workflow ec_doc_gen_writes_manual_from_inventory` - generated manual artifact schema and output convention
Root WI: #57
Status: verified
Required Verification: smoke
Promise:
AW treats generated product manuals as first-class EC evidence artifacts. A manual artifact records its project-local output path, the runner command that produces it, and optional screenshots, highlights, or step metadata without requiring every manual to use a visual overlay recorder.
Gate Inventory:
- projects/agentic-workflow/src/tools/common_change_spec.rs; projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md; /Users/chris.cheng/projects/ai-studio/docs/user-manual

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Generated manual EC evidence schema | change | #57 | implemented | verified | smoke | `cargo test -p agentic-workflow ec_generated_manual_artifact` |
| Manual runner output convention | change | #57 | implemented | verified | smoke | `cargo test -p agentic-workflow ec_doc_gen_writes_manual_from_inventory`; projects/agentic-workflow/src/tools/common_change_spec.rs |

### Existing Project Standardization

ID: existing-project-standardization
Type: DeveloperTool
Surfaces:
- CLI: `aw standardize` + `aw health` - brownfield takeover guidance and readiness rollup.
EC Dimensions:
- behavior: `cargo test -p agentic-workflow standardize_subcommands_registered` - takeover command surface and readiness reporting.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Existing projects can be adopted one bounded tick at a time: capability readiness stays in `aw capability`, takeover runs through managed/semantic/traceability, and generator gaps route back into normal WI/TD/CB work.
Gate Inventory:
- projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Brownfield takeover surface | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow standardize_subcommands_registered` |
| Managed and semantic production gates | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow semantic_coverage_prioritizes_missing_td_before_generator_gap` |
| Traceability closure gate | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow traceability` covers command, TD, source, and CB closure |
| CB and cold verification gates | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow cb_gen_cold_rebuild_targets_include_codegen_changes` |
| Regenerability maturity loop (optional) | epic | - | out_of_scope | none | none | - |
