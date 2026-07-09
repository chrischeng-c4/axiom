<!-- HANDWRITE-BEGIN gap="missing-generator:logic:e2141d18" tracker="pending-tracker" reason="New docs page (mirrors projects/lumen/docs/deployment-handoff.md): image/binary (dockerfile render + serve), CLI surface, runbooks (binary/docker/k8s kustomize-equivalent operator path #1328), environment variables (TAPE_BIND/STORE/GRACE_SECS/AUTH/TOKEN_REGISTRY_FILE/DATA_DIR/PEER_SERVICE/PEERS #1326 #1327), HTTP surface and probes, smoke sequence, backup/restore runbook (#1329), and release-readiness gates." -->
# tape — test-environment deployment handoff

> One verified path for another team to stand up tape (topic replay journal)
> in a test environment without reading source. tape is the **durable
> replay/archive surface** — `relay` is the online broker and delivery
> surface; tape is for re-reading history after the fact (offset/time replay,
> consumer checkpoints, backfill/audit).

Covers: which image to run, the supported CLI surface, runbooks for binary /
Docker / Kubernetes (operator-driven), the environment variables, an
end-to-end smoke sequence, the backup/restore runbook, and the gates that
prove a build is release-ready.

---

## 1. Image / binary

| Path | Use | Runtime base |
|------|-----|--------------|
| `Dockerfile.release` | **Production-like.** Fetches the published release binary — no Rust toolchain, smallest attack surface. | distroless |
| `Dockerfile` | Dev/CI build from source (`cargo build --release -p tape`). | distroless |

```bash
tape dockerfile render --variant release --version 0.1.0 --out /tmp/tape-image
docker build -f /tmp/tape-image/Dockerfile.release -t tape:0.1.0 .
docker run --rm -p 7137:7137 tape:0.1.0 serve
```

`tape dockerfile render --variant source` renders the source-build Dockerfile
for local/CI images. Raft peer RPCs share the same h2c HTTP port as the
serving API (no separate peer port).

---

## 2. CLI surface (first-level commands)

`tape --help` (from `src/bin/tape.rs`):

| Command | Purpose |
|---------|---------|
| `tape serve` | Run a serving node: HTTP API + (in HA mode) the raft group. |
| `tape append` / `tape replay` / `tape checkpoint` | Local file-backed journal admin (no server needed). |
| `tape spec` / `tape spec gen` | Print the machine-readable contract, or generate a typed ts/py/rust client. |
| `tape backup` | Pull a snapshot from a running node's `/admin/backup` and ship it to a destination sink (#1329). |
| `tape dockerfile` | Render source/release Dockerfiles. |
| `tape k8s` | Render Kubernetes layers: `crd render`, `operator render\|run`, `instance render`. |
| `tape llm` / `tape upgrade` / `tape issue` | Shared ecosystem CLI convention (agent docs, self-update, issue search/view/create). |

---

## 3. Runbooks

### 3a. Local binary (single node, embedded journal)

```bash
./apps/tape/scripts/dev-single.sh
# or directly:
tape serve --bind 127.0.0.1:7137 --store .tape/journal.json
```

### 3b. Docker

```bash
docker run --rm -p 7137:7137 tape:0.1.0 serve --bind 0.0.0.0:7137
```

### 3c. Local multi-node raft (auto-mode HA)

```bash
./apps/tape/scripts/dev-cluster.sh
```

Boots 3 `tape serve` processes on `:7137`/`:7138`/`:7139` with
`REPLICAS_PER_SHARD=3` so they form one raft group; append on any node
replicates to the others.

### 3d. Kubernetes (operator, CRD-driven)

```bash
tape k8s crd render --out /tmp/tape-k8s/crd.yaml
tape k8s operator render --namespace tape-system --out /tmp/tape-k8s
kubectl apply -f /tmp/tape-k8s/crd.yaml       # cluster-scoped API
kubectl apply -f /tmp/tape-k8s/operator.yaml  # control-plane namespace/RBAC/controller
```

Then create a `Tape` custom resource in the app namespace; the operator
reconciles the StatefulSet, Service, and PDB for that CR's namespace:

```bash
tape k8s instance render --profile staging --namespace journal --name tape --out /tmp/tape-k8s
kubectl apply -f /tmp/tape-k8s/tape.yaml
```

Profiles: `dev` (one pod, small disk, verbose logs), `staging` (prod-shaped
single node), `prod` (3-replica raft-HA group, auth required), `template`
(fill-in-the-blanks skeleton). HA (`replicasPerShard > 1`) instances must
keep the CR name `tape` — `tape serve` derives raft peer DNS as
`tape-<ordinal>.<peer-service>`.

> **Note (#1328):** the CRD/operator/instance render path and its offline
> render tests are implemented and verified; there is no live kind-cluster
> failover proof yet (deferred — no cluster was available in that slice). Do
> not claim live-cluster dogfood evidence beyond the offline render/CLI
> gates until that follow-up lands.

---

## 4. Environment variables

`serve` reads flags and env (flag wins). Source: `src/bin/tape.rs::ServeArgs`,
`src/auth.rs`, `libs/raft-host/src/cluster.rs`.

| Area | Env (≡ flag) | Default |
|------|--------------|---------|
| Bind | `TAPE_BIND` (`--bind`) | `127.0.0.1:7137` |
| Storage | `TAPE_STORE` (`--store`) | unset (in-memory) |
| Shutdown | `TAPE_GRACE_SECS` (`--grace-secs`) | `10` |
| **Auth** | `TAPE_AUTH` (`--auth`) — `off`\|`required`; `TAPE_TOKEN_REGISTRY_FILE` (`--token-registry-file`) bearer-token registry JSON; `TAPE_TOKENS` legacy/dev inline fallback | `off` |
| **Raft HA** | `TAPE_DATA_DIR` (`--data-dir`), `TAPE_PEER_SERVICE` (`--peer-service`), `TAPE_PEERS`, plus the standard `POD_NAME`/`SHARD_COUNT`/`REPLICAS_PER_SHARD`/`VOTER_COUNT` downward-API quartet | peer-service `tape` |
| Peer mTLS (config-surface only) | `TAPE_PEER_TLS_CERT`/`_KEY`/`_CA`, `TAPE_PEER_MTLS=on\|off` | unset |
| Backup | `TAPE_BACKUP_TOKEN` (`tape backup --token`) | unset |

> **Production:** set `TAPE_AUTH=required` + `TAPE_TOKEN_REGISTRY_FILE`, and
> for HA deployments set `REPLICAS_PER_SHARD>1` plus a durable `TAPE_DATA_DIR`
> volume. Peer-mTLS material validates at startup but termination is not yet
> applied on the raft peer port (raft-host's h2c transport has no TLS seam
> yet — peer RPCs stay plain h2c even with `TAPE_PEER_MTLS=on`; tracked as a
> known gap, not a silent security regression).

---

## 5. HTTP surface & probes

Registered in `src/server.rs` (standard probes via the shared
`libs/service-http` shell; data-plane routes are tape-specific):

| Path | Purpose | Auth |
|------|---------|------|
| `GET /healthz` | Liveness. | no |
| `GET /readyz` | Readiness — `503` while draining after SIGTERM. | no |
| `GET /metrics` | Prometheus text. | no |
| `GET /openapi.json`, `GET /docs` | OpenAPI 3 + Swagger UI. | no |
| `POST /topics/{topic}/append` | Append one event envelope. | per `TAPE_AUTH` (write) |
| `GET /topics/{topic}/replay` | Replay by offset or timestamp. | per `TAPE_AUTH` (read) |
| `PUT /topics/{topic}/consumers/{consumer}/checkpoint` | Advance a consumer cursor. | per `TAPE_AUTH` (write) |
| `GET /admin/backup` | Stream a whole-journal snapshot (#1329). | per `TAPE_AUTH` (needs `admin` role on `*` when `--auth required`) |

---

## 6. Smoke sequence (end-to-end)

```bash
BASE=http://localhost:7137
curl -fs $BASE/healthz                                    # -> ok
curl -fs $BASE/readyz                                     # -> ok (200)

curl -fs -X POST $BASE/topics/orders.created/append \
  -H 'content-type: application/json' \
  -d '{"payload":{"order_id":"o-1"}}'

curl -fs "$BASE/topics/orders.created/replay?from_offset=0" | jq '.events'

curl -fs -X PUT $BASE/topics/orders.created/consumers/c1/checkpoint \
  -H 'content-type: application/json' -d '{"offset":0}'

curl -fs $BASE/metrics | head -5                          # Prometheus text
```

---

## 7. Backup / restore runbook (#1329)

```bash
# operator-driven or ad hoc, against a running node:
tape backup --url http://localhost:7137 --dest file:///tmp/tape-backups \
  --token "$TAPE_BACKUP_TOKEN" --retention-secs 604800
```

`--dest` accepts `file://` (always supported) or `s3://` (feature `backup`,
`service-backup/s3`). The bearer token needs `admin` on `*`; omit it entirely
when the node runs `--auth off`. `--retention-secs` prunes older backup
objects at the destination after a successful put; omit to keep everything.
In-cluster, the operator's backup CronJob passes
`http://<name>.<namespace>.svc.cluster.local:7137` as `--url`.

---

## 8. Release-readiness gates

A build is **not** production-ready until these pass (run from repo root
unless noted):

```bash
cargo build -p tape
cargo test -p tape

aw health --project tape
aw td code-check <wi-id>   # per-slice terminal gate

cd apps/tape && ../../target/debug/vat run meter-perf
cd apps/tape && ../../target/debug/vat run guard-security
```

> Retention/backfill, long-running soak, and security-hardening (authz/audit/
> secret-rotation) gates remain genuinely `planned` — see
> `apps/tape/README.md`'s Capability Index. They are out of scope for the
> service-archetype convergence epic (#1324) and are not claimed here.

---

*Generated as the tape docs+scripts+traits polish handoff (#1331, epic
#1324). Coordinates (env names, ports, paths) are sourced from
`src/bin/tape.rs`, `src/server.rs`, `src/auth.rs`, `libs/raft-host/src/
cluster.rs`, `k8s/`, `Dockerfile.release`.*
<!-- HANDWRITE-END -->
