# lumen — test-environment deployment handoff

> One verified path for another team to stand up lumen (search index) in a test
> environment without reading source. lumen is the **search layer**; the OLTP
> store stays the source of truth and the caller owns CDC/ingestion.
> Output is `external_id` + score for the caller to hydrate.

Covers: which image to run, the supported CLI surface, runbooks for binary /
Docker / Kubernetes, the environment variables, an end-to-end smoke sequence,
and the exact gates that prove a build is release-ready.

---

## 1. Image / binary

| Path | Use | Runtime base |
|------|-----|--------------|
| **`Dockerfile.release`** | **Production-like.** Fetches the published release binary — no Rust toolchain, smallest attack surface. | `gcr.io/distroless/cc-debian12:nonroot` |
| `Dockerfile` | Dev/CI build from source (`cargo build --release -p lumen --features "otel operator raft-wal jieba"`). | `gcr.io/distroless/cc-debian12:nonroot` |

- Runtime is **distroless, nonroot (uid 65532)**, binary at `/usr/local/bin/lumen`.
- Exposes **`7373`** (HTTP API). `ENTRYPOINT ["/usr/local/bin/lumen"]`, `CMD ["serve"]`.
- Raft peer RPCs share the same h2c HTTP port as the serving API.

```bash
lumen dockerfile render --variant release --version 0.4.5 --out /tmp/lumen-image
docker build -f /tmp/lumen-image/Dockerfile.release -t lumen:0.4.5 .
docker run --rm -p 7373:7373 lumen:0.4.5            # serves by default
```

---

## 2. CLI surface (first-level commands)

`lumen --help` (from `src/bin/lumen.rs`):

| Command | Purpose |
|---------|---------|
| `lumen serve` | Run a serving node: HTTP API + background apply loop. |
| `lumen spec` | Print the machine-readable contract (OpenAPI 3 / JSON-schema) — offline, no server. |
| `lumen llm` | Print agent-facing integration topics — offline. |
| `lumen dockerfile` | Render source/release Dockerfiles for compose, kind, or a registry build. |
| `lumen k8s` | Render Kubernetes layers: `crd render`, `operator render|run`, and `instance render`. |
| `lumen upgrade` | Self-update from the published GitHub release. |
| `lumen issue` | Search / view / file Lumen issues through the standard `issue search`, `issue view`, and `issue create` group. |

---

## 3. Runbooks

### 3a. Local binary (single node, embedded WAL)

```bash
lumen serve --host 0.0.0.0 --port 7373          # in-process log; no broker needed
# logs: "auth=off — set LUMEN_AUTH=required for production"
```

### 3b. Docker

```bash
docker run --rm -p 7373:7373 \
  -e LUMEN_LOG_FORMAT=json \
  lumen:0.4.5 serve --host 0.0.0.0
```

### 3c. Local multi-node raft

```bash
apps/lumen/scripts/dev-cluster.sh
```

### 3d. Kubernetes (kustomize overlays)

```bash
kubectl apply -k apps/lumen/k8s/overlays/dev      # 1 serving, pretty logs, auth off
kubectl apply -k apps/lumen/k8s/overlays/staging  # 3 serving, json logs, ServiceMonitor
kubectl apply -k apps/lumen/k8s/overlays/prod     # 6 serving (HPA 6–12), auth required
```

Structure: `k8s/base` (Deployment, Service, HPA, PDB, ConfigMap),
`k8s/components/observability` (ServiceMonitor + PrometheusRule, staging/prod),
`k8s/overlays/{dev,staging,prod}`, `k8s/operator` (CRD + controller).

### 3e. Operator (CRD-driven)

```bash
lumen k8s crd render --out /tmp/lumen-k8s/crd.yaml
lumen k8s operator render --namespace lumen-system --out /tmp/lumen-k8s
kubectl apply -f /tmp/lumen-k8s/crd.yaml              # cluster-scoped API
kubectl apply -f /tmp/lumen-k8s/operator.yaml         # control-plane namespace/RBAC/controller
```

Then create a `Lumen` CR in the app namespace; the operator reconciles all
child objects into that CR's namespace:

```bash
lumen k8s instance render --profile staging --namespace search --name catalog --out /tmp/lumen-k8s
kubectl apply -f /tmp/lumen-k8s/lumen.yaml
```

### 3f. CRD versioning & upgrade order

**Versioning.** The CRD ships one version, `v1alpha1`. Additive `spec`/`status`
fields land within `v1alpha1` release-to-release — the served schema is
regenerated from source every release, so a new optional field is a normal
minor-version bump, not a CRD-version bump. A breaking shape change
(rename/remove/retype an existing field) requires a new CRD version
(`v1alpha2`/`v1`) plus an explicit conversion story between versions. **No
conversion webhook exists today** — that is a current limitation, not a
promise, so only additive `v1alpha1` evolution is supported until one ships.

**Upgrade order — CRD first, always** (the #2456 lesson): the API server
silently prunes any field the *stored* schema doesn't know yet, so applying a
CR with a new field (e.g. `serviceAccountName`, `serviceAccountAnnotations`)
against the *old* CRD drops that field without an error.

```bash
lumen k8s crd render --out /tmp/lumen-k8s/crd.yaml
kubectl apply -f /tmp/lumen-k8s/crd.yaml               # 1. CRD first
lumen k8s operator render --namespace lumen-system --out /tmp/lumen-k8s
kubectl apply -f /tmp/lumen-k8s/operator.yaml          # 2. operator next
kubectl apply -f /tmp/lumen-k8s/lumen.yaml             # 3. CR last
```

**Rollback contract:**

| Path | Contract |
|------|----------|
| Binary/image downgrade | Supported. Same-minor rollback (e.g. 0.4.25 → 0.4.24) is expected safe; cross-minor rollback is untested — no compatibility matrix is published for the WAL (`WAL_FORMAT_VERSION`) or segment (`FORMAT_VER`) on-disk formats, so treat it as unverified rather than assumed-safe. |
| CRD downgrade | Not recommended once CRs use newer-version fields — reverting/removing the CRD is destructive (the API server re-validates stored objects against the reverted schema and can reject or prune them). Leave the newer CRD in place across a binary rollback; only the workload image needs to move back. |

---

## 4. Environment variables

`serve` reads flags and env (flag wins). Source: `src/bin/lumen.rs::ServeArgs`, `src/auth.rs`.

| Area | Env (≡ flag) | Default |
|------|--------------|---------|
| Bind | `LUMEN_HOST` (`--host`) / `LUMEN_PORT` (`--port`) | `127.0.0.1` / `7373` |
| Logging | `LUMEN_LOG_LEVEL` (`--log-level`) / `LUMEN_LOG_FORMAT` (`--log-format` pretty\|json) | `info` / `pretty` |
| **WAL backend** | `LUMEN_WAL` (`--wal`) — `auto\|embedded\|nats\|raft` | `auto` (raft if `replicasPerShard > 1`, else embedded) |
| NATS | `LUMEN_NATS_URL`, `LUMEN_NATS_CONNECT_TIMEOUT_SECS` | `nats://localhost:4222`, `120` |
| Raft HA | `LUMEN_RAFT_DATA_DIR`, `LUMEN_HEADLESS_SERVICE`, `LUMEN_PEERS`, `POD_NAME`, `SHARD_COUNT`, `REPLICAS_PER_SHARD`, `VOTER_COUNT` | `/var/lib/lumen/raft`, `lumen-headless`, unset, k8s downward API |
| Sharding/storage | `SHARD_COUNT`, `LUMEN_DATA_DIR`, `LUMEN_PERSISTENCE` (cbor\|segment), `LUMEN_SNAPSHOT_SECS` | `1`, unset, `cbor`, `300` |
| Shutdown | `LUMEN_GRACE_SECS` | `30` |
| Tracing | `LUMEN_OTLP_ENDPOINT` (OTLP/gRPC; traces off when unset) | unset |
| **Auth** | `LUMEN_AUTH` (`off`\|`required`), `LUMEN_TOKEN_REGISTRY_FILE` (path to the credential registry JSON) | `off`, unset |

The registry file holds two disjoint namespaces — `tokens` keys **are** bearer
secrets, `identities` keys are provider-verified emails, and a presented string
is only ever matched against `tokens` (#2678):

```json
{
  "tokens":     { "s3cret": { "subject": "ingest", "roles": { "*": "write" } } },
  "identities": { "dev@example.com": { "subject": "dev", "roles": { "products": "read" } } }
}
```

A flat `{secret: {subject, roles}}` document is still read as bearer secrets only.

> **Production:** set `LUMEN_AUTH=required` + `LUMEN_TOKEN_REGISTRY_FILE`,
> `LUMEN_LOG_FORMAT=json`, and an OTLP endpoint. There is no inline-credential
> env var: a credential passed in the environment is a credential in
> `kubectl describe pod`.
>
> **Under the operator there is no credential field to set at all.** `spec.auth:
> required` is the CRD default and names no source: the CR carried
> `spec.tokensSecret`, `spec.identities` and `spec.identityAudiences` until
> #2872 removed them from the schema, and a CR that still sets one is now
> rejected by the API server rather than applied and ignored. Client identity
> comes from the cluster — a short-lived, audience-bound ServiceAccount token
> that lumen resolves through TokenReview and SubjectAccessReview — so nothing
> mounts a registry into the serving pod (#2870). The `LUMEN_TOKEN_REGISTRY_FILE`
> path above is the standalone-process contract only, and it goes with the
> runtime bearer path in #2871.

---

## 5. HTTP surface & probes

Registered in `src/api.rs::router`:

| Path | Purpose | Auth |
|------|---------|------|
| `GET /healthz` | Liveness — always `ok`. | no |
| `GET /readyz` | Readiness — `200 ok`, `503 draining`. | no |
| `GET /version` | `version` / `git_sha` / `built_at`. | no |
| `GET /metrics` | Prometheus text (v0.0.4). | no |
| `GET /openapi.json`, `GET /docs` | OpenAPI 3 + Swagger UI. | no |
| `GET /debug/cluster` | Raft role / peer-lag snapshot. | no |
| `/collections...`, `/admin/backup...` | Data + admin plane. | per `LUMEN_AUTH` |

K8s probes (`k8s/base/deployment.yaml`): readiness `GET /readyz` (period 10s),
liveness `GET /healthz` (period 30s), startup `GET /healthz`. Prometheus scrape
via `prometheus.io/{scrape,port=7373,path=/metrics}` annotations.

---

## 6. Smoke sequence (end-to-end)

```bash
BASE=http://localhost:7373
curl -fs $BASE/healthz                                   # -> ok
curl -fs $BASE/readyz                                    # -> ok (200)
curl -fs $BASE/version | jq .                            # version/git_sha/built_at

# create a collection
curl -fs -X PUT $BASE/collections/docs -H 'content-type: application/json' -d '{
  "fields": { "bio": {"type":"text","analyzer":"white_space_lower"}, "email": {"type":"keyword"} } }'

# index a document (external_id + field/value items)
curl -fs -X POST $BASE/collections/docs/index -H 'content-type: application/json' -d '{
  "items": [ {"external_id":"doc-1","field":"bio","value":"rust engineer search specialist"},
             {"external_id":"doc-1","field":"email","value":"engineer@example.com"} ] }'

# BM25 search -> expect doc-1 in hits
curl -fs -X POST $BASE/collections/docs/search -H 'content-type: application/json' -d '{
  "query": { "match": {"field":"bio","text":"rust"} }, "limit": 10 }' | jq '.hits'

curl -fs $BASE/metrics | head -5                         # Prometheus text
```

---

## 7. Release-readiness gates

A build is **not** production-ready until these pass (run from repo root unless noted):

```bash
# code quality
cargo fmt -p lumen --check
cargo clippy -p lumen -- -D warnings
RUSTFLAGS=-Dwarnings cargo check -p lumen

# tests
cargo test -p lumen

# spec/codegen + EC
aw ec check --project lumen
aw health --project lumen ec --verify-ec        # EC command matrix green

# aggregate readiness (capability / managed / semantic / traceability / cb / cold / tests)
aw health --project lumen full

# performance (only when perf is part of the release claim) — competitive x100 profile
cd apps/lumen && ../../target/debug/vat run ec-efficiency-meter
```

> The aggregate gate is `aw health --project lumen full` →
> `readiness.production_ready=true`. See `aw.toml` for the EC case matrix and
> `vat.toml` for the perf profiles (`s`/`m`/`l`, competitive vs Postgres/OpenSearch).
> Note (2026-06): the heavy `--verify-ec` / cold-rebuild gates need a
> longer-budget runner than a sandboxed agent session; run them in CI.

---

*Generated as the lumen production-readiness deployment handoff (#163). Coordinates
(env names, ports, paths) are sourced from `src/bin/lumen.rs`, `src/api.rs`,
`src/auth.rs`, `k8s/`, `Dockerfile.release`.*
