---
id: lumen-cli-snapshot-data-movement
summary: >
  Add direct ad hoc data movement verbs to the lumen CLI without changing the
  existing SnapshotV1 format or the scheduled backup contract. `lumen dump` and
  `lumen export` fetch a running node's `GET /admin/backup` SnapshotV1 JSON and
  write it to stdout or `--out`; `lumen load` and `lumen import` read SnapshotV1
  JSON from `--file` or stdin and post it to `/admin/restore`. `lumen backup`
  remains the scheduled/off-node destination-sink verb with retention.
capability_refs:
  - id: "cli-interface"
    role: primary
    gap: "service-process-interface"
    claim: "service-process-interface"
    coverage: partial
    rationale: >
      Extends lumen's agent-facing CLI with direct dump/load/export/import
      verbs for the already-implemented admin snapshot workflow.
  - id: "backup-restore"
    role: primary
    gap: "rdb-snapshot-restore-localfsrdbstore"
    claim: "rdb-snapshot-restore-localfsrdbstore"
    coverage: partial
    rationale: >
      Reuses the existing SnapshotV1 backup/restore contract; the change is a
      CLI wrapper, not a new data format or backup mechanism.
fill_sections: [logic, unit-test, changes]
---

# TD: Lumen CLI snapshot data movement verbs

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-cli-snapshot-data-movement-contract
entry: user
nodes:
  user: { kind: start, label: "operator/agent needs ad hoc data movement" }
  export_cmd: { kind: decision, label: "dump/export?" }
  import_cmd: { kind: decision, label: "load/import?" }
  get_backup: { kind: process, label: "GET {url}/admin/backup" }
  write_target: { kind: decision, label: "--out provided?" }
  write_file: { kind: terminal, label: "write SnapshotV1 JSON file" }
  write_stdout: { kind: terminal, label: "stream SnapshotV1 JSON to stdout" }
  read_target: { kind: decision, label: "--file provided?" }
  read_file: { kind: process, label: "read SnapshotV1 JSON file" }
  read_stdin: { kind: process, label: "read SnapshotV1 JSON from stdin" }
  post_restore: { kind: process, label: "POST {url}/admin/restore" }
  destructive: { kind: terminal, label: "replace all engine state" }
  backup: { kind: process, label: "existing lumen backup" }
  sink: { kind: terminal, label: "destination sink transport + retention" }
edges:
  - { from: user, to: export_cmd }
  - { from: user, to: import_cmd }
  - { from: export_cmd, to: get_backup }
  - { from: get_backup, to: write_target }
  - { from: write_target, to: write_file, label: "yes" }
  - { from: write_target, to: write_stdout, label: "no" }
  - { from: import_cmd, to: read_target }
  - { from: read_target, to: read_file, label: "yes" }
  - { from: read_target, to: read_stdin, label: "no" }
  - { from: read_file, to: post_restore }
  - { from: read_stdin, to: post_restore }
  - { from: post_restore, to: destructive }
  - { from: backup, to: sink }
  - { from: sink, to: get_backup }
---
flowchart TD
    user["operator/agent needs ad hoc data movement"] --> export_cmd{"dump/export?"}
    user --> import_cmd{"load/import?"}
    export_cmd --> get_backup["GET {url}/admin/backup"]
    get_backup --> write_target{"--out provided?"}
    write_target -->|yes| write_file["write SnapshotV1 JSON file"]
    write_target -->|no| write_stdout["stream SnapshotV1 JSON to stdout"]
    import_cmd --> read_target{"--file provided?"}
    read_target -->|yes| read_file["read SnapshotV1 JSON file"]
    read_target -->|no| read_stdin["read SnapshotV1 JSON from stdin"]
    read_file --> post_restore["POST {url}/admin/restore"]
    read_stdin --> post_restore
    post_restore --> destructive["replace all engine state"]
    backup["existing lumen backup"] --> sink["destination sink transport + retention"]
    sink -. remains separate .-> get_backup
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-cli-snapshot-data-movement-verification
requirements:
  help_surface:
    id: R1
    text: "`lumen --help` lists dump, export, load, import, and backup with distinct snapshot wording."
    kind: functional
    risk: medium
    verify: test
  export_output:
    id: R2
    text: "`lumen export` / `lumen dump` fetch SnapshotV1 JSON and can write it to stdout or `--out` without altering the bytes."
    kind: functional
    risk: high
    verify: test
  import_input:
    id: R3
    text: "`lumen import` / `lumen load` read SnapshotV1 JSON from `--file` or stdin and POST it to `/admin/restore`."
    kind: functional
    risk: high
    verify: test
  token_parity:
    id: R4
    text: "All four direct verbs accept `--token` with the same `LUMEN_BACKUP_TOKEN` fallback as `lumen backup`."
    kind: functional
    risk: medium
    verify: test
---
flowchart TD
    r1[R1 help surface] --> cli_convention[cargo test -p lumen --test cli_convention]
    r2[R2 export/dump output] --> backup_restore[cargo test -p lumen --test backup_restore_e2e]
    r3[R3 import/load input] --> backup_restore
    r4[R4 token parity] --> cli_convention
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/backup.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add shared admin snapshot fetch/restore helpers used by backup, dump/export, and load/import."
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add dump/export/load/import commands and route them through the shared admin snapshot helpers."
  - path: projects/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Update the storage LLM topic to document the direct CLI snapshot movement verbs."
  - path: projects/lumen/tests/cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Verify the new verbs are visible in the top-level help surface."
  - path: projects/lumen/tests/backup_restore_e2e.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Cover an HTTP export/import round trip through the CLI helper path."
```
