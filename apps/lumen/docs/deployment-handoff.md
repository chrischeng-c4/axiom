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
| `lumen k8s` | Render Kubernetes layers: `crd render`, `operator render|run`, `instance render`, and `access render` (client RBAC). |
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
# Local / dev example (use --issuer cas --ca-pool ... for staging/prod):
lumen k8s operator render --namespace lumen-system --issuer ephemeral --trust-domain lumen-dev.svc.id.goog --out /tmp/lumen-k8s
kubectl apply -f /tmp/lumen-k8s/crd.yaml              # cluster-scoped API
kubectl apply -f /tmp/lumen-k8s/operator.yaml         # control-plane namespace/RBAC/controller
```

Then create a `Lumen` CR in the app namespace; the operator reconciles all
child objects into that CR's namespace:

```bash
lumen k8s instance render --profile staging --namespace search --name catalog --out /tmp/lumen-k8s
kubectl apply -f /tmp/lumen-k8s/lumen.yaml
```

**How the control plane authenticates itself (#2877).** When a CR sets
`spec.auth: required`, the operator and the backup CronJob call the fleet's
admin API as themselves. Neither holds a credential you supply: each pod
mounts a `serviceAccountToken` projection with audience `lumen.axiom.dev` at
`/var/run/secrets/lumen.axiom.dev/token`, and the kubelet rotates that file in
place. The two workloads use **distinct** ServiceAccounts — `lumen-operator`
in `lumen-system` and `<name>-backup` in the CR's namespace — and neither uses
the serving ServiceAccount, so revoking one caller's access does not touch the
other. There is no admin-token Secret to create and no token environment
variable to set; if either caller is denied, grant its ServiceAccount the
`lumenadmin` role rather than looking for material to install.

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
lumen k8s operator render --namespace lumen-system --issuer ephemeral --trust-domain lumen-dev.svc.id.goog --out /tmp/lumen-k8s
kubectl apply -f /tmp/lumen-k8s/operator.yaml          # 2. operator next
kubectl apply -f /tmp/lumen-k8s/lumen.yaml             # 3. CR last
```

**Rollback contract:**

| Path | Contract |
|------|----------|
| Binary/image downgrade | Supported. Same-minor rollback (e.g. 0.4.25 → 0.4.24) is expected safe; cross-minor rollback is untested — no compatibility matrix is published for the WAL (`WAL_FORMAT_VERSION`) or segment (`FORMAT_VER`) on-disk formats, so treat it as unverified rather than assumed-safe. |
| CRD downgrade | Not recommended once CRs use newer-version fields — reverting/removing the CRD is destructive (the API server re-validates stored objects against the reverted schema and can reject or prune them). Leave the newer CRD in place across a binary rollback; only the workload image needs to move back. |

### 3g. Client access — the two-hop grant (#2889)

A caller's identity is a Kubernetes identity, and it changes hands once on the
way in:

1. a human Google account or a Google service account authenticates to
   kube-apiserver through its kubeconfig credential plugin, and RBAC decides
   whether that principal may create a **TokenRequest** for one named client
   ServiceAccount;
2. the short-lived, audience-bound ServiceAccount token that comes back is the
   only credential Lumen ever sees, and Lumen authorizes **that ServiceAccount**
   through SubjectAccessReview.

`lumen k8s access render` renders both halves as one bundle. It reads names and
writes YAML: it never calls `gcloud`, never applies anything, and never handles
material.

```bash
kubectl auth whoami                       # the username RBAC will match
lumen k8s access render \
  --namespace search --client-sa app-client \
  --issuer alice@example.com \
  --issuer lumen-client@my-project.iam.gserviceaccount.com \
  --grant docs=read --grant orders=write --instance-admin \
  --out /tmp/lumen-k8s
kubectl apply -f /tmp/lumen-k8s/access.yaml
```

Five objects: the client `ServiceAccount`; `<sa>-token-issuer` Role +
RoleBinding (hop 1); `<sa>-lumen-access` Role + RoleBinding (hop 2).

**The subject kinds are not interchangeable.** The issuer binding names
`kind: User` — the people who authenticate to the API server. The Lumen binding
names `kind: ServiceAccount` — the identity that arrives at Lumen. Binding the
Google principal to the Lumen Role authorizes a caller that never appears, and
the cluster accepts it without complaint; the request is simply denied at
runtime.

**The issuer grant is deliberately narrow.** `create` on
`serviceaccounts/token` *without* `resourceNames` mints a token for every
ServiceAccount in the namespace, including the operator's — so the rendered
Role always names exactly the one client ServiceAccount, and the renderer
refuses to emit a wildcard anywhere in the bundle.

Grants map to the verbs Lumen's SubjectAccessReview asks for, and are
cumulative: `read` → `get`; `write` → `get`, `update`; `admin` → `get`,
`update`, `delete`. `--instance-admin` adds the instance-wide `lumenadmin`
resource, and only when asked for.

Then mint a token — nothing is stored, so a client repeats this when the old
one expires:

```bash
kubectl create token app-client -n search --audience lumen.axiom.dev --duration 10m
```

Verified on GKE v1.35.6 against an impersonated issuer: the bound principal
mints `app-client`'s token and is refused a sibling ServiceAccount's in the
same namespace and the operator's in `lumen-system`; the client ServiceAccount
is allowed `get` on `docs` and denied `update`, `delete`, an ungranted
collection, and `lumenadmin`; and the Google principal itself is denied
everything on the Lumen side.

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
| **Auth** | `LUMEN_AUTH` (`disabled`\|`required`) — the only value a server starts with is `disabled` | `disabled` |

There is no credential env var, no registry file, and no CRD field that names
a credential source. The two-namespace registry this section used to document
(`tokens` keys were bearer secrets, `identities` keys were provider-verified
emails) was deleted in #2871 along with the code that read it, and the CR
fields that projected it (`spec.tokensSecret`, `spec.identities`,
`spec.identityAudiences`) were deleted from the schema in #2872 — a CR that
still sets one is rejected by the API server rather than applied and ignored.

> **What a deployer configures today: nothing, and that is the honest state.**
> `LUMEN_AUTH=required` (`spec.auth: required`, the CRD default) does not
> start — the process exits at startup naming the missing
> TokenReview/SubjectAccessReview verifier rather than serve an API that
> accepts everyone while claiming to require identity. A Lumen you can reach
> on the network is open; keep it behind a NetworkPolicy.
>
> **Clients need a URL and nothing else.** The Lumen CLI has no credential
> path: no token flag, no token environment variable, and no Kubernetes Secret
> lookup behind either (#2873). `lumen connect` hands its child `LUMEN_URL`
> and nothing more.
>
> **Where identity comes back.** Client identity comes from the cluster — your
> kubeconfig authenticates you to kube-apiserver, an RBAC-authorized
> `TokenRequest` mints a short-lived, audience-bound token for one explicitly
> named client ServiceAccount, and Lumen resolves *that* through TokenReview
> and SubjectAccessReview. A Google access token, ID token, ADC credential,
> GSA key, or metadata-server token is never a Lumen credential. Render both
> RBAC halves with `lumen k8s access render` — see §3g; the CLI's TokenRequest
> call is #2878.
>
> Still set `LUMEN_LOG_FORMAT=json` and an OTLP endpoint in production.

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
