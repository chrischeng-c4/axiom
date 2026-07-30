---
project:
  name: axiom
  owner: chrischeng-c4
  url: https://github.com/chrischeng-c4/axiom
  ssh: git@github.com:chrischeng-c4/axiom.git
  default_branch: main
---

# AGENTS.md - Repository Bootstrap

Read `README.md` for repository inventory and `CONTRIBUTING.md` for the
authoritative repo-wide authoring, service, CLI, META-doc, and verification
contracts. Project promises live in each project's `CAPABILITIES.md`; local
workflow and verification live in its `CONTRIBUTING.md`.

## Authority Order

- `CONTRIBUTING.md` owns repo-wide authoring and operational contracts.
- `<project>/CAPABILITIES.md` owns product promises, work roots, and gates.
- `<project>/CONTRIBUTING.md` owns project-local edit and verification rules.
- `.agents/rules/**/*.md` owns reusable agent instructions, one concern per semantic path.
- Skills are thin human-invoked entry points; hooks and `aw guard` own hard runtime enforcement.
- `aw` stdout and `aw llm` own the mid-loop agent protocol.

## Runtime Boundaries

- Codex uses this hierarchical `AGENTS.md` bootstrap and generated rule index.
- Claude imports `@AGENTS.md` and receives deterministic `.claude/rules` projections.
- AGY consumes the canonical `.agents/rules` workspace tree.
- `.codex/rules/*.rules` is command-approval policy, never instruction content.

## External Contract Boundaries

- Externally observable product behavior belongs in a project-local Python
  external-contract project; Agentic Workflow's own lives under
  `apps/agentic-workflow/external-contracts/`, where `pyproject.toml` is the
  inventory and `src/cases/*.py` holds one black-box verifier per case.
- Rules observable only inside the Rust implementation are
  colocated Rust invariants under their semantic `src/**` owner, and run
  separately with `cargo test -p agentic-workflow --lib`.
- Never wrap a Python external contract in an app-level Rust tree and never
  delegate one to `cargo test`.

<!-- aw:start -->
## Agentic Workflow CLI Surface

Agentic Workflow is the workflow protocol. Treat stdout, payload paths,
`invoke.command`, validation errors, and `next.command` as the live contract.
Run `aw <verb> --help` when argument shape matters and `aw llm` for offline
orientation.

For goal roots, completion means `completion.workflow_complete=true`.
`action=done` can finish only the current child. A HITL envelope requires real
human input; never fabricate approval.

Agentic Workflow dogfoods the same Python-first `aw goal` roots as every other
project. Use a bounded direct implementation commit with `Refs #<issue>` only
when the exact worker verb required by the current root is itself broken.

Codex should translate Claude slash-command references such as `/aw:td` or
`/aw:wi` to the equivalent `aw ...` CLI command unless the user
explicitly asks for Claude-specific behavior.

### Workflow CLI

<!-- aw:cli-table:workflow:start -->
| Verb | About |
|------|-------|
| `aw meta` | Initialize, synchronize, and check repository/project META-docs |
| `aw wi` | Manage work-items — list/show/create/validate across local + GitHub backends |
| `aw capability` | Product capability completion loop: report/next/run/check |
| `aw td` | Tech-design authoring and validation lifecycle |
| `aw ec` | Python EC lifecycle: scaffold/check source, independently review, then verify |
| `aw health` | Aggregate project readiness, production gates, and blocker status |
| `aw conf` | Manage `aw.toml` and Agentic Workflow configuration producers |
| `aw coordination` | Persist and reconcile AW-owned task, dispatch, gate, event, and decision state |
<!-- aw:cli-table:workflow:end -->

The lifecycle is linear: `aw wi` → `aw ec` → `aw td` → `aw cb`. Drive one
root with `aw goal wi <id>`, `aw goal capability --project <project>`, or
`aw goal backlog --project <project>`, then follow the emitted command.
`aw health` is read-only and names its worker remediation in `next.command`.

### Work-item terminal states

The closed work-item enum is terminology-first:

| Type | Terminal state |
|---|---|
| `epic` | all owned children are terminal |
| `change` | EC is green for the generated codebase and the lifecycle closes the change |
| `spike` | an ADR-style decision records spawned WI refs or explicit no-action; expiry converges to `gave_up` |
| `report` | typed `triage` accepts and links a spawned change/epic, or closes as `duplicate`, `invalid`, or `by-design` |

Only `change` enters executable backlog work. A `spike` never lands
investigation code in product source. A `report` remains in the intake queue
until triage, and both converge by spawn-and-link instead of changing type in
place.

Canonical reusable instructions live one concern per `.agents/rules` file.
Codex follows the generated AGENTS index, Claude receives `.claude/rules`
projections, and AGY consumes the canonical tree. `.codex/rules` is command
approval policy, not instruction content.

### Support CLI

<!-- aw:cli-table:support:start -->
| Verb | About |
|------|-------|
| `aw guard` | Agent-runtime direct edit/create guard for Codex, Claude Code, and AGY |
| `aw llm` | Offline agent orientation: outline + capability/td/ec pillars + loop |
| `aw upgrade` | Self-update this binary from a published GitHub release |
| `aw issue` | Search, view, or create Agentic Workflow issues |
| `aw goal` | Unified loop verb: lifecycle root types (`wi`, `capability`, `backlog`) plus the ad-hoc CLI-owned verifiable-condition loop for bounded work outside the WI lifecycle (`set`/`check`/`show`/`list`/`clear`) |
<!-- aw:cli-table:support:end -->
<!-- aw:end -->

<!-- aw:meta-rule-index:start -->
## Canonical Agent Rules

Reusable instructions live in `.agents/rules`; this generated index routes Codex by semantic path. Claude projections live in `.claude/rules`; AGY consumes the canonical tree. `.codex/rules` remains command-approval policy, never instruction content.

| Rule | Scope | Targets | Enforcement | Source |
|---|---|---|---|---|
| `authoring.artifact-layout` — Right-sized artifact layout | always | claude, codex, agy | Advisory | [`.agents/rules/authoring/artifact-layout.md`](.agents/rules/authoring/artifact-layout.md) |
| `operations.persistent-branches` — Persistent work-area branches | always | claude, codex, agy | Guard | [`.agents/rules/operations/persistent-branches.md`](.agents/rules/operations/persistent-branches.md) |
| `workflow.agentic-workflow` — Agentic Workflow protocol | always | claude, codex, agy | Guard | [`.agents/rules/workflow/agentic-workflow.md`](.agents/rules/workflow/agentic-workflow.md) |
<!-- aw:meta-rule-index:end -->
