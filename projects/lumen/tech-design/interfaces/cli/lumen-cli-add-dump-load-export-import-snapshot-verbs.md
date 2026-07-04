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
entry: start
nodes:
  start: { kind: start, label: "lumen <dump|export|load|import>" }
  token: { kind: process, label: "token = --token or LUMEN_BACKUP_TOKEN; omitted when auth off" }
  verb: { kind: decision, label: "verb class" }
  dump: { kind: process, label: "dump/export: GET {url}/admin/backup with optional Bearer token" }
  dump_ok: { kind: decision, label: "2xx?" }
  dump_err: { kind: terminal, label: "bail with status + response body" }
  out: { kind: decision, label: "--out provided?" }
  stdout: { kind: terminal, label: "write exact response bytes to stdout" }
  outfile: { kind: terminal, label: "create parent dir if needed; write exact response bytes to file" }
  load: { kind: decision, label: "load/import input" }
  infile: { kind: process, label: "--file path: read exact bytes" }
  stdin: { kind: process, label: "no --file: read all stdin bytes" }
  post: { kind: process, label: "POST {url}/admin/restore Content-Type: application/json with optional Bearer token" }
  restore_ok: { kind: decision, label: "2xx/204?" }
  restore_err: { kind: terminal, label: "bail with status + response body" }
  restored: { kind: terminal, label: "print JSON {status:'restored', url}" }
edges:
  - { from: start, to: token }
  - { from: token, to: verb }
  - { from: verb, to: dump, label: "dump/export" }
  - { from: dump, to: dump_ok }
  - { from: dump_ok, to: dump_err, label: "no" }
  - { from: dump_ok, to: out, label: "yes" }
  - { from: out, to: outfile, label: "yes" }
  - { from: out, to: stdout, label: "no" }
  - { from: verb, to: load, label: "load/import" }
  - { from: load, to: infile, label: "--file" }
  - { from: load, to: stdin, label: "stdin" }
  - { from: infile, to: post }
  - { from: stdin, to: post }
  - { from: post, to: restore_ok }
  - { from: restore_ok, to: restore_err, label: "no" }
  - { from: restore_ok, to: restored, label: "yes" }
---
flowchart TD
    start([lumen dump/export/load/import]) --> token[token = --token or LUMEN_BACKUP_TOKEN]
    token --> verb{verb class}
    verb -->|dump/export| dump[GET /admin/backup]
    dump --> dump_ok{2xx?}
    dump_ok -->|no| dump_err([bail with status + body])
    dump_ok -->|yes| out{--out?}
    out -->|yes| outfile([write exact bytes to file])
    out -->|no| stdout([write exact bytes to stdout])
    verb -->|load/import| load{input source}
    load -->|--file| infile[read file bytes]
    load -->|stdin| stdin[read stdin bytes]
    infile --> post[POST /admin/restore]
    stdin --> post
    post --> restore_ok{2xx/204?}
    restore_ok -->|no| restore_err([bail with status + body])
    restore_ok -->|yes| restored([print JSON restored status])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-cli-snapshot-data-movement-verification
requirements:
  help_surface:
    id: R1
    text: "`lumen --help` lists dump/export/load/import with wording that distinguishes ad hoc SnapshotV1 movement from `backup` sink transport."
    kind: functional
    risk: medium
    verify: test
  export_file:
    id: R2
    text: "Export helper writes the exact `/admin/backup` response bytes to `--out` and the parsed JSON has `version: 1` plus collections."
    kind: functional
    risk: high
    verify: test
  import_file:
    id: R3
    text: "Import helper reads SnapshotV1 JSON from a file and restores it into a fresh server through `/admin/restore`."
    kind: functional
    risk: high
    verify: test
  aliases:
    id: R4
    text: "`dump` behaves as an export alias and `load` behaves as an import alias through shared dispatch."
    kind: regression
    risk: medium
    verify: test
  token_fallback:
    id: R5
    text: "The new verbs expose `--token` with `LUMEN_BACKUP_TOKEN` fallback like `backup`."
    kind: functional
    risk: medium
    verify: test
---
flowchart TD
    r1[R1 help surface] --> cli_convention[cargo test -p lumen --test cli_convention]
    r5[R5 token fallback] --> cli_convention
    r2[R2 export file] --> backup_restore[cargo test -p lumen --test backup_restore_e2e]
    r3[R3 import file] --> backup_restore
    r4[R4 alias dispatch] --> backup_restore
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/backup.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Factor `fetch_snapshot_bytes` and add `restore_snapshot_bytes` so direct CLI verbs and `lumen backup` share admin API HTTP behavior."
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add top-level dump/export/load/import commands with `--url`, token fallback, `--out`, and `--file`/stdin routing."
  - path: projects/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Document the direct CLI snapshot movement verbs in the storage LLM topic."
  - path: projects/lumen/tests/cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Assert top-level help exposes dump/export/load/import and that token flags are present on each direct verb."
  - path: projects/lumen/tests/backup_restore_e2e.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Exercise export-to-file and import-from-file through the shared HTTP helper path."
```
