---
id: tape-backup-service-tls-spec-gen-clients
summary: >
  Backup + client-codegen slice for apps/tape (WI #1329, epic #1324),
  mirroring relay's WI #1209 slice. Adds a `GET /admin/backup` route (inside
  the bearer-auth data plane, `admin` on `*`) that streams the exact bytes of
  `tape::raft::snapshot_bytes` — the same whole-journal `JournalSnapshot`
  serialization `TapeStateMachine::snapshot`/`restore` already round-trip
  (#1327), reused not reimplemented; a `tape backup --url --dest --token
  --retention-secs` CLI subcommand behind a new `backup` cargo feature
  (`dep:reqwest` + `service-backup/s3`) that fetches the snapshot and ships it
  to a `libs/service-backup` destination sink (`file://` always, `s3://` with
  the feature); `tape spec gen --lang ts|py|rust --out <dir>` generating a
  typed client from tape's existing offline OpenAPI document via the shared
  `libs/openapi-codegen` crate (already a workspace member, not yet wired
  into tape's binary); and `apps/tape/clients/` (Makefile, README.md,
  generated `openapi.json`) mirroring lumen's `clients/` scaffold. Peer-mTLS
  (`apps/tape/src/peer_tls.rs`) is UNCHANGED — WI #1327 already delivered the
  full config-surface + fail-fast validation scope for tape (the
  `TAPE_PEER_TLS_*` / `TAPE_PEER_MTLS=on|off` env contract), so this WI's TLS
  mention is redundant with #1327 and no new TLS code lands here. Restore is
  the existing raft-side `TapeStateMachine::restore` merge path (loaded
  offline/out of band); no new restore CLI verb is added, matching relay's
  scope. No live cluster is available in this environment; verification is
  offline (unit tests over the snapshot round trip + CLI parse tests) plus an
  in-process axum router test for the new route.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-backup-service-tls-spec-gen-clients-flow
entry: route
nodes:
  route:
    kind: start
    label: "tape binary gains Backup subcommand behind feature backup; Spec gains a gen subcommand; server.rs gains GET /admin/backup"
  admin_req:
    kind: process
    label: "GET /admin/backup arrives on the existing bearer-auth data plane"
  admin_auth:
    kind: decision
    label: "authorize principal, admin on '*'"
  admin_deny:
    kind: terminal
    label: "403 forbidden ApiErr envelope"
  admin_snap:
    kind: process
    label: "tape::raft::snapshot_bytes(journal_handle, applied_index) -- reuses JournalSnapshot shape from TapeStateMachine::snapshot/restore (#1327)"
  admin_ok:
    kind: terminal
    label: "200 application/json JournalSnapshot bytes"
  cli_backup:
    kind: process
    label: "tape backup --url --dest --token --retention-secs, feature backup; without feature: nonzero exit + rebuild hint"
  backup_fetch:
    kind: process
    label: "backup::fetch_snapshot_bytes: reqwest GET {url}/admin/backup, optional Bearer, non-2xx bails with status+body"
  backup_ship:
    kind: process
    label: "backup::run_backup: service_backup::sink_from_destination(dest) + run_backup_once(sink, now, bytes, retention)"
  backup_done:
    kind: terminal
    label: "print BackupRunResult JSON"
  cli_spec_gen:
    kind: process
    label: "tape spec gen --lang ts|py|rust --out DIR --http fetch|axios"
  spec_gen_call:
    kind: process
    label: "cclab_openapi_codegen::generate(tape::spec::openapi_json(), opts)"
  spec_gen_done:
    kind: terminal
    label: "write generated files under --out; print each path"
  clients_dir:
    kind: terminal
    label: "apps/tape/clients/: Makefile + README.md + openapi.json, mirrors lumen's clients/ layout"
  peer_tls_note:
    kind: terminal
    label: "apps/tape/src/peer_tls.rs UNCHANGED -- #1327 already delivered the config-surface + fail-fast validation scope; no new TLS code lands here"
edges:
  - { from: route, to: admin_req }
  - { from: admin_req, to: admin_auth }
  - { from: admin_auth, to: admin_deny, label: "denied" }
  - { from: admin_auth, to: admin_snap, label: "admin on *" }
  - { from: admin_snap, to: admin_ok }
  - { from: route, to: cli_backup, label: "tape backup" }
  - { from: cli_backup, to: backup_fetch }
  - { from: backup_fetch, to: backup_ship }
  - { from: backup_ship, to: backup_done }
  - { from: route, to: cli_spec_gen, label: "tape spec gen" }
  - { from: cli_spec_gen, to: spec_gen_call }
  - { from: spec_gen_call, to: spec_gen_done }
  - { from: route, to: clients_dir, label: "clients scaffold" }
  - { from: route, to: peer_tls_note, label: "scope check #1327" }
---
flowchart TD
    route[tape binary gains Backup subcommand behind feature backup; Spec gains a gen subcommand; server.rs gains GET /admin/backup] --> admin_req[GET /admin/backup arrives on the existing bearer-auth data plane]
    admin_req --> admin_auth{authorize principal, admin on '*'}
    admin_auth -->|denied| admin_deny([403 forbidden ApiErr envelope])
    admin_auth -->|admin on *| admin_snap[tape::raft::snapshot_bytes journal_handle, applied_index -- reuses JournalSnapshot shape from TapeStateMachine::snapshot/restore #1327]
    admin_snap --> admin_ok([200 application/json JournalSnapshot bytes])
    route -->|tape backup| cli_backup[tape backup --url --dest --token --retention-secs, feature backup; without feature: nonzero exit + rebuild hint]
    cli_backup --> backup_fetch[backup::fetch_snapshot_bytes: reqwest GET url/admin/backup, optional Bearer, non-2xx bails with status+body]
    backup_fetch --> backup_ship[backup::run_backup: service_backup::sink_from_destination dest + run_backup_once sink, now, bytes, retention]
    backup_ship --> backup_done([print BackupRunResult JSON])
    route -->|tape spec gen| cli_spec_gen[tape spec gen --lang ts py rust --out DIR --http fetch axios]
    cli_spec_gen --> spec_gen_call[cclab_openapi_codegen::generate tape::spec::openapi_json, opts]
    spec_gen_call --> spec_gen_done([write generated files under --out; print each path])
    route -->|clients scaffold| clients_dir[apps/tape/clients/: Makefile + README.md + openapi.json, mirrors lumen clients layout]
    route -->|scope check #1327| peer_tls_note[apps/tape/src/peer_tls.rs UNCHANGED -- #1327 already delivered the config-surface + fail-fast validation scope; no new TLS code]
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-backup-service-tls-spec-gen-clients-verification
requirements:
  admin_backup_route:
    id: R1
    text: "GET /admin/backup requires admin role on '*' and streams tape::raft::snapshot_bytes JSON"
    kind: functional
    risk: medium
    verify: server::tests::admin_backup_requires_admin_and_streams_snapshot
  backup_cli_parses:
    id: R3
    text: "tape backup CLI subcommand parses --url/--dest/--token/--retention-secs behind the backup feature"
    kind: functional
    risk: low
    verify: bin/tape.rs::tests::backup_verb_parses
  backup_cli_snapshot_fetch:
    id: R2
    text: "tape backup fetches /admin/backup bytes and ships them unmodified to a service-backup destination sink"
    kind: functional
    risk: medium
    verify: backup::tests::run_backup_ships_fetched_bytes_to_sink
  clients_scaffold_present:
    id: R5
    text: "apps/tape/clients/ ships Makefile, README.md, and a generated openapi.json mirroring lumen's clients/ layout"
    kind: regression
    risk: low
    verify: manual: apps/tape/clients/{Makefile,README.md,openapi.json} exist
  peer_tls_unchanged:
    id: R6
    text: "peer_tls.rs config-surface + fail-fast validation from WI #1327 is left unchanged; no new TLS termination code is added"
    kind: regression
    risk: low
    verify: peer_tls::tests (existing, unmodified) still pass
  spec_gen_client_codegen:
    id: R4
    text: "tape spec gen --lang ts|py|rust --out DIR writes a typed client from tape's own OpenAPI document via libs/openapi-codegen"
    kind: functional
    risk: medium
    verify: bin/tape.rs::tests::spec_gen_verbs_parse_and_generate
---
flowchart TD
    r1[R1 admin backup route] --> server_tests_admin_backup_requires_admin_and_streams_snapshot[server::tests::admin_backup_requires_admin_and_streams_snapshot]
    r2[R2 backup cli snapshot fetch] --> backup_tests_run_backup_ships_fetched_bytes_to_sink[backup::tests::run_backup_ships_fetched_bytes_to_sink]
    r3[R3 backup cli parses] --> bin_tape_rs_tests_backup_verb_parses[bin/tape.rs::tests::backup_verb_parses]
    r4[R4 spec gen client codegen] --> bin_tape_rs_tests_spec_gen_verbs_parse_and_generate[bin/tape.rs::tests::spec_gen_verbs_parse_and_generate]
    r5[R5 clients scaffold present] --> manual_apps_tape_clients_makefile_readme_md_openapi_json_exist[manual: apps/tape/clients/{Makefile,README.md,openapi.json} exist]
    r6[R6 peer tls unchanged] --> peer_tls_tests_existing_unmodified_still_pass[peer_tls::tests (existing, unmodified) still pass]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add unconditional cclab-openapi-codegen and service-backup deps (schema/local-sink types are cheap); add an optional reqwest dep for the backup feature's HTTP fetch (already a dev-dependency); add a `backup = [\"dep:reqwest\", \"service-backup/s3\"]` feature entry, mirroring relay's Cargo.toml block."
  - path: apps/tape/src/raft.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add a free fn snapshot_bytes(journal: &Arc<Mutex<TapeJournal>>, up_to: Index) -> Result<Vec<u8>> that serializes the SAME (now pub(crate)) JournalSnapshot { up_to, journal } shape TapeStateMachine::snapshot/restore already round-trip, callable without a live raft group (single-node serving has no TapeStateMachine instance)."
  - path: apps/tape/src/server.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add GET /admin/backup on the auth-guarded data-plane router (admin on \"*\" via crate::auth::authorize), returning tape::raft::snapshot_bytes(journal_handle, applied_index) as application/json (applied_index from state.raft() when set in HA mode, else 0 in single-node mode)."
  - path: apps/tape/src/backup.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "New module (feature backup): fetch_snapshot_bytes(base_url, token) GETs {base_url}/admin/backup via reqwest (Bearer when set, non-2xx bails with status+body); run_backup(base_url, token, dest, retention) hands the exact bytes to service_backup::run_backup_once against sink_from_destination -- relay's src/backup.rs pattern verbatim (transport + shipping only, no snapshot logic)."
  - path: apps/tape/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Register #[cfg(feature = \"backup\")] pub mod backup;"
  - path: apps/tape/src/bin/tape.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add Backup(BackupArgs) top-level subcommand (feature-gated dispatch, nonzero-exit rebuild hint without the feature, mirroring K8s operator run's pattern); add a Gen(GenArgs) subcommand under Spec (spec gen --lang ts|py|rust --out DIR --http fetch|axios) calling cclab_openapi_codegen::generate(tape::spec::openapi_json(), opts) and writing files to --out."
  - path: apps/tape/clients/Makefile
    action: create
    section: logic
    impl_mode: hand-written
    description: "make gen-ts/gen-py/gen-rust targets wrapping `cargo run -p tape --features self-update -- spec gen --lang <lang> --out clients/<lang>`, plus a refresh-openapi target regenerating clients/openapi.json, mirroring lumen's apps/lumen/clients/Makefile layout."
  - path: apps/tape/clients/README.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Usage doc for the clients/ scaffold: what openapi.json is, how to regenerate it and the per-language clients via the Makefile, mirroring lumen's clients/README.md."
  - path: apps/tape/clients/openapi.json
    action: create
    section: logic
    impl_mode: hand-written
    description: "Checked-in snapshot of tape spec --format openapi (the same document GET /openapi.json serves) for offline client generation without a running server."
  - path: apps/tape/tests/backup.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Feature-gated (backup) integration test: admin_backup route denies non-admin principals and returns 200 JSON for an admin token; fetch_snapshot_bytes/run_backup round-trip against an in-process axum server, shipping to a file:// destination sink."
  - path: apps/tape/README.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Update the HTTP/2 API List capability's spec-gen sub-claim (tape spec gen now real, not just documented) and note the /admin/backup + tape backup surface where the existing capability rows reference backup/DR, without touching unrelated rows."
```
