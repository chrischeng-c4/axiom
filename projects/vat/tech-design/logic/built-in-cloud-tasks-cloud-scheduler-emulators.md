---
id: vat-built-in-cloud-tasks-cloud-scheduler-emulators
summary: Ship built-in Rust emulators for Cloud Tasks (REST v2) and Cloud Scheduler (REST v1) in vat, dispatching task/job HTTP targets, run as vat emulator subcommands.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Extends vat's built-in emulator framework with Cloud Tasks and Cloud Scheduler — services that have no official emulator — so task-queue and cron-job code can run locally through vat's run and evidence surface."
---

# Vat Built-in Cloud Tasks and Cloud Scheduler Emulators

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-built-in-cloud-tasks-cloud-scheduler-emulators-logic
entry: start
nodes:
  start: { kind: start, label: "dispatch builtin preset cloud-tasks or cloud-scheduler" }
  spawn: { kind: process, label: "spawn self vat emulator kind host-port" }
  rt: { kind: process, label: "emulator builds tokio runtime" }
  kind_q: { kind: decision, label: "cloud-tasks or cloud-scheduler" }
  tasks_srv: { kind: process, label: "axum REST v2 queues and tasks" }
  sched_srv: { kind: process, label: "axum REST v1 jobs plus cron ticker" }
  create_task: { kind: process, label: "createTask store and schedule dispatch at scheduleTime" }
  create_job: { kind: process, label: "createJob store with cron schedule" }
  due_q: { kind: decision, label: "scheduleTime reached or run forced" }
  cron_q: { kind: decision, label: "cron due or run forced" }
  dispatch: { kind: process, label: "dispatch_http reqwest POST target with optional OIDC JWT" }
  export: { kind: process, label: "export EMULATOR_HOST var into runner" }
  ready: { kind: process, label: "tcp readiness on host-port" }
  runner: { kind: process, label: "run runner as host process" }
  teardown: { kind: process, label: "stop service kills emulator child" }
  done: { kind: terminal, label: "return exit code" }
edges:
  - { from: start, to: spawn }
  - { from: spawn, to: rt }
  - { from: rt, to: kind_q }
  - { from: kind_q, to: tasks_srv, label: "cloud-tasks" }
  - { from: kind_q, to: sched_srv, label: "cloud-scheduler" }
  - { from: tasks_srv, to: ready }
  - { from: sched_srv, to: ready }
  - { from: ready, to: export }
  - { from: export, to: runner }
  - { from: runner, to: create_task, label: "via client" }
  - { from: runner, to: create_job, label: "via client" }
  - { from: create_task, to: due_q }
  - { from: due_q, to: dispatch, label: "yes" }
  - { from: create_job, to: cron_q }
  - { from: cron_q, to: dispatch, label: "yes" }
  - { from: runner, to: teardown }
  - { from: teardown, to: done }
---
flowchart TD
    start([dispatch builtin preset cloud-tasks or cloud-scheduler]) --> spawn[spawn self vat emulator kind host-port]
    spawn --> rt[emulator builds tokio runtime]
    rt --> kind_q{cloud-tasks or cloud-scheduler}
    kind_q -- cloud-tasks --> tasks_srv[axum REST v2 queues and tasks]
    kind_q -- cloud-scheduler --> sched_srv[axum REST v1 jobs plus cron ticker]
    tasks_srv --> ready[tcp readiness on host-port]
    sched_srv --> ready
    ready --> export[export EMULATOR_HOST var into runner]
    export --> runner[run runner as host process]
    runner -- via client --> create_task[createTask store and schedule dispatch at scheduleTime]
    runner -- via client --> create_job[createJob store with cron schedule]
    create_task --> due_q{scheduleTime reached or run forced}
    due_q -- yes --> dispatch[dispatch_http reqwest POST target with optional OIDC JWT]
    create_job --> cron_q{cron due or run forced}
    cron_q -- yes --> dispatch
    runner --> teardown[stop service kills emulator child]
    teardown --> done([return exit code])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cloud-tasks-scheduler-evidence.schema.json"
title: "Vat Cloud Tasks / Scheduler emulator evidence"
type: object
description: "Service-evidence shape for vat's built-in Cloud Tasks / Scheduler emulators."
properties:
  preset:
    type: string
    enum: [cloud-tasks, cloud-scheduler]
  prepare_mode:
    type: string
    enum: [builtin_emulator]
  exported_env:
    type: array
    items: { type: string }
    description: "Host env var exported to the runner: CLOUD_TASKS_EMULATOR_HOST or CLOUD_SCHEDULER_EMULATOR_HOST."
  dispatch:
    type: object
    description: "How the emulator delivers a task/job: an outbound HTTP request to the target."
    properties:
      uri: { type: string }
      http_method: { type: string }
      oidc: { type: [boolean] }
    additionalProperties: true
additionalProperties: true
```
