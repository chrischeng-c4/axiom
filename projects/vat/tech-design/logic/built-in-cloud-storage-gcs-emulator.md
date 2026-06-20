---
id: vat-built-in-cloud-storage-gcs-emulator
summary: Ship a built-in Rust Cloud Storage (GCS) emulator in vat — the GCS JSON API v1 subset over an in-memory store, driven by STORAGE_EMULATOR_HOST.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Adds a built-in Cloud Storage emulator (GCS JSON API v1, in-memory) so object-storage code runs locally through vat's run and evidence surface, driven by the standard STORAGE_EMULATOR_HOST with no gcloud/Docker."
---

# Vat Built-in Cloud Storage (GCS) Emulator

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-built-in-cloud-storage-gcs-emulator-logic
entry: start
nodes:
  start: { kind: start, label: "dispatch builtin preset cloud-storage" }
  spawn: { kind: process, label: "spawn self vat emulator cloud-storage host-port" }
  rt: { kind: process, label: "emulator builds tokio runtime; axum REST" }
  ready: { kind: process, label: "tcp readiness; export STORAGE_EMULATOR_HOST" }
  runner: { kind: process, label: "runner uses GCS SDK against STORAGE_EMULATOR_HOST" }
  route: { kind: decision, label: "request kind" }
  upload: { kind: process, label: "upload media or multipart or resumable; store bytes md5 size" }
  download: { kind: process, label: "download alt=media returns bytes" }
  meta: { kind: process, label: "get or list object metadata" }
  del: { kind: process, label: "delete object or bucket" }
  bucket: { kind: process, label: "bucket create get list; auto-create on upload" }
  store: { kind: process, label: "in-memory buckets and objects" }
  teardown: { kind: process, label: "stop service kills emulator child; blobs vanish" }
  done: { kind: terminal, label: "return exit code" }
edges:
  - { from: start, to: spawn }
  - { from: spawn, to: rt }
  - { from: rt, to: ready }
  - { from: ready, to: runner }
  - { from: runner, to: route }
  - { from: route, to: upload, label: "upload" }
  - { from: route, to: download, label: "download alt=media" }
  - { from: route, to: meta, label: "metadata or list" }
  - { from: route, to: del, label: "delete" }
  - { from: route, to: bucket, label: "bucket op" }
  - { from: upload, to: store }
  - { from: download, to: store }
  - { from: meta, to: store }
  - { from: del, to: store }
  - { from: bucket, to: store }
  - { from: store, to: route, label: "next request" }
  - { from: runner, to: teardown }
  - { from: teardown, to: done }
---
flowchart TD
    start([dispatch builtin preset cloud-storage]) --> spawn[spawn self vat emulator cloud-storage host-port]
    spawn --> rt[emulator builds tokio runtime; axum REST]
    rt --> ready[tcp readiness; export STORAGE_EMULATOR_HOST]
    ready --> runner[runner uses GCS SDK against STORAGE_EMULATOR_HOST]
    runner --> route{request kind}
    route -- upload --> upload[upload media or multipart or resumable; store bytes md5 size]
    route -- download alt=media --> download[download alt=media returns bytes]
    route -- metadata or list --> meta[get or list object metadata]
    route -- delete --> del[delete object or bucket]
    route -- bucket op --> bucket[bucket create get list; auto-create on upload]
    upload --> store[in-memory buckets and objects]
    download --> store
    meta --> store
    del --> store
    bucket --> store
    store --> route
    runner --> teardown[stop service kills emulator child; blobs vanish]
    teardown --> done([return exit code])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cloud-storage-evidence.schema.json"
title: "Vat Cloud Storage emulator evidence"
type: object
description: "Service-evidence shape and the GCS object resource for vat's built-in Cloud Storage emulator."
properties:
  preset:
    type: string
    enum: [cloud-storage]
  prepare_mode:
    type: string
    enum: [builtin_emulator]
  exported_env:
    type: array
    items: { type: string }
    description: "Host env var exported to the runner: STORAGE_EMULATOR_HOST (the var the GCS SDKs read)."
  object:
    type: object
    description: "A GCS object resource as returned by the JSON API."
    properties:
      kind: { type: string }
      bucket: { type: string }
      name: { type: string }
      size: { type: string }
      contentType: { type: string }
      generation: { type: string }
      md5Hash: { type: string }
      updated: { type: string }
    additionalProperties: true
additionalProperties: true
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-cloud-storage.schema.json"
title: "vat.toml (Cloud Storage preset addition)"
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
          enum: [postgres, redis, nats, rabbitmq, mysql, mongo, firestore, pubsub, datastore, bigtable, spanner, firebase, firebase-auth, cloud-tasks, cloud-scheduler, cloud-workflows, cloud-storage]
          description: >
            cloud-storage runs vat's built-in GCS emulator under runtime=auto
            (no gcloud/Java/Docker — Google ships no standalone GCS emulator). It
            exports STORAGE_EMULATOR_HOST, which the GCS client SDKs read, so the
            runner needs no code change. Blob state is in-memory and per-run.
            Built-in only: runtime must stay auto.
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
    usage: "vat emulator cloud-storage --host-port 127.0.0.1:<PORT>"
    behavior:
      - "Hidden verb: vat spawns itself as the service process for the cloud-storage preset."
      - "Serves the GCS JSON API v1 subset over an in-memory store: bucket create/get/list/delete (auto-create on upload), object upload (media/multipart/minimal resumable), download (alt=media), metadata, list (prefix), delete."
      - "Object names with slashes are percent-decoded; size and md5Hash are reported so SDK integrity checks pass. The runner reaches it through STORAGE_EMULATOR_HOST."
      - "Built without the emulator feature, the verb errors cleanly (no panic); an unknown object returns a structured 404."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-built-in-cloud-storage-gcs-emulator-unit-tests
---
requirementDiagram
    requirement preset_parses_builtin {
      id: UT1
      text: "ServicePreset round-trips cloud-storage, and it classifies as built-in and built-in-only (validate rejects an explicit runtime)."
      risk: medium
      verifymethod: test
    }
    requirement exports_storage_host {
      id: UT2
      text: "prepare_builtin_service exports STORAGE_EMULATOR_HOST and builds the self-exec emulator command."
      risk: medium
      verifymethod: test
    }
    requirement media_upload_download {
      id: UT3
      text: "A media upload then download (alt=media) returns the same bytes; metadata reports the right size and md5Hash."
      risk: high
      verifymethod: test
    }
    requirement multipart_and_list_delete {
      id: UT4
      text: "A multipart upload round-trips; list with a prefix returns the object; delete removes it (404 afterward)."
      risk: high
      verifymethod: test
    }
    requirement slashed_names {
      id: UT5
      text: "Object names containing '/' (percent-encoded in the path) upload and download correctly."
      risk: medium
      verifymethod: test
    }
    test config_cloud_storage_tests {
      type: functional
      verifies: preset_parses_builtin
    }
    test prepare_storage_builtin_tests {
      type: functional
      verifies: exports_storage_host
    }
    test storage_media_roundtrip_tests {
      type: functional
      verifies: media_upload_download
    }
    test storage_multipart_list_delete_tests {
      type: functional
      verifies: multipart_and_list_delete
    }
    test storage_slashed_name_tests {
      type: functional
      verifies: slashed_names
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-cloud-storage-roundtrip-smoke
    name: "Cloud Storage emulator uploads, downloads, lists, and deletes"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_storage -- --nocapture"
    assertions:
      - "a media upload then download (alt=media) returns the same bytes; a multipart upload round-trips; list with a prefix finds the object; delete removes it (404 after)."
      - "object names with slashes work; no gcloud / Java / Docker required; the emulator starts in well under a second."
  - id: vat-cloud-storage-preset-run-smoke
    name: "cloud-storage preset exports STORAGE_EMULATOR_HOST"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat cloud_storage_preset_exports_host -- --nocapture --ignored"
    assertions:
      - "a preset = cloud-storage vat.toml run exports STORAGE_EMULATOR_HOST and the runner uploads then downloads an object byte-identical; nothing remains after teardown."
  - id: vat-cloud-storage-lean-build
    name: "lean build still compiles"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo build -p vat --no-default-features"
    assertions:
      - "vat compiles without the emulator feature; the cloud-storage emulator verb then errors cleanly, never a panic."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md
    action: create
    section: changes
    impl_mode: hand-written
    reason: "Define the Cloud Storage emulator TD."
  - path: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md
    action: validate
    section: logic
    impl_mode: hand-written
    reason: "Record the JSON API + upload/download + in-memory store lifecycle."
  - path: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md
    action: validate
    section: schema
    impl_mode: hand-written
    reason: "Record the object evidence and exported env."
  - path: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md
    action: validate
    section: config
    impl_mode: hand-written
    reason: "Record the cloud-storage builtin-only preset."
  - path: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md
    action: validate
    section: cli
    impl_mode: hand-written
    reason: "Record the vat emulator cloud-storage kind."
  - path: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md
    action: validate
    section: unit-test
    impl_mode: hand-written
    reason: "Record preset, export, media/multipart, list/delete, and slashed-name coverage."
  - path: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md
    action: validate
    section: e2e-test
    impl_mode: hand-written
    reason: "Record storage-roundtrip, preset-run, and lean-build coverage."
  - path: projects/vat/Cargo.toml
    action: modify
    section: config
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#config"
    summary: "Add percent-encoding to the emulator feature (object-name decode) and the integration test entry."
  - path: projects/vat/src/emulator/storage.rs
    action: add
    section: logic
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#logic"
    summary: "GCS JSON API v1 emulator: axum bucket/object routes (upload media/multipart/resumable, download alt=media, metadata, list, delete) over an in-memory store with md5/size."
  - path: projects/vat/src/emulator/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#logic"
    summary: "Register the storage module and the CloudStorage serve arm."
  - path: projects/vat/src/cli.rs
    action: modify
    section: cli
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#cli"
    summary: "Add the CloudStorage EmulatorKind arm."
  - path: projects/vat/src/commands/emulator.rs
    action: modify
    section: cli
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#cli"
    summary: "Map the CloudStorage EmulatorKind to the emulator serve dispatch."
  - path: projects/vat/src/config.rs
    action: modify
    section: config
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#config"
    summary: "Add the CloudStorage ServicePreset and include it in is_emulator/is_builtin/is_builtin_only."
  - path: projects/vat/src/commands/run.rs
    action: modify
    section: logic
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#logic"
    summary: "Extend builtin_emulator_info and service_preset_name and fill the new exhaustive preset arm."
  - path: projects/vat/src/commands/llm.rs
    action: modify
    section: config
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#config"
    summary: "Document the built-in Cloud Storage emulator."
  - path: projects/vat/README.md
    action: modify
    section: config
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#config"
    summary: "Document the built-in Cloud Storage emulator preset."
  - path: projects/vat/tests
    action: modify
    section: unit-test
    impl_mode: hand-written
    refs:
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#unit-test"
      - "projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#e2e-test"
    summary: "Add tests/vat_emulator_storage.rs integration test."
```
