# service-durability

## Brief

`service-durability` defines shared durable local storage primitives for axiom
services: fsync policy, temp-file atomic replacement, CRC-framed append logs, and
sequence-named local snapshot stores. Services keep their own domain codecs and
state-machine semantics.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Service Durability Contract | - | implemented | verified | smoke | ready | fsync policy, atomic writes, framed logs, and snapshot files |

### Shared Service Durability Contract

ID: shared-service-durability-contract
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `service_durability`.
EC Dimensions: behavior: `cargo test -p service-durability` - durable file primitive coverage
Required Verification: smoke
Promise:
Services can compose one shared durable local storage layer instead of
reimplementing fsync, atomic rename, append-log frame parsing, torn-tail
recovery, or sequence-named snapshot retention locally.
Gate Inventory: `cargo test -p service-durability`; libs/service-durability/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-service-durability-contract | epic | - | implemented | verified | smoke | `cargo test -p service-durability`; libs/service-durability/src/lib.rs |
