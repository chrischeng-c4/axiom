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
            HA: `relay-raft` is the Raft-backed node for Kubernetes (identity/peers from \
            the StatefulSet downward API); see projects/relay/k8s.\n",
    },
];
// HANDWRITE-END
