// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-commands.md#schema
// CODEGEN-BEGIN
//! `vat llm` — compact agent-facing usage contract.

use std::process::ExitCode;

use anyhow::Result;

/// Stable guide text intended for LLM/tool agents.
/// @spec projects/vat/tech-design/logic/llm-agent-usage-guide.md#cli
const GUIDE: &str = r#"# vat LLM Guide

vat is a local, ephemeral agent test runner. Use it to prepare a real local
workspace, run one command or one named vat.toml runner, and inspect structured
evidence afterward.

## First Choice

- If the project has `vat.toml`, prefer plain `vat run`.
- Use `vat run <runner-id>` only when you need a non-default runner.
- If you only need one ad-hoc command, use `vat run -- <command>`.
- `vat run` prints sparse JSONL checkpoints; the final line has
  `"type":"result"`.
- After a retained run, inspect `vat state <id>`, `vat diff <id>`, and
  `vat logs <id> [runner|service-id]`.
- Use `vat --help` for flag syntax and `vat <command> --help` for command flags.

## vat.toml Contract

```toml
version = 1
default_runner = "e2e"

[workspace]
base = "."
workdir = "."
keep = "failed" # failed | always | never

[[services]]
id = "pg"
preset = "postgres"        # native binary preferred; Docker image fallback
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

[[services]]
id = "fs"                  # GCP Firestore emulator (exports FIRESTORE_EMULATOR_HOST)
preset = "firestore"       # firestore | pubsub | datastore | bigtable | spanner | firebase

[[runners]]
id = "e2e"
requires = ["pg"]
cmd = ["pnpm", "run", "test:e2e"]
artifacts = ["test-results/**", "playwright-report/**"]
```

## Services: native or Docker

- A `preset` service prefers the native Homebrew binary and falls back to the
  preset's official Docker image when the binary is missing. Force it with
  `runtime = "native"` or `runtime = "docker"`. Datastore/broker presets:
  postgres, redis, nats, rabbitmq, mysql, mongo.
- Emulator presets: `firestore`, `pubsub`, `datastore`, `bigtable`, `spanner`
  wrap the GCP `gcloud beta emulators` family (native needs gcloud + Java + the
  gcloud component; `runtime = auto` falls back to the cloud-cli Docker image —
  Spanner uses its own image — when the component is missing). Each exports the
  well-known host var (`FIRESTORE_EMULATOR_HOST`, `PUBSUB_EMULATOR_HOST`,
  `DATASTORE_EMULATOR_HOST`, `BIGTABLE_EMULATOR_HOST`, `SPANNER_EMULATOR_HOST`).
- `preset = "firebase"` is the Firebase Emulator Suite bundle: it requires a
  `firebase.json` in the workspace, runs `firebase emulators:start`, and exports
  each configured emulator's host var (`FIRESTORE_EMULATOR_HOST`,
  `FIREBASE_AUTH_EMULATOR_HOST`, `FIREBASE_DATABASE_EMULATOR_HOST`,
  `FIREBASE_STORAGE_EMULATOR_HOST`, `PUBSUB_EMULATOR_HOST`,
  `FIREBASE_EMULATOR_HUB`). It is native-only (firebase-tools + Java); there is
  no Docker fallback for firebase.
- An `image` service is always a Docker container — use it for dependencies with
  no native binary (e.g. AlloyDB). It requires `container_port`; `image_env` is
  passed into the container; in `export`, `{host}`/`{port}` resolve to the mapped
  host endpoint, and `VAT_SERVICE_<ID>_{HOST,PORT}` are always exported.
- Built-in emulators: `preset = "pubsub"`, `"firebase-auth"`, `"cloud-tasks"`,
  `"cloud-scheduler"`, `"cloud-workflows"`, and `"cloud-storage"` run vat's OWN
  in-process Rust emulator under `runtime = auto` — no gcloud, Java,
  firebase-tools, or Docker, and instant start. They export
  `PUBSUB_EMULATOR_HOST` / `FIREBASE_AUTH_EMULATOR_HOST` /
  `CLOUD_TASKS_EMULATOR_HOST` / `CLOUD_SCHEDULER_EMULATOR_HOST` /
  `CLOUD_WORKFLOWS_EMULATOR_HOST` / `STORAGE_EMULATOR_HOST` — point your client's
  base URL at `http://$HOST` (the GCS SDKs read `STORAGE_EMULATOR_HOST`
  automatically). `cloud-tasks` (v2 REST) delivers each task's httpRequest to its
  target at scheduleTime (or `tasks/{t}:run`); `cloud-scheduler` (v1 REST) fires
  a job's httpTarget on its cron schedule or `jobs/{j}:run`; `cloud-workflows`
  (v1 REST) runs a subset Workflows interpreter whose `call: http.*` steps can
  orchestrate the other emulators; `cloud-storage` (GCS JSON API v1) is an
  in-memory object store (bucket CRUD, media/multipart upload, `alt=media`
  download, list, delete). `pubsub` still accepts `runtime = native` (gcloud) /
  `runtime = docker` (image) as a fidelity fallback; the others are built-in
  only (no official emulator exists).
- A `cluster` service spins up an ephemeral local Kubernetes cluster (kind, k3d,
  or minikube; `auto` picks the first installed). vat creates it before the
  runner, exports `KUBECONFIG` (the `{kubeconfig}` token) plus
  `VAT_SERVICE_<ID>_KUBECONFIG`, probes readiness with `kubectl get nodes`, and
  deletes it at teardown per the `keep` policy. With no backend it emits a
  structured `cluster_backend_unavailable` error (no panic). All backends need
  Docker on Apple Silicon.
- Docker-backed services need a reachable Docker daemon; vat emits a structured
  `docker_unavailable` error (no panic) when it is missing. The runner itself is
  never containerized.

## Command Patterns

- `vat run`: select the default runner, prepare or clone service images, start
  required services, wait for readiness, run the runner, capture evidence, stop
  services, and return the runner exit code.
- `vat run e2e`: explicitly run the `e2e` runner.
- `vat run -- cargo test -p app`: run one direct command without requiring
  vat.toml; the child exit code is forwarded.
- `vat logs <id> runner`: print retained runner stdout/stderr.
- `vat logs <id> <service-id>`: print retained service stdout/stderr.
- `vat state <id>`: read the agent-legible JSON state.
- `vat diff <id> --json`: read filesystem changes vs. the vat base.
- `vat cluster create [--backend auto|kind|k3d|minikube] [--name N]`: create a
  standalone local Kubernetes cluster (outlives a run); `vat cluster ls --json`,
  `vat cluster kubeconfig <name>`, and `vat cluster delete <name>` manage it.

## Retention

Default `keep = "failed"` means successful configured runs clean up after
emitting JSON, while failed runs keep workspace state and logs for inspection.

## Boundaries

- vat is not a Docker/OCI/Compose replacement, a Linux runtime, a VM, a daemon,
  or a long-lived process manager. It builds no images and ships none.
- The runner is always a host process (never containerized) — the GPU story.
  Docker is only an option for run-scoped dependency *services*.
- Services in `vat.toml` are run-scoped dependencies of one runner invocation;
  containers are ephemeral (`docker run --rm`) and removed at teardown.
- vat does not schedule production work or manage restart policy.
- Standalone `vat cluster` clusters outlive a run as a convenience, but vat does
  not supervise them (no daemon, no restart, no health monitoring) — it only
  creates/lists/deletes/reports on explicit command, like kind/k3d themselves.
"#;

/// @spec projects/vat/tech-design/logic/llm-agent-usage-guide.md#cli
pub fn exec() -> Result<ExitCode> {
    print!("{GUIDE}");
    Ok(ExitCode::SUCCESS)
}
// CODEGEN-END
