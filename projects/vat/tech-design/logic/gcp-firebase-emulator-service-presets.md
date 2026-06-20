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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-gcp-firebase-emulator-service-presets-unit-tests
---
requirementDiagram
    requirement emulator_preset_parses {
      id: UT1
      text: "ServicePreset round-trips firestore/pubsub/datastore/bigtable/spanner/firebase via serde from vat.toml preset values."
      risk: medium
      verifymethod: test
    }
    requirement firebase_requires_json {
      id: UT2
      text: "validate() rejects a firebase preset service when the workspace has no firebase.json."
      risk: high
      verifymethod: test
    }
    requirement native_available_checks_component {
      id: UT3
      text: "The native-availability decision treats a GCP emulator as native-unavailable when its gcloud component is not installed, even if gcloud is on PATH."
      risk: high
      verifymethod: test
    }
    requirement emulator_exports_host_var {
      id: UT4
      text: "preset_exports yields the correct EMULATOR_HOST variable for each GCP emulator preset, honoring export overrides."
      risk: medium
      verifymethod: test
    }
    requirement emulator_unavailable_no_panic {
      id: UT5
      text: "An emulator preset with neither native tooling nor docker emits a structured unavailable error and never panics."
      risk: high
      verifymethod: test
    }
    test config_emulator_preset_tests {
      type: functional
      verifies: emulator_preset_parses
    }
    test config_firebase_requires_json_tests {
      type: functional
      verifies: firebase_requires_json
    }
    test preset_native_available_tests {
      type: functional
      verifies: native_available_checks_component
    }
    test preset_exports_emulator_tests {
      type: functional
      verifies: emulator_exports_host_var
    }
    test emulator_unavailable_jsonl_tests {
      type: functional
      verifies: emulator_unavailable_no_panic
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-emulator-unavailable-smoke
    name: "emulator preset reports structured unavailable error"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat gcloud_emulator_unavailable_reports_jsonl_error -- --nocapture"
    assertions:
      - "a firestore preset with an empty PATH (no gcloud, no docker) emits a structured service_runtime_unavailable JSONL error and a non-zero exit."
      - "vat never panics on the unavailable path."
  - id: vat-firestore-native-smoke
    name: "native Firestore emulator exports FIRESTORE_EMULATOR_HOST"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat firestore_emulator_exports_host -- --nocapture --ignored"
    assertions:
      - "with gcloud + Java + the firestore component, vat starts the emulator, the runner sees FIRESTORE_EMULATOR_HOST, and vat state shows the service ready with the var in exported_env."
      - "the test skips gracefully when the native firestore emulator is unavailable."
  - id: vat-pubsub-docker-smoke
    name: "Pub/Sub emulator falls back to docker"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat pubsub_emulator_docker_fallback -- --nocapture --ignored"
    assertions:
      - "without the pubsub gcloud component, runtime=auto resolves to docker, exports PUBSUB_EMULATOR_HOST, and removes the container at teardown."
      - "the test skips gracefully when docker is unavailable."
  - id: vat-firebase-bundle-smoke
    name: "Firebase bundle exports configured emulator hosts"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat firebase_bundle_exports_hosts -- --nocapture --ignored"
    assertions:
      - "a firebase preset with a firebase.json starts the suite and exports the configured *_EMULATOR_HOST vars."
      - "the test skips gracefully when firebase-tools and docker are both unavailable."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md
    action: create
    section: changes
    impl_mode: hand-written
    reason: "Define the GCP/Firebase emulator preset TD."
  - path: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md
    action: validate
    section: logic
    impl_mode: hand-written
    reason: "Record the native/docker/firebase emulator resolution and lifecycle."
  - path: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md
    action: validate
    section: schema
    impl_mode: hand-written
    reason: "Record the emulator service evidence and exported env shape."
  - path: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md
    action: validate
    section: config
    impl_mode: hand-written
    reason: "Record the preset enum additions and firebase.json requirement."
  - path: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md
    action: validate
    section: unit-test
    impl_mode: hand-written
    reason: "Record preset parsing, firebase validation, native-availability, export, and unavailable coverage."
  - path: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md
    action: validate
    section: e2e-test
    impl_mode: hand-written
    reason: "Record firestore-native, pubsub-docker, firebase-bundle, and unavailable smoke coverage."
  - path: projects/vat/src/config.rs
    action: modify
    section: config
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config"
    summary: "Add firestore/pubsub/datastore/bigtable/spanner/firebase ServicePreset variants and the firebase.json validation."
  - path: projects/vat/src/commands/run.rs
    action: modify
    section: logic
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#logic"
      - "projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config"
    summary: "Extend the preset tables (binaries/command/readiness/exports/image/port/env) for the five GCP emulators, add the gcloud-component-aware preset_native_available used by resolve_preset_runtime, and add prepare_firebase_service for the firebase bundle."
  - path: projects/vat/src/commands/llm.rs
    action: modify
    section: config
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config"
    summary: "Document the emulator presets, their *_EMULATOR_HOST exports, and the firebase bundle in the agent usage guide."
  - path: projects/vat/README.md
    action: modify
    section: config
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config"
    summary: "Document the GCP/Firebase emulator presets and a firebase bundle example."
  - path: projects/vat/tests
    action: modify
    section: unit-test
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#unit-test"
      - "projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#e2e-test"
    summary: "Add emulator config unit tests and gated integration smokes (firestore native, pubsub docker, firebase bundle, unavailable)."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] The Mermaid Plus flow cleanly splits the firebase bundle path (firebase.json gating, native-or-docker resolve) from the single-service GCP path (component-aware native vs docker), with a shared export/readiness/runner/teardown tail and an explicit unavailable terminal — no panic.
- [schema] The emulator evidence shape (preset enum, prepare_mode native/docker/firebase, exported_env host vars) is precise and matches how ServiceRunRecord surfaces through vat state.
- [config] The preset enum extension keeps the existing six and adds the five GCP emulators plus the firebase bundle; the firebase.json requirement and runtime=auto semantics are unambiguous.
- [unit-test] UT1..UT5 cover serde parsing, firebase.json validation, the gcloud-component native-availability decision, export var derivation, and the no-panic unavailable path — all deterministic and Docker-free.
- [e2e-test] The smoke set pairs an always-run unavailable assertion with gated native-firestore, docker-pubsub, and firebase-bundle lifecycles that skip without their tooling; commands and assertions are concrete.
- [changes] The source change list is bounded and scoped: config.rs, run.rs, llm.rs, README, and tests, each ref-linked to its driving section, with no unrelated scope.
