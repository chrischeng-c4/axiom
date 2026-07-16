# service-backup

## Brief

`service-backup` defines the shared backup contract for axiom services:
runtime destination/policy schema, a flat CRD-safe scheduled policy, sink and
source traits, local/cloud adapters, and runner primitives.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Service Backup Contract | - | implemented | verified | smoke | ready | runtime + CRD-safe policy, destination, source, sink, and runner primitives |

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
through a shared contract. Kubernetes operators reuse `ScheduledBackupPolicy`
for flat `schedule`, `destination`, and `retentionSecs` fields, then use its
validated conversion to obtain the tagged runtime `BackupPolicy`.
Gate Inventory: `cargo test -p service-backup`; libs/service-backup/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-service-backup-contract | epic | - | implemented | verified | smoke | `cargo test -p service-backup`; libs/service-backup/src/lib.rs |
| crd-safe-scheduled-backup-policy | change | #1778 | implemented | passing | conformance | `cargo test -p service-backup`; operator schema/render suites for Lumen, Keep, and Relay |
