---
id: vat-microvm-phase-2-vat-build-dockerfile-build-via-container-cli
summary: (fill)
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-build-phase2-exec-logic
entry: start
nodes:
  start: { kind: start, label: "vat build --file --context --tag --build-arg K=V --json invoked" }
  resolve_paths: { kind: process, label: "resolve file default Dockerfile and context default current directory to absolute paths" }
  dockerfile_check: { kind: decision, label: "dockerfile path exists on disk" }
  err_missing_dockerfile: { kind: terminal, label: "hard Err dockerfile not found before any container CLI invocation AC3 no subprocess spawned" }
  avail_check: { kind: decision, label: "ensure_microvm_available container binary on PATH via microvm available and system responsive via microvm system_up or bounded microvm ensure_system_started poll mirrors run.rs ensure_docker_available" }
  err_unavailable: { kind: terminal, label: "hard Err container CLI not installed or system not running install it and run container system start" }
  argv_build: { kind: process, label: "container_build_command builds argv container build -f dockerfile -t tag --build-arg K=V repeated context exact order per R2" }
  mode_check: { kind: decision, label: "json flag on Args" }
  human_run: { kind: process, label: "exec spawns the container_build_command argv directly with inherited stdio so live BuildKit layer progress streams to the terminal in real time" }
  human_result: { kind: decision, label: "child process exit status success" }
  human_err: { kind: terminal, label: "failure already visible in the streamed output exec returns a non-zero ExitCode no BuildReport constructed" }
  human_ok: { kind: terminal, label: "print a one-line tag and elapsed-time summary exec returns ExitCode SUCCESS" }
  json_run: { kind: process, label: "exec calls build_image the reusable in-process entry point Phase 3 compose will call directly which spawns container_build_command argv with captured stdout and stderr internally" }
  json_result: { kind: decision, label: "build_image result" }
  json_err: { kind: terminal, label: "propagate the Err as a structured json error object exec returns a non-zero ExitCode" }
  json_ok: { kind: terminal, label: "print only the structured BuildReport as json the captured build log is never echoed exec returns ExitCode SUCCESS" }
edges:
  - { from: start, to: resolve_paths }
  - { from: resolve_paths, to: dockerfile_check }
  - { from: dockerfile_check, to: err_missing_dockerfile, label: "missing" }
  - { from: dockerfile_check, to: avail_check, label: "present" }
  - { from: avail_check, to: err_unavailable, label: "unavailable" }
  - { from: avail_check, to: argv_build, label: "available" }
  - { from: argv_build, to: mode_check }
  - { from: mode_check, to: human_run, label: "human" }
  - { from: mode_check, to: json_run, label: "json" }
  - { from: human_run, to: human_result }
  - { from: human_result, to: human_err, label: "nonzero" }
  - { from: human_result, to: human_ok, label: "zero" }
  - { from: json_run, to: json_result }
  - { from: json_result, to: json_err, label: "err" }
  - { from: json_result, to: json_ok, label: "ok" }
---
```
