# vat — local agent test runner capsules

## Brief

`vat` is a local development test runner for the one operator Docker was never
designed for: a **coding/ML agent**. It is *not* "Docker minus the GUI" and it
is not a long-lived process manager. An agent writes `vat.toml`; vat prepares an
ephemeral copy-on-write workspace, starts run-scoped services, waits for
readiness, runs the named runner, captures logs/artifacts/diff/state, and then
cleans up according to the run policy.

1. **The GPU just works — because there is no VM.** On Apple Silicon, Docker
   runs Linux containers inside a Linux VM, and Metal has no compute
   passthrough into that guest. So `torch.mps`, MLX, and `tensorflow-metal` all
   report *no GPU* inside a container, and there is no `--gpus all` that fixes
   it. A vat is **not a VM** — it's a sandboxed *host process* over a
   copy-on-write workspace. The workload never leaves macOS, so the Apple GPU
   was never taken away. Nothing to "bridge".

2. **The operating surface faces the agent, not a human dev.** Docker's
   ergonomics (a daemon, a desktop app, `ps`/`inspect`/`logs`/`diff` as
   separate human-readable text dumps) are tradeoffs *for developers*. vat's
   tradeoffs are *for agents*: one structured [`vat state`](#vat-state) JSON
   that answers "what is this environment right now", forwarded exit codes,
   copy-on-write disposability, and git-like fork/snapshot — all on the
   **unflagged** path.

## Capabilities

Canonical field-style capability contracts below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Agent-Native GPU-Native Dev Containers | #4152 | implemented | verified | smoke | ready | vat runs sandboxed host-process environments over copy-on-write workspaces so coding and ML agents get structured state, local test runner evidence, fork/snapshot, ephemeral local Kubernetes clusters (kind/k3d/minikube), GCP/Firebase emulators (including built-in Rust Pub/Sub, Firebase Auth, Cloud Tasks, Cloud Scheduler, a Cloud Workflows interpreter, and Cloud Storage), and host GPU access without a VM. |

### Agent-Native GPU-Native Dev Containers

ID: agent-native-gpu-native-dev-containers
Type: AgentFirst
Surfaces: CLI: `vat run` + `vat emulator` + `vat state/diff/fork/snapshot` - Agent-facing dev-container CLI: copy-on-write run + structured state/diff, fork/snapshot, built-in GCP/Firebase emulators (REST+gRPC), and the network sandbox (routes/egress/hermetic).
EC Dimensions: behavior: `cargo test -p vat` - vat.toml run protocol, built-in emulators (REST + gRPC), transparent routing, and seatbelt egress/hermetic conformance.
Root WI: #4152
Status: verified
Required Verification: smoke
Promise:
vat runs sandboxed host-process environments over copy-on-write workspaces so coding and ML agents get structured state, local test runner evidence, fork/snapshot, ephemeral local Kubernetes clusters (kind/k3d/minikube), GCP/Firebase emulators (including built-in Rust Pub/Sub, Firebase Auth, Cloud Tasks, Cloud Scheduler, a Cloud Workflows interpreter, and Cloud Storage), and host GPU access without a VM.
Gate Inventory:
- `cargo test -p vat`; `rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured projects/vat/README.md`; `rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal projects/vat/README.md projects/vat/src/gpu.rs`; `rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS projects/vat/README.md`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Host-process execution and GPU visibility | epic | - | implemented | verified | smoke | `rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal projects/vat/README.md projects/vat/src/gpu.rs` |
| Agent-legible state and diff surface | epic | - | implemented | verified | smoke | `rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured projects/vat/README.md` |
| Local agent test runner protocol | epic | #4152 | implemented | verified | smoke | `cargo test -p vat vat_toml_runner -- --nocapture` |
| Production-like integration scenarios | change | #701 | implemented | verified | smoke | `cargo test -p vat --test vat_toml_runner scenario_ -- --nocapture` |
| Local Kubernetes cluster service and `vat cluster` | change | #141 | implemented | verified | smoke | `cargo test -p vat --test vat_cluster -- --nocapture` |
| GCP / Firebase emulator service presets | change | #143 | implemented | verified | smoke | `cargo test -p vat --test vat_emulators -- --nocapture` |
| Built-in Rust emulators (Pub/Sub gRPC + Firebase Auth REST) | change | #145 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_auth --test vat_emulator_pubsub -- --nocapture` |
| Built-in Rust emulators (Cloud Tasks + Cloud Scheduler) | change | #146 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_tasks --test vat_emulator_scheduler -- --nocapture` |
| Built-in Rust emulator (Cloud Workflows subset interpreter) | change | #147 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_workflows -- --nocapture` |
| Built-in Rust emulator (Cloud Storage / GCS) | change | #148 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_storage -- --nocapture` |
| Built-in HTTP mock + record/replay proxy (HTTPS MITM) | change | #149 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_httpmock -- --nocapture` |
| OpenAPI-driven mock HTTP service (spec → responses) | change | #150 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_openapi -- --nocapture` |
| `vat llm` / `vat upgrade` / `vat issue` (mandatory CLI convention) | change | #491 | implemented | verified | smoke | `cargo test -p vat --test vat_cli_convention -- --nocapture` |
| Dual-protocol emulators (Cloud Tasks + Scheduler gRPC alongside REST) | change | #499 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_tasks_grpc --test vat_emulator_scheduler_grpc -- --nocapture` |
| Network sandbox v1 — transparent HTTP host-routing | change | #503 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_httpmock_routing -- --nocapture` |
| Network sandbox v2 — transparent gRPC routing (h2 MITM) | change | #509 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture` |
| Adopt the shared cli-std crate | change | #514 | implemented | verified | smoke | `cargo test -p vat --test vat_cli_convention -- --nocapture` |
| gRPC reverse-proxy h2c connection pool | change | #516 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture` |
| Network sandbox v3 — seatbelt egress policy | change | #518 | implemented | verified | smoke | `cargo test -p vat --test vat_sandbox_egress -- --nocapture` |
| Sandbox applied to runner-mode commands | change | #527 | implemented | verified | smoke | `cargo test -p vat --test vat_runner_sandbox -- --nocapture` |
| Full-hermetic http-mock no-forward mode | change | #530 | implemented | verified | smoke | `cargo test -p vat --test vat_emulator_httpmock_hermetic -- --nocapture` |
| Copy-on-write fork and snapshot lifecycle | epic | - | implemented | verified | smoke | `rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS projects/vat/README.md` |
| Resource isolation boundary | epic | - | implemented | verified | smoke | `rg -n -e sandbox -e isolation -e seatbelt projects/vat/README.md projects/vat/src/sandbox` |


## AW Verification Snapshot

| Field | Value |
|---|---|
| Last verified | 2026-06-20 |
| Production readiness | ready |
| Tech design root | `projects/vat/tech-design` |
| TD lock | `projects/vat/tech-design/td.lock` |
| External-contract inventory | `projects/vat/tests/aw-ec.toml` |
| Source ownership | full codegen, 100.0% (65/65) |
| Semantic coverage | 100.0% |
| Traceability coverage | 95.6% |
| External-contract gate | passed, 6/6 |
| Test gate | `cargo test -p vat` passed |
| Health gate | `aw health vat --verify-traceability --verify-cb --verify-cold --verify-tests --verify-ec` |

## What vat is *not*

- **Not a VM, not a Linux-container emulator.** v1 runs host processes. That's
  the GPU win; it's also the limit — you get the *host* OS, not a clean Linux
  userland. A Linux-namespaces backend (and, if ever needed, a VM backend that
  trades the GPU away) slot in behind the same [`Sandbox`] trait.
- **Not a resource scheduler.** vat owns resource isolation: copy-on-write
  workspaces, sandbox backends, and agent-readable state. It does not decide
  admission, throttling, pausing, or kill policy. That is cap's job. Compose
  them explicitly when scheduling is needed, for example
  `cap run --label "vat train" -- vat run -- python train.py`.
- **Not a long-lived process manager.** Services in `vat.toml` are dependencies
  of one runner invocation. vat starts them, waits for readiness, runs the
  runner, captures evidence, and terminates them. Standalone `vat cluster`
  clusters outlive a run as a convenience, but vat does not *supervise* them (no
  daemon, no restart, no health monitoring) — it creates/lists/deletes/reports
  only on explicit command, exactly like kind/k3d/minikube do.
- **Not an image registry / build system.** No Dockerfile, and vat builds no
  images. A vat's environment is a declarative [`EnvSpec`](src/spec.rs) an agent
  reads and rewrites. A `vat.toml` *service* may run as an ephemeral `docker run`
  container (a `preset` with `runtime = "docker"`, or an explicit `image`), but
  the runner is always a host process — vat never containerizes your workload.

## Quick start

```bash
projects/vat/build.sh debug         # build + install ~/.cargo/bin/vat

# run a command in a fresh copy-on-write clone of the current dir
vat run -- python train.py

# run the default local test protocol from vat.toml
vat run
vat logs <id> runner

# give an LLM/tool agent the compact vat usage contract
vat llm

# what GPU can my vats see? (the headline claim, in one command)
vat gpu
#   vendor   apple
#   chip     Apple M1 Pro
#   backends metal, mps, mlx
#   status   ✓ accessible

# what happened / what changed — one JSON doc, for an agent
vat state <id>
vat diff  <id>

# branch a running environment, git-style
vat fork <id>          # new runnable working copy
vat snapshot <id>      # frozen restore point
```

## The model

A **vat** =
copy-on-write workspace ([`overlay`](src/overlay.rs))
+ declarative [`EnvSpec`](src/spec.rs)
+ append-only [`event`](src/event.rs) log
+ projected [`VatState`](src/state.rs).

`vat run` clones a base (a host dir, or another vat via `--from`) into a fresh
rootfs, runs your command in the chosen [`sandbox`](src/sandbox/) backend with
live stdio, then records the run and recomputes the filesystem diff. Because
clones are APFS `clonefile(2)` (near-instant, block-shared until written),
fork/snapshot are cheap — an agent can try two approaches, fail, and roll back
without rebuilding.

Vat state is repo-local by default: the store root is `<repo>/.vat` (ignored by
git). Set `VAT_HOME` only when an external runner intentionally wants a
different store root.

### vat state

The command an agent calls to understand a vat. One document, no log-scraping:

```jsonc
{
  "id": "vat-5oyh3vc",
  "status": { "state": "exited", "code": 0 },
  "spec":   { "isolation": "none", "gpu": "auto", ... },
  "lineage": ["vat-..."],            // the fork tree this vat sits in
  "last_run": { "command": [...], "exit_code": 0, "duration_ms": 30 },
  "workspace": { "rootfs": "...", "file_count": 12, "size_bytes": 4096 },
  "changes": { "added": 1, "deleted": 1, "sample_added": ["made.txt"], ... },
  "gpu": { "chip": "Apple M1 Pro", "accessible": true,
           "backends": ["metal","mps","mlx"] },
  "events_tail": [ ... ]
}
```

## CLI

| Verb | Purpose |
|------|---------|
| `vat run` | Load `vat.toml`, select `default_runner` or the only runner, emit sparse JSONL checkpoints, run setup/services/readiness/runner, capture logs/artifacts/diff/state, and cleanup. |
| `vat run <runner-id>` | Run a specific `vat.toml` runner. |
| `vat run --scenario <id>` | Run a named app-under-test scenario: app service + scenario deps + runner deps, with topology evidence in `vat state`. |
| `vat run --keep always\|failed\|never [runner-id]` | Override `[workspace].keep` for one configured run, e.g. retain logs for a passing probe without editing `vat.toml`. |
| `vat run -- <cmd>` | Clone a base, run one direct command, record the result. `--base DIR`, `--from VAT`, `--isolation none\|seatbelt`, `--gpu auto\|required\|none`, `--json`. |
| `vat llm [--topic <t>] [--format md\|json]` | Print offline agent-facing docs. Default `outline`; use `--topic guide` for the detailed vat.toml/service/evidence/boundary guide. |
| `vat upgrade` | Self-update to the latest `vat@*` GitHub release (`--check` to report only, `--version <tag>` to pin). One of the three mandatory CLI-convention verbs (`llm`/`upgrade`/`issue`), via the shared `cli-std` crate. |
| `vat issue search\|view\|create` | Search, read, and file diagnostics-rich GitHub issues under `project:vat`; `issue create --dry-run --title <t>` previews version + target + OS/arch diagnostics without submitting. |
| `vat ls` | List vats (one line each, or `--json` array of full states). |
| `vat state <id>` | Full agent-legible state as JSON (`--compact` for one line). |
| `vat diff <id>` | Every filesystem change vs. the vat's base (`--json`). |
| `vat logs <id> [service-id\|runner]` | Print captured logs from a retained vat.toml runner invocation. |
| `vat fork <id>` | Copy-on-write a new **runnable** working copy. |
| `vat snapshot <id>` | Copy-on-write a **frozen** restore point. |
| `vat rm <id>` | Delete a vat and its workspace. |
| `vat gpu` | Report the GPU every vat on this host can reach. |
| `vat cluster create\|ls\|delete\|kubeconfig` | Manage standalone local Kubernetes clusters (kind/k3d/minikube), independent of a run. |

## vat.toml

`vat.toml` is the project-local protocol an agent edits when it needs vat to
prepare and run a real local test environment:

```toml
version = 1
name = "local-e2e"
default_runner = "e2e"

[workspace]
base = "."
workdir = "."
keep = "failed" # failed | always | never

[env]
NODE_ENV = "test"

[[setup]]
id = "install"
cmd = ["pnpm", "install", "--frozen-lockfile"]
when = "missing:node_modules/.modules.yaml"

[[services]]
id = "pg"
preset = "postgres"        # native binary preferred, Docker image fallback
# runtime = "auto"         # auto (default) | native | docker
seed = ["schema.sql", "fixtures.sql"]
export = { DATABASE_URL = "DATABASE_URL" }

[[services]]
id = "alloy"               # Docker-only dependency (no native binary)
image = "google/alloydbomni:latest"
container_port = 5432
image_env = { POSTGRES_PASSWORD = "pw" }
export = { ALLOY_URL = "postgres://postgres:pw@{host}:{port}/postgres" }

[[services]]
id = "ci-pg"               # already started by GitLab CI services / Docker Compose
external = { host = "postgres", port = 5432 }
export = { DATABASE_URL = "postgres://postgres@{host}:{port}/app" }

[[services]]
id = "k8s"                 # ephemeral local Kubernetes cluster
cluster = "auto"           # auto (kind→k3d→minikube) | kind | k3d | minikube
# k8s_version = "1.30"
# nodes = 1
export = { KUBECONFIG = "{kubeconfig}" }

[[services]]
id = "web"                 # app under test; {port} is auto-allocated
cmd = ["pnpm", "run", "dev", "--", "--host", "127.0.0.1", "--port", "{port}"]
ready_http = "http://127.0.0.1:{port}/"
export = { APP_URL = "APP_URL" }
timeout_s = 30

[[services]]
id = "http"
preset = "http-mock"       # required for hermetic scenario routing/no-forward

[[runners]]
id = "e2e"
requires = ["pg", "k8s"]
cmd = ["pnpm", "run", "test:e2e"]
timeout_s = 300
artifacts = ["test-results/**", "playwright-report/**"]

[[scenarios]]
id = "prod-like"
app = "web"
requires = ["pg", "k8s", "http"]
runner = "e2e"
network = "hermetic"       # open | hermetic
```

`[[scenarios]]` is the production-like path: the app-under-test service starts
with its declared dependencies, runner dependencies are deduped into the same
service set, and `vat state <id>` records `test_run.scenario` with app, runner,
services, routes, and whether hermetic mode was active. `network = "hermetic"`
requires a participating `preset = "http-mock"` service, sets localhost-only
egress, defaults the run to seatbelt isolation, runs direct-start services under
that sandbox, and starts the proxy in no-forward mode. Docker/image, cluster, and
native preset service backends remain external local services; the app and runner
are still host processes, not containers or VMs.

A service is provided in one of five ways, and **native (Homebrew) is
preferred**:

- `preset` — a built-in service. With the default `runtime = "auto"` vat uses
  the native binary when it is installed and falls back to the preset's official
  Docker image when it is not; `runtime = "native"` / `"docker"` force one path.
  Datastore/broker presets: `postgres`, `redis`, `nats`, `rabbitmq`, `mysql`,
  `mongo`.
- `preset` (built-in Rust emulators) — `pubsub`, `firebase-auth`, `cloud-tasks`,
  `cloud-scheduler`, and `cloud-workflows` run vat's **own** in-process emulator
  under `runtime = auto`: pure Rust, instant start, **no gcloud / Java /
  firebase-tools / Docker**. `pubsub` is a google.pubsub.v1 gRPC server
  (topics/subscriptions, Publish, Pull, StreamingPull, Acknowledge);
  `firebase-auth` is a Firebase Auth (Identity Toolkit) REST server;
  `cloud-tasks` serves **both the Cloud Tasks v2 gRPC service and the v2 REST API
  on one port** and delivers each task's httpRequest to its target at scheduleTime
  (or `tasks/{t}:run`); `cloud-scheduler` likewise serves **gRPC + v1 REST** and
  fires a job's httpTarget on its cron schedule (or `jobs/{j}:run`); `cloud-workflows` is a
  Cloud Workflows v1 REST server (createWorkflow → createExecution →
  getExecution) running a **subset Workflows interpreter** (assign / call http.* /
  switch / for / try-retry-except / subworkflow + `${...}` expressions) whose
  `call: http.*` steps **orchestrate the other emulators** or any HTTP endpoint;
  `cloud-storage` is a GCS JSON API v1 server over an in-memory object store
  (bucket CRUD, media/multipart upload, `alt=media` download, list with prefix,
  delete; reports size + md5Hash); `http-mock` is a **transparent HTTP stub +
  record/replay proxy with HTTPS MITM** — the mock-killer for third-party APIs.
  Each exports its host var (`PUBSUB_EMULATOR_HOST`, `FIREBASE_AUTH_EMULATOR_HOST`,
  `CLOUD_TASKS_EMULATOR_HOST`, `CLOUD_SCHEDULER_EMULATOR_HOST`,
  `CLOUD_WORKFLOWS_EMULATOR_HOST`, `STORAGE_EMULATOR_HOST` — point your client's
  base URL at `http://$HOST` for host:port vars; `STORAGE_EMULATOR_HOST` is
  exported as `http://127.0.0.1:<port>` because the GCS REST SDK expects a
  schemed endpoint). `http-mock` instead exports `HTTP(S)_PROXY` + a CA-trust bundle
  (`SSL_CERT_FILE`, `NODE_EXTRA_CA_CERTS`, `REQUESTS_CA_BUNDLE`, …) so the runner's
  outbound HTTP/HTTPS — even hardcoded `https://api.example.com` — is intercepted
  with **no app code change**: register stubs at `$VAT_HTTP_MOCK_HOST/__admin/stubs`,
  and unstubbed calls record to a cassette once then replay offline forever.
  `openapi` (`preset = "openapi"`, `spec = "api.yaml"`) reads an **OpenAPI
  document and serves spec-derived responses** (the response `example`, else a
  schema-synthesized body; path templating like `/users/{id}` and `$ref`) — a
  working fake of a documented API with no stubs or recording. It runs standalone
  (point your base URL at `$OPENAPI_MOCK_HOST`) and also backs the http-mock proxy:
  `POST $VAT_HTTP_MOCK_HOST/__admin/openapi` registers a spec for a host, so a
  proxied `https://` call is answered from the contract (resolution order **stub >
  openapi > cassette > forward**). `pubsub` still accepts `runtime = native`
  (gcloud) / `runtime = docker` (the cloud-cli image) as a full-fidelity fallback;
  the others are built-in only (no official emulator exists). The async emulator
  stack sits behind a default-on `emulator` Cargo feature (`--no-default-features`
  drops it). **Wiring a `cloud-tasks` / `cloud-scheduler` client:** these SDKs don't
  read `CLOUD_TASKS_EMULATOR_HOST` / `CLOUD_SCHEDULER_EMULATOR_HOST` (Google ships no
  emulator). Since the emulators now serve **both gRPC and REST**, point the stock
  gRPC client at the host var with an insecure endpoint override (Python:
  `CloudTasksClient(client_options={"api_endpoint": host})`), or use `transport="rest"`
  + `http://$HOST`, or POST the v2 REST API directly. For **zero app config**, add an
  `http-mock` service + a `[network]` route (see *Network sandbox* below): vat then
  transparently routes the real `cloudtasks.googleapis.com` host — REST *and* gRPC —
  to the local emulator.
  ```toml
  [[services]]
  id = "ps"
  preset = "pubsub"          # built-in gRPC emulator → PUBSUB_EMULATOR_HOST
  ```
- `preset` (external emulators) — `firestore`, `datastore`, `bigtable`,
  `spanner` wrap the GCP `gcloud beta emulators` family. Native needs gcloud +
  Java + the gcloud component; `runtime = auto` falls back to the cloud-cli
  Docker image (Spanner uses its own image) when the component is missing.
  Each exports the well-known host var (e.g. `FIRESTORE_EMULATOR_HOST`).
  `preset = "firebase"` is the Firebase Emulator Suite bundle: it requires a
  `firebase.json`, runs `firebase emulators:start`, and exports each configured
  emulator's `*_EMULATOR_HOST` (native-only — no Docker fallback).
- `image` — a Docker-only dependency that has no native equivalent (e.g.
  AlloyDB). Requires `container_port`; `image_env` is passed into the container;
  in `export`, `{host}`/`{port}` resolve to the mapped host endpoint and
  `VAT_SERVICE_<ID>_{HOST,PORT}` are always exported.
- `external` — an already provisioned endpoint, such as a GitLab CI `services:`
  sidecar, GitHub Actions service container, local Docker Compose service, or
  host daemon. vat does not start or stop it; it waits for readiness, substitutes
  `{host}`/`{port}` in `ready_http`, `ready_cmd`, and `export`, injects
  `VAT_SERVICE_<ID>_{HOST,PORT}`, and records `owned_by_vat = false` in
  `vat state`.
- `cluster` — an ephemeral local Kubernetes cluster, for testing K8s-native
  targets. `auto` picks the first installed of kind → k3d → minikube (all need
  Docker on Apple Silicon); `kind`/`k3d`/`minikube` force one. Optional
  `k8s_version` and `nodes`. vat creates the cluster before the runner with an
  isolated kubeconfig (it never touches `~/.kube/config`), exports `KUBECONFIG`
  (the `{kubeconfig}` token) and `VAT_SERVICE_<ID>_KUBECONFIG`, probes readiness
  with `kubectl get nodes`, and deletes it at teardown per the `keep` policy. A
  missing backend fails with a structured `cluster_backend_unavailable` error
  (never a panic). `vat cluster` manages clusters standalone, outside a run.
- `cmd` — an explicit native command.

Env export contract:

| Service backing | Default exports | `export` map semantics | Raw service vars |
|---|---|---|---|
| `preset` datastore/broker | postgres/mysql → `DATABASE_URL`; redis → `REDIS_URL`; nats → `NATS_URL`; rabbitmq → `AMQP_URL`; mongo → `MONGODB_URI`; opensearch → `OPENSEARCH_URL` | Value containing `{host}`/`{port}` uses the map key as the env var name; otherwise the value is a legacy alias name receiving the default URL. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
| `preset` built-in emulator | `PUBSUB_EMULATOR_HOST`, `FIREBASE_AUTH_EMULATOR_HOST`, `CLOUD_TASKS_EMULATOR_HOST`, `CLOUD_SCHEDULER_EMULATOR_HOST`, `CLOUD_WORKFLOWS_EMULATOR_HOST`, `STORAGE_EMULATOR_HOST`, `VAT_HTTP_MOCK_HOST`, or `OPENAPI_MOCK_HOST` | Same template/alias rule as other presets. `STORAGE_EMULATOR_HOST` includes `http://`; the others are host:port unless documented by the service. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
| `image` | none | Key is always the env var name; value may use `{host}`/`{port}`. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
| `external` | none | Key is always the env var name; value may use `{host}`/`{port}` from the attached endpoint. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT`; state records `owned_by_vat = false` |
| `cmd` | `VAT_SERVICE_<ID>_URL` when `ready_http` exists and no custom export is set | Value containing `{host}`/`{port}` uses the map key as the env var name; otherwise the value aliases `ready_http`. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` only when the command needs/allocates a port |
| `cluster` | `KUBECONFIG` | `{kubeconfig}` expands to the isolated kubeconfig path. | `VAT_SERVICE_<ID>_KUBECONFIG` |

Runner scripts can detect a configured vat run with `VAT_WORKSPACE_BASE`; it is
set for `vat.toml` runner and scenario modes and points at the source workspace
that vat cloned.

For the native path vat checks for required binaries, cold-prepares cached
service data when needed, and clones it on later runs. For the Docker path it
runs an ephemeral `docker run --rm` container bound to loopback, removed at
teardown. For the `external` path vat treats the surrounding environment as the
lifecycle owner and only records/probes the endpoint. The **runner itself is
never containerized**, so the host GPU is untouched. Managed paths auto-allocate
ports, every path exports runner env vars, and vat reports only a few JSONL
checkpoints unless the agent asks for logs/state/diff.
A Docker-backed service with no reachable daemon fails with a structured
`docker_unavailable` error rather than a panic.

On macOS, connection-heavy runners against native TCP presets can hit the host
accept-backlog ceiling (`kern.ipc.somaxconn`, often 128) and see intermittent
`ECONNREFUSED` even while Redis/Postgres/etc. are still running. vat surfaces the
Redis startup warning as a structured `hint` event. Prefer connection pooling in
the app under test, or raise the host limit for the session, for example
`sudo sysctl -w kern.ipc.somaxconn=1024`, then rerun vat.

## Network sandbox

An optional `[network]` block turns a run into a confined, hermetic environment —
on macOS with **no VM** (Apple Seatbelt + the http-mock proxy), so the host GPU
stays untouched.

```toml
[network]
egress = "localhost-only"   # open (default) | localhost-only | deny

# Transparent service routing: a real host → a local target. Auto-derived for
# declared GCP emulator presets, so you usually don't write these by hand.
[[network.routes]]
host = "cloudtasks.googleapis.com"
target = "http://127.0.0.1:8123"   # or a local emulator's host:port
```

- **Transparent routing** (`[network].routes`, needs an `http-mock` service):
  an outbound request to a known host is served by a local emulator/mock instead
  of the real service, with **zero app code change**. Works for **HTTP/REST**
  (resolution `route > stub > openapi > cassette > forward`) **and gRPC** (the
  CONNECT MITM negotiates ALPN h2 and stream-reverse-proxies routed gRPC, trailers
  preserved, to the emulator's h2c port). Declaring a GCP emulator preset
  (`cloud-tasks`, `cloud-scheduler`, …) plus an `http-mock` service auto-adds the
  route from its real `*.googleapis.com` host to the local emulator.
- **Egress policy** (`[network].egress`, enforced under `--isolation seatbelt`):
  `localhost-only` denies outbound network except loopback (so the run reaches
  only vat's local emulators/proxy); `deny` blocks all outbound; `open` (default)
  is unrestricted. Reads stay open and the GPU is untouched. Applies to both
  direct (`vat run -- cmd`) and runner (`vat run <runner>`) commands. Regular
  runner-mode services keep their network; `vat run --scenario <id>` with
  `network = "hermetic"` also wraps direct-start app/dependency services while
  leaving Docker/image, cluster, and preset service backends on their native
  local-service path. With `--isolation none` a non-`open` policy warns that
  confinement needs seatbelt.
- **Fully hermetic**: when `egress` is `localhost-only`/`deny`, vat also runs the
  `http-mock` proxy in **no-forward** mode — an unmatched request returns
  `502 hermetic: … forwarding disabled` instead of reaching the internet. Net:
  the runner is confined to localhost *and* the proxy refuses upstream, so the run
  is fail-closed (routes/stubs/OpenAPI/cassette-replays still serve).

> Seatbelt enforcement uses `sandbox-exec` (Apple-deprecated but functional; the
> [`Sandbox`] trait keeps a future Endpoint Security backend local). Routing/egress
> only catch proxy-honoring / loopback-confined clients — non-cooperating egress is
> *blocked* (fail-closed), not transparently rerouted.

[`Sandbox`]: src/sandbox/mod.rs
