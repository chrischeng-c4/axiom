# Agentic Workflow

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

AW uses short agent-facing command names for the main lifecycle:

| CLI | Long name | Role |
|---|---|---|
| `aw caps` | `aw capability` | Define the project capability tree, claims, maturity, release scope, and required external contracts. |
| `aw ec` | External Contracts | Define behavior, efficiency, security, and stability contracts; generate tests and tool configs. |
| `aw td` | Tech Design | Describe implementation design, source mapping, APIs, data, and control flow. |
| `aw cb` | Code artifact lifecycle | Generate, check, and fill source artifacts from TD. |
| `aw hc` | `aw health` | Aggregate caps, EC, TD, CB, tests, claim closure, locks, and blocker status. |

The canonical flow for greenfield projects is:

```text
aw caps -> aw ec draft/fill -> aw ec gen -> aw td -> aw cb -> aw hc
```

Greenfield starts by defining capabilities and required external contracts. EC
contracts may begin red: `aw ec gen` materializes the tests, runner stubs, and
tool manifests first, then TD/CB/code work drives those contracts green.

The canonical flow for brownfield projects is:

```text
aw caps check -> aw ec check/gen -> aw td claim -> aw cb claim/gen/fill -> aw hc
```

Brownfield starts by adding capabilities around existing behavior, then
externalizes the missing behavior/efficiency/security/stability contracts before
TD and CB claim the source. Missing EC is a normal adoption gap until production
readiness is requested.

Capability claim closure is the deterministic production link between caps, EC,
TD, and generated artifacts. Agents make the semantic judgment, but they must
write it down as explicit metadata: capability claims name `claim_id`, EC cases
name the same `capability_id` and `claim_id`, and TD frontmatter names the same
claim in a primary `capability_refs` entry. `aw hc claims --project <name>`
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
and integrated tool configuration for `rig`, `meter`, `arena`, `guard`, and
`vat`. It does not write generated state into `aw.toml`; `aw.toml` remains the
project root config. Legacy TD `e2e-test` and `tool-contract` sections remain a
compatibility import source only when `external-contracts/` has no contracts.

TD section types and EC contracts are artifact-producing inputs. A typed TD
section must drive source, tests, config, manifests, deployment artifacts, or a
verification/tool artifact; EC contracts must generate tests or native tool
configs. Pure product explanation, semantic notes, and non-generating evidence
belong in README capability text or ordinary docs, not in TD/EC typed sections.

## Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| AW Core Client Model | #3893 | implemented | verified | smoke | ready | verified; shared AW Core nouns, WorkItem-first artifact admission, and client boundaries |
| Workflow Root Runner | - | implemented | verified | smoke | ready | verified; CLI workflow chain and root-to-child rollup contract |
| Capability Control Plane | - | implemented | verified | smoke | ready | verified; README capability map, `aw caps`, and verification summaries |
| Work Item Planning | - | implemented | verified | smoke | ready | verified; epic/change split and bounded planning artifacts |
| TD/CB Lifecycle Automation | - | implemented | verified | smoke | ready | verified; WI to TD to CB to merge workflow |
| Project-Local TD and EC Gates | - | implemented | verified | smoke | ready | verified; TD roots default to `<project.path>/tech-design`, EC contracts default to `<project.path>/external-contracts`, and generated tests/tool configs stay project-local |
| Manual Evidence Artifacts | #57 | planned | planned | smoke | planned | confirmed; generated product manuals are tracked as EC evidence artifacts with runner commands and optional media |
| Existing Project Standardization | - | implemented | verified | smoke | ready | verified; takeover readiness, managed/semantic/traceability gates, and generator gap requests |

## AW Core Client Model

| Field | Value |
|---|---|
| ID | aw-core-client-model-workitem-first-artifact-lifecycle |
| Root WI | #3893 |
| Status | verified |
| Promise | AW Core defines the client-independent workflow and domain protocol shared by `aw CLI`, Cue, and future clients, with WorkItem-first artifact admission and evidence-backed rollup. |
| Required Verification | smoke |
| Gate Inventory | projects/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md; projects/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md; projects/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Core concept model and invariants | change | #3894 | implemented | verified | smoke | projects/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md |
| WorkItem artifact admission gate | change | #3895 | implemented | verified | smoke | projects/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md |
| Client boundary model | change | #3896 | implemented | verified | smoke | projects/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md |

## Workflow Root Runner

| Field | Value |
|---|---|
| ID | workflow-root-runner |
| Root WI | - |
| Status | verified |
| Promise | `aw run` emits a CLI workflow chain from README, capability, epic, or change roots and keeps rolling work upward until the project root is complete or blocked. |
| Required Verification | smoke |
| Gate Inventory | projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CLI workflow chain | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow root_parser_accepts_capability_and_wi_roots` |
| Root envelope completion contract | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow create_wi_blocks_on_pending_epicize_artifact` |
| Parent rollup routing | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow closed_change_outputs_parent_inspection` |

## Capability Control Plane

| Field | Value |
|---|---|
| ID | capability-control-plane |
| Root WI | - |
| Status | verified |
| Promise | Project READMEs can describe capabilities as readable Markdown headings and tables while detailed proof lives in validation inventories and external contracts. |
| Required Verification | smoke |
| Gate Inventory | projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Markdown capability schema | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow markdown_capability_tables` |
| Capability readiness reporting | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow fixture_reference_can_verify_required_claim` |

## Work Item Planning

| Field | Value |
|---|---|
| ID | work-item-planning |
| Root WI | - |
| Status | verified |
| Promise | Capability information can be projected into epic roots, and epic roots can be atomized into bounded change WIs for agent-sized execution. |
| Required Verification | smoke |
| Gate Inventory | projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Capability to epic planning | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow epicize_artifact_includes_markdown_capability_roots` |
| Epic to change atomization | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow prioritize_lanes_put_bounded_bug_in_ready_now` |

## TD/CB Lifecycle Automation

| Field | Value |
|---|---|
| ID | td-cb-lifecycle-automation |
| Root WI | - |
| Status | verified |
| Promise | Atomic change WIs can move through TD authoring, review, code generation, handwrite fill, review, and merge with CLI-emitted next steps. |
| Required Verification | smoke |
| Gate Inventory | projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| TD lifecycle dispatch | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow td_branch_activation_only_uses_main` |
| CB lifecycle dispatch | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow cb_gen_force_regen_verify_parses_without_slug` |

## Project-Local TD and EC Gates

| Field | Value |
|---|---|
| ID | project-local-td-and-ec-gates |
| Root WI | - |
| Status | verified |
| Promise | AW-managed projects keep their README, external contracts, tech designs, source, tests, and generated tool configs under the project tree by default: `td_path` is only an override, EC contracts live under `<project.path>/external-contracts`, and generated EC state does not mutate `aw.toml`. |
| Required Verification | smoke |
| Gate Inventory | `cargo test -p agentic-workflow --lib`; `cargo test -p agentic-workflow ec_doc`; `aw td check projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md`; `aw td check projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md`; `aw td check projects/agentic-workflow/tech-design/surface/interfaces/src/standardize.md` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Project-local TD root resolver | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib` |
| TD lock and external-contract target resolution | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib` |
| CB generation and standardize scan defaults | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib` |
| Project dirty-scope protection | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow --lib` |
| EC evidence documentation | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow ec_doc` |
| EC external-contract source | change | #13 | implemented | verified | smoke | `aw ec draft/fill` authors `external-contracts/{behavior,efficiency,security,stability}` markdown; `aw ec gen` generates tests plus rig/meter/arena/guard/vat tool configs without writing `aw.toml` |

## Manual Evidence Artifacts

| Field | Value |
|---|---|
| ID | manual-evidence-artifacts |
| Root WI | #57 |
| Status | confirmed |
| Promise | AW treats generated product manuals as first-class EC evidence artifacts. A manual artifact records its project-local output path, the runner command that produces it, and optional screenshots, highlights, or step metadata without requiring every manual to use a visual overlay recorder. |
| Required Verification | smoke |
| Gate Inventory | projects/agentic-workflow/src/tools/common_change_spec.rs; projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md; /Users/chris.cheng/projects/ai-studio/docs/user-manual |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Generated manual EC evidence schema | change | #57 | planned | planned | smoke | projects/agentic-workflow/src/tools/common_change_spec.rs; projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md |
| Manual runner output convention | change | #57 | planned | planned | smoke | /Users/chris.cheng/projects/ai-studio/vat.toml; /Users/chris.cheng/projects/ai-studio/docs/user-manual |

## Existing Project Standardization

| Field | Value |
|---|---|
| ID | existing-project-standardization |
| Root WI | - |
| Status | verified |
| Promise | Existing projects can be adopted one bounded tick at a time: capability readiness stays in `aw capability`, takeover runs through managed/semantic/traceability, and generator gaps route back into normal WI/TD/CB work. |
| Required Verification | smoke |
| Gate Inventory | projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Brownfield takeover surface | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow standardize_subcommands_registered` |
| Managed and semantic production gates | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow semantic_coverage_prioritizes_missing_td_before_generator_gap` |
| Traceability closure gate | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow traceability` covers command, TD, source, and CB closure |
| CB and cold verification gates | epic | - | implemented | verified | smoke | `cargo test -p agentic-workflow cb_gen_cold_rebuild_targets_include_codegen_changes` |
| Regenerability maturity loop (optional) | epic | - | out_of_scope | none | none | - |
