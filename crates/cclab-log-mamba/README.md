# Cclab Log Mamba

## Brief

Cclab Log Mamba is the Mamba native binding for the `cclab-log` structured
logging API.

It registers the `cclab.log` module through the shared Mamba registry and
exposes native-call entrypoints for `get_logger`, `info`, `error`, `debug`, and
`warning`. The binding owns the Mamba value conversion boundary, logger handle
fallback behavior, and JSON-shaped log records emitted by the native methods.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Mamba Structured Logging Binding | - | implemented | passing | conformance | not_ready | exposes `cclab.log` logger creation plus info/error/debug/warning native methods |

### Mamba Structured Logging Binding

ID: mamba-structured-logging-binding
Type: DeveloperTool
Surfaces: Mamba module: `cclab.log`; Native ABI: `mb_log_get_logger`, `mb_log_info`, `mb_log_error`, `mb_log_debug`, `mb_log_warning`; Rust module registrar: `LogMambaModule`
EC Dimensions: behavior: `cargo test -p cclab-log-mamba`
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Log Mamba exposes `cclab-log` structured logging to Mamba scripts through the `cclab.log` native module, including logger creation and info/error/debug/warning methods that accept Mamba string values and emit JSON-structured log records.
Gate Inventory: `cargo test -p cclab-log-mamba`; crates/cclab-log-mamba/src/lib.rs; crates/cclab-log-mamba/src/methods.rs; crates/cclab-log-mamba/tests/methods_test.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Mamba structured logging ABI contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-log-mamba`; crates/cclab-log-mamba/src/lib.rs; crates/cclab-log-mamba/src/methods.rs; crates/cclab-log-mamba/tests/methods_test.rs |
