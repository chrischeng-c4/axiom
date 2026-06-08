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

AW standardization has four layers:

- `capability`: product-control layer. README capability roots use Markdown tables and can feed `aw run --project`.
- `managed`: adoption layer. Every in-scope source file has `CODEGEN` or `HANDWRITE`.
- `semantic`: coverage layer. Source IR is mapped to semantic TD sections and generator primitive gaps.
- `regenerable`: automation-maturity layer. `HANDWRITE` shrinks as generator primitives mature, but this is not a production-readiness gate unless a capability explicitly declares it.

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
8. When semantic has no deterministic next action, run the production gate:
   ```bash
   aw project health <project> --verify-cold --json
   ```
9. If health reports `production_ready=false` or blockers, fix the reported
   managed, semantic, cb verify, cold rebuild, stack, or workflow-lock gate and
   rerun the layer that produced the blocker.
10. When `production_ready=true`, the project can roll up as standardize-complete
   even if `regenerable_percent < 100`. Run regenerability only as optional
   automation-maturity work:
   ```bash
   aw standardize regenerable run <project> --non-interactive --json
   ```
11. If regenerable reports a deterministic next action, perform one bounded
   maturity tick only when the user or capability asks for CODEGEN promotion.
12. If any required layer reports a blocker, read `next_action` and do the mainthread
   work directly:
   - `capability-format-migration`: migrate YAML/legacy capability maps into Markdown tables.
   - `fix_spec_rule`: edit the target TD spec until `aw td check <target>` passes.
   - `regen_drift`: regenerate or repair the affected CODEGEN block, then run `aw cb check <target>`.
   - `semantic_td_missing`: create or update the semantic TD only when AST/source evidence supports the claim.
   - `generator_primitive_gap`: improve the generator primitive or open/update the work item that owns that gap.
   - other blocked actions: answer the question in the envelope or make the indicated targeted edit.
13. After each mainthread edit, rerun the layer that produced the blocker.

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
- Production readiness is gated by capability, managed, semantic, cb verify,
  cold verify, and unresolved blocker/HITL state. Regenerability percentage is
  an automation-maturity signal, not a required 100% gate.
- Regenerable completion means no next deterministic `HANDWRITE` -> `CODEGEN`
  promotion remains; partial regenerability is acceptable when remaining gaps
  are tracked or require generator design work.
- Use `aw project health <project> --json` to report `managed_percent`,
  `semantic_percent`, `regenerable_percent`, `production_ready`, `next_gap`,
  `blocked_gap_count`, `human_decision_required_count`, and optional
  regenerability gaps.
