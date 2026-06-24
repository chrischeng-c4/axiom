# Cclab Cli Registry

## Brief

Cclab CLI Registry is the shared Rust registry layer for auto-registering
ecosystem CLI subcommands.

It gives CLI crates a `CliModule` trait plus a `linkme` distributed slice so
the main binary can discover command definitions and dispatch implementations
without hand-maintaining a central command table.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| CLI Module Auto Registration | - | implemented | passing | smoke | not_ready | link-time CLI module registration trait and registry inventory |

### CLI Module Auto Registration

ID: cli-module-auto-registration
Type: DeveloperTool
Surfaces: Rust API: `CliModule`, `CLI_MODULES`, `find_module`, `registered_names`
EC Dimensions: behavior: `cargo test -p cclab-cli-registry` - module registry access and name inventory behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab CLI Registry lets Rust crates self-register CLI modules through a shared `CliModule` trait and `linkme` distributed slice so the main CLI can discover command definitions and dispatch implementations without hand-maintaining a central command table.
Gate Inventory: `cargo test -p cclab-cli-registry`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CLI registry lookup contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-cli-registry` |
