---
id: relay-backup-tls-spec-cli
summary: >
  Complete relay's remaining shared-kit archetype rows — Backup/restore, peer
  TLS, and OpenAPI client codegen — as the deploy-CLI tail on the single relay
  bin. `relay spec [--format openapi|openapi-yaml|json-schema]` is the offline
  twin of /openapi.json and `relay spec gen --lang ts|py|rust --out <dir>`
  generates typed clients through the shared cclab-openapi-codegen (keep #777
  pattern; relay has no request-shape/value catalogs, so keep's
  --shapes/--fields are deliberately omitted). `relay backup --url --dest`
  (feature `backup` = dep:reqwest + service-backup/s3, lumen #808 layout)
  fetches a consistent snapshot from a RUNNING node's new admin-guarded
  `GET /admin/backup` endpoint — which returns the exact bytes
  RelayStateMachine::snapshot produces (dump_live live-state serialization +
  applied raft index; one format, shared via public raft::snapshot_bytes) —
  and ships them to a libs/service-backup destination sink with optional age
  retention; restore is a load_live merge on a fresh node (idempotent per
  message_id — surplus entries redeliver, at-least-once). Peer-mTLS material
  loads through a thin src/peer_tls.rs adapter over libs/service-tls
  (PeerTlsConfig::from_env("RELAY_PEER"): RELAY_PEER_TLS_CERT/KEY/CA +
  RELAY_PEER_MTLS=on|off), validated fail-fast at serve startup in replica
  mode; mTLS termination on the raft peer port is a filed raft-host/h2c seam
  gap, not hacked around. The operator CRD gains spec.backup (flat URI string
  destination — keep #776 structural-schema trap) rendering an optional
  <name>-backup CronJob via the shared operator::render::cron_job. The llm
  operations topic documents all three surfaces.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-backup-tls-spec-cli-flow
entry: main
nodes:
  main:
    kind: start
    label: "Single relay bin gains the deploy-CLI tail: Command::Spec + Command::Backup arms beside serve/llm/upgrade/issue/k8s/dockerfile. main still installs the rustls crypto provider FIRST — now unconditionally via service_tls::install_default_crypto_provider (service-tls links rustls in every build, so the private rustls-provider feature indirection collapses; src/tls.rs delegates)"
  verb:
    kind: decision
    label: "Which verb? spec | spec gen | backup. spec/spec gen are OFFLINE (no server, no network, stdout/--out only); backup needs a RUNNING node (HTTP fetch) and the `backup` build feature"
  spec:
    kind: process
    label: "relay spec [--format openapi|openapi-yaml|json-schema]: the offline twin of GET /openapi.json. openapi = crate::openapi::api_doc_json() (pretty JSON, same utoipa document the served route renders); openapi-yaml = serde_yaml::to_string(&openapi()) (new openapi::openapi_yaml accessor, keep's pattern); json-schema = just the component schemas as {components: ...} (new openapi::json_schema_json — relay's utoipa doc registers no named schemas today, so components serializes empty; emitted honestly, never faked). keep's --shapes/--fields flags are OMITTED: relay has no request-shape cookbook or value-type catalog equivalent — deliberate contract difference, not a gap"
  gen:
    kind: process
    label: "relay spec gen --lang ts|py|rust --out <dir> [--http fetch|axios]: cclab_openapi_codegen::generate(&openapi::api_doc_json(), &GenOptions { lang, client_name createClient, emit_types + emit_client, emit_hooks only for Ts }) — keep spec_gen verbatim. create_dir_all(--out) then write each GeneratedFile.rel_path; prints one `generated <path>` line per file. One shared codegen path, no external tool"
  backup:
    kind: process
    label: "relay backup --url <base> --dest <uri> [--token (env RELAY_BACKUP_TOKEN)] [--retention-secs N] (feature `backup` = dep:reqwest + service-backup/s3, lumen #808 layout; without the feature the arm bails with the rebuild hint): src/backup.rs fetch_snapshot_bytes GETs {url}/admin/backup (Bearer token when set; non-2xx bails with status + body), then run_backup hands the EXACT response bytes to service_backup::run_backup_once against sink_from_destination(BackupDestination::from_uri(dest)) + RetentionPolicy — file:// always works, s3:// via the lib's s3 feature, gs:// parses but the lib's sink fails loudly. Prints the BackupRunResult as pretty JSON"
  endpoint:
    kind: process
    label: "Server side: GET /admin/backup joins the data-plane router INSIDE the shared service-auth middleware (probes stay tokenless; this route does not). Handler authorizes Role::Admin on resource '*' (lumen's guard: a cluster-wide admin op) then returns application/json bytes from raft::snapshot_bytes(&relay, applied) where applied = raft-attached ? RelayRaft::applied_index() : 0 (single-node has no raft floor). Registered in the utoipa ApiDoc so /openapi.json and relay spec both list it"
  fmt:
    kind: process
    label: "ONE snapshot format (never a second serialization): raft.rs makes EngineSnapshot pub ({ up_to: Index, subjects: Vec<SubjectLive> }) and factors RelayStateMachine::snapshot into pub fn snapshot_bytes(relay, up_to) — serde_json of dump_live() (live un-acked backlog per (subject, shard) at/above the committed watermark) + the applied index — and restore into pub fn load_snapshot_bytes(relay, bytes) -> Index (parse + load_live, returning up_to). The state machine, the /admin/backup endpoint, and the backup artifact all carry these bytes"
  restore:
    kind: process
    label: "Restore semantics (documented, exercised library-side; no HTTP restore endpoint this slice): feed the artifact's bytes to load_snapshot_bytes on a fresh node — load_live re-publishes every entry through the normal engine path, idempotent per message_id, preserving not_before/priority/appended_at. It is a MERGE, not a wipe: entries the target already holds dedupe; entries it holds that the snapshot lacks stay and redeliver — at-least-once, exactly the raft InstallSnapshot semantics. Leases/acks are NOT in the snapshot (node-local by design, #544): restored work is un-leased and redelivers"
  tls:
    kind: process
    label: "src/peer_tls.rs (new, always compiled): thin adapter over libs/service-tls — pub PeerTlsConfig { cert, key, ca, required } + From conversions, from_env() = service_tls::PeerTlsConfig::from_env(RELAY_PEER) deriving RELAY_PEER_TLS_CERT / RELAY_PEER_TLS_KEY / RELAY_PEER_TLS_CA / RELAY_PEER_MTLS=on|off (lumen's env contract byte-for-byte with the RELAY prefix), plus rustls_server_config()/rustls_client_config() passthroughs. Ok(None) when nothing is set; partial config or a mis-pointed path is an Err naming the variable/path (fail fast, never a silent fallback)"
  tlswire:
    kind: decision
    label: "serve_main, replica/HA mode only: peer_tls::PeerTlsConfig::from_env()? BEFORE the raft group spawns — a misconfigured deployment exits nonzero at startup. None => plain h2c peer transport (today's behavior, logged at info). Some => material validated + rustls builders proven constructible; log identity. required=true => WARN: mTLS termination on the raft peer port is NOT YET APPLIED"
  gap:
    kind: terminal
    label: "FILED SEAM GAP (mirrors lumen, which also ships config-surface-only): raft-host's peer transport is h2c prior-knowledge (raft_host router merged onto the cleartext serve port; peers dialed via reqwest http://) with no TLS acceptor/connector seam. Wiring real mTLS needs raft-host to accept a rustls ServerConfig/ClientConfig pair (or a dedicated TLS peer listener) — a libs/raft-host change benefiting keep/lumen/relay alike. This slice deliberately does NOT hack a parallel TLS stack into h2c; the adapter + startup validation + env contract land now so material can be mounted and verified before the seam exists"
  crd:
    kind: process
    label: "Operator/CRD (feature operator): RelaySpec gains backup: Option<RelayBackupSpec> { schedule (CronJob cron), destination (FLAT URI STRING — file:///path | s3://bucket/prefix | gs://bucket/prefix; keep #776 trap: k8s structural schemas cannot represent the shared tagged-union BackupDestination), retention_secs: Option<u64>, admin_token_secret: Option<String> (Secret whose token key holds a bearer with admin on '*', injected as RELAY_BACKUP_TOKEN — needed when auth: required) }"
  cron:
    kind: terminal
    label: "render(): when spec.backup is Some, append backup_cron_job via the SHARED operator::render::cron_job helper (manifest-only; lumen #808 pattern) — name {name}-backup, component backup, schedule from spec, same relay image, command [relay], args [backup, --url, http://{name}.{ns}.svc.cluster.local:7000, --dest, <destination>, (--retention-secs N)], env RELAY_BACKUP_TOKEN from secretKeyRef when admin_token_secret set, serviceAccountName {name}, 100m/128Mi, history limits 3/3. None => no CronJob; /admin/backup stays reachable for ad hoc use. llm operations topic documents spec/spec gen, backup (CLI + endpoint + CronJob), and the peer-TLS env contract + gap"
edges:
  - { from: main, to: verb }
  - { from: verb, to: spec, label: "spec" }
  - { from: verb, to: gen, label: "spec gen" }
  - { from: verb, to: backup, label: "backup" }
  - { from: backup, to: endpoint, label: "GET /admin/backup" }
  - { from: endpoint, to: fmt, label: "bytes from" }
  - { from: fmt, to: restore, label: "round-trip" }
  - { from: main, to: tls, label: "serve path" }
  - { from: tls, to: tlswire }
  - { from: tlswire, to: gap, label: "required=on" }
  - { from: backup, to: crd, label: "operator schedules" }
  - { from: crd, to: cron }
  - { from: spec, to: gen, label: "same document" }
---
flowchart TD
    main([relay bin: Spec + Backup arms; rustls provider install now unconditional via service-tls]) --> verb{verb?}
    verb -->|spec| spec[offline OpenAPI: json pretty / yaml / components-only json-schema; no --shapes/--fields — relay has no catalogs]
    verb -->|spec gen| gen[cclab_openapi_codegen::generate on relay's own document — ts/py/rust clients into --out]
    verb -->|backup| backup[feature backup: fetch snapshot bytes over HTTP, ship to service-backup sink, print BackupRunResult]
    backup -->|GET /admin/backup| endpoint[admin-guarded route inside the auth middleware: Role::Admin on *]
    endpoint -->|bytes from| fmt[ONE format: pub EngineSnapshot + raft snapshot_bytes / load_snapshot_bytes shared by state machine, endpoint, artifact]
    fmt -->|round-trip| restore[restore = load_live MERGE on a fresh node: idempotent per message_id, leases not replicated, at-least-once]
    main -->|serve path| tls[src/peer_tls.rs: service_tls PeerTlsConfig from_env RELAY_PEER — cert/key/ca + RELAY_PEER_MTLS=on]
    tls --> tlswire{replica mode: material set?}
    tlswire -->|None| plain[plain h2c peers — today's behavior]
    tlswire -->|Some| valid[validate + prove rustls builders; fail fast on partial/mis-pointed config]
    valid -->|required=on| gap([filed gap: raft-host/h2c has no TLS seam — no parallel stack hacked in])
    backup -->|operator schedules| crd[RelaySpec.backup: flat URI destination string + schedule + retentionSecs + adminTokenSecret]
    crd --> cron([shared render cron_job: name-backup CronJob invoking relay backup; RELAY_BACKUP_TOKEN secretKeyRef])
    spec -->|same document| gen
```
