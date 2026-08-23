# service-backup

## Brief

`service-backup` defines the shared backup contract for axiom services:
runtime destination/policy schema, a flat CRD-safe scheduled policy, sink and
source traits, local/cloud adapters, and runner primitives.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Shared Service Backup Contract | - | runtime + CRD-safe policy, destination, source, sink, and runner primitives |

### Shared Service Backup Contract

Services can produce consistent snapshots and let backup runners upload them
through a shared contract. Kubernetes operators reuse `ScheduledBackupPolicy`
for flat `schedule`, `destination`, and `retentionSecs` fields, then use its
validated conversion to obtain the tagged runtime `BackupPolicy`.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `service_backup`.
- Gate — behavior: `cargo test -p service-backup` - backup policy, sink,
  source, and runner coverage
- Gate: `cargo test -p service-backup`
- Source: `libs/service-backup/src/lib.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| shared-service-backup-contract | epic | - | `cargo test -p service-backup`; libs/service-backup/src/lib.rs |
| crd-safe-scheduled-backup-policy | change | #1778 | `cargo test -p service-backup`; operator schema/render suites for Lumen, Keep, and Relay |
