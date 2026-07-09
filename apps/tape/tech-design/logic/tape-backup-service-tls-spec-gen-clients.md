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
