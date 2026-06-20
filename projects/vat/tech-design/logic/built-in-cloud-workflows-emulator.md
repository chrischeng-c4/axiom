---
id: vat-built-in-cloud-workflows-emulator
summary: Ship a built-in Rust Cloud Workflows emulator in vat — a subset interpreter behind the Workflows Executions v1 REST API whose call http steps drive vat's other local emulators.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Adds a built-in Cloud Workflows emulator that orchestrates vat's other local emulators (and any HTTP endpoint), turning the individual emulator presets into end-to-end local orchestration through vat's run and evidence surface."
---

# Vat Built-in Cloud Workflows Emulator

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-built-in-cloud-workflows-emulator-logic
entry: start
nodes:
  start: { kind: start, label: "dispatch builtin preset cloud-workflows" }
  spawn: { kind: process, label: "spawn self vat emulator cloud-workflows host-port" }
  rt: { kind: process, label: "emulator builds tokio runtime; axum REST" }
  ready: { kind: process, label: "tcp readiness; export CLOUD_WORKFLOWS_EMULATOR_HOST" }
  runner: { kind: process, label: "runner createWorkflow then createExecution" }
  parse: { kind: process, label: "parse sourceContents yaml or json" }
  step: { kind: decision, label: "next step kind" }
  assign: { kind: process, label: "assign evaluate expr into scope" }
  call: { kind: decision, label: "http or sys.log or subworkflow" }
  dispatch: { kind: process, label: "dispatch_http reqwest call target capture result" }
  subwf: { kind: process, label: "bind args to params run subworkflow steps" }
  switch: { kind: process, label: "switch eval conditions choose branch" }
  forl: { kind: process, label: "for iterate list run body steps" }
  tryb: { kind: process, label: "try run; on error retry then except" }
  ret: { kind: terminal, label: "return or raise unwinds execution" }
  finish: { kind: process, label: "store execution state SUCCEEDED or FAILED with result" }
  teardown: { kind: process, label: "stop service kills emulator child" }
  done: { kind: terminal, label: "return exit code" }
edges:
  - { from: start, to: spawn }
  - { from: spawn, to: rt }
  - { from: rt, to: ready }
  - { from: ready, to: runner }
  - { from: runner, to: parse }
  - { from: parse, to: step }
  - { from: step, to: assign, label: "assign" }
  - { from: step, to: call, label: "call" }
  - { from: step, to: switch, label: "switch" }
  - { from: step, to: forl, label: "for" }
  - { from: step, to: tryb, label: "try" }
  - { from: step, to: ret, label: "return or raise" }
  - { from: call, to: dispatch, label: "http" }
  - { from: call, to: subwf, label: "subworkflow" }
  - { from: assign, to: step }
  - { from: dispatch, to: step }
  - { from: subwf, to: step }
  - { from: switch, to: step }
  - { from: forl, to: step }
  - { from: tryb, to: step }
  - { from: ret, to: finish }
  - { from: finish, to: teardown }
  - { from: teardown, to: done }
---
flowchart TD
    start([dispatch builtin preset cloud-workflows]) --> spawn[spawn self vat emulator cloud-workflows host-port]
    spawn --> rt[emulator builds tokio runtime; axum REST]
    rt --> ready[tcp readiness; export CLOUD_WORKFLOWS_EMULATOR_HOST]
    ready --> runner[runner createWorkflow then createExecution]
    runner --> parse[parse sourceContents yaml or json]
    parse --> step{next step kind}
    step -- assign --> assign[assign evaluate expr into scope]
    step -- call --> call{http or sys.log or subworkflow}
    step -- switch --> switch[switch eval conditions choose branch]
    step -- for --> forl[for iterate list run body steps]
    step -- try --> tryb[try run; on error retry then except]
    step -- return or raise --> ret([return or raise unwinds execution])
    call -- http --> dispatch[dispatch_http reqwest call target capture result]
    call -- subworkflow --> subwf[bind args to params run subworkflow steps]
    assign --> step
    dispatch --> step
    subwf --> step
    switch --> step
    forl --> step
    tryb --> step
    ret --> finish[store execution state SUCCEEDED or FAILED with result]
    finish --> teardown[stop service kills emulator child]
    teardown --> done([return exit code])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cloud-workflows-evidence.schema.json"
title: "Vat Cloud Workflows emulator evidence"
type: object
description: "Service-evidence shape and the Workflows execution result for vat's built-in Cloud Workflows emulator."
properties:
  preset:
    type: string
    enum: [cloud-workflows]
  prepare_mode:
    type: string
    enum: [builtin_emulator]
  exported_env:
    type: array
    items: { type: string }
    description: "Host env var exported to the runner: CLOUD_WORKFLOWS_EMULATOR_HOST."
  execution:
    type: object
    description: "A Workflows execution result returned by getExecution."
    properties:
      name: { type: string }
      state: { type: string, enum: [ACTIVE, SUCCEEDED, FAILED] }
      result: {}
      error:
        type: [object, "null"]
        properties:
          message: { type: string }
        additionalProperties: true
    additionalProperties: true
additionalProperties: true
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-cloud-workflows.schema.json"
title: "vat.toml (Cloud Workflows preset addition)"
type: object
properties:
  services:
    type: array
    items:
      type: object
      required: [id]
      properties:
        preset:
          type: string
          enum: [postgres, redis, nats, rabbitmq, mysql, mongo, firestore, pubsub, datastore, bigtable, spanner, firebase, firebase-auth, cloud-tasks, cloud-scheduler, cloud-workflows]
          description: >
            cloud-workflows runs vat's built-in Workflows emulator under
            runtime=auto (no gcloud/Java/Docker — Cloud Workflows has no official
            emulator). It exports CLOUD_WORKFLOWS_EMULATOR_HOST; point your client's
            base URL at http://$HOST. Built-in only: runtime must stay auto. The
            interpreter supports a subset (assign/call http+sys.log/switch/for/
            try-retry-except/subworkflow + a ${...} expression evaluator), enough
            to orchestrate vat's other local emulators and HTTP endpoints.
        runtime:
          type: string
          enum: [auto, native, docker]
          default: auto
        export:
          type: object
          additionalProperties: { type: string }
      additionalProperties: true
additionalProperties: true
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat emulator
    usage: "vat emulator cloud-workflows --host-port 127.0.0.1:<PORT>"
    behavior:
      - "Hidden verb: vat spawns itself as the service process for the cloud-workflows preset."
      - "Serves the Workflows v1 REST API subset: create/get a workflow (YAML or JSON sourceContents), createExecution (runs the workflow to completion), getExecution (terminal state + result)."
      - "Executes the Core + try/retry + subworkflow step set with a ${...} expression evaluator; call: http.* steps deliver via the shared dispatcher so a workflow can drive vat's other emulators or any HTTP endpoint."
      - "Built without the emulator feature, the verb errors cleanly (no panic). An unsupported expression/step fails the execution rather than panicking."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-built-in-cloud-workflows-emulator-unit-tests
---
requirementDiagram
    requirement preset_parses_builtin {
      id: UT1
      text: "ServicePreset round-trips cloud-workflows, and it classifies as built-in and built-in-only (validate rejects an explicit runtime)."
      risk: medium
      verifymethod: test
    }
    requirement expr_evaluates {
      id: UT2
      text: "The ${...} evaluator handles literals, var/member/index, arithmetic, comparison, logic, string concat/interpolation, and builtins; unsupported input yields an error, never a panic."
      risk: high
      verifymethod: test
    }
    requirement interp_core_steps {
      id: UT3
      text: "The interpreter runs assign + switch + for + return to the expected result with no network."
      risk: high
      verifymethod: test
    }
    requirement interp_try_and_subworkflow {
      id: UT4
      text: "A try block whose body errors falls through to except; a named subworkflow call binds args to params and returns its value."
      risk: high
      verifymethod: test
    }
    requirement workflow_http_dispatch {
      id: UT5
      text: "An execution whose main calls http.post to a local sink delivers the call and returns a SUCCEEDED execution with the expected result."
      risk: high
      verifymethod: test
    }
    test config_cloud_workflows_tests {
      type: functional
      verifies: preset_parses_builtin
    }
    test expr_evaluator_tests {
      type: functional
      verifies: expr_evaluates
    }
    test interp_core_tests {
      type: functional
      verifies: interp_core_steps
    }
    test interp_try_subworkflow_tests {
      type: functional
      verifies: interp_try_and_subworkflow
    }
    test workflows_dispatch_tests {
      type: functional
      verifies: workflow_http_dispatch
    }
```
