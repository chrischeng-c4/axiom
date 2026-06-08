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
- Agent API: use `aw run`, `aw capability report|next|run|check`,
  `aw standardize capability ...`, `aw wi list/show`, `aw td ...`, and
  `aw cb ...` as needed to gather evidence.
- Artifact: `cap_path`, defaulting to the project README when configured or
  implied by `[[projects]].path`.
- Canonical CLI namespace: `aw capability`. Do not use the old shorthand;
  it is ambiguous with capacity.

## Flow

1. Resolve the project from the prompt, current branch, or `.aw/config.toml`.
2. Run `aw capability report <project> --json` to inspect README capability
   sections, WI inventory, TD refs, and evidence.
3. Run `aw capability next <project> --json` when deciding the next bounded
   action. Follow the single `next_action` unless it requires HITL.
4. For root-driven execution, run `aw run --project <project> --max-ticks 1
   --json` and follow `invoke.command` plus `agent_prompt` until
   `completion.workflow_complete=true` or `requires_hitl=true`. Do not stop on
   `action=done` alone; a child root can be done while the parent still needs
   rollup.
5. Use `aw capability check <project> --json --verify` after README or TD linkage edits when production proof matters; omit `--verify` only for a fast structural check.
6. Only after explicit confirmation, propose edits that create or materially
   change capability promises.

## README Schema

Published capability maps are Markdown-first. H1 is the project root. H2-Hn
headings are capability/sub-capability roots, and each heading maps to an
epic/subepic work root. Atomic `change` WIs usually come from `aw wi atomize`,
not README rows.

```md
## Package Manager

| Field | Value |
|---|---|
| ID | package-manager |
| Root WI | #3779 |
| Status | auditing |
| Promise | Jet can replace npm/pnpm/Bun package-management flows. |
| Required Verification | smoke, conformance, corpus, negative |
| Gate Inventory | projects/jet/validation/pkg-manager.toml |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lockfile parity | epic | #3779 | partial | planned | conformance | projects/jet/validation/pkg-manager.toml |
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
- Gate pass/fail is runtime-only from `aw capability report --verify`; do not
  store pass timestamps in README.

YAML `## Capability:` sections and legacy capability tables are migration input
only. They must produce `format_migration_required` and cannot count as
verified until migrated to Markdown tables.

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
  pass under `aw capability report --verify`, and TD/WI refs resolve.
- JSON envelope completion is authoritative for automation. If
  `completion.workflow_complete=false`, run the envelope `invoke.command` or
  resolve the listed `completion.missing` items before reporting completion.
- Prefer one bounded tick at a time: `report -> next -> run --max-ticks 1`.
