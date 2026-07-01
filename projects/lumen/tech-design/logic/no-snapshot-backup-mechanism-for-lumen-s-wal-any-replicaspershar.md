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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: serving-backup-cronjob-and-admin-backup-docs-tests
requirements:
  llm_storage_documents_admin_backup:
    id: R1
    text: "lumen llm storage documents /admin/backup, /admin/backup/local, and /admin/restore (routes, admin-role auth requirement, and that Engine::snapshot() is the same quiesce-free call the raft snapshotter itself uses) as the manual/scriptable snapshot-restore procedure."
    kind: doc
    risk: low
    verify: test
  no_backup_field_renders_no_cronjob:
    id: R2
    text: "When spec.serving.backup is None, render() emits no CronJob object for the serving fleet."
    kind: behavior
    risk: low
    verify: test
  backup_field_renders_cronjob:
    id: R3
    text: "When spec.serving.backup is set, render() emits exactly one batch/v1 CronJob named '<name>-backup' with the configured schedule and a container invoking `lumen backup --url http://<name>.<ns>.svc.cluster.local:7373 --dest <destination>`."
    kind: behavior
    risk: medium
    verify: test
  retention_and_token_wired_on_cronjob:
    id: R4
    text: "retentionSecs (when set) appears as --retention-secs on the CronJob container args, and adminTokenSecret (when set) appears as a LUMEN_BACKUP_TOKEN env var sourced via secretKeyRef."
    kind: behavior
    risk: medium
    verify: test
  backup_cli_round_trip:
    id: R5
    text: "lumen backup --url <base> --dest file://<dir> fetches /admin/backup from a running server and writes a backup object into the destination sink via service_backup::run_backup_once, returning a BackupRunResult."
    kind: behavior
    risk: medium
    verify: test
  backup_feature_gated_off_default_build:
    id: R6
    text: "cargo build -p lumen with no features still compiles with no reqwest/HTTP client linked; the `backup` feature (pulled in transitively by `operator`) is required to enable the `lumen backup` verb."
    kind: behavior
    risk: low
    verify: inspection
elements:
  spec_cli_unit_tests:
    kind: test
    path: projects/lumen/tests/spec_cli.rs
  operator_render_unit_tests:
    kind: test
    path: projects/lumen/tests/operator_render.rs
  backup_unit_tests:
    kind: test
    path: projects/lumen/src/backup.rs
relations:
  - { from: spec_cli_unit_tests, verifies: llm_storage_documents_admin_backup }
  - { from: operator_render_unit_tests, verifies: no_backup_field_renders_no_cronjob }
  - { from: operator_render_unit_tests, verifies: backup_field_renders_cronjob }
  - { from: operator_render_unit_tests, verifies: retention_and_token_wired_on_cronjob }
  - { from: backup_unit_tests, verifies: backup_cli_round_trip }
  - { from: backup_unit_tests, verifies: backup_feature_gated_off_default_build }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "llm storage documents admin backup/restore"
      risk: low
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "no backup field -> no CronJob"
      risk: low
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "backup field -> CronJob rendered"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "retention + token wired on CronJob"
      risk: medium
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: "lumen backup CLI round trip"
      risk: medium
      verifymethod: test
    }
    requirement R6 {
      id: R6
      text: "backup feature gated off default build"
      risk: low
      verifymethod: inspection
    }
    element spec_cli_unit_tests {
      type: test
    }
    element operator_render_unit_tests {
      type: test
    }
    element backup_unit_tests {
      type: test
    }
    spec_cli_unit_tests - satisfies -> R1
    operator_render_unit_tests - satisfies -> R2
    operator_render_unit_tests - satisfies -> R3
    operator_render_unit_tests - satisfies -> R4
    backup_unit_tests - satisfies -> R5
    backup_unit_tests - satisfies -> R6
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/operator/crd.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add ServingSpec.backup: Option<ServingBackupSpec> (default None, skip_serializing_if none) plus a new ServingBackupSpec struct with schedule: String, destination: String, retention_secs: Option<u64>, admin_token_secret: Option<String>, JsonSchema-derived like the rest of the CRD; no changes to unrelated ServingSpec/LumenSpec fields."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited crd.rs."
  - path: projects/lumen/src/operator/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add a backup_cron_job(lumen) -> Option<Value> function using the shared operator::render::cron_job helper; when spec.serving.backup is set it renders a batch/v1 CronJob named '<name>-backup' on the configured schedule with a container invoking `lumen backup --url http://<name>.<namespace>.svc.cluster.local:7373 --dest <destination> [--retention-secs <n>]`, adding a LUMEN_BACKUP_TOKEN env sourced via secretKeyRef when admin_token_secret is set; render() pushes this object (via .into_iter().chain / conditional push) only when Some."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-render-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited render.rs."
  - path: projects/lumen/src/backup.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "New module gated #[cfg(feature = \"backup\")]: run_backup(base_url, token, dest, retention) fetches {base_url}/admin/backup via reqwest (Bearer auth when token is Some), then hands the response bytes to service_backup::run_backup_once against sink_from_destination(dest) and the given RetentionPolicy, returning a BackupRunResult; unit tests use wiremock to stand in for the admin API and a tempdir + file:// destination for the sink."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-backup-rs.md
    action: create
    section: source
    impl_mode: hand-written
    description: "New SPEC-MANAGED tech-design doc for the new backup.rs module (rust-source-unit), mirroring the format of the other projects-lumen-src-operator-*-rs.md docs."
  - path: projects/lumen/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add `#[cfg(feature = \"backup\")] pub mod backup;` alongside the existing `#[cfg(feature = \"operator\")] pub mod operator;` line."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-lib-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited lib.rs."
  - path: projects/lumen/Cargo.toml
    action: modify
    section: manifest
    impl_mode: hand-written
    description: "Add a `backup = [\"dep:reqwest\"]` feature and extend `operator = [...]` to also pull in `backup`, so every build that already ships the operator (and therefore reqwest via raft-wal in production images) also ships the backup CLI verb; the default (no-feature) build still links no HTTP client."
  - path: projects/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Extend llm_storage_md() with a Snapshot / backup section documenting GET /admin/backup, POST /admin/backup/local, and POST /admin/restore (auth: admin role on \"*\", and that Engine::snapshot()/restore() is the same quiesce-free call the raft snapshotter itself uses -- no separate flush/quiesce step), plus the optional spec.serving.backup CRD field and the `lumen backup` CLI verb it schedules; replaces the current dead forward-reference to a non-existent operator backup surface."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited spec.rs."
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add Command::Backup(BackupArgs) with --url, --dest, --token (env LUMEN_BACKUP_TOKEN), --retention-secs; add a dispatch_backup twin-impl (#[cfg(feature = \"backup\")] real impl calling lumen::backup::run_backup and printing the BackupRunResult as JSON; #[cfg(not(feature = \"backup\"))] fallback that bails with a message to rebuild with --features backup, matching the run_operator/crd_yaml pattern); wire a new match arm in main()."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-bin-lumen-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited bin/lumen.rs."
  - path: projects/lumen/tests/operator_render.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add tests asserting no CronJob is rendered when serving.backup is None (existing fixtures), and that setting serving.backup renders exactly one batch/v1 CronJob named '<name>-backup' with the configured schedule, --dest/--retention-secs args, and a LUMEN_BACKUP_TOKEN env from secretKeyRef when adminTokenSecret is set."
  - path: projects/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Extend the llm storage doc test to assert the new backup/snapshot content (admin/backup routes, admin-role auth, no-quiesce-needed guarantee, and the serving.backup CRD field / lumen backup verb)."
  - path: projects/lumen/tech-design/semantic/lumen-tests.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited operator_render.rs and spec_cli.rs."
```
