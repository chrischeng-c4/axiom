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

## CLI Verb Lifecycle and Removal Gate

Every registered `aw` CLI verb carries a lifecycle class in
`src/cli/chain.rs`'s `VERB_LIFECYCLE_REGISTRY` (epic #1270 R4+R9):

- `Core` — the `wi`/`td`/`ec`/`capability`/`health`/`conf` lifecycle and loop
  surface: the verbs the LINEAR loop (`aw wi` -> `aw td create` -> `gen` ->
  `fill` -> `code-check`) and its sibling `ec`/`capability`/`health` loops
  actually dispatch through.
- `Utility` — support tooling that is not itself a lifecycle-loop step: the
  CLI-convention trio (`llm`/`upgrade`/`issue`), `chat`/`guard`/`view`/`new`/
  `report-issue`/`generator`, `standardize audit`, and the read-only/debug
  `td` verbs (`ast`, `check`, `lock`, `gen-source`, `promote`).
- `Migration` — scheduled for removal or fold-in once a stated condition
  holds. Every `Migration` entry MUST carry a non-empty `sunset_criterion`
  naming that condition (for example `td code-claim`: "folded into `td
  create --from-source` per epic #1270 R5 / #1273").

`cargo test -p agentic-workflow chain::tests::leaf_verb_paths_are_all_classified`
enumerates the real registered clap tree and fails if any verb lacks a class
entry, any entry references a verb that no longer exists, or any
`Migration`-class verb has an empty `sunset_criterion`.

**Removal precondition gate**: a verb may only be removed once (1) chain
conformance is green — `EMIT_REGISTRY` and `validate_aw_command_string` in
`src/cli/chain.rs` prove no emitted `next.command`/`invoke.command` string
still dangles on the verb being removed — and (2) `VERB_LIFECYCLE_REGISTRY`
and its conformance test are updated in the same change to drop the removed
verb's entry. Do not remove a verb and leave a stale registry entry, and do
not add a `Migration`-class verb without a concrete `sunset_criterion`.

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
