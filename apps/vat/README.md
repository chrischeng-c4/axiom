# vat — local agent test runner capsules

## Brief

`vat` is a headless local development test runner for the one operator Docker
was never designed for: a **coding/ML agent**. GUI and Desktop surfaces are
permanently out of scope; agents use the CLI and structured output. vat is not a
long-lived process manager. An agent writes `vat.toml`; vat prepares an
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
| Agent-Native GPU-Native Dev Containers | #4152 | implemented | verified | smoke | ready | vat runs sandboxed host-process environments over copy-on-write workspaces so coding and ML agents get structured state, local test runner evidence, fork/snapshot, Docker-backed local Kubernetes clusters (kind/k3d/minikube), and a separately bounded one-boot Apple Container K3s session, plus GCP/Firebase emulators and host GPU access without a VM. |
| Developer & Agent Experience | #1819 | in_progress | partial | smoke | blocked | Offline command contracts, task-scoped onboarding, and host preflight evidence for local agents. |

### Agent-Native GPU-Native Dev Containers

ID: agent-native-gpu-native-dev-containers
Type: AgentFirst
Surfaces: CLI: `vat run` + `vat emulator` + `vat state/diff/fork/snapshot` - Agent-facing dev-container CLI: copy-on-write run + structured state/diff, fork/snapshot, built-in GCP/Firebase emulators (REST+gRPC), and the network sandbox (routes/egress/hermetic).
EC Dimensions: behavior: `cargo test -p vat` - vat.toml run protocol, built-in emulators (REST + gRPC), transparent routing, and seatbelt egress/hermetic conformance.
Root WI: #4152
Status: verified
Required Verification: smoke
Promise:
vat runs sandboxed host-process environments over copy-on-write workspaces so coding and ML agents get structured state, local test runner evidence, fork/snapshot, Docker-backed local Kubernetes clusters (kind/k3d/minikube), and a separately bounded one-boot Apple Container K3s session, plus GCP/Firebase emulators and host GPU access without a VM.
Gate Inventory:
- `cargo test -p vat`; `rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured apps/vat/README.md`; `rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal apps/vat/README.md apps/vat/src/gpu.rs`; `rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS apps/vat/README.md`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Host-process execution and GPU visibility | epic | - | implemented | verified | smoke | `rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal apps/vat/README.md apps/vat/src/gpu.rs` |
| Agent-legible state and diff surface | epic | - | implemented | verified | smoke | `rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured apps/vat/README.md` |
| Local agent test runner protocol | epic | #4152 | implemented | verified | smoke | `cargo test -p vat vat_toml_runner -- --nocapture` |
| Interrupt-safe owned process cleanup | change | #2394 | implemented | verified | smoke | `cargo test -p vat --test vat_signal_cleanup -- --test-threads=1` proves real SIGINT/SIGTERM cleanup for configured and direct runs. |
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
| Sandbox egress policy fails closed when isolation cannot enforce it | change | #1300 | implemented | verified | smoke | `cargo test -p vat --test vat_sandbox_egress_fail_closed -- --nocapture` |
| MicroVm sandbox backend for vat run | change | #1474 | planned | none | none | `cargo test -p vat --test vat_sandbox_microvm --test vat_sandbox_microvm_fail_closed -- --nocapture` |
| vat build: Dockerfile build via container CLI | change | #1479 | planned | none | none | `cargo test -p vat --test vat_build -- --nocapture` |
| vat compose: bounded compose subset, up/down/ps/logs | change | #1484 | planned | none | none | `cargo test -p vat --test vat_compose --test vat_compose_import -- --nocapture` |
| Compose runtime-local build artifacts | change | #1529 | implemented | verified | smoke | `cargo test -p vat --test vat_compose_build -- --nocapture` |
| Headless Docker-command shim over Apple Container | change | #1685 | implemented | verified | conformance | real host/build/dual-service E2E: `RUST_TEST_THREADS=1 VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_compose_host_facing_independent_profile_contract -- --ignored --nocapture` |
| Headless Apple Container K3s one-shot, lease, local-image delivery, and loopback Service port-forward | change | #1693 | implemented | verified | conformance | deterministic fake regression passed, including bounded session-exec lifecycle/marker coverage; independent-kubectl one-shot E2E passed 1/1 (36 filtered, 28.38s), leased E2E passed 1/1 (36 filtered, 29.97s), local-image E2E passed 1/1 (36 filtered, 49.73s), and Service-forward E2E passed 1/1 (36 filtered, 49.57s). Requires an independently installed PATH `kubectl`; VAT rejects OrbStack-provided kubectl. Evidence is bounded to text commands, strict one-document JSON exec with explicit `--timeout 30`, one already-local Apple `alpine:3.20` pod with `imagePullPolicy=Never` and a marker log, and one Service-only loopback JSON tunnel; it does not claim registry-pull generality, persistent Kubernetes, GUI, Docker Engine/API, or OS-sandbox behavior. Gate: `RUST_TEST_THREADS=1 VAT_K8S_LOCAL_IMAGE_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_lease_imports_local_image_without_registry_pull -- --ignored --nocapture` |
| Apple Container k3s local Kubernetes | epic | #1537 | partial | verified | conformance | one-shot, leased, local-image, and Service-forward independent-kubectl real-host E2Es passed; each remains bounded. Phase 0 is a bounded Docker-free path: `vat k8s ephemeral` runs one foreground host command and cleans up, while `vat k8s session create/exec/port-forward/image/status/delete` keeps one running guest and private credentials across explicit agent calls until its bounded lease is deleted or reclaimed. Every K3s command requires an independently installed `kubectl` first on PATH and rejects an OrbStack-provided binary. Persistent/reboot-safe kubeconfig, storage/PVC, ingress/LB, multi-node networking, and `microvm-k3s` remain blocked. |
| Copy-on-write fork and snapshot lifecycle | epic | - | implemented | verified | smoke | `rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS apps/vat/README.md` |
| Resource isolation boundary | epic | - | implemented | verified | smoke | `rg -n -e sandbox -e isolation -e seatbelt apps/vat/README.md apps/vat/src/sandbox` |

### Developer & Agent Experience

ID: developer-agent-experience
Type: AgentFirst
Surfaces: CLI: `vat llm`, `vat --help`, `vat doctor --host-only`, and machine-readable command output.
EC Dimensions: behavior: `cargo test -p vat --test vat_cli_convention --test vat_toml_runner` - offline onboarding, documented command inventory, and configuration-free host preflight.
Root WI: #1819
Status: in_progress
Required Verification: smoke
Promise:
An agent can discover VAT's supported command surface and boundaries offline, select concise task-specific guidance, and inspect the host substrate before a project has a `vat.toml`.
Gate Inventory:
- `cargo test -p vat --test vat_cli_convention -- --nocapture`; `cargo test -p vat --test vat_toml_runner vat_doctor_host_only_needs_no_vat_toml -- --nocapture`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Offline command contract | change | #1817 | implemented | verified | smoke | `cargo test -p vat --test vat_cli_convention documented_agent_commands_match_help -- --nocapture` |
| Agent onboarding topics | change | #1818 | implemented | verified | smoke | `cargo test -p vat --test vat_cli_convention cli_convention_llm_topics_are_task_scoped -- --nocapture` |
| Interactive tooling | n/a | - | n/a — no remote surface | verified | smoke | VAT is a local CLI and intentionally has no GUI, daemon dashboard, or remote control plane. |
| Integration contract | change | #701 | implemented | verified | smoke | `cargo test -p vat --test vat_toml_runner scenario_ -- --nocapture` |
| Configuration-free host preflight | change | #1820 | implemented | verified | smoke | `cargo test -p vat --test vat_toml_runner vat_doctor_host_only_needs_no_vat_toml -- --nocapture` |


## AW Verification Snapshot

| Field | Value |
|---|---|
| Last verified | 2026-06-20 |
| Production readiness | ready |
| Tech design root | `apps/vat/tech-design` |
| TD lock | `apps/vat/tech-design/td.lock` |
| External-contract inventory | `apps/vat/aw.toml` (`aw.ec.generated`) |
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
- **Not a durable Apple Container Kubernetes backend.** `vat k8s ephemeral`
  is a one-boot, single-node K3s session for one foreground host command.
  `vat k8s session` adds a bounded active lease so an agent can make several
  explicit calls with the same private kubeconfig, but it is neither a daemon
  nor restart-safe: lease expiry needs explicit cleanup, Apple machine restart
  is not trusted, and there is no reboot-safe kubeconfig, multi-node,
  storage/PVC, ingress, or load-balancer promise. A bounded active lease can
  run only with an independently installed `kubectl` first on `PATH`; VAT
  rejects an OrbStack-provided `kubectl` before K3s use. This is a concrete
  host-tool provenance requirement, not a GUI or Docker Engine dependency. On
  this host Homebrew `kubernetes-cli` now supplies `/opt/homebrew/bin/kubectl`.
  The independent-kubectl one-shot, leased, local-image, and Service-forward
  E2Es passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s
  respectively. The local-image E2E loaded an already-local Apple `alpine:3.20`
  into one lease, ran a pod with `imagePullPolicy=Never`, observed its marker
  log, then completed exact session cleanup. This is not registry-pull
  generality. All four remain bounded one-guest evidence, not a durable cluster
  promise. It can import one locally inspected `linux/arm64` Apple image and expose one literal
  `service/<name>` only to `127.0.0.1` while one foreground host child runs.
  That temporary tunnel is not arbitrary-resource port-forwarding, a public
  listener, or a background proxy. VAT strips K3s credential variables from the
  child environment, but the child remains a same-UID host process: this is not
  an OS sandbox or an adversarial-child security boundary. The child joins
  kubectl's tracked process group, so ordinary cooperative descendants that do
  not daemonize or escape it are gone before cleanup is confirmed; intentional
  daemonization or group escape is outside this contract. This bounded path is
  useful while a retained `microvm-k3s` backend remains blocked. On a bootstrap
  failure, VAT keeps the root error first, then adds bounded non-sensitive
  installer/guest/machine evidence before the same exact cleanup. That is
  diagnosis only: the existing 300-second bootstrap behavior is unchanged, no
  private kubeconfig/cache or host credential is rendered, and it neither
  retries bootstrap nor reruns `k3s --version` or introduces a wrapper/recovery
  path.
- **Not a GUI or Desktop application — permanently.** vat is operated through
  its CLI and machine-readable output for agents. Do not add graphical controls,
  dashboards, tray/menu-bar UI, or a Desktop lifecycle surface.
- **Not a Docker Engine compatibility endpoint.** vat has an opt-in,
  fail-closed `docker` command shim over Apple Container, installed only with
  `vat docker install-shim --dir <directory-on-PATH>`. It supports the
  documented CLI subset (build/pull/push/run/lifecycle/logs/exec/copy/inspect
  and basic explicit-name image/network/volume commands), rejects unknown flags before
  runtime launch, and requires an explicit host port for `docker run -p`.
  Its Compose support has exactly three named profiles, not general Compose:
  `strict-single-image-v1` is one literal-image service with `up -d`;
  `strict-single-build-v1` is one literal short `build: <context>` service with
  no `image:` and `up -d --build`; and `host-facing-independent-v1` is selected
  only by the exact top-level marker
  `x-vat-compose-profile: host-facing-independent-v1`. The host-facing profile
  accepts two through four literal-image services, each with one nonzero,
  unique `host:container` port; VAT publishes them only on loopback and does not
  provide a bridge network or service-name DNS. All profiles allow only literal
  environment values and reject DNS/topology, `depends_on`, networks, volumes,
  build on the host-facing profile, interpolation, and `--env-file` before
  runtime launch. A host-facing successful `up` makes that negative contract
  machine-readable with `"profile":"host-facing-independent-v1"`,
  `"service_name_dns":false`, and `"host_loopback_only":true`.
  `docker compose -f FILE -p PROJECT up -d --wait [--wait-timeout SECONDS]`
  is a bounded VAT-only readiness wait, not generic Docker Compose semantics:
  explicit `-d`/`--detach` remains required, `--wait` is accepted once, and
  `--wait-timeout` is accepted only with it as positive whole seconds (default
  300, maximum 1200). Its clock begins after validated import and any source
  build, immediately before detached runner launch, and covers handoff plus
  observations. It waits only for durable VAT runner readiness/topology proof,
  never a Docker healthcheck, application HTTP probe, or service DNS. VAT pins
  the waiter to the profile, generation, and launch ticket and releases the
  registry lock between polls, so an old waiter cannot attach after `down`,
  re-import, or relaunch. A ready wait emits one final `up` JSON result with
  `wait` and ready topology; a timeout retains runtime and registry. A `ps`
  handoff is supplied only after a current pinned-target observation; terminal,
  replaced, or bare-deadline failures have no unsafe next. A degraded result
  publishes no endpoint. For a source build, `cleanup_next` is emitted only on
  that verified-ready wait result.
  `docker compose -p PROJECT ps` has two exact output shapes. The no-format
  form preserves its text surface and ends with an additive `vat_docker_compose`
  JSON record for the known profile. `docker compose -p PROJECT ps --format json`
  and `--format=json` instead emit exactly one VAT-owned JSON document with
  `schema="vat.docker-compose.ps.v1"` and `format="vat_json"`, carrying the
  same claim-held profile/topology proof and no human table. Its `topology` is
  `{ phase, ready, services }`: `phase` is
  `inactive`, `starting`, `ready`, `degraded`, or `stopping`; services follow
  the registered Compose service-ID order, not runtime-evidence order; each has
  `name` and `state`; and an endpoint is the canonical string
  `127.0.0.1:<port>`. `ready=true` and all
  endpoints appear only when every expected service has exactly one Ready,
  VAT-owned `container_run` record for its exact MicroVM name, with a nonzero
  loopback port and no cleanup error. Otherwise a nominally ready lifecycle is
  reported as `degraded` with `ready=false` and no endpoints; starting,
  stopping, and inactive also publish no endpoints. This is lifecycle and
  ownership evidence, not an application health check. The JSON form is not
  Docker Compose JSON/template/table compatibility; all other `ps` formats
  fail closed. Generic, missing, and unknown shim provenance fail closed before
  any topology is emitted. Text `logs SERVICE` preserves its original log bytes,
  then starts its additive VAT handoff JSON on a new line after those observed
  bytes. `logs --format json [--tail LINES] SERVICE` (also `--format=json` /
  `--tail=N`, with service final) emits exactly one capture-only
  `vat.docker-compose.logs.v1` JSON document: separate stdout/stderr snapshots,
  default `tail_lines=200` bounded to 1..=1000, per-stream `truncated` and
  `utf8_lossy`, `capture_only=true`, `runtime_invoked=false`, and
  `compose_record_mutated=false`. It holds the existing claim/provenance then
  reads VAT-captured logs only: no Apple Container call, project.json mutation,
  topology, or endpoints. VAT first caps each read, then after lossy UTF-8 and
  JSON escaping retains a valid UTF-8 suffix whose serialized JSON string value
  remains within the same 64 KiB per-stream cap and marks it `truncated`; it is
  also line-tailed. Its `next` is the VAT-native JSON ps command. It is not Docker Compose merged,
  follow, timestamp, or template-schema compatibility; `--follow`, timestamps,
  and all other flags fail closed. The full serial `vat_docker_shim` aggregate is
  intentionally not recorded because an independent serial run exposed a
  nondeterministic pre-existing Compose JSON logs timing race; the focused
  serialized-cap unit passed 1/1 for `0xff`-heavy and NUL/control-heavy streams
  after actual JSON serialization. The opt-in real dual-service logs-JSON
  coverage is recorded below. Then use text
  `exec -T SERVICE -- COMMAND`, agent JSON
  `exec -T --format json SERVICE -- COMMAND` (or `--format=json`), or `down`.
  Text exec preserves its observed child bytes, then starts its additive VAT
  handoff JSON on a new line after them. Both forms acquire one same-read project snapshot
  under the existing claim, with known shim provenance and one exact unique
  ready VAT-owned MicroVM service; ambiguous or incomplete evidence fails
  closed. The Docker-facing `--` is parsed and validated but not forwarded;
  VAT invokes Apple Container as `container exec CONTAINER COMMAND [ARG...]`.
  JSON exec
  releases that claim immediately after spawning the authorized child, before waiting
  for the arbitrary child duration. It emits exactly one VAT-native
  `vat.docker-compose.exec.v1` document carrying `profile`, `child_exit_code`,
  separate stdout/stderr, per-stream `truncated`/`utf8_lossy`,
  `runtime_invoked=true`, and `compose_record_mutated=false`. It replays no raw
  child output and exposes neither topology nor endpoints. Child stdout and
  stderr are drained concurrently, and each serialized JSON string value is
  capped at 64 KiB. Misordered JSON flags, a missing JSON delimiter, default
  TTY, and all other exec flags fail closed. This is not Docker Compose exec
  output compatibility. The full serial shim aggregate is intentionally not
  recorded because an independent serial run exposed a nondeterministic
  pre-existing Compose JSON logs timing race; the precise serialized-cap unit
  passed 1/1; the real-host Compose JSON-exec scope is stated with the recorded
  E2E below. A
  successful source-build `up` additionally returns its exact VAT-owned
  `images` array plus `cleanup_next` (`down && docker image rm` for that exact
  tag); literal-image projects deliberately do not claim image ownership.
  Shim provenance is also fail-closed: generic `vat compose up`, `ps`, `logs`,
  and `down` cannot operate a known shim-created record. An explicit inactive
  generic `vat compose import` transfers a known record back to generic
  lifecycle by clearing its shim provenance. An inactive unknown-profile record
  may be removed only with registry-only `vat compose down`, which preserves
  `vat.toml`; an unknown active record requires a matching or newer VAT that
  recognizes its profile. On this host, the opt-in gated real Apple Container
  dual-service command
  `RUST_TEST_THREADS=1 VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED=1 cargo
  test -p vat --test vat_docker_shim
  apple_container_docker_compose_host_facing_independent_profile_contract --
  --ignored --nocapture` passed 1/1 (50 filtered) in 4.54 seconds. It proves
  the `host-facing-independent-v1` two-Service `up -d --wait` path, both
  loopback endpoints, one-document JSON `ps`, `logs`, and `exec`, text logs,
  text exec including a no-final-newline child handoff, and `down` cleanup of
  exact containers, ports, and registry. The text-handoff ordering covers only
  bytes VAT observes from its managed child/log stream; it makes no ordering
  claim for descendants that escape that managed process. The gate remains
  opt-in and proves neither service-name DNS, general Compose, a Docker Engine
  API, nor Kubernetes. VAT never exposes
  a Docker Engine socket/API and does not imply general Compose, SDK,
  Testcontainers, devcontainer, Docker output-schema, or Docker-network parity.
- **Not a shared Apple Container builder manager.** `vat capabilities --json`
  can report `apple_container.builder` as a bounded, read-only advisory. Its
  `container builder status` observation has
  `ownership="shared_unknown"` and `automatic_cleanup=false`; a supported,
  parseable status can add configured builder resources separately from live
  `observed_stats`, while optional `container system df` evidence is explicitly
  host-global (`global_apple_container`), never VAT-attributed. Status, stats,
  and disk observations can be unsupported, malformed, or time out; their
  `probe_errors`/unknown state are nonfatal advisory evidence, not a reason to
  infer a running builder. VAT never starts, stops, deletes, or prunes the
  shared builder or its cache. This does not widen the Docker shim or bounded
  Apple K8s contracts.
- **Not an image registry or remote image-build service.** The optional shim
  delegates `docker pull`, `push`, `login`, and `logout` to the user's
  Apple Container registry configuration; VAT does not host or manage a
  registry service. `vat build` and a compose `build:` service can build a
  Dockerfile into the selected local image store (Docker or Apple Container).
  A vat's environment is a declarative
  [`EnvSpec`](src/spec.rs) an agent reads and rewrites. A `vat.toml` *service*
  may run as an ephemeral container, but the runner is always a host process —
  vat never containerizes your workload.

## Quick start

```bash
apps/vat/build.sh debug         # build + install ~/.cargo/bin/vat

# run a command in a fresh copy-on-write clone of the current dir
vat run -- python train.py

# run the default local test protocol from vat.toml
vat capabilities --json  # full host probe, including Docker and shared-builder advisory
vat plan --json          # inspect selected runner/services without side effects
vat doctor --json        # selected-plan preflight; Apple-only plans skip Docker
vat run
vat logs <id> runner

# let an upstream planner/TIA tool choose tests; vat only injects the plan
vat run --plan impact.json impacted

# give an LLM/tool agent the compact vat usage contract
vat llm

# opt in to Docker-shaped agent shell commands over Apple Container
vat docker install-shim --dir "$HOME/.local/bin"
export PATH="$HOME/.local/bin:$PATH"
docker --help

# `strict-single-image-v1`: exactly one literal image service, one explicit port
docker compose --dry-run -f compose.yml -p agent-web up -d
# File/profile-only preflight: it emits one VAT JSON document, performs no
# Apple Container call/build/import/start, writes no registry, and its returned
# launch_argv/next use the parser's canonical source path, so an agent may
# change cwd and still revalidate the same file at real launch.
docker compose -f compose.yml -p agent-web up -d
# Optional bounded runner/topology wait: detached mode is still explicit; the
# default is 300 seconds and --wait-timeout is positive seconds through 1200.
docker compose -f compose.yml -p agent-web up -d --wait --wait-timeout 60
docker compose -p agent-web ps
docker compose -p agent-web logs web
docker compose -p agent-web exec -T web -- sh -ec 'printf agent-ready'
docker compose -p agent-web down

# `strict-single-build-v1`: exactly one short `build: .` service, no `image:`.
docker compose -f compose.build.yml -p agent-web-build up -d --build
docker compose -p agent-web-build exec -T web -- sh -ec 'npm test'
# The successful up JSON includes `images` and a runnable `cleanup_next`.
# Run that cleanup_next after the agent is finished; it executes down, then
# removes only this exact VAT-built image (never a shared-store prune).

# `host-facing-independent-v1` needs this exact top-level Compose marker:
# x-vat-compose-profile: host-facing-independent-v1
# It accepts 2–4 literal-image services, each with one unique nonzero
# `host:container` port. VAT publishes every one on 127.0.0.1 only; it provides
# no service-name DNS, dependencies, networks, volumes, build, interpolation,
# or --env-file support.
docker compose -f compose.independent.yml -p agent-tools up -d
docker compose -p agent-tools ps
docker compose -p agent-tools ps --format json
docker compose -p agent-tools logs docs
docker compose -p agent-tools logs --format json --tail 100 docs
docker compose -p agent-tools exec -T inspector -- sh -ec 'printf agent-ready'
docker compose -p agent-tools exec -T --format json inspector -- sh -ec 'printf agent-ready'
docker compose -p agent-tools down
# Its up JSON includes profile=host-facing-independent-v1,
# service_name_dns=false, and host_loopback_only=true. Its opt-in real Apple
# Container dual-service E2E passed 1/1 (50 filtered) on this host in 4.54s:
# it proves up -d --wait, both loopback endpoints, one-document JSON ps/logs/exec,
# text logs, text exec with a no-final-newline handoff, and down cleanup of
# exact containers, ports, and registry. It does not establish service-name
# DNS, general Compose, a Docker Engine API, or Kubernetes behavior.
# The no-argument ps preserves its table then appends topology={phase,ready,services}
# for the known profile. `ps --format json` (or `--format=json`) instead emits
# exactly one VAT-owned document with schema=vat.docker-compose.ps.v1 and
# format=vat_json, never a human table. Services stay in Compose registration order; an endpoint is
# only `127.0.0.1:<port>` when every expected service has unique Ready,
# exact-VAT-owned container_run/MicroVM evidence, a loopback nonzero port, and
# no cleanup error. Otherwise ps has no endpoints (and ready lifecycle becomes
# phase=degraded, ready=false); it is not an app-healthcheck and rejects --format.

# Read a strict Apple Container-native inventory snapshot. Only the two listed
# container aliases share this JSON form; it is not Docker Engine ps compatibility.
docker ps --format json --all
# `docker container ls` and `docker container list` accept the same JSON form;
# `docker container ps --format json` remains rejected.

# Read a strict Apple Container-native image inventory. It is not Docker Engine
# image-schema or ownership/provenance/readiness evidence.
docker images --format json
# `docker image ls` and `docker image list` accept the same JSON form.

# Build one image through the strict direct VAT receipt. All listed options must
# precede the one existing local-directory context; the documented equals forms
# are equivalent.
docker build --format json --timeout 300 --tag agent-tools:dev --file Dockerfile \
  --build-arg MODE=development --label io.cclab.agent=tools .
# VAT strips only `--format` and `--timeout`, invokes public `container build`,
# and returns one bounded `vat.docker.build.v1` receipt. The selected tag is
# retained with no product auto-cleanup: success points to strict image inspect;
# a normal build failure retains its receipt but deliberately does not inspect a
# possibly partial/replaced tag. A timeout or VAT setup/capture failure emits no
# receipt and does not claim builder cancellation or rollback.

# Pull one opaque image reference through the strict direct VAT receipt. The
# selector order may vary, but both selectors precede the single image reference.
docker pull --format json --timeout 120 alpine:3.20
# This is only direct `docker pull`: raw pull and `docker image pull` retain their
# inherited paths. VAT strips the JSON/deadline selectors, invokes only `container
# image pull IMAGE`, then emits one bounded `vat.docker.pull.v1` receipt. Images
# are shared/non-owning and never cleaned by VAT; timeout observes the host client
# only and does not promise cancellation, transfer completion, or rollback.

# Read one strict Apple Container-native image document. This direct image-only
# JSON form is not Docker image-inspect schema, template, or Engine API parity.
docker image inspect --format json alpine:3.20
# `--format=json` is equivalent. VAT strips its selector before it invokes only
# `container image inspect IMAGE`.

# Read one strict Apple Container-native container document. The JSON selector is
# VAT-only, and the result is not Docker Engine inspect-schema compatibility.
docker inspect --format json agent-tools-inspector
# `docker container inspect` accepts the same JSON form; unformatted inspect
# retains its existing behavior.

# Read one bounded VAT-owned JSON log snapshot. It is not a Docker multiplex or
# demux contract, and unformatted logs retain the inherited text translation.
docker logs --format json --tail 200 agent-tools-inspector
# `docker container logs` accepts the same strict JSON snapshot form.

# Read one strict Apple Container-native resource sample for explicit containers.
# This separate opt-in form accepts no stream, template, --all, or implicit-container
# variant. Its successful stdout is validated native Apple JSON, not a VAT or
# Docker Engine schema/wrapper.
docker stats --no-stream --format json agent-tools-inspector agent-tools-docs

# Keep one Docker-free local K3s guest across explicit agent steps (bounded lease).
# Prerequisite: an independently installed kubectl must be first on PATH; VAT rejects
# an OrbStack-provided kubectl. On this host Homebrew kubectl is at /opt/homebrew/bin.
# Independent-kubectl one-shot, leased, local-image, and Service-forward E2Es
# passed. The local-image proof is one already-local Apple alpine:3.20 pod with
# imagePullPolicy=Never and a marker log, followed by exact session cleanup; it
# is not registry-pull generality. All remain bounded one-guest evidence rather
# than a durable cluster claim.
vat k8s ephemeral image build
vat k8s session create --ttl 30m
# stdout returns id; use it in subsequent tool calls
vat k8s session status --verify-api <id>
vat k8s session exec --timeout 30 <id> -- kubectl get nodes
vat k8s session exec <id> -- kubectl get namespaces
# Text exec is unchanged. For one agent document rather than raw child streams,
# use JSON exec; its process exit remains the child exit code.
vat k8s session exec --format json --timeout 30 <id> -- kubectl get nodes -o json
# move a pre-existing Apple Container image into this active K3s lease only
vat k8s session image load <id> alpine:3.20
# prove the workload cannot fall back to a registry pull
vat k8s session exec <id> -- kubectl run local-alpine --image=alpine:3.20 --restart=Never --image-pull-policy=Never --command -- /bin/sh -ec 'echo local'
# Test one already-created ClusterIP Service through a loopback-only tunnel.
# VAT strips KUBECONFIG, VAT_K8S_CACHE_DIR, VAT_K8S_API_SERVER, and VAT_HOME
# from the child environment. The child shares kubectl's tracked process group,
# so keep it cooperative and non-daemonizing; this hygiene is not a same-UID OS sandbox.
# Text preserves direct child streams. JSON waits for verified tunnel cleanup,
# then emits one bounded agent document with no raw child-stream replay.
vat k8s session port-forward run --format json <id> service/api 8080 -- /bin/sh -ec 'curl -fsS "http://$VAT_K8S_PORT_FORWARD_ADDR/healthz"'
vat k8s session delete <id>

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
  "plan": { "source_path": "impact.json", "rootfs_path": ".../.vat-plan/impact.json",
            "digest": "fnv1a64:..." },
  "test_run": { "topology": { "runners": ["e2e"], "services": ["pg"] },
                "plan": { "...": "..." }, ... },
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
| `vat run --plan <path> [runner-id]` | Copy an opaque upstream plan (for example TIA output) into the vat, expose it as `VAT_PLAN_PATH` / `VAT_PLAN_DIGEST`, and record it in `vat state`. |
| `vat run -- <cmd>` | Clone a base, run one direct command, record the result. `--base DIR`, `--from VAT`, `--isolation none\|seatbelt`, `--gpu auto\|required\|none`, `--json`. |
| `vat capabilities --json` | Full host capability discovery: report COW clone method, isolation backends, Docker provider/daemon state, service-provider capabilities, and an Apple Container shared-builder advisory. It retains the normal Docker daemon probe regardless of a later selected plan. `services.docker_services` is an explicit availability string: a full Docker probe yields `available` or `unavailable`. The advisory is bounded and read-only: `builder status` yields `ownership=shared_unknown` and `automatic_cleanup=false`; parseable configuration is distinct from optional live `observed_stats`, and optional `system df` is `global_apple_container` host evidence rather than VAT-owned disk. Unsupported, malformed, or timed-out status/stats/df appear as advisory unknown/probe errors without failing capability discovery; VAT never starts, stops, deletes, or prunes the builder/cache. A live builder state is reported only when the installed Apple Container CLI supports and returns it. |
| `vat plan [runner-id...] --json` | Print the selected configured run topology without creating a vat, starting services, or running tests. |
| `vat doctor [runner-id...] --json` | Run cheap read-only preflight checks with capability discovery scoped to the selected topology. A selected explicit MicroVm/Apple-Container-only plan performs exactly one read-only `container system status` probe per doctor invocation and projects that result to its selected MicroVm services; it never executes Docker even when it is on `PATH`. In that deliberate no-probe state, `services.docker_services` is `not_probed`, while `docker.daemon_probe.state=skipped` with `Docker daemon probe skipped for Apple-Container-only selected plan` supplies provenance. `docker.daemon=false` is not Docker-unavailable evidence because no Docker command ran. An unselected Docker service cannot poison that runner. The selected plan also reports the shared-builder advisory, but its unknown/timeout/probe errors never change runtime success. Doctor neither autostarts Apple Container nor falls back to Docker: unsupported MicroVm presets with no declared OCI route and MicroVm preset named volumes fail closed. Docker-runtime services, Auto image services, eligible Auto preset Docker fallbacks, and selected clusters retain the normal Docker daemon probe, yielding `services.docker_services=available|unavailable`; a cluster requires its Docker backend. |
| `vat doctor --host-only [--json]` | Configuration-free read-only host preflight. It does not read `vat.toml`, select a runner, create a workspace, or start services. It reports copy-on-write, requested isolation availability, host GPU visibility, Apple Container, Docker daemon, and independent (non-OrbStack) `kubectl` evidence. Missing optional substrates are reported as `unavailable` observations; the command itself completes successfully with `next: vat capabilities --json`. |
| `vat llm [--topic <t>] [--format md\|json]` | Print offline agent-facing docs. Default `outline`; use `--topic guide` for the detailed vat.toml/service/evidence/boundary guide. |
| `vat upgrade` | Self-update to the latest `vat@*` GitHub release (`--check` to report only, `--version <tag>` to pin). One of the three mandatory CLI-convention verbs (`llm`/`upgrade`/`issue`), via the shared `cli-std` crate. |
| `vat issue search\|view\|create` | Search, read, and file diagnostics-rich GitHub issues under `app:vat`; `issue create --dry-run --title <t>` previews version + target + OS/arch diagnostics without submitting. |
| `vat docker install-shim --dir <dir>` | Safely create an opt-in `docker -> vat` symlink in an explicit directory; never overwrites a real Docker client. Add that directory to `PATH`, then use the documented fail-closed Apple Container command subset. `vat docker status --dir <dir>` verifies ownership. |
| `docker ps --format json [--all/-a]` / `--format=json` (also `docker container ls` and `docker container list`) | Strict shim-only direct inventory. JSON accepts optional exactly-once `--all` or `-a`, then invokes canonical `container list --format json [--all]`. VAT validates one opaque Apple-native JSON value and replays those stdout bytes unchanged—no VAT wrapper or Docker Engine `ps` schema. Templates/table output, filter, quiet with JSON, duplicate/unknown flags, and positionals fail before Apple Container starts. `docker container ps --format json` remains rejected; inherited text behavior is unchanged. This is a read-only inventory snapshot, not ownership, health, readiness, or liveness proof. A single five-second deadline plus bounded isolated-process-group cleanup cover root exit and both pipe EOFs; stdout/stderr are each capped at 256 KiB, and malformed, oversized, or escaped-pipe stdout is not replayed. Recorded validation: `cargo check -p vat --no-default-features` passed; shared `docker_shim` lib passed 54/54; focused direct-ps integration passed 4/4; the full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; `ps` is a global read-only inventory smoke observation, not a targeted ownership result. It proves one valid native JSON document only; fake/unit tests prove byte-preservation and fail-closed behavior. |
| `docker images --format json` / `--format=json` (also `docker image ls` and `docker image list`) | Strict shim-only image inventory. VAT invokes canonical `container image list --format json`, bounded-captures and validates one opaque Apple-native JSON value, then byte-for-byte replays stdout without a VAT wrapper or Docker Engine image schema. Template/table/YAML/TOML output, filter, quiet, verbose, all, digests, no-trunc, positionals, duplicates, unknown flags, and `--` fail before Apple Container starts; existing text/quiet image-list behavior is unchanged. The snapshot makes no ownership, provenance, security, executability, registry, build-readiness, health, readiness, or liveness claim. A single five-second deadline plus bounded isolated-process-group cleanup cover root exit and both pipe EOFs; stdout/stderr are each capped at 256 KiB, and malformed, oversized, or escaped-pipe stdout is not replayed. Recorded validation: `cargo check -p vat --no-default-features` passed; shared `docker_shim` lib passed 54/54; focused `docker_images_json` integration passed 4/4; the full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; `images` is a global read-only inventory smoke observation, not a targeted ownership result. It proves one valid native JSON document only; fake/unit tests prove byte-preservation and fail-closed behavior. |
| `docker image inspect --format json IMAGE` / `--format=json` | Strict shim-only direct image inspect. It accepts exactly one JSON selector before exactly one opaque safe image reference (nonempty; no leading `-`, whitespace, or control characters); selectors, `--`, a second reference, and every other option/template fail before Apple Container starts. VAT strips the selector, invokes only `container image inspect IMAGE`, bounded-captures and validates one opaque Apple-native JSON document, then byte-for-byte replays complete native stdout. A five-second isolated observer covers root exit and both pipe EOFs; stdout/stderr are capped at 256 KiB, valid JSON preserves a nonzero child exit, and malformed, oversized, or escaped-pipe capture has no raw replay. This is not Docker image-inspect schema/template/Engine API parity, nor provenance, security, registry, build-completion, readiness, or secret-redaction evidence. Recorded validation: cargo check passed; `cargo test -p vat --lib docker_shim -- --nocapture` passed 58/58; `RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_image_inspect_json -- --nocapture` passed 4/4 with 1 ignored; `RUST_TEST_THREADS=1 VAT_DOCKER_IMAGE_INSPECT_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_image_inspect_json_contract -- --ignored --nocapture` passed 1/1 (61 filtered) in 1.21s. The host test proves only one direct `container image inspect alpine:3.20` invocation and one valid native document; fake/unit tests prove selector stripping, byte preservation, and fail-closed bounds. |
| `docker inspect --format json CONTAINER` / `--format=json` (also `docker container inspect`) | Strict shim-only direct container inspect. It accepts exactly one safe explicit container id after exactly one JSON selector; the selector must precede the id, is VAT-only, and is never forwarded. Unformatted inspect retains existing behavior. VAT invokes canonical `container inspect CONTAINER`, bounded-captures and validates one opaque Apple-native JSON value, then byte-for-byte replays stdout without a VAT wrapper or Docker Engine inspect schema. `--type`, `--size`, templates/table/YAML/TOML, filters, a second id, `--`, and unknown flags fail before Apple Container starts. A five-second bounded isolated observer governs root exit and both pipe EOFs; stdout/stderr are each capped at 256 KiB. Valid native JSON with a nonzero child exit preserves that status; malformed, oversized, or flooding output suppresses raw stdout. This makes no ownership, provenance, security, image, registry, build-status, health, readiness, liveness, or port-reachability claim, and provides no secret-redaction guarantee. Recorded validation: `cargo check -p vat --no-default-features` passed; shared `docker_shim` lib passed 54/54; focused `docker_inspect` integration passed 5/5; the full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; `inspect` targets the temporary owner-labeled nginx container. It proves one valid native JSON document only; fake/unit tests prove byte-preservation and fail-closed behavior. |
| `docker logs --format json --tail LINES CONTAINER` / equals forms (also `docker container logs`) | Strict shim-only finite VAT-JSON snapshot, separate from the five Apple-native JSON forms. The format and tail appear exactly once, may use mixed separated/equals spellings, and must precede one safe final id; `LINES` is 1..=1000. Unformatted logs retain inherited text translation. VAT invokes only canonical `container logs -n LINES CONTAINER`; the Docker JSON selector is never forwarded. Apple exposes text stdout only, so stdout is exactly one VAT-owned `schema="vat.docker.logs.v1"`, `format="vat_json"` wrapper: untrusted `apple_container_stdio`, bounded diagnostic stderr, truncation/lossy flags, backend/container/requested_tail/runtime/child outcome, and a safe inspect next. Ordinary child nonzero retains the wrapper and exit code; timeout, setup failure, or an escaped pipe holder fails closed without a partial wrapper. A five-second observation plus one-second cleanup drains both pipes, retains suffixes, and caps each capture and actual JSON string value at 64 KiB. `--follow`, boot, timestamps, since/until, templates, and every other modifier fail before Apple Container. This is not Docker schema, multiplex/demux, ownership, provenance, security, image, registry, build, health, readiness, liveness, port-reachability, or secret-redaction evidence. Recorded validation: `cargo check -p vat --no-default-features` passed; canonical `cargo test -p vat --lib docker_shim -- --nocapture` passed 54/54; focused `RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_logs_json -- --nocapture` passed 6/6; the full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; VAT `logs` targets the high-entropy nonce+PID owner-labeled temporary nginx container. Exact owner-label rechecks are conservative best-effort precautions, the emergency guard retains the container on uncertainty, and Apple Container has no atomic conditional delete; this is not a race-free or impossible-to-misdelete cleanup guarantee. The shared/cacheable nginx image is not cleaned up. It proves one VAT wrapper only; fake/unit tests prove byte-preservation and fail-closed behavior. |
| `docker exec --format json --timeout SECONDS CONTAINER -- COMMAND [ARG...]` / equals forms (also `docker container exec`) | Strict shim-only finite VAT-JSON foreground command snapshot, separate from the native JSON forms. One `--format json`/`--format=json` and one `--timeout SECONDS`/`--timeout=SECONDS` occur exactly once, in either order before one safe container id; `SECONDS` is 1..=1200, and the Docker-facing literal `--` plus at least one raw command argument are mandatory. Unformatted/raw exec retains inherited generic translation, including selector-looking raw arguments after its delimiter. VAT removes those Docker-only selectors and delimiter, then invokes Apple Container as `container exec CONTAINER COMMAND [ARG...]`. Stdout is exactly one `schema="vat.docker.exec.v1"`, `format="vat_json"` wrapper with backend/container/requested timeout, `timeout_scope="host-container-client-observation"`, runtime/child outcome, bounded stdout/stderr suffixes with truncation/lossy flags, untrusted command output, no secret-redaction guarantee, and a safe inspect next. Ordinary child nonzero preserves wrapper plus exit; timeout or setup/capture failure emits no partial wrapper. Both streams are captured concurrently and each serialized JSON string value is capped at 64 KiB. The timeout bounds only VAT's host Apple Container client observation; it makes no guest-command termination claim. TTY, interactive, detach, environment/user/workdir, templates, duplicate/misordered selectors, malformed delimiter, and all other exec flags fail before Apple Container. This is not Docker Engine stream/TTY behavior, ownership, readiness, health, or secret-redaction evidence. Recorded validation: `cargo check -p vat --no-default-features` passed; canonical `cargo test -p vat --lib docker_shim -- --nocapture` passed 54/54; focused `RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_exec_json -- --nocapture` passed 4/4. The direct real-host E2E passed 1/1 (50 filtered) on Apple Container 1.1.0 and observed one direct exec wrapper with both stdout and stderr markers; it remains bounded direct-command evidence only, not guest-timeout termination, Docker Engine parity, generic runtime, Compose, or Kubernetes evidence. |
| `docker run --format json --timeout SECONDS IMAGE [COMMAND...]` / equals forms | Strict shim-only foreground one-shot, available only as direct `docker run`. Exactly one format selector and one 1..=1200 timeout may appear in either order before `IMAGE`; optional command argv follows the image directly. A Docker `--` before `IMAGE` or immediately after it is rejected; once a non-`--` command token begins, later `--` is opaque child argv. Detach, TTY, interactive, caller name/label, ports, network, mounts, env, and every other run option also fail before Apple Container starts. VAT generates a high-entropy name plus an independent owner label, captures bounded stdout/stderr, and emits exactly one `schema="vat.docker.run.v1"`, `format="vat_json"` result only after exact owner-label cleanup confirms absence. A normal nonzero child exit is preserved only with that confirmed cleanup; timeout, setup, or cleanup uncertainty emits no partial wrapper. Apple inspect proves absence only with its explicit `Error: container not found: <name>` diagnostic; other cleanup ambiguity fails closed. The timeout is host Apple Container client observation only, not guest-wide termination. It makes no crash-recovery cleanup, Docker Engine parity, or secret-redaction claim. Focused deterministic validation (`docker_run_json`) passed 5 plus 1 ignored in 1.80s; the local `alpine:3.20` real E2E passed 1/1 (56 filtered) in 2.30s with one wrapper and exact cleanup. |
| `docker build --format json --timeout SECONDS --tag TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE] [--platform PLATFORM] [--label K=V ...] CONTEXT` / documented equals forms | Strict shim-only direct build receipt, separate from the five Apple-native JSON observations. Exactly one `--format json`, one positive whole `--timeout` in 1..=1200, and one `--tag` are required; `--file`, `--target`, and `--platform` are each optional once, build args/labels may repeat, and every option precedes exactly one canonical existing local-directory context. `--`, any second/misordered/missing selector, a second context, and unsupported flags fail before a builder starts; raw builds without either receipt selector keep their inherited translator. VAT strips only the JSON/deadline selectors and calls public `container build --tag TAG [--file ...] [--build-arg ...] [--target ...] [--platform ...] [--label ...] CONTEXT`. After the Apple client exits it emits one bounded `schema="vat.docker.build.v1"`, `format="vat_json"` receipt with untrusted stdout/stderr, truncation/lossy flags, timeout scope, and child outcome. `image_lifecycle="retained_no_auto_cleanup"`: this product path neither removes a successful image nor cleans a partial/replaced tag. Success safely hands off to `docker image inspect --format json TAG`; a normal child nonzero keeps the receipt and exit but returns `terminal="build_failed"` and `next="docker --help"`, never a stale inspect handoff. Timeout, setup, or capture failure emits no receipt. The deadline only observes the host Apple Container client; it does not cancel the builder or roll back/remove an image. No Docker Engine/API, provenance, ownership, readiness, security, secret-redaction, cancellation, or rollback claim is made; build arguments, labels, and output are opaque/untrusted. Recorded validation: `cargo check -p vat --no-default-features` passed; `cargo test -p vat --lib docker_shim -- --nocapture` passed 62/62; focused `docker_build_json` passed 4 plus 1 ignored (63 filtered); `native_image_owner_guard...` passed 1/1 (67 filtered); and `RUST_TEST_THREADS=1 VAT_DOCKER_BUILD_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_build_json_receipt_contract -- --ignored --nocapture` passed 1/1 (67 filtered) in 2.53s. That host test proves one strict mapped BuildKit invocation and receipt only. Its cleanup is test-only: a high-entropy tag and exact `io.cclab.vat.e2e-owner` label require exact native absence before build, an exact label recheck before delete, and exact absence afterward. Apple Container has no conditional build/delete, so this is best effort and ambiguity leaks; it never changes the product receipt's retained/no-auto-cleanup behavior. |
| `docker pull --format json --timeout SECONDS IMAGE` / documented equals forms | Strict shim-only direct pull receipt, separate from the Apple-native JSON observations. Exactly one `--format json` and one positive whole `--timeout` in 1..=1200 may be reordered before one opaque safe image reference (nonempty, no leading `-`, whitespace/control characters, URL-style `://`, or leading Git-style `git@` remote); ordinary OCI `@digest` references remain opaque. `--`, a second reference, missing/duplicate/misordered selectors, and unsupported flags fail before the client; raw `docker pull IMAGE` and `docker image pull` stay on inherited paths. VAT strips only JSON/deadline selectors and invokes public `container image pull IMAGE`. After the Apple client exits it emits one bounded `schema="vat.docker.pull.v1"`, `format="vat_json"` receipt with untrusted stdout/stderr, truncation/lossy flags, timeout scope, and child outcome. `image_lifecycle="not_owned_no_auto_cleanup"`, `cleanup_attempted=false`, and `registry_management_implemented=false`: the image is shared, not VAT-owned, and never cleaned; VAT neither manages registry login/auth/credential lifecycle nor proves a registry transfer. Success safely hands off to `docker image inspect --format json IMAGE`, without asserting image state/completion; a normal child nonzero retains the receipt and exit but returns `terminal="pull_failed"` and `next="docker --help"`, never stale inspect. Timeout, setup, capture, or pipe failure emits no receipt. The deadline observes only the host Apple client/pipes; it neither cancels a transfer nor guarantees download completion, rollback, or local/backend image state. No Docker Engine/API, provenance, digest, platform, freshness, image-state, ownership, security, secret-redaction, cancellation, download-completion, or rollback claim is made. Recorded validation: `cargo check -p vat --no-default-features` passed; `cargo test -p vat --lib docker_shim -- --nocapture` passed 65/65; focused `docker_pull_json` passed 5 plus 1 ignored (68 filtered); and `RUST_TEST_THREADS=1 VAT_DOCKER_PULL_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_pull_json_receipt_contract -- --ignored --nocapture` passed 1/1 (73 filtered) in 27.14s. The host test invokes real `container image pull alpine:3.20` but deliberately uses a shared/cacheable image and neither deletes it nor asserts ownership, even on failure. |
| `docker stats --no-stream --format json <container> [<container>...]` | Shim-only one-shot resource observation, separate from the direct, image, and inspect forms above. Only `--format json` or `--format=json`, exactly one `--no-stream`, and one or more explicit container ids are accepted; both flags must precede the ids. Streaming, templates, `--all`, duplicate/unknown flags, and options after an id fail before Apple Container starts. VAT invokes Apple Container as `container stats --format json --no-stream …`, validates one native JSON document, and replays that exact native JSON without a VAT/Docker Engine wrapper or schema. It is read-only and is not ownership, health, readiness, or liveness proof. One five-second observation deadline and bounded isolated-process-group cleanup govern root exit and both pipe EOFs. VAT replays stdout only after a complete bounded capture validates as one native JSON document; an escaped pipe holder fails closed with no stdout replay. Each stdout/stderr capture is bounded at 256 KiB; malformed or oversized stdout is suppressed rather than partially replayed. Shared `docker_shim` library validation passed 54/54; the full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; `stats` targets the temporary owner-labeled nginx container and proves one valid native JSON document only. Fake/unit tests prove byte-preservation and fail-closed behavior. |
| `docker compose [--dry-run] -f <file> -p <project> up -d [--build] [--wait [--wait-timeout <seconds>]]` | Shim-only Apple Container Compose has exactly three profiles: `strict-single-image-v1` is one literal-image service with `up -d`; `strict-single-build-v1` is one literal short `build: <context>` service with no `image:` and `up -d --build`; `host-facing-independent-v1` requires the exact top-level `x-vat-compose-profile: host-facing-independent-v1` marker and has 2–4 literal-image services, each with one unique nonzero `host:container` port published only on loopback. `docker compose --dry-run -f FILE -p PROJECT up -d [--build]` is a strict file/profile-only preflight for those same profiles. It emits exactly one `schema="vat.docker-compose.preflight.v1"` VAT JSON document with `validated=true`, `runtime_started=false`, `registry_written=false`, `image_built=false`, `launch_revalidates=true`, structured `launch_argv`, and an executable `next`; `launch_argv`/`next` use the parser's canonical source path, so an agent may change cwd and still revalidate the same file. It never calls Apple Container, builds, imports, starts, or writes a registry. It rejects `--wait`, `--wait-timeout`, and every other global/Compose flag, and the returned real launch revalidates the file before import or runtime start. The host-facing JSON states `profile`, `service_name_dns=false`, and `host_loopback_only=true`: no bridge/service-name DNS. All profiles reject dependencies, networks, volumes, host-facing build, interpolation, `--env-file`, and other unsupported forms before runtime launch. `-d`/`--detach` is mandatory even with one `--wait`; `--wait-timeout` requires `--wait`, is positive whole seconds, defaults to 300, and caps at 1200. Its budget begins after validated import/build but before launch, covers detached handoff plus VAT observations, and waits only durable VAT runner/topology proof—not Docker healthchecks, app HTTP, service DNS, or generic Compose. Profile/generation/ticket pinning and polling without the registry lock prevent stale waiters attaching after down/re-import/relaunch. Ready returns one final up JSON with `wait` and ready topology; timeout retains runtime/registry and offers `ps` only after a current-target observation, while terminal/replaced/bare-deadline failures have no unsafe next and degraded emits no endpoint. For source builds, `cleanup_next` accompanies only verified ready wait success. Follow with `docker compose -p <project> ps`, `logs <service>`, text `exec -T <service> -- <command...>`, agent JSON `exec -T --format json <service> -- <command...>`, or `down`; known shim provenance cannot be operated by generic `vat compose` commands, while an inactive generic re-import explicitly clears that provenance. Unknown inactive provenance can only be registry-cleaned (preserving `vat.toml`); unknown active provenance needs matching/newer VAT. A non-wait source-build up emits exact VAT-owned `images` and `cleanup_next`; literal-image up does not. Deterministic fake coverage is supplemented by the opt-in real two-Service host E2E passed 1/1 (50 filtered) in 4.54 seconds, limited to this profile's up-wait/endpoints, one-document JSON ps/logs/exec, text logs, no-final-newline text-exec handoff, and down-cleanup contract—not general Compose behavior. |
| `docker compose -p <project> ps [--format json\|--format=json]` | Accepts exactly two shapes. No format preserves text and ends with additive JSON `profile` plus `topology { phase, ready, services }`. `--format json` or `--format=json` emits exactly one VAT-owned JSON document, `schema="vat.docker-compose.ps.v1"` and `format="vat_json"`, with the same claim-held profile/topology proof and no human table. This is not Docker Compose JSON/template/table compatibility; every other `ps` format/flag fails closed. `phase` is `inactive`/`starting`/`ready`/`degraded`/`stopping`; services use registered Compose order; an `endpoint` is only canonical `127.0.0.1:<port>`. VAT emits every endpoint only if every expected service has one unique Ready, VAT-owned `container_run` record for its exact MicroVM, a loopback nonzero port, and no cleanup error. Any failed proof makes a nominal ready lifecycle `degraded` with `ready=false` and no endpoints; inactive/starting/stopping also have no endpoints. This is not an app-healthcheck. Generic, missing, or unknown provenance fails closed without topology. The full serial `vat_docker_shim` aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race; this document makes no aggregate claim for this sandbox. The historical real dual-service host E2E is opt-in bounded-profile evidence only, never service-name DNS, general Compose, Docker Engine API, or Kubernetes. |
| `docker compose -p <project> logs [--format json\|--format=json] [--tail <lines>\|--tail=<lines>] <service>` | Text is exactly `logs SERVICE`: VAT preserves observed log bytes, then starts its additive VAT handoff JSON on a new line after them. JSON accepts only `--format json`/`--format=json` with optional `--tail LINES`/`--tail=N`, service final. It emits exactly one `schema="vat.docker-compose.logs.v1"` VAT JSON document with separate stdout/stderr snapshots; `tail_lines` defaults to 200 and is 1..=1000; each stream declares `truncated` and `utf8_lossy`. It is capture-only (`capture_only=true`, `runtime_invoked=false`, `compose_record_mutated=false`): VAT holds existing claim/provenance, reads its captured logs, calls no Apple Container runtime, and does not mutate project.json. VAT first caps each read and bounded line tail, then after lossy UTF-8 plus JSON escaping retains a valid UTF-8 suffix whose serialized JSON string value is within the same 64 KiB per-stream cap and marks it truncated. JSON contains neither topology nor endpoints and supplies `next` as `docker compose -p <project> ps --format json`. It is not Docker Compose merged/follow/timestamps/template-schema compatibility: `--follow`, timestamps, and all other flags fail closed. The full serial `vat_docker_shim` aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race; its focused serialized-cap unit passed 1/1 for `0xff`-heavy and NUL/control-heavy streams after actual JSON serialization. The opt-in dual-service real-host E2E passed 1/1 (50 filtered) in 4.54 seconds and includes this one-document JSON logs shape only for `host-facing-independent-v1`; it is not general Compose evidence. |
| `docker compose -p <project> exec -T <service> [-- <command...>]` | Text exec with exact `-T` runs an existing known-profile service, preserves observed child bytes, then starts its additive VAT handoff JSON on a new line after them. JSON has exactly two accepted spellings: `docker compose -p PROJECT exec -T --format json SERVICE -- COMMAND` and `--format=json`; its format marker is immediately after `-T`, the service follows it, and the Docker-facing `--` is mandatory. One same-read claim-held snapshot proves known shim provenance and one exact unique ready VAT-owned MicroVM service before child spawn; incomplete or ambiguous evidence fails closed. VAT parses and validates that Docker-facing delimiter but does not forward it, invoking Apple Container as `container exec CONTAINER COMMAND [ARG...]`, then releases the claim immediately after spawn. JSON emits exactly one `schema="vat.docker-compose.exec.v1"`, `format="vat_json"` document with `profile`, `child_exit_code`, separate stdout/stderr, per-stream `truncated`/`utf8_lossy`, `runtime_invoked=true`, and `compose_record_mutated=false`. It does not replay raw child output or expose topology/endpoints. The two child streams are drained concurrently; each serialized JSON string value is capped at 64 KiB. The text handoff ordering covers only bytes VAT observes from its managed child, not descendants that escape it. Default TTY, all other exec flags, JSON-format misordering, and a missing delimiter fail closed. It is not Docker Compose exec output compatibility. The full serial shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race; `bounded_log_stream_keeps_agent_snapshots_line_and_serialized_json_bounded` passed 1/1 and includes the serialized exec-stream cap. The opt-in dual-service real-host E2E passed this one-document JSON exec shape 1/1 (50 filtered) in 4.54 seconds for `host-facing-independent-v1` only; it is not general Compose evidence. |
| K3s host CLI prerequisite | Every `vat k8s` command requires an independently installed `kubectl` first on `PATH`; VAT rejects an OrbStack-provided binary before K3s use. This is host-tool provenance, not a GUI or Docker Engine requirement. Homebrew `kubernetes-cli` at `/opt/homebrew/bin/kubectl` is installed on this host. Independent-kubectl one-shot, leased, local-image, and Service-forward E2Es each passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s respectively; the local-image proof is one already-local Apple `alpine:3.20` pod with `imagePullPolicy=Never`, a marker log, and exact session cleanup—not registry-pull generality. |
| `vat k8s ephemeral image build` | Explicitly build VAT's embedded systemd image into the Apple Container image store. Its local tag identifies the embedded build-asset revision, not a verified supply-chain image digest. It never starts a cluster. |
| `vat k8s ephemeral run [--image <ref>] -- <command...>` | Start exactly one disposable Apple K3s guest, prove host API access through a private kubeconfig, run one foreground command, then delete credentials and the exact owned machine. The child receives `KUBECONFIG`, `VAT_K8S_CACHE_DIR`, `VAT_K8S_API_SERVER`, and an isolated `HOME`; direct kubectl keeps its normal cache under that private HOME. Its final stdout line is a `vat_k8s_ephemeral_result` terminal JSON record. The independent-kubectl one-shot real-host E2E passed 1/1 (36 filtered) in 28.38s. On bootstrap failure, VAT renders the root error first, then a best-effort 6-second total / 1-second-per-probe read-only diagnostic block with exactly `guest_install_log`, `guest_k3s_system`, `backing_container_logs`, `machine_boot_log`, `machine_inspect`, and `container_system_status`; staged installer evidence is non-sensitive, private kubeconfig/cache and host credentials are excluded, and the existing exact cleanup still runs. This does not fix or retry the existing 300-second bootstrap behavior, rerun `k3s --version`, or add a wrapper/recovery command. `vat k8s ephemeral cleanup` reconciles interrupted sessions only after the recorded PID is gone; an interrupted create retains its marker until Apple Container can prove a terminal create/cancellation state. |
| `vat k8s session create [--ttl 30m]` | Create one bounded, one-boot Apple K3s lease with private 0600 credentials. The result includes an opaque id and runnable `next`; it never exposes the kubeconfig path. TTL accepts whole seconds or `s`/`m`/`h`, from 1 minute through 4 hours. Its shared K3s bootstrap path uses the same primary-error-first, fixed read-only diagnostic block and exact cleanup on failure; advisory diagnostics never make this a persistent Kubernetes backend. |
| `vat k8s session exec [--format json] [--timeout <seconds>] <id> -- <command...>` | Both text and JSON exec prove the active lease, exact Apple backing-ID/API endpoint, private credentials, and owned host API under the private operation lock. Omit `--timeout` to use the remaining lease TTL; an explicit timeout is 1..=14400 seconds and cannot exceed that remaining TTL. VAT rechecks expiry immediately before spawn, puts every credentialed host command in an owned process group, and holds the lock through its cleanup. Normal exit, deadline, or SIGINT/SIGTERM stops and reaps that group; the private exec marker is removed only after the group is absent. If VAT crashes after marker creation, a starting or live exec marker makes later exec, delete, or cleanup fail closed rather than signal an arbitrary recovered command; this is not a crash-safe termination guarantee. `--format json` then emits exactly one `schema="vat.k8s.session.exec.v1"`, `format="vat_json"` document with separate stdout/stderr, child exit code, stream truncation/lossy flags, `api_verified=true`, `runtime_invoked=true`, `session_record_mutated=false`, and a `status --verify-api` next step; raw child streams are not replayed. Both streams drain concurrently and retain only a latest suffix whose serialized JSON string is at most 64 KiB. JSON errors mask private credential/cache paths. The child intentionally receives credentials, so this is not an untrusted-child security boundary. Deterministic fake/unit coverage exists. The independent-kubectl leased real-host E2E passed 1/1 (36 filtered) in 29.97s and proved text commands, strict one-document JSON exec with `--timeout 30`, `status --verify-api`, and exact delete; it does not establish crash recovery termination or persistent Kubernetes. |
| `vat k8s session port-forward run [--format json] <id> service/<name> <remote-port> [--namespace <ns>] [--local-port <port>] -- <command...>` | Requires an independently installed `kubectl` first on `PATH`; VAT rejects an OrbStack-provided binary before K3s use. Text forwards exactly one literal Service port to `127.0.0.1` for one foreground host child and writes its terminal record on a new line after child output. `--format json` is the only JSON form. It remains Service-only, loopback-only, and credential-free for the host child: `--local-port 0` (the default) lets kubectl choose a loopback port; the child receives only `VAT_K8S_PORT_FORWARD_{HOST,PORT,ADDR,RESOURCE,NAMESPACE}` and a private `HOME`, while VAT strips `KUBECONFIG`, `VAT_K8S_CACHE_DIR`, `VAT_K8S_API_SERVER`, `VAT_K8S_EPHEMERAL`, and `VAT_HOME`. This is credential hygiene rather than a same-UID OS sandbox or adversarial-child security boundary. The child joins kubectl's authenticated process group, and VAT holds the private operation lock through group cleanup: normal cleanup reaps the leader and confirms ordinary cooperative, non-daemonizing descendants are gone before `cleanup=confirmed`; children that daemonize or escape the group are outside the contract. JSON emits exactly one `schema="vat.k8s.session.port-forward.v1"`, `format="vat_json"` document only after cleanup is confirmed, with separate 64 KiB serialized-capped stdout/stderr, truncation/lossy flags, child exit, and a `status --verify-api` next step; it never replays raw child streams. VAT-owned lease/setup/API/tunnel/cleanup failures are masked, while opaque credential-free child output is preserved in a successful document. It rechecks the lease silently after API proof and immediately before exact kubectl and host-child spawns, so expiry prevents a tunnel; a partial reader setup reaps the direct child and completes outer-group cleanup before joining readers. The independent-kubectl Service-forward E2E passed 1/1 (36 filtered) in 49.57s, including one loopback Service text and strict one-document JSON tunnel with a credential-free child, confirmed cleanup, and closed local ports. This is not ingress/LB, a public listener, a background tunnel, arbitrary resource forwarding, persistent Kubernetes, or a same-UID OS sandbox. |
| `vat k8s session image load <id> <local-ref> [--platform linux/arm64]` | Deliver one already-local Apple Container image into the active lease's K3s `k8s.io` namespace without Docker or a registry pull. VAT requires exactly one inspected `linux/arm64` variant, uses a private 2 GiB-bounded OCI archive, verifies the canonical reference after import, then removes archive copies from host and guest. The opt-in real-host local-image E2E passed 1/1 (36 filtered) in 49.73s: one already-local Apple `alpine:3.20` loaded into one lease, a pod ran it with `imagePullPolicy=Never` and emitted its marker log, then exact session cleanup completed. This is not registry-pull generality, persistence, GUI, or Docker Engine/API evidence. Arbitrary tar files and cross-platform delivery fail closed. |
| `vat k8s session status [--verify-api] <id> \| delete <id> \| cleanup` | No-flag `status` is unchanged: it reports only non-secret lease and exact-machine state. `status --verify-api <id>` only probes an active, unexpired session with no retained port-forward or exec marker. Under the private operation lock it rechecks expiry, proves the exact backing identity/endpoint and private credentials, rechecks expiry immediately before one bounded API probe, then reports `api_checked=true`, `api_state="reachable"` on success. Expired/recovery-marker paths do not probe and report `api_checked=false`, `api_state="not_checked"`; busy, unavailable, and identity-mismatched sessions fail closed without mutating the lease or credentials. A live or starting exec marker similarly blocks exec, delete, and cleanup rather than claiming it safely terminated a prior credentialed group. Focused fake status coverage passed 4/4; the precise status unit passed 1/1. The independent-kubectl leased E2E passed 1/1 (36 filtered) in 29.97s and includes `status --verify-api` after text and strict JSON exec; this is bounded active-lease evidence, not persistence or a general API-status guarantee. This remains a one-boot, nonpersistent Apple Container lease with no GUI or Docker Engine/API. `delete` confirms removal of the exact machine before removing credentials. `cleanup` reclaims expired leases and abandoned creates; there is no background cleanup daemon. |
| `vat ls` | List vats (one line each, or `--json` array of full states). |
| `vat state <id>` | Full agent-legible state as JSON (`--compact` for one line). |
| `vat diff <id>` | Every filesystem change vs. the vat's base (`--json`). |
| `vat logs <id> [service-id\|runner]` | Print captured logs from a retained vat.toml runner invocation. |
| `vat fork <id>` | Copy-on-write a new **runnable** working copy. |
| `vat snapshot <id>` | Copy-on-write a **frozen** restore point. |
| `vat rm <id>` | Delete a vat and its workspace. |
| `vat gc [--execute]` | Report retained vat disk usage and prune old workspaces. Dry-run by default; protects running/snapshot/failed/interrupted/newest vats unless explicit flags opt in. |
| `vat gpu` | Report the GPU every vat on this host can reach. |
| `vat cluster create\|ls\|delete\|kubeconfig` | Manage standalone local Kubernetes clusters (kind/k3d/minikube), independent of a run. |

### Disk cleanup

Retained vats can accumulate large copy-on-write workspaces. Use `vat gc` to
inspect disk pressure before deleting anything:

```bash
vat gc --json                         # dry-run, machine-readable metadata report
vat gc --measure --json               # also run du -sk for disk sizes
vat gc --keep-last 5                  # dry-run: keep the newest 5 vats
vat gc --execute --keep-last 5        # delete non-running, non-snapshot,
                                      # non-failed candidates
vat gc --execute --include-failed --keep-last 5
                                      # also prune failed/interrupted retained runs
vat gc --apparent --json              # also walk files for apparent size
```

The default GC report avoids walking large rootfs trees, so it stays usable when
hundreds of vats exist. Add `--measure` when you need `disk_size_bytes` from
`du -sk`. Add `--apparent` only when you need file-length totals; it walks every
retained rootfs and is slower on large stores. APFS/reflink clones can make
apparent size much larger than physical blocks.

### Interrupt cleanup

`vat run` installs scoped SIGINT/SIGTERM cancellation before it owns children.
The first signal wins; the handler only records it, while the run thread stops
runner process groups first and VAT-owned services in reverse start order. Each
group receives TERM, a bounded grace period, KILL when still present, leader
reaping, and an explicit PGID-absence check before terminal metadata is written.
Direct and configured runs then persist `status.state = "interrupted"` with the
signal and reason, clear child PIDs, retain the VAT as failure evidence, and
exit 130 for SIGINT or 143 for SIGTERM. Explicit `external` services and other
unrelated listeners are observed only and are never signalled by this cleanup.

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
# runtime = "auto"         # auto (default) | native | docker | micro_vm (explicit Apple Container)
seed = ["schema.sql", "fixtures.sql"]
export = { DATABASE_URL = "DATABASE_URL" }

[[services]]
id = "alloy"               # OCI image dependency (no native binary)
image = "google/alloydbomni:latest"
runtime = "micro_vm"       # explicit Apple Container route; never silently falls back to Docker
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
  `runtime = "micro_vm"` is explicit — `auto` never selects it and it never
  falls back to Docker. For presets with a declared OCI route (for example
  Redis/Postgres/NATS) it uses Apple `container`, checks the local image store,
  performs a bounded pull if missing, verifies it again, then runs it through a
  loopback-only published port. Presets without a declared OCI route and
  MicroVM preset named volumes fail closed.
  Datastore/broker presets: `postgres`, `redis`, `nats`, `rabbitmq`, `mysql`,
  `mongo`.
- `preset` (built-in Rust emulators) — `gcloud-pubsub`, `firebase-auth`, `gcloud-cloud-tasks`,
  `cloud-scheduler`, and `cloud-workflows` run vat's **own** in-process emulator
  under `runtime = auto`: pure Rust, instant start, **no gcloud / Java /
  firebase-tools / Docker**. `pubsub` is a google.pubsub.v1 gRPC server
  (topics/subscriptions, Publish, Pull, StreamingPull, Acknowledge);
  `firebase-auth` is a Firebase Auth (Identity Toolkit) REST server;
  `gcloud-cloud-tasks` serves **both the Cloud Tasks v2 gRPC service and the v2 REST API
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
  drops it). **Wiring a `gcloud-cloud-tasks` / `cloud-scheduler` client:** these SDKs don't
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
  preset = "gcloud-pubsub"   # built-in gRPC emulator → PUBSUB_EMULATOR_HOST
  ```
- `preset` (external emulators) — `gcloud-firestore`, `gcloud-datastore`,
  `gcloud-bigtable`, and `gcloud-spanner` wrap the GCP `gcloud beta emulators`
  family. Native needs gcloud +
  Java + the gcloud component; `runtime = auto` falls back to the cloud-cli
  Docker image (Spanner uses its own image) when the component is missing.
  Each exports the well-known host var (e.g. `FIRESTORE_EMULATOR_HOST`).
  `preset = "firebase"` is the Firebase Emulator Suite bundle: it requires a
  `firebase.json`, runs `firebase emulators:start`, and exports each configured
  emulator's `*_EMULATOR_HOST` (native-only — no Docker fallback).
- `preset = "lumen"` — a versioned native Lumen service. Set
  `version = "lumen@X.Y.Z"` to pin a release; omit it to resolve the newest
  `lumen@*` release. VAT downloads the target-native archive into its own cache,
  verifies a published checksum when present, starts `lumen serve` on loopback,
  waits for `/readyz`, and exports `LUMEN_URL`. It never replaces a global
  `lumen` installation and rejects Docker/MicroVM runtimes. Lumen state is
  ephemeral for the VAT run: no source build, import, seed, or persistence is implied.
- `image` — an OCI image dependency that has no native equivalent (e.g.
  AlloyDB). Requires `container_port`; `image_env` is passed into the container;
  `runtime = "docker"` uses Docker, while explicit `runtime = "micro_vm"` uses
  Apple Container with the same bounded inspect/pull/verify preflight and no
  Docker fallback. In `export`, `{host}`/`{port}` resolve to the mapped host
  endpoint and `VAT_SERVICE_<ID>_{HOST,PORT}` are always exported.
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
- `cmd` — an explicit native command. When the command owns an IPv4 endpoint on
  literal `127.0.0.1` (through `port`, `{host}`/`{port}`, or a fixed
  `127.0.0.1` `ready_http`), vat reserves that exact endpoint for the run
  through preparation and releases it only at the child-spawn boundary.
  `localhost`, `::1`, and other loopback spellings are rejected because they
  cannot be proven by the same exact IPv4 reservation. An already occupied
  endpoint fails closed with the exact service and endpoint; vat never treats
  the existing listener as the owned child's readiness. After spawn, both the
  owned child and the endpoint's unavailable-to-ready transition must remain
  valid before a dependent runner starts. Declare
  `external = { host = "...", port = ... }` when attaching to an intentionally
  pre-existing listener.

Env export contract:

| Service backing | Default exports | `export` map semantics | Raw service vars |
|---|---|---|---|
| `preset` datastore/broker | postgres/mysql → `DATABASE_URL`; redis → `REDIS_URL`; nats → `NATS_URL`; rabbitmq → `AMQP_URL`; mongo → `MONGODB_URI`; opensearch → `OPENSEARCH_URL` | Value containing `{host}`/`{port}` uses the map key as the env var name; otherwise the value is a legacy alias name receiving the default URL. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
| `preset` built-in emulator | `PUBSUB_EMULATOR_HOST`, `FIREBASE_AUTH_EMULATOR_HOST`, `CLOUD_TASKS_EMULATOR_HOST`, `CLOUD_SCHEDULER_EMULATOR_HOST`, `CLOUD_WORKFLOWS_EMULATOR_HOST`, `STORAGE_EMULATOR_HOST`, `VAT_HTTP_MOCK_HOST`, or `OPENAPI_MOCK_HOST` | Same template/alias rule as other presets. `STORAGE_EMULATOR_HOST` includes `http://`; the others are host:port unless documented by the service. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
| `preset` Lumen | `LUMEN_URL=http://127.0.0.1:<port>` | Same template/alias rule as other presets. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
| `image` | none | Key is always the env var name; value may use `{host}`/`{port}`. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
| `external` | none | Key is always the env var name; value may use `{host}`/`{port}` from the attached endpoint. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT`; state records `owned_by_vat = false` |
| `cmd` | `VAT_SERVICE_<ID>_URL` when `ready_http` exists and no custom export is set | Value containing `{host}`/`{port}` uses the map key as the env var name; otherwise the value aliases `ready_http`. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` only when the command needs/allocates a port |
| `cluster` | `KUBECONFIG` | `{kubeconfig}` expands to the isolated kubeconfig path. | `VAT_SERVICE_<ID>_KUBECONFIG` |

Runner scripts can detect a configured vat run with `VAT_WORKSPACE_BASE`; it is
set for `vat.toml` runner and scenario modes and points at the source workspace
that vat cloned. When `vat run --plan <path>` is used, vat copies that opaque
plan into the rootfs, injects `VAT_PLAN_PATH` and `VAT_PLAN_DIGEST`, and records
the plan evidence in `vat state`; vat never interprets the plan semantics.

For the native path vat checks for required binaries, cold-prepares cached
service data when needed, and clones it on later runs. Native preset and
endpoint-bearing `cmd` services reserve their exact `127.0.0.1` endpoints until
spawn, then require the owned child to stay live while the endpoint becomes
ready; a probe response from a pre-existing listener cannot certify ownership.
The Docker path runs an
ephemeral `docker run --rm` container bound to loopback; the explicit MicroVM
path runs an ephemeral Apple `container run --rm` service after the bounded
image preflight, with stricter host-port readiness evidence. Both are removed at
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
