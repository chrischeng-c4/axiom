---
id: vat-built-in-rust-emulators-pub-sub-firebase-auth
summary: Ship built-in Rust local-test emulators in vat (Pub/Sub gRPC + Firebase Auth REST) run as a hidden vat emulator subcommand, preferred over gcloud/docker for emulator presets.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Extends the local agent test runner protocol with vat's own in-process Rust emulators (Pub/Sub, Firebase Auth) so cloud-targeting agents get instant, dependency-free local emulation, preferred over the external gcloud/docker paths."
---

# Vat Built-in Rust Emulators (Pub/Sub + Firebase Auth)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-built-in-rust-emulators-pub-sub-firebase-auth-logic
entry: start
nodes:
  start: { kind: start, label: "dispatch preset service" }
  builtin_q: { kind: decision, label: "preset has built-in emulator and runtime auto" }
  resolve_legacy: { kind: process, label: "resolve_preset_runtime native or docker (gcloud fallback)" }
  spawn: { kind: process, label: "spawn self vat emulator name host-port" }
  rt: { kind: process, label: "emulator subcommand builds tokio runtime" }
  kind_q: { kind: decision, label: "pubsub gRPC or firebase-auth REST" }
  pubsub_srv: { kind: process, label: "tonic Publisher and Subscriber in-memory" }
  auth_srv: { kind: process, label: "axum identity-toolkit REST in-memory" }
  ready: { kind: process, label: "tcp readiness on host-port" }
  export: { kind: process, label: "export EMULATOR_HOST var into runner" }
  runner: { kind: process, label: "run runner as host process" }
  teardown: { kind: process, label: "stop service kills emulator child" }
  done: { kind: terminal, label: "return exit code" }
edges:
  - { from: start, to: builtin_q }
  - { from: builtin_q, to: spawn, label: "yes" }
  - { from: builtin_q, to: resolve_legacy, label: "no" }
  - { from: resolve_legacy, to: export }
  - { from: spawn, to: rt }
  - { from: rt, to: kind_q }
  - { from: kind_q, to: pubsub_srv, label: "pubsub" }
  - { from: kind_q, to: auth_srv, label: "firebase-auth" }
  - { from: pubsub_srv, to: ready }
  - { from: auth_srv, to: ready }
  - { from: ready, to: export }
  - { from: export, to: runner }
  - { from: runner, to: teardown }
  - { from: teardown, to: done }
---
flowchart TD
    start([dispatch preset service]) --> builtin_q{preset has built-in emulator and runtime auto}
    builtin_q -- yes --> spawn[spawn self vat emulator name host-port]
    builtin_q -- no --> resolve_legacy[resolve_preset_runtime native or docker gcloud fallback]
    resolve_legacy --> export[export EMULATOR_HOST var into runner]
    spawn --> rt[emulator subcommand builds tokio runtime]
    rt --> kind_q{pubsub gRPC or firebase-auth REST}
    kind_q -- pubsub --> pubsub_srv[tonic Publisher and Subscriber in-memory]
    kind_q -- firebase-auth --> auth_srv[axum identity-toolkit REST in-memory]
    pubsub_srv --> ready[tcp readiness on host-port]
    auth_srv --> ready
    ready --> export
    export --> runner[run runner as host process]
    runner --> teardown[stop service kills emulator child]
    teardown --> done([return exit code])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-builtin-emulator-evidence.schema.json"
title: "Vat built-in emulator evidence"
type: object
description: "Service-evidence shape for vat's built-in Rust emulators."
properties:
  preset:
    type: string
    enum: [pubsub, firebase-auth]
  prepare_mode:
    type: string
    enum: [builtin_emulator, direct_start, docker_run]
    description: "builtin_emulator = vat's own in-process emulator (the vat emulator subcommand)."
  exported_env:
    type: array
    items: { type: string }
    description: "Host env var exported to the runner: PUBSUB_EMULATOR_HOST or FIREBASE_AUTH_EMULATOR_HOST."
  emulator_command:
    type: array
    items: { type: string }
    description: "Self-exec invocation: [<vat exe>, emulator, <kind>, --host-port, 127.0.0.1:<port>]."
additionalProperties: true
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-builtin-emulator.schema.json"
title: "vat.toml (built-in emulator preset additions)"
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
          enum: [postgres, redis, nats, rabbitmq, mysql, mongo, firestore, pubsub, datastore, bigtable, spanner, firebase, firebase-auth]
          description: >
            pubsub and firebase-auth have a built-in Rust emulator: under
            runtime=auto vat runs its own in-process emulator (no gcloud/Java/
            firebase-tools/Docker). pubsub keeps runtime=native (gcloud) and
            runtime=docker (image) as fidelity fallback; firebase-auth is
            built-in only (runtime must be auto).
        runtime:
          type: string
          enum: [auto, native, docker]
          default: auto
          description: "auto prefers the built-in emulator when the preset has one."
        export:
          type: object
          additionalProperties: { type: string }
      additionalProperties: true
additionalProperties: true
```
