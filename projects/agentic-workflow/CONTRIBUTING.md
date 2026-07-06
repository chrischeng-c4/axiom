# Agentic Workflow Contributing

## Brief

Project-local authoring rules for Agentic Workflow: authoritative inputs,
self-hosting boundaries, and meta-doc placement. Repo-wide authoring rules
remain in [../../CONTRIBUTING.md](../../CONTRIBUTING.md).

## Overview

This file owns project-local authoring rules for Agentic Workflow. Repo-wide
authoring, CLI, service, and meta-doc rules remain in
[../../CONTRIBUTING.md](../../CONTRIBUTING.md).

## Authoritative Inputs

- Product promises and work-root closure live in
  [CAPABILITIES.md](CAPABILITIES.md).
- External contracts live under `external-contracts/`.
- Tech design and generated-code structure live under `tech-design/`.
- CLI behavior must be proven by focused tests under `tests/` or crate unit
  tests, and public command output must remain chainable.

## Self-Hosting Boundary

Agentic Workflow may repair its own lifecycle directly when the lifecycle is
broken. Do not require a full AW loop to fix AW itself. Self-health production
readiness is gated by capability contracts and EC claim closure; managed,
semantic, traceability, cold rebuild, and workspace tests are readiness signals
unless a capability or EC explicitly makes them blocking.

## Meta Docs

The project README is the orientation surface. Keep it small and fixed:

- `## Brief`
- `## Contributing`
- `## Capability Contract`

Do not add project-local `CLAUDE.md` or `AGENTS.md`. Agent-runtime behavior
belongs at the repo/global layer, in skills/templates, or in command output.
