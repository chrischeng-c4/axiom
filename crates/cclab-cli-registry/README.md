# Cclab Cli Registry

## Brief

Cclab CLI Registry is the shared Rust registry layer for auto-registering
ecosystem CLI subcommands.

It gives CLI crates a `CliModule` trait plus a `linkme` distributed slice so
the main binary can discover command definitions and dispatch implementations
without hand-maintaining a central command table.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| CLI Module Auto Registration | - | link-time CLI module registration trait and registry inventory |

### CLI Module Auto Registration

Cclab CLI Registry lets Rust crates self-register CLI modules through a shared
`CliModule` trait and `linkme` distributed slice so the main CLI can discover
command definitions and dispatch implementations without hand-maintaining a
central command table.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `CliModule`, `CLI_MODULES`, `find_module`,
  `registered_names`
- Gate — behavior: `cargo test -p cclab-cli-registry` - module registry access
  and name inventory behavior
- Gate: `cargo test -p cclab-cli-registry`
- Evidence: `cargo test -p cclab-cli-registry`
