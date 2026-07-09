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
    label: "tape binary gains Backup(BackupArgs) (behind cargo feature backup) and Spec gains a gen subcommand; server.rs gains GET /admin/backup on the existing bearer-auth data plane"
  admin_req:
    kind: process
    label: "GET /admin/backup request arrives inside the /topics-adjacent auth-guarded router"
  admin_auth:
    kind: decision
    label: "crate::auth::authorize(principal, \"*\", Role::Admin)"
  admin_deny:
    kind: terminal
    label: "403 forbidden ({error, message} ApiErr envelope)"
  admin_snap:
    kind: process
    label: "tape::raft::snapshot_bytes(journal_handle, applied_index) -- applied_index from state.raft() when set, else 0; reuses the SAME JournalSnapshot serde_json shape TapeStateMachine::snapshot/restore already round-trip (#1327), just callable without a live raft group"
  admin_ok:
    kind: terminal
    label: "200 application/json JournalSnapshot bytes"
  cli_backup:
    kind: process
    label: "tape backup --url --dest --token --retention-secs (feature = backup; without it, exits nonzero with a rebuild hint like tape k8s operator run)"
  backup_fetch:
    kind: process
    label: "backup::fetch_snapshot_bytes: reqwest GET {url}/admin/backup with optional Bearer token; non-2xx bails with status+body"
  backup_ship:
    kind: process
    label: "backup::run_backup: service_backup::sink_from_destination(dest) + run_backup_once(sink, now, bytes, retention) -- file:// always, s3:// needs backup feature's service-backup/s3"
  backup_done:
    kind: terminal
    label: "print BackupRunResult JSON to stdout"
  cli_spec_gen:
    kind: process
    label: "tape spec gen --lang ts|py|rust --out DIR --http fetch|axios"
  spec_gen_call:
    kind: process
    label: "cclab_openapi_codegen::generate(tape::spec::openapi_json(), GenOptions{lang, out_dir, emit_types/client/hooks}) -- ts/py/rust client + types (+ TanStack Query hooks for ts)"
  spec_gen_done:
    kind: terminal
    label: "write generated files under --out; print each written path"
  clients_dir:
    kind: process
    label: "apps/tape/clients/ scaffold: Makefile (make gen-ts/gen-py/gen-rust wrapping tape spec gen), README.md (usage), openapi.json (checked-in tape spec --format openapi snapshot) -- mirrors lumen's projects/lumen/clients/ layout"
  peer_tls_note:
    kind: process
    label: "apps/tape/src/peer_tls.rs is UNCHANGED -- WI #1327 already delivered the full TAPE_PEER_TLS_*/TAPE_PEER_MTLS=on|off config-surface + fail-fast validation scope; this WI adds no TLS code"
edges:
  - { from: route, to: admin_req, label: "GET /admin/backup" }
  - { from: admin_req, to: admin_auth }
  - { from: admin_auth, to: admin_deny, label: "not admin" }
  - { from: admin_auth, to: admin_snap, label: "admin on *" }
  - { from: admin_snap, to: admin_ok }
  - { from: route, to: cli_backup, label: "tape backup" }
  - { from: cli_backup, to: backup_fetch }
  - { from: backup_fetch, to: backup_ship }
  - { from: backup_ship, to: backup_done }
  - { from: route, to: cli_spec_gen, label: "tape spec gen" }
  - { from: cli_spec_gen, to: spec_gen_call }
  - { from: spec_gen_call, to: spec_gen_done }
  - { from: route, to: clients_dir, label: "apps/tape/clients/ scaffold" }
  - { from: route, to: peer_tls_note, label: "scope check (#1327)" }
---
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
