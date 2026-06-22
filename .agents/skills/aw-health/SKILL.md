---
name: aw:health
description: Run AW project health/readiness gates. If no project is supplied, infer it from the current project-<name> branch and run aw health --project with the resolved project token.
user-invocable: true
aliases: [aw:project-health]
---

# /aw:health

Human-facing entrypoint for Agentic Workflow project readiness. Use it when the
user asks whether a project is healthy, production-ready, blocked, or why
`aw run` / `aw standardize` cannot finish.

## Project Resolution

1. If the prompt includes a project token, use it directly:
   ```bash
   aw health --project <project>
   ```
2. If no project is supplied, infer from the current branch:
   ```bash
   git branch --show-current
   ```
   - `project-<token>` -> run `aw health --project <token>`.
   - Example: branch `project-aw` -> `aw health --project aw`.
3. If the inferred token is rejected as an unknown project, read
   `.aw/config.toml` and resolve the token against `[[projects]].name` and
   `[[projects]].aliases`, then rerun with the matching project name.
4. If the branch is not `project-<token>` and the user did not provide a
   project, stop and ask for the project name.

## Command

`aw health` emits a low-token AW takeover envelope by default: humans scan
`adoption`, and agents chase incomplete items through `followups`. Do not add
`--json`; it is a deprecated compatibility no-op. Use `--human` or `--pretty`
only when the user asks for a human-readable or debug-formatted report. Use
`-v/--verbose` only when progress events are useful. Use `full` or focused
sections such as `regenerable`, `gates`, `hygiene`, `ec`, `td-lock`, or
`blockers` only when detail is needed.

```bash
aw health --project <project>
aw health --project <project> regenerable
aw health --project <project> hygiene
aw health --project <project> full
```

Use the stdout envelope as authoritative:

- `ready=true` and `completion.workflow_complete=true`: report AW takeover
  ready.
- `completion.requires_hitl=true`: surface the HITL reason and stop.
- `next.kind=run_command`: run the exact `next.command` only if the user asked
  to fix or continue the workflow; otherwise report it as the next action.
- `adoption`: report only the five big areas and their `state`/`summary` when
  the user wants a status read.
- `followups`: use these as the agent work queue for incomplete areas; each
  item may include `command`, compact `metrics`, and `blockers_preview`.
- `implementation` followups include configured test and code hygiene gates;
  use `aw health --project <project> hygiene` when format, lint, or warning
  status is the only needed detail.
- `blockers.production_blocker_count` or `blockers.blocker_count` > 0: treat as
  global readiness context, but prefer `followups` for what to do next.
- `payload_path`: inspect only when stdout is too summarized or the user asks
  for detail.

## Rules

- Health is a measurement surface. Do not edit files just because health is
  failing unless the user asked to fix the project.
- Do not confuse `regenerable_percent` with production readiness. The gate is
  the default `ready` value for AW takeover, and detailed production readiness
  lives in `aw health --project <project> full`.
- If `aw health --verbose` prints progress events before the final result,
  wait for the final `event=result` envelope before answering.
- Prefer the installed `aw` only after it has been built or verified recently;
  when results look stale, build or use `target/debug/aw` from the checkout.
