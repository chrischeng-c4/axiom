// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-commands-llm-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `vat llm` — compact agent-facing usage contract.

use std::process::ExitCode;

use anyhow::Result;

/// Stable guide text intended for LLM/tool agents.
/// @spec apps/vat/tech-design/logic/llm-agent-usage-guide.md#cli
const GUIDE: &str = r#"# vat LLM Guide

vat is a local, ephemeral agent test runner. Use it to prepare a real local
workspace, run one command or one named vat.toml runner, and inspect structured
evidence afterward.

## First Choice

- If the project has `vat.toml`, prefer plain `vat run`.
- If `vat.toml` declares `[[scenarios]]` for an app-under-test, use
  `vat run --scenario <id>`.
- Use `vat run <runner-id>` only when you need a non-default runner.
- Use `vat capabilities --json` to inspect this host's effective substrate:
  COW clone method, isolation backends, Docker provider/daemon state,
  service-provider capabilities, and the Apple Container shared-builder
  advisory. This is a full host probe and retains its normal Docker daemon
  probe: `services.docker_services` is `available` or `unavailable` from that
  conclusive full probe. The builder record is bounded and read-only: `container builder
  status` reports `ownership=shared_unknown` and `automatic_cleanup=false`;
  parseable configured resources are distinct from optional live
  `observed_stats`, and optional `container system df` is global host evidence,
  not VAT-owned disk. Unsupported, malformed, or timed-out status/stats/df is
  nonfatal advisory `unknown`/`probe_errors`; VAT never starts, stops, deletes,
  or prunes the shared builder/cache. A real live state appears only when the
  installed Apple Container CLI supports and returns it.
- Use `vat plan --json` to inspect selected runners, services, env keys, and
  artifacts without creating a vat or starting services.
- Use `vat doctor --json` for cheap selected-plan preflight before a CI/local
  run. An explicit MicroVm/Apple-Container-only selected plan performs exactly
  one read-only `container system status` probe per invocation and projects it
  to its selected MicroVm services; it does not execute Docker even when
  Docker is on `PATH`, and JSON reports
  `docker.daemon_probe.state=skipped` with the truthful reason
  `Docker daemon probe skipped for Apple-Container-only selected plan`; in that
  deliberate no-probe state `services.docker_services=not_probed`.
  `docker.daemon=false` is not Docker-unavailable evidence because no Docker
  command ran. Unselected Docker services do not affect that
  runner. Docker runtime, Auto image, eligible Auto preset fallback, and
  selected cluster plans retain the Docker probe (a cluster needs Docker).
  Doctor neither autostarts Apple Container nor falls back to Docker;
  unsupported MicroVm presets without a declared OCI route and MicroVm preset
  named volumes fail closed. Its shared-builder result is advisory only, so a
  builder timeout/unknown/probe error never changes the runtime success result.
- If an upstream planner/TIA tool selected tests, pass the opaque file with
  `vat run --plan impact.json <runner-id>`; vat copies and records it but does
  not interpret test-selection semantics.
- If you only need one ad-hoc command, use `vat run -- <command>`.
- `vat run` prints sparse JSONL checkpoints; the final line has
  `"type":"result"`.
- After a retained run, inspect `vat state <id>`, `vat diff <id>`, and
  `vat logs <id> [runner|service-id]`.
- Use `vat fork <id> [--name N]` to branch a retained vat into a new runnable
  copy that carries its lineage, and `vat snapshot <id> [--name N]` to freeze
  one into an immutable, non-runnable point-in-time copy.
- Use `vat gpu --json` to report the GPU every vat on this host can reach.
- For Docker-free, one-command local Kubernetes work on Apple Container, first
  ensure an independently installed `kubectl` is first on `PATH` (VAT rejects
  an OrbStack-provided binary), run `vat k8s ephemeral image build`, then run
  `vat k8s ephemeral run -- kubectl get nodes` (or another host command). VAT
  injects a private `KUBECONFIG`, `VAT_K8S_CACHE_DIR`, and
  `VAT_K8S_API_SERVER` only into that command and deletes them with the exact
  machine after a confirmed create. If create terminal completion is uncertain,
  VAT retains a non-secret recovery marker and fails safely instead of claiming
  cleanup. Its final stdout line is a terminal
  `vat_k8s_ephemeral_result` JSON record, even when the child exit code is
  forwarded unchanged.
- For an agent sequence such as apply → inspect → test → delete, use
  `vat k8s session create --ttl 30m` after the same image build. Keep the
  returned id, then use text `vat k8s session exec --timeout 30 <id> -- kubectl
  ...` or agent JSON `vat k8s session exec --format json --timeout 30 <id> --
  kubectl ...` for each host command and finish with `vat k8s session delete
  <id>`. Omit `--timeout` only when the remaining lease TTL is the intended
  bound; an explicit timeout is 1..=14400 seconds and cannot exceed that TTL.
  Each exec re-inspects the exact backing id and API endpoint and re-verifies
  host API access before injecting the private kubeconfig, rechecks the lease
  at spawn, owns the child process group, and holds the private operation lock
  through cleanup. Normal exit, deadline, or SIGINT/SIGTERM reaps the group;
  VAT removes its private exec marker only after that group is absent. If VAT
  crashes, a starting or live marker blocks later exec, delete, and cleanup
  fail-closed rather than claiming an arbitrary recovered command was stopped.
  JSON emits one `vat.k8s.session.exec.v1` document with separate bounded
  stdout/stderr, the child exit code, no raw-stream replay, and a
  `status --verify-api` next step; its child intentionally has private
  credentials, so it is not an untrusted-child boundary. A session is one-boot
  and lease-bounded, not restart-safe or daemon-managed; `vat k8s session
  cleanup` reclaims expired leases and abandoned creates. The independent-
  kubectl leased real-host E2E passed 1/1 (36 filtered) in 29.97s, including
  text commands, strict JSON exec with `--timeout 30`, status verification, and
  exact delete. It does not establish crash-safe termination or persistence.
- `vat k8s session status <id>` is unchanged and reports non-secret lease and
  exact-machine state only. `vat k8s session status --verify-api <id>` is an
  opt-in bounded proof, not a lifecycle operation: it proceeds only for an
  active, unexpired session with no retained port-forward or exec marker. Under the
  private operation lock it rechecks expiry, proves the exact backing identity,
  endpoint, and private credential material, rechecks expiry immediately before
  one bounded API probe, and reports `api_checked=true`, `api_state=reachable`
  on success. Expired or recovery-marker state returns the non-probing
  `api_checked=false`, `api_state=not_checked`; busy, unavailable, or
  identity-mismatched state fails closed without changing lease or credentials.
  Focused fake coverage passed 4/4 and the precise status unit passed 1/1. The
  independent-kubectl leased E2E passed 1/1 (36 filtered) in 29.97s and includes
  `status --verify-api` after text and strict JSON exec; it is bounded active-
  lease evidence, not persistence or a general API-status guarantee.
- If either one-shot or leased K3s bootstrap fails, VAT renders the original
  error first, then emits only six fixed read-only diagnostics within a
  six-second total and one-second-per-probe budget:
  `guest_install_log`, `guest_k3s_system`, `backing_container_logs`,
  `machine_boot_log`, `machine_inspect`, and `container_system_status`.
  `guest_install_log` is staged non-sensitive installer evidence; private
  kubeconfig/cache and host credentials are excluded. This diagnostic path
  leaves the existing 300-second bootstrap behavior unchanged, does not retry
  or rerun `k3s --version`, introduces no wrapper/recovery command, and still
  runs exact cleanup. The deterministic fake regression passed. The independent-
  kubectl Service-port-forward E2E passed 1/1 (36 filtered) in 49.57s: it loaded
  the local alpine fixture, used an in-pod HTTP probe because BusyBox lacks
  `httpd`, verified the Service endpoint, text and strict one-document JSON
  loopback forwarding to a credential-free host child, confirmed cleanup and
  closed local ports, then deleted the exact active lease. This remains one
  Service-only session; it does not establish persistence or OS-sandbox behavior.
- To use a locally built or already-pulled image without Docker or a registry,
  run `vat k8s session image load <id> <local-image-ref>` before the Kubernetes
  workload. VAT requires exactly one locally inspected `linux/arm64` variant,
  saves it to a private bounded OCI archive, imports it into that lease's
  `k8s.io` namespace, verifies the canonical reference, and removes the host
  and guest archives before reporting success. It accepts no arbitrary tar;
  use `imagePullPolicy: Never` to prove a workload used the local image. The
  opt-in local-image real-host E2E passed 1/1 (36 filtered) in 49.73s: one
  already-local Apple `alpine:3.20` loaded into one active lease, a pod ran it
  with `imagePullPolicy=Never` and emitted its marker log, then exact session
  cleanup completed. This is not registry-pull generality, persistence, GUI,
  or Docker Engine/API evidence.
- To test one active K3s Service from a host assertion without injecting cluster
  credential variables into that assertion's child environment, run text
  `vat k8s session port-forward run <id> service/<name> <remote-port> --
  <host-command>` or the only JSON form `vat k8s session port-forward run
  --format json <id> service/<name> <remote-port> -- <host-command>`. VAT
  accepts only one literal Service selector and starts kubectl with `--address
  127.0.0.1`; `--local-port 0` lets kubectl choose the loopback port. It waits
  for readiness, then gives exactly one foreground host child only
  `VAT_K8S_PORT_FORWARD_HOST`, `_PORT`, `_ADDR`, `_RESOURCE`, and `_NAMESPACE`
  plus a private HOME. VAT strips `KUBECONFIG`, `VAT_K8S_CACHE_DIR`,
  `VAT_K8S_API_SERVER`, `VAT_K8S_EPHEMERAL`, and `VAT_HOME` from that child
  environment. This is credential hygiene, not a same-UID OS sandbox or
  adversarial-child security boundary.
  The host child joins the authenticated kubectl process group, so normal
  cleanup reaps the leader and waits for ordinary cooperative, non-daemonizing
  descendants to be gone; a child that daemonizes or escapes the group is out of
  contract. Each v2 marker carries a CSPRNG private recovery token, and its
  retained 0600 `operation.lock` is `CLOEXEC`, so kubectl/host work cannot keep
  the flock after a SIGKILLed VAT and the next mutating session operation can
  reconcile from the held lock rather than recorded owner-PID liveness. Recovery
  signals only a leader authenticated from that v2 identity
  and forward shape; a missing or changed leader leaves the marker in place and
  fails closed. Before unlinking storage VAT writes a durable `cleaning`
  tombstone, so torn cleanup is retried. Historical v1 markers are never
  signalled: they permit storage-only cleanup only after their recorded process
  group is already absent.
  Text forwards child output and starts its terminal record on a new line after
  that output. JSON waits until the shared group and private
  marker are confirmed cleaned, then emits exactly one
  `vat.k8s.session.port-forward.v1` `vat_json` document with the child exit,
  separate 64 KiB serialized-capped stdout/stderr, truncation/lossy flags, and
  a `status --verify-api` next step—never raw child-stream replay. VAT-owned
  setup, API, tunnel, and cleanup errors are masked; opaque credential-free
  child output in a successful result is not arbitrarily redacted. It silently
  rechecks the lease after API verification and immediately before both the
  exact kubectl and host-child spawns, so expiry starts no tunnel. A partial
  capture-reader setup reaps the direct child and completes outer-group cleanup
  before reader joining. This is not a public listener, ingress/LB, a background
  tunnel, or arbitrary resource port-forwarding. The independent-kubectl
  real-host gate passed 1/1 (36 filtered) in 49.57s after loading a local alpine
  fixture, using an in-pod HTTP probe because BusyBox lacks `httpd`, verifying
  the Service endpoint, text and strict one-document JSON loopback forwarding to
  a credential-free host child, confirmed cleanup and closed local ports, and
  exact lease deletion. It is not broader Kubernetes, ingress, public-listener,
  or detached-descendant evidence.
- The opt-in shim has five strict Apple-native JSON observation forms: direct
  container inventory, image inventory, direct image inspect, direct container
  inspect, and resource stats. Direct inventory is only `docker ps --format json`
  or `--format=json`,
  with optional exactly-once `--all` or `-a`; `docker container ls` and `docker
  container list` share it, while `docker container ps --format json` remains
  rejected; inherited text behavior is unchanged. VAT invokes `container list
  --format json [--all]`, validates one opaque Apple-native JSON value, then
  replays those stdout bytes unchanged without a VAT wrapper or Docker Engine
  `ps` schema. Templates/table output, filters, quiet plus JSON,
  duplicate/unknown flags, and positionals fail before Apple Container starts.
  It is a read-only inventory snapshot, not ownership/health/readiness/liveness
  proof. One five-second deadline plus bounded isolated-process-group cleanup
  cover root exit and both pipe EOFs; stdout/stderr are each capped at 256 KiB,
  and malformed, oversized, or escaped-pipe stdout fails closed without replay.
  `cargo check -p vat --no-default-features` passed; the shared `docker_shim`
  library passed 54/54 and focused direct-ps integration passed 4/4. The full
  serial fake-shim aggregate is intentionally not recorded: an independent serial
  run exposed a nondeterministic pre-existing Compose JSON logs timing race.
  The real-host direct-observation gate passed 1/1 (50 filtered) on Apple Container
  1.1.0; `ps` is a global read-only inventory smoke observation, not a targeted
  ownership result, and proves one valid native JSON document only. Fake/unit
  tests prove byte-preservation and fail-closed behavior.
- Direct image inspect accepts only `docker image inspect --format json IMAGE`
  or `--format=json`. Exactly one JSON selector must precede exactly one opaque
  safe image reference (nonempty, with no leading `-`, whitespace, or control
  characters); templates, `--`, extra references, and every other option fail
  before Apple Container starts. VAT strips the selector, invokes only `container
  image inspect IMAGE`, bounded-captures and validates one opaque Apple-native
  JSON document, then byte-for-byte replays complete native stdout. A five-second
  bounded isolated observer covers root exit and both pipe EOFs; each stream is
  capped at 256 KiB, valid JSON preserves a nonzero child exit, and malformed,
  oversized, or escaped-pipe capture has no raw replay. It supplies no Docker
  image-inspect schema/template/Engine API, provenance, security, registry,
  build-completion, readiness, or secret-redaction claim. Recorded validation:
  cargo check passed; `cargo test -p vat --lib docker_shim -- --nocapture`
  passed 58/58; `RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim
  docker_image_inspect_json -- --nocapture` passed 4/4 with 1 ignored; and
  `RUST_TEST_THREADS=1 VAT_DOCKER_IMAGE_INSPECT_JSON_E2E_REQUIRED=1 cargo test
  -p vat --test vat_docker_shim apple_container_docker_image_inspect_json_contract
  -- --ignored --nocapture` passed 1/1 (61 filtered) in 1.21s. The host test
  proves only one direct `container image inspect alpine:3.20` invocation and
  one valid native document; fake/unit tests prove selector stripping,
  byte-preservation, and fail-closed bounds.
- Image inventory is only `docker images --format json` or `--format=json`;
  `docker image ls` and `docker image list` share that exact form. VAT invokes
  `container image list --format json`, bounded-captures and validates one opaque
  Apple-native JSON value, then byte-for-byte replays stdout without a VAT wrapper
  or Docker Engine image schema. Template/table/YAML/TOML output, filters, quiet,
  verbose, all, digests, no-trunc, positionals, duplicates, unknown flags, and
  `--` fail before Apple Container starts; inherited text/quiet image-list behavior
  is unchanged. This read-only snapshot makes no ownership, provenance, security,
  executability, registry, build-readiness, health, readiness, or liveness claim.
  One five-second deadline plus bounded isolated-process-group cleanup cover root
  exit and both pipe EOFs; stdout/stderr are each capped at 256 KiB, and malformed,
  oversized, or escaped-pipe stdout fails closed without replay. `cargo check -p
  vat --no-default-features` passed; shared `docker_shim` library validation passed
  54/54 and focused `docker_images_json` integration passed 4/4. The full serial
  fake-shim aggregate is intentionally not recorded: an independent serial run
  exposed a nondeterministic pre-existing Compose JSON logs timing race. The
  real-host direct-observation gate passed 1/1 (50 filtered) on Apple Container 1.1.0;
  `images` is a global read-only inventory smoke observation, not a targeted
  ownership result, and proves one valid native JSON document only. Fake/unit
  tests prove byte-preservation and fail-closed behavior.
- Direct container inspect accepts only `docker inspect --format json CONTAINER`
  or `--format=json`; `docker container inspect` shares that exact form. Exactly
  one safe explicit container id follows exactly one JSON selector, which must
  precede the id and is VAT-only, never forwarded. Unformatted inspect retains its existing
  generic behavior. VAT invokes canonical `container inspect CONTAINER`,
  bounded-captures and validates one opaque Apple-native JSON value, then
  byte-for-byte replays stdout without a VAT wrapper or Docker Engine inspect
  schema. `--type`, `--size`, templates/table/YAML/TOML, filters, a second id,
  `--`, and unknown flags fail before Apple Container starts. A five-second
  bounded isolated observer covers root exit and both pipe EOFs; stdout and stderr
  are each capped at 256 KiB. Valid native JSON with a nonzero child exit preserves
  that exit status; malformed, oversized, or flood output suppresses raw stdout.
  It makes no Docker Engine inspect-schema, ownership, provenance, security,
  image, registry, build-status, health, readiness, liveness, or port-reachability
  claim, and supplies no secret-redaction guarantee. `cargo check -p vat
  --no-default-features` passed; shared `docker_shim` library validation passed
  54/54 and focused `docker_inspect` integration passed 5/5. The full serial
  fake-shim aggregate is intentionally not recorded: an independent serial run
  exposed a nondeterministic pre-existing Compose JSON logs timing race. The
  real-host direct-observation gate passed 1/1 (50 filtered) on Apple Container 1.1.0;
  `inspect` targets the temporary owner-labeled nginx container and proves one
  valid native JSON document only. Fake/unit tests prove byte-preservation and
  fail-closed behavior.
- Direct logs JSON is a separate finite VAT wrapper, not a sixth Apple-native
  JSON observation. Only `docker logs --format json --tail LINES CONTAINER` or
  equals forms reach it through direct `logs` or `docker container logs`; one
  format and one tail may use mixed spellings, must precede one safe final id, and
  `LINES` is 1..=1000. Unformatted logs keep inherited text translation. VAT
  invokes only `container logs -n LINES CONTAINER`; the Docker JSON selector is
  never forwarded. Apple has text stdout only and no multiplex/demux contract, so
  stdout is exactly one `vat.docker.logs.v1` / `vat_json` wrapper with untrusted
  `apple_container_stdio`, bounded diagnostic stderr, truncation/lossy flags,
  backend/container/requested_tail/runtime/child outcome, and a safe inspect next.
  An ordinary child nonzero keeps the wrapper and exit code; timeout, setup
  failure, or escaped-pipe capture fails closed without a partial wrapper. VAT
  observes for five seconds, then has one second for cleanup while draining both
  pipes; it retains suffixes and caps each capture plus the actual serialized JSON
  string value at 64 KiB. Follow, boot, timestamps, since/until, templates, and
  every other modifier fail before Apple Container. It makes no Docker schema,
  multiplex/demux, ownership, provenance, security, image, registry, build,
  health, readiness, liveness, port-reachability, or secret-redaction claim.
  `cargo check -p vat --no-default-features` passed; canonical `cargo test -p vat
  --lib docker_shim -- --nocapture` passed 54/54; focused `docker_logs_json`
  integration passed 6/6. The full serial fake-shim aggregate is intentionally
  not recorded: an independent serial run exposed a nondeterministic pre-existing
  Compose JSON logs timing race. `VAT_DOCKER_SHIM_E2E_REQUIRED=1 cargo test -p vat
  --test vat_docker_shim apple_container_docker_run_published_port_contract --
  --ignored --nocapture` passed 1/1 (50 filtered) on Apple Container 1.1.0: VAT
  logs targets a high-entropy nonce+PID owner-labeled temporary nginx container.
  Cleanup uses a high-entropy name and exact owner-label rechecks as conservative
  best-effort precautions; the emergency guard retains the container on uncertainty.
  Apple Container has no atomic conditional delete, so this is not a race-free
  guarantee against a TOCTOU replacement; it does not clean up the shared/cacheable
  nginx image. The host smoke proves one
  VAT wrapper only;
  fake/unit tests prove byte-preservation and fail-closed behavior.
- Direct exec JSON is a separate finite VAT wrapper for a foreground command,
  not Apple-native JSON or a Docker Engine stream/TTY contract. It accepts only
  `docker exec --format json --timeout SECONDS CONTAINER -- COMMAND [ARG...]`
  (or equals forms) and the same form through `docker container exec`: exactly one
  format and one timeout may occur in either order before one safe container id,
  `SECONDS` is 1..=1200, and the Docker-facing literal delimiter plus at least one
  command argument are mandatory. Unformatted/raw exec keeps inherited generic
  translation, including selector-looking raw arguments after its delimiter. VAT
  strips the selectors and Docker-only delimiter, then invokes Apple Container as
  `container exec CONTAINER COMMAND [ARG...]`. Stdout is exactly one
  `vat.docker.exec.v1` / `vat_json` wrapper with backend/container/requested
  timeout, `timeout_scope=host-container-client-observation`, runtime/child
  outcome, bounded untrusted stdout/stderr suffixes and truncation/lossy flags,
  no secret-redaction guarantee, and a safe inspect next. An ordinary child
  nonzero keeps the wrapper and exit code; timeout or setup/capture failure emits
  no partial wrapper. Both pipes drain concurrently and each serialized JSON
  string is capped at 64 KiB. The timeout bounds only VAT's host Apple Container
  client observation, not a guest command's termination. TTY, interactive,
  detach, environment/user/workdir, templates, duplicate/misordered selectors,
  malformed delimiters, and every other exec flag fail before runtime. `cargo
  check -p vat --no-default-features` passed; canonical `cargo test -p vat --lib
  docker_shim -- --nocapture` passed 54/54 and focused `docker_exec_json`
  integration passed 4/4. The direct-observation E2E passed 1/1 (50 filtered) on
  Apple Container 1.1.0 and observed one exec wrapper with both stdout and stderr
  markers. This is bounded direct-command evidence only, not guest-timeout
  termination, Docker Engine parity, generic runtime, Compose, or Kubernetes
  evidence.
- Strict direct run JSON is a foreground, owner-cleaned one-shot: only direct
  `docker run --format json --timeout SECONDS IMAGE [COMMAND...]` (or equals
  forms) is accepted. Exactly one format and one 1..=1200 timeout may occur in
  either order before IMAGE; optional command argv follows IMAGE directly. The
  JSON form rejects a Docker `--` before IMAGE or immediately after IMAGE;
  after the first non-`--` command token, later `--` is opaque child argv. It
  also rejects detach, TTY, interactive, caller name/label, ports, network,
  mounts, env, and every other run option before Apple Container starts. VAT
  generates a high-entropy name and independent
  owner label, captures bounded stdout/stderr into exactly one
  `vat.docker.run.v1` / `vat_json` document, and exposes a normal nonzero child
  exit only after exact owner-label cleanup confirms absence. Timeout, setup, or
  cleanup uncertainty emits no partial wrapper; only Apple's explicit
  `Error: container not found: <name>` proves an already-absent generated
  container. The timeout bounds host Apple Container client observation only;
  it makes no guest-wide termination, crash-recovery cleanup, Docker Engine
  parity, or secret-redaction claim. Focused deterministic validation passed 5
  passed plus 1 ignored in 1.80s. The real local `alpine:3.20` E2E passed 1/1
  (56 filtered) in 2.30s with one JSON document and exact cleanup.
- Strict direct build JSON is a separate bounded VAT receipt, not a sixth
  Apple-native JSON observation. Only direct `docker build --format json --timeout
  SECONDS --tag TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE]
  [--platform PLATFORM] [--label K=V ...] CONTEXT` (or documented equals forms)
  reaches it. Format `json`, a positive whole timeout in 1..=1200, and tag occur
  exactly once; file/target/platform occur at most once; build args/labels may
  repeat; and every option precedes exactly one canonical existing local-directory
  context. `--`, missing/duplicate/misordered selectors, a second context, and
  unsupported flags fail before the builder; raw builds without either receipt
  selector retain the inherited translator. VAT strips only its JSON/deadline
  selectors and invokes public `container build --tag TAG [--file ...]
  [--build-arg ...] [--target ...] [--platform ...] [--label ...] CONTEXT`.
  After the Apple client exits it emits one `vat.docker.build.v1` / `vat_json`
  receipt with bounded untrusted stdout/stderr, truncation/lossy flags, timeout
  scope, and child outcome. `image_lifecycle=retained_no_auto_cleanup`: this
  product path does no cleanup and makes no ownership claim. Success safely
  hands off to strict `docker image inspect --format json TAG`; a normal child
  nonzero retains its receipt and exit but emits `terminal=build_failed` with
  `next=docker --help`, never an image-inspect handoff for a partial/replaced
  tag. Timeout, setup, or capture failure emits no receipt. The deadline only
  observes the host Apple Container client: it neither cancels builder work nor
  rolls back/removes an image. This is not Docker Engine/API, provenance,
  ownership, readiness, security, secret-redaction, cancellation, or rollback
  proof; build args, labels, and output remain opaque/untrusted. Cargo check
  passed; `docker_shim` lib passed 62/62; focused `docker_build_json` passed 4
  plus 1 ignored (63 filtered); `native_image_owner_guard...` passed 1/1 (67
  filtered); and the opt-in real-host receipt E2E passed 1/1 (67 filtered) in
  2.53s. That test proves one strict mapped build/receipt only. Its cleanup is
  test-only: a high-entropy tag and exact `io.cclab.vat.e2e-owner` label require
  exact native absence before build, an exact label recheck before delete, and
  exact absence afterward. Apple has no conditional build/delete, so races are
  best effort and ambiguity leaks; this never changes retained/no-auto-cleanup
  product behavior.
- Strict direct pull JSON is a separate bounded VAT receipt, not an Apple-native
  JSON observation. Only direct `docker pull --format json --timeout SECONDS
  IMAGE` (or documented equals forms) reaches it: exactly one `json` format and
  one positive whole 1..=1200 timeout may be reordered before one safe opaque
  image reference. Empty, leading-dash, whitespace/control, URL-style `://`, and
  leading Git-style `git@` remote forms reject; ordinary OCI `@digest` remains
  opaque. `--`, a second reference, missing/duplicate/misordered
  selectors, and unsupported flags fail before the Apple client. Raw direct pull
  without either receipt selector and every `docker image pull` form retain their
  inherited paths. VAT strips only the JSON/deadline selectors and invokes public
  `container image pull IMAGE`. After that Apple client exits it emits one
  `vat.docker.pull.v1` / `vat_json` receipt with bounded untrusted stdout/stderr,
  truncation/lossy flags, timeout scope, and child outcome.
  `image_lifecycle=not_owned_no_auto_cleanup`: the shared image is neither
  VAT-owned nor cleaned, and VAT implements no registry login/auth/credential
  lifecycle. Success safely points to `docker image inspect --format json IMAGE`
  without proving image state or download completion. A normal child nonzero
  keeps its receipt and exit but is `terminal=pull_failed` with
  `next=docker --help`, never a stale inspect handoff. Timeout, setup, capture,
  or pipe failure emits no receipt. The deadline observes only the host Apple
  Container client and copied pipes; it does not cancel a registry transfer or
  guarantee download completion, rollback, or local/backend image state. This
  makes no Docker Engine/API, registry-management, provenance, digest, platform,
  freshness, image-state, ownership, security, secret-redaction, cancellation,
  download-completion, or rollback claim. Cargo check passed; `docker_shim` lib
  passed 65/65; focused `docker_pull_json` passed 5 plus 1 ignored (68 filtered);
  and the opt-in real-host receipt E2E passed 1/1 (73 filtered) in 27.14s. That
  E2E invokes real `container image pull alpine:3.20` against a shared/cacheable
  image; it neither deletes the image nor asserts ownership on success or
  failure.
- Resource observation remains only `docker stats --no-stream --format json
  CONTAINER [CONTAINER...]` (or `--format=json`). Both flags must occur before
  one or more explicit container ids; streaming, templates, `--all`,
  duplicate/unknown flags, and options after an id fail before Apple Container
  starts. VAT invokes `container stats --format json --no-stream …`, validates
  one Apple-native JSON document, then replays that exact JSON without a VAT or
  Docker Engine wrapper/schema. It is read-only, not
  ownership/health/readiness/liveness proof. One five-second observation deadline
  and bounded isolated-process-group cleanup govern root exit and both pipe EOFs.
  VAT replays stdout only after complete bounded capture and native JSON
  validation; an escaped pipe holder fails closed without stdout replay. Stdout
  and stderr each have a 256 KiB capture cap; malformed or oversized stdout is
  suppressed, not partially replayed. Shared `docker_shim` library validation
  passed 54/54. The full serial fake-shim aggregate is intentionally not recorded:
  an independent serial run exposed a nondeterministic pre-existing Compose JSON
  logs timing race. The real-host direct-observation gate passed 1/1 (50 filtered) on
  Apple Container 1.1.0; `stats` targets the temporary owner-labeled nginx
  container and proves one valid native JSON document only. Fake/unit tests prove
  byte-preservation and fail-closed behavior.
- For a Docker-shaped Apple Container lifecycle, install the opt-in shim
  (`vat docker install-shim --dir <directory-on-PATH>`). Its Compose surface has
  exactly three profiles: `strict-single-image-v1` is one literal-image service
  with `docker compose -f FILE -p PROJECT up -d`;
  `strict-single-build-v1` is one literal short `build: <context>` service with
  no `image:` and `up -d --build`; and `host-facing-independent-v1` requires
  the exact top-level marker
  `x-vat-compose-profile: host-facing-independent-v1`. The host-facing profile
  accepts two through four literal-image services, each with one unique nonzero
  `host:container` port published only on loopback. It has no bridge network or
  service-name DNS: its successful JSON declares
  `profile=host-facing-independent-v1`, `service_name_dns=false`, and
  `host_loopback_only=true`. Before a real launch, use
  `docker compose --dry-run -f FILE -p PROJECT up -d [--build]` to parse only
  those existing strict profiles. It emits exactly one
  `vat.docker-compose.preflight.v1` JSON document with `validated=true`,
  `runtime_started=false`, `registry_written=false`, `image_built=false`,
  `launch_revalidates=true`, structured `launch_argv`, and an executable `next`.
  It never calls Apple Container, builds, imports, starts, or writes a registry;
  it rejects `--wait` and every other global/Compose flag, and the returned
  launch uses the parser's canonical source path so an agent may change cwd and
  still revalidates the same file before import or runtime start.
  `docker compose -f FILE -p PROJECT up -d --wait
  [--wait-timeout SECONDS]` is VAT's bounded readiness protocol, not generic
  Compose: explicit `-d`/`--detach` remains required, one `--wait` is
  accepted, and `--wait-timeout` is legal only with it as positive whole
  seconds (default 300, maximum 1200). The budget starts after validated import
  and any source build, immediately before detached launch, and covers handoff
  plus observations. It waits only durable VAT runner readiness/topology proof,
  never Docker healthchecks, application HTTP, or service DNS. The waiter is
  pinned to profile/generation/ticket and releases the registry lock between
  polls, so it cannot attach after down, re-import, or relaunch. Ready produces
  one final up JSON with `wait` and ready topology; timeout retains
  runtime/registry and offers `ps` only after a current target observation.
  Terminal, replaced, and bare-deadline failures have no unsafe next; degraded
  has no endpoint. A source build's `cleanup_next` is emitted only for
  verified ready wait success. `docker compose -p PROJECT ps` has two exact
  output shapes. The no-format form preserves its text surface and ends with
  additive JSON retaining the known `profile` plus
  `topology={phase,ready,services}`. `--format json` and `--format=json`
  instead emit exactly one VAT-owned document with
  `schema=vat.docker-compose.ps.v1` and `format=vat_json`, the same claim-held
  profile/topology proof, and no human table. `phase` is one of inactive, starting,
  ready, degraded, or stopping; service entries follow registered Compose
  service-ID order, not runtime evidence order. An `endpoint` is only the
  canonical string `127.0.0.1:<port>`, and VAT emits every endpoint only when
  every expected service has exactly one Ready VAT-owned `container_run`
  record for its exact MicroVM, a nonzero loopback port, and no cleanup error.
  Otherwise a nominally ready project is `phase=degraded, ready=false` with no
  endpoints; starting, stopping, and inactive also have no endpoints. This is
  lifecycle/ownership evidence, not an app health check. The JSON mode is not
  Docker Compose JSON/template/table compatibility; every other `ps` format
  fails closed. Generic, missing, or unknown shim provenance also fails closed
  before VAT emits topology. Text `logs SERVICE` preserves observed log bytes,
  then starts its additive VAT handoff JSON on a new line after them.
  `logs --format json [--tail LINES] SERVICE` (also
  `--format=json` / `--tail=N`, with the service final) emits exactly one
  capture-only `vat.docker-compose.logs.v1` JSON document. It has separate
  stdout/stderr snapshots, `tail_lines` default 200 and range 1..=1000,
  per-stream `truncated`/`utf8_lossy`, `capture_only=true`,
  `runtime_invoked=false`, and `compose_record_mutated=false`. VAT holds the
  existing claim/provenance then reads captured logs only: no Apple Container
  call or project.json mutation. VAT first caps each read and line tail, then
  after lossy UTF-8 plus JSON escaping retains a valid UTF-8 suffix whose
  serialized JSON string value remains within the same 64 KiB per-stream cap
  and marks it truncated; there are no topology/endpoints. Its `next` is
  VAT-native JSON ps. This
  is not Docker Compose merged/follow/timestamp/template compatibility;
  `--follow`, timestamps, and other flags fail closed. The full serial
  `vat_docker_shim` aggregate is intentionally not recorded because an
  independent serial run exposed a nondeterministic pre-existing Compose JSON
  logs timing race; its focused serialized-cap unit passed 1/1 for `0xff`-heavy
  and NUL/control-heavy streams after actual JSON serialization. The recorded
  opt-in real dual-service E2E includes this JSON logs shape for its bounded
  profile.
  `PROJECT` must already match
  `[a-z0-9][a-z0-9_-]*`. Dependencies, networks, volumes, host-facing build,
  interpolation, `--env-file`, and every unsupported Compose form fail before
  runtime launch. Then use `docker compose -p PROJECT ps`, `logs SERVICE`,
  text `exec -T SERVICE -- COMMAND`, agent JSON
  `exec -T --format json SERVICE -- COMMAND` (or `--format=json`), or `down`.
  Text exec preserves observed child bytes, then starts its additive VAT
  handoff JSON on a new line after them. Both forms require one same-read claim-held
  snapshot with known shim provenance and one exact unique ready VAT-owned
  MicroVM service; incomplete or ambiguous evidence fails closed. VAT parses
  and validates the Docker-facing `--` but does not forward it, invoking Apple
  Container as `container exec CONTAINER COMMAND [ARG...]`; JSON exec drops the claim
  immediately after spawning the authorized child. It emits exactly one VAT-native
  `vat.docker-compose.exec.v1` document with `profile`, `child_exit_code`,
  separate stdout/stderr, per-stream `truncated`/`utf8_lossy`,
  `runtime_invoked=true`, and `compose_record_mutated=false`; it replays no raw
  child output and has no topology/endpoints. Child streams drain concurrently,
  with each serialized JSON string value capped at 64 KiB. JSON format
  misordering, a missing delimiter, default TTY, and every other exec flag fail
  closed. This is not Docker Compose exec output compatibility. The full serial
  shim aggregate is intentionally not recorded because an independent serial run
  exposed a nondeterministic pre-existing Compose JSON logs timing race; the
  precise serialized-cap unit passed 1/1; the bounded real-host JSON-exec
  evidence is stated below. `up` emits
  `vat_docker_compose` plus an executable `next`; `ps`/`logs` end with
  `terminal=observed`, and `down` with `terminal=cleaned_up`. A successful
  source-build `up` additionally returns its exact VAT-owned `images` and a
  `cleanup_next` that runs `down && docker image rm <exact-tag>`; literal-image
  projects deliberately do not claim image ownership. Generic `vat compose`
  cannot operate a known shim record; an inactive generic re-import explicitly
  clears known provenance. Unknown inactive provenance allows only registry
  cleanup that preserves `vat.toml`; unknown active provenance requires a
  matching or newer VAT. Its deterministic fake coverage is supplemented by an
  opt-in gated real Apple Container dual-service E2E that passed 1/1 (50
  filtered) on this host in 4.54 seconds:
  `RUST_TEST_THREADS=1 VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED=1 cargo
  test -p vat --test vat_docker_shim
  apple_container_docker_compose_host_facing_independent_profile_contract --
  --ignored --nocapture`. It proves `host-facing-independent-v1` `up -d
  --wait`, both loopback endpoints, one-document JSON `ps`, `logs`, and `exec`,
  text logs, text exec including a no-final-newline handoff, and `down` cleanup
  of exact containers, ports, and registry. Text handoff ordering covers only
  bytes VAT observes from the managed child/log stream, not descendants that
  escape it. It remains opt-in and proves neither service-name DNS nor general
  Compose, a Docker Engine API, or Kubernetes.
- Use `vat --help` for flag syntax and `vat <command> --help` for command flags.

## vat.toml Contract

```toml
version = 1
default_runner = "e2e"

[workspace]
base = "."
workdir = "."
keep = "failed" # failed | always | never

[network]
egress = "open" # open | localhost-only | deny

[[services]]
id = "pg"
preset = "postgres"        # native binary preferred; Docker image fallback
# runtime = "auto"         # auto (default) | native | docker | micro_vm
seed = ["schema.sql", "fixtures.sql"]
export = { DATABASE_URL = "DATABASE_URL" }

[[services]]
id = "alloy"               # OCI image dependency (no native binary)
image = "google/alloydbomni:latest"
runtime = "micro_vm"       # explicit Apple Container route; never falls back to Docker
container_port = 5432
image_env = { POSTGRES_PASSWORD = "pw" }
export = { ALLOY_URL = "postgres://postgres:pw@{host}:{port}/postgres" }

[[services]]
id = "ci-pg"               # already started by GitLab CI services / Compose
external = { host = "postgres", port = 5432 }
export = { DATABASE_URL = "postgres://postgres@{host}:{port}/app" }

[[services]]
id = "k8s"                 # ephemeral local Kubernetes cluster
cluster = "auto"           # auto (kind→k3d→minikube) | kind | k3d | minikube
# k8s_version = "1.30"
# nodes = 1
export = { KUBECONFIG = "{kubeconfig}" }

[[services]]
id = "fs"                  # gcloud Firestore emulator (exports FIRESTORE_EMULATOR_HOST)
preset = "gcloud-firestore" # gcloud-firestore | gcloud-datastore | gcloud-bigtable | gcloud-spanner

[[services]]
id = "web"                 # app under test; {port} is auto-allocated
cmd = ["pnpm", "run", "dev", "--", "--host", "127.0.0.1", "--port", "{port}"]
ready_http = "http://127.0.0.1:{port}/"
export = { APP_URL = "APP_URL" }

[[services]]
id = "http"
preset = "http-mock"       # required by hermetic scenarios

[[runners]]
id = "e2e"
requires = ["pg"]
cmd = ["pnpm", "run", "test:e2e"]
artifacts = ["test-results/**", "playwright-report/**"]

[[scenarios]]
id = "prod-like"
app = "web"
requires = ["pg", "http"]
runner = "e2e"
network = "hermetic"       # open | hermetic
```

## Services: native, Docker, or explicit Apple Container, plus external sidecars

- A `preset` service prefers the native Homebrew binary and falls back to the
  preset's official Docker image when the binary is missing. Force it with
  `runtime = "native"` or `runtime = "docker"`. Explicit
  `runtime = "micro_vm"` never falls back to Docker: for presets with a declared
  OCI route it checks Apple's image store, emits `image_pull` and performs a
  bounded pull when needed, re-verifies the image, then runs Apple `container`
  with loopback port readiness. Unsupported presets and MicroVM preset named
  volumes fail closed. Datastore/broker presets: postgres, redis, nats,
  rabbitmq, mysql, mongo.
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
- An `image` service is an OCI dependency with no native binary (e.g. AlloyDB).
  `runtime = "docker"` uses Docker; explicit `runtime = "micro_vm"` uses Apple
  Container's bounded inspect/pull/verify preflight and never silently invokes
  Docker. It requires `container_port`; `image_env` is passed into the
  container; in `export`, `{host}`/`{port}` resolve to the mapped host endpoint,
  and `VAT_SERVICE_<ID>_{HOST,PORT}` are always exported.
- An `external` service is an already provisioned endpoint, such as a GitLab CI
  `services:` sidecar, GitHub Actions service container, local Docker Compose
  service, or host daemon. vat does not start or stop it; it waits for readiness,
  substitutes `{host}`/`{port}` in `ready_http`, `ready_cmd`, and `export`,
  injects `VAT_SERVICE_<ID>_{HOST,PORT}`, and records `owned_by_vat = false` in
  `vat state`.
- A `cmd` service is VAT-owned. For an IPv4 endpoint on literal `127.0.0.1`
  declared through `port`, `{host}`/`{port}`, or a fixed `127.0.0.1`
  `ready_http`, vat holds an exact run-scoped reservation through preparation
  and releases it only at the spawn boundary. `localhost`, `::1`, and other
  loopback spellings are rejected because they are not the same exact IPv4
  endpoint. An occupied endpoint fails closed with the exact service/endpoint;
  after spawn, the owned child must stay live and the endpoint must transition
  to ready before any runner starts. Use `external`, not `cmd`, to attach to an
  intentionally existing listener.
- Built-in emulators: `preset = "gcloud-pubsub"`, `"firebase-auth"`, `"gcloud-cloud-tasks"`,
  `"cloud-scheduler"`, `"cloud-workflows"`, and `"cloud-storage"` run vat's OWN
  in-process Rust emulator under `runtime = auto` — no gcloud, Java,
  firebase-tools, or Docker, and instant start. They export
  `PUBSUB_EMULATOR_HOST` / `FIREBASE_AUTH_EMULATOR_HOST` /
  `CLOUD_TASKS_EMULATOR_HOST` / `CLOUD_SCHEDULER_EMULATOR_HOST` /
  `CLOUD_WORKFLOWS_EMULATOR_HOST` / `STORAGE_EMULATOR_HOST` — point your client's
  base URL at `http://$HOST` for host:port vars; `STORAGE_EMULATOR_HOST` already
  includes `http://` because the GCS REST SDK expects a schemed endpoint.
  `cloud-tasks` (v2 REST) delivers each task's httpRequest to its
  target at scheduleTime (or `tasks/{t}:run`); `cloud-scheduler` (v1 REST) fires
  a job's httpTarget on its cron schedule or `jobs/{j}:run`; `cloud-workflows`
  (v1 REST) runs a subset Workflows interpreter whose `call: http.*` steps can
  orchestrate the other emulators; `cloud-storage` (GCS JSON API v1) is an
  in-memory object store (bucket CRUD, media/multipart upload, `alt=media`
  download, list, delete); `http-mock` is a transparent HTTP stub + record/replay
  proxy with HTTPS MITM — `preset = "http-mock"` exports `HTTP(S)_PROXY` + a
  CA-trust bundle so the runner's outbound third-party API calls (even hardcoded
  `https://`) are intercepted with no code change. Register stubs at
  `$VAT_HTTP_MOCK_HOST/__admin/stubs`; unstubbed calls record once then replay
  offline. `openapi` reads an OpenAPI document (`preset = "openapi"`,
  `spec = "api.yaml"`) and serves spec-derived responses (example, else a
  schema-synthesized body; path templating + `$ref`) — a working fake of a
  documented API with no stubs or recording. It runs standalone (the runner points
  its base URL at `$OPENAPI_MOCK_HOST`) and also backs the http-mock proxy:
  `POST $VAT_HTTP_MOCK_HOST/__admin/openapi` registers a spec for a host, so a
  proxied `https://` call is answered from the contract (resolution: stub >
  openapi > cassette > forward). `pubsub` still accepts `runtime = native`
  (gcloud) / `runtime = docker` (image) as a fidelity fallback; the others are
  built-in only (no official emulator exists).
- Pointing a client at `cloud-tasks` / `cloud-scheduler`: unlike `pubsub` /
  `firebase-auth` / `firestore` / GCS (whose SDKs auto-read their host var), the
  official Cloud Tasks / Cloud Scheduler SDKs do NOT read
  `CLOUD_TASKS_EMULATOR_HOST` / `CLOUD_SCHEDULER_EMULATOR_HOST` (Google ships no
  emulator) and default to gRPC, while vat serves REST — so an env/DNS host
  redirect fails. Build the client through one factory that, when the host var is
  set, forces the REST transport, an `http://$HOST` endpoint, and anonymous
  credentials. Python: `CloudTasksClient(transport="rest",
  credentials=AnonymousCredentials(), client_options={"api_endpoint":
  f"http://{host}"})`. Node: `new CloudTasksClient({fallback:'rest', apiEndpoint,
  port, protocol:'http'})`. Or skip the SDK and POST the v2 REST API directly
  (see `tests/vat_emulator_tasks.rs`).
- Removing mocks: declare the emulator presets your code touches (the runner hits
  real local services), add `http-mock` for arbitrary third-party HTTP, and
  `openapi` to fake a documented API from its spec — tests then need no
  hand-rolled service or HTTP-client mocks.
- Production-like scenarios: declare `[[scenarios]]` when you want vat to start
  the app-under-test plus dependencies and then run a test runner against it.
  `network = "hermetic"` requires a participating `preset = "http-mock"` service,
  sets localhost-only egress, defaults the run to seatbelt isolation, wraps
  direct-start app/dependency services, and records `test_run.scenario` topology
  in `vat state`.
- A `cluster` service spins up an ephemeral local Kubernetes cluster (kind, k3d,
  or minikube; `auto` picks the first installed). vat creates it before the
  runner, exports `KUBECONFIG` (the `{kubeconfig}` token) plus
  `VAT_SERVICE_<ID>_KUBECONFIG`, probes readiness with `kubectl get nodes`, and
  deletes it at teardown per the `keep` policy. With no backend it emits a
  structured `cluster_backend_unavailable` error (no panic). All backends need
  Docker on Apple Silicon.
- `vat k8s ephemeral` is not that persistent-cluster surface. It uses only an
  auto-boot Apple systemd machine plus its exact inspected backing container;
  it bootstraps one K3s node for one foreground host command and removes the
  private 0600 kubeconfig, kubectl cache, recovery marker, and exact machine
  afterwards. It does not use Docker, `container machine run`, a background
  daemon, a durable kubeconfig, or `vat cluster` state. Use
  `vat k8s ephemeral cleanup` only to reconcile a marker whose recorded VAT
  process is no longer alive.
- `vat k8s session` extends that Docker-free substrate only as a bounded active
  lease: `create --ttl 30m`, separate text `exec [--timeout SECONDS] <id> --
  kubectl ...` or one-document `exec --format json [--timeout SECONDS] <id> --
  kubectl ...` calls, `status`, then explicit `delete`. Omitted timeout means
  the remaining lease TTL; an explicit 1..=14400-second timeout cannot exceed
  it. Every exec owns a process group and holds the operation lock through
  cleanup; normal exit, deadline, or interrupt reaps the group before marker
  removal. A crash leaves a starting/live marker that blocks later lifecycle
  operations fail-closed rather than implying crash-safe termination. JSON exec
  captures separate stdout/stderr concurrently, retains only a serialized-
  JSON-bounded 64 KiB suffix per stream, and never replays raw child output; it
  does not turn the credential-bearing child into a sandbox. VAT keeps the
  private credential/cache directory at mode 0700/0600, does not print its
  path, rejects an expired lease or changed backing id/API endpoint, and removes
  credentials only after exact machine absence is confirmed. It has no
  background reaper; `session cleanup` reclaims expired leases and abandoned
  creates when an agent invokes it.
- `vat k8s session port-forward run` is a narrower leased-session operation:
  one `service/<lowercase-dns-label>` port is exposed only at `127.0.0.1` while
  one host child runs. Text behavior is unchanged; `--format json` is the only
  machine form. VAT holds the kubeconfig only for kubectl and gives that child
  endpoint metadata after stripping `KUBECONFIG`, K3s cache/API variables, and
  `VAT_HOME` from the child environment. That is not a same-UID OS sandbox or an
  adversarial-child security boundary. The child joins kubectl's authenticated
  process group and must remain cooperative and non-daemonizing. VAT owns that
  group, private cache, a v2 CSPRNG recovery marker, and a retained `CLOEXEC`
  operation lock through cleanup. Normal JSON completion emits one
  `vat.k8s.session.port-forward.v1` document only after group cleanup and marker
  removal are confirmed; it contains the child exit plus separately bounded
  64 KiB serialized stdout/stderr and never replays raw streams. VAT masks its
  own setup/API/tunnel/cleanup failures but does not arbitrarily redact opaque
  credential-free child output. It silently rechecks the lease after API proof
  and immediately before kubectl and host-child spawns; if it expires, no tunnel
  starts. A partial reader setup reaps the direct child and completes outer-group
  cleanup before readers join. An interrupted marker is reconciled under a later
  held lock rather than owner-PID liveness; if it cannot be authenticated, it is
  retained fail-closed before any further mutation. The independent-kubectl
  Service-forward E2E passed 1/1 (36 filtered) in 49.57s and covers one
  loopback Service text and strict JSON tunnel with a credential-free host child,
  confirmed cleanup, and closed local ports; it is not a general Kubernetes
  tunnel guarantee.
- Docker-backed services need a reachable Docker daemon; vat emits a structured
  `docker_unavailable` error (no panic) when it is missing. The runner itself is
  never containerized.
- Env export contract:

  | Service backing | Default exports | `export` map semantics | Raw service vars |
  |---|---|---|---|
  | `preset` datastore/broker | postgres/mysql -> `DATABASE_URL`; redis -> `REDIS_URL`; nats -> `NATS_URL`; rabbitmq -> `AMQP_URL`; mongo -> `MONGODB_URI`; opensearch -> `OPENSEARCH_URL` | If the value contains `{host}`/`{port}`, the key is the env var name; otherwise the value is a legacy alias receiving the default URL. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
  | `preset` built-in emulator | `PUBSUB_EMULATOR_HOST`, `FIREBASE_AUTH_EMULATOR_HOST`, `CLOUD_TASKS_EMULATOR_HOST`, `CLOUD_SCHEDULER_EMULATOR_HOST`, `CLOUD_WORKFLOWS_EMULATOR_HOST`, `STORAGE_EMULATOR_HOST`, `VAT_HTTP_MOCK_HOST`, or `OPENAPI_MOCK_HOST` | Same template/alias rule. `STORAGE_EMULATOR_HOST` includes `http://`; most other host vars are host:port. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
  | `image` | none | Key is always the env var name; value may use `{host}`/`{port}`. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` |
  | `external` | none | Key is always the env var name; value may use `{host}`/`{port}` from the attached endpoint. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT`; state records `owned_by_vat = false` |
  | `cmd` | `VAT_SERVICE_<ID>_URL` when `ready_http` exists and no custom export is set | Template values use the key as env var name; otherwise the value aliases `ready_http`. | `VAT_SERVICE_<ID>_HOST`, `VAT_SERVICE_<ID>_PORT` only when a port is allocated |
  | `cluster` | `KUBECONFIG` | `{kubeconfig}` expands to the isolated kubeconfig path. | `VAT_SERVICE_<ID>_KUBECONFIG` |

- Runner scripts can detect configured vat runner/scenario mode with
  `VAT_WORKSPACE_BASE`; it points at the source workspace that vat cloned.
- `vat run --plan <path>` copies the opaque plan into the rootfs, injects
  `VAT_PLAN_PATH` and `VAT_PLAN_DIGEST`, and records the same evidence in
  `vat state`. The wrapped app/test tool owns the plan semantics.
- macOS native TCP presets can hit `kern.ipc.somaxconn` under connection churn
  and produce intermittent `ECONNREFUSED` even while the service is up. vat emits
  a structured `hint` when a service log reports that backlog cap. Prefer app
  connection pooling or raise the host limit, e.g.
  `sudo sysctl -w kern.ipc.somaxconn=1024`.

## Isolation and Egress

- `--isolation none|seatbelt` (also `vat.toml`-less default: `none`) picks the
  sandbox backend. `none` runs the command as a plain host process confined
  only by the copy-on-write rootfs — full native GPU/IO, zero syscall
  confinement. `seatbelt` wraps it in a macOS `sandbox-exec` profile that
  confines writes to the rootfs + temp and can enforce `[network].egress`;
  Metal still works because it's still a host process.
- `[network].egress` (or per-scenario `network = "hermetic"`, which implies
  `localhost-only`) is `open` (default, no restriction), `localhost-only`
  (deny outbound except loopback + unix sockets — vat's local
  emulators/http-mock proxy stay reachable), or `deny` (block all outbound,
  including localhost).
- Egress enforcement fails closed, not silently: picking a backend that
  cannot actually enforce a non-`open` egress policy is a hard error, not a
  warn-and-continue. `--isolation none` with `[network].egress` set to
  anything but `open` refuses to run. `--isolation seatbelt` with
  `sandbox-exec` unavailable on the host and a non-`open` policy also refuses
  to run, rather than silently falling back to the unconfined `none` backend;
  it only falls back when the policy is already `open`.
- This applies uniformly to both direct-command mode (`vat run -- <cmd>`) and
  runner-mode `vat.toml` commands — a declared runner cannot bypass the
  spec's isolation/egress policy.

## Command Patterns

- `vat run`: select the default runner, prepare or clone service images, start
  required services, wait for readiness, run the runner, capture evidence, stop
  services, and return the runner exit code.
- `vat run --scenario prod-like`: start the named scenario's app service,
  scenario deps, and runner deps, then run its selected runner.
- `vat run e2e`: explicitly run the `e2e` runner.
- `vat run --keep always e2e`: override `[workspace].keep` for one invocation so
  a passing probe run remains inspectable via `vat logs` / `vat state`.
- `vat capabilities --json`: full host backend/isolation/Docker/service
  discovery without requiring vat.toml; it keeps the normal Docker daemon
  probe, so `services.docker_services` is conclusively `available` or
  `unavailable`, and adds a bounded read-only Apple Container shared-builder advisory.
  `builder status` gives shared ownership (`shared_unknown`) and no automatic
  cleanup; supported configuration, optional live stats, and host-global disk
  observations remain distinct. A timeout, unsupported output, or probe error
  is nonfatal advisory evidence; VAT never mutates the builder/cache.
- `vat plan --json [e2e]`: print the selected configured topology without side
  effects.
- `vat doctor --json [e2e]`: check only the selected topology's host
  prerequisites without running app/tests. An explicit MicroVm/Apple-Container
  plan probes read-only `container system status` exactly once per invocation,
  never Docker even if it is on `PATH`, and returns
  `docker.daemon_probe.state=skipped` with a selected-plan reason and
  `services.docker_services=not_probed`—not unavailable. `docker.daemon=false`
  has no unavailable meaning there because no Docker command ran. An unselected Docker service is
  irrelevant; Docker runtime, Auto image, eligible Auto preset fallback, and
  selected cluster plans retain Docker probing, and cluster requires Docker.
  Doctor does not autostart Apple Container or fall back to Docker: unsupported
  MicroVm presets without an OCI route and MicroVm preset named volumes fail
  closed. The separate shared-builder advisory may report timeout/unknown/error
  but never changes the runtime success result.
- `vat run --plan impact.json impacted`: expose an upstream plan to the runner
  through `VAT_PLAN_PATH` / `VAT_PLAN_DIGEST` and preserve plan evidence.
- `vat run -- cargo test -p app`: run one direct command without requiring
  vat.toml; the child exit code is forwarded.
- `vat logs <id> runner`: print retained runner stdout/stderr.
- `vat logs <id> <service-id>`: print retained service stdout/stderr.
- `vat state <id>`: read the agent-legible JSON state.
- `vat diff <id> --json`: read filesystem changes vs. the vat base.
- `vat gc --json`: dry-run retained workspace cleanup and report candidates
  without deleting anything.
- `vat gc --measure --json`: include `du -sk` disk sizes; omit it for fast
  metadata-only cleanup planning on huge stores.
- `vat gc --execute --keep-last 5`: prune old successful/created vats while
  preserving running, snapshot, failed, and newest retained vats.
- `vat cluster create [--backend auto|kind|k3d|minikube] [--name N]`: create a
  standalone local Kubernetes cluster (outlives a run); `vat cluster ls --json`,
  `vat cluster kubeconfig <name>`, and `vat cluster delete <name>` manage it.
- Every `vat k8s` command requires an independently installed `kubectl` first
  on `PATH`; VAT rejects an OrbStack-provided binary before K3s use. This is
  host-tool provenance, not a GUI or Docker Engine requirement. On this host
  Homebrew `kubernetes-cli` now supplies `/opt/homebrew/bin/kubectl`. The
  independent-kubectl one-shot, leased, local-image, and Service-forward E2Es
  passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s respectively.
  The local-image proof is one already-local Apple `alpine:3.20` pod with
  `imagePullPolicy=Never`, a marker log, and exact session cleanup—not
  registry-pull generality, persistence, GUI, or Docker Engine/API evidence.
- `vat k8s ephemeral image build`: explicitly build VAT's embedded systemd
  image into the Apple Container store. Its local tag identifies the embedded
  build asset revision, not a verified supply-chain image digest. It never
  starts a cluster.
- `vat k8s ephemeral run -- kubectl get nodes`: boot one disposable Apple K3s
  node, prove host API access, run that command with a private kubeconfig, then
  clean credentials and the exact owned machine. Its isolated HOME keeps
  kubectl's normal cache private; only a child shell can expand
  `$VAT_K8S_CACHE_DIR`, because arbitrary direct argv is never shell-expanded
  by VAT.
- Failed one-shot or leased K3s bootstrap keeps the root error first, then
  reports staged non-sensitive installer evidence through exactly
  `guest_install_log`, `guest_k3s_system`, `backing_container_logs`,
  `machine_boot_log`, `machine_inspect`, and `container_system_status` under a
  six-second total / one-second-per-probe read-only budget. It excludes private
  kubeconfig/cache and host credentials, preserves the existing 300-second
  bootstrap behavior, does not retry or rerun `k3s --version`, adds no
  wrapper/recovery command, and still performs exact cleanup. The deterministic
  fake regression passed. The independent-kubectl one-shot, leased, local-image,
  and Service-forward E2Es passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s,
  and 49.57s. The local-image result loads one already-local Apple `alpine:3.20`
  into one lease, runs a pod with `imagePullPolicy=Never`, observes its marker
  log, then completes exact session cleanup; it is not registry-pull generality.
  The leased result covers strict JSON exec with `--timeout 30`; the
  Service-forward result covers one Service-only loopback strict JSON tunnel,
  credential-free child, confirmed cleanup, and closed local ports. These are
  bounded one-guest results, not persistence or a general cluster claim.

- `vat k8s session create --ttl 30m`: create one bounded active Apple K3s lease
  and print its opaque id plus a runnable next command. `vat k8s session exec
  --timeout 30 <id> -- kubectl get nodes` is one bounded text invocation; omit
  the timeout only to use the remaining lease TTL. JSON uses
  `vat k8s session exec --format json --timeout 30 <id> -- kubectl get nodes -o
  json`. An explicit timeout is 1..=14400 seconds and cannot exceed remaining
  TTL. Both forms retain the private lock through owned-process-group cleanup;
  normal exit, timeout, or SIGINT/SIGTERM reaps the group and removes the exec
  marker only once absent. A crash marker blocks later exec, delete, and cleanup
  fail-closed rather than claiming recovery termination. After the same active-
  lease, exact-backing/API, private-credential, and owned API proof plus a final
  TTL recheck, JSON emits exactly one `vat.k8s.session.exec.v1` document with
  separate 64 KiB serialized-bounded streams, the child exit code, no raw replay,
  and no lease-record mutation. It masks private credential/cache paths on
  failure, but its child intentionally receives credentials. The independent-
  kubectl leased E2E passed 1/1 (36 filtered) in 29.97s, covering text commands,
  JSON exec with `--timeout 30`, status verification, and exact delete. No-flag
  `status <id>` remains lease/machine-state only. Opt-in `status --verify-api <id>`
  proceeds only for an active unexpired session with no retained port-forward or
  exec marker; it acquires the private operation lock,
  rechecks expiry after lock and immediately before its bounded private-
  credential API probe, and verifies the exact backing identity and endpoint.
  Success adds `api_checked=true`, `api_state=reachable`; expired/recovery
  states are non-probing `api_checked=false`, `api_state=not_checked`; busy,
  unavailable, and identity-mismatched state fails closed without lease or
  credential mutation. Focused fake coverage passed 4/4 and the precise status
  unit passed 1/1. The independent-kubectl leased E2E passed 1/1 (36 filtered)
  in 29.97s and includes `status --verify-api` after text and strict JSON exec;
  it is bounded active-lease evidence, not persistence or a general API-status
  guarantee. `delete <id>` confirms exact cleanup before removing credentials, and `cleanup` reclaims
  expired leases. This is explicitly one-boot, not a persistent/restartable
  local Kubernetes backend.
- `vat k8s session image load <id> <local-image-ref>`: deliver one already
  local Apple Container `linux/arm64` image to that active K3s lease without a
  Docker daemon or registry pull. VAT verifies one inspected variant and its
  OCI descriptor, keeps the transient OCI archive private and bounded, imports
  into `k8s.io`, verifies the canonical reference, then removes both archive
  copies. It does not accept arbitrary tar files or promise cross-platform
  delivery.
- `vat k8s session port-forward run [--format json] <id> service/<name>
  <remote-port> [--namespace <ns>] [--local-port <port>] -- <command...>`:
  start one foreground, loopback-only Service tunnel for one host child. Text
  forwards child output and starts its terminal record on a new line afterward;
  `--format json` is the only JSON spelling.
  The child gets `VAT_K8S_PORT_FORWARD_{HOST,PORT,ADDR,RESOURCE,NAMESPACE}` and
  a private HOME; VAT strips `KUBECONFIG`, K3s cache/API variables, and
  `VAT_HOME` from its environment. This is child-environment credential hygiene,
  not a same-UID OS sandbox or adversarial-child security boundary. The child
  shares the authenticated kubectl process group, so it and ordinary descendants
  must stay cooperative and non-daemonizing. VAT stops that group and removes
  forward state before terminal JSON reports `cleanup=confirmed`; recovery uses
  only a v2 CSPRNG-authenticated leader, persists a `cleaning` tombstone across
  torn cleanup, and fails closed rather than signal an unauthenticated group.
  A legacy v1 marker is never signalled and can only clear already-absent-group
  storage.
  `--local-port 0` selects an ephemeral loopback port. JSON holds the private
  operation lock through tunnel/group cleanup and emits exactly one
  `vat.k8s.session.port-forward.v1` result only after cleanup is confirmed; it
  preserves child exit, separately caps serialized stdout/stderr at 64 KiB, and
  supplies `status --verify-api` as next without raw-stream replay. VAT-owned
  setup/API/tunnel/cleanup errors are masked, while opaque credential-free child
  output is not arbitrarily redacted. Silent lease checks follow API verification
  and precede exact kubectl and host-child spawns; a crossed TTL creates no
  tunnel. Partial reader setup reaps the direct child and completes outer-group
  cleanup before reader join. Only Services are accepted; there is no
  pod/arbitrary-resource forwarding, ingress/LB, public bind, or background
  tunnel. The independent-kubectl Service-forward E2E passed 1/1 (36 filtered)
  in 49.57s, covering one loopback Service text and strict JSON tunnel with a
  credential-free host child, confirmed cleanup, and closed local ports. It is
  not a persistent Kubernetes, ingress/LB, public-listener, or general tunnel
  guarantee.
- `vat fork <id> [--name N]`: copy-on-write fork a retained vat's rootfs into a
  new runnable vat that records the source as its lineage; the fork is
  independent afterward (writes to one do not affect the other).
- `vat snapshot <id> [--name N]`: freeze a retained vat's rootfs into an
  immutable snapshot for later inspection or forking; a snapshot itself is not
  runnable.
- `vat gpu --json`: report the GPU(s) every vat on this host can reach,
  independent of any specific vat or run.

## Retention

Default `keep = "failed"` means successful configured runs clean up after
emitting JSON, while failed runs keep workspace state and logs for inspection.
Use `vat run --keep always ...` to retain one passing configured run without
editing `vat.toml`; use `--keep never` to force cleanup.
If retained vats accumulate, run `vat gc --json` first. GC is dry-run by
default, reads metadata only, and requires `--execute` before deleting. Add
`--measure` when `du -sk` disk sizes are needed. Add `--apparent` only when
file-length totals are needed; it walks every retained rootfs. Add
`--include-failed` only when failed debug workspaces are no longer needed.

## Boundaries

- vat is not a Docker Engine/API or general-Compose replacement, a Linux runtime, a VM, a daemon,
  or a long-lived process manager. It is permanently headless: GUI/Desktop,
  dashboard, and tray/menu-bar surfaces are out of scope.
- vat offers an opt-in, fail-closed `docker` command shim over Apple
  Container: run `vat docker install-shim --dir <directory-on-PATH>`, add the
  directory to PATH, then use its documented command subset. It requires an
  explicit host port for `docker run -p` and rejects unsupported flags before
  runtime launch. Its five strict Apple-native JSON observations are direct
  container inventory, image inventory, direct image inspect, direct container
  inspect, and resource stats. Direct inventory accepts only `docker ps
  --format json` / `--format=json`, with optional exactly-once `--all` or `-a`;
  only `docker container ls` and `docker container list` share it, while
  `docker container ps --format json` remains rejected; inherited text behavior
  is unchanged. It invokes `container
  list --format json [--all]`, validates one opaque Apple-native JSON value,
  then byte-for-byte replays stdout without a VAT wrapper or Docker Engine `ps`
  schema. Templates/table output, filters, quiet plus JSON, duplicate/unknown
  flags, and positionals fail before Apple Container starts. The read-only
  snapshot is not ownership/health/readiness/liveness proof. A five-second
  deadline plus bounded isolated-process-group cleanup cover root exit and both
  pipe EOFs; each stream is 256 KiB, and malformed, oversized, or escaped-pipe
  stdout is not replayed. `cargo check -p vat --no-default-features` passed;
  shared `docker_shim` library validation passed 54/54 and focused direct-ps fake
  integration passed 4/4. The full serial fake-shim aggregate is intentionally
  not recorded: an independent serial run exposed a nondeterministic pre-existing
  Compose JSON logs timing race. The real-host direct-observation gate
  passed 1/1 (50 filtered) on Apple Container 1.1.0; `ps` is a global read-only inventory smoke
  observation, not targeted ownership evidence, and proves one valid native JSON
  document only. Fake/unit tests prove
  byte-preservation and fail-closed behavior.
  Image inventory accepts only `docker images --format json` / `--format=json`;
  `docker image ls` and `docker image list` share it. It invokes `container image
  list --format json`, validates one opaque Apple-native JSON value, and
  byte-for-byte replays stdout without a VAT wrapper or Docker Engine image schema.
  Template/table/YAML/TOML output, filters, quiet, verbose, all, digests, no-trunc,
  positionals, duplicates, unknown flags, and `--` fail before Apple Container;
  inherited text/quiet image-list behavior is unchanged. It makes no ownership,
  provenance, security, executability, registry, build-readiness, health,
  readiness, or liveness claim. The five-second bounded isolated cleanup covers
  root exit and both pipe EOFs, each stream is 256 KiB, and malformed, oversized,
  or escaped-pipe stdout is not replayed. `cargo check -p vat --no-default-features`
  passed; shared `docker_shim` library validation passed 54/54 and focused
  `docker_images_json` integration passed 4/4. The full serial fake-shim aggregate
  is intentionally not recorded: an independent serial run exposed a nondeterministic
  pre-existing Compose JSON logs timing race. The real-host direct-observation
  gate passed 1/1 (50 filtered) on Apple Container 1.1.0; `images` is a global
  read-only inventory smoke observation, not targeted ownership evidence, and
  proves one valid native JSON document only. Fake/unit
  tests prove byte-preservation and fail-closed behavior.
  Direct image inspect accepts only `docker image inspect --format json IMAGE`
  / `--format=json`. Exactly one JSON selector must precede exactly one opaque
  safe image reference (nonempty, with no leading `-`, whitespace, or control
  characters); templates, `--`, extra references, and every other option fail
  before Apple Container starts. VAT strips the selector, invokes only `container
  image inspect IMAGE`, bounded-captures and validates one opaque Apple-native
  JSON document, then byte-for-byte replays complete native stdout. A five-second
  bounded isolated observer covers root exit and both pipe EOFs; each stream is
  256 KiB, valid JSON preserves a nonzero child exit, and malformed, oversized,
  or escaped-pipe capture has no raw replay. It supplies no Docker image-inspect
  schema/template/Engine API, provenance, security, registry, build-completion,
  readiness, or secret-redaction claim. Recorded validation: cargo check passed;
  `cargo test -p vat --lib docker_shim -- --nocapture` passed 58/58;
  `RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim
  docker_image_inspect_json -- --nocapture` passed 4/4 with 1 ignored; and
  `RUST_TEST_THREADS=1 VAT_DOCKER_IMAGE_INSPECT_JSON_E2E_REQUIRED=1 cargo test
  -p vat --test vat_docker_shim apple_container_docker_image_inspect_json_contract
  -- --ignored --nocapture` passed 1/1 (61 filtered) in 1.21s. The host test
  proves only one direct `container image inspect alpine:3.20` invocation and
  one valid native document; fake/unit tests prove selector stripping,
  byte-preservation, and fail-closed bounds.
  Direct container inspect accepts only `docker inspect --format json CONTAINER`
  / `--format=json`; only `docker container inspect` shares it. Exactly one safe
  explicit id follows exactly one JSON selector, which is VAT-only and never
  forwarded; unformatted inspect remains inherited behavior. It invokes `container
  inspect CONTAINER`, fully validates one bounded Apple-native JSON value, then
  byte-for-byte replays it without a VAT wrapper or Docker Engine inspect schema.
  `--type`, `--size`, templates/table/YAML/TOML, filters, a second id, `--`, and
  unknown flags fail before Apple Container. One five-second bounded isolated
  observer covers root exit and both pipe EOFs; each stream is 256 KiB. Valid JSON
  with a nonzero child exit preserves that status, while malformed, oversized, or
  flood output suppresses raw stdout. It makes no ownership/provenance/security/
  image/registry/build-status/health/readiness/liveness/port-reachability claim
  and gives no secret-redaction guarantee. `cargo check -p vat --no-default-features`
  passed; shared `docker_shim` library validation passed 54/54 and focused
  `docker_inspect` integration passed 5/5. The full serial fake-shim aggregate
  is intentionally not recorded: an independent serial run exposed a nondeterministic
  pre-existing Compose JSON logs timing race. The real-host direct-observation
  gate passed 1/1 (50 filtered) on Apple Container 1.1.0; `inspect` targets the
  temporary owner-labeled nginx container and proves one valid native JSON
  document only. Fake/unit tests prove
  byte-preservation and fail-closed behavior.
  Direct logs JSON is separate from those five Apple-native forms: it accepts only
  `docker logs --format json --tail LINES CONTAINER` / equals forms and the same
  form through `docker container logs`. Exactly one format and one tail may mix
  spellings, precede one safe final id, and require `LINES` in 1..=1000;
  unformatted logs stay inherited. It invokes only `container logs -n LINES
  CONTAINER`, never forwarding the selector. Apple has text stdout only, so VAT
  returns exactly one `vat.docker.logs.v1` / `vat_json` wrapper—not Docker
  multiplex/demux—with untrusted `apple_container_stdio`, bounded diagnostic
  stderr, truncation/lossy flags, backend/container/requested_tail/runtime/child
  outcome, and safe inspect next. Child nonzero keeps wrapper plus exit; timeout,
  setup, or escaped-pipe failure has no partial wrapper. Five-second observation
  plus one-second cleanup drains both pipes and retains a suffix under 64 KiB
  capture and actual JSON-string caps. Follow/boot/timestamps/since/until/templates
  and every other modifier fail before runtime. It claims no Docker schema,
  multiplex/demux, ownership/provenance/security/image/registry/build,
  health/readiness/liveness/port-reachability, or secret redaction. `cargo check
  -p vat --no-default-features` passed; canonical `cargo test -p vat --lib
  docker_shim -- --nocapture` passed 54/54; focused `docker_logs_json` integration
  passed 6/6. The full serial fake-shim aggregate is intentionally not recorded:
  an independent serial run exposed a nondeterministic pre-existing Compose JSON
  logs timing race. The real-host direct-observation gate passed 1/1 (50
  filtered) on Apple Container 1.1.0; VAT logs targets the high-entropy nonce+PID
  owner-labeled temporary nginx container. Cleanup uses a high-entropy name and
  exact owner-label rechecks as conservative best-effort precautions; the emergency
  guard retains the container on uncertainty. Apple Container has no atomic
  conditional delete, so this is not a race-free guarantee against a TOCTOU
  replacement; the shared/cacheable nginx image is not cleaned up.
  The host smoke proves one VAT wrapper only. Fake/unit tests prove byte-preservation
  and fail-closed behavior.
  Direct exec JSON is a separate finite VAT wrapper, not Apple-native JSON or a
  Docker Engine stream/TTY contract. Only `docker exec --format json --timeout
  SECONDS CONTAINER -- COMMAND [ARG...]` (or equals forms) and the matching
  `docker container exec` form are accepted. One format and one timeout occur in
  either order before one safe container id; `SECONDS` is 1..=1200, and the
  Docker-facing delimiter plus a nonempty command are mandatory. Raw/unformatted
  exec stays on the inherited generic path, including selector-looking command
  arguments after its delimiter. VAT strips the selectors and Docker-only
  delimiter, then invokes `container exec CONTAINER COMMAND [ARG...]`. Its one
  `vat.docker.exec.v1` / `vat_json` wrapper includes requested timeout,
  `timeout_scope=host-container-client-observation`, backend/container/runtime and
  child outcome, untrusted bounded stdout/stderr suffixes with truncation/lossy
  flags, no secret-redaction guarantee, and a safe inspect next. Ordinary child
  nonzero preserves wrapper plus exit; timeout or setup/capture failure has no
  partial wrapper. Both pipes drain concurrently and each serialized JSON string
  is capped at 64 KiB. The timeout bounds only the host Apple Container client
  observation and does not claim guest command termination. TTY, interactive,
  detach, environment/user/workdir, templates, duplicate/misordered selectors,
  malformed delimiters, and every other exec flag fail before runtime. `cargo
  check -p vat --no-default-features` passed; canonical `cargo test -p vat --lib
  docker_shim -- --nocapture` passed 54/54 and focused `docker_exec_json`
  integration passed 4/4. The direct-observation E2E passed 1/1
  (50 filtered) on Apple Container 1.1.0 and observed one exec wrapper with both
  stdout and stderr markers. This is bounded direct-command evidence only, not
  guest-timeout termination, Docker Engine parity, generic runtime, Compose, or
  Kubernetes evidence.
  Strict direct run JSON is a foreground, owner-cleaned one-shot: only direct
  `docker run --format json --timeout SECONDS IMAGE [COMMAND...]` (or equals
  forms) is accepted. Exactly one format and one 1..=1200 timeout may occur in
  either order before IMAGE; optional command argv follows IMAGE directly. The
  JSON form rejects a Docker `--` before IMAGE or immediately after IMAGE;
  after the first non-`--` command token, later `--` is opaque child argv. It
  also rejects detach, TTY, interactive, caller name/label, ports, network,
  mounts, env, and every other run option before Apple Container starts. VAT
  generates a high-entropy name and independent
  owner label, captures bounded stdout/stderr into exactly one
  `vat.docker.run.v1` / `vat_json` document, and exposes a normal nonzero child
  exit only after exact owner-label cleanup confirms absence. Timeout, setup, or
  cleanup uncertainty emits no partial wrapper; only Apple's explicit
  `Error: container not found: <name>` proves an already-absent generated
  container. The timeout bounds host Apple Container client observation only;
  it makes no guest-wide termination, crash-recovery cleanup, Docker Engine
  parity, or secret-redaction claim. Focused deterministic validation passed 5
  passed plus 1 ignored in 1.80s. The real local `alpine:3.20` E2E passed 1/1
  (56 filtered) in 2.30s with one JSON document and exact cleanup.
  Strict direct build JSON is a separate bounded VAT receipt, not a sixth
  Apple-native JSON observation. Only direct `docker build --format json --timeout
  SECONDS --tag TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE]
  [--platform PLATFORM] [--label K=V ...] CONTEXT` (or documented equals forms)
  is accepted: format `json`, a positive whole 1..=1200 timeout, and tag occur
  exactly once; file/target/platform occur at most once; build args/labels may
  repeat; and every option precedes one canonical existing local-directory
  context. `--`, missing/duplicate/misordered selectors, a second context, and
  unsupported flags fail before the builder; raw builds without either selector
  retain the inherited translator. VAT strips only its JSON/deadline selectors,
  invokes public `container build --tag TAG [--file ...] [--build-arg ...]
  [--target ...] [--platform ...] [--label ...] CONTEXT`, then emits one
  `vat.docker.build.v1` / `vat_json` receipt after the Apple client exits. It
  carries bounded untrusted stdout/stderr, truncation/lossy flags, timeout scope,
  and child outcome with `image_lifecycle=retained_no_auto_cleanup`: product
  builds get no VAT cleanup or ownership claim. Success safely points to strict
  `docker image inspect --format json TAG`; normal child failure preserves its
  receipt and exit but is `terminal=build_failed` with `next=docker --help`, not
  a stale image-inspect handoff. Timeout/setup/capture failure emits no receipt.
  The deadline is only host client observation, not builder cancellation or
  rollback/removal. No Docker Engine/API, provenance, ownership, readiness,
  security, secret-redaction, cancellation, or rollback claim follows; args,
  labels, and output are opaque/untrusted. Cargo check passed; docker_shim lib
  passed 62/62; focused `docker_build_json` passed 4 plus 1 ignored (63 filtered);
  `native_image_owner_guard...` passed 1/1 (67 filtered); and the opt-in host
  E2E passed 1/1 (67 filtered) in 2.53s. The host test's high-entropy tag/exact
  `io.cclab.vat.e2e-owner` label plus exact pre/post absence and pre-delete label
  recheck are test-only cleanup safeguards. Apple has no conditional build/delete;
  they are best effort and ambiguity leaks, never product auto-cleanup behavior.
  Strict direct pull JSON is a separate bounded VAT receipt, not Apple-native
  JSON: only direct `docker pull --format json --timeout SECONDS IMAGE` (or
  documented equals forms) is accepted. Exactly one json format and one positive
  whole 1..=1200 timeout may be reordered before one safe opaque image reference.
  Empty, leading-dash, whitespace/control, URL-style `://`, and leading Git-style
  `git@` remote forms reject; ordinary OCI `@digest` remains opaque. `--`, a
  second reference, missing/duplicate/misordered selectors, and
  unsupported flags fail before the client; raw direct pull without either
  selector and every `docker image pull` form stay inherited. VAT strips only
  JSON/deadline selectors, invokes public `container image pull IMAGE`, then
  emits one `vat.docker.pull.v1` / `vat_json` receipt after the Apple client
  exits. It carries bounded untrusted stdout/stderr, truncation/lossy flags,
  timeout scope, and child outcome with
  `image_lifecycle=not_owned_no_auto_cleanup`: no VAT ownership, cleanup, or
  registry login/auth/credential lifecycle. Success safely points to strict image
  inspect without proving image state/download completion; normal child failure
  preserves receipt/exit but is `terminal=pull_failed` with `docker --help`, not
  stale inspect. Timeout/setup/capture/pipe failure emits no receipt. The deadline
  observes only the host client and copied pipes, not transfer cancellation,
  download completion, rollback, or local/backend image state. No Docker
  Engine/API, registry-management, provenance, digest, platform, freshness,
  image-state, ownership, security, secret-redaction, cancellation,
  download-completion, or rollback claim follows. Cargo check passed; docker_shim
  lib passed 65/65; focused `docker_pull_json` passed 5 plus 1 ignored (68
  filtered); and the opt-in host E2E passed 1/1 (73 filtered) in 27.14s. Its real
  `container image pull alpine:3.20` uses shared/cacheable state and deliberately
  neither deletes the image nor asserts ownership on success or failure.
  Resource stats remains strict `docker stats --no-stream --format json
  CONTAINER [CONTAINER...]` (or `--format=json`): it runs one bounded,
  read-only Apple Container native-JSON observation, not a Docker Engine
  schema/wrapper or ownership/health/liveness proof. A single five-second
  deadline plus bounded isolated-process-group cleanup governs root exit and
  both pipe EOFs; only complete bounded native JSON is replayed, so an escaped
  pipe holder fails closed without stdout replay. Stdout/stderr are each bounded
  at 256 KiB, and invalid or oversized native stdout is suppressed. Shared
  `docker_shim` library validation passed 54/54. The full serial fake-shim
  aggregate is intentionally not recorded: an independent serial run exposed a
  nondeterministic pre-existing Compose JSON logs timing race. The real-host
  direct-observation gate passed 1/1 (50 filtered) on Apple Container 1.1.0; `stats` targets
  the temporary owner-labeled nginx container and proves one valid native JSON
  document only. Fake/unit tests prove
  byte-preservation and fail-closed behavior.
- The shim's Compose support has exactly three profiles: one literal-image
  service (`strict-single-image-v1`, `up -d`); one literal short build-only
  service (`strict-single-build-v1`, `up -d --build`); or two through four
  literal-image services selected only by the exact top-level
  `x-vat-compose-profile: host-facing-independent-v1` marker. Every
  host-facing service has one unique nonzero `host:container` port published on
  loopback only, and successful JSON makes the negative contract explicit with
  `profile=host-facing-independent-v1`, `service_name_dns=false`, and
  `host_loopback_only=true`. It has no service-name DNS or bridge networking.
  `docker compose --dry-run -f FILE -p PROJECT up -d [--build]` is a strict
  file/profile-only preflight for those same profiles. It emits one
  `vat.docker-compose.preflight.v1` JSON document with `validated=true`,
  `runtime_started=false`, `registry_written=false`, `image_built=false`,
  `launch_revalidates=true`, structured `launch_argv`, and executable `next`.
  It calls no Apple Container command, does not build/import/start or write a
  registry, rejects `--wait` and every other global/Compose flag, and the real
  launch uses the canonical source path so it revalidates the same file after a
  cwd change. It is not generic Docker Compose dry-run parity.
  `docker compose -f FILE -p PROJECT up -d --wait [--wait-timeout SECONDS]`
  remains detached-only: accept one `--wait`; accept `--wait-timeout` only
  with wait as positive whole seconds (default 300, maximum 1200). Its budget
  begins after validated import/source build and immediately before launch,
  covering runner handoff and durable VAT observations rather than Docker
  healthchecks, app HTTP, service DNS, or generic Compose readiness. VAT pins
  the wait to profile/generation/ticket and drops the registry lock between
  polls, preventing stale attachment after down/re-import/relaunch. Ready emits
  one final up JSON with wait plus ready topology. Timeout retains runtime and
  registry; a ps handoff exists only after a current pinned observation, while
  terminal/replaced/bare-deadline cases have no unsafe next and degraded has no
  endpoint. Source-build cleanup_next is included only with verified ready wait
  success.
  `docker compose -p PROJECT ps` has two exact output shapes: no format returns
  its final additive JSON after the human text, while `--format json` and
  `--format=json` emit exactly one VAT-owned `vat.docker-compose.ps.v1`
  `format=vat_json` document with the same claim-held profile/topology proof and
  no human table. Services
  stay in registered Compose order; an endpoint is only
  `127.0.0.1:<port>` and is emitted for every service only after all expected
  services prove unique Ready VAT-owned `container_run` evidence for their
  exact MicroVMs, loopback nonzero ports, and no cleanup errors. A failed proof
  converts ready to `degraded` with `ready=false` and no endpoints; inactive,
  starting, and stopping also have no endpoints. This is not an app health
  check. JSON mode is not Docker Compose JSON/template/table compatibility;
  every other `ps` format and generic/missing/unknown provenance fails closed
  before topology output. Text `logs SERVICE` preserves observed log bytes,
  then starts its additive VAT handoff JSON on a new line after them.
  `logs --format json [--tail LINES] SERVICE` (also equals forms, service final)
  emits one capture-only `vat.docker-compose.logs.v1` document with separate
  stdout/stderr snapshots, default-200/range-1..=1000 tail_lines, per-stream
  truncated/utf8_lossy, capture_only=true, runtime_invoked=false, and
  compose_record_mutated=false. It holds claim/provenance and reads VAT-captured
  logs only: no Apple Container call/project.json mutation. VAT first caps each
  read and line tail, then after lossy UTF-8 plus JSON escaping retains a valid
  UTF-8 suffix whose serialized JSON string value remains within the same
  64 KiB per-stream cap and marks it truncated; there are no topology/endpoints,
  and `next` points to JSON ps. It is not Docker Compose
  merged/follow/timestamp/template compatibility; follow, timestamps, and other
  flags fail closed. The full serial `vat_docker_shim` aggregate is intentionally
  not recorded because an independent serial run exposed a nondeterministic
  pre-existing Compose JSON logs timing race; its focused serialized-cap unit
  passed 1/1 for `0xff`-heavy and NUL/control-heavy streams after actual JSON
  serialization. The recorded opt-in real dual-service E2E includes this JSON
  logs shape for its bounded profile.
  Dependencies, networks, volumes, host-facing build, interpolation,
  `--env-file`, default TTY, and other unsupported flags fail before runtime
  launch. Then `ps`, `logs SERVICE`, text `exec -T SERVICE -- COMMAND`, and
  JSON `exec -T --format json SERVICE -- COMMAND` (or `--format=json`) operate
  only through one same-read valid-shim-provenance/exact-unique-ready-MicroVM
  snapshot. Text exec preserves observed child bytes, then starts its additive
  VAT handoff JSON on a new line after them. VAT parses and validates the
  Docker-facing `--` but does not forward it, invoking Apple Container as
  `container exec CONTAINER COMMAND [ARG...]`. JSON format must be immediately
  after `-T`, with service next and mandatory `--`; misordering, a missing
  delimiter, TTY, or other exec flags fail closed. JSON holds the existing
  claim/provenance/ready proof only through child spawn, then releases it before
  waiting. It emits one `vat.docker-compose.exec.v1` VAT JSON document with
  profile, child exit code, separate stdout/stderr, per-stream
  truncated/utf8_lossy, runtime_invoked=true, and compose_record_mutated=false;
  it replays no raw output and has no topology/endpoints. Both child streams
  drain concurrently and each serialized JSON string is capped at 64 KiB. This
  is not Docker Compose exec output compatibility. The full serial shim aggregate
  is intentionally not recorded because an independent serial run exposed a
  nondeterministic pre-existing Compose JSON logs timing race; the precise
  serialized-cap unit passed 1/1; the bounded real-host JSON-exec evidence is
  stated below.
  Generic `vat compose` cannot operate a known
  shim record; a normal inactive generic re-import clears known provenance,
  while unknown inactive provenance receives registry-only cleanup preserving
  `vat.toml` and unknown active provenance requires matching or newer VAT.
  A successful source-build `up` additionally returns its exact VAT-owned
  `images` and `cleanup_next` (`down && docker image rm <exact-tag>`); a
  literal-image project deliberately receives neither ownership field. The
  host-facing two-to-four-service path has deterministic fake-lifecycle
  coverage plus an opt-in real Apple Container dual-service E2E passed 1/1 (50
  filtered) on this host in 4.54 seconds. That gate proves only
  `host-facing-independent-v1` `up -d --wait`, both loopback endpoints,
  one-document JSON `ps`, `logs`, and `exec`, text logs, text exec including a
  no-final-newline handoff, and `down` cleanup of exact containers, ports, and
  registry—not service-name DNS, general Compose, a Docker Engine API, or
  Kubernetes.
- The shim still does not expose a Docker Engine socket/API and does not imply
  general Compose, SDK, Testcontainers, devcontainer, Docker output-schema, or
  Docker network compatibility. It is an agent shell-command bridge, not a daemon.
- Apple Container K3s has a separate bounded headless path, not a durable
  replacement for a Kubernetes Desktop integration: `vat k8s ephemeral` is one
  command, while `vat k8s session` is a bounded active lease across explicit
  commands. Every `vat k8s` command requires an independently installed
  `kubectl` first on `PATH`; VAT rejects an OrbStack-provided binary before K3s
  use. This is host-tool provenance, not a GUI or Docker Engine dependency. On
  this host Homebrew `kubernetes-cli` now supplies `/opt/homebrew/bin/kubectl`.
  Independent-kubectl one-shot, leased, local-image, and Service-forward E2Es
  each passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s
  respectively. The local-image E2E loaded one already-local Apple `alpine:3.20`
  into one lease, ran a pod with `imagePullPolicy=Never`, observed its marker
  log, then completed exact session cleanup; it is not registry-pull generality.
  A lease can load one verified local `linux/arm64` image and run one
  foreground, Service-only `127.0.0.1` port-forward whose child receives endpoint
  metadata while VAT strips K3s credential variables and `VAT_HOME` from the
  child environment. That filtering is not a same-UID OS sandbox or
  adversarial-child security boundary; the child must not daemonize or escape its
  authenticated kubectl process group. Neither path has restart safety,
  reboot-safe retention, PVC/storage, ingress/LB, public listener, background
  tunnel, or multi-node promise because Apple's machine restart path is still a
  hard blocker. A bootstrap failure remains diagnostic-only: the root error is
  primary, then exactly the fixed six-label read-only evidence is emitted under
  its six-second total / one-second-per-probe budget before exact cleanup. It
  excludes private kubeconfig/cache and host credentials, changes neither the
  existing 300-second bootstrap behavior nor persistence, and does not retry or
  rerun `k3s --version` or add a wrapper/recovery command. The deterministic
  fake regression passed. The leased real-host result includes strict JSON exec
  with `--timeout 30`; the Service-forward result includes one Service-only
  loopback strict JSON tunnel with a credential-free child, confirmed cleanup,
  and closed local ports. Neither result establishes persistence, crash-safe
  termination, a general cluster, or OS-sandbox behavior.
- The runner is always a host process (never containerized) — the GPU story.
  Docker is only an option for run-scoped dependency *services*.
- Services in `vat.toml` are run-scoped dependencies of one runner invocation;
  containers are ephemeral (`docker run --rm`) and removed at teardown; external
  services are attached and probed but not lifecycle-managed by vat.
- vat does not schedule production work or manage restart policy.
- Standalone `vat cluster` clusters outlive a run as a convenience, but vat does
  not supervise them (no daemon, no restart, no health monitoring) — it only
  creates/lists/deletes/reports on explicit command, like kind/k3d themselves.
"#;

const CORE: &str = r#"# VAT core workflow

Use `vat run` for an ad-hoc command or a configured runner. Use `vat plan --json`
before side effects, `vat doctor --json` for selected-topology readiness, and
`vat state`, `vat diff`, or `vat logs` to inspect retained evidence.
"#;

const SERVICES: &str = r#"# VAT services

Declare run-scoped dependencies in `vat.toml`, then inspect the resolved shape
with `vat plan --json`. Built-in emulators include `gcloud-pubsub` and
`gcloud-cloud-tasks`; native Lumen uses `preset = "lumen"` with an optional
`version = "lumen@<version>"`. Use `vat doctor --json` before a service-backed
run. Services are test dependencies, not durable production infrastructure.
"#;

const CONTAINER: &str = r#"# VAT container and compose workflow

Use `vat build` for a local Dockerfile build and `vat compose` for VAT's
documented bounded Compose subset. `vat docker install-shim` is opt-in and is
not a Docker Engine/API, generic Compose, SDK, or daemon compatibility layer.
"#;

const K8S: &str = r#"# VAT local Kubernetes workflow

Use `vat k8s ephemeral image build` followed by `vat k8s ephemeral run -- ...`
for one command, or `vat k8s session create`, `exec`, and `delete` for a bounded
multi-command lease. An independently installed `kubectl` is required. This is
not persistent Kubernetes, a Desktop integration, or a general cluster manager.
"#;

const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "core",
        summary: "run, plan, doctor, and inspect one local VAT workflow",
        body: CORE,
    },
    cli_std::llm::Topic {
        id: "services",
        summary: "declare and preflight run-scoped vat.toml dependencies",
        body: SERVICES,
    },
    cli_std::llm::Topic {
        id: "container",
        summary: "use bounded build, compose, and opt-in docker-shim commands",
        body: CONTAINER,
    },
    cli_std::llm::Topic {
        id: "k8s",
        summary: "run one-shot or leased ephemeral Apple Container K3s work",
        body: K8S,
    },
    cli_std::llm::Topic {
        id: "guide",
        summary: "complete backward-compatible VAT agent usage contract",
        body: GUIDE,
    },
];

// <HANDWRITE gap="missing-generator:logic" tracker="#1817" reason="DX command-inventory contract and offline guide text are hand-written pending codegen support">
/// @spec apps/vat/tech-design/logic/llm-agent-usage-guide.md#cli
pub fn exec(topic: &str, format: cli_std::llm::Format) -> Result<ExitCode> {
    let out = cli_std::llm::render("vat", crate::VERSION, TOPICS, topic, format)?;
    println!("{out}");
    Ok(ExitCode::SUCCESS)
}
// </HANDWRITE>
// CODEGEN-END
