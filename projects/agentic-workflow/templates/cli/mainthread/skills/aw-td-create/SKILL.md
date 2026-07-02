---
name: aw:td:create
description: Resume a stalled TD lifecycle — picks up from current issue phase and drives EC/code-check iteration. Mainthread-only; no subagent dispatch.
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
> expected payload and exact command in the WI projection; mainthread fills
> only that initialized payload, then either lets the hook run the expected command or runs
> it literally. Section apply commands are atomic gates: they validate,
> update WI projection/labels, commit git trailers, and emit the next command.

## Usage

```
/aw:td:create <slug>
```

## Flow

1. Run `aw wi show <slug>` and inspect the hidden `aw:workflow-state`
   block if a workflow lock is active.
2. If the projection has `expected_payload` and `expected_command`, fill the
   initialized payload and run the exact command if the hook did not auto-run it.
3. If no projection lock is active, use the phase table below to resume:

| Phase | Mainthread action |
|-------|-------------------|
| `td_inited` | Run `aw td create <slug>` to initialize the applicability queue and WI projection |
| `td_applicability_in_progress` | Fill `.aw/payloads/<slug>/applicability/<section>.md`, then run the projection's exact `aw td create --apply --phase applicability --section <section>` command |
| `td_applicability_created` | Transient — the linear lifecycle advances straight to the first contract section (or `aw td gen` if the contract pass has no sections), no review step. If no lock is active here, read the `Next-Command` git trailer off the last td commit for this slug and run it verbatim |
| `td_contract_in_progress` | Fill `.aw/payloads/<slug>/contract/<section>.md`, then run the projection's exact `aw td create --apply --phase contract --section <section>` command |
| `td_created` | Transient — the linear lifecycle advances straight to `aw td gen`. If no lock is active here, read the `Next-Command` git trailer off the last td commit for this slug and run it verbatim |
| `td_reviewed` | Retired CRRR phase (issue #850); self-heals to `td_created` on read (`td_phase::normalize`), so `aw wi show` never actually surfaces it — treat as `td_created` and run `aw td gen <slug> --spec-path <path>` |
| `cb_genned` | Run `aw td fill` to fill HANDWRITE markers |
| `cb_filled` | Run `aw td code-check <slug>`; terminal code-check commits closure, and EC/health decide the next iteration |
| `cb_reviewed` / `cb_revised` / `cb_arbitrated` | Retired CRRR phases (issue #850); self-heal to `cb_filled` on read (`td_phase::normalize`), so `aw wi show` never actually surfaces them — treat as `cb_filled` and run `aw td code-check <slug>` |
| `td_merged` | Already done — report success |

`td_revised` is a dead legacy phase with no writer anywhere in the CLI and no
successor command in the current linear lifecycle — if an issue is ever found
resting there, treat it as a bug, not a normal resume state. The four rows
above it (`td_reviewed`, `cb_reviewed`, `cb_revised`, `cb_arbitrated`) predate
the CRRR collapse but DO self-heal via `td_phase::normalize` at every
issue-read site, so they remain safe, documented resume states even though
you should rarely observe them directly.

3. For phases that need the spec_path, find it by scanning `projects/agentic-workflow/tech-design/` in the current checkout for `.md` files with `fill_sections` in their frontmatter.

4. Run the mainthread loop directly from the envelope protocol in `AGENTS.md`.
   Do not author status updates by hand; the CLI updates WI projection and git
   trailers from fixed state.

### When to use

- Session ended mid-section or mid-review while a WI projection lock is active
- Manual restart after a lifecycle command stopped before emitting the next command
