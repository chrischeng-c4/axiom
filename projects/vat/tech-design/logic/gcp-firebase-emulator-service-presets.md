---
id: vat-gcp-firebase-emulator-service-presets
summary: Add GCP (firestore/pubsub/datastore/bigtable/spanner) and Firebase emulators as vat.toml service presets with native-preferred, docker-fallback resolution.
fill_sections: [logic, schema, config, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Extends the local agent test runner protocol's preset model with GCP and Firebase emulators so cloud-targeting agents can declare local emulators as run-scoped services."
---

# Vat GCP and Firebase Emulator Service Presets

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-gcp-firebase-emulator-service-presets-logic
entry: start
nodes:
  start: { kind: start, label: "dispatch preset service" }
  kind: { kind: decision, label: "firebase bundle or single-service emulator" }
  fb_json: { kind: decision, label: "firebase.json present" }
  fb_resolve: { kind: decision, label: "firebase-tools available else docker" }
  fb_start: { kind: process, label: "firebase emulators:start read firebase.json ports" }
  fb_export: { kind: process, label: "export each configured EMULATOR_HOST var" }
  resolve: { kind: process, label: "resolve_preset_runtime auto" }
  native_ok: { kind: decision, label: "binary and gcloud component installed" }
  docker_ok: { kind: decision, label: "docker daemon up" }
  unavailable: { kind: terminal, label: "emit service_runtime_unavailable bail no panic" }
  native_start: { kind: process, label: "gcloud beta emulators x start host-port" }
  docker_start: { kind: process, label: "docker run cloud-cli or spanner image" }
  export: { kind: process, label: "export EMULATOR_HOST var into runner" }
  ready: { kind: process, label: "tcp readiness on emulator port" }
  runner: { kind: process, label: "run runner as host process" }
  record: { kind: process, label: "record ServiceRunRecord evidence" }
  teardown: { kind: process, label: "stop service and docker rm -f if container" }
  done: { kind: terminal, label: "return exit code" }
edges:
  - { from: start, to: kind }
  - { from: kind, to: fb_json, label: "firebase" }
  - { from: kind, to: resolve, label: "gcp emulator" }
  - { from: fb_json, to: unavailable, label: "missing" }
  - { from: fb_json, to: fb_resolve, label: "present" }
  - { from: fb_resolve, to: unavailable, label: "neither" }
  - { from: fb_resolve, to: fb_start, label: "native or docker" }
  - { from: fb_start, to: fb_export }
  - { from: fb_export, to: ready }
  - { from: resolve, to: native_ok }
  - { from: native_ok, to: native_start, label: "yes" }
  - { from: native_ok, to: docker_ok, label: "no" }
  - { from: docker_ok, to: docker_start, label: "yes" }
  - { from: docker_ok, to: unavailable, label: "no" }
  - { from: native_start, to: export }
  - { from: docker_start, to: export }
  - { from: export, to: ready }
  - { from: ready, to: runner }
  - { from: runner, to: record }
  - { from: record, to: teardown }
  - { from: teardown, to: done }
---
flowchart TD
    start([dispatch preset service]) --> kind{firebase bundle or single-service emulator}
    kind -- firebase --> fb_json{firebase.json present}
    kind -- gcp emulator --> resolve[resolve_preset_runtime auto]
    fb_json -- missing --> unavailable([emit service_runtime_unavailable bail no panic])
    fb_json -- present --> fb_resolve{firebase-tools available else docker}
    fb_resolve -- neither --> unavailable
    fb_resolve -- native or docker --> fb_start[firebase emulators start read firebase.json ports]
    fb_start --> fb_export[export each configured EMULATOR_HOST var]
    fb_export --> ready[tcp readiness on emulator port]
    resolve --> native_ok{binary and gcloud component installed}
    native_ok -- yes --> native_start[gcloud beta emulators x start host-port]
    native_ok -- no --> docker_ok{docker daemon up}
    docker_ok -- yes --> docker_start[docker run cloud-cli or spanner image]
    docker_ok -- no --> unavailable
    native_start --> export[export EMULATOR_HOST var into runner]
    docker_start --> export
    export --> ready
    ready --> runner[run runner as host process]
    runner --> record[record ServiceRunRecord evidence]
    record --> teardown[stop service and docker rm -f if container]
    teardown --> done([return exit code])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-emulator-evidence.schema.json"
title: "Vat emulator service evidence"
type: object
description: "Service-evidence additions for GCP / Firebase emulator presets."
properties:
  emulator_preset:
    type: string
    enum: [firestore, pubsub, datastore, bigtable, spanner, firebase]
  prepare_mode:
    type: string
    enum: [direct_start, docker_run, firebase_emulators]
    description: "How the emulator service was provided: native binary, docker image, or the firebase suite."
  exported_env:
    type: array
    items: { type: string }
    description: >
      Host env var names exported to the runner, e.g. FIRESTORE_EMULATOR_HOST,
      PUBSUB_EMULATOR_HOST, DATASTORE_EMULATOR_HOST, BIGTABLE_EMULATOR_HOST,
      SPANNER_EMULATOR_HOST, and for the firebase bundle additionally
      FIREBASE_AUTH_EMULATOR_HOST, FIREBASE_DATABASE_EMULATOR_HOST,
      FIREBASE_STORAGE_EMULATOR_HOST.
additionalProperties: true
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-emulator.schema.json"
title: "vat.toml (emulator preset additions)"
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
          enum: [postgres, redis, nats, rabbitmq, mysql, mongo, firestore, pubsub, datastore, bigtable, spanner, firebase]
          description: >
            firestore/pubsub/datastore/bigtable/spanner are GCP emulators (native
            gcloud + Java + the gcloud component, with a docker-image fallback);
            firebase is a bundle that requires a firebase.json in the workspace and
            runs the Firebase Emulator Suite.
        runtime:
          type: string
          enum: [auto, native, docker]
          default: auto
          description: "auto prefers the native emulator and falls back to the docker image when the native binary or gcloud component is missing."
        export:
          type: object
          additionalProperties: { type: string }
          description: "Override the default *_EMULATOR_HOST export target name(s)."
      additionalProperties: true
additionalProperties: true
```
