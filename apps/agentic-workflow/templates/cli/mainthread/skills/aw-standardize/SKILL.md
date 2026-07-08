---
name: aw:standardize
description: Bring an existing project under AW ownership using the audit-first preservation protocol, then follow aw health for bounded remediation.
user-invocable: true
aliases: [aw:standardize-run]
---

# /aw:standardize

Human-facing entrypoint for adopting or standardizing an existing project
under Agentic Workflow ownership. The user asks for the outcome; the agent
uses `aw standardize audit` plus `aw health`-routed worker verbs as the
bounded protocol surface.

`aw standardize` is reduced to the audit-first preservation protocol
(`audit check` / `audit record`); it is not a parent-workflow loop driver.
Project readiness metrics -- capability, managed, semantic, traceability,
command traceability, and regenerable maturity, plus cb/cold verify and
configured test gates -- live entirely in `aw health`, and `aw health`'s
`next.command` already names the exact worker verb to run next
(`aw td promote <target>`, `aw td code-claim <path>`, `aw td gen <slug>` /
`aw td code-check <slug>`, `aw wi create ...`, `aw capability run --project
<p>`, ...). There is no `aw standardize` layer-driver subcommand (`managed`,
`semantic`, or `traceability`) anymore.

## Workflow

1. Resolve the project from the prompt, current branch, or `aw.toml`.
2. Run the audit-first preservation check before any remediation:
   ```bash
   aw standardize audit check --project <project>
   ```
   If `audit_required=true`, record the preservation baseline once:
   ```bash
   aw standardize audit record --project <project>
   ```
   `surfaces_to_preserve` / `safe_levers` name the surfaces that must survive
   remediation unchanged; treat them as guardrails for every later tick, not
   optional advice.
3. Run the project health report:
   ```bash
   aw health --project <project>
   ```
4. Follow stdout exactly. If `completion.workflow_complete=true` (or
   `production_ready=true`), stop. If `next.kind=run_command`, run the exact
   `next.command` as one bounded tick, then rerun `aw health --project
   <project>` to pick up the next `next.command`/`followups` entry.
5. If a traceability blocker names a command or TD/source/CB edge, do one
   bounded classification tick:
   - For command blockers, decide `promote` by mapping command -> TD
     `command_refs` -> README `capability_refs`, or `delete` by removing the
     command from runtime, active docs, skills, templates, tests, and support
     code.
   - For TD/source/CB blockers, attach the edge to a capability-owned TD,
     mark TDs `capability_scope: internal` only when no production
     source/CB edge exists, or delete dead material.
   - Do not bulk backfill unrelated TDs or commands.
   Then rerun `aw health --project <project>`.
6. When health reports `production_ready=true`, standardization can stop for
   the current scope even if `regenerable_percent < 100`. Treat
   regenerability as optional automation-maturity work unless the user or a
   capability explicitly requires CODEGEN promotion; `aw health`'s
   `followups` still lists any remaining regenerable-layer worker verb if the
   user wants to keep going.
7. If `next.command` is not obviously actionable, read `followups` /
   `blockers` and do the mainthread work directly:
   - `fix_spec_rule`: edit the target TD spec until `aw td check <target>`
     passes.
   - `regen_drift`: regenerate or repair the affected CODEGEN block, then run
     `aw td code-check <target>`.
   - `semantic_td_missing` / `generator_primitive_gap`: create or update the
     semantic TD only when AST/source evidence supports the claim, or open a
     work item for the generator gap.
   - `command_no_td_ref`: classify one command as promote/delete; promote by
     adding a TD `command_refs` claim with valid `capability_refs`, or delete
     it from the active surface.
   - other blocked actions: answer the question in the envelope or make the
     indicated targeted edit.
8. After each mainthread edit, rerun `aw health --project <project>`.

## Rules

- Do one bounded action at a time.
- Do not skip verification after a mainthread edit.
- `aw standardize audit` guardrails apply for the whole remediation loop: do
  not remove or degrade a surface listed in `surfaces_to_preserve` while
  chasing a health gap.
- Capability completion only means README root structure is runnable; it
  does not imply source ownership or production readiness.
- Generated, vendored, or explicitly out-of-scope files still need binary
  ownership: use tracked `HANDWRITE` when the generator cannot produce them.
- Managed completion only means ownership coverage; it does not imply
  semantic completeness.
- Semantic completion means no next deterministic Source IR -> TD coverage
  gap remains without a human decision or generator design issue.
- Traceability completion means every active command, TD, source ref, and CB
  block closes to at least one README capability unless the TD is valid
  internal scope.
- Production readiness is reported by `aw health`, not by `aw standardize`
  itself. Health is gated by capability, managed, semantic, traceability, cb
  verify, cold verify, configured test gates, and unresolved blocker/HITL
  state. Regenerability percentage is an automation-maturity signal, not a
  required 100% gate.
- Regenerability maturity means deterministic `HANDWRITE` -> `CODEGEN`
  promotions have been exhausted for the current generator surface; partial
  regenerability is acceptable when remaining gaps are tracked or require
  generator design work.
- Use `aw health <project>` for the full metric surface: capability
  readiness, `managed_percent`, `semantic_percent`, `traceability_percent`,
  `command_traceability_percent`, `regenerable_percent`, cb verify, cold
  verify, test gates, `production_ready`, `next_gap`, `blocked_gap_count`,
  `human_decision_required_count`, and optional regenerability gaps.
