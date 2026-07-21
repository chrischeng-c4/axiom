---
id: vat-source-projects-vat-src-commands-llm-rs
summary: >
  rust-source-unit TD AST payload for apps/vat/src/commands/llm.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized apps/vat/src/commands/llm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/vat/src/commands/llm.rs` generated from AST during Score force-regeneration standardization.

The agent guide documents that text Compose logs/exec preserve observed bytes
then start their additive VAT handoff JSON on a new line; its exact VAT-native
JSON exec form has same-snapshot known-provenance/unique-ready-MicroVM
authorization through spawn, parses the Docker-facing delimiter without
forwarding it to `container exec CONTAINER COMMAND [ARG...]`, and uses
concurrent 64 KiB serialized-stream capture. Its bounded real-host evidence is
the host-facing-independent-v1 dual-service E2E, not a general Compose claim. It also distinguishes
observation-only K3s session status from bounded private-lock `--verify-api`
proof without expanding the one-boot/no-GUI/no-Docker-Engine boundary.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | apps/vat/src/commands/llm.rs | function | pub | 254 | exec(topic: &str, format: cli_std::llm::Format) -> Result<ExitCode> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
- Use `vat capabilities --json` for full host backend/isolation/Docker/service
  discovery; it retains the normal Docker daemon probe, mapping
  `services.docker_services` to `available` or `unavailable`, and includes a
  bounded, read-only Apple Container shared-builder advisory. `container builder status`
  records `ownership=shared_unknown` and `automatic_cleanup=false`; parseable
  configuration is separate from optional live `observed_stats`, and optional
  `container system df` is global host evidence rather than VAT-owned disk.
  Unsupported, malformed, or timed-out observations stay nonfatal
  `unknown`/`probe_errors`; VAT never starts, stops, deletes, or prunes the
  shared builder/cache. A real live state appears only when the installed CLI
  supports and returns it.
- Use `vat doctor --json` for selected-plan preflight. A selected explicit
  MicroVm/Apple-Container-only plan checks read-only `container` availability
  and exactly one `container system status` probe per invocation, never
  executes Docker even when it is on `PATH`, and returns
  `docker.daemon_probe.state=skipped` with the truthful Apple-Container-only
  selected-plan reason and `services.docker_services=not_probed`. In that
  deliberate no-probe state, `daemon=false` is not Docker-unavailable evidence
  because no Docker command ran. Unselected Docker services do not affect that runner; Docker
  runtime, Auto image, eligible Auto preset fallback, and selected cluster
  plans retain Docker probing (cluster needs Docker). Doctor neither autostarts
  Apple Container nor falls back to Docker: unsupported MicroVm presets without
  a declared OCI route and MicroVm preset named volumes fail closed. The
  shared-builder advisory is nonblocking, so its timeout/unknown/probe errors
  never change runtime success.
- If you only need one ad-hoc command, use `vat run -- <command>`.
- `vat run` prints sparse JSONL checkpoints; the final line has
  `"type":"result"`.
- SIGINT/SIGTERM cleanup is owned by the running VAT process. The first signal
  wins; VAT stops runners first, then VAT-owned services in reverse order with
  bounded TERM/grace/KILL/reap/PGID-absence proof. It persists terminal
  `interrupted` state with no child PID and exits 130/143. Interrupted evidence
  is retained for `vat state`/`vat gc`; explicit `external` services and
  unrelated listeners are never signalled.
- After a retained run, inspect `vat state <id>`, `vat diff <id>`, and
  `vat logs <id> [runner|service-id]`.
- Every `vat k8s` command requires an independently installed `kubectl` first
  on `PATH`; VAT rejects an OrbStack-provided binary before K3s use. This is a
  host-tool provenance prerequisite, not a GUI or Docker Engine dependency.
  Homebrew `/opt/homebrew/bin/kubectl` is installed locally. Independent-
  kubectl one-shot, leased, local-image, and Service-forward E2Es passed 1/1
  (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s respectively. The
  local-image proof is one already-local Apple `alpine:3.20` pod with
  `imagePullPolicy=Never`, a marker log, and exact session cleanup—not
  registry-pull generality, persistence, GUI, or Docker Engine/API evidence.
- For Docker-free one-command local Kubernetes work on Apple Container, first
  run `vat k8s ephemeral image build`, then run
  `vat k8s ephemeral run -- kubectl get nodes`. VAT injects a private
  `KUBECONFIG`, `VAT_K8S_CACHE_DIR`, and `VAT_K8S_API_SERVER` only into that
  command and removes them with the exact machine afterwards. Its final stdout
  line is a terminal `vat_k8s_ephemeral_result` JSON record.
- For a bounded multi-command agent workflow, use `vat k8s session create
  --ttl 30m`, retain the opaque id, run each host command through text
  `vat k8s session exec --timeout 30 <id> -- kubectl ...` or JSON
  `vat k8s session exec --format json --timeout 30 <id> -- kubectl ...`, then
  finish with `vat k8s session delete <id>`. Omitted timeout uses the remaining
  lease TTL; explicit timeout is 1..=14400 seconds and cannot exceed it. Every
  exec revalidates the exact backing id, endpoint, and host API before
  injecting private credentials, rechecks expiry before spawn, owns a process
  group, and holds the private operation lock through cleanup. Normal exit,
  deadline, or SIGINT/SIGTERM reaps that group and removes the exec marker only
  after it is absent. A starting or live crash marker blocks later exec, delete,
  and cleanup fail-closed rather than claiming recovery termination. JSON emits
  one `vat.k8s.session.exec.v1` document with separate 64 KiB serialized-bounded
  streams, child exit, no raw replay, and a status-verify next. Its child
  intentionally receives private credentials, so this is not an untrusted-child
  boundary. The independent-kubectl leased E2E passed 1/1 (36 filtered) in
  29.97s, including text commands, strict JSON exec with `--timeout 30`, status
  verification, and exact delete; it does not establish crash-safe termination
  or persistent Kubernetes.
- On failed one-shot or leased K3s bootstrap, VAT renders the root error first,
  then emits staged non-sensitive installer evidence through exactly
  `guest_install_log`, `guest_k3s_system`, `backing_container_logs`,
  `machine_boot_log`, `machine_inspect`, and `container_system_status` under a
  six-second total / one-second-per-probe fixed read-only budget. It excludes
  private kubeconfig/cache and host credentials, leaves the existing 300-second
  bootstrap behavior unchanged, does not retry or rerun `k3s --version`, adds
  no wrapper/recovery command, and still performs exact cleanup. The
  deterministic fake regression passed. Independent-kubectl one-shot, leased,
  local-image, and Service-forward E2Es passed 1/1 (36 filtered) in 28.38s,
  29.97s, 49.73s, and 49.57s. The local-image result loads one already-local
  Apple `alpine:3.20` into one lease, runs a pod with `imagePullPolicy=Never`,
  observes its marker log, then completes exact session cleanup; it is not
  registry-pull generality. The leased result covers strict JSON exec with
  `--timeout 30`; the
  Service-forward result covers one Service-only loopback strict JSON tunnel,
  a credential-free child, confirmed cleanup, and closed local ports. These are
  bounded one-guest results, not persistence, a general cluster, or OS-sandbox
  behavior.
- To use an already-local Apple image without Docker or a registry, run
  `vat k8s session image load <id> <local-image-ref>` before the workload. VAT
  requires exactly one inspected `linux/arm64` variant, uses a private bounded
  OCI archive, imports it into `k8s.io`, verifies the canonical reference, and
  removes host and guest archive copies before success. It accepts no arbitrary
  tar; use `imagePullPolicy: Never` to prove the workload uses the local image.
  The opt-in real-host local-image E2E passed 1/1 (36 filtered) in 49.73s: one
  already-local Apple `alpine:3.20` loaded into one active lease, a pod ran it
  with `imagePullPolicy=Never` and emitted its marker log, then exact session
  cleanup completed. It is not registry-pull generality, persistence, GUI, or
  Docker Engine/API evidence.
- Text `vat k8s session port-forward run <id> service/<name> <port> -- COMMAND`
  forwards child output and starts its terminal record on a new line afterward.
  Its only JSON form is `run --format json <id>
  service/<name> <port> -- COMMAND`: one Service-only loopback tunnel gives a
  credential-free host child endpoint metadata and private HOME, shares
  kubectl's authenticated process group, and holds the private operation lock
  through cleanup. After confirmed cleanup it emits exactly one
  `vat.k8s.session.port-forward.v1` `vat_json` document with child exit,
  separate 64 KiB serialized-capped streams, no raw replay, and status-verify
  next. VAT masks setup/API/tunnel/cleanup failures but preserves opaque child
  output in a successful result; silent post-API/pre-spawn lease checks prevent
  a crossed TTL from opening a tunnel, and partial reader setup cleans direct and
  outer children before joining readers. The independent-kubectl Service-forward
  E2E passed 1/1 (36 filtered) in 49.57s, including one Service-only loopback
  text and strict one-document JSON tunnel with a credential-free child,
  confirmed cleanup, and closed local ports. It does not establish ingress/LB,
  a public listener, a background tunnel, arbitrary resource forwarding,
  persistent Kubernetes, or a same-UID OS sandbox.
- The opt-in shim has five strict Apple-native JSON observation forms: direct
  container inventory, image inventory, direct image inspect, direct container
  inspect, and resource stats. Direct
  inventory is only `docker ps --format json` or equals with optional
  exactly-once `--all` or `-a`; `docker container ls` and `docker container list`
  share it, while `docker container ps --format json` remains rejected; inherited
  text behavior is unchanged. VAT runs
  canonical `container list --format json [--all]`, validates one opaque
  Apple-native JSON value, and byte-for-byte replays stdout with no VAT wrapper
  or Docker Engine ps schema. Templates/table output, filters, quiet plus JSON,
  duplicate/unknown flags, and positionals fail before Apple Container starts.
  It is read-only inventory, not ownership/health/readiness/liveness proof. A
  five-second deadline plus bounded isolated cleanup covers root exit and both
  pipe EOFs; streams are capped at 256 KiB and malformed, oversized, or
  escaped-pipe stdout fails closed without replay. `cargo check -p vat
  --no-default-features` passed; shared `docker_shim` library passed 54/54,
  focused direct-ps integration passed 4/4. The full serial fake-shim aggregate
  is intentionally not recorded because an independent serial run exposed a
  nondeterministic pre-existing Compose JSON logs timing race. The real-host
  direct-observation gate passed 1/1 (50 filtered) on Apple Container 1.1.0; `ps` is a global
  read-only inventory smoke observation, not targeted ownership evidence, and
  proves one valid native JSON document only. Fake/unit tests prove
  byte-preservation and fail-closed behavior.
- Image inventory is only `docker images --format json` or equals; `docker image
  ls` and `docker image list` share that exact form. VAT runs `container image
  list --format json`, bounded-captures and validates one opaque Apple-native
  JSON value, then byte-for-byte replays stdout with no VAT wrapper or Docker
  Engine image schema. Template/table/YAML/TOML output, filters, quiet, verbose,
  all, digests, no-trunc, positionals, duplicates, unknown flags, and `--` fail
  before Apple Container starts; text/quiet image-list behavior is unchanged. It
  makes no ownership/provenance/security/executability/registry/build-readiness/
  health/readiness/liveness claim. One five-second deadline plus bounded isolated
  cleanup covers root exit and both pipe EOFs; streams are capped at 256 KiB and
  malformed, oversized, or escaped-pipe stdout fails closed without replay.
  `cargo check -p vat --no-default-features` passed; shared `docker_shim` library
  passed 54/54 and focused `docker_images_json` integration passed 4/4. The full
  serial fake-shim aggregate is intentionally not recorded because an independent
  serial run exposed a nondeterministic pre-existing Compose JSON logs timing race.
  The real-host direct-observation gate passed 1/1 (50 filtered) on Apple Container
  1.1.0; `images` is a global read-only inventory smoke observation, not targeted
  ownership evidence, and proves one valid native JSON document only. Fake/unit
  tests prove byte-preservation and fail-closed behavior.
- Direct image inspect accepts only `docker image inspect --format json IMAGE`
  or equals. One JSON selector precedes exactly one opaque safe image reference
  (nonempty, no leading `-`, whitespace, or control characters); templates,
  `--`, extra references, and every other option fail before Apple Container.
  VAT strips the selector, invokes only `container image inspect IMAGE`,
  bounded-captures and validates one opaque Apple-native JSON document, then
  byte-for-byte replays complete native stdout. A five-second isolated observer
  caps each stream at 256 KiB, preserves valid JSON plus a nonzero child exit,
  and suppresses malformed, oversized, or escaped-pipe capture. It claims no
  Docker image-inspect schema/template/Engine API, provenance, security,
  registry, build-completion, readiness, or secret redaction. Cargo check
  passed; `cargo test -p vat --lib docker_shim -- --nocapture` passed 58/58;
  focused `docker_image_inspect_json` passed 4/4 with 1 ignored; and its
  opt-in host E2E passed 1/1 (61 filtered) in 1.21s, proving one direct
  `container image inspect alpine:3.20` call and one native document.
- Direct container inspect accepts only `docker inspect --format json CONTAINER`
  or equals; `docker container inspect` shares that exact form. Exactly one safe
  explicit id follows exactly one JSON selector, which must precede the id, is
  VAT-only, and is never forwarded; unformatted inspect remains inherited behavior. VAT
  runs canonical `container inspect CONTAINER`, bounded-captures and validates one
  opaque Apple-native JSON value, then byte-for-byte replays stdout with no VAT
  wrapper or Docker Engine inspect schema. `--type`, `--size`, templates/table/
  YAML/TOML, filters, a second id, `--`, and unknown flags fail before Apple
  Container starts. A five-second bounded isolated observer covers root exit and
  both pipe EOFs; streams are capped at 256 KiB. Valid JSON plus a nonzero child
  exit preserves status, while malformed, oversized, or flood output suppresses
  raw stdout. It makes no ownership/provenance/security/image/registry/build-status/
  health/readiness/liveness/port-reachability claim and gives no secret-redaction
  guarantee. `cargo check -p vat --no-default-features` passed; shared
  `docker_shim` library passed 54/54 and focused `docker_inspect` integration
  passed 5/5. The full serial fake-shim aggregate is intentionally not recorded
  because an independent serial run exposed a nondeterministic pre-existing Compose
  JSON logs timing race. The real-host direct-observation gate passed 1/1 (50 filtered)
  on Apple Container 1.1.0; `inspect` targets the temporary owner-labeled
  nginx container and proves one valid native JSON document only. Fake/unit tests
  prove byte-preservation and fail-closed behavior.
- Direct logs JSON is separate from the five Apple-native forms. It accepts only
  `docker logs --format json --tail LINES CONTAINER` / equals forms and the same
  form through `docker container logs`: exactly one format and tail may mix
  spellings, must precede one safe final id, and requires `LINES` in 1..=1000;
  unformatted logs remains inherited text behavior. VAT invokes only `container
  logs -n LINES CONTAINER`, never forwards the selector, and emits exactly one
  `vat.docker.logs.v1` / `vat_json` wrapper with untrusted
  `apple_container_stdio`, bounded diagnostic stderr, truncation/lossy flags,
  backend/container/requested_tail/runtime/child outcome, and safe inspect next—
  not Docker schema or multiplex/demux. Ordinary child nonzero preserves wrapper
  plus exit; follow/boot/timestamps/since/until/templates and all other modifiers
  fail before runtime, while timeout/setup/escaped-pipe paths emit no partial
  wrapper after five-second plus one-second bounded dual-stream suffix and
  serialized-string caps. `cargo check -p vat --no-default-features` passed;
  canonical `cargo test -p vat --lib docker_shim -- --nocapture` passed 54/54; focused
  `docker_logs_json` integration passed 6/6. The full serial aggregate is
  intentionally not recorded because of the nondeterministic pre-existing Compose
  JSON logs timing race. `VAT_DOCKER_SHIM_E2E_REQUIRED=1 cargo test -p vat --test
  vat_docker_shim apple_container_docker_run_published_port_contract -- --ignored
  --nocapture` passed 1/1 (50 filtered) on Apple Container 1.1.0: logs targets a
  high-entropy nonce+PID owner-labeled temporary nginx container. Exact-label
  rechecks are conservative best-effort precautions, and the emergency guard
  retains the container on uncertainty. Apple Container has no atomic conditional
  delete, so this is not a race-free or impossible-to-misdelete cleanup guarantee;
  the shared/cacheable nginx image is not cleaned up. The host smoke proves one VAT wrapper only;
  fake/unit tests prove byte-preservation and fail-closed behavior.
- Direct exec JSON is a finite VAT wrapper for `docker exec --format json
  --timeout SECONDS CONTAINER -- COMMAND [ARG...]` / equals forms and the same
  form through `docker container exec`. Exactly one format and one timeout may
  occur in either order before one safe id, `SECONDS` is 1..=1200, and the
  Docker-facing delimiter plus a nonempty command are mandatory; unformatted/raw
  exec remains inherited. VAT removes selectors and the delimiter, then invokes
  Apple `container exec CONTAINER COMMAND [ARG...]`. It emits one
  `vat.docker.exec.v1` / `vat_json` wrapper with requested timeout,
  `timeout_scope=host-container-client-observation`, runtime/child outcome,
  untrusted bounded stdout/stderr suffixes with truncation/lossy flags, no
  secret-redaction guarantee, and safe inspect next. Ordinary child nonzero
  preserves wrapper plus exit; timeout or setup/capture failure emits no partial
  wrapper; each serialized stream value caps at 64 KiB. The timeout is only the
  host Apple Container client observation and does not claim guest command
  termination. TTY, interactive, detach, env/user/workdir, templates, malformed
  delimiters, and other exec flags fail before runtime. Canonical docker_shim lib
  validation passed 54/54 and focused `docker_exec_json` integration passed 4/4.
  The direct-observation E2E passed 1/1 (50 filtered) on Apple Container 1.1.0
  and observed one exec wrapper with both stdout and stderr markers; it is not
  Docker Engine parity, generic runtime, Compose, or Kubernetes evidence.
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
- Resource stats remains only `docker stats --no-stream --format json
  CONTAINER [CONTAINER...]` or the exact equals form. It accepts only those
  flags before explicit ids, runs canonical `container stats --format json
  --no-stream`, and replays one complete validated Apple-native JSON document
  with no VAT/Docker Engine wrapper/schema. It is read-only, not
  ownership/health/readiness/liveness proof. A five-second deadline plus bounded
  isolated cleanup governs root exit and both pipe EOFs; escaped pipe capture
  fails closed without stdout replay. Streams are capped at 256 KiB and
  malformed/oversized stdout is suppressed. Shared strict-native-observation
  library validation passed 54/54. The full serial fake-shim aggregate is
  intentionally not recorded because an independent serial run exposed a
  nondeterministic pre-existing Compose JSON logs timing race. The real-host
  direct-observation gate passed 1/1 (50 filtered) on Apple Container 1.1.0; `stats` targets
  the temporary owner-labeled nginx container and proves one valid native JSON
  document only. Fake/unit tests prove byte-preservation and fail-closed behavior.
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
  `host_loopback_only=true`. `docker compose --dry-run -f FILE -p PROJECT up -d
  [--build]` parses only those existing strict image/build/host-facing profiles
  and emits exactly one `vat.docker-compose.preflight.v1` VAT JSON document:
  `validated=true`, `runtime_started=false`, `registry_written=false`,
  `image_built=false`, `launch_revalidates=true`, structured `launch_argv`, and
  executable `next`. It invokes no Apple Container command, does not
  build/import/start or write a registry, rejects `--wait` and every other
  global/Compose flag, and the returned real launch uses the parser's canonical
  source path to revalidate the same file after a cwd change.
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
  shapes: no format preserves text and ends with additive JSON retaining the
  known profile plus `topology={phase,ready,services}`; `--format json` and
  `--format=json` emit exactly one VAT-owned `schema=vat.docker-compose.ps.v1`,
  `format=vat_json` document with the same claim-held profile/topology proof and
  no human table. Phase is
  inactive, starting, ready, degraded, or stopping; services follow registered
  Compose service-ID order. An endpoint is only `127.0.0.1:<port>`, and every
  endpoint appears only when every expected service has exactly one Ready
  VAT-owned `container_run` record for its exact MicroVM, a nonzero loopback
  port, and no cleanup error. Otherwise ready becomes degraded with
  `ready=false` and no endpoints; inactive, starting, and stopping also have
  none. This is lifecycle/ownership evidence, not an app health check. JSON
  mode is not Docker Compose JSON/template/table compatibility; every other ps
  format and generic/missing/unknown shim provenance fail closed before topology
  output. Text `logs SERVICE` preserves observed log bytes, then starts its
  additive VAT handoff JSON on a new line after them. `logs --format
  json [--tail LINES] SERVICE` (including equals forms, service final) emits one
  capture-only `vat.docker-compose.logs.v1` document with separate stdout/stderr,
  default-200/range-1..=1000 tail_lines, per-stream truncated/utf8_lossy,
  capture_only=true, runtime_invoked=false, and compose_record_mutated=false.
  It holds claim/provenance then reads VAT-captured logs: no Apple Container call
  or project.json mutation. VAT first caps each read and line tail, then after
  lossy UTF-8 plus JSON escaping retains a valid UTF-8 suffix whose serialized
  JSON string value remains within the same 64 KiB per-stream cap and marks it
  truncated; there are no topology/endpoints and JSON ps is next. It is not
  merged/follow/timestamp/template compatibility; follow, timestamps, and other
  flags fail closed. The full serial `vat_docker_shim` aggregate is intentionally
  not recorded because an independent serial run exposed a nondeterministic
  pre-existing Compose JSON logs timing race; the focused serialized-cap unit
  passed 1/1 for `0xff`-heavy and NUL/control-heavy streams after actual JSON
  serialization. The recorded opt-in real dual-service E2E includes this JSON
  logs shape for its bounded host-facing profile.
  Dependencies, networks, volumes, host-facing
  build, interpolation, `--env-file`, and every unsupported Compose form fail
  before runtime launch. Then use `docker compose -p PROJECT ps`, `logs
  SERVICE`, non-interactive `exec -T SERVICE -- COMMAND`, or `down`. Text
  exec preserves observed child bytes, then starts its additive VAT handoff JSON
  on a new line after them; its JSON form parses and validates the Docker-facing
  delimiter but does not forward it, invoking `container exec CONTAINER COMMAND [ARG...]`.
  `exec` proves the exact ready VAT-owned MicroVM service, forwards child exit
  code, and emits a runnable `ps` next handoff. A successful source-build `up` also
  returns exact VAT-owned `images` plus `cleanup_next`
  (`down && docker image rm <exact-tag>`); literal-image projects deliberately
  do not claim image ownership. Generic `vat compose` cannot operate a known
  shim record; an inactive generic re-import clears known provenance. Unknown
  inactive provenance permits registry-only cleanup preserving `vat.toml`, and
  unknown active provenance requires matching or newer VAT. Its deterministic
  fake coverage is supplemented by an opt-in gated real Apple Container
  dual-service E2E that passed 1/1 (50 filtered) on this host in 4.54 seconds:
  `RUST_TEST_THREADS=1 VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED=1 cargo
  test -p vat --test vat_docker_shim
  apple_container_docker_compose_host_facing_independent_profile_contract --
  --ignored --nocapture`. It proves `host-facing-independent-v1` `up -d
  --wait`, both loopback endpoints, one-document JSON `ps`, `logs`, and `exec`,
  text logs, text exec including a no-final-newline handoff, and `down` cleanup
  of exact containers, ports, and registry. It remains opt-in and proves
  neither service-name DNS nor general Compose, a Docker Engine API, or
  Kubernetes.
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
id = "fs"                  # GCP Firestore emulator (exports FIRESTORE_EMULATOR_HOST)
preset = "firestore"       # firestore | pubsub | datastore | bigtable | spanner | firebase

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
  `runtime = "native"` or `runtime = "docker"`. Explicit `runtime = "micro_vm"`
  uses Apple Container only: it checks the local image store, bounded-pulls and
  re-verifies a missing declared OCI route, then proves a loopback published
  port. Unsupported presets and MicroVM preset named volumes fail closed.
  Datastore/broker presets: postgres, redis, nats, rabbitmq, mysql, mongo.
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
  Container after the same bounded inspect/pull/verify preflight and never
  silently invokes Docker. It requires `container_port`; `image_env` is passed
  into the container; in `export`, `{host}`/`{port}` resolve to the mapped host
  endpoint, and `VAT_SERVICE_<ID>_{HOST,PORT}` are always exported.
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
- Built-in emulators: `preset = "pubsub"`, `"firebase-auth"`, `"cloud-tasks"`,
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
- `vat k8s ephemeral` is not that cluster surface. It uses an auto-boot Apple
  systemd machine and its exact inspected backing container for one K3s host
  command, then removes the private 0600 kubeconfig, cache, marker, and exact
  machine. It has no Docker daemon, durable kubeconfig, background daemon, or
  `vat cluster` state.
- `vat k8s session` extends that Docker-free substrate only as a bounded active
  lease: create with a TTL, run multiple explicit `exec <id> -- kubectl ...`
  calls with the same private credentials, inspect non-secret state, then
  delete exactly. Expired leases, changed backing identity/API endpoints, and
  uncertain creation fail closed; `session cleanup` is explicit, not a
  background reaper.
- `vat k8s session image load <id> <local-image-ref>` delivers one locally
  inspected `linux/arm64` Apple image to the active lease's K3s `k8s.io`
  namespace without Docker or a registry pull. Its private OCI archive is
  bounded and removed from host and guest after the canonical reference verifies;
  arbitrary tar files and cross-platform delivery fail closed.
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
- macOS native TCP presets can hit `kern.ipc.somaxconn` under connection churn
  and produce intermittent `ECONNREFUSED` even while the service is up. vat emits
  a structured `hint` when a service log reports that backlog cap. Prefer app
  connection pooling or raise the host limit, e.g.
  `sudo sysctl -w kern.ipc.somaxconn=1024`.

## Command Patterns

- `vat run`: select the default runner, prepare or clone service images, start
  required services, wait for readiness, run the runner, capture evidence, stop
  services, and return the runner exit code.
- `vat run --scenario prod-like`: start the named scenario's app service,
  scenario deps, and runner deps, then run its selected runner.
- `vat run e2e`: explicitly run the `e2e` runner.
- `vat run --keep always e2e`: override `[workspace].keep` for one invocation so
  a passing probe run remains inspectable via `vat logs` / `vat state`.
- `vat run -- cargo test -p app`: run one direct command without requiring
  vat.toml; the child exit code is forwarded.
- `vat logs <id> runner`: print retained runner stdout/stderr.
- `vat logs <id> <service-id>`: print retained service stdout/stderr.
- `vat state <id>`: read the agent-legible JSON state.
- `vat diff <id> --json`: read filesystem changes vs. the vat base.
- `vat cluster create [--backend auto|kind|k3d|minikube] [--name N]`: create a
  standalone local Kubernetes cluster (outlives a run); `vat cluster ls --json`,
  `vat cluster kubeconfig <name>`, and `vat cluster delete <name>` manage it.
- `vat k8s ephemeral image build`: explicitly build VAT's embedded systemd
  image into Apple Container. Its local tag identifies the embedded build asset
  revision, not a verified supply-chain image digest; it never starts a cluster.
- `vat k8s` requires an independently installed `kubectl` first on `PATH` and
  rejects an OrbStack-provided binary. The current Homebrew binary is installed
  at `/opt/homebrew/bin/kubectl`. Independent-kubectl one-shot, leased,
  local-image, and Service-forward E2Es passed 1/1 (36 filtered) in 28.38s,
  29.97s, 49.73s, and 49.57s. The local-image result is one already-local
  Apple `alpine:3.20` pod with `imagePullPolicy=Never`, a marker log, and exact
  session cleanup; it is not registry-pull generality.
- `vat k8s ephemeral run -- kubectl get nodes`: run one disposable K3s host
  command with a private kubeconfig; the isolated HOME keeps kubectl's normal
  cache private, and only a child shell can expand `VAT_K8S_CACHE_DIR`.
- Failed one-shot or leased bootstrap preserves the primary error, then emits
  only staged non-sensitive installer evidence under the fixed six-label,
  six-second-total / one-second-per-probe read-only diagnostic contract before
  exact cleanup. It never exposes private kubeconfig/cache or host credentials,
  changes neither the existing 300-second behavior nor persistence, does not
  rerun `k3s --version`, and adds no wrapper/recovery command. The deterministic
  fake regression passed. Independent-kubectl one-shot, leased, local-image,
  and Service-forward E2Es passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s,
  and 49.57s. The local-image result loads one already-local Apple `alpine:3.20`
  into one lease, runs a pod with `imagePullPolicy=Never`, observes its marker
  log, then completes exact session cleanup; it is not registry-pull generality.
  The leased result covers strict JSON exec with `--timeout 30`; the
  Service-forward result covers one Service-only loopback strict JSON tunnel,
  a credential-free child, confirmed cleanup, and closed local ports. These are
  bounded one-guest results, not persistence or a general cluster claim.
- `vat k8s session create --ttl 30m`: create one active one-boot K3s lease;
  use text `session exec --timeout 30 <id> -- kubectl get nodes` or JSON
  `session exec --format json --timeout 30 <id> -- kubectl get nodes -o json`
  for separate agent calls, then `session delete <id>` for exact
  credential/machine cleanup. Omitted timeout uses remaining lease TTL; explicit
  timeout is 1..=14400 seconds and cannot exceed it. Each command owns and reaps
  its process group under the private lock; normal exit, deadline, or
  SIGINT/SIGTERM removes the exec marker only after group absence. A starting or
  live crash marker blocks later exec, delete, and cleanup fail-closed rather
  than claiming termination. JSON emits one `vat.k8s.session.exec.v1` document
  with separate 64 KiB serialized-bounded streams and no raw replay. The
  independent-kubectl leased E2E passed 1/1 (36 filtered) in 29.97s with text
  commands, strict JSON `--timeout 30`, status verification, and exact delete.
  It is not a durable/restartable cluster or crash-safe termination guarantee.
- `vat k8s session image load <id> <local-image-ref>`: transfer one already
  local `linux/arm64` Apple image privately into K3s `k8s.io`, verify it, and
  remove both archive copies. The opt-in real-host local-image E2E passed 1/1
  (36 filtered) in 49.73s: one already-local Apple `alpine:3.20` pod ran with
  `imagePullPolicy=Never`, emitted its marker log, and then exact session cleanup
  completed. It does not establish registry-pull generality, persistence, GUI,
  Docker Engine/API, arbitrary tar, or a cross-platform local-image pipeline.

## Retention

Default `keep = "failed"` means successful configured runs clean up after
emitting JSON, while failed runs keep workspace state and logs for inspection.
Use `vat run --keep always ...` to retain one passing configured run without
editing `vat.toml`; use `--keep never` to force cleanup.

## Boundaries

- vat is not a Docker Engine/API or general-Compose replacement, a Linux runtime, a VM, a daemon,
  or a long-lived process manager. It is permanently headless: GUI/Desktop,
  dashboard, and tray/menu-bar surfaces are out of scope.
- vat offers an opt-in, fail-closed `docker` command shim over Apple
  Container: run `vat docker install-shim --dir <directory-on-PATH>`, add the
  directory to PATH, then use its documented command subset. It requires an
  explicit host port for `docker run -p` and rejects unsupported flags before
  runtime launch. Its five strict Apple-native JSON observation forms are
  direct container inventory, image inventory, direct image inspect, direct
  container inspect, and resource stats. Direct
  container inventory is only `docker ps --format
  json` / equals with optional exactly-once `--all` or `-a`, shared only by
  `docker container ls` and `docker container list`; inherited text behavior is
  unchanged. It runs `container list
  --format json [--all]`, validates one opaque Apple-native JSON value, and
  byte-for-byte replays stdout without a VAT wrapper or Docker Engine ps schema.
  Templates/table output, filters, quiet plus JSON, duplicate/unknown flags,
  positionals, and `docker container ps` JSON fail closed before runtime. It is
  read-only inventory, not ownership/health/readiness/liveness proof; one
  five-second bounded isolated cleanup path covers root exit and both pipe EOFs,
  and malformed, oversized, or escaped-pipe stdout fails closed without replay.
  Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0;
  `ps` is a global read-only inventory smoke observation, not targeted ownership
  evidence, and proves one valid native JSON document only. Fake/unit tests prove
  byte-preservation and fail-closed behavior. Image inventory is only `docker images
  --format json` / equals, shared only by `docker image ls` and `docker image
  list`; text/quiet image listing remains inherited. It runs `container image
  list --format json`, bounded-captures and validates one opaque Apple-native
  JSON value, then byte-for-byte replays stdout without a VAT wrapper or Docker
  Engine image schema. Template/table/YAML/TOML output, filters, quiet, verbose,
  all, digests, no-trunc, positionals, duplicates, unknown flags, and `--` fail
  before runtime. It makes no ownership/provenance/security/executability/
  registry/build-readiness/health/readiness/liveness claim; the same five-second
  bounded isolated cleanup covers root exit and both pipe EOFs, and malformed,
  oversized, or escaped-pipe stdout fails closed without replay. Direct real-host
  observation passed 1/1 (50 filtered) on Apple Container 1.1.0; `images` is a global
  read-only inventory smoke observation, not targeted ownership evidence, and
  proves one valid native JSON document only. Fake/unit tests prove byte-preservation
  and fail-closed behavior. Direct image inspect accepts only `docker image inspect --format json IMAGE` / equals: one selector precedes one opaque safe image reference, VAT strips it and invokes only `container image inspect IMAGE`, then bounded-captures, validates, and byte-for-byte replays one Apple-native document. Templates, `--`, extra references, and every other option reject before runtime; it claims no Docker image-inspect schema/template/Engine API, provenance, security, registry, build-completion, readiness, or secret redaction. Cargo check passed; docker_shim lib passed 58/58; focused coverage passed 4/4 with 1 ignored; its host E2E passed 1/1 (61 filtered) in 1.21s. Direct container inspect accepts only `docker inspect
  --format json CONTAINER` / equals, shared only by `docker container inspect`.
  It requires one safe explicit id after one VAT-only selector before runtime;
  unformatted inspect remains inherited. It runs `container inspect CONTAINER`,
  bounded-captures and validates one opaque Apple-native JSON value, then
  byte-for-byte replays stdout without a VAT wrapper or Docker Engine inspect
  schema. `--type`, `--size`, templates/table/YAML/TOML, filters, a second id,
  `--`, and unknown flags fail before runtime. One five-second bounded isolated
  observer covers root exit and both pipe EOFs; each stream is 256 KiB, valid JSON
  plus a nonzero child exit preserves status, and malformed, oversized, or flood
  output suppresses raw stdout. It makes no ownership/provenance/security/image/
  registry/build-status/health/readiness/liveness/port-reachability claim and gives
  no secret-redaction guarantee. Direct real-host observation passed 1/1 (50 filtered)
  on Apple Container 1.1.0; `inspect` targets the temporary owner-labeled
  nginx container and proves one valid native JSON document only. Fake/unit tests
  prove byte-preservation and fail-closed behavior. Direct logs JSON is a separate
  finite VAT wrapper, not a sixth Apple-native form: it accepts only `docker logs
  --format json --tail LINES CONTAINER` / equals forms and the same form through
  `docker container logs`, with one format and one tail before one safe final id
  and `LINES` in 1..=1000; unformatted logs remains inherited. VAT invokes only
  `container logs -n LINES CONTAINER`, never forwards the selector, and emits one
  `vat.docker.logs.v1` / `vat_json` wrapper—not Docker multiplex/demux—with
  untrusted stdio, bounded diagnostic stderr, truncation/lossy flags,
  backend/container/requested_tail/runtime/child outcome, and safe inspect next.
  Follow/boot/timestamps/since/until/templates and all other modifiers reject
  before runtime; timeout/setup/escaped-pipe capture yields no partial wrapper
  after five-second plus one-second bounded cleanup with dual-stream
  suffix/serialized caps. Focused `docker_logs_json` integration passed 6/6;
  direct real-host observation passed 1/1 (50 filtered) on Apple
  Container 1.1.0 and targets the high-entropy nonce+PID owner-labeled temporary
  nginx container, proving one VAT wrapper only; exact-label rechecks are conservative
  best-effort precautions, the emergency guard retains on uncertainty, and Apple
  Container has no atomic conditional delete, so this is not a race-free or
  impossible-to-misdelete cleanup guarantee. It does not clean up the shared/cacheable
  nginx image. Fake/unit
  tests prove byte-preservation and fail-closed behavior. Direct exec JSON is only
  `docker exec --format json --timeout SECONDS CONTAINER -- COMMAND [ARG...]` /
  equals forms and the matching `docker container exec`: one format plus one
  1..=1200 timeout precede a safe id and a mandatory Docker-facing delimiter/raw
  command, while raw/unformatted exec stays inherited. VAT strips selectors and
  delimiter before `container exec CONTAINER COMMAND [ARG...]`, then emits one
  `vat.docker.exec.v1` / `vat_json` wrapper with bounded untrusted stdout/stderr
  suffixes, `timeout_scope=host-container-client-observation`, child outcome, no
  redaction guarantee, and safe inspect next. Both serialized stream values cap
  at 64 KiB; ordinary child failure retains wrapper+exit and timeout/setup/capture
  failure emits no partial wrapper. The timeout does not claim guest command
  termination. Docker shim library validation passed 54/54 and focused
  `docker_exec_json` integration passed 4/4; direct host evidence passed 1/1 (50
  filtered) with an exec wrapper carrying both stdout and stderr markers. It is not
  Docker Engine parity, generic runtime, Compose, or Kubernetes evidence. Strict
  direct run JSON is a foreground, owner-cleaned one-shot: only direct `docker
  run --format json --timeout SECONDS IMAGE [COMMAND...]` (or equals forms) is
  accepted. Exactly one format and one 1..=1200 timeout may occur in either order
  before IMAGE; optional command argv follows IMAGE directly. The JSON form
  rejects a Docker `--` before IMAGE or immediately after IMAGE; after the first
  non-`--` command token, later `--` is opaque child argv. It also rejects detach,
  TTY, interactive, caller name/label, ports, network, mounts, env, and every
  other run option before Apple Container starts. VAT generates a high-entropy
  name and independent owner label, captures
  bounded stdout/stderr into one `vat.docker.run.v1` / `vat_json` document, and
  exposes a normal nonzero child exit only after exact owner-label cleanup confirms
  absence. Timeout, setup, or cleanup uncertainty emits no partial wrapper; only
  Apple's explicit `Error: container not found: <name>` proves an already-absent
  generated container. The timeout bounds host Apple Container client observation
  only and makes no guest-wide termination, crash-recovery cleanup, Docker Engine
  parity, or secret-redaction claim. Focused deterministic validation passed 5
  passed plus 1 ignored in 1.80s. The real local `alpine:3.20` E2E passed 1/1
  (56 filtered) in 2.30s with one JSON document and exact cleanup. Strict direct
  build JSON is a separate bounded VAT receipt, not a sixth Apple-native JSON
  observation. Only direct `docker build --format json --timeout SECONDS --tag
  TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE] [--platform
  PLATFORM] [--label K=V ...] CONTEXT` (or documented equals forms) is accepted:
  format `json`, positive whole timeout 1..=1200, and tag occur exactly once;
  file/target/platform occur at most once; build args/labels may repeat; every
  option precedes one canonical existing local-directory context. `--`, missing,
  duplicate, misordered, or unsupported forms fail before the builder; raw builds
  without either selector remain inherited. VAT strips only JSON/deadline selectors,
  invokes public `container build --tag TAG [--file ...] [--build-arg ...]
  [--target ...] [--platform ...] [--label ...] CONTEXT`, then emits one bounded
  `vat.docker.build.v1` / `vat_json` receipt after the Apple client exits. It has
  untrusted bounded stdout/stderr, truncation/lossy flags, timeout scope, child
  outcome, and `image_lifecycle=retained_no_auto_cleanup`: no product cleanup or
  ownership claim. Success safely points to strict image inspect; normal child
  failure retains receipt/exit but is terminal `build_failed` with `docker --help`,
  never a stale inspect handoff. Timeout/setup/capture failure has no receipt; the
  deadline is host observation only, not cancellation/rollback/removal. It makes
  no Docker Engine/API, provenance, ownership, readiness, security, redaction,
  cancellation, or rollback claim. Current evidence: cargo check passed,
  docker_shim lib 62/62, focused build JSON 4 plus 1 ignored (63 filtered),
  native image owner guard 1/1 (67 filtered), and host receipt E2E 1/1 (67
  filtered) in 2.53s. That test's high-entropy tag/exact owner label plus pre/post
  absence and pre-delete recheck are test-only safety: no conditional Apple build/
  delete means best-effort races can leak, never product auto-cleanup. Strict
  direct pull JSON is a separate bounded VAT receipt, not Apple-native JSON: only
  direct `docker pull --format json --timeout SECONDS IMAGE` (or documented
  equals forms) is accepted. Exactly one json format and one positive whole
  1..=1200 timeout may be reordered before one safe opaque image reference.
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
  neither deletes the image nor asserts ownership on success or failure. Its
  strict `docker stats --no-stream
  --format json CONTAINER [CONTAINER...]` form is a separate five-second bounded
  native-JSON Apple observation, not a Docker Engine wrapper/schema or
  ownership/health/liveness proof; only complete 256 KiB-capped capture is
  replayed, and escaped pipes fail closed without stdout replay. Direct real-host
  observation passed 1/1 (50 filtered) on Apple Container 1.1.0; `stats` targets the
  temporary owner-labeled nginx container and proves one valid native JSON
  document only. Fake/unit tests prove byte-preservation and fail-closed behavior.
- The shim's Compose support has exactly three profiles: one literal-image
  service (`strict-single-image-v1`, `up -d`); one literal short build-only
  service (`strict-single-build-v1`, `up -d --build`); or two through four
  literal-image services selected only by the exact top-level
  `x-vat-compose-profile: host-facing-independent-v1` marker. Every
  host-facing service has one unique nonzero `host:container` port published on
  loopback only, and successful JSON makes the negative contract explicit with
  `profile=host-facing-independent-v1`, `service_name_dns=false`, and
  `host_loopback_only=true`. It has no service-name DNS or bridge networking.
  Strict `docker compose --dry-run -f FILE -p PROJECT up -d [--build]` parses
  only those profiles, emits its one VAT preflight JSON document with the
  validated/no-runtime/no-registry/no-image-build/launch-revalidation fields,
  invokes no Apple Container command, and makes the returned real launch
  use the canonical source path to revalidate the same file after a cwd change.
  It rejects `--wait` and every other global/Compose flag;
  it is not generic Compose dry-run compatibility.
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
  `docker compose -p PROJECT ps` keeps no-format text plus its additive
  known-profile JSON, while `--format json` and `--format=json` emit exactly
  one VAT-owned `vat.docker-compose.ps.v1` / `vat_json` document with the same
  claim-held topology and no human table. Services retain registered Compose
  order and endpoints are canonical `127.0.0.1:<port>` strings only
  when every expected service proves unique Ready VAT-owned `container_run`
  evidence for its exact MicroVM, loopback nonzero port, and no cleanup error.
  Any missing proof makes ready degraded with `ready=false` and no endpoints;
  inactive, starting, and stopping also have none. It is not an app health
  check. The JSON form is not Docker Compose JSON/template/table compatibility;
  every other ps format and generic/missing/unknown provenance fail closed before
  topology output. Text `logs SERVICE` preserves observed log bytes, then
  starts its additive VAT handoff JSON on a new line after them, while JSON logs
  has one capture-only VAT document with separate stdout/stderr,
  bounded default-200/range-1..=1000 tail, per-stream truncated/utf8_lossy,
  no runtime call/project.json mutation/topology/endpoints, and JSON-ps next.
  VAT first caps each read and line tail, then after lossy UTF-8 plus JSON
  escaping retains a valid UTF-8 suffix whose serialized JSON string value
  remains within the same 64 KiB per-stream cap and marks it truncated. It is
  not Docker Compose merged/follow/timestamp/template compatibility; follow,
  timestamps, and other flags fail closed. The full serial `vat_docker_shim`
  aggregate is intentionally not recorded because an independent serial run
  exposed a nondeterministic pre-existing Compose JSON logs timing race; the
  focused serialized-cap unit passed 1/1 for `0xff`-heavy and NUL/control-heavy
  streams after actual JSON serialization. The recorded opt-in real dual-service
  E2E includes this JSON logs shape for its bounded host-facing profile.
  Dependencies, networks, volumes, host-facing build, interpolation,
  `--env-file`, default TTY, and other unsupported flags fail before runtime
  launch. Then `ps`, `logs SERVICE`, non-interactive `exec -T SERVICE --
  COMMAND`, and `down` operate only through valid shim provenance on VAT's
  exact ready MicroVM service. Text exec starts its additive VAT handoff JSON
  on a new line after observed child bytes; its JSON form parses and validates
  the Docker-facing delimiter without forwarding it to `container exec CONTAINER COMMAND [ARG...]`.
  Generic `vat compose` cannot operate a known
  shim record; a normal inactive generic re-import clears known provenance,
  while unknown inactive provenance receives registry-only cleanup preserving
  `vat.toml` and unknown active provenance requires matching or newer VAT.
  A successful source-build `up` additionally returns exact VAT-owned `images`
  and `cleanup_next` (`down && docker image rm <exact-tag>`); a literal-image
  project deliberately receives neither ownership field. The host-facing
  two-to-four-service path has deterministic fake-lifecycle coverage plus an
  opt-in real Apple Container dual-service E2E passed 1/1 (50 filtered) on this
  host in 4.54 seconds. That gate proves only `host-facing-independent-v1`
  `up -d --wait`, both loopback endpoints, one-document JSON `ps`, `logs`, and
  `exec`, text logs, text exec including a no-final-newline handoff, and `down`
  cleanup of exact containers, ports, and registry—not service-name DNS,
  general Compose, a Docker Engine API, or Kubernetes.
- The shim still does not expose a Docker Engine socket/API and does not imply
  Compose, SDK, Testcontainers, devcontainer, Docker output-schema, or Docker
  network compatibility. It is an agent shell-command bridge, not a daemon.
- Apple Container K3s is a bounded one-boot path, not a persistent Kubernetes
  Desktop replacement: `ephemeral` is one command and `session` is a bounded
  active lease. Both require an independently installed `kubectl` first on
  `PATH` and reject an OrbStack-provided binary; this is host-tool provenance,
  not a GUI or Docker Engine dependency. A lease can load one verified local `linux/arm64` image, but
  neither path has restart safety, reboot-safe retention, storage, ingress/LB,
  or multi-node promise while machine restart is blocked. Diagnostics do not
  repair bootstrap or alter its existing 300-second behavior: root error comes
  first, then exactly the fixed six-label read-only evidence under its bounded
  six-second-total / one-second-per-probe budget before exact cleanup. No
  private kubeconfig/cache or host credential is exposed, `k3s --version` is
  not rerun, and no wrapper/recovery command is added. Homebrew
  `/opt/homebrew/bin/kubectl` is installed locally; independent-kubectl one-shot,
  leased, local-image, and Service-forward E2Es passed 1/1 (36 filtered) in
  28.38s, 29.97s, 49.73s, and 49.57s. The local-image result loaded one
  already-local Apple `alpine:3.20` into one lease, ran a pod with
  `imagePullPolicy=Never`, observed its marker log, and completed exact session
  cleanup; it is not registry-pull generality. The leased result includes strict
  JSON exec with `--timeout 30`.
  Text Service-forward starts its terminal record on a new line after child output;
  its `--format json` form passed the independent-kubectl Service-forward E2E,
  which covered one Service-only loopback strict JSON tunnel with a
  credential-free child, confirmed cleanup, and closed local ports. It remains
  bounded one-guest evidence, not a general tunnel, persistence, ingress/LB,
  public listener, or same-UID OS-sandbox claim.
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

const TOPICS: &[cli_std::llm::Topic] = &[cli_std::llm::Topic {
    id: "guide",
    summary: "complete vat agent usage contract: run modes, services, evidence, and boundaries",
    body: GUIDE,
}];

/// @spec apps/vat/tech-design/logic/llm-agent-usage-guide.md#cli
pub fn exec(topic: &str, format: cli_std::llm::Format) -> Result<ExitCode> {
    let out = cli_std::llm::render("vat", crate::VERSION, TOPICS, topic, format)?;
    println!("{out}");
    Ok(ExitCode::SUCCESS)
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/src/commands/llm.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `apps/vat/src/commands/llm.rs` captured during #39 vat standardization.
```
