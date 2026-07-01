---
id: keep-service-backup-adoption
summary: >
  Keep produces consistent snapshot bytes from its recovered engine state and
  wires backup through the shared libs/service-backup contract: a `keep backup`
  verb that runs run_backup_once to a destination, plus an operator-rendered
  backup CronJob emitted only when a KeepSpec backup policy is configured.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-service-backup-adoption-contract
entry: backup_cmd
nodes:
  backup_cmd: { kind: start, label: "keep backup parses --dest/--data-dir/--shards/--retention-secs" }
  recover: { kind: process, label: "RecoveryManager::recover(data_dir, shards) rebuilds engine from snapshot + WAL" }
  snapshot_bytes: { kind: process, label: "KeepSnapshot::from_engine dumps values; snapshot_bytes serializes JSON payload" }
  parse_dest: { kind: process, label: "BackupDestination::from_uri maps file/s3/gs URI" }
  sink: { kind: process, label: "sink_from_destination picks LocalFsSink or fail-loud UnsupportedCloudSink" }
  run_once: { kind: process, label: "run_backup_once writes payload then prunes per RetentionPolicy" }
  print: { kind: terminal, label: "print BackupRunResult (sink/key/bytes/pruned)" }
  operator_render: { kind: start, label: "operator render(keep) inspects spec.backup" }
  cron: { kind: process, label: "cron_job emits CronJob invoking keep backup with policy schedule/dest/retention" }
  omit: { kind: terminal, label: "no backup policy -> no CronJob rendered" }
edges:
  - { from: backup_cmd, to: recover }
  - { from: recover, to: snapshot_bytes }
  - { from: snapshot_bytes, to: parse_dest }
  - { from: parse_dest, to: sink }
  - { from: sink, to: run_once }
  - { from: run_once, to: print }
  - { from: operator_render, to: cron }
  - { from: operator_render, to: omit }
---
flowchart TD
    backup_cmd([keep backup parses dest data-dir shards retention-secs]) --> recover[RecoveryManager recover rebuilds engine from snapshot and WAL]
    recover --> snapshot_bytes[KeepSnapshot from_engine dumps values snapshot_bytes serializes JSON payload]
    snapshot_bytes --> parse_dest[BackupDestination from_uri maps file s3 gs URI]
    parse_dest --> sink[sink_from_destination picks LocalFsSink or fail-loud UnsupportedCloudSink]
    sink --> run_once[run_backup_once writes payload then prunes per RetentionPolicy]
    run_once --> print([print BackupRunResult sink key bytes pruned])
    operator_render([operator render inspects spec backup]) --> cron[cron_job emits CronJob invoking keep backup with policy schedule dest retention]
    operator_render --> omit([no backup policy no CronJob rendered])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-service-backup-adoption-tests
requirements:
  consistent_snapshot_bytes:
    id: R1
    text: "Keep produces consistent snapshot bytes from its engine state via dump_values; snapshot_bytes round-trips the live key/value set."
    kind: behavior
    risk: medium
    verify: test
  local_backup_runner:
    id: R2
    text: "keep backup adopts service_backup and writes a snapshot artifact to a file:// destination via run_backup_once, applying RetentionPolicy pruning."
    kind: behavior
    risk: medium
    verify: test
  operator_backup_cronjob:
    id: R3
    text: "The operator renders a backup CronJob invoking keep backup when spec.backup is configured, and omits it otherwise."
    kind: behavior
    risk: medium
    verify: test
elements:
  keep_backup_tests:
    kind: test
    path: projects/keep/tests/backup.rs
  keep_operator_tests:
    kind: test
    path: projects/keep/tests/operator.rs
relations:
  - { from: keep_backup_tests, verifies: consistent_snapshot_bytes }
  - { from: keep_backup_tests, verifies: local_backup_runner }
  - { from: keep_operator_tests, verifies: operator_backup_cronjob }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "consistent snapshot bytes"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "local backup runner"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "operator backup cronjob"
      risk: medium
      verifymethod: test
    }
    element keep_backup_tests {
      type: "rs/#[test]"
    }
    element keep_operator_tests {
      type: "rs/#[test]"
    }
    keep_backup_tests - verifies -> R1
    keep_backup_tests - verifies -> R2
    keep_operator_tests - verifies -> R3
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the service-backup path dependency (always linked; used by the backup module and, behind the operator feature, by KeepSpec)."
  - path: projects/keep/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Declare and export the new `backup` module."
  - path: projects/keep/src/backup.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "keep's backup adoption: re-export service_backup types (mirrors lumen's backup_sink), a KeepSnapshot payload built from engine.dump_values, snapshot_bytes/snapshot_bytes_from_data_dir, and run_backup that recovers the engine, serializes a consistent snapshot, and calls sink_from_destination + run_backup_once."
  - path: projects/keep/src/bin/keep.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the `keep backup` verb (--dest, --data-dir, --shards, --retention-secs) that builds a BackupDestination, runs run_backup, and prints the BackupRunResult."
  - path: projects/keep/src/operator/crd.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add an optional `backup: Option<service_backup::BackupPolicy>` field to KeepSpec (schedule + destination + retention); the crd_yaml uint normalization already covers the retention maxAgeSeconds u64."
  - path: projects/keep/src/operator/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Render a backup CronJob via operator::render::cron_job when spec.backup is Some, invoking `keep backup` with the policy's schedule, destination URI, and retention; no CronJob otherwise."
  - path: projects/keep/tests/backup.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "R1/R2: snapshot_bytes round-trips the engine's key/value set, and run_backup_once writes then prunes an artifact at a file:// destination."
  - path: projects/keep/tests/operator.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "R3: render emits a backup CronJob invoking `keep backup` when spec.backup is set and omits it otherwise; update the spec() helper for the new field."
```
