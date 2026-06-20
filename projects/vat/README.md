# vat — local agent test runner capsules

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

## Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Agent-Native GPU-Native Dev Containers | - | implemented | verified | smoke | ready | vat runs sandboxed host-process environments over copy-on-write workspaces so coding and ML agents get structured state, local test runner evidence, fork/snapshot, ephemeral local Kubernetes clusters (kind/k3d/minikube), and host GPU access without a VM. |

## AW Verification Snapshot

| Field | Value |
|---|---|
| Last verified | 2026-06-20 |
| Production readiness | ready |
| Tech design root | `projects/vat/tech-design` |
| TD lock | `projects/vat/tech-design/td.lock` |
| External-contract inventory | `projects/vat/tests/aw-ec.toml` |
| Source ownership | full codegen, 100.0% (39/39) |
| Semantic coverage | 100.0% |
| Traceability coverage | 94.7% |
| External-contract gate | passed, 6/6 |
| Test gate | `cargo test -p vat` passed |
| Health gate | `aw health vat --verify-traceability --verify-cb --verify-cold --verify-tests --verify-ec` |

## Agent-Native GPU-Native Dev Containers

| Field | Value |
|---|---|
| ID | agent-native-gpu-native-dev-containers |
| Root WI | - |
| Status | verified |
| Promise | vat runs sandboxed host-process environments over copy-on-write workspaces so coding and ML agents get structured state, local test runner evidence, fork/snapshot, ephemeral local Kubernetes clusters (kind/k3d/minikube), and host GPU access without a VM. |
| Required Verification | smoke |
| Gate Inventory | `cargo test -p vat`; `rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured projects/vat/README.md`; `rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal projects/vat/README.md projects/vat/src/gpu.rs`; `rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS projects/vat/README.md` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Host-process execution and GPU visibility | epic | - | implemented | verified | smoke | `rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal projects/vat/README.md projects/vat/src/gpu.rs` |
| Agent-legible state and diff surface | epic | - | implemented | verified | smoke | `rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured projects/vat/README.md` |
| Local agent test runner protocol | epic | #4152 | implemented | verified | smoke | `cargo test -p vat vat_toml_runner -- --nocapture` |
| Local Kubernetes cluster service and `vat cluster` | change | #141 | implemented | verified | smoke | `cargo test -p vat --test vat_cluster -- --nocapture` |
| Copy-on-write fork and snapshot lifecycle | epic | - | implemented | verified | smoke | `rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS projects/vat/README.md` |
| Resource isolation boundary | epic | - | implemented | verified | smoke | `rg -n -e sandbox -e isolation -e seatbelt projects/vat/README.md projects/vat/src/sandbox` |

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
| `vat run -- <cmd>` | Clone a base, run one direct command, record the result. `--base DIR`, `--from VAT`, `--isolation none\|seatbelt`, `--gpu auto\|required\|none`, `--json`. |
| `vat llm` | Print the compact LLM/agent usage guide: when to use `vat.toml`, direct runs, evidence commands, retention, and non-Docker boundaries. |
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
id = "k8s"                 # ephemeral local Kubernetes cluster
cluster = "auto"           # auto (kind→k3d→minikube) | kind | k3d | minikube
# k8s_version = "1.30"
# nodes = 1
export = { KUBECONFIG = "{kubeconfig}" }

[[runners]]
id = "e2e"
requires = ["pg", "k8s"]
cmd = ["pnpm", "run", "test:e2e"]
timeout_s = 300
artifacts = ["test-results/**", "playwright-report/**"]
```

A service is provided in one of four ways, and **native (Homebrew) is
preferred**:

- `preset` — a built-in service. With the default `runtime = "auto"` vat uses
  the native binary when it is installed and falls back to the preset's official
  Docker image when it is not; `runtime = "native"` / `"docker"` force one path.
  Datastore/broker presets: `postgres`, `redis`, `nats`, `rabbitmq`, `mysql`,
  `mongo`.
- `preset` (emulators) — `firestore`, `pubsub`, `datastore`, `bigtable`,
  `spanner` wrap the GCP `gcloud beta emulators` family. Native needs gcloud +
  Java + the gcloud component; `runtime = auto` falls back to the cloud-cli
  Docker image (Spanner uses its own image) when the component is missing.
  Each exports the well-known host var (e.g. `FIRESTORE_EMULATOR_HOST`,
  `PUBSUB_EMULATOR_HOST`). `preset = "firebase"` is the Firebase Emulator Suite
  bundle: it requires a `firebase.json`, runs `firebase emulators:start`, and
  exports each configured emulator's `*_EMULATOR_HOST` (native-only — no Docker
  fallback). Example:
  ```toml
  [[services]]
  id = "fb"
  preset = "firebase"        # reads ./firebase.json, exports *_EMULATOR_HOST
  ```
- `image` — a Docker-only dependency that has no native equivalent (e.g.
  AlloyDB). Requires `container_port`; `image_env` is passed into the container;
  in `export`, `{host}`/`{port}` resolve to the mapped host endpoint and
  `VAT_SERVICE_<ID>_{HOST,PORT}` are always exported.
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

For the native path vat checks for required binaries, cold-prepares cached
service data when needed, and clones it on later runs. For the Docker path it
runs an ephemeral `docker run --rm` container bound to loopback, removed at
teardown — the **runner itself is never containerized**, so the host GPU is
untouched. Either way vat auto-allocates ports, exports runner env vars, and
reports only a few JSONL checkpoints unless the agent asks for logs/state/diff.
A Docker-backed service with no reachable daemon fails with a structured
`docker_unavailable` error rather than a panic.

[`Sandbox`]: src/sandbox/mod.rs
