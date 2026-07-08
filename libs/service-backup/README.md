# service-backup

## Brief

`service-backup` defines the shared backup contract for axiom services:
destination and policy schema, sink trait, local sink, source trait, and runner
primitive.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Service Backup Contract | - | implemented | verified | smoke | ready | backup destination, policy, source, sink, and runner primitives |

### Shared Service Backup Contract

ID: shared-service-backup-contract
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `service_backup`.
EC Dimensions: behavior: `cargo test -p service-backup` - backup policy, sink, source, and runner coverage
Required Verification: smoke
Promise:
Services can produce consistent snapshots and let backup runners upload them
through a shared contract.
Gate Inventory: `cargo test -p service-backup`; libs/service-backup/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-service-backup-contract | epic | - | implemented | verified | smoke | `cargo test -p service-backup`; libs/service-backup/src/lib.rs |
