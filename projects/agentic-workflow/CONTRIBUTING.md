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

## Local Workflow

- Treat `CAPABILITIES.md` as the project promise map and keep capability IDs,
  work-root rows, WI refs, TD refs, and EC evidence resolvable.
- For AW self-hosting repairs, direct commits are allowed when the lifecycle
  itself is broken; otherwise prefer the active `aw wi` -> `aw td` -> `aw td
  code-check` route.
- For CLI surface changes, update the command implementation, generated or
  mirrored agent docs, active skill templates, and focused CLI tests together.
- For `SPEC-MANAGED` or generated files, update the owning TD/source unit and
  regenerate or run the matching code-check instead of hand-editing only the
  projected output.
- Keep project-specific agent behavior in this file, `CAPABILITIES.md`, scoped
  convention docs, skills/templates, or command output. Do not add live
  project-layer `CLAUDE.md` or `AGENTS.md`.

## Verification

Before claiming an Agentic Workflow change is complete, run the narrowest
checks that prove the changed surface:

- Formatting/build: `cargo fmt --check -p agentic-workflow` and
  `cargo check -p agentic-workflow` for Rust changes.
- Focused behavior: the smallest relevant `cargo test -p agentic-workflow ...`
  target for the changed command, parser, validator, or doc contract.
- Generated-code closure: `aw td code-check <changed-source> --project
  agentic-workflow` when changing SPEC-MANAGED source or its projection.
- Meta-doc changes: run or update the root doc allowlist/mirror tests when
  changing root/project meta-doc policy, README shells, or agent-doc placement.

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
