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

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-build-phase2.schema.json"
title: "vat build Phase 2: data model additions"
type: object
properties:
  build_args_struct:
    type: object
    description: "New apps/vat/src/commands/build.rs struct Args: file: Option<PathBuf> (defaults to `Dockerfile` inside the resolved context dir), context: Option<PathBuf> (defaults to the current directory), tag: Option<String> (defaults to `<context-dir-basename>:latest`, sanitized to a valid OCI reference — lowercased, non [a-z0-9._-] runs collapsed to `-` — resolved once in exec() before any subprocess is spawned; build_image() itself never guesses a tag, it always receives a concrete &str), build_args: Vec<(String,String)> (one pair per repeated --build-arg K=V flag, parsed via split_once('='), CLI-supplied order preserved — no BTreeMap reordering needed here since the input is already a deterministic Vec, unlike Phase 1's EnvSpec.env map), json: bool."
  build_report_struct:
    type: object
    description: "New apps/vat/src/commands/build.rs struct BuildReport (derives serde::Serialize): tag: String, dockerfile: String (resolved absolute path), context: String (resolved absolute path), build_args: BTreeMap<String,String> (sorted for deterministic JSON field ordering in the report, independent of the argv-ordering rule above), duration_ms: u64. Constructed only on a successful build — build_image()'s Result<BuildReport> Err variant covers every failure path (missing Dockerfile, container CLI/system unavailable, nonzero container build exit); there is no success:false variant."
  container_build_command_fn:
    type: object
    description: "New apps/vat/src/commands/build.rs fn container_build_command(dockerfile: &Path, tag: &str, build_args: &[(String,String)], context: &Path) -> Vec<String>. Pure, deterministic argv builder (no subprocess, no I/O) producing exactly: [\"container\", \"build\", \"-f\", <dockerfile>, \"-t\", <tag>, \"--build-arg\", \"K=V\", ... one --build-arg pair per entry in the given slice order ..., <context>] (R2), matching the real invocation Phase 0 verified (`container build -f \"$WORKDIR/Dockerfile\" -t vat-spike-test:latest \"$WORKDIR\"`). Unlike sandbox/microvm.rs's resolve() (which returns a (program, argv) tuple), this fn returns the program name (\"container\") as argv[0] itself."
  build_image_fn:
    type: object
    description: "New apps/vat/src/commands/build.rs fn build_image(context: &Path, dockerfile: &Path, tag: &str, build_args: &[(String,String)]) -> Result<BuildReport> — the in-process entry point Phase 3's `vat compose` will call directly for compose `build:` keys (not a shell-out to the vat binary). Validates the dockerfile path exists (AC3, no subprocess on failure), calls ensure_microvm_available(), builds argv via container_build_command(), spawns `container` with captured stdout/stderr (never inherited — the always-captured behavior a reusable in-process caller like compose needs), waits for exit, and returns Err on a nonzero exit or Ok(BuildReport) on success."
  ensure_microvm_available_fn:
    type: object
    description: "New apps/vat/src/commands/build.rs fn ensure_microvm_available() -> Result<()>, mirroring run.rs's ensure_docker_available: requires sandbox::microvm::available() (container binary on PATH); if sandbox::microvm::system_up() is not immediately true, waits via sandbox::microvm::ensure_system_started(<bounded timeout>) before failing. Fail-closed and clean — never auto-installs the container CLI, never silently proceeds without a responsive system."
  microvm_system_up_fn:
    type: object
    description: "New additive apps/vat/src/sandbox/microvm.rs fn: pub fn system_up() -> bool. Bounded-timeout `container system status` probe (spawn with stdout/stderr to null, short internal timeout, return status.success()), mirroring run.rs's docker_daemon_up(). Additive only — no change to MicroVmBackend, resolve(), or available() (Phase 1, untouched)."
  microvm_ensure_system_started_fn:
    type: object
    description: "New additive apps/vat/src/sandbox/microvm.rs fn: pub fn ensure_system_started(timeout: Duration) -> Result<(), String>. Poll+timeout loop (Instant::now() deadline, short sleep between polls of system_up()) mirroring cluster.rs's run_capture() poll pattern; returns Err naming the elapsed timeout when the container system never reports up within the bound. Additive only, same non-interference guarantee as system_up() above."
  cargo_serde_yaml_promotion:
    type: object
    description: "apps/vat/Cargo.toml: serde_yaml (version 0.9, unchanged) loses `optional = true` and becomes an unconditional dependency; removed from the `emulator` feature's `dep:serde_yaml` list. Zero new crate, needed unconditionally ahead of Phase 3's compose YAML parsing (not used by this WI's own code)."
additionalProperties: true
```
