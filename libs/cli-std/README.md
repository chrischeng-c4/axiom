# cli-std

## Brief

`cli-std` provides the standard agent-facing command implementations shared by
axiom CLIs: `llm`, `upgrade`, and `issue`, plus chainable output helpers.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Standard Agent CLI Commands | - | implemented | verified | smoke | ready | shared llm, upgrade, issue, and chainable output APIs |

### Standard Agent CLI Commands

ID: standard-agent-cli-commands
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `cli_std::{llm, upgrade, issue, chainable}`.
EC Dimensions: behavior: `cargo test -p cli-std` - shared CLI command contract coverage
Required Verification: smoke
Promise:
Projects can expose consistent agent-facing CLI commands without duplicating
GitHub issue, self-update, or LLM orientation logic.
Gate Inventory: `cargo test -p cli-std`; libs/cli-std/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| standard-agent-cli-commands-contract | epic | - | implemented | verified | smoke | `cargo test -p cli-std`; libs/cli-std/src/lib.rs |
