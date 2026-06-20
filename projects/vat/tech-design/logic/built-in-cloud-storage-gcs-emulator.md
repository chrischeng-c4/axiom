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
