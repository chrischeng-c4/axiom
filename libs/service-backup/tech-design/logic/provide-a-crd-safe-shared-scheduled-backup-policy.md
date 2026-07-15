---
id: '1778'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-crd-safe-backup-policy-flow
entry: decode
nodes:
  decode: { kind: start, label: Kubernetes decodes flat schedule destination and retentionSecs fields }
  app_fields: { kind: process, label: Lumen and Relay flatten ScheduledBackupPolicy beside app-specific adminTokenSecret; Keep aliases it directly }
  validate_schedule: { kind: decision, label: Is the schedule non-empty after trimming }
  reject_schedule: { kind: terminal, label: Reject empty schedule through shared conversion }
  parse_destination: { kind: decision, label: BackupDestination from_uri accepts the destination URI }
  reject_destination: { kind: terminal, label: Return the canonical shared URI validation error }
  retention: { kind: process, label: Map retentionSecs unchanged into RetentionPolicy max_age_seconds }
  runtime: { kind: terminal, label: Produce BackupPolicy with unchanged runtime semantics }
edges:
  - { from: decode, to: app_fields }
  - { from: app_fields, to: validate_schedule }
  - { from: validate_schedule, to: reject_schedule, label: no }
  - { from: validate_schedule, to: parse_destination, label: yes }
  - { from: parse_destination, to: reject_destination, label: invalid }
  - { from: parse_destination, to: retention, label: valid }
  - { from: retention, to: runtime }
---
flowchart TD
  decode([Decode flat CRD fields]) --> app_fields[Compose shared policy with app secret fields]
  app_fields --> validate_schedule{Schedule non-empty?}
  validate_schedule -->|no| reject_schedule([Reject])
  validate_schedule -->|yes| parse_destination{Destination URI valid?}
  parse_destination -->|no| reject_destination([Canonical URI error])
  parse_destination -->|yes| retention[Map retentionSecs]
  retention --> runtime([Runtime BackupPolicy])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - { path: libs/service-backup/tech-design/semantic/source/libs-service-backup-src-policy-rs.md, action: modify, section: logic, impl_mode: hand-written, description: Define ScheduledBackupPolicy and validated runtime conversion in the canonical source unit. }
  - { path: libs/service-backup/tech-design/semantic/source/libs-service-backup-src-lib-rs.md, action: modify, section: logic, impl_mode: hand-written, description: Export the shared CRD-safe policy. }
  - { path: libs/service-backup/README.md, action: modify, section: logic, impl_mode: hand-written, description: Document CRD and runtime policy ownership. }
  - { path: CONTRIBUTING.md, action: modify, section: logic, impl_mode: hand-written, description: Record the shared service-kit boundary. }
  - { path: apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-crd-rs.md, action: modify, section: logic, impl_mode: hand-written, description: Flatten the shared policy beside Lumen token-secret fields. }
  - { path: apps/lumen/tests/operator_render.rs, action: modify, section: unit-test, impl_mode: hand-written, description: Prove serialization schema and operator rendering compatibility. }
  - { path: apps/keep/src/operator/crd.rs, action: modify, section: logic, impl_mode: hand-written, description: Replace Keep's duplicate DTO with the shared type. }
  - { path: apps/keep/tests/operator.rs, action: modify, section: unit-test, impl_mode: hand-written, description: Prove Keep CRD and rendering compatibility. }
  - { path: apps/relay/src/operator/crd.rs, action: modify, section: logic, impl_mode: hand-written, description: Flatten the shared policy beside Relay token-secret fields. }
  - { path: apps/relay/tests/operator.rs, action: modify, section: unit-test, impl_mode: hand-written, description: Prove Relay CRD and rendering compatibility. }
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: shared-crd-safe-backup-policy-verification
requirements:
  crd_compatibility:
    id: R3
    text: "Lumen, Keep, and Relay generated CRDs retain flat schedule, destination, retentionSecs, and app-specific adminTokenSecret fields without tagged destination unions."
    kind: regression
    risk: high
    verify: cargo test -p lumen --test operator_render; cargo test -p keep --test operator; cargo test -p relay --test operator
  render_compatibility:
    id: R4
    text: "All three operators render the same CronJob schedule, destination, retention, and token-secret wiring as before adoption."
    kind: regression
    risk: medium
    verify: cargo test -p lumen --test operator_render; cargo test -p keep --test operator; cargo test -p relay --test operator
  shared_projection:
    id: R1
    text: "ScheduledBackupPolicy serializes as the flat schedule, destination, and retentionSecs contract and exposes a structural-schema-safe JsonSchema."
    kind: functional
    risk: high
    verify: cargo test -p service-backup
  validated_conversion:
    id: R2
    text: "One shared conversion rejects blank schedules and invalid destination URIs while preserving valid retention semantics."
    kind: functional
    risk: high
    verify: cargo test -p service-backup
---
flowchart TD
    r1[R1 shared projection] --> cargo_test_p_service_backup[cargo test -p service-backup]
    r2[R2 validated conversion] --> cargo_test_p_service_backup
    r3[R3 crd compatibility] --> cargo_test_p_lumen_test_operator_render_cargo_test_p_keep_test_operator_cargo_test_p_relay_test_operator[cargo test -p lumen --test operator_render; cargo test -p keep --test operator; cargo test -p relay --test operator]
    r4[R4 render compatibility] --> cargo_test_p_lumen_test_operator_render_cargo_test_p_keep_test_operator_cargo_test_p_relay_test_operator
```
