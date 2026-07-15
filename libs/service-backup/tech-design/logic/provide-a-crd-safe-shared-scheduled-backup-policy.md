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
  app_fields: { kind: process, label: Lumen and Relay flatten the shared policy beside app-specific adminTokenSecret; Keep uses it directly }
  validate_schedule: { kind: decision, label: Is the schedule non-empty after trimming }
  reject_schedule: { kind: terminal, label: Reject empty schedule through shared conversion }
  parse_destination: { kind: decision, label: BackupDestination from_uri accepts the destination URI }
  reject_destination: { kind: terminal, label: Return the canonical shared URI validation error }
  runtime: { kind: terminal, label: Produce BackupPolicy with RetentionPolicy and unchanged runtime semantics }
edges:
  - { from: decode, to: app_fields }
  - { from: app_fields, to: validate_schedule }
  - { from: validate_schedule, to: reject_schedule, label: no }
  - { from: validate_schedule, to: parse_destination, label: yes }
  - { from: parse_destination, to: reject_destination, label: invalid }
  - { from: parse_destination, to: runtime, label: valid }
---
flowchart TD
  decode([Decode flat CRD fields]) --> app_fields[Compose app-specific secret fields]
  app_fields --> validate_schedule{Schedule non-empty?}
  validate_schedule -->|no| reject_schedule([Reject])
  validate_schedule -->|yes| parse_destination{Destination URI valid?}
  parse_destination -->|no| reject_destination([Canonical URI error])
  parse_destination -->|yes| runtime([Runtime BackupPolicy])
```
