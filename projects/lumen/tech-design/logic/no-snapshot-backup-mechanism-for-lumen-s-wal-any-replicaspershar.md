---
id: serving-backup-cronjob-and-admin-backup-docs
summary: >
  Document lumen's already-existing /admin/backup, /admin/backup/local, and
  /admin/restore admin API as the safe manual snapshot-restore procedure in
  `lumen llm storage`, and add an optional `spec.serving.backup` CRD field
  that makes the operator render a `<name>-backup` CronJob invoking a new
  `lumen backup` CLI verb, which hits the serving Service's own
  /admin/backup over the network and ships the bytes to the configured
  destination via `service_backup::run_backup_once`.
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "lumen-crd-reconcile-loop-kube-rs-operator"
    coverage: partial
    rationale: >
      Issue #808 (re-scoped after #812 landed the unconditional PVC) closes
      the remaining durability gap: the already-tested admin backup/restore
      surface was invisible to deployers and had no operator/CRD automation
      to schedule it, leaving `lumen llm storage`'s backup guidance as a dead
      forward-reference and the long-running-stability capability's
      Gate Inventory evidence (`backup_restore_e2e.rs`) undiscoverable and
      unscheduled.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: serving-backup-cronjob-and-backup-cli
entry: render_start
nodes:
  render_start: { kind: start,    label: "render(lumen) — after #812's unconditional StatefulSet+PVC objects" }
  has_backup:   { kind: decision, label: "spec.serving.backup set?" }
  no_cronjob:   { kind: process,  label: "no CronJob rendered — lumen llm storage documents the manual /admin/backup procedure" }
  build_cronjob: { kind: process, label: "backup_cron_job(): batch/v1 CronJob '<name>-backup' via shared operator::render::cron_job, schedule=policy.schedule, args=[backup, --url, http://<name>.<ns>.svc.cluster.local:7373, --dest, policy.destination, --retention-secs?]" }
  token_env:    { kind: decision, label: "adminTokenSecret set?" }
  with_token:   { kind: process,  label: "add container env LUMEN_BACKUP_TOKEN <- secretKeyRef(adminTokenSecret, key=token)" }
  render_emit:  { kind: terminal, label: "render() returns full object set (+ optional backup CronJob)" }
  cron_fire:    { kind: process,  label: "Kubernetes CronJob fires a Job on schedule" }
  cli_start:    { kind: start,    label: "lumen backup --url <base> --dest <uri> [--token] [--retention-secs]" }
  fetch:        { kind: process,  label: "GET {url}/admin/backup (Bearer token if set) -> live SnapshotV1 bytes via Engine::snapshot(), the same call raft's own snapshotter uses — no separate quiesce/flush step" }
  ship:         { kind: process,  label: "service_backup::run_backup_once(sink_from_destination(dest), now, bytes, retention) -> BackupRunResult" }
  cli_emit:     { kind: terminal, label: "print BackupRunResult JSON; Job succeeds/fails on the HTTP + sink result" }
edges:
  - { from: render_start,  to: has_backup }
  - { from: has_backup,    to: no_cronjob,    label: "no" }
  - { from: has_backup,    to: build_cronjob, label: "yes" }
  - { from: build_cronjob, to: token_env }
  - { from: token_env,     to: with_token,    label: "yes" }
  - { from: token_env,     to: render_emit,   label: "no" }
  - { from: with_token,    to: render_emit }
  - { from: no_cronjob,    to: render_emit }
  - { from: render_emit,   to: cron_fire,     label: "only when the CronJob was rendered" }
  - { from: cron_fire,     to: cli_start }
  - { from: cli_start,     to: fetch }
  - { from: fetch,         to: ship }
  - { from: ship,          to: cli_emit }
---
flowchart TD
    render_start([render lumen serving fleet, post-#812]) --> has_backup{spec.serving.backup set?}
    has_backup -->|no| no_cronjob[no CronJob; lumen llm storage documents manual /admin/backup procedure]
    has_backup -->|yes| build_cronjob["backup_cron_job(): CronJob '<name>-backup' via operator::render::cron_job(schedule, args=[backup --url http://<name>.<ns>.svc:7373 --dest <destination> [--retention-secs]])"]
    build_cronjob --> token_env{adminTokenSecret set?}
    token_env -->|yes| with_token[env LUMEN_BACKUP_TOKEN <- secretKeyRef]
    token_env -->|no| render_emit([render emits full object set + optional CronJob])
    with_token --> render_emit
    no_cronjob --> render_emit
    render_emit --> cron_fire[k8s CronJob fires a Job on schedule]
    cron_fire --> cli_start([lumen backup --url --dest --token? --retention-secs?])
    cli_start --> fetch[GET admin/backup: live SnapshotV1 via Engine::snapshot, no separate quiesce/flush]
    fetch --> ship[service_backup::run_backup_once against resolved sink + retention]
    ship --> cli_emit([print BackupRunResult JSON; Job succeeds/fails accordingly])
```
