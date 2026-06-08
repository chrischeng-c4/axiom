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

SDD is the methodology + library that powers `score` and (eventually)
`conductor`. It owns:

- CRRR lifecycle types (`phase`, `Lifecycle-Stage` trailers, state machines)
- Spec / TD / CB artifact models (`issues/`, `tech_design/`, `generate/`)
- Validation rules (`validate/rules/`)
- Codegen primitives (`generate/`)

## Two workflows (do not conflate)

| Workflow | Drives | Direction | Termination |
|---|---|---|---|
| **正流程 (forward CRRR)** | One change at a time: issue → td → cb → merge | Forward, single-issue | Issue closes |
| **標準化 (regenerability)** | Audit the whole repo + apply 1 fix per tick until invariant holds | Loop, cross-issue | coverage = 100% |

A 標準化 fix often opens a 正流程 issue to land the change, but the two
flows are distinct: 正流程 ends when one issue closes; 標準化 ends only
when the invariant holds across the entire scope. 正流程 is human-paced;
標準化 is loop-paced (cron / driver).

## The standardization actions

Every fix during 標準化 falls into exactly one of these. Drives the
priority order in `aw standardize managed next` (highest priority first):

| # | Action | What it does | Today's CLI |
|---|---|---|---|
| 0 | **inventory** | Classify every `.rs` in scope; compute coverage % | `aw standardize managed report` / `aw standardize managed next` |
| 1 | **regen_drift** | Re-emit a CODEGEN block that drifted from spec output | `aw cb gen` regenerates; `cb check` detects |
| 2 | **promote_handwrite** | Gap-blocker closed → HANDWRITE → CODEGEN (byte-equiv) | none — promotion is manual |
| 3 | **issue_marker_gap** | HANDWRITE without gap-blocker → file issue + update marker | manual |
| 4 | **fix_spec_rule** | TD spec violates R1–R7 → fix | `td check` reports; fix via CRRR `td revise` |
| 5 | **fold_shadow** | Spec exists but hand-written shadow code lives outside markers | none — hardest gap |
| 6 | **claim_code** | Untracked in-scope code → write spec + wrap HANDWRITE | `aw cb claim` covers code→spec; CODEGEN promotion is follow-up |

After managed coverage reaches 100%, use `aw standardize regenerable run sdd`
to drive the second layer: no remaining HANDWRITE blocks in SDD.

## Boundary

SDD should NOT contain:
- Score-specific orchestration (lives in `projects/agentic-workflow/`)
- Conductor-specific UX (lives in `projects/conductor/`)
- LLM provider wiring (lives in `crates/cclab-agent`)

If a primitive in SDD looks score-specific, push it up to `projects/agentic-workflow/src/cli/`.

## Trajectory

Currently positioned as an arsenal crate so multiple projects (score,
conductor) can consume it. **Planned**: promote to `projects/agentic-workflow/` once
it grows its own skills + CLI surface — at that point follow the same
surfaces principle as `projects/agentic-workflow/CLAUDE.md` (skills face-to-user,
CLI face-to-agent).
