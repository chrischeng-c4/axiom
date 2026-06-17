---
name: aw:capability
description: Product capability completion loop — inspect README capability sections, plan one next action, and keep cap_path/TD refs aligned.
user-invocable: true
---

# /aw:capability

Human-facing entrypoint for project capability alignment. Capability is the
human-confirmed product promise tracked in the project `cap_path`, defaulting to
README. The model may infer and propose, but it must not publish new capability
promises from inference alone.

## Contract

- Human API: `/aw:capability <prompt>`.
- Agent API: use `aw run`, `aw capability report|next|migrate|run|check`,
  `aw standardize <project>`, `aw wi list/show`, `aw td ...`, and `aw cb ...`
  as needed to gather evidence.
- Artifact: `cap_path`, defaulting to the project README when configured or
  implied by `[[projects]].path`.
- Canonical CLI namespace: `aw capability`. Do not use the old shorthand;
  it is ambiguous with capacity.

## Flow

1. Resolve the project from the prompt, current branch, or `.aw/config.toml`.
2. Run `aw capability report --project <project>` to inspect README capability
   sections, WI inventory, TD refs, and evidence.
3. Run `aw capability next --project <project>` when deciding the next bounded
   action. Follow the single `next_action` unless it requires HITL.
4. If `next_action.kind=format_migration_required`, run
   `aw capability migrate --project <project>`, then rerun
   `aw capability check --project <project>`.
5. For root-driven execution, run `aw run --project <project> --max-ticks 1`
   and follow `invoke.command` plus `agent_prompt` until
   `completion.workflow_complete=true` or `requires_hitl=true`. Do not stop on
   `action=done` alone; a child root can be done while the parent still needs
   rollup.
6. Use `aw capability check --project <project> --verify` after README or TD
   linkage edits when production proof matters; omit `--verify` only for a
   fast structural check.
7. Only after explicit confirmation, propose edits that create or materially
   change capability promises.

## README Schema

Published capability maps are Markdown-first. H1 is the project root,
`## Brief` is the agent-readable project summary, `## Capabilities` is the
capability registry, and `### Capability Index` is the compact scan surface.
Capability roots are H3-Hn headings under `## Capabilities`; each heading maps
to an epic/subepic work root. Atomic `change` WIs usually come from
`aw wi atomize`, not README rows.

```md
# Jet

## Brief

Rust-native frontend toolchain for package, build, dev-server, test, e2e, and
browser automation workflows.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Package Manager | #3779 | implemented | verified | conformance | ready | npm/pnpm-style install and lockfile replacement |

### Package Manager

ID: package-manager
Type: DeveloperTool
Root WI: #3779
Status: auditing
Surfaces: CLI: `jet install` + `jet add` + `jet remove` - package lifecycle entrypoints
EC Dimensions: behavior: `jet test` - lockfile and install conformance; efficiency: `meter` - install/cache resource profile
Efficiency Operating Point: local-vat-jet-install
Efficiency Cube: projects/jet/.aw/ec/efficiency/install.cube.json
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet can replace npm/pnpm/Bun package-management flows.
Gate Inventory:
- projects/jet/validation/pkg-manager.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lockfile parity | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |

#### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)

Operating point: local-vat-jet-install
Cube: projects/jet/.aw/ec/efficiency/install.cube.json
```

Status enums:

- capability `status`: `candidate`, `confirmed`, `auditing`, `blocked`,
  `verified`, `retired`.
- gap `status`: `open`, `in_progress`, `blocked`, `closed`, `deferred`.
- work-root `impl`: `planned`, `partial`, `implemented`, `blocked`, `out_of_scope`.
- work-root `verification`: `none`, `planned`, `failing`, `passing`,
  `verified`, `blocked`.
- claim/work-root `maturity`: `none`, `smoke`, `conformance`, `corpus`,
  `negative`, `dogfood`.

Status-gated contract rules:

- `candidate` capabilities may omit required verification.
- `confirmed`, `auditing`, `blocked`, and `verified` capabilities must define
  required verification through tables.
- Required claims default `required_for_verified: true` and must include a
  maturity plus either a gate command or fixture/inventory reference.
- Gate pass/fail is runtime-only from
  `aw capability report --project <project> --verify`; do not store pass
  timestamps in README.

YAML `## Capability:` sections, Field/Value contract tables, and one-row
capability contract tables are migration input only. They must produce
`format_migration_required` and cannot count as verified until migrated to the
canonical field-style Markdown contract above.

## TD Linkage

New non-internal TDs should declare frontmatter `capability_refs`; internal-only
TDs declare `capability_scope: internal`.

```yaml
capability_refs:
  - id: package-manager
    role: primary
    gap: package-manager-readiness
    claim: package-manager-lockfile-parity
    coverage: partial
    rationale: "This TD closes the package-manager readiness audit gap."
```

`role` is `primary`, `contributes`, `affected`, `regression_guard`, or
`out_of_scope`. `coverage` is `full`, `partial`, `enabling`, or `guardrail`.
At least one `primary` ref is required when refs are present and the TD is not
internal. Primary refs to capabilities with `verification_contract` must include
`claim`. `aw td check` validates declared refs against README capability, gap,
and claim IDs.

## Rules

- Never silently edit `cap_path`.
- Do not treat README prose as confirmed if the human says the direction has
  changed; produce a revised candidate and ask again.
- If TD/CB evidence contradicts the capability map, report drift and ask before
  updating the anchor.
- 100% means every non-retired capability is `verified`, all non-deferred gaps
  are closed, every required claim is linked to TD/WI evidence, required gates
  pass under `aw capability report --project <project> --verify`, and TD/WI
  refs resolve.
- Stdout envelope completion is authoritative for automation. If
  `completion.workflow_complete=false`, run the envelope `invoke.command` or
  resolve the listed `completion.missing` items before reporting completion.
- Prefer one bounded tick at a time: `report -> next -> migrate/check` for
  format conversion, otherwise `report -> next -> run --max-ticks 1`.
