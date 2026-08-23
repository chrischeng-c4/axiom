# Cclab Log Mamba

## Brief

Cclab Log Mamba is the Mamba native binding for the `cclab-log` structured
logging API.

It registers the `cclab.log` module through the shared Mamba registry and
exposes native-call entrypoints for `get_logger`, `info`, `error`, `debug`, and
`warning`. The binding owns the Mamba value conversion boundary, logger handle
fallback behavior, and JSON-shaped log records emitted by the native methods.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Mamba Structured Logging Binding | - | exposes `cclab.log` logger creation plus info/error/debug/warning native methods |

### Mamba Structured Logging Binding

Cclab Log Mamba exposes `cclab-log` structured logging to Mamba scripts through
the `cclab.log` native module, including logger creation and
info/error/debug/warning methods that accept Mamba string values and emit
JSON-structured log records.

- Root WI: none; this capability predates the tracker.
- Surfaces: Mamba module: `cclab.log`; Native ABI: `mb_log_get_logger`,
  `mb_log_info`, `mb_log_error`, `mb_log_debug`, `mb_log_warning`; Rust module
  registrar: `LogMambaModule`
- Gate — behavior: `cargo test -p cclab-log-mamba`
- Gate: `cargo test -p cclab-log-mamba`
- Source: `crates/cclab-log-mamba/src/lib.rs`,
  `crates/cclab-log-mamba/src/methods.rs`,
  `crates/cclab-log-mamba/tests/methods_test.rs`
- Evidence: `cargo test -p cclab-log-mamba`; crates/cclab-log-mamba/src/lib.rs;
  crates/cclab-log-mamba/src/methods.rs;
  crates/cclab-log-mamba/tests/methods_test.rs
