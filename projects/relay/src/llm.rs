// HANDWRITE-BEGIN gap="missing-generator:logic:c57e7a39" tracker="pending-tracker" reason="relay's cli_std::llm::Topic list (outline, http-api, operations) + the stamped ToolInfo constructor shared by llm/upgrade/issue."
//! relay's agent-facing CLI identity: the build-stamped [`TOOL`] info and the
//! `relay llm` topic list — the single in-code source of truth shared by the
//! standard `llm` / `upgrade` / `issue` commands (CONTRIBUTING.md CLI
//! convention). Compiled into the `relay` bin via `#[path]` include.

/// This binary's identity + build provenance for the standard CLI ops
/// (`upgrade` / `issue`), per the CONTRIBUTING.md CLI convention.
pub const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "relay",
    repo: "chrischeng-c4/axiom",
    target: env!("RELAY_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("RELAY_GIT_SHA"),
    built_at: env!("RELAY_BUILT_AT"),
};

/// relay's agent-facing `llm` topics — the single in-code source of truth.
pub const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "http-api",
        summary: "the HTTP/2 + OpenAPI work-queue surface (publish, consume, len, probes)",
        body: "# relay — HTTP/2 API surface\n\n\
            One port speaks h2c (HTTP/2 cleartext, prior-knowledge). JSON bodies, plus an \
            `application/cbor` fast path on hot verbs. The payload is opaque JSON — relay \
            knows nothing about what it carries.\n\n\
            - `POST /v1/{subject}/publish` — append one message (idempotent on `message_id`; \
              optional `not_before`/`delay_ms` visibility gate + `priority`).\n\
            - `POST /v1/{subject}/publish-batch` — group-commit many messages, one outcome each.\n\
            - `POST /v1/{subject}/consume` — the streaming work-queue consume path (lease/ack \
              ride the stream). New consumers must use this.\n\
            - `POST /v1/{subject}/lease|ack|lease-batch|ack-batch|heartbeat` — DEPRECATED \
              polling verbs, retained for direct-worker deployments.\n\
            - `GET /v1/{subject}/len` — current append count.\n\
            - `/healthz`, `/openapi.json` — probe + machine-readable contract.\n\n\
            The full document: `GET /openapi.json` (served by the binary).\n",
    },
    cli_std::llm::Topic {
        id: "operations",
        summary: "run / configure / deploy — flags, env vars, durability, k8s",
        body: "# relay — operating the server\n\n\
            Bare `relay` runs the server (env-driven; flags override). Key knobs:\n\n\
            - `--bind` (`RELAY_BIND`, default `0.0.0.0:7000`) — h2c listen address.\n\
            - `--data-dir` (`RELAY_DATA_DIR`) — durable log directory; group-commit fsync.\n\
            - `--auth` (`RELAY_AUTH`, `off`|`required`, default `off`) — bearer auth on the \
              /v1 data plane (shared service-auth contract). Probes (`/healthz` `/readyz` \
              `/metrics` `/openapi.json` `/docs`) stay tokenless either way.\n\
            - `--token-registry-file` (`RELAY_TOKEN_REGISTRY_FILE`, production \
              `/var/run/secrets/relay/token-registry.json`) — JSON \
              `{token: {subject, roles: {\"<subject>|*\": \"read|write|admin\"}}}`; \
              validated at startup when auth is required (missing/bad file = exit). \
              publish/publish-batch need `write` on the subject; \
              consume/lease/ack/heartbeat/len need `read`; `admin >= write >= read`, \
              `*` grants cover every subject.\n\
            - clients: `RELAY_URL` for routing + `RELAY_TOKEN` for credentials, sent as \
              `Authorization: Bearer <token>`.\n\
            - lease reclaim runs on a background reconciler (config `reconcile_interval_ms`).\n\n\
            Delivery model: single-cast work-queue — each message is leased to exactly one \
            competing consumer, acked, then deleted (delete-on-ack retention), with \
            lease-expiry redelivery, a dead-letter path, and priority bands.\n\n\
            HA is auto-mode raft (shared raft-host driver): scale the StatefulSet and set \
            `REPLICAS_PER_SHARD` > 1 (plus `POD_NAME`, `SHARD_COUNT=1`, `VOTER_COUNT` from \
            the downward API) and the same `relay` bin runs a raft group — publishes \
            replicate (leader propose; follower publishes are forwarded to the leader), \
            peer RPCs (`/raft/*`, `/raftz`) ride the serve port as tokenless cluster \
            traffic, and `--peer-service` (`RELAY_PEER_SERVICE`) names the headless \
            Service for peer DNS. `RELAY_PEERS=host:port,...` overrides peer DNS for a \
            local multi-node group. No cluster env = plain single-node (zero flags). \
            Limitation: leases/acks are NOT replicated (node-local, like the old driver) \
            — a failover redelivers unacked work; delivery stays at-least-once.\n\n\
            Deploy artifacts (offline renders; the checked-in files are fixtures):\n\n\
            - `relay k8s crd render` — the Relay CustomResourceDefinition \
              (relay.dev/v1alpha1).\n\
            - `relay k8s operator render [--namespace relay-system]` — operator \
              RBAC + Deployment; `relay k8s operator run` runs the controller \
              (needs a build with `--features operator`).\n\
            - `relay k8s instance render --profile dev|staging|prod|template` — a \
              `kind: Relay` CR; prod is the 3-replica raft-HA shape (the operator \
              renders the StatefulSet topology — `k8s/` base stays a single-node \
              direct install for kind/smoke).\n\
            - `relay dockerfile render --variant source|release [--version]` — the \
              from-source and published-release images.\n\
            - `relay spec [--format openapi|openapi-yaml|json-schema]` — the offline \
              twin of `GET /openapi.json`; `relay spec gen --lang ts|py|rust --out \
              <dir>` generates a typed client from it (shared openapi-codegen; \
              relay has no keep-style `--shapes`/`--fields` catalogs).\n\n\
            Backup/restore (`--features backup`): `relay backup --url \
            http://<node>:7000 --dest file:///path|s3://bucket/prefix \
            [--retention-secs N]` fetches a consistent snapshot from a RUNNING \
            node's `GET /admin/backup` (the exact raft-snapshot bytes: the live \
            un-acked backlog + applied index) and ships it to a service-backup \
            sink. The endpoint needs `admin` on `*` when auth is required — pass \
            `--token` or `RELAY_BACKUP_TOKEN` (the operator injects it from \
            `spec.backup.adminTokenSecret`; `spec.backup` renders a `<name>-backup` \
            CronJob). Restore: feed the artifact to `load_live` on a fresh node — \
            an idempotent per-message_id MERGE; leases are node-local, so restored \
            work redelivers (at-least-once).\n\n\
            Peer TLS (replica/HA mode): mount PEM material and set \
            `RELAY_PEER_TLS_CERT` / `RELAY_PEER_TLS_KEY` / `RELAY_PEER_TLS_CA` \
            (+ `RELAY_PEER_MTLS=on` to require client certs). Serve validates the \
            material fail-fast at startup (partial config or a mis-pointed path \
            exits nonzero). HONEST LIMIT: mTLS termination on the raft peer port \
            is not yet applied — raft-host's h2c transport has no TLS seam (filed \
            gap; peer RPCs stay cleartext h2c inside the cluster until it lands).\n",
    },
];
// HANDWRITE-END
