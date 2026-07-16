# Agentic Workflow

## Brief

Agentic Workflow (`aw`) is a project-iteration CLI that lets coding agents
ship bounded, verified work without a human steering every step. You describe
what a project promises (capabilities), and `aw` drives the rest of the loop:
each promise becomes bounded work items, every work item gets an external
contract (an independent test-backed definition of "done"), tech design and
code generation produce the implementation, and nothing closes until the
contract's gates run green. Progress rolls up from work item to capability to
project, so "is it actually done?" is always an evidence-backed answer, not a
claim.

As a human you mostly touch three surfaces:

- `aw capability run --project <name>` (or `aw wi run <id>`) — hand a project,
  capability, or single work item to an agent and let the CLI drive it end to
  end.
- `aw health --project <name>` — the read-only dashboard: readiness, gates,
  blockers, and the exact next command when something needs attention.
- `aw ec review` — the human judgment point: approve or bounce the external
  contracts that define what "done" means (reviews can be agent-backed or
  deferred for post-completion batch review via `ec_review_backing` /
  `ec_review_mode` in `aw.toml`).

Everything else is agent-facing: agents orient with `aw llm` and then follow
the CLI's own stdout (`next.command`) from one step to the next.

## Contributing

Project-local authoring rules for Agentic Workflow: authoritative inputs,
self-hosting boundaries, and meta-doc placement. Repo-wide authoring rules
remain in [../../CONTRIBUTING.md](../../CONTRIBUTING.md). Full rules:
[CONTRIBUTING.md](CONTRIBUTING.md).

## Capability Contract

Machine-readable capability contract for Agentic Workflow. Full contract:
[CAPABILITIES.md](CAPABILITIES.md).
