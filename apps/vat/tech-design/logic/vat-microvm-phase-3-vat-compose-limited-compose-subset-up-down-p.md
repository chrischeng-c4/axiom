---
id: vat-microvm-phase-3-vat-compose-limited-compose-subset-up-down-p
summary: >
  Add `vat compose`, a bounded importer/runner for an existing, unmodified
  `docker-compose.yml` — expand its supported subset
  (services/volumes/version; per-service
  image/build/ports/environment/depends_on/volumes) into a real `vat.toml`
  and drive `up`/`down`/`ps`/`logs` for the resulting
  services via Apple's `container` CLI or Docker (project-wide
  `--runtime auto|docker|micro-vm`). Phase 3 (final) of the microVM epic
  (#1471); Phase 1 (#1474, merged) added `Isolation::MicroVm` for
  `vat run`, Phase 2 (#1479, merged) added `vat build`. The #1526
  fail-closed endpoint and compose-startup reconciliation is a correctness
  dependency: active compose state is imported -> starting -> ready -> stopping.
  Both foreground and detached startup use one crash-safe, token-owned
  ComposeHandoff; only its owner may synchronously publish a durably-created
  VAT. Current registries retain `handoff_protocol: 1` after their transient
  handoff fields clear, so a later missing metadata file cannot turn a current
  binding into a reclaimable historical record. Down is acknowledged by the VAT
  parent before terminal cleanup resets only the active run binding back to
  imported. #1529 makes build-bearing imports source-location deterministic:
  `build.context` and an explicit `build.dockerfile` are resolved from the
  canonical compose source directory, `build.args` retain list order or stable
  map-key order, and an OCI-safe readable project-scoped tag with a BLAKE3
  raw-pair identity suffix is built into the same local image store that the
  generated service will later use. A selected builder is preflighted before any
  build or `vat.toml` replacement; materialization itself is temp-write, sync,
  and rename, with rollback handling if its matching registry record cannot be
  published.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: vat-compose-bounded-compose-subset-up-down-ps-logs
    claim: vat-compose-bounded-compose-subset-up-down-ps-logs
    coverage: full
    rationale: "Adds compose.rs (parse/expand/materialize), commands/compose.rs (Import/Up/Down/Ps/Logs verbs and the ComposeRecord registry), the Cmd::Compose CLI variant, ServiceRuntime::MicroVm plus ServiceConfig.volumes in config.rs, prepare_microvm_service()/container_run_command() plus runtime dispatch and early-persist changes in run.rs, and RunnerRunRecord.pid in state.rs. #1526 makes foreground and detached startup evidence-based through one persistent advisory-lock and token-matched ComposeHandoff, with durable handoff_protocol: 1 provenance after PID/token clear; compose down writes a parent-owned stop request and waits for terminal VAT cleanup before resetting only the active binding. #1529 canonicalizes the compose source, resolves context/dockerfile paths and args deterministically, maps runtime to an isolated image store before import, uses OCI-safe project-scoped tags with raw-pair BLAKE3 identity suffixes, atomically replaces generated vat.toml with rollback handling if registry publication fails, and checks an inactive imported registry's order-insensitive service-ID set against a parseable materialized config before a fresh up."
---

# vat MicroVm Phase 3: vat compose (limited compose subset + up/down/ps/logs)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-compose-phase3-exec-logic
entry: start
nodes:
  start: { kind: start, label: "vat compose <subcommand> invoked: import|up|down|ps|logs" }
  route: { kind: decision, label: "which subcommand" }
  parse: { kind: process, label: "compose::parse canonicalizes the compose source path, reads YAML into ComposeFile, retains that path outside the document for diagnostics and build resolution, then rejects unsupported top-level keys other than services volumes version x-" }
  key_check: { kind: decision, label: "every top-level and per-service key is in the supported subset R2 or an ignored x- key" }
  reject: { kind: terminal, label: "hard Err naming file plus service plus key: compose file file service id uses unsupported key key reason remove it or edit the generated vat.toml directly after vat compose import R3; deploy secrets configs extends networks profiles healthcheck command entrypoint override bind mount volumes all land here" }
  build_present: { kind: decision, label: "any service is build-bearing: image is absent and build is present" }
  build_runtime: { kind: process, label: "for a build-bearing import, map runtime auto native docker to Docker and microvm to Apple Container, then preflight that exact store before any build or vat.toml replacement #1529" }
  expand_build: { kind: process, label: "image wins unchanged; a build-only short or full build form resolves context and explicit dockerfile from the canonical compose source directory, defaults Dockerfile inside resolved context, parses args list K=V in list order or non-null map values in stable key order, tags vat-sanitized-project-sanitized-service-b3-BLAKE3(project NUL service):latest, builds in the preflighted store, writes returned tag to ServiceConfig.image, and pins the built service to that concrete runtime #1529" }
  expand_fields: { kind: process, label: "expand maps ports H:C and bare C forms, environment list or map a bare key with no value is a hard reject, depends_on list or map condition service_healthy is a hard reject else mapped 1:1 onto ServiceConfig.requires, and named volume volumes entries onto ServiceConfig.volumes" }
  depends_warn: { kind: process, label: "any service whose depends_on is non empty after expansion gets a printed non fatal warning: vat compose does not simulate container to container bridge network DNS; depends_on only orders startup use VAT_SERVICE_ID_HOST PORT instead R3" }
  runtime_assign: { kind: process, label: "image-only services retain the project-wide --runtime flag; build-backed services persist Docker for auto native docker or MicroVm for microvm, preventing later run dispatch from drifting to a different image store #1529" }
  materialize: { kind: process, label: "materialize serializes one service per compose service plus synthesized runner project.up cmd sleep 2147483647 requires every service id, then atomically temp-writes syncs and renames vat.toml so a failure cannot expose a truncated replacement #1529" }
  registry_commit: { kind: process, label: "import re-reads materialized vat.toml for service_ids then atomically publishes a matching project.json; if validation or registry publication fails, it attempts to restore the prior vat.toml or remove a fresh one, and a rollback failure is reported while a later fresh inactive up fails closed through the registry/config gate #1529" }
  import_ok: { kind: terminal, label: "matching vat.toml and project.json committed at the compose registry directory ExitCode SUCCESS ready for vat compose up" }
  up_expand: { kind: process, label: "under the persistent startup.lock claim, up loads project.json; only an inactive imported record with no vat_id compares its order-insensitive service-ID set to a parseable current vat.toml, and a mismatch fails with re-import remediation. Bound or active records reconcile from VAT evidence without this config gate, while malformed or unreadable config defers to vat run's existing parse failure; compatible direct edits need no full config digest #1529" }
  up_mode: { kind: decision, label: "--detach flag" }
  up_fg_handoff: { kind: process, label: "foreground constructs ComposeHandoff project plus token, persists the token while holding the claim, and passes that exact handoff directly to in-process run::exec; it does not start a VAT-store poller" }
  up_fg_call: { kind: process, label: "foreground call commands::run::exec run::Args target run::Target::Runner runner_ids project.up name Some project compose_handoff Some handoff in process; the token owner registers then synchronously publishes after durable VAT creation before services start, and the call blocks until teardown completes" }
  runtime_route: { kind: decision, label: "inside that run run_configured prepare_service dispatches each image backed ServiceConfig on its ServiceRuntime R4 R5" }
  docker_path: { kind: process, label: "ServiceRuntime Auto Native or Docker unchanged prepare_image_service plus docker_run_command path existing untouched" }
  microvm_path: { kind: process, label: "ServiceRuntime MicroVm new prepare_microvm_service plus container_run_command path structurally mirroring the Docker pair R5" }
  runner_persist: { kind: process, label: "run_configured persists an interim RunnerRunRecord status Running pid Some child.id into vat.meta.test_run.runner runners immediately after spawning the runner process before blocking wait_runner_processes; this is readiness evidence only, never compose down authority" }
  up_fg_result: { kind: decision, label: "commands::run::exec result once the run and its teardown complete" }
  up_fg_err: { kind: terminal, label: "propagate the non zero ExitCode; ps or down later reconciles retained terminal VAT evidence and resets the active binding to imported" }
  up_fg_ok: { kind: terminal, label: "ExitCode SUCCESS once the foreground run and its full stop_services teardown tail completes" }
  up_bg: { kind: process, label: "detached constructs the same ComposeHandoff project plus token, persists its token plus startup_started_at, re-execs vat run project.up with VAT_COMPOSE_PROJECT and VAT_COMPOSE_STARTUP_TOKEN, records the spawned pid while still holding the claim, then releases it" }
  handoff_register: { kind: process, label: "at vat run entry the token owner waits at most ten seconds on the same internal handoff claim, token-matches the starting record, records its own launcher pid, and aborts before creating a VAT if ownership or lifecycle no longer matches; external lifecycle commands remain non-blocking" }
  handoff_publish: { kind: process, label: "immediately after durable VAT creation and before services start, the token owner waits at most ten seconds on the internal claim, token-matches, synchronously publishes vat_id, clears startup_pid startup_token startup_started_at, and retains handoff_protocol 1; publication failure hard-fails the run" }
  up_bg_reread: { kind: process, label: "parent waits about 10s by rereading only the token-owned project.json registry; it may accept a child-published vat_id but never discovers or writes vat_id from global VAT-store name or time evidence" }
  up_bg_result: { kind: decision, label: "child-published vat id and reconciled evidence state" }
  up_bg_timeout: { kind: terminal, label: "emit status starting only while the token-backed child is live; token plus no launcher pid becomes terminal after the two-second grace window so a parent crash before spawn cannot wedge future up ps or down" }
  up_bg_ok: { kind: terminal, label: "print project vat_id and status starting ready or stopping; ready requires every service Ready and a live synthesized runner pid, while stopping retains a published binding until VAT and service cleanup are terminal" }
  down_read: { kind: process, label: "down acquires and holds the registry claim through reconciliation stop acknowledgement cleanup confirmation and reset" }
  down_reconcile: { kind: decision, label: "reconcile durable evidence as starting, ready, stopping, evidence unavailable, terminal, or cleanup unconfirmed; persisted runner pid is readiness evidence only" }
  down_starting: { kind: terminal, label: "starting is retryable and retains the active binding until the child publishes or fails" }
  down_stopping: { kind: process, label: "runner-exited or terminal-service evidence while VAT remains Status::Running is stopping; retain the binding and wait for the VAT parent, service terminalization, and cleanup confirmation rather than resetting" }
  down_terminal: { kind: process, label: "for current handoff_protocol 1 records, only Status::Exited plus every tracked service terminal and no cleanup_error is terminal; then reset only the active binding to imported and report already terminated" }
  down_evidence_unavailable: { kind: terminal, label: "current handoff_protocol 1 VAT metadata load/read/malformed/missing failure is EvidenceUnavailable, never terminal: return retry remediation and retain the registry; only a protocol-absent historic record plus a separate metadata NotFound may take compatibility recovery" }
  down_cleanup_retry: { kind: process, label: "cleanup-unconfirmed loads the retained VAT and retries only each persisted Docker or MicroVM resource by name; a nonzero rm -f clears cleanup_error only when a successful bounded exact-name list proves absent, otherwise evidence remains retained" }
  down_cleanup_blocked: { kind: terminal, label: "a cleanup retry failure leaves the binding retained and returns state plus retry-down remediation; no future up can reuse the published port" }
  down_request: { kind: process, label: "ready down loads vat_id and writes vat-dir .compose-stop-request; it never signals a persisted OS pid directly" }
  down_wait: { kind: process, label: "VAT wait_runner_processes consumes the request, kills its own runner tree, and stop_services marks even an already-reaped owned Ready child Exited; compose waits bounded for Status::Exited, all compose services terminal, and no cleanup_error, otherwise it leaves status stopping and retains the registry" }
  down_reset: { kind: process, label: "after acknowledgement and confirmed cleanup clear vat_id startup_pid startup_token and startup_started_at, retain handoff_protocol 1 plus project/service import metadata, and persist status imported in project.json" }
  down_ok: { kind: terminal, label: "ExitCode SUCCESS with the project immediately reusable by another compose up AC5" }
  ps_read: { kind: process, label: "ps holds the claim while it reconciles the ComposeRecord; starting, ready, stopping, evidence-unavailable, terminal, and cleanup-unconfirmed are evidence states before it may load vat_id and service_ids" }
  ps_evidence_unavailable: { kind: terminal, label: "print actionable retry error and retain the current handoff_protocol 1 binding; malformed, unreadable, and missing metadata are never terminal/resettable, while only protocol-absent historic metadata NotFound may recover" }
  ps_stopping: { kind: terminal, label: "print stopping and retain the project binding; runner exit alone is not permission to reset while VAT Status::Running, services, or cleanup are pending" }
  ps_load: { kind: process, label: "for ready records ps loads vat_id then store::load vat_id to get that run test_run.services" }
  ps_filter: { kind: process, label: "filter test_run.services by service_ids membership R10 reusing commands logs.rs existing linear scan by id approach" }
  ps_ok: { kind: terminal, label: "print the filtered per service id runtime status port table ExitCode SUCCESS" }
  logs_read: { kind: process, label: "logs reads the ComposeRecord for project then store::load vat_id" }
  logs_filter: { kind: process, label: "locate the named service among the service_ids filtered set R10" }
  logs_missing: { kind: terminal, label: "hard Err service not found in this compose project service_ids" }
  logs_ok: { kind: terminal, label: "print that service captured stdout stderr the same way commands logs.rs existing per source branch does ExitCode SUCCESS" }
edges:
  - { from: start, to: route }
  - { from: route, to: parse, label: "import" }
  - { from: route, to: up_expand, label: "up" }
  - { from: route, to: down_read, label: "down" }
  - { from: route, to: ps_read, label: "ps" }
  - { from: route, to: logs_read, label: "logs" }
  - { from: parse, to: key_check }
  - { from: key_check, to: reject, label: "unsupported" }
  - { from: key_check, to: build_present, label: "supported" }
  - { from: build_present, to: build_runtime, label: "build-bearing" }
  - { from: build_present, to: expand_build, label: "image-only" }
  - { from: build_runtime, to: expand_build, label: "preflight passes" }
  - { from: expand_build, to: expand_fields }
  - { from: expand_fields, to: depends_warn }
  - { from: depends_warn, to: runtime_assign }
  - { from: runtime_assign, to: materialize }
  - { from: materialize, to: registry_commit }
  - { from: registry_commit, to: import_ok }
  - { from: up_expand, to: up_mode }
  - { from: up_mode, to: up_fg_handoff, label: "foreground" }
  - { from: up_mode, to: up_bg, label: "detach" }
  - { from: up_fg_handoff, to: up_fg_call }
  - { from: up_fg_call, to: runtime_route }
  - { from: runtime_route, to: docker_path, label: "auto native or docker" }
  - { from: runtime_route, to: microvm_path, label: "microvm" }
  - { from: docker_path, to: runner_persist }
  - { from: microvm_path, to: runner_persist }
  - { from: runner_persist, to: up_fg_result }
  - { from: up_fg_result, to: up_fg_err, label: "nonzero" }
  - { from: up_fg_result, to: up_fg_ok, label: "zero" }
  - { from: up_bg, to: handoff_register }
  - { from: handoff_register, to: handoff_publish }
  - { from: handoff_publish, to: up_bg_reread }
  - { from: up_bg_reread, to: up_bg_result }
  - { from: up_bg_result, to: up_bg_timeout, label: "not found" }
  - { from: up_bg_result, to: up_bg_ok, label: "found" }
  - { from: down_read, to: down_reconcile }
  - { from: down_reconcile, to: down_starting, label: "starting" }
  - { from: down_reconcile, to: down_stopping, label: "stopping" }
  - { from: down_reconcile, to: down_evidence_unavailable, label: "evidence unavailable" }
  - { from: down_reconcile, to: down_terminal, label: "terminal" }
  - { from: down_reconcile, to: down_cleanup_retry, label: "cleanup unconfirmed" }
  - { from: down_reconcile, to: down_request, label: "ready" }
  - { from: down_terminal, to: down_ok }
  - { from: down_cleanup_retry, to: down_terminal, label: "confirmed" }
  - { from: down_cleanup_retry, to: down_cleanup_blocked, label: "still unconfirmed" }
  - { from: down_stopping, to: down_wait, label: "continue acknowledgement" }
  - { from: down_request, to: down_wait }
  - { from: down_wait, to: down_reset, label: "exited plus terminal services plus cleanup confirmed" }
  - { from: down_wait, to: down_stopping, label: "still running or cleanup pending" }
  - { from: down_reset, to: down_ok }
  - { from: ps_read, to: ps_stopping, label: "stopping" }
  - { from: ps_read, to: ps_evidence_unavailable, label: "evidence unavailable" }
  - { from: ps_read, to: ps_load, label: "ready" }
  - { from: ps_load, to: ps_filter }
  - { from: ps_filter, to: ps_ok }
  - { from: logs_read, to: logs_filter }
  - { from: logs_filter, to: logs_missing, label: "not found" }
  - { from: logs_filter, to: logs_ok, label: "found" }
---
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
title: vat MicroVm Phase 3 -- vat compose data model additions
type: object
properties:
  compose_file_struct:
    type: object
    description: |-
      apps/vat/src/compose.rs struct ComposeFile (Deserialize via serde_yaml):
      a serde-skipped canonical source_path: PathBuf, services:
      BTreeMap<String, ComposeService>, ignored volumes/version fields, and flattened
      extra keys. source_path() exposes the canonical file location to import
      diagnostics and build resolution; it is never taken from caller cwd or YAML.
  compose_service_struct:
    type: object
    description: |-
      New apps/vat/src/compose.rs struct ComposeService (Deserialize): image: Option<String>,
      build: Option<ComposeBuild> (enum Short(String) or Full{context: String,
      dockerfile: Option<String>, args: Option<ComposeEnv>}), ports: Vec<String> (raw
      H:C or bare C strings, parsed by expand()), environment: Option<ComposeEnv>
      (enum List(Vec<String>) or Map(BTreeMap<String, Option<String>>) -- a Map entry
      whose value is None, or a List entry with no `=`, is R3's bare-key-no-value hard
      reject), depends_on: Option<ComposeDependsOn> (enum List(Vec<String>) or
      Map(BTreeMap<String, ComposeDependsOnEntry{condition: Option<String>}>) --
      condition equal to service_healthy is R3's hard reject, every other form or
      absence is pure start-ordering), volumes: Vec<String> (named-volume:path entries
      only; any entry whose host segment before the colon looks like a filesystem path
      is bind-mount form and a hard reject per R3), plus #[serde(flatten)]
      extra: BTreeMap<String, serde_yaml::Value> capturing every other key so parse()
      can detect x- prefixed keys (ignored) vs. hard-reject keys (deploy, secrets,
      configs, extends, networks, profiles, healthcheck, command, entrypoint) by
      iterating extra's keys.
  compose_parse_fn:
    type: object
    description: |-
      apps/vat/src/compose.rs fn parse(path: &Path) -> Result<ComposeFile>. It first
      canonicalizes and reads the source file, stores that path in the skipped
      source_path field, deserializes via serde_yaml, then walks every top-level key
      (services, volumes, version, x- prefixed keys allowed, everything else
      hard-reject) and every per-service extra key (x- prefixed ignored, R3's list
      hard-reject) -- producing this exact error text:
      compose file {file} service {id} uses unsupported key {key} -- {reason}; remove
      it or edit the generated vat.toml directly after vat compose import. Top-level
      (non-service) hard-reject keys use {file} with no service segment.
  compose_expand_fn:
    type: object
    description: |-
      apps/vat/src/compose.rs fn expand(file: &ComposeFile, project: &str, runtime:
      ServiceRuntime) -> Result<Vec<ServiceConfig>> (R1/#1529). If any service is
      build-bearing (image absent, build present), it calls
      commands::build::resolve_image_builder(runtime) before the service loop:
      Auto/Native/Docker select Docker; MicroVm selects Apple Container. That selected
      store must be usable before a build or vat.toml materialization can occur.
      Image services remain unchanged. A build-only service resolves short context or
      full build.context and an explicit build.dockerfile relative to the canonical
      compose source directory (default Dockerfile is inside resolved context), parses
      build.args as K=V list entries in declaration order or non-null map values in
      BTreeMap key order, assigns the deterministic
      `vat-<sanitized-project>-<sanitized-service>-b3-<BLAKE3(project NUL service)>:latest`
      tag, and calls build_image_with_builder() in process. The readable prefix is
      OCI-safe but is not the ownership proof: the BLAKE3 suffix is over the raw
      project/service pair with a NUL separator, preventing delimiter ambiguity and
      normalization collisions. Its returned tag becomes ServiceConfig.image and its
      runtime becomes the concrete selected store (so auto cannot later drift).
      Image-only services retain the requested runtime. It then maps
      ports/env/depends_on/volumes as before and emits the non-fatal
      no-bridge-network-DNS warning.
  compose_materialize_fn:
    type: object
    description: |-
      apps/vat/src/compose.rs fn materialize(services: &[ServiceConfig], out: &Path)
      -> Result<()> (R1/#1529). Builds a VatConfig with cfg.services set to the given
      slice and one synthesized RunnerConfig (id: project.up, cmd: sleep 2147483647,
      requires every service id in expand order), serializes it via toml, then creates
      a unique sibling temporary file, writes and syncs it, and renames it over out.
      An error cleans up the temporary file and preserves the prior materialized
      vat.toml rather than exposing a truncate/write interval. Import re-parses the
      resulting service IDs before writing its matching registry record; if that
      validation or the atomic project.json publication fails, commands::compose
      calls restore_materialized_config() to restore the old contents (or removes a
      fresh config). If that rollback itself fails, the error reports both failures
      and a later fresh inactive up is refused by the registry/config gate.
  service_runtime_microvm_variant:
    type: object
    description: |-
      apps/vat/src/config.rs: ServiceRuntime gains a MicroVm variant (Auto, Native,
      Docker, MicroVm), still deriving clap::ValueEnum -- the same enum vat.toml's
      [[services]].runtime key already parses is reused verbatim for the new
      CLI-facing vat compose --runtime auto|docker|micro-vm flag (R4/R8), so no new
      parsing surface is introduced.
  compose_build_runtime_contract:
    type: object
    description: |-
      #1529 defines the compose-only build contract shared with commands/build.rs:
      ImageBuilder::Docker owns ServiceRuntime::Auto, Native, and Docker;
      ImageBuilder::MicroVm owns ServiceRuntime::MicroVm. Docker and Apple Container
      image stores are independent. A build-bearing import preflights exactly one of
      them before it invokes a builder or replaces its generated configuration, while
      an image-only import does not require either builder.
  service_config_volumes_field:
    type: object
    description: |-
      apps/vat/src/config.rs: ServiceConfig gains a
      #[serde(default, skip_serializing_if = "Vec::is_empty")] pub volumes:
      Vec<VolumeMount> field and a new struct VolumeMount { pub name: String,
      pub path: String } (named-volume-to-container-path pairs from a compose
      service's volumes: list, R2/R4). Applied as -v name:path argv entries by both
      docker_run_command and the new container_run_command when non-empty; empty on
      every non-compose vat.toml, so this is additive with zero effect on existing
      services.
  validate_runtime_gate_relaxation:
    type: object
    description: |-
      apps/vat/src/config.rs validate(): the existing gate that bails when
      service.runtime is not Auto and there is no preset becomes a gate that also
      allows an image-backed service (R4) -- an image-backed ServiceConfig may now
      declare runtime: docker or runtime: microvm explicitly (cmd services remain
      always-native and still bail); the error text is updated to name image services
      as an accepted case alongside preset services.
  prepare_service_dispatch_update:
    type: object
    description: |-
      apps/vat/src/commands/run.rs: prepare_service's image branch (the
      else-if-let-Some(image) arm), which today calls prepare_image_service
      unconditionally, gains a match on service.runtime: ServiceRuntime::MicroVm
      calls the new prepare_microvm_service(vat, service, image); every other value
      (Auto, Docker, Native) keeps calling prepare_image_service(vat, service, image)
      unchanged -- so the default (auto, and today's implicit Docker-only behavior) is
      bit-for-bit identical to pre-Phase-3 behavior (R4/R5).
  prepare_microvm_service_fn:
    type: object
    description: |-
      New apps/vat/src/commands/run.rs fn prepare_microvm_service(vat: &store::Vat,
      service: &ServiceConfig, image: &str) -> Result<ServicePlan>, structurally
      mirroring prepare_image_service line for line: calls the new private
      ensure_microvm_available() (R5) instead of ensure_docker_available(), builds its
      argv via the new container_run_command() instead of docker_run_command(), and
      returns a ServicePlan with the new microvm_name: Some(name) field set
      (docker_name stays None) so teardown removes the right container kind.
  run_ensure_microvm_available_fn:
    type: object
    description: |-
      New private apps/vat/src/commands/run.rs fn ensure_microvm_available() ->
      Result<()>, structurally mirroring the existing ensure_docker_available and
      commands/build.rs's private fn of the same name (not reusable across files
      since it is not pub there): requires sandbox::microvm::available() (container
      binary on PATH) and, if sandbox::microvm::system_up() is not immediately true,
      waits via sandbox::microvm::ensure_system_started(bounded timeout) before
      failing with a structured container_unavailable emit_jsonl error, mirroring
      ensure_docker_available's docker_unavailable shape.
  container_run_command_fn:
    type: object
    description: |-
      New apps/vat/src/commands/run.rs fn container_run_command(name: &str,
      image: &str, host_port: u16, container_port: u16,
      container_env: &BTreeMap<String, String>, volumes: &[VolumeMount]) ->
      Vec<String>, structurally mirroring docker_run_command: container, run, --rm,
      --name, name, -p, 127.0.0.1:host_port:container_port, then -v name:path per
      volumes entry, then -e key=value per sorted container_env entry, then image.
      Env and volumes both iterate in deterministic (sorted-key / input-slice) order,
      matching docker_run_command's existing determinism guarantee (the
      container_run_command_shape unit test asserts this exact argv).
  service_plan_handle_microvm_name:
    type: object
    description: |-
      apps/vat/src/commands/run.rs: ServicePlan and ServiceHandle both gain
      microvm_name: Option<String> (R5), set only by prepare_microvm_service --
      parallel to the existing docker_name: Option<String> field both structs already
      carry for the Docker path; start_service copies plan.microvm_name into the
      handle exactly the way it already copies plan.docker_name.
  stop_services_microvm_teardown:
    type: object
    description: |-
      apps/vat/src/commands/run.rs: stop_services() records both docker_name
      and microvm_name teardown evidence. Each docker rm -f or container rm -f
      uses a bounded, stdio-closed child; a timeout, wait error, or nonzero
      exit is recorded in ServiceRunRecord.cleanup_error (never
      readiness_error) unless a successful bounded exact-name list query proves
      absence: Docker uses container ls -a with an anchored name filter and an
      exact output-line comparison; MicroVM uses container list --all --format
      json and rejects a matching id. Query errors, timeouts, oversized or
      malformed output, and matches all retain cleanup_error. This accepts
      normal --rm auto-removal without accepting daemon outages. An already-reaped
      VAT-owned child is also marked Exited rather than left Ready. A non-empty
      cleanup_error forces runner/scenario nonzero retention and blocks
      published-port reuse until compose down retry confirms cleanup
      (R5/#1526).
  runner_run_record_pid_field:
    type: object
    description: |-
      apps/vat/src/state.rs: RunnerRunRecord gains a
      #[serde(default, skip_serializing_if = "Option::is_none")] pub pid: Option<u32>
      field (R7) -- the same optional, backward-compatible shape ServiceRunRecord.pid
      already uses; legacy metadata without this field deserializes with pid: None.
  runner_early_persist_behavior:
    type: object
    description: |-
      apps/vat/src/commands/run.rs run_configured(): immediately after the
      runner-spawn loop and strictly before the blocking
      wait_runner_processes(procs, stop_request_path)
      call, an interim Vec<RunnerRunRecord> is built from procs (status:
      ProcessStatus::Running, pid: Some(proc.child.id()), exit_code: None,
      duration_ms: None, command/stdout_log/stderr_log copied from each RunnerProc)
      and written into vat.meta.test_run.runner and test_run.runners, followed by
      vat.save() -- mirroring persist_services()'s existing early-write pattern for
      services. The PID is startup/readiness evidence only: compose down never
      treats it as a signal target. Instead it writes a stop request consumed
      by the VAT parent that owns the runner and service tree.
  compose_record_struct:
    type: object
    description: |-
      New apps/vat/src/commands/compose.rs struct ComposeRecord
      (Serialize/Deserialize, mirrors commands/cluster.rs's ClusterRecord):
      project: String, vat_id: Option<String>, handoff_protocol: u8
      (serde default 0 and omitted only at 0; current records persist 1),
      startup_pid: Option<u32>, startup_token: Option<String>,
      startup_started_at: Option<String>,
      service_ids: Vec<String>, status: String (imported, starting, ready, or
      stopping),
      created_at: String (RFC3339). Every compose registry read-modify-write
      holds StartupClaim on the persistent startup.lock inode and writes
      project.json by unique sibling temp file, sync_all, then rename, so
      readers observe a complete record and concurrent lifecycle transitions
      are serialized. Import seeds service_ids by parsing the newly materialized
      vat.toml. Before an inactive record (status imported with no vat_id) starts a
      fresh runtime set, load_and_validate_registry() attempts to parse the current
      vat.toml and compares its service-ID set to project.json's service_ids without
      table-order sensitivity; a parseable mismatch refuses launch with re-import
      remediation because cleanup ownership would otherwise be incomplete. A bound
      or active record bypasses this config check and reconciles from durable VAT
      evidence, so a later config edit cannot block cleanup. An unreadable or
      malformed vat.toml likewise defers to vat run's existing parse/validation
      failure rather than being masked by ownership bookkeeping. This is deliberately
      not a full vat.toml digest, so compatible direct edits that retain the
      service-ID set remain supported. ComposeHandoff { project, token } is the short-lived
      ownership proof for both foreground and detached starts. Up persists a
      fresh token and time before calling or spawning vat run. At vat run
      entry, only the token owner may record its PID; after durable VAT
      creation and before services start, only that same owner may
      synchronously publish vat_id and clear all three transient handoff fields
      while retaining handoff_protocol: 1. The durable marker is P1
      provenance: PID/token are deliberately transient, so without it a later
      missing metadata file could misclassify a current binding as historical.
      The parent may reread this token-owned registry for detached feedback,
      but never performs global VAT-store name/time discovery or writes a
      discovered vat_id. A token with no PID is terminal after the two-second
      grace window. The PID is liveness evidence only, never a compose-down
      signal target. Reset to imported clears active run fields but retains
      handoff_protocol: 1. Only protocol-absent historic records whose separate
      metadata stat is NotFound use legacy recovery; malformed, unreadable,
      permission, and all current-record failures remain fail-closed.
      Persisted at root/compose/project/project.json, where root is
      paths::root()? (R10) -- computed inline in commands/compose.rs rather than
      adding a helper to paths.rs, since paths.rs is not in this WI's file scope.
  compose_cmd_cli_variant:
    type: object
    description: |-
      apps/vat/src/cli.rs gains Cmd::Compose { cmd: ComposeCmd } and
      enum ComposeCmd { Import { file: PathBuf, project: Option<String>,
      runtime: ServiceRuntime }, Up { project: Option<String>, detach: bool },
      Down { project: String }, Ps { project: String }, Logs { project: String,
      service: String } } (R8). Import/Up's project defaults to the compose file's
      parent directory basename (sanitized the same way container_name() sanitizes)
      when omitted; Down/Ps/Logs require an explicit project naming an
      already-imported or running project (no other file to default from). runtime
      defaults to auto via ServiceRuntime's existing #[default], applied
      project-wide at import/up time, never per-service.
```

## Config
<!-- type: config lang: yaml -->

```yaml
files:
  - path: apps/vat/src/config.rs
    changes:
      - "ServiceRuntime enum gains a MicroVm variant (Auto, Native, Docker, MicroVm), still deriving clap::ValueEnum; existing vat.toml documents accepting runtime = docker or runtime = native today gain a fourth accepted string, runtime = microvm."
      - "ServiceConfig gains a new optional volumes array-of-tables key (default empty), each entry shaped { name = ..., path = ... } (VolumeMount), serialized/deserialized via a new pub struct VolumeMount { pub name: String, pub path: String }."
      - "validate()'s existing gate that requires a preset when runtime is not auto is relaxed to also accept an image-backed service (image = ... present) declaring runtime = docker or runtime = microvm explicitly."
    rationale: >-
      R2/R4: vat compose import materializes ServiceConfig entries with an
      explicit runtime it read from the project-wide --runtime flag and named
      volumes from a compose service's volumes: list; both need a vat.toml
      shape able to express them so the generated file is a normal,
      hand-editable vat.toml with no compose-only sidecar format.
    verification: >-
      AC2/AC3: a vat.toml produced by vat compose import round-trips through
      VatConfig::validate() and vat run without any config.rs error, for
      services carrying runtime = microvm and a non-empty volumes list.
no_new_top_level_keys: true
notes: >-
  vat compose introduces no new top-level vat.toml section (no [compose]
  table) and no new environment variable. All per-project compose state
  (project name, runtime selection, live vat_id, durable handoff_protocol,
  startup_pid and startup_token plus startup_started_at, service_ids, runner
  pid) lives in
  the new root/compose/project/project.json registry file (see CLI and
  Changes sections), not in vat.toml or aw.toml. The only vat.toml shape
  changes are the two additive per-service keys above (runtime = microvm,
  volumes = [ { name = ..., path = ... } ]), which are optional and backward
  compatible with every existing vat.toml. project.json is retained after
  down or terminal startup failure with status imported so the materialized
  project can be retried without re-importing.
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat compose import
    usage: "vat compose import <file> [--project <name>] [--runtime auto|docker|micro-vm]"
    new_cmd_variant: "Cmd::Compose { cmd: ComposeCmd::Import { file: PathBuf, project: Option<String>, runtime: ServiceRuntime } }"
    dispatch: "cli.rs routes Cmd::Compose{cmd} to commands::compose::exec(cmd); ComposeCmd::Import calls compose::parse then compose::expand then compose::materialize, validates the materialized service IDs, and commits the matching registry"
    behavior:
      - "file is the path to an existing, unmodified docker-compose.yml; parse canonicalizes it and vat compose never edits, lints, or rewrites it (R1/#1529)."
      - "--project defaults to the canonical compose file's parent directory basename (sanitized the same way container_name() sanitizes) when omitted."
      - "--runtime defaults to auto. Image-only services retain that project-wide value; build-backed services resolve it at import to Docker for auto/native/docker or MicroVm for microvm so future run dispatch uses the same image store that received the build (#1529)."
      - "compose::parse hard-rejects any top-level or per-service key outside the supported subset (services, volumes, version, x- prefixed keys; per-service image, build, ports, environment, depends_on, volumes) with: compose file {file} service {id} uses unsupported key {key} -- {reason}; remove it or edit the generated vat.toml directly after vat compose import (R2/R3)."
      - "For build-only services, compose::expand resolves build.context and explicit build.dockerfile from the canonical compose source directory (or default Dockerfile inside resolved context), accepts only K=V list args in declaration order or non-null map args in stable key order, uses vat-<sanitized-project>-<sanitized-service>-b3-<BLAKE3(project NUL service)>:latest, and calls resolve_image_builder() plus build_image_with_builder() in-process. The raw pair's BLAKE3 suffix, not the lossy readable prefix, prevents tag collisions. image: wins unchanged when both keys are present (#1529)."
      - "A build-bearing import preflights the selected Docker or Apple Container store before any build or materialization. Failure includes a retryable import command and leaves the previous generated vat.toml intact; image-only imports do not require a local builder (#1529)."
      - "compose::materialize writes a real vat.toml at the compose registry directory: one ServiceConfig per compose service plus one synthesized RunnerConfig (id project.up, cmd sleep 2147483647, requires every compose service id). It temp-writes, syncs, and renames, so no reader observes a partial replacement. Import then reloads service IDs and atomically publishes matching project.json; on a validation or registry-write failure it attempts to restore the previous vat.toml (or remove a fresh one). A rollback failure reports both errors and a later fresh inactive up fails closed through the registry/config gate (R1/#1529)."
      - "AC1/AC2/AC3: on success prints the written vat.toml path and exits SUCCESS; the file is immediately usable by vat run and by vat compose up."
      - "Generic import is a separate lifecycle from the Docker-shaped shim: it never mints shim provenance. An explicit inactive generic import transfers a known shim project to generic VAT lifecycle by clearing its known profile; it refuses to adopt an unknown profile, which must first receive the narrow inactive registry-only cleanup described under down. This does not widen the shim's three-profile parser."
  - name: vat compose up
    usage: "vat compose up --project <name> [--detach]"
    new_cmd_variant: "Cmd::Compose { cmd: ComposeCmd::Up { project: Option<String>, detach: bool } }"
    dispatch: "cli.rs routes to commands::compose::exec(cmd); ComposeCmd::Up runs the already-materialized registry vat.toml; source parsing, build preflight, and materialization happen only during explicit Import"
    behavior:
      - "--project is required for up; it selects one already materialized compose registry. Import is the only command that derives a project name from the source path."
      - "Generic up cannot operate a registry carrying known or unknown Docker shim provenance. A known shim project must use its matching Docker-shaped lifecycle until an explicit inactive generic re-import clears that provenance; unknown provenance fails closed."
      - "Generic vat compose up does not inherit Docker-shaped --wait semantics. Bounded wait belongs only to the known-provenance docker compose ... up -d --wait surface below and never widens generic Compose behavior."
      - "Every registry read-modify-write takes StartupClaim on the persistent startup.lock inode. It publishes <root>/compose/<project>/project.json by unique temp-file write, sync_all, and rename; no reader sees a truncated record and no concurrent up/down/ps transition can race its state change."
      - "Under that claim, load_and_validate_registry() confirms project.json belongs to --project. Only an inactive imported record with no vat_id attempts to parse the current vat.toml and compares its service-ID set to project.json without table-order sensitivity; a parseable mismatch refuses launch with re-import remediation because cleanup ownership would be incomplete. Bound or active records bypass the config gate and reconcile from VAT evidence, and an unreadable or malformed vat.toml defers to vat run's existing parse/validation failure. The gate intentionally does not require a full vat.toml digest, so direct compatible edits that retain the service-ID set remain supported. Rebuild/re-import is explicit, so up cannot silently resolve caller-relative build paths or replace a prior import (#1526/#1529)."
      - "Foreground (no --detach): constructs ComposeHandoff { project, token }, persists its token while holding the claim, and calls commands::run::exec(run::Args { target: run::Target::Runner, runner_ids: [project.up], name: Some(project), compose_handoff: Some(handoff), .. }) in-process. There is no foreground background VAT-store poll: the token owner registers, then synchronously publishes after durable VAT creation and before service startup. Persisted PID is liveness evidence only, never a stop target."
      - "Detach (--detach): constructs the same ComposeHandoff, persists a fresh startup_token plus startup_started_at and handoff_protocol: 1, re-execs vat run project.up with VAT_COMPOSE_PROJECT and VAT_COMPOSE_STARTUP_TOKEN, and persists the spawned PID before releasing the claim. The environment reconstructs that exact handoff; the child token owner waits at most ten seconds on the internal claim at vat run entry, records its own PID only when ownership matches, or aborts before VAT creation if the record belongs to a newer lifecycle. External up/down/ps claim attempts remain non-blocking."
      - "After durable VAT creation and before services start, the token owner waits at most ten seconds on the internal claim, token-matches, synchronously publishes vat_id, clears startup_pid/startup_token/startup_started_at, and retains handoff_protocol: 1; a mismatch hard-fails the run. The detached parent may reread only the token-owned project.json for quick feedback and may accept the child-published value, but it never discovers or writes vat_id from global VAT-store name/time evidence."
      - "P1 safety rationale: PID/token are transient handoff proof and clear after publication. The durable handoff_protocol: 1 marker survives publication and later reset, so a current binding with missing/malformed/unreadable VAT metadata remains EvidenceUnavailable and blocks reuse. Only protocol-absent historic JSON with a separately confirmed metadata NotFound may use the narrow legacy recovery path."
      - "A token-backed record with no launcher PID becomes terminal after a two-second handoff grace window, so a parent crash before spawn cannot wedge later up, ps, or down. A no-token record with no VAT id remains conservatively starting; protocol provenance rather than token absence controls later missing-metadata recovery."
      - "Inside the run, run_configured's prepare_service dispatches each image-backed ServiceConfig by service.runtime: MicroVm to the new prepare_microvm_service/container_run_command path, everything else unchanged to prepare_image_service/docker_run_command (R4/R5)."
      - "run_configured persists an interim RunnerRunRecord (status Running, pid Some(child.id())) into vat.meta.test_run.runner/runners immediately after spawning the runner, before the blocking wait_runner_processes call. This is readiness evidence only; compose down requests shutdown from the VAT parent that owns the runner and service tree (R9)."
      - "Foreground exit: propagates the run's ExitCode without claiming a legacy started status; ps or down reconciles retained evidence. It resets only after Status::Exited, every tracked service is terminal, and cleanup_error is absent."
      - "Detach exit: prints status starting until VAT evidence has every service Ready plus a live project.up runner pid; only then prints ready. Runner-exited or terminal-service evidence while VAT remains Status::Running is stopping, not terminal, and retains the binding. A child that fails before writing VAT evidence, or fully terminal evidence without cleanup_error, resets the record to imported and returns an actionable error. Unconfirmed Docker or MicroVM cleanup forces nonzero VAT retention and retains the binding for retry."
  - name: vat compose down
    usage: "vat compose down <project>"
    new_cmd_variant: "Cmd::Compose { cmd: ComposeCmd::Down { project: String } }"
    dispatch: "cli.rs routes to commands::compose::exec(cmd); reads the ComposeRecord for project from the registry"
    behavior:
      - "Takes and holds StartupClaim from registry read through reconciliation, stop acknowledgement, cleanup confirmation, and reset. Concurrent up is rejected while down waits, so a new service set cannot bind the old run's published ports."
      - "Generic down cannot clean a known Docker shim project. For unknown shim provenance it is only a registry-only escape hatch when the record is inactive (status imported and no vat_id): remove project.json, preserve the materialized vat.toml, and do not touch runtime state. Unknown active provenance fails closed and requires matching or newer VAT that recognizes the profile, or the matching Docker shim."
      - "Hard-errors if the registry has no ComposeRecord or only imported metadata. A starting record remains retryable and retains its binding. A stopping record means runner-exited or terminal-service evidence while VAT is still Status::Running, so down continues acknowledgement without resetting. For handoff_protocol: 1, every VAT load/read/malformed/missing error remains retained EvidenceUnavailable and never terminal/resettable. Only a protocol-absent historic record with a separate metadata NotFound may use legacy recovery; otherwise only Status::Exited plus every tracked service terminal and no cleanup_error may reset the active fields (#1526)."
      - "For CleanupUnconfirmed, loads the retained VAT and calls retry_unconfirmed_service_cleanup. It retries only the persisted Docker or MicroVM resource name. A nonzero rm -f is successful only if a bounded exact-name list query succeeds and proves absence: Docker container ls -a uses an anchored name filter and exact line comparison; MicroVM parses container list --all --format json with no matching id. Query failure, timeout, malformed output, or a match leaves cleanup_error intact. Only a confirmed retry clears the terminal binding; failures retain the record and return inspect/state plus retry-down remediation. Binding release is impossible before cleanup is confirmed."
      - "For Ready, writes the VAT directory's .compose-stop-request. It never directly signals a persisted OS PID. The live VAT parent consumes that request, kills its own runner tree, runs stop_services, persists Status::Exited, and marks an already-reaped owned child Exited."
      - "Waits bounded for Status::Exited, every compose service terminal, and no cleanup_error. A timeout or remaining teardown keeps status stopping and retains the registry. Only after that acknowledgement clears vat_id, startup_pid, startup_token, and startup_started_at while preserving handoff_protocol: 1 and project/service import metadata with status imported, then exits SUCCESS (AC5)."
  - name: vat compose ps
    usage: "vat compose ps <project>"
    new_cmd_variant: "Cmd::Compose { cmd: ComposeCmd::Ps { project: String } }"
    dispatch: "cli.rs routes to commands::compose::exec(cmd)"
    behavior:
      - "Generic ps rejects any Docker shim provenance rather than inspecting a project owned by a known or unknown Docker-shaped profile."
      - "Generic ps never manufactures the Docker-shaped topology JSON. That final additive profile/topology result belongs only to the exact known-provenance `docker compose -p PROJECT ps` path described below."
      - "Reconciles ComposeRecord state first: imported tells the user to run up, starting prints starting, stopping prints stopping and retains the binding, and handoff_protocol: 1 load/read/malformed/missing evidence returns an actionable EvidenceUnavailable retry without resetting. Only protocol-absent historic JSON with a separately confirmed metadata NotFound may use compatibility recovery; otherwise only fully terminal startup evidence (Status::Exited, terminal services, no cleanup_error) resets the record to imported while returning an actionable nonzero error."
      - "For a ready record, reads vat_id and then store::load(vat_id) for that run's test_run.services."
      - "Filters test_run.services down to service_ids membership (R10), reusing commands/logs.rs's existing linear-scan-by-id approach."
      - "Prints the filtered per-service id/runtime/status/port table and exits SUCCESS."
  - name: vat compose logs
    usage: "vat compose logs <project> <service>"
    new_cmd_variant: "Cmd::Compose { cmd: ComposeCmd::Logs { project: String, service: String } }"
    dispatch: "cli.rs routes to commands::compose::exec(cmd)"
    behavior:
      - "Generic logs rejects any Docker shim provenance rather than inspecting a project owned by a known or unknown Docker-shaped profile."
      - "Reads the ComposeRecord for project, then store::load(vat_id); locates service among the service_ids-filtered set (R10)."
      - "Hard-errors with service not found in this compose project's service_ids when the named service is absent from that filtered set."
      - "On success, prints that service's captured stdout/stderr the same way commands/logs.rs's existing per-source branch does, and exits SUCCESS."
  - name: docker compose -f FILE -p PROJECT up -d --wait [--wait-timeout SECONDS]
    usage: "docker compose -f <file> -p <project> up -d --wait [--wait-timeout <positive-seconds>]"
    dispatch: "the Docker-shaped multicall shim validates and imports its captured profile, completes any source build, then starts the deadline immediately before the detached typed Compose launch and calls the target-pinned durable VAT observer"
    behavior:
      - "Requires explicit -d/--detach even when waiting, accepts --wait at most once, and accepts --wait-timeout only with wait. Timeout is a positive whole number of seconds, defaults to 300, and cannot exceed 1200."
      - "The deadline starts only after validated import and any source build complete; it covers detached runner handoff and subsequent observations. It waits for durable VAT runner readiness/topology proof only, not Docker healthcheck, application HTTP, service DNS, or generic Docker Compose readiness."
      - "The target captures exact known profile, launch generation, and ticket. Each observation validates that triple under a claim, then releases the claim before the next poll, so down, generic re-import, or relaunch cannot be blocked by or accidentally satisfy an older waiter."
      - "On ready, emits one final vat_docker_compose up JSON with wait { requested=true, timeout_seconds, outcome=ready } and the same ready topology contract as Docker-shaped ps. A source-build cleanup_next is included only on this verified-ready final result."
      - "On timeout, retains runtime and registry. A safe ps next exists only if a current pinned target was observed; terminal evidence, a replaced target, and a deadline reached before any current observation are terminal without unsafe next. Degraded wait has no endpoint."
      - "On this host, the opt-in gated real Apple Container command RUST_TEST_THREADS=1 VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_compose_host_facing_independent_profile_contract -- --ignored --nocapture passed 1/1 (50 filtered) in 4.54 seconds. It proves this host-facing-independent-v1 up -d --wait path, both loopback endpoints, one-document JSON ps/logs/exec, text logs, text exec including a no-final-newline handoff, and exact down cleanup of containers, ports, and registry only—not service-name DNS, general Compose, Docker Engine API, or Kubernetes."
  - name: docker compose -p PROJECT ps
    usage: "docker compose -p <project> ps"
    dispatch: "the Docker-shaped multicall shim calls commands::compose::docker_shim_ps while its Compose registry claim is still held; it never reopens a later registry revision to construct public JSON"
    behavior:
      - "Accepts only the exact no-argument shape; Docker --format and every other ps flag fail before observation."
      - "Requires known shim provenance. Generic, missing, and unknown provenance fail closed before text or topology output; generic vat compose ps remains the separate text-only lifecycle path above."
      - "Preserves the human-readable text and ends with additive vat_docker_compose JSON retaining profile plus topology { phase, ready, services }. Phase is inactive, starting, ready, degraded, or stopping; services use the registered Compose service-ID order rather than persisted evidence order."
      - "Endpoint is only canonical 127.0.0.1:<port>. VAT emits endpoints for every service only when all expected service IDs have one unique Ready VAT-owned container_run record for the exact expected MicroVM, a loopback nonzero port, and no cleanup error."
      - "Any incomplete or unsafe proof turns nominal ready into degraded with ready=false and no endpoints. Inactive, starting, and stopping also expose no endpoints. This is lifecycle/ownership evidence, not application-healthcheck behavior."
  - name: "vat run / vat build / vat cluster (unaffected)"
    usage: "(no change)"
    behavior:
      - "Not modified by this WI beyond the additive prepare_service dispatch check and the early RunnerRunRecord persist described above: no change to Isolation::MicroVm's resolve() argv shape (Phase 1) or vat build's container_build_command/build_image (Phase 2); vat cluster's own registry and run_capture poll pattern are read as precedent, not touched."
      - "Out of scope for this WI: bridge-network DNS simulation for depends_on (a printed warning only), registry push/pull, and any change to the default --runtime auto Docker-only behavior."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-microvm-phase-3-vat-compose-limited-compose-subset-up-down-p-verification
requirements:
  container_run_command_argv_shape:
    id: R5
    text: "container_run_command builds the exact argv order: [\"container\",\"run\",\"--rm\",\"--name\",<name>,\"-p\",\"127.0.0.1:<host>:<container>\", (\"-v\",\"<name>:<path>\") per volumes entry, (\"-e\",\"<K>=<V>\") per sorted env entry, <image>] -- deterministic ordering matching docker_run_command's guarantee."
    kind: functional
    risk: high
    verify: commands::run::tests::container_run_command_shape
  depends_on_prints_no_bridge_dns_warning:
    id: R3
    text: "Any service whose depends_on is non-empty after expansion triggers a printed, non-fatal warning that vat compose does not simulate container-to-container bridge-network DNS."
    kind: functional
    risk: medium
    verify: compose::tests::expand_depends_on_prints_no_bridge_dns_warning
  registry_claim_and_compose_handoff:
    id: R9
    text: "Foreground and detached compose up operations serialize on the persistent startup.lock claim; exactly one fresh token owner may register and synchronously publish a VAT after durable creation, while parents only reread the token-owned registry. Internal parent/child claim reacquisition waits at most ten seconds; external lifecycle commands remain non-blocking."
    kind: functional
    risk: high
    verify: apps/vat/tests/vat_compose.rs::test_compose_detached_up_uses_atomic_claim_and_creates_one_run
  durable_handoff_protocol_prevents_current_legacy_misclassification:
    id: R9
    text: "P1 safety rationale: PID and startup token clear after publish, but handoff_protocol: 1 persists. A current record with missing, malformed, or unreadable VAT metadata is EvidenceUnavailable and retains its binding. Only protocol-absent historic JSON plus a separate metadata NotFound is recoverable, preventing a missing current metadata file from freeing a possibly live published port."
    kind: regression
    risk: high
    verify: apps/vat/tests/vat_compose.rs
  detached_handoff_expires_without_launcher_pid:
    id: R9
    text: "A token-backed starting record with neither VAT evidence nor a launcher PID becomes reclaimable after the two-second grace period; it cannot wedge a later compose up."
    kind: regression
    risk: high
    verify: apps/vat/tests/vat_compose.rs::test_compose_up_reclaims_expired_token_without_launcher_pid
  down_requests_parent_owned_shutdown_and_holds_claim:
    id: R9
    text: "compose down writes the VAT-owned stop request and retains the registry claim until the VAT parent persists terminal service cleanup; another up is rejected during that acknowledgement window, and no persisted PID is used as a direct signal target."
    kind: regression
    risk: high
    verify: apps/vat/tests/vat_compose.rs::test_compose_up_is_rejected_while_down_holds_lifecycle_claim
  cleanup_unconfirmed_retains_binding_until_retry:
    id: R9
    text: "A Docker or MicroVM service with cleanup_error is CleanupUnconfirmed rather than releasable; compose retains its binding until retry_unconfirmed_service_cleanup clears the persisted error. A nonzero rm -f counts only after a successful bounded exact-name list proves absence; any query error, timeout, malformed output, or match remains retained."
    kind: regression
    risk: high
    verify: commands::compose::tests::cleanup_unconfirmed_blocks_compose_reuse_until_retry_succeeds
  already_reaped_owned_service_is_terminal:
    id: R9
    text: "When an owned service that was recorded Ready exits naturally before down acknowledgement, VAT clears its PID and persists Exited so compose can observe terminal cleanup instead of waiting forever on stale Ready evidence."
    kind: regression
    risk: high
    verify: apps/vat/tests/vat_compose.rs::test_compose_down_marks_already_exited_ready_service_terminal
  environment_bare_key_hard_reject:
    id: R2
    text: "compose::parse hard-rejects an environment entry with a bare key and no value (list form with no \"=\", or map form with a null value) rather than silently inheriting the caller's shell environment."
    kind: functional
    risk: high
    verify: compose::tests::parse_rejects_bare_environment_key
  expand_bare_ports_become_auto:
    id: R2
    text: "compose::expand maps a bare \"C\" ports entry to PortSpec::Auto with container_port C, and an \"H:C\" entry to PortSpec::Fixed(H) with container_port C."
    kind: functional
    risk: high
    verify: compose::tests::expand_ports_bare_container_form_becomes_auto
  expand_build_resolves_via_runtime_local_builder:
    id: "#1529"
    text: "compose::expand preflights the runtime-selected image store once for a build-bearing import, then resolves build.context/build.dockerfile from the canonical compose source directory, forwards list-ordered or map-key-stable build.args, calls build_image_with_builder() in-process, and writes an OCI-safe project-scoped tag with a BLAKE3 raw project/NUL/service identity suffix into ServiceConfig.image. image: wins unchanged; built services persist a concrete Docker or MicroVm runtime."
    kind: functional
    risk: high
    verify: cargo test -p vat --test vat_compose_build -- --nocapture
  expand_depends_on_maps_to_requires:
    id: R2
    text: "compose::expand maps a depends_on list or map form 1:1 onto ServiceConfig.requires, preserving compose-declared order."
    kind: functional
    risk: medium
    verify: compose::tests::expand_derives_requires_from_depends_on
  materialize_writes_synthesized_runner:
    id: R1
    text: "compose::materialize writes a vat.toml with one [[services]] per compose service plus one synthesized [[runners]] (id \"project.up\", cmd [\"sleep\",\"2147483647\"], requires = every compose service id in expand()'s order), satisfying VatConfig::validate()'s existing at-least-one-runner requirement unmodified."
    kind: functional
    risk: high
    verify: compose::tests::materialize_writes_synthesized_runner_requires_all_services
  compose_build_paths_args_and_atomic_materialization:
    id: "#1529"
    text: "A build-bearing import resolves relative context and explicit dockerfile paths from the canonical compose source (not the caller cwd), accepts only explicit build.args values with deterministic list/map order, derives collision-safe readable tags from the raw project/service pair, and preflights/builds before materialization. materialize temp-writes, syncs, and renames so failure retains the previous vat.toml."
    kind: regression
    risk: high
    verify: cargo test -p vat --test vat_compose_build -- --nocapture
  compose_up_validates_registry_service_ownership:
    id: "#1529"
    text: "Before starting a fresh inactive imported project, up rejects a parseable vat.toml whose service-ID set differs from project.json, before either runtime is launched; service-table order is irrelevant. A user-edited vat.toml with the same service-ID set is accepted because the contract intentionally does not compare a full config digest. Bound or active records bypass this gate so their VAT-evidence reconciliation and cleanup cannot be blocked, while malformed configs defer to vat run's existing parse failure."
    kind: regression
    risk: high
    verify: cargo test -p vat --test vat_compose_build -- --nocapture
  parse_ignores_x_extension_keys:
    id: R2
    text: "compose::parse silently ignores any top-level or per-service key prefixed x- (compose's own extension convention), never treating it as unsupported."
    kind: functional
    risk: low
    verify: compose::tests::parse_accepts_x_extension_keys
  parse_rejects_unsupported_keys_exact_format:
    id: R3
    text: "compose::parse hard-rejects deploy/secrets/configs/extends/networks/profiles/healthcheck/command/entrypoint-override/bind-mount-form volumes with the exact error: compose file `{file}` service `{id}` uses unsupported key `{key}` -- {reason}; remove it or edit the generated vat.toml directly after `vat compose import`."
    kind: functional
    risk: high
    verify: compose::tests::parse_rejects_deploy_key
  prepare_service_dispatches_on_runtime:
    id: R4
    text: "prepare_service routes an image-backed ServiceConfig to prepare_microvm_service only when service.runtime is MicroVm; every other runtime value (Auto, Docker, Native) is routed to prepare_image_service unchanged, so today's default behavior is bit-for-bit preserved."
    kind: regression
    risk: high
    verify: commands::run::tests::prepare_service_dispatches_microvm_runtime_only
  ps_logs_filter_by_service_ids:
    id: R10
    text: "compose ps and compose logs both filter a loaded run's flat test_run.services by the ComposeRecord's service_ids membership before printing/locating, reusing commands/logs.rs's existing linear-scan-by-id approach; a service outside that set is treated as not found."
    kind: functional
    risk: medium
    verify: commands::compose::tests::ps_and_logs_filter_by_service_ids
  runner_early_persist_writes_live_pid:
    id: R9
    text: "run_configured writes an interim RunnerRunRecord (status Running, pid Some(child.id())) into vat.meta.test_run.runner/runners immediately after spawning the runner process, strictly before the blocking wait_runner_processes call returns. The PID is readiness evidence for reconciliation, not compose-down signal authority."
    kind: functional
    risk: high
    verify: commands::run::tests::run_configured_persists_runner_pid_before_wait
  runner_run_record_pid_field_roundtrip:
    id: R7
    text: "RunnerRunRecord serializes and deserializes an optional pid: Option<u32> field; legacy metadata without the field deserializes with pid: None (backward compatible)."
    kind: regression
    risk: medium
    verify: state::tests::runner_run_record_pid_field_roundtrip
  service_runtime_microvm_variant_validate:
    id: R4
    text: "ServiceConfig::validate() accepts an image-backed service declaring runtime: docker or runtime: microvm explicitly (relaxed from requiring a preset), while a cmd-only service with a non-auto runtime still bails."
    kind: functional
    risk: high
    verify: config::tests::validate_allows_image_service_explicit_runtime
  stop_services_removes_microvm_container:
    id: R5
    text: "stop_services() force-removes a MicroVm-backed service's container via `container rm -f <name>`; an already-reaped VAT-owned Ready child becomes Exited, while a failed removal persists cleanup_error so compose retains the binding for retry."
    kind: regression
    risk: medium
    verify: commands::run::tests::stop_services_removes_microvm_container
---
flowchart TD
    r1[R1 materialize writes synthesized runner] --> compose_tests_materialize_writes_synthesized_runner_requires_all_services[compose::tests::materialize_writes_synthesized_runner_requires_all_services]
    r2[R2 environment bare key hard reject] --> compose_tests_parse_rejects_bare_environment_key[compose::tests::parse_rejects_bare_environment_key]
    r2[R2 expand bare ports become auto] --> compose_tests_expand_ports_bare_container_form_becomes_auto[compose::tests::expand_ports_bare_container_form_becomes_auto]
    r2[R2 expand depends on maps to requires] --> compose_tests_expand_derives_requires_from_depends_on[compose::tests::expand_derives_requires_from_depends_on]
    r2[R2 parse ignores x extension keys] --> compose_tests_parse_accepts_x_extension_keys[compose::tests::parse_accepts_x_extension_keys]
    r3[R3 depends on prints no bridge dns warning] --> compose_tests_expand_depends_on_prints_no_bridge_dns_warning[compose::tests::expand_depends_on_prints_no_bridge_dns_warning]
    r3[R3 parse rejects unsupported keys exact format] --> compose_tests_parse_rejects_deploy_key[compose::tests::parse_rejects_deploy_key]
    r4[R4 prepare service dispatches on runtime] --> commands_run_tests_prepare_service_dispatches_microvm_runtime_only[commands::run::tests::prepare_service_dispatches_microvm_runtime_only]
    r4[R4 service runtime microvm variant validate] --> config_tests_validate_allows_image_service_explicit_runtime[config::tests::validate_allows_image_service_explicit_runtime]
    r5[R5 container run command argv shape] --> commands_run_tests_container_run_command_shape[commands::run::tests::container_run_command_shape]
    r5[R5 stop services removes microvm container] --> commands_run_tests_stop_services_removes_microvm_container[commands::run::tests::stop_services_removes_microvm_container]
    r1529[#1529 runtime-local compose build] --> vat_compose_build[cargo test -p vat --test vat_compose_build -- --nocapture]
    r7[R7 runner run record pid field roundtrip] --> state_tests_runner_run_record_pid_field_roundtrip[state::tests::runner_run_record_pid_field_roundtrip]
    r9[R9 registry claim and child handoff] --> vat_compose_atomic_claim[apps/vat/tests/vat_compose.rs::test_compose_detached_up_uses_atomic_claim_and_creates_one_run]
    r9[R9 no pid handoff expiry] --> vat_compose_expired_handoff[apps/vat/tests/vat_compose.rs::test_compose_up_reclaims_expired_token_without_launcher_pid]
    r9[R9 down parent acknowledgement holds claim] --> vat_compose_down_claim[apps/vat/tests/vat_compose.rs::test_compose_up_is_rejected_while_down_holds_lifecycle_claim]
    r9[R9 cleanup unconfirmed retains binding] --> commands_compose_tests_cleanup_unconfirmed[commands::compose::tests::cleanup_unconfirmed_blocks_compose_reuse_until_retry_succeeds]
    r9[R9 already reaped owned service terminal] --> vat_compose_reaped_service[apps/vat/tests/vat_compose.rs::test_compose_down_marks_already_exited_ready_service_terminal]
    r9[R9 runner early persist writes live pid] --> commands_run_tests_run_configured_persists_runner_pid_before_wait[commands::run::tests::run_configured_persists_runner_pid_before_wait]
    r10[R10 ps logs filter by service ids] --> commands_compose_tests_ps_and_logs_filter_by_service_ids[commands::compose::tests::ps_and_logs_filter_by_service_ids]
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-compose-import-pure-fixture-shape
    name: "pure fixture: vat compose import expands services + runner + hard-reject keys with no container/docker required"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: vat-compose-bounded-compose-subset-up-down-ps-logs
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_compose_import -- --nocapture"
    assertions:
      - "AC2: image-only expansion-shape assertions over a fixture compose file -- one ServiceConfig per compose service, the synthesized project.up runner with requires listing every service id in expand()'s order, environment injected onto ServiceConfig.image_env, and ports mapped per R2's H:C / bare C rules. These image-only cases run with no container/docker binary on PATH."
      - "R3: one assertion per hard-reject key (deploy, secrets, configs, extends, networks, profiles, healthcheck, command/entrypoint override, bind-mount-form volumes) asserting compose::parse returns the exact error text naming file/service/key."
      - "AC7: feeding a fixture containing deploy: or healthcheck: into vat compose import fails with an error naming the exact file, service, and key."
  - id: vat-compose-container-gated-full-cycle
    name: "container-gated: up -d / ps / logs / down full lifecycle against one image: and one build: service"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: vat-compose-bounded-compose-subset-up-down-ps-logs
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_compose -- --nocapture"
    assertions:
      - "AC5: gated on a container_available() skip helper (mirroring vat_cluster.rs's Docker-gated pattern and vat_sandbox_microvm.rs's container-gated tests): compose up -d against a fixture with one image: service and one build: service, then compose ps reports starting or ready truthfully, compose logs <project> <service> returns non-empty captured output for each, and compose down terminates the backing runner/service processes while retaining project.json as imported metadata ready for retry."
      - "R9: foreground and detached up share one project/token ComposeHandoff; only the token owner publishes the durable VAT id and the parent never performs global VAT-store name/time discovery."
      - "R9: down writes .compose-stop-request and waits for the VAT parent to persist terminal runner/service cleanup before resetting project.json. Runner exit while VAT remains Running projects stopping and retains the binding. Current handoff_protocol: 1 VAT load/read/malformed/missing failure is EvidenceUnavailable, which retains the binding and requests retry rather than terminal reset; only protocol-absent historic JSON plus metadata NotFound may recover. A concurrent up is rejected during that window; runner PID evidence is never used as a direct signal target."
      - "R9: Docker or MicroVM cleanup_error retains the VAT, project binding, and published-port ownership and forces nonzero lifecycle retention. A later down retries only the persisted runtime resource; a failed rm -f releases only after successful bounded exact-name list proof of absence (Docker anchored name filter/exact line, MicroVM parsed JSON/no id)."
      - "Registered in apps/vat/tests/aw-ec.toml (R11) alongside vat_compose_import.rs's pure test, so aw ec gen --verify / aw health --verify-tests pick both up as configured EC-gated test commands for the agent-native-gpu-native-dev-containers capability."
  - id: vat-compose-runtime-local-build-artifacts
    name: "runtime-local compose build: canonical context/dockerfile/args, image-store mapping, and failure-safe materialization"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: compose-runtime-local-build-artifacts
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_compose_build -- --nocapture"
    assertions:
      - "#1529: a build-only service resolves short/full build paths relative to the canonical compose source, not the invoking cwd; explicit Dockerfile and deterministically ordered build.args reach the selected builder; generated tags use an OCI-safe readable project/service prefix plus a BLAKE3 raw-pair identity suffix, so normalization or delimiter ambiguity cannot collide."
      - "#1529: auto/native/docker select Docker and MicroVm selects Apple Container; a preflight/build failure occurs before generated vat.toml replacement, preserving a prior materialized import. Image-only compose files remain builder-independent."
      - "#1529: a fresh inactive imported compose up refuses a parseable registry/config service-ID-set mismatch before Docker or Apple Container starts, ignoring service-table order; it accepts a user-edited valid vat.toml when its identity set still matches project.json. Bound or active records bypass this gate for VAT-evidence cleanup, and malformed configs retain vat run's existing parse failure; no full config digest blocks compatible local edits."
  - id: vat-compose-mainstream-manual-smoke
    name: "manual smoke: a real unmodified mainstream docker-compose.yml round-trips import -> up -d -> ps -> logs -> down"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: vat-compose-bounded-compose-subset-up-down-ps-logs
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "vat compose import ./docker-compose.yml && vat compose up -d --project demo && vat compose ps demo && vat compose logs demo web && vat compose down demo"
    assertions:
      - "AC6: a real, unmodified mainstream docker-compose.yml (one image: service, one build: service, a depends_on entry) succeeds through the full import -> up -d -> ps -> logs -> down cycle, retaining an imported registry ready for another up, and the source compose file itself required no edits."
      - "Not part of the cargo test / aw-ec.toml gated surface (no CI-portable fixture repo is bundled for this manual smoke); recorded here as the human verification step named by AC6, run once against a developer-supplied compose file during this WI's own close-out."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/src/compose.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: "R1-R3/R6: new parse()/expand()/materialize() -- a real YAML compose-subset parser plus a supported-vs-hard-reject key walk that produces the exact per-key error text, a build:-to-image() in-process resolution call, and a depends_on no-bridge-DNS warning. #1529 adds canonical source-relative build.context/build.dockerfile resolution, supported build.args parsing, OCI-safe project-scoped tags with a BLAKE3 raw-pair identity suffix, selected runtime image-store preflight, concrete runtime persistence for built services, and temp-write/sync/rename vat.toml materialization. restore_materialized_config() lets commands::compose roll back a published replacement if its matching registry record cannot be committed. No existing generated module has this parse/validate/expand shape, so the whole file is hand-authored this WI (missing-generator:logic:compose-subset-parser, tracker #1484)."
  - path: apps/vat/src/commands/build.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "#1529: ImageBuilder maps Auto/Native/Docker to Docker and MicroVm to Apple Container, then preflights the exact local store before compose can materialize. build_image_with_builder() preserves captured import output and Docker's deterministic argv/probe mirrors the existing Container path, preventing build/run image-store drift."
  - path: apps/vat/src/commands/compose.rs
    action: create
    section: cli
    impl_mode: hand-written
    reason: "R8-R10/#1526: Cmd dispatch for import/up/down/ps/logs; ComposeRecord registry transitions at <root>/compose/<project>/project.json guarded by a persistent advisory claim and atomic temp-write/sync/rename; one foreground/detached ComposeHandoff with token-owner publication, bounded ten-second internal claim reacquisition, no name/time VAT-store polling, and durable handoff_protocol: 1 provenance after transient PID/token clear; current VAT read/load/malformed/missing evidence retains as EvidenceUnavailable while only protocol-absent historic JSON plus metadata NotFound can recover; and down's VAT-parent stop-request acknowledgement. #1529 commits the registry only after parsing the materialized service IDs and attempts vat.toml rollback on a later validation or registry-write failure; a rollback failure is reported and the registry/config gate refuses a later fresh inactive up. For that inactive state, up compares a parseable service-ID set without table-order sensitivity and without a full config digest; active/bound records reconcile from VAT evidence without config gating, and malformed configs defer to vat run's parse failure. #1526 retains cleanup-unconfirmed Docker or MicroVM bindings until bounded exact-name list proof confirms absence, preventing published-port reuse. This process-orchestration shape (in-process call vs. self-re-exec plus token-matched child publication vs. parent-owned teardown acknowledgement) is genuinely new -- no existing vat command proxies a long-running run in these lifecycle modes -- so the whole file is hand-authored this WI (missing-generator:cli:compose-lifecycle-orchestration, tracker #1484), the same class of gap Phase 2's commands/build.rs recorded for its own dual-mode divergence (missing-generator:cli:streamed-subprocess-dual-mode, tracker #1479)."
  - path: apps/vat/src/commands/mod.rs
    action: modify
    section: cli
    impl_mode: codegen
    reason: "R1: add `pub mod compose;` -- mechanical module registration, no logic, consistent with this file's existing codegen ownership."
  - path: apps/vat/src/cli.rs
    action: modify
    section: cli
    impl_mode: codegen
    reason: "R8: add `Cmd::Compose { cmd: ComposeCmd }` and `enum ComposeCmd { Import, Up, Down, Ps, Logs }` plus dispatch to `commands::compose::exec`; mechanical clap variant + dispatch addition, consistent with this file's existing codegen ownership (mirrors Phase 1's `--microvm-image` flag and Phase 2's `Cmd::Build` additions)."
  - path: apps/vat/src/config.rs
    action: modify
    section: schema
    impl_mode: codegen
    reason: "R4: additive `ServiceRuntime::MicroVm` variant (mirrors the existing `Docker` variant, same clap::ValueEnum derive), additive `ServiceConfig.volumes: Vec<VolumeMount>` field with the new `VolumeMount { name, path }` struct (mirrors existing optional Vec fields), and a widened boolean condition in `validate()` (adds `|| has_image` to the existing preset-required gate) -- pure data-model addition plus a mechanical condition widening, no new control flow shape."
  - path: apps/vat/src/commands/run.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "R4/R5/R9/#1526: new `prepare_microvm_service()`/`container_run_command()`/`ensure_microvm_available()` structurally mirror the existing `prepare_image_service`/`docker_run_command`/`ensure_docker_available` trio, and the `ServicePlan`/`ServiceHandle.microvm_name` fields plus the `stop_services()` teardown branch mirror the existing `docker_name` fields/branch. The hand-written control-flow additions are `prepare_service` runtime dispatch, `run_configured` early runner evidence plus parent-owned stop-request consumption, terminalization of already-reaped owned children, and durable runtime-generic `cleanup_error` plus retry_unconfirmed_service_cleanup. A failed rm is accepted only after a successful bounded exact-name list proves absence; query errors, timeouts, malformed output, or a match retain runner/scenario evidence nonzero. Runner PID is reconciliation evidence only, never a compose-down kill target. Hand-authored this WI (missing-generator:logic:runner-early-persist-and-runtime-dispatch, tracker #1484)."
  - path: apps/vat/src/state.rs
    action: modify
    section: schema
    impl_mode: codegen
    reason: "R7: additive `RunnerRunRecord.pid: Option<u32>` field (mirrors the existing `ServiceRunRecord.pid` field verbatim, same optional/skip_serializing_if shape) -- pure data-model addition, no control flow."
  - path: apps/vat/tests/vat_compose_import.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "AC2/AC7: pure fixture-based expansion-shape assertions and one assertion per R3 hard-reject key, requiring no container/docker binary -- new test file, hand-authored per this project's e2e-test convention (mirrors vat_build.rs's split between a pure and a gated test file)."
  - path: apps/vat/tests/vat_compose.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "AC5: gated full up -d / ps / logs / down cycle test against a real container/docker backend, using a `container_available()` skip helper mirroring `vat_cluster.rs`'s Docker-gated pattern and `vat_sandbox_microvm.rs`'s container-gated tests -- new test file, hand-authored per this project's e2e-test convention."
  - path: apps/vat/tests/aw-ec.toml
    action: modify
    section: e2e-test
    impl_mode: hand-written
    reason: "R11: register `vat_compose_import.rs` and `vat_compose.rs`'s EC-gated test command(s) as configured test commands for the agent-native-gpu-native-dev-containers capability, so `aw ec gen --verify` / `aw health --verify-tests` pick them up (mirrors Phase 2's `vat_build.rs` registration)."
```
