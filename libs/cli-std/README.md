# cli-std

## Brief

`cli-std` provides the standard agent-facing command implementations shared by
axiom CLIs: `llm`, `upgrade`, and `issue`, plus chainable output helpers.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Standard Agent CLI Commands | - | shared llm, upgrade, issue, and chainable output APIs |

### Standard Agent CLI Commands

Projects can expose consistent agent-facing CLI commands without duplicating
GitHub issue, self-update, or LLM orientation logic.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `cli_std::{llm, upgrade, issue, chainable}`.
- Gate — behavior: `cargo test -p cli-std` - shared CLI command contract
  coverage
- Gate: `cargo test -p cli-std`
- Source: `libs/cli-std/src/lib.rs`
- Evidence: `cargo test -p cli-std`; libs/cli-std/src/lib.rs
