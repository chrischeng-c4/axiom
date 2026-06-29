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
| `Dockerfile` | Dev/CI build from source (`cargo build --release -p lumen --features "otel operator relay-wal jieba"`). | `gcr.io/distroless/cc-debian12:nonroot` |

- Runtime is **distroless, nonroot (uid 65532)**, binary at `/usr/local/bin/lumen`.
- Exposes **`7373`** (HTTP API). `ENTRYPOINT ["/usr/local/bin/lumen"]`, `CMD ["serve"]`.
- Raft peer traffic (HA) uses **`7374`**; relay broker uses **`7000`**.

```bash
docker build -f projects/lumen/Dockerfile.release -t lumen:0.4.4 projects/lumen
docker run --rm -p 7373:7373 lumen:0.4.4            # serves by default
```

---

## 2. CLI surface (first-level commands)

`lumen --help` (from `src/bin/lumen.rs`):

| Command | Purpose |
|---------|---------|
| `lumen serve` | Run a serving node: HTTP API + background apply loop. |
| `lumen spec` | Print the machine-readable contract (OpenAPI 3 / JSON-schema) — offline, no server. |
| `lumen llm` | Print agent-facing integration topics — offline. |
| `lumen k8s` | Operator controller + CRD generation (`k8s operator`, `k8s gen-crd`). |
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
  lumen:0.4.4 serve --host 0.0.0.0
```

### 3c. With a relay broadcast WAL (multi-node fan-out)

```bash
# relay broker on :7000 (see projects/relay), then:
lumen serve --host 0.0.0.0 --wal relay --relay-url http://relay:7000 --relay-subject lumen-wal
```

### 3d. Kubernetes (kustomize overlays)

```bash
kubectl apply -k projects/lumen/k8s/overlays/dev      # 1 serving + 1 relay, pretty logs, auth off
kubectl apply -k projects/lumen/k8s/overlays/staging  # 3 serving, json logs, ServiceMonitor
kubectl apply -k projects/lumen/k8s/overlays/prod     # 6 serving (HPA 6–12), auth required
```

Structure: `k8s/base` (Deployment, Service, HPA, PDB, ConfigMap, relay StatefulSet),
`k8s/components/observability` (ServiceMonitor + PrometheusRule, staging/prod),
`k8s/overlays/{dev,staging,prod}`, `k8s/operator` (CRD + controller).

### 3e. Operator (CRD-driven)

```bash
kubectl apply -k projects/lumen/k8s/operator          # installs CRD lumens.lumen.dev + controller
```

Then create a `Lumen` CR; the operator reconciles all child objects into the CR's namespace:

```yaml
apiVersion: lumen.dev/v1alpha1
kind: Lumen
metadata: { name: my-lumen, namespace: lumen }
spec:
  shards: 3
  auth: required          # or: disabled
  serving: { replicas: 3, cpu: "2", memory: 4Gi }
  broker:  { replicas: 1, storage: 20Gi }
```

---

## 4. Environment variables

`serve` reads flags and env (flag wins). Source: `src/bin/lumen.rs::ServeArgs`, `src/auth.rs`.

| Area | Env (≡ flag) | Default |
|------|--------------|---------|
| Bind | `LUMEN_HOST` (`--host`) / `LUMEN_PORT` (`--port`) | `127.0.0.1` / `7373` |
| Logging | `LUMEN_LOG_LEVEL` (`--log-level`) / `LUMEN_LOG_FORMAT` (`--log-format` pretty\|json) | `info` / `pretty` |
| **WAL backend** | `LUMEN_WAL` (`--wal`) — `auto\|embedded\|nats\|relay\|raft` | `auto` (raft if `replicas>1`, else embedded) |
| NATS | `LUMEN_NATS_URL`, `LUMEN_NATS_CONNECT_TIMEOUT_SECS` | `nats://localhost:4222`, `120` |
| Relay | `LUMEN_RELAY_URL`, `LUMEN_RELAY_SUBJECT`, `LUMEN_RELAY_SUBSCRIBER_ID` | `http://localhost:7000`, `lumen-wal`, pod/host name |
| Raft HA | `LUMEN_RAFT_DATA_DIR`, `LUMEN_RAFT_PORT`, `LUMEN_HEADLESS_SERVICE` | `/var/lib/lumen/raft`, `7374`, `lumen-headless` |
| Sharding/storage | `SHARD_COUNT`, `LUMEN_DATA_DIR`, `LUMEN_PERSISTENCE` (cbor\|segment), `LUMEN_SNAPSHOT_SECS` | `1`, unset, `cbor`, `300` |
| Shutdown | `LUMEN_GRACE_SECS` | `30` |
| Tracing | `LUMEN_OTLP_ENDPOINT` (OTLP/gRPC; traces off when unset) | unset |
| **Auth** | `LUMEN_AUTH` (`off`\|`required`), `LUMEN_TOKENS` (JSON: `{token:{subject,roles:{collection|*:read\|write\|admin}}}`) | `off`, empty |

> **Production:** set `LUMEN_AUTH=required` + `LUMEN_TOKENS`, `LUMEN_LOG_FORMAT=json`,
> and an OTLP endpoint. The prod overlay already sets auth required.

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
cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter
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
