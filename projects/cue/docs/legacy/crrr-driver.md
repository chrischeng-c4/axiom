# cue CRRR driver - legacy envelope handling

Status: legacy scaffold.

cue is now a web-based Prompt-to-Governed-App platform. This document describes
the old terminal issue-CRRR prototype and should not guide new product work.
Keep it only as implementation history; the Rust TUI code it described has been
removed from the active Cue product workspace.

## Scope

Issue-CRRR phases only (per issue R13):

```
Requirements → Scope → ReferenceContext → Review → (Revise →) Merged
```

Post-issue SDD phases (change_init, spec, impl, merge) are out of scope.

## Envelope shapes

Mirror `projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md`.

| Action     | Fields                                   | Driver effect                                         |
|------------|------------------------------------------|-------------------------------------------------------|
| `dispatch` | `agent?`, `slug`, `invoke?`              | Log; infer next phase from `agent` / `invoke.command` |
| `done`     | `slug`, `message?`                       | Phase → `Merged`                                      |
| `error`    | `slug`, `message`                        | Log; phase unchanged (UI surfaces the error)          |
| `batch`    | `recommendations[]`                      | Log count; driver stays idle (operator dispatches)    |

## Phase inference from dispatch

| `invoke.command` contains | Phase effect                              |
|---------------------------|-------------------------------------------|
| `merge`                   | → `Merged`                                |
| `validate`                | Phase unchanged (round-trip marker only)  |

| `agent`                 | Phase effect (current → next)                         |
|-------------------------|-------------------------------------------------------|
| `score-issue-author`    | `Idle`/`Requirements`/`Scope`/`ReferenceContext` → same phase (author drives the section) |
| `score-issue-reviewer`  | any → `Review`                                         |
| `score-issue-reviser`   | any → `Revise`                                         |

## Gate (needs-revision) state machine

```
Review ──open_gate(flagged)──▶ AwaitingGate
AwaitingGate ──approve_override──▶ Merged
AwaitingGate ──dispatch_reviser · review_count<2──▶ Revise
AwaitingGate ──dispatch_reviser · review_count≥2──▶ Arbitrate
AwaitingGate ──cancel──▶ Idle
```

`review_count` increments each time `open_gate` is called; two increments → third
`dispatch_reviser` is redirected to `Arbitrate` (per issue R10 + CRRR spec R12).

## Wiring note (R7)

The historical MVP driver processed envelopes but did **not** itself spawn
agents. The referenced `projects/cue/src/tui/` implementation no longer exists;
new Cue orchestration belongs in the web control plane, not this terminal
driver.
