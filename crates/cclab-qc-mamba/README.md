# Cclab Qc Mamba

## Brief

Cclab QC Mamba is the Mamba native binding for pytest-like test decorator
metadata.

It registers the `cclab.qc` module through the shared Mamba registry and
exposes native-call entrypoints for fixture metadata, a mark namespace sentinel,
raises context metadata, and parametrize metadata. The crate owns the Mamba ABI
surface for those primitives; downstream runner discovery/execution consumes
the metadata outside this binding.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Mamba QC Decorator Binding | - | implemented | passing | conformance | not_ready | exposes pytest-like fixture, mark, raises, and parametrize metadata primitives for Mamba |

### Mamba QC Decorator Binding

ID: mamba-qc-decorator-binding
Type: DeveloperTool
Surfaces: Mamba module: `cclab.qc`; Native ABI: `mb_qc_fixture`, `mb_qc_mark`, `mb_qc_raises`, `mb_qc_parametrize`; Rust module registrar: `QcMambaModule`
EC Dimensions: behavior: `cargo test -p cclab-qc-mamba`
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab QC Mamba exposes pytest-like test decorator primitives to Mamba scripts through the `cclab.qc` native module, including fixture metadata, a mark namespace sentinel, raises context metadata, and parametrize metadata for the downstream test runner to consume.
Gate Inventory: `cargo test -p cclab-qc-mamba`; crates/cclab-qc-mamba/src/lib.rs; crates/cclab-qc-mamba/src/methods.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Mamba QC decorator metadata ABI contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-qc-mamba`; crates/cclab-qc-mamba/src/lib.rs; crates/cclab-qc-mamba/src/methods.rs |
