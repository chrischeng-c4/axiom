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
