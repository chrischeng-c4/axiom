---
id: vat-local-agent-test-runner-protocol
summary: Add vat.toml as the local agent test runner protocol for ephemeral environment runs.
fill_sections: [scenarios, mindmap, state-machine, interaction, logic, dependency, db-model, schema, rest-api, rpc-api, async-api, cli, wireframe, component, design-token, config, manifest, runtime-image, deployment, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: full
    rationale: "This TD is the primary verification link for the README local agent test runner protocol work root."
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: agent-legible-state-and-diff-surface
    claim: agent-legible-state-and-diff-surface
    coverage: partial
    rationale: "This TD makes vat.toml the local agent-to-vat protocol for preparing ephemeral test environments and returning structured runner evidence."
---

# Vat Local Agent Test Runner Protocol

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: vat-local-agent-test-runner-protocol-scenarios
scenarios:
  - id: config_runner_success_cleans_up
    given:
      - "a project contains vat.toml with one runner that requires one local service"
      - "the service reaches its configured HTTP readiness endpoint"
    when:
      - "an agent runs `vat run e2e --json`"
    then:
      - "vat creates a copy-on-write workspace for the run"
      - "vat starts the required service only for the duration of the run"
      - "vat executes the selected runner command"
      - "vat emits structured JSON with runner, service, log, artifact, diff, and cleanup metadata"
      - "successful runs use the default failed-only retention policy and remove retained workspace state after emitting the result"
      - "each Vat::save atomically replaces meta.json through a unique sibling temporary file, write, sync_all, and rename, so a concurrent compose reconciler can observe only a complete metadata revision"
  - id: failed_runner_keeps_evidence
    given:
      - "a vat.toml runner exits non-zero"
    when:
      - "the agent runs the named runner"
    then:
      - "vat forwards the runner exit code"
      - "vat keeps the vat directory, logs, artifact list, and diff evidence"
      - "`vat state <id>`, `vat logs <id> runner`, and `vat diff <id>` remain useful for diagnosis"
  - id: service_readiness_timeout_terminates_run
    given:
      - "a runner requires a service with ready_http"
      - "the service never becomes ready before timeout_s"
    when:
      - "vat waits for service readiness"
    then:
      - "the run fails with a readiness timeout"
      - "vat terminates all service process groups"
      - "failure evidence is retained according to the failed-only policy"
  - id: occupied_owned_native_endpoint_fails_closed
    given:
      - "a command-backed native service declares a literal 127.0.0.1 endpoint already served by an unrelated listener"
    when:
      - "vat prepares the owned service and waits for readiness"
    then:
      - "vat rejects the exact service and endpoint before spawning the owned child"
      - "vat does not start dependent runners or kill the unrelated listener"
      - "only an explicit external service may attach to an existing listener"
      - "localhost, IPv6 loopback, and other non-literal loopback readiness hosts are rejected because they cannot share the exact 127.0.0.1 reservation proof"
      - "auto ports remain reserved for the run through preparation and are released immediately at the owned-child spawn boundary"
      - "after spawn, readiness requires the endpoint to transition from unavailable to reachable while the owned child remains live"
  - id: unconfirmed_runtime_cleanup_forces_retention
    given:
      - "a runner or scenario has a VAT-owned Docker or MicroVM service"
      - "the final docker rm -f or container rm -f teardown fails or times out and cannot be proven absent by a successful exact-name bounded list query"
      - "workspace.keep is never or failed"
    when:
      - "the runner or scenario lifecycle reaches finalization"
    then:
      - "vat emits microvm_cleanup_unconfirmed and returns a nonzero lifecycle result"
      - "vat retains its state, logs, artifacts, diff evidence, persisted runtime name, and cleanup_error despite the normal keep policy"
      - "a nonzero rm -f is considered auto-removal only when its exact-name list query succeeds and reports no match: Docker uses container ls -a with an anchored name filter and exact line comparison; MicroVM uses container list --all --format json with no matching id. Query error, timeout, malformed JSON, or a live resource remains unconfirmed"
      - "a later compose down retries only the recorded Docker or MicroVM resource; Kubernetes cluster deletion remains deferred to its own phase"
  - id: direct_command_mode_stays_compatible
    given:
      - "an existing caller uses `vat run -- <cmd>`"
    when:
      - "the caller runs a direct command"
    then:
      - "vat does not require vat.toml"
      - "vat preserves current foreground stdio behavior"
      - "vat forwards the child command exit code"
```

## State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: vat-local-agent-test-runner-protocol-state
initial: start
nodes:
  start: { kind: initial, label: "start runner invocation" }
  load_config: { kind: normal, label: "load vat.toml" }
  create_workspace: { kind: normal, label: "create COW workspace" }
  run_setup: { kind: normal, label: "run setup commands" }
  start_services: { kind: normal, label: "start required services" }
  wait_ready: { kind: choice, label: "services ready" }
  run_runner: { kind: normal, label: "run selected runner" }
  collect_evidence: { kind: normal, label: "collect logs artifacts diff state" }
  cleanup_services: { kind: normal, label: "terminate service processes and record any unconfirmed Docker/MicroVM cleanup" }
  cleanup_confirmed: { kind: choice, label: "rm succeeded or successful exact-name list query proves absent" }
  retain: { kind: choice, label: "apply configured retention policy" }
  forced_retain: { kind: normal, label: "force nonzero result and retain VAT evidence for cleanup retry" }
  done: { kind: terminal, label: "return runner or scenario exit code" }
  timeout: { kind: terminal, label: "readiness timeout" }
edges:
  - { from: start, to: load_config, event: "vat run <runner-id>" }
  - { from: load_config, to: create_workspace, event: "runner found" }
  - { from: create_workspace, to: run_setup, event: "workspace ready" }
  - { from: run_setup, to: start_services, event: "setup complete" }
  - { from: start_services, to: wait_ready, event: "services spawned" }
  - { from: wait_ready, to: run_runner, event: "ready" }
  - { from: wait_ready, to: timeout, event: "timeout" }
  - { from: run_runner, to: collect_evidence, event: "runner exited" }
  - { from: collect_evidence, to: cleanup_services, event: "evidence recorded" }
  - { from: cleanup_services, to: cleanup_confirmed, event: "teardown recorded" }
  - { from: cleanup_confirmed, to: retain, event: "yes" }
  - { from: cleanup_confirmed, to: forced_retain, event: "cleanup_error present" }
  - { from: retain, to: done, event: "policy applied" }
  - { from: forced_retain, to: done, event: "retained nonzero" }
---
stateDiagram-v2
    [*] --> load_config
    load_config --> create_workspace: runner found
    create_workspace --> run_setup: workspace ready
    run_setup --> start_services: setup complete
    start_services --> wait_ready: services spawned
    wait_ready --> run_runner: ready
    wait_ready --> timeout: timeout
    run_runner --> collect_evidence: runner exited
    collect_evidence --> cleanup_services: evidence recorded
    cleanup_services --> cleanup_confirmed: teardown recorded
    cleanup_confirmed --> retain: yes
    cleanup_confirmed --> forced_retain: cleanup_error present
    retain --> done: policy applied
    forced_retain --> done: retained nonzero
```

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: vat-local-agent-test-runner-protocol-interaction
actors:
  - { id: agent, kind: actor }
  - { id: vat_cli, kind: system }
  - { id: config_loader, kind: participant }
  - { id: store, kind: participant }
  - { id: service_manager, kind: participant }
  - { id: runner, kind: participant }
messages:
  - { from: agent, to: vat_cli, name: "vat run e2e --json", returns: "structured run result" }
  - { from: vat_cli, to: config_loader, name: "load vat.toml", returns: "runner contract" }
  - { from: vat_cli, to: store, name: "create workspace vat", returns: "vat id and rootfs" }
  - { from: vat_cli, to: service_manager, name: "start required services", returns: "service handles and logs" }
  - { from: service_manager, to: vat_cli, name: "wait readiness", returns: "ready or timeout" }
  - { from: vat_cli, to: runner, name: "execute runner command", returns: "exit code" }
  - { from: vat_cli, to: store, name: "persist evidence", returns: "state projection" }
---
sequenceDiagram
    actor agent
    participant vat_cli
    participant config_loader
    participant store
    participant service_manager
    participant runner
    agent->>vat_cli: vat run e2e --json
    vat_cli->>config_loader: load vat.toml
    config_loader-->>vat_cli: runner contract
    vat_cli->>store: create workspace vat
    store-->>vat_cli: vat id and rootfs
    vat_cli->>service_manager: start required services
    service_manager-->>vat_cli: service handles and logs
    service_manager->>vat_cli: wait readiness
    vat_cli->>runner: execute runner command
    runner-->>vat_cli: exit code
    vat_cli->>store: persist evidence
    store-->>vat_cli: state projection
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-local-agent-test-runner-protocol-logic
entry: start
nodes:
  start: { kind: start, label: "dispatch run" }
  direct_mode: { kind: decision, label: "direct command after --" }
  legacy_run: { kind: process, label: "run existing foreground command path" }
  load_config: { kind: process, label: "load nearest vat.toml" }
  resolve_runner: { kind: decision, label: "runner exists" }
  create_vat: { kind: process, label: "create vat workspace and atomically persist metadata" }
  setup: { kind: process, label: "run setup entries" }
  services: { kind: process, label: "spawn required services with log files" }
  readiness: { kind: decision, label: "ready before timeout" }
  runner: { kind: process, label: "run runner with env and cwd" }
  evidence: { kind: process, label: "record logs artifacts diff result by unique sibling temp write, sync_all, and rename of VAT metadata" }
  cleanup: { kind: process, label: "terminate services and persist any unconfirmed Docker/MicroVM cleanup" }
  cleanup_confirmed: { kind: decision, label: "rm succeeded or successful exact-name list query proves absent" }
  retain: { kind: process, label: "apply configured retention policy" }
  forced_retain: { kind: process, label: "force nonzero result and retain VAT state/logs/diff for cleanup retry" }
  done: { kind: terminal, label: "return runner or scenario exit code" }
  error: { kind: terminal, label: "return config or readiness error" }
edges:
  - { from: start, to: direct_mode }
  - { from: direct_mode, to: legacy_run, label: "yes" }
  - { from: legacy_run, to: done }
  - { from: direct_mode, to: load_config, label: "no" }
  - { from: load_config, to: resolve_runner }
  - { from: resolve_runner, to: error, label: "missing" }
  - { from: resolve_runner, to: create_vat, label: "found" }
  - { from: create_vat, to: setup }
  - { from: setup, to: services }
  - { from: services, to: readiness }
  - { from: readiness, to: error, label: "timeout" }
  - { from: readiness, to: runner, label: "ready" }
  - { from: runner, to: evidence }
  - { from: evidence, to: cleanup }
  - { from: cleanup, to: cleanup_confirmed }
  - { from: cleanup_confirmed, to: retain, label: "yes" }
  - { from: cleanup_confirmed, to: forced_retain, label: "cleanup_error" }
  - { from: retain, to: done }
  - { from: forced_retain, to: done }
---
flowchart TD
    start([dispatch run]) --> direct_mode{direct command after --}
    direct_mode -- yes --> legacy_run[run existing foreground command path]
    legacy_run --> done([return exit code])
    direct_mode -- no --> load_config[load nearest vat.toml]
    load_config --> resolve_runner{runner exists}
    resolve_runner -- missing --> error([return config or readiness error])
    resolve_runner -- found --> create_vat[create vat workspace and metadata]
    create_vat --> setup[run setup entries]
    setup --> services[spawn required services with log files]
    services --> readiness{ready before timeout}
    readiness -- timeout --> error
    readiness -- ready --> runner[run runner with env and cwd]
    runner --> evidence[record logs artifacts diff result]
    evidence --> cleanup[terminate services]
    cleanup --> cleanup_confirmed{all runtime cleanup confirmed}
    cleanup_confirmed -- yes --> retain[apply configured retention policy]
    cleanup_confirmed -- cleanup_error --> forced_retain[force nonzero and retain evidence for cleanup retry]
    retain --> done
    forced_retain --> done
```

## Dependency
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: vat-local-agent-test-runner-protocol-dependency
---
classDiagram
    class VatConfig
    class WorkspaceConfig
    class SetupStep
    class ServiceConfig
    class RunnerConfig
    class TestRunEvidence
    class ServiceRunState
    class RunnerRunState
    class VatState
    VatConfig --> WorkspaceConfig
    VatConfig --> SetupStep
    VatConfig --> ServiceConfig
    VatConfig --> RunnerConfig
    RunnerConfig --> ServiceConfig
    TestRunEvidence --> ServiceRunState
    TestRunEvidence --> RunnerRunState
    VatState --> TestRunEvidence
```

## Data Model
<!-- type: db-model lang: mermaid -->

```mermaid
---
id: vat-local-agent-test-runner-protocol-data-model
---
erDiagram
    VAT ||--o| TEST_RUN : records
    TEST_RUN ||--o{ SERVICE_RUN : starts
    TEST_RUN ||--|| RUNNER_RUN : executes
    TEST_RUN ||--o{ ARTIFACT : lists
    TEST_RUN {
      string config_path
      string config_digest
      string runner_id
      string retention
      int exit_code
    }
    SERVICE_RUN {
      string id
      string status
      string ready_http
      string docker_name
      string microvm_name
      string readiness_error
      string cleanup_error
      string stdout_log
      string stderr_log
    }
    RUNNER_RUN {
      string id
      string status
      int exit_code
      int pid
      string stdout_log
      string stderr_log
    }
    ARTIFACT {
      string path
      int size_bytes
    }
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-test-run-evidence.schema.json"
title: "Vat test run evidence"
type: object
required: [config_path, runner_id, retention, services, runner, artifacts]
properties:
  config_path:
    type: string
  config_digest:
    type: string
  runner_id:
    type: string
  retention:
    type: string
    enum: [failed, always, never]
  services:
    type: array
    items:
      type: object
      required: [id, status, stdout_log, stderr_log]
      properties:
        id: { type: string }
        status: { type: string, enum: [created, running, ready, exited, failed, timeout] }
        exit_code: { type: [integer, "null"] }
        ready_http: { type: [string, "null"] }
        docker_name:
          type: [string, "null"]
          description: VAT-owned Docker container resource retained for teardown retry.
        microvm_name:
          type: [string, "null"]
          description: VAT-owned Apple container resource for a MicroVM-backed service.
        readiness_error:
          type: [string, "null"]
          description: Last terminal readiness observation, including published host-endpoint diagnostics.
        cleanup_error:
          type: [string, "null"]
          description: Unconfirmed VAT-owned Docker or MicroVM cleanup; a nonzero rm -f clears it only after a successful bounded exact-name list query proves absence: Docker container ls -a with anchored name filtering and exact line comparison, or MicroVM container list JSON with no matching id. Query error, timeout, malformed JSON, or a match remains unconfirmed. Runner and scenario finalization then force a nonzero result and retain the VAT regardless of keep=never or keep=failed. Compose retains the binding and retries the persisted runtime name before another run can reuse the host port.
        stdout_log: { type: string }
        stderr_log: { type: string }
      additionalProperties: false
  runner:
    type: object
    required: [id, status, command, stdout_log, stderr_log]
    properties:
      id: { type: string }
      status: { type: string, enum: [created, running, exited, failed] }
      command:
        type: array
        items: { type: string }
      exit_code: { type: [integer, "null"] }
      pid:
        type: [integer, "null"]
        description: Live runner PID while the VAT parent owns the child; readiness/reconciliation evidence only, never a compose-down signal target.
      stdout_log: { type: string }
      stderr_log: { type: string }
    additionalProperties: false
  artifacts:
    type: array
    items:
      type: object
      required: [path]
      properties:
        path: { type: string }
        size_bytes: { type: [integer, "null"] }
      additionalProperties: false
additionalProperties: false
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info:
  title: "No REST API change"
  version: "0.0.0"
paths: {}
components:
  schemas: {}
x-aw-contract:
  surface: none
  reason: "Vat remains a local CLI tool for this slice."
```

## RPC API
<!-- type: rpc-api lang: yaml -->

```yaml
openrpc: 1.3.2
info:
  title: "No JSON-RPC API change"
  version: "0.0.0"
methods: []
x-aw-contract:
  surface: none
  reason: "No RPC surface is introduced."
```

## Async API
<!-- type: async-api lang: yaml -->

```yaml
asyncapi: 2.6.0
info:
  title: "No async API change"
  version: "0.0.0"
channels: {}
x-aw-contract:
  surface: none
  reason: "No pub-sub or streaming protocol is introduced."
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat run
    forms:
      - usage: "vat run -- <command>"
        behavior:
          - "Preserves existing direct foreground command mode."
          - "Does not require vat.toml."
      - usage: "vat run <runner-id> [--json]"
        behavior:
          - "Loads vat.toml from the current directory or an ancestor."
          - "Executes the named runner with its setup and required services."
          - "Returns the runner exit code."
          - "If Docker or MicroVM cleanup remains unconfirmed after rm success or a successful exact-name list absence proof, emits microvm_cleanup_unconfirmed, returns nonzero, and retains the VAT despite keep=never or keep=failed so its state/logs/diff and retry path remain available."
  - name: vat logs
    usage: "vat logs <vat-id> [service-id|runner]"
    behavior:
      - "Prints captured stdout and stderr evidence for the selected source."
      - "Defaults to all captured logs for the vat run."
```

## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
layout:
  id: vat-local-agent-test-runner-wireframe
  surfaces: []
  note: "No UI surface is added."
```

## Component
<!-- type: component lang: yaml -->

```yaml
schemaVersion: "1.0.0"
readme: "No web component contract is changed."
modules: []
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
$schema: "https://design-tokens.github.io/community-group/format/"
tokens: {}
metadata:
  reason: "No visual design token is introduced."
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config.schema.json"
title: "vat.toml"
type: object
required: [version, runners]
properties:
  version:
    type: integer
    const: 1
  name:
    type: string
  workspace:
    type: object
    properties:
      base: { type: string, default: "." }
      workdir: { type: string, default: "." }
      keep: { type: string, enum: [failed, always, never], default: failed }
    additionalProperties: false
  env:
    type: object
    additionalProperties: { type: string }
  setup:
    type: array
    items:
      type: object
      required: [id, cmd]
      properties:
        id: { type: string }
        cmd:
          type: array
          items: { type: string }
          minItems: 1
        when: { type: string }
      additionalProperties: false
  services:
    type: array
    items:
      type: object
      required: [id]
      description: >
        A run-scoped dependency service. It is backed by exactly one of: cmd
        (an explicit native command), preset (a built-in service whose runtime
        decides native-binary, Docker, or explicit Apple Container MicroVM), or
        image (an OCI service such as AlloyDB, which requires container_port).
        `micro_vm` is explicit: it never silently falls back to Docker and its
        declared OCI route must pass bounded Apple image inspect/pull/verify
        preflight. The runner that requires the service is always a host process
        — only the service may be a container, so the host GPU story is
        unaffected.
      properties:
        id: { type: string }
        requires:
          type: array
          items: { type: string }
        cmd:
          type: array
          items: { type: string }
          minItems: 1
        preset: { type: string, enum: [postgres, redis, nats, rabbitmq, mysql, mongo] }
        image: { type: string }
        container_port: { type: integer }
        image_env:
          type: object
          additionalProperties: { type: string }
        runtime: { type: string, enum: [auto, native, docker, micro_vm], default: auto }
        version: { type: string }
        port:
          oneOf:
            - { type: string, const: auto }
            - { type: integer }
        seed:
          type: array
          items: { type: string }
        export:
          type: object
          additionalProperties: { type: string }
        ready_http: { type: string }
        timeout_s: { type: integer, default: 60 }
      additionalProperties: false
  runners:
    type: array
    items:
      type: object
      required: [id, cmd]
      properties:
        id: { type: string }
        requires:
          type: array
          items: { type: string }
        cmd:
          type: array
          items: { type: string }
          minItems: 1
        timeout_s: { type: integer }
        artifacts:
          type: array
          items: { type: string }
      additionalProperties: false
additionalProperties: false
```

## Manifest
<!-- type: manifest lang: yaml -->

```yaml
manifests:
  - path: apps/vat/Cargo.toml
    changes:
      - "Add TOML parsing dependency for vat.toml."
      - "Use libc as a normal dependency for process cleanup on Unix hosts."
```

## Runtime Image
<!-- type: runtime-image lang: yaml -->

```yaml
images: []
build_contexts: []
x-aw-contract:
  surface: none
  reason: >
    Vat ships no image and builds none. It may run a run-scoped dependency
    service as an ephemeral `docker run` or explicit Apple `container run`
    container (a preset/image with the selected runtime), but it is not an image
    registry/builder and the runner is always a host process.
```

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployments: []
operations:
  - id: local-vat-cli
    action: "build and run the local vat binary"
    verification:
      - "cargo test -p vat"
      - "aw health vat --verify-traceability --verify-cb --verify-cold --verify-tests --verify-ec"
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-local-agent-test-runner-unit-tests
---
requirementDiagram
    requirement parse_config {
      id: UT1
      text: "Vat parses valid vat.toml and rejects duplicate runner or service ids."
      risk: high
      verifymethod: test
    }
    requirement resolve_paths {
      id: UT2
      text: "Relative workspace and artifact paths resolve from the vat.toml directory."
      risk: medium
      verifymethod: test
    }
    requirement keep_direct_run {
      id: UT3
      text: "Direct command mode still forwards child exit codes."
      risk: high
      verifymethod: test
    }
    requirement own_native_endpoint {
      id: UT4
      text: "Concurrent native auto-port allocations are unique, fixed occupied 127.0.0.1 endpoints fail closed, unsupported loopback spellings are rejected, and every failed or finished plan releases its reservation."
      risk: high
      verifymethod: test
    }
    requirement bind_readiness_to_child {
      id: UT5
      text: "An owned network service is Ready only after its reserved endpoint transitions to reachable and its child stays live before and after the configured probe."
      risk: high
      verifymethod: test
    }
    test config_parse_tests {
      type: functional
      verifies: parse_config
    }
    test direct_run_regression {
      type: functional
      verifies: keep_direct_run
    }
    test native_endpoint_ownership_regression {
      type: functional
      verifies: own_native_endpoint
    }
    test owned_child_readiness_regression {
      type: functional
      verifies: bind_readiness_to_child
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-toml-runner-local-service-smoke
    name: "vat.toml runner local service smoke"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: local-agent-test-runner-protocol
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_toml_runner -- --nocapture"
    assertions:
      - "vat run <runner-id> starts a local readiness service, runs the runner, captures logs, records artifacts, and returns JSON evidence."
      - "failed runner evidence remains inspectable."
      - "An unconfirmed Docker or MicroVM cleanup forces a nonzero result and keeps the VAT evidence regardless of the normal keep policy."
      - "An occupied command-backed native endpoint fails before service or runner spawn, reports the exact service/endpoint, and leaves the unrelated listener reachable."
      - "A child that exits before its reserved endpoint becomes ready is terminal and no dependent runner starts."
      - "Explicit external services retain attach-and-probe behavior without VAT lifecycle ownership."
      - "direct vat run -- <cmd> compatibility is preserved."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: create
    section: changes
    impl_mode: hand-written
    reason: "Define the local agent test runner protocol TD."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: scenarios
    impl_mode: hand-written
    reason: "Record success, failure retention, readiness timeout, and direct-run compatibility scenarios."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: mindmap
    impl_mode: hand-written
    reason: "Record protocol, workspace, execution, evidence, and boundary concepts."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: state-machine
    impl_mode: hand-written
    reason: "Record the ephemeral runner lifecycle."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: interaction
    impl_mode: hand-written
    reason: "Record the agent, CLI, config, store, service, and runner interaction."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: dependency
    impl_mode: hand-written
    reason: "Record the config and evidence type relationships."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: db-model
    impl_mode: hand-written
    reason: "Record the persisted vat, test run, service run, runner run, and artifact relationships."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: rest-api
    impl_mode: hand-written
    reason: "Record non-applicability because this slice adds no REST API."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: rpc-api
    impl_mode: hand-written
    reason: "Record non-applicability because this slice adds no RPC API."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: async-api
    impl_mode: hand-written
    reason: "Record non-applicability because this slice adds no async API."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: wireframe
    impl_mode: hand-written
    reason: "Record non-applicability because this slice adds no UI surface."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: component
    impl_mode: hand-written
    reason: "Record non-applicability because this slice adds no component contract."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: design-token
    impl_mode: hand-written
    reason: "Record non-applicability because this slice adds no design tokens."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: runtime-image
    impl_mode: hand-written
    reason: "Record non-applicability because vat does not add Docker or OCI runtime behavior."
  - path: apps/vat/tech-design/logic/local-agent-test-runner-protocol.md
    action: validate
    section: deployment
    impl_mode: hand-written
    reason: "Record local CLI verification as the deployment impact."
  - path: apps/vat/src/config.rs
    action: add
    section: config
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config"
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema"
    summary: "Load, validate, and resolve vat.toml runner contracts."
  - path: apps/vat/src/commands/run.rs
    action: modify
    section: logic
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic"
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#cli"
    summary: "Dispatch vat run <runner-id> through setup, services, readiness, runner execution, evidence, and cleanup."
  - path: apps/vat/src/state.rs
    action: modify
    section: schema
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema"
    summary: "Persist and project runner evidence metadata."
  - path: apps/vat/src/store.rs
    action: modify
    section: logic
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic"
    summary: "Atomically replace VAT metadata with a unique sibling temporary file, write, sync_all, and rename so concurrent compose reconciliation never observes a truncate/write interval."
  - path: apps/vat/src/commands/logs.rs
    action: add
    section: cli
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#cli"
    summary: "Expose captured per-run logs."
  - path: apps/vat/README.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#scenarios"
    summary: "Document vat as an ephemeral local agent test runner protocol."
  - path: apps/vat/tests
    action: modify
    section: unit-test
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#unit-test"
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#e2e-test"
    summary: "Add parser and runner smoke coverage."
  - path: apps/vat/tests
    action: modify
    section: e2e-test
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#e2e-test"
    summary: "Add vat.toml local service runner smoke coverage."
  - path: apps/vat/Cargo.toml
    action: modify
    section: manifest
    impl_mode: hand-written
    refs:
      - "apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#manifest"
    summary: "Add TOML parsing and process cleanup dependencies."
```

## Mindmap
<!-- type: mindmap lang: mermaid -->

```mermaid
---
id: vat-local-agent-test-runner-protocol-map
---
mindmap
  root((vat local agent test runner))
    Contract
      vat.toml
      named runners
      run scoped services
    Workspace
      copy on write rootfs
      failed only retention
      diff evidence
    Execution
      setup
      service readiness
      runner command
      process group cleanup
    Evidence
      structured state
      logs
      artifacts
      exit code
    Boundaries
      no daemon
      no long lived process manager
      no Docker runtime
```

# Reviews

### Review 1
**Verdict:** approved

- [scenarios] The TD captures the intended ephemeral agent test runner lifecycle, including success cleanup, failed evidence retention, service readiness timeout, and direct-run compatibility.
- [config] The `vat.toml` schema is intentionally small and project-local, with `workspace`, `env`, `setup`, `services`, and `runners` only.
- [logic] The orchestration flow keeps services run-scoped and explicitly avoids daemon or long-lived process management.
- [changes] The implementation scope is bounded to vat config loading, run orchestration, state evidence, logs, README, manifest, and tests.
