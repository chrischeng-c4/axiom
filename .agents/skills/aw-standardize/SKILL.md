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

The parent workflow coordinates these readiness layers and maturity signals:

- `capability`: product-control layer routed through `aw capability`. README capability roots use Markdown tables and can feed `aw run --project`.
- `managed`: adoption layer. Every in-scope source file has `CODEGEN` or `HANDWRITE`.
- `semantic`: coverage layer. Source IR is mapped to semantic TD sections and generator primitive gaps.
- `traceability`: closure workflow. Active commands, TDs, source refs, and CB blocks must close to README capabilities through TD refs.
- `regenerable`: automation-maturity signal reported by `aw health`. `HANDWRITE` shrinks as generator primitives mature, but this is not a production-readiness gate unless a capability explicitly declares it.

## Workflow

1. Resolve the project from the prompt, current branch, or `.aw/config.toml`.
2. Run the parent workflow first:
   ```bash
   aw standardize <project>
   ```
3. Follow stdout exactly. If `completion.workflow_complete=true`, stop. If
   `next.kind=run_command`, run the exact `next.command`, perform one bounded
   tick, then rerun `aw standardize <project>` for rollup.
4. Capability remediation is routed through `aw capability run <project>
   --non-interactive --max-ticks 1` when the parent emits that command.
5. For targeted layer debugging, use the layer command without compatibility
   flags:
   ```bash
   aw standardize managed run <project> --non-interactive --max-ticks 1
   aw standardize semantic run <project> --non-interactive --max-ticks 1
   aw standardize traceability run <project> --non-interactive --max-ticks 1
   ```
6. If traceability reports a blocker, do one bounded classification tick:
   - For command blockers, decide `promote` by mapping command -> TD `command_refs` -> README `capability_refs`, or `delete` by removing the command from runtime, active docs, skills, templates, tests, and support code.
   - For TD/source/CB blockers, attach the edge to a capability-owned TD, mark TDs `capability_scope: internal` only when no production source/CB edge exists, or delete dead material.
   - Do not bulk backfill unrelated TDs or commands.
   Then rerun `aw standardize <project>`.
7. When standardization layers are ready, run the project-health metric and gate
   report:
   ```bash
   aw health <project>
   ```
8. If health reports `production_ready=false` or blockers, fix the reported
   managed, semantic, traceability, cb verify, cold rebuild, configured test,
   stack, or workflow-lock gate and rerun the workflow layer that produced the
   blocker.
9. When health reports `production_ready=true`, the standardize remediation
   workflow can stop for the current scope even if `regenerable_percent < 100`.
   Treat regenerability as optional automation-maturity work unless the user or
   capability asks for CODEGEN promotion.
10. If any required layer reports a blocker, read `next_action` and do the mainthread
   work directly:
   - `capability-format-migration`: migrate YAML/legacy capability maps into Markdown tables.
   - `fix_spec_rule`: edit the target TD spec until `aw td check <target>` passes.
   - `regen_drift`: regenerate or repair the affected CODEGEN block, then run `aw td code-check <target>`.
   - `semantic_td_missing`: create or update the semantic TD only when AST/source evidence supports the claim.
   - `generator_primitive_gap`: improve the generator primitive or open/update the work item that owns that gap.
   - `command_no_td_ref`: classify one command as promote/delete; promote by adding a TD `command_refs` claim with valid `capability_refs`, or delete it from the active surface.
   - other blocked actions: answer the question in the envelope or make the indicated targeted edit.
11. After each mainthread edit, rerun `aw standardize <project>`.

## Rules

- Do one bounded action at a time.
- Do not skip verification after a mainthread edit.
- Capability completion only means README root structure is runnable; it does
  not imply source ownership or production readiness.
- Capability completion in the stdout envelope is authoritative for this layer. A
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
  Artifact quality gates emitted by `aw run` or project health are part of this
  production gate. For frontend/UI surfaces, do not claim readiness without
  machine-verifiable desktop/mobile viewport, interaction, readability, and
  placeholder-free evidence.
  Regenerability percentage is an automation-maturity signal, not a required
  100% gate.
- Regenerability maturity means deterministic `HANDWRITE` -> `CODEGEN`
  promotions have been exhausted for the current generator surface; partial
  regenerability is acceptable when remaining gaps are tracked or require
  generator design work.
- Use `aw health <project>`
  for the full metric surface: capability readiness, `managed_percent`,
  `semantic_percent`, `traceability_percent`, `command_traceability_percent`,
  `regenerable_percent`, cb verify, cold verify, test gates, `production_ready`,
  `next_gap`, `blocked_gap_count`, `human_decision_required_count`, and optional
  regenerability gaps.
