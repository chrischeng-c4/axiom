# Agentic Workflow

## Brief

Agentic Workflow (`aw`) is an agent-first project-iteration CLI for coding
agents. It owns next-action guidance, durable artifact skeletons, strict format
and phase validation, and code generation.

## Contributing

Project-local authoring rules for Agentic Workflow: authoritative inputs,
self-hosting boundaries, and meta-doc placement. Repo-wide authoring rules
remain in [../../CONTRIBUTING.md](../../CONTRIBUTING.md). Full rules:
[CONTRIBUTING.md](CONTRIBUTING.md).

## Capability Contract

Machine-readable capability contract for Agentic Workflow. Full contract:
[CAPABILITIES.md](CAPABILITIES.md).

## Overview

Agentic Workflow (`aw`) coordinates bounded project work through one CLI chain
from project META-doc goals through work items, external contracts, tech
design/codegen, verification evidence, and parent-root rollup. Its public nouns
are Project, Capability, WorkItem, Artifact, Gate, Evidence, and Rollup. The CLI
is the product boundary; there is no separate product-client architecture.

The canonical product model lives in
`apps/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md`, and
the single CLI boundary lives in
`apps/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md`.
Artifact admission is WorkItem-first: no durable artifact is created before an
accepted WorkItem, and the requested artifact type must be allowed by that
WorkItem's target artifact route. See
`apps/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md`.

Milestone persistence is enforced by the CLI workflow state, not by agent memory
or a separate agent-called commit command. Mutating TD, CB, and standardization
verbs create scoped lifecycle commits when they change repo-owned artifacts, and
`aw wi run`/`aw capability run` block project completion with structured
persistence details when configured repo scopes still have uncommitted
lifecycle changes. While those changes are dirty, the envelope must report
repo commit and WI evidence as incomplete so agents do not mistake a local
persistence request for published outward evidence.

`aw wi run <id>` and `aw capability run [<cap-id>] --project <p>` are the
canonical root runners for coding agents: `aw wi run <id>` drives one work
item, `aw capability run <cap-id>` scopes to one capability's work-root WIs,
and `aw capability run --project <p>` is the project-wide run-to-end driver.
Every JSON envelope uses `schema_version: aw.cli.v1`, exposes
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
| `aw meta` | META-doc control plane | Initialize repo/project skeletons, refresh only AW-owned marker blocks, and fail read-only checks with exact remediation. |
| `aw capability` | Product capabilities | Define the project capability tree, claims, maturity, release scope, and required external contracts. |
| `aw ec` | External Contracts | Define behavior, efficiency, security, and stability contracts; generate tests and tool configs. |
| `aw td` | Tech Design + code artifacts | Describe implementation design and own generated-code verbs: `gen`, `gen-source`, `fill`, `code-check`, and `create --from-source`. TD output is a candidate implementation that iterates until EC and health gates pass. |
| `aw health` | Project health | Aggregate capabilities, EC, TD, CB, tests, claim closure, locks, and blocker status. |

The canonical flow for greenfield projects is:

```text
aw meta init/check -> aw capability report/next/migrate/check -> aw ec draft/fill -> aw ec gen -> aw td create/gen/fill/code-check -> aw health
```

Greenfield starts by creating the repo/project META-doc control plane, then
defining capabilities and required external contracts. EC
contracts may begin red: `aw ec gen` materializes the tests, runner stubs, and
tool manifests first, then TD/CB/code work drives those contracts green.

The canonical flow for brownfield projects is:

```text
aw meta check -> aw capability check -> aw ec check/gen -> aw td claim/create --from-source/gen/fill/code-check -> aw health
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

The quick map below points to the canonical entries in
[CAPABILITIES.md](CAPABILITIES.md).

### Large Capabilities

| Capability | Product promise | Production | Full contract |
|---|---|---|---|
| AW Agent-First CLI Model | One coding-agent CLI that owns next-action guidance, artifact skeletons, strict validation/phases, codegen, WorkItem-first admission, and evidence-backed rollup. | ready | [CAPABILITIES.md](CAPABILITIES.md#aw-agent-first-cli-model) |
| Workflow Root Runner | Root-scoped project, capability, and WI workflow envelopes with child-command rollup. | ready | [CAPABILITIES.md](CAPABILITIES.md#workflow-root-runner) |
| Capability Control Plane | Markdown capability contracts, readiness reporting, project sweep, and contract field setters. | ready | [CAPABILITIES.md](CAPABILITIES.md#capability-control-plane) |
| Work Item Planning | Capability roots can become epic/subepic candidates, then bounded change WIs. | ready | [CAPABILITIES.md](CAPABILITIES.md#work-item-planning) |
| TD/CB Lifecycle Automation | Atomic WIs move through TD, code generation/fill, code-check, and merge gates. | ready | [CAPABILITIES.md](CAPABILITIES.md#tdcb-lifecycle-automation) |
| Project-Local TD and EC Gates | Project-local TD roots, external contracts, generated gates, and dirty-scope protections. | ready | [CAPABILITIES.md](CAPABILITIES.md#project-local-td-and-ec-gates) |
| Manual Evidence Artifacts | EC-derived product manuals are tracked as generated evidence artifacts. | ready | [CAPABILITIES.md](CAPABILITIES.md#manual-evidence-artifacts) |
| Existing Project Standardization | Brownfield takeover guidance, readiness rollup, and generator-gap routing. | ready | [CAPABILITIES.md](CAPABILITIES.md#existing-project-standardization) |
