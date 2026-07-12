---
id: vat-microvm-phase-3-vat-compose-limited-compose-subset-up-down-p
summary: (fill)
fill_sections: [logic]
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
