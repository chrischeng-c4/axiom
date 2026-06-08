---
name: aw:td:create
description: Resume a stalled TD lifecycle — picks up from current issue phase and drives the CRRR loop. Mainthread-only; no subagent dispatch.
user-invocable: true
amended_by: aw-mainthread-phase-2-skill-rewrite-and-agent-delete.md
amended_on: "2026-05-03"
---

# /aw:td:create

Resume entry point for stalled or interrupted tech-design workflows.
Reads the current issue phase and picks up where the chain left off.

> **Mainthread-only model (post Phase-2).** Every dispatch envelope
> carries `agent: null`. There is no `aw-td-author` /
> `aw-td-reviewer` / `aw-td-reviser` subagent to dispatch — those
> agent definitions were removed atomically with this skill rewrite.
> Mainthread takes over each step directly. The CLI records the current
> expected payload and exact command in the WI projection; mainthread writes
> only that payload, then either lets the hook run the expected command or runs
> it literally. Section/review apply commands are atomic gates: they validate,
> update WI projection/labels, commit git trailers, and emit the next command.

## Usage

```
/aw:td:create <slug>
```

## Flow

1. Run `aw wi show <slug>` and inspect the hidden `aw:workflow-state`
   block if a workflow lock is active.
2. If the projection has `expected_payload` and `expected_command`, write the
   requested payload and run the exact command if the hook did not auto-run it.
3. If no projection lock is active, use the phase table below to resume:

| Phase | Mainthread action |
|-------|-------------------|
| `td_inited` | Run `aw td create <slug>` to initialize the applicability queue and WI projection |
| `td_applicability_in_progress` | Write `.aw/payloads/<slug>/applicability/<section>.md`, then run the projection's exact `aw td create --apply --phase applicability --section <section>` command |
| `td_applicability_created` | Run `aw td review <slug> --phase applicability --spec-path <path>`, write `.aw/payloads/<slug>/applicability/review.md`, then run the projection's exact review apply command |
| `td_contract_in_progress` | Write `.aw/payloads/<slug>/contract/<section>.md`, then run the projection's exact `aw td create --apply --phase contract --section <section>` command |
| `td_created` | Run `aw td review <slug> --phase contract --spec-path <path>`, write `.aw/payloads/<slug>/contract/review.md`, then run the projection's exact review apply command |
| `td_reviewed` | Approved contract is ready for `aw cb gen`; if review requested revision, follow the active projection lock |
| `td_revised` | Legacy phase: run `aw td review <slug> --spec-path <path>` and follow the emitted envelope |
| `cb_genned` | Dispatch `/aw:cb:fill` to fill HANDWRITE markers |
| `cb_filled` | Run `aw cb review` (mainthread writes review payload + `--apply`) |
| `cb_reviewed` | Check verdict — if `needs-revision`, run `aw cb revise` (mainthread); if `approved`, run `aw td merge` |
| `td_merged` | Already done — report success |

3. For phases that need the spec_path, find it by scanning `projects/agentic-workflow/tech-design/` in the current checkout for `.md` files with `fill_sections` in their frontmatter.

4. Run the mainthread loop directly from the envelope protocol in `AGENTS.md`.
   Do not author status updates by hand; the CLI updates WI projection and git
   trailers from fixed state.

5. If the envelope includes `artifact_quality_profile` or an `Artifact Quality
   Gate`, treat the listed hard preflight gates as required lifecycle evidence.
   For frontend/UI artifacts, the TD must account for desktop and mobile
   viewport evidence, interaction smoke proof, accessibility/readability smoke,
   and placeholder-free primary-state verification through `e2e-test` or other
   machine-checkable artifacts.

### When to use

- Session ended mid-section or mid-review while a WI projection lock is active
- Manual restart after `aw td arbitrate` or `aw cb arbitrate`
