# Cclab Mamba Registry

## Brief

Cclab Mamba Registry is the Rust registry and ABI bridge for native Mamba
modules.

It lets binding crates self-register importable Mamba modules, share the
`MbValue` conversion boundary, and call runtime-provided object, exception,
HTTP status, and async helpers without depending on private Mamba layouts.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Native Module Auto Registration | - | implemented | passing | smoke | not_ready | link-time registration surface for native Mamba modules |
| MbValue Conversion ABI | - | implemented | passing | conformance | not_ready | NaN-boxed Mamba value ABI plus Rust conversion traits and native handles |
| Runtime Bridge Helpers | - | implemented | passing | smoke | not_ready | object ops, exception helpers, HTTP status table, and shared tokio runtime bridge |

### Native Module Auto Registration

ID: native-module-auto-registration
Type: RuntimeTool
Surfaces: Rust API: `MambaModule`, `ModuleRegistrar`, `RuntimeSymbol`, `RuntimeValue`, `MAMBA_MODULES`, `rt_sym!`, `all_modules`, `find_module`
EC Dimensions: behavior: `cargo test -p cclab-mamba-registry` - module slice, registrar, symbol, and integration smoke behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Mamba Registry lets Rust binding crates self-register native Mamba modules, exported symbols, and module values at link time so the Mamba runtime can discover importable native modules without a hand-maintained central table.
Gate Inventory: `cargo test -p cclab-mamba-registry`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Native module registration contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-mamba-registry` |

### MbValue Conversion ABI

ID: mbvalue-conversion-abi
Type: RuntimeTool
Surfaces: Rust API: `MbValue`, `FromMbValue`, `IntoMbValue`, `MbConvError`, native wrapping helpers
EC Dimensions: behavior: `cargo test -p cclab-mamba-registry` - scalar roundtrip, conversion, overflow, collection, option, and native handle behavior
Root WI: -
Status: confirmed
Required Verification: smoke, conformance
Promise:
Cclab Mamba Registry provides the shared `MbValue` ABI and conversion traits that let Rust binding crates move primitive values, collections, optional values, strings, and opaque native handles across the Mamba runtime boundary.
Gate Inventory: `cargo test -p cclab-mamba-registry`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| MbValue scalar and conversion contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-mamba-registry` |

### Runtime Bridge Helpers

ID: runtime-bridge-helpers
Type: RuntimeTool
Surfaces: Rust API: `ObjectOps`, `set_object_ops`, `ops`, `raise_*`, `http::status_phrase`, `runtime::handle`
EC Dimensions: behavior: `cargo test -p cclab-mamba-registry` - ops table, exception, HTTP status, and runtime helper behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Mamba Registry gives binding crates a shared runtime bridge for object operations, exception propagation, canonical HTTP status metadata, and process-wide async execution without depending on private Mamba runtime layouts.
Gate Inventory: `cargo test -p cclab-mamba-registry`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Runtime bridge helper contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-mamba-registry` |
