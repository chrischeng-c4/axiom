---
id: cloud-tasks-and-cloud-scheduler-emulator-grpc-front-end-alongsid
summary: Give vat's cloud-tasks and cloud-scheduler emulators a gRPC front-end alongside their existing REST surface, sharing the same in-memory store + dispatcher and multiplexed on the same port, so the stock gRPC SDK clients connect like pubsub does.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "An app using the stock gRPC Cloud Tasks / Cloud Scheduler client cannot reach a REST-only emulator without code-level transport surgery; adding a gRPC front-end over the same store makes the emulator behave like the real service over either protocol."
---

# Cloud Tasks / Cloud Scheduler emulator gRPC front-end (alongside REST)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cloud-tasks-scheduler-grpc-frontend-logic
entry: start
nodes:
  start: { kind: start, label: "vat emulator cloud-tasks or cloud-scheduler binds host_port" }
  build: { kind: process, label: "build shared in-memory store plus dispatcher once" }
  rest: { kind: process, label: "axum REST router over the shared store" }
  grpc: { kind: process, label: "tonic service over the same shared store" }
  mux: { kind: process, label: "multiplex by content-type application/grpc grpc else rest" }
  serve: { kind: process, label: "serve on one TcpListener via hyper auto builder h1 plus h2" }
  req: { kind: decision, label: "incoming request content-type" }
  togrpc: { kind: process, label: "tonic handles CreateTask CreateJob RunTask RunJob etc" }
  torest: { kind: process, label: "axum handles v2 v1 REST routes" }
  store: { kind: process, label: "both write the SAME store create task or job" }
  dispatch: { kind: process, label: "dispatcher fires httpRequest httpTarget identically" }
  done: { kind: terminal, label: "task job delivered to target same path either protocol" }
edges:
  - { from: start, to: build }
  - { from: build, to: rest }
  - { from: build, to: grpc }
  - { from: rest, to: mux }
  - { from: grpc, to: mux }
  - { from: mux, to: serve }
  - { from: serve, to: req }
  - { from: req, to: togrpc, label: "application/grpc" }
  - { from: req, to: torest, label: "other" }
  - { from: togrpc, to: store }
  - { from: torest, to: store }
  - { from: store, to: dispatch }
  - { from: dispatch, to: done }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cloud-tasks-scheduler-grpc-frontend.schema.json"
title: "Dual-protocol emulator wiring"
type: object
properties:
  emulator:
    type: string
    enum: [cloud-tasks, cloud-scheduler]
  protocols:
    type: array
    items: { type: string, enum: [rest, grpc] }
    description: "Both served on the SAME host:port; routed by content-type."
  grpc_service:
    type: string
    enum: [google.cloud.tasks.v2.CloudTasks, google.cloud.scheduler.v1.CloudScheduler]
  implemented_rpcs:
    type: array
    items: { type: string }
    description: "Tasks: CreateQueue,GetQueue,ListQueues,DeleteQueue,CreateTask,GetTask,ListTasks,DeleteTask,RunTask. Scheduler: CreateJob,GetJob,ListJobs,DeleteJob,RunJob,PauseJob,ResumeJob. Others → Unimplemented."
  body_normalization:
    type: string
    description: "gRPC httpRequest.body is raw bytes; REST body is base64 — both normalize to the same internal Vec<u8> before dispatch."
additionalProperties: true
```
## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-cloud-tasks-scheduler-grpc.schema.json"
title: "vat.toml (no new keys; dual-protocol is transparent)"
type: object
properties:
  services:
    type: array
    items:
      type: object
      properties:
        preset:
          type: string
          enum: [cloud-tasks, cloud-scheduler]
          description: >
            Unchanged preset. Still exports CLOUD_TASKS_EMULATOR_HOST /
            CLOUD_SCHEDULER_EMULATOR_HOST; that one host:port now serves BOTH
            gRPC and REST. A gRPC client points its endpoint there with an
            insecure channel; a REST client uses it as before.
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat emulator cloud-tasks
    usage: "vat emulator cloud-tasks --host-port 127.0.0.1:<PORT>"
    behavior:
      - "Serve google.cloud.tasks.v2.CloudTasks over gRPC AND the Cloud Tasks v2 REST API on the same port."
      - "gRPC requests (content-type application/grpc) route to the tonic service; everything else to the axum REST router."
      - "Both share one in-memory store + dispatcher; a task created via either protocol dispatches its httpRequest identically (gRPC bytes / REST base64 body normalize to the same bytes)."
      - "Unused RPCs (UpdateQueue, PurgeQueue, Pause/Resume, IAM) return Unimplemented."
  - name: vat emulator cloud-scheduler
    usage: "vat emulator cloud-scheduler --host-port 127.0.0.1:<PORT>"
    behavior:
      - "Serve google.cloud.scheduler.v1.CloudScheduler over gRPC AND the Cloud Scheduler v1 REST API on the same port, sharing the store + cron dispatcher."
      - "Implements CreateJob, GetJob, ListJobs, DeleteJob, RunJob, PauseJob, ResumeJob; UpdateJob and others return Unimplemented."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cloud-tasks-scheduler-grpc-frontend-unit-tests
---
requirementDiagram
    requirement grpc_rest_share_store {
      id: UT1
      text: "A task created over gRPC is visible to the REST GetTask on the same store, and vice versa (single shared store, not two)."
      risk: high
      verifymethod: test
    }
    requirement body_normalization {
      id: UT2
      text: "gRPC httpRequest.body (raw bytes) and REST body (base64) normalize to the same internal bytes handed to the dispatcher."
      risk: medium
      verifymethod: test
    }
    requirement mux_routes_by_content_type {
      id: UT3
      text: "The multiplexer routes application/grpc to the tonic service and other content-types to the axum REST router."
      risk: medium
      verifymethod: test
    }
    test grpc_rest_shared_store_tests {
      type: functional
      verifies: grpc_rest_share_store
    }
    test grpc_body_normalization_tests {
      type: functional
      verifies: body_normalization
    }
    test grpc_mux_routing_tests {
      type: functional
      verifies: mux_routes_by_content_type
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-cloud-tasks-grpc-dispatch-smoke
    name: "Cloud Tasks gRPC client dispatches a task"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_tasks_grpc -- --nocapture"
    assertions:
      - "a generated google.cloud.tasks.v2 gRPC client (insecure channel to the emulator host:port) CreateQueue + CreateTask, and the emulator POSTs the task body to a local sink."
      - "the REST surface on the same port still works."
  - id: vat-cloud-scheduler-grpc-dispatch-smoke
    name: "Cloud Scheduler gRPC client fires a job on RunJob"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_scheduler_grpc -- --nocapture"
    assertions:
      - "a generated google.cloud.scheduler.v1 gRPC client CreateJob + RunJob, and the emulator POSTs the job httpTarget to a local sink."
  - id: vat-cloud-grpc-rest-coexist
    name: "REST tests still pass against the dual-protocol port"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_tasks --test vat_emulator_scheduler -- --nocapture"
    assertions:
      - "the existing REST e2e tests pass unchanged (REST + gRPC coexist on one port)."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/emulator/grpc_mux.rs
    action: create
    section: source
    impl_mode: hand-written
    reason: "Shared helper: multiplex a tonic gRPC service and an axum REST router on one TcpListener, routing by the application/grpc content-type, served via hyper-util auto Builder (h1 + h2)."
  - path: projects/vat/src/emulator/tasks.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Make the store/dispatcher protocol-agnostic; add the tonic CloudTasks service over it; serve gRPC + REST via grpc_mux."
  - path: projects/vat/src/emulator/scheduler.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Same for cloud-scheduler: protocol-agnostic store + tonic CloudScheduler service + multiplexed serve."
  - path: projects/vat/src/emulator/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Declare the grpc_mux module and the generated google.cloud.{tasks.v2,scheduler.v1} proto modules."
  - path: projects/vat/Cargo.toml
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add tower (Steer/Service plumbing) if needed for the multiplex; ensure tonic server feature is available under the emulator feature."
  - path: projects/vat/tests/vat_emulator_tasks_grpc.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "gRPC e2e: generated CloudTasks client → emulator → sink; REST coexists."
  - path: projects/vat/tests/vat_emulator_scheduler_grpc.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "gRPC e2e: generated CloudScheduler client → emulator → sink."
```
