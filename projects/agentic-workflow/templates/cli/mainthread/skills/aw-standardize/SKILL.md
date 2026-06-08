---
name: aw:standardize
description: Bring an existing project under AW ownership one bounded standardization tick at a time.
user-invocable: true
aliases: [aw:standardize-run]
---

# /aw:standardize

Human-facing entrypoint for adopting, migrating, or standardizing an existing
project under Agentic Workflow ownership. The user asks for the outcome; the
agent uses `aw standardize ...` CLI commands as the bounded protocol surface.

AW standardization is the workflow/remediation surface for making a project
adoptable by Agentic Workflow. Project readiness metrics live in
`aw health`; standardize commands should tell the agent what bounded
work to do next.

AW standardization has five workflow layers plus one optional
automation-maturity workflow:

- `capability`: product-control layer. README capability roots use Markdown tables and can feed `aw run --project`.
- `managed`: adoption layer. Every in-scope source file has `CODEGEN` or `HANDWRITE`.
- `semantic`: coverage layer. Source IR is mapped to semantic TD sections and generator primitive gaps.
- `traceability`: closure workflow. Active commands, TDs, source refs, and CB blocks must close to README capabilities through TD refs.
- `regenerable`: automation-maturity workflow. `HANDWRITE` shrinks as generator primitives mature, but this is not a production-readiness gate unless a capability explicitly declares it.

## Workflow

1. Resolve the project from the prompt, current branch, or `.aw/config.toml`.
2. Run:
   ```bash
   aw standardize capability run <project> --max-ticks 1 --json
   ```
3. If `completion.complete=false`, inspect `completion.missing` and
   `next_action.command`, do one bounded tick, and rerun `capability run`. Do
   not proceed to managed standardization until capability completion is true.
4. When capability has no deterministic next action, run:
   ```bash
   aw standardize managed run <project> --non-interactive --json
   ```
5. If managed reports a deterministic next action, perform one bounded tick
   and rerun `managed run`.
6. When managed has no deterministic next action, run:
   ```bash
   aw standardize semantic run <project> --non-interactive --json
   ```
7. If semantic reports a deterministic next action, perform one bounded tick
   and rerun `semantic run`.
8. When semantic has no deterministic next action, run:
   ```bash
   aw standardize traceability run <project> --non-interactive --json
   ```
9. If traceability reports a blocker, do one bounded classification tick:
   - For command blockers, decide `promote` by mapping command -> TD `command_refs` -> README `capability_refs`, or `delete` by removing the command from runtime, active docs, skills, templates, tests, and support code.
   - For TD/source/CB blockers, attach the edge to a capability-owned TD, mark TDs `capability_scope: internal` only when no production source/CB edge exists, or delete dead material.
   - Do not bulk backfill unrelated TDs or commands.
   Then rerun `aw standardize traceability run <project> --non-interactive --json`.
10. When traceability has no deterministic next action, run the project-health
   metric and gate report:
   ```bash
   aw health <project> --verify-traceability --verify-cb --verify-cold --verify-tests --json
   ```
11. If health reports `production_ready=false` or blockers, fix the reported
   managed, semantic, traceability, cb verify, cold rebuild, configured test,
   stack, or workflow-lock gate and rerun the workflow layer that produced the
   blocker.
12. When health reports `production_ready=true`, the standardize remediation
   workflow can stop for the current scope even if `regenerable_percent < 100`.
   Run regenerability only as optional
   automation-maturity work:
   ```bash
   aw standardize regenerable run <project> --non-interactive --json
   ```
13. If regenerable reports a deterministic next action, perform one bounded
   maturity tick only when the user or capability asks for CODEGEN promotion.
14. If any required layer reports a blocker, read `next_action` and do the mainthread
   work directly:
   - `capability-format-migration`: migrate YAML/legacy capability maps into Markdown tables.
   - `fix_spec_rule`: edit the target TD spec until `aw td check <target>` passes.
   - `regen_drift`: regenerate or repair the affected CODEGEN block, then run `aw cb check <target>`.
   - `semantic_td_missing`: create or update the semantic TD only when AST/source evidence supports the claim.
   - `generator_primitive_gap`: improve the generator primitive or open/update the work item that owns that gap.
   - `command_no_td_ref`: classify one command as promote/delete; promote by adding a TD `command_refs` claim with valid `capability_refs`, or delete it from the active surface.
   - other blocked actions: answer the question in the envelope or make the indicated targeted edit.
15. After each mainthread edit, rerun the layer that produced the blocker.

## Rules

- Do one bounded action at a time.
- Do not skip verification after a mainthread edit.
- Capability completion only means README root structure is runnable; it does
  not imply source ownership or production readiness.
- Capability standardize JSON completion is authoritative for this layer. A
  `next_action.kind=none` without `completion.complete=true` is a blocker, not
  a finished standardization layer.
- Generated, vendored, or explicitly out-of-scope files still need binary
  ownership: use tracked `HANDWRITE` when the generator cannot produce them.
- Managed completion only means ownership coverage; it does not imply semantic
  completeness.
- Semantic completion means no next deterministic Source IR -> TD coverage gap
  remains without a human decision or generator design issue.
- Traceability completion means every active command, TD, source ref, and CB block closes to at least one README capability unless the TD is valid internal scope.
- Production readiness is reported by `aw health`, not by
  `aw standardize` itself. Health is gated by capability, managed, semantic,
  traceability, cb verify, cold verify, configured test gates, and unresolved blocker/HITL state.
  Regenerability percentage is an automation-maturity signal, not a required
  100% gate.
- Regenerable completion means no next deterministic `HANDWRITE` -> `CODEGEN`
  promotion remains; partial regenerability is acceptable when remaining gaps
  are tracked or require generator design work.
- Use `aw health <project> --verify-traceability --verify-cb --verify-cold --verify-tests --json`
  for the full metric surface: capability readiness, `managed_percent`,
  `semantic_percent`, `traceability_percent`, `command_traceability_percent`,
  `regenerable_percent`, cb verify, cold verify, test gates, `production_ready`,
  `next_gap`, `blocked_gap_count`, `human_decision_required_count`, and optional
  regenerability gaps.
