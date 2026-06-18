# Cclab Log

## Brief

Cclab Log is the Rust structured-logging API surface for cclab crates.

It owns the logger facade, context binding, level filtering, sink contract,
and logging error types. File rotation and network delivery are still
implementation gaps, so this README records those surfaces as partial
contracts rather than production-ready logging behavior.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Structured Logger API | - | partial | passing | smoke | not_ready | Logger API with context binding, level filtering, and sink fan-out |
| Sink Contract And Configuration | - | partial | passing | smoke | not_ready | Sink trait and console/file/network configuration; file and network writes are stubs |
| Log Error Contract | - | implemented | passing | smoke | not_ready | shared logging error enum and result alias |

### Structured Logger API

ID: structured-logger-api
Type: DeveloperTool
Surfaces: Rust API: `cclab_log::Logger`, `BoundLogger`, level methods, context binding
EC Dimensions: behavior: `cargo test -p cclab-log` - compile smoke for logger API surface; behavior tests still needed
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Log exposes a structured logger API with bound context, level filtering, convenience level methods, and fan-out to registered sinks.
Gate Inventory: `cargo test -p cclab-log`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Logger API compile contract | epic | - | partial | passing | smoke | `cargo test -p cclab-log` |

### Sink Contract And Configuration

ID: sink-contract-and-configuration
Type: DeveloperTool
Surfaces: Rust API: `Sink`, `LogRecord`, `LogLevel`, `ConsoleSink`, `FileSink`, `NetworkSink`, `SinkConfig`
EC Dimensions: behavior: `cargo test -p cclab-log` - compile smoke for sink contract; file/network behavior tests still needed
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Log defines the sink contract and configuration types for console, file, and network logging; console formatting is implemented, while file rotation and network delivery remain implementation gaps.
Gate Inventory: `cargo test -p cclab-log`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Sink contract compile smoke | epic | - | partial | passing | smoke | `cargo test -p cclab-log` |

### Log Error Contract

ID: log-error-contract
Type: RuntimeTool
Surfaces: Rust API: `LogError`, `Result`
EC Dimensions: behavior: `cargo test -p cclab-log` - compile smoke for error contract
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Log provides a shared logging error contract for invalid levels, sink failures, format failures, rotation failures, and I/O errors.
Gate Inventory: `cargo test -p cclab-log`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Log error type compile contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-log` |
