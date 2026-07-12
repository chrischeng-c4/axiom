---
id: vat-microvm-phase-3-vat-compose-limited-compose-subset-up-down-p
summary: (fill)
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-compose-phase3-exec-logic
entry: start
nodes:
  start: { kind: start, label: "vat compose <subcommand> invoked: import|up|down|ps|logs" }
  route: { kind: decision, label: "which subcommand" }
  parse: { kind: process, label: "compose::parse reads the compose YAML via serde_yaml into ComposeFile R1; unsupported top-level keys other than services volumes version x- are rejected the same way as per-service unsupported keys below" }
  key_check: { kind: decision, label: "every top-level and per-service key is in the supported subset R2 or an ignored x- key" }
  reject: { kind: terminal, label: "hard Err naming file plus service plus key: compose file file service id uses unsupported key key reason remove it or edit the generated vat.toml directly after vat compose import R3; deploy secrets configs extends networks profiles healthcheck command entrypoint override bind mount volumes all land here" }
  expand_build: { kind: process, label: "expand resolves build keys short string or full mapping form by calling commands::build::build_image in process R6 and writes the returned tag back into that service ServiceConfig.image so image and build entries produce one uniform image shaped ServiceConfig" }
  expand_fields: { kind: process, label: "expand maps ports H:C and bare C forms, environment list or map a bare key with no value is a hard reject, depends_on list or map condition service_healthy is a hard reject else mapped 1:1 onto ServiceConfig.requires, and named volume volumes entries onto ServiceConfig.volumes" }
  depends_warn: { kind: process, label: "any service whose depends_on is non empty after expansion gets a printed non fatal warning: vat compose does not simulate container to container bridge network DNS; depends_on only orders startup use VAT_SERVICE_ID_HOST PORT instead R3" }
  runtime_assign: { kind: process, label: "expand sets every service ServiceConfig.runtime from the project wide --runtime flag auto docker microvm default auto the same config::ServiceRuntime enum vat.toml already uses extended with MicroVm R4 R8; auto is written through unchanged preserving today Docker routing behavior" }
  materialize: { kind: process, label: "materialize writes a real vat.toml one services entry per compose service plus one synthesized runners entry id project.up cmd sleep infinity requires all compose service ids so vat run existing at least one runner requirement is satisfied unmodified R1" }
  import_ok: { kind: terminal, label: "vat.toml written at the project directory ExitCode SUCCESS AC1 AC2 AC3 ready for vat compose up" }
  up_expand: { kind: process, label: "up reuses the same parse expand materialize path as import writing or overwriting vat.toml then constructs a ComposeRecord project vat_id None service_ids status starting and writes it to root compose project project.json before any blocking call R9 R10" }
  up_mode: { kind: decision, label: "--detach flag" }
  up_fg_poll: { kind: process, label: "foreground spawn a background thread polling store::list every 200ms mirrors cluster::run_capture poll pattern looking for a Vat whose meta.name equals project to fill the registry vat_id once created" }
  up_fg_call: { kind: process, label: "foreground call commands::run::exec run::Args target run::Target::Runner runner_ids project.up name Some project in process not a subprocess this call blocks until the run teardown completes" }
  runtime_route: { kind: decision, label: "inside that run run_configured prepare_service dispatches each image backed ServiceConfig on its ServiceRuntime R4 R5" }
  docker_path: { kind: process, label: "ServiceRuntime Auto or Docker unchanged prepare_image_service plus docker_run_command path existing untouched" }
  microvm_path: { kind: process, label: "ServiceRuntime MicroVm new prepare_microvm_service plus container_run_command path structurally mirroring the Docker pair R5" }
  runner_persist: { kind: process, label: "run_configured persists an interim RunnerRunRecord status Running pid Some child.id into vat.meta.test_run.runner runners immediately after spawning the runner process before the blocking wait_runner_processes call necessary so down SIGTERM path R9 can read a live pid while the runner is still executing existing code only wrote a real RunnerRunRecord after wait returned" }
  up_fg_result: { kind: decision, label: "commands::run::exec result once the run and its teardown complete" }
  up_fg_err: { kind: terminal, label: "propagate the non zero ExitCode the registry entry remains down is still available for cleanup" }
  up_fg_ok: { kind: terminal, label: "ExitCode SUCCESS once the foreground run and its full stop_services teardown tail completes" }
  up_bg: { kind: process, label: "detached after the same parse expand materialize registry write re exec self via Command::new current_exe args run project.up --name project spawn not the in process call above since the caller must return before the run ends" }
  up_bg_poll: { kind: process, label: "poll store::list for up to about 10s looking for a Vat whose meta.name equals project to fill the registry vat_id" }
  up_bg_result: { kind: decision, label: "vat_id observed within the poll window" }
  up_bg_timeout: { kind: terminal, label: "Err vat_id not observed within the poll window the spawned process may still be starting registry entry left in status starting for a retry or vat compose down" }
  up_bg_ok: { kind: terminal, label: "print project vat_id status started ExitCode SUCCESS the detached re exec'd process now owns the same runtime_route runner_persist path above once it reaches run_configured" }
  down_read: { kind: process, label: "down reads the ComposeRecord for project from the registry" }
  down_pid_check: { kind: decision, label: "that vat_id test_run.runner.pid R7 is present" }
  down_pid_missing: { kind: terminal, label: "hard Err runner has no recorded pid never started or already exited registry entry left for inspection" }
  down_signal: { kind: process, label: "libc::kill pid as i32 SIGTERM sent directly to that leaf child process not the outer vat process R9" }
  down_wait: { kind: process, label: "the still running vat run process blocking child.wait inside wait_runner_processes returns through normal control flow the existing stop_services teardown tail docker rm -f container rm -f etc runs completely unmodified" }
  down_remove: { kind: process, label: "remove the root compose project project.json registry entry" }
  down_ok: { kind: terminal, label: "ExitCode SUCCESS registry entry removed AC5" }
  ps_read: { kind: process, label: "ps reads the ComposeRecord for project vat_id plus service_ids then store::load vat_id to get that run test_run.services" }
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
  - { from: key_check, to: expand_build, label: "supported" }
  - { from: expand_build, to: expand_fields }
  - { from: expand_fields, to: depends_warn }
  - { from: depends_warn, to: runtime_assign }
  - { from: runtime_assign, to: materialize }
  - { from: materialize, to: import_ok }
  - { from: up_expand, to: up_mode }
  - { from: up_mode, to: up_fg_poll, label: "foreground" }
  - { from: up_mode, to: up_bg, label: "detach" }
  - { from: up_fg_poll, to: up_fg_call }
  - { from: up_fg_call, to: runtime_route }
  - { from: runtime_route, to: docker_path, label: "auto or docker" }
  - { from: runtime_route, to: microvm_path, label: "microvm" }
  - { from: docker_path, to: runner_persist }
  - { from: microvm_path, to: runner_persist }
  - { from: runner_persist, to: up_fg_result }
  - { from: up_fg_result, to: up_fg_err, label: "nonzero" }
  - { from: up_fg_result, to: up_fg_ok, label: "zero" }
  - { from: up_bg, to: up_bg_poll }
  - { from: up_bg_poll, to: up_bg_result }
  - { from: up_bg_result, to: up_bg_timeout, label: "not found" }
  - { from: up_bg_result, to: up_bg_ok, label: "found" }
  - { from: down_read, to: down_pid_check }
  - { from: down_pid_check, to: down_pid_missing, label: "absent" }
  - { from: down_pid_check, to: down_signal, label: "present" }
  - { from: down_signal, to: down_wait }
  - { from: down_wait, to: down_remove }
  - { from: down_remove, to: down_ok }
  - { from: ps_read, to: ps_filter }
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
      New apps/vat/src/compose.rs struct ComposeFile (Deserialize via serde_yaml):
      services: BTreeMap<String, ComposeService>, volumes: BTreeMap<String, ComposeVolume>
      (named-volume declarations only -- driver: local or unspecified; any other
      driver/driver_opts form is a hard reject per R3), version: serde(default) String
      (parsed but ignored, matching Docker Compose's own deprecation of the version key).
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
      New apps/vat/src/compose.rs fn parse(path: &Path) -> Result<ComposeFile>. Reads
      the file, deserializes via serde_yaml (already an unconditional dependency as of
      Phase 2), then walks every top-level key (services, volumes, version, x- prefixed
      keys allowed, everything else hard-reject) and every per-service extra key (x-
      prefixed ignored, R3's list hard-reject) -- producing this exact error text:
      compose file {file} service {id} uses unsupported key {key} -- {reason}; remove
      it or edit the generated vat.toml directly after vat compose import. Top-level
      (non-service) hard-reject keys use {file} with no service segment.
  compose_expand_fn:
    type: object
    description: |-
      New apps/vat/src/compose.rs fn expand(file: &ComposeFile, project: &str) ->
      Result<Vec<ServiceConfig>> (R1). For each ComposeService: resolves build: by
      calling commands::build::build_image() in-process (R6), writing the returned
      BuildReport.tag into ServiceConfig.image (so image: and build: entries converge
      to one uniform image-shaped ServiceConfig); parses ports (H:C becomes
      PortSpec::Fixed(H) plus container_port C, bare C becomes PortSpec::Auto plus
      container_port C); flattens environment into ServiceConfig.image_env; maps
      depends_on 1:1 onto ServiceConfig.requires (service_healthy already rejected by
      parse()); maps named volumes entries onto the new ServiceConfig.volumes:
      Vec<VolumeMount>; and sets every ServiceConfig.runtime from the project-wide
      --runtime selection (R4/R8). Prints one non-fatal warning per service with a
      non-empty depends_on, naming the no-bridge-network-DNS caveat (R3).
  compose_materialize_fn:
    type: object
    description: |-
      New apps/vat/src/compose.rs fn materialize(services: &[ServiceConfig],
      out: &Path) -> Result<()> (R1). Builds a VatConfig with cfg.services set to the
      given slice and exactly one synthesized RunnerConfig (id: project.up, cmd: sleep
      infinity, requires: every service's id, in the same order expand() produced
      them), serializes via toml, and writes it to out (the project directory's
      vat.toml) -- satisfying VatConfig::validate()'s existing at-least-one-runner
      requirement with no change to that function's control flow.
  service_runtime_microvm_variant:
    type: object
    description: |-
      apps/vat/src/config.rs: ServiceRuntime gains a MicroVm variant (Auto, Native,
      Docker, MicroVm), still deriving clap::ValueEnum -- the same enum vat.toml's
      [[services]].runtime key already parses is reused verbatim for the new
      CLI-facing vat compose --runtime auto|docker|microvm flag (R4/R8), so no new
      parsing surface is introduced.
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
      apps/vat/src/commands/run.rs: stop_services() gains a microvm_name branch
      alongside the existing docker_name branch that shells out to docker rm -f --
      the new branch shells out to container rm -f name, force-removing the
      container regardless of how the container run child fared, identical semantics
      to the Docker branch (R5).
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
      runner-spawn loop and strictly before the blocking wait_runner_processes(procs)
      call, an interim Vec<RunnerRunRecord> is built from procs (status:
      ProcessStatus::Running, pid: Some(proc.child.id()), exit_code: None,
      duration_ms: None, command/stdout_log/stderr_log copied from each RunnerProc)
      and written into vat.meta.test_run.runner and test_run.runners, followed by
      vat.save() -- mirroring persist_services()'s existing early-write pattern for
      services. Required by R9: without this, test_run.runner.pid is only ever
      populated after wait_runner_processes returns (i.e. after the runner has
      already exited), which would make vat compose down's SIGTERM-while-running
      path impossible.
  compose_record_struct:
    type: object
    description: |-
      New apps/vat/src/commands/compose.rs struct ComposeRecord
      (Serialize/Deserialize, mirrors commands/cluster.rs's ClusterRecord):
      project: String, vat_id: Option<String>, service_ids: Vec<String>,
      status: String (starting, started, or running), created_at: String (RFC3339).
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
  (project name, runtime selection, live vat_id, service_ids, pid) lives in
  the new root/compose/project/project.json registry file (see CLI and
  Changes sections), not in vat.toml or aw.toml. The only vat.toml shape
  changes are the two additive per-service keys above (runtime = microvm,
  volumes = [ { name = ..., path = ... } ]), which are optional and backward
  compatible with every existing vat.toml.
```
