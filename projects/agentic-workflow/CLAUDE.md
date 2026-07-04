# SDD — Spec-Driven Development

## Mission: regenerability invariant

> **Standardized** means: delete the entire codebase, re-run codegen on
> `.aw/tech-design/`, replay all `HANDWRITE-BEGIN/END` blocks from
> their payload sources, and the resulting tree is **byte-equivalent**
> to the deleted tree.

Implications:

- **Spec is the source of truth.** Code is a derived artifact.
- **HANDWRITE blocks are the only legitimate deviation**, and only because
  codegen does not yet cover that gap. Every HANDWRITE block names the
  gap-blocker (issue / primitive / generator) that will eventually retire it.
- **Closure**: when the gap-blocker lands, `HANDWRITE` → `CODEGEN`, and the
  invariant tightens. A repo at 100% standardization can be deleted and
  rebuilt deterministically.

Full contract: `projects/agentic-workflow/tech-design/surface/specs/score-standardization.md`.

## What this is

SDD is the methodology + library behind the `aw` CLI (`agentic-workflow`).
It owns:

- CRRR lifecycle types (`phase`, `Lifecycle-Stage` trailers, state machines)
- Spec / TD / CB artifact models (`issues/`, `tech_design/`, `generate/`)
- Validation rules (`validate/rules/`)
- Codegen primitives (`generate/`)

## Two workflows (do not conflate)

| Workflow | Drives | Direction | Termination |
|---|---|---|---|
| **正流程 (forward, linear)** | One change at a time: issue → td (author → gen → fill) → code-check. Linear — no review/revise; the gate is EC. | Forward, single-issue | Issue closes |
| **標準化 (regenerability)** | Audit the whole repo + apply 1 fix per tick until invariant holds | Loop, cross-issue | coverage = 100% |

A 標準化 fix often opens a 正流程 issue to land the change, but the two
flows are distinct: 正流程 ends when one issue closes; 標準化 ends only
when the invariant holds across the entire scope. 正流程 is human-paced;
標準化 is loop-paced (cron / driver).

## The standardization actions

Every fix during 標準化 falls into exactly one of these. Drives the
priority order surfaced by `aw health` (highest priority first):

| # | Action | What it does | Today's CLI |
|---|---|---|---|
| 0 | **inventory** | Classify every `.rs` in scope; compute coverage % | `aw health --project <p>` (managed axis) |
| 1 | **regen_drift** | Re-emit a CODEGEN block that drifted from spec output | `aw td gen` regenerates; `aw td code-check` detects |
| 2 | **promote_handwrite** | Gap-blocker closed → HANDWRITE → CODEGEN (byte-equiv) | `aw td promote <path>` |
| 3 | **issue_marker_gap** | HANDWRITE without gap-blocker → file issue + update marker | routed via `aw health` → real `aw wi create` path |
| 4 | **fix_spec_rule** | TD spec violates R1–R7 → fix | `td check` reports; fix by re-authoring the TD section + re-`gen` (no review/revise) |
| 5 | **fold_shadow** | Spec exists but hand-written shadow code lives outside markers | none — hardest gap |
| 6 | **claim_code** | Untracked in-scope code → write spec + wrap HANDWRITE | `aw td code-claim` covers code→spec; CODEGEN promotion is follow-up |

After managed coverage reaches 100%, drive remaining HANDWRITE→CODEGEN
promotions with `aw td promote <path>` (one tick at a time, routed by
`aw health`'s `next.command`): no remaining HANDWRITE blocks in SDD.
The standardize layer verbs are retired — `aw standardize` keeps only
`audit`; `regenerable` maturity is reported via `aw health`.

## Boundary

`agentic-workflow` (the `aw` CLI) should NOT contain:
- Product-specific business logic for any consuming project (lives in that
  project's own `projects/<name>/` tree — e.g. mamba's Python semantics,
  jet's bundler internals)
- Domain-specific CLI UX for other projects (each ships its own binary per
  the CLI convention in the repo-root `CONTRIBUTING.md`)

LLM/agent dispatch wiring for `aw` itself is in-scope here (`src/agents/`,
`src/cli/llm.rs`) — it is not pushed out to a separate crate.

If a primitive here looks specific to one consuming project, push it down
into that project instead of generalizing it in `agentic-workflow/src/cli/`.
