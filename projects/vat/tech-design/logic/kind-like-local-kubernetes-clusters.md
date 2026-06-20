---
id: vat-kind-like-local-kubernetes-clusters
summary: Add kind-like local Kubernetes clusters to vat as a run-scoped vat.toml service and standalone vat cluster commands, with auto backend selection across kind k3d minikube.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Extends the local agent test runner protocol with a fourth service kind (cluster) and a standalone vat cluster lifecycle so agents can provision local Kubernetes clusters as run-scoped dependencies."
---

# Vat Kind-Like Local Kubernetes Clusters

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-kind-like-local-kubernetes-clusters-logic
entry: start
nodes:
  start: { kind: start, label: "dispatch cluster service or vat cluster command" }
  mode: { kind: decision, label: "run scoped service or standalone command" }
  resolve: { kind: process, label: "resolve_backend auto kind k3d minikube" }
  backend_ok: { kind: decision, label: "usable backend and docker daemon up" }
  unavailable: { kind: terminal, label: "emit cluster_backend_unavailable and bail no panic" }
  create: { kind: process, label: "create cluster with isolated kubeconfig" }
  created: { kind: decision, label: "ready before create timeout" }
  create_timeout: { kind: terminal, label: "kill and delete emit cluster_create_timeout" }
  ready: { kind: process, label: "ready probe kubectl get nodes" }
  export_env: { kind: process, label: "export KUBECONFIG into runner env" }
  runner: { kind: process, label: "run runner as host process" }
  record: { kind: process, label: "record ClusterRunRecord evidence" }
  keep: { kind: decision, label: "keep policy should remove" }
  delete: { kind: process, label: "driver delete cluster" }
  retain: { kind: process, label: "retain cluster for kubectl diagnosis" }
  sa_dispatch: { kind: decision, label: "vat cluster verb" }
  sa_create: { kind: process, label: "create and write registry entry" }
  sa_list: { kind: process, label: "list registry reconcile against driver list" }
  sa_kubeconfig: { kind: process, label: "print isolated kubeconfig path" }
  sa_delete: { kind: process, label: "driver delete then remove registry dir" }
  done: { kind: terminal, label: "return exit code" }
edges:
  - { from: start, to: mode }
  - { from: mode, to: resolve, label: "run scoped" }
  - { from: mode, to: sa_dispatch, label: "standalone" }
  - { from: resolve, to: backend_ok }
  - { from: backend_ok, to: unavailable, label: "none" }
  - { from: backend_ok, to: create, label: "ok" }
  - { from: create, to: created }
  - { from: created, to: create_timeout, label: "timeout" }
  - { from: created, to: ready, label: "ready" }
  - { from: ready, to: export_env }
  - { from: export_env, to: runner }
  - { from: runner, to: record }
  - { from: record, to: keep }
  - { from: keep, to: delete, label: "should remove" }
  - { from: keep, to: retain, label: "keep" }
  - { from: delete, to: done }
  - { from: retain, to: done }
  - { from: sa_dispatch, to: sa_create, label: "create" }
  - { from: sa_dispatch, to: sa_list, label: "ls" }
  - { from: sa_dispatch, to: sa_kubeconfig, label: "kubeconfig" }
  - { from: sa_dispatch, to: sa_delete, label: "delete" }
  - { from: sa_create, to: done }
  - { from: sa_list, to: done }
  - { from: sa_kubeconfig, to: done }
  - { from: sa_delete, to: done }
---
flowchart TD
    start([dispatch cluster service or vat cluster command]) --> mode{run scoped service or standalone command}
    mode -- run scoped --> resolve[resolve_backend auto kind k3d minikube]
    mode -- standalone --> sa_dispatch{vat cluster verb}
    resolve --> backend_ok{usable backend and docker daemon up}
    backend_ok -- none --> unavailable([emit cluster_backend_unavailable and bail no panic])
    backend_ok -- ok --> create[create cluster with isolated kubeconfig]
    create --> created{ready before create timeout}
    created -- timeout --> create_timeout([kill and delete emit cluster_create_timeout])
    created -- ready --> ready[ready probe kubectl get nodes]
    ready --> export_env[export KUBECONFIG into runner env]
    export_env --> runner[run runner as host process]
    runner --> record[record ClusterRunRecord evidence]
    record --> keep{keep policy should remove}
    keep -- should remove --> delete[driver delete cluster]
    keep -- keep --> retain[retain cluster for kubectl diagnosis]
    delete --> done([return exit code])
    retain --> done
    sa_dispatch -- create --> sa_create[create and write registry entry]
    sa_dispatch -- ls --> sa_list[list registry reconcile against driver list]
    sa_dispatch -- kubeconfig --> sa_kubeconfig[print isolated kubeconfig path]
    sa_dispatch -- delete --> sa_delete[driver delete then remove registry dir]
    sa_create --> done
    sa_list --> done
    sa_kubeconfig --> done
    sa_delete --> done
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cluster-evidence.schema.json"
title: "Vat cluster evidence"
type: object
description: "Cluster additions to vat run evidence and the standalone cluster registry."
properties:
  service_cluster:
    description: "ClusterRunRecord attached to a service evidence item when the service is a local Kubernetes cluster. Null for non-cluster services."
    type: [object, "null"]
    required: [backend, name, kubeconfig, node_count]
    properties:
      backend: { type: string, enum: [kind, k3d, minikube] }
      name: { type: string }
      kubeconfig: { type: string }
      node_count: { type: integer }
      ready_ms: { type: [integer, "null"] }
    additionalProperties: false
  cluster_registry_record:
    description: "Standalone cluster registry entry under .vat/clusters/<name>/cluster.json."
    type: object
    required: [backend, name, kubeconfig, node_count, created_at]
    properties:
      backend: { type: string, enum: [kind, k3d, minikube] }
      name: { type: string }
      kubeconfig: { type: string }
      node_count: { type: integer }
      created_at: { type: string }
    additionalProperties: false
additionalProperties: false
```
