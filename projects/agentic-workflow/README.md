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
configured repo scopes still have uncommitted lifecycle changes. While those
changes are dirty, the envelope must report repo commit and WI evidence as
incomplete so agents do not mistake a local persistence request for published
outward evidence.

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

Agentic Workflow is the generator-authoritative implementation of this protocol,
but its own production gate is intentionally narrower than a full AW takeover of
another project. For `agentic-workflow` / `aw` self-health, capability contracts
and EC claim closure are authoritative. Managed ownership, semantic coverage,
traceability, TD lock, CB verify, cold rebuild, and workspace test gates remain
observable readiness metrics, but they do not block self-health production
readiness unless they are expressed as capability or EC contract failures.

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
belong in capability-map text or ordinary docs, not in TD/EC typed sections.

## Capabilities

The full machine-readable capability contract lives in [CAPABILITIES.md](CAPABILITIES.md). This README keeps only the large capability map for quick project orientation.

### Large Capabilities

| Capability | Product promise | Production | Full contract |
|---|---|---|---|
| AW Core Client Model | Shared AW Core nouns, WorkItem-first artifact admission, and client boundary semantics for `aw CLI`, Cue, and future clients. | ready | [CAPABILITIES.md](CAPABILITIES.md#aw-core-client-model) |
| Workflow Root Runner | Root-scoped project, capability, and WI workflow envelopes with child-command rollup. | ready | [CAPABILITIES.md](CAPABILITIES.md#workflow-root-runner) |
| Capability Control Plane | Markdown capability contracts, readiness reporting, project sweep, and contract field setters. | ready | [CAPABILITIES.md](CAPABILITIES.md#capability-control-plane) |
| Work Item Planning | Capability roots can become epic/subepic candidates, then bounded change WIs. | ready | [CAPABILITIES.md](CAPABILITIES.md#work-item-planning) |
| TD/CB Lifecycle Automation | Atomic WIs move through TD, code generation/fill, code-check, and merge gates. | ready | [CAPABILITIES.md](CAPABILITIES.md#tdcb-lifecycle-automation) |
| Project-Local TD and EC Gates | Project-local TD roots, external contracts, generated gates, and dirty-scope protections. | ready | [CAPABILITIES.md](CAPABILITIES.md#project-local-td-and-ec-gates) |
| Manual Evidence Artifacts | EC-derived product manuals are tracked as generated evidence artifacts. | ready | [CAPABILITIES.md](CAPABILITIES.md#manual-evidence-artifacts) |
| Repo View Desktop App | Native repo reader for repo/project selection, Caps/EC detail, terminal context, screenshots, and app bundle generation. | ready | [CAPABILITIES.md](CAPABILITIES.md#repo-view-desktop-app) |
| Existing Project Standardization | Brownfield takeover guidance, readiness rollup, and generator-gap routing. | ready | [CAPABILITIES.md](CAPABILITIES.md#existing-project-standardization) |
