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

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-cluster.schema.json"
title: "vat.toml (cluster service additions)"
type: object
required: [version, runners]
properties:
  version:
    type: integer
    const: 1
  services:
    type: array
    items:
      type: object
      required: [id]
      description: >
        A run-scoped dependency service backed by exactly one of: cmd (an
        explicit native command), preset (a built-in service whose runtime
        decides native-binary vs official Docker image), image (a Docker-only
        service), or cluster (an ephemeral local Kubernetes cluster). The runner
        that requires the service is always a host process — only the service
        may be a container or cluster — so the host GPU story is unaffected.
      properties:
        id: { type: string }
        requires:
          type: array
          items: { type: string }
        cmd:
          type: array
          items: { type: string }
          minItems: 1
        preset: { type: string, enum: [postgres, redis, nats, rabbitmq, mysql, mongo] }
        image: { type: string }
        container_port: { type: integer }
        image_env:
          type: object
          additionalProperties: { type: string }
        runtime: { type: string, enum: [auto, native, docker], default: auto }
        cluster:
          type: string
          enum: [auto, kind, k3d, minikube]
          description: >
            Declares this service as an ephemeral local Kubernetes cluster.
            Mutually exclusive with cmd/preset/image. auto resolves to the first
            installed of kind -> k3d -> minikube whose Docker daemon is reachable.
        k8s_version:
          type: string
          description: "Optional Kubernetes version for the cluster node image, e.g. 1.30."
        nodes:
          type: integer
          minimum: 1
          maximum: 9
          description: "Cluster node count. Defaults to a single node."
        version: { type: string }
        port:
          oneOf:
            - { type: string, const: auto }
            - { type: integer }
        seed:
          type: array
          items: { type: string }
        export:
          type: object
          additionalProperties: { type: string }
          description: >
            For a cluster service the token {kubeconfig} resolves to the isolated
            kubeconfig path; KUBECONFIG and VAT_SERVICE_<ID>_KUBECONFIG are always
            exported to the runner regardless of explicit export entries.
        ready_http: { type: string }
        timeout_s: { type: integer, default: 60 }
      additionalProperties: false
  runners:
    type: array
    items:
      type: object
      required: [id, cmd]
      properties:
        id: { type: string }
        requires:
          type: array
          items: { type: string }
        cmd:
          type: array
          items: { type: string }
          minItems: 1
        timeout_s: { type: integer }
        artifacts:
          type: array
          items: { type: string }
      additionalProperties: false
additionalProperties: false
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat cluster create
    usage: "vat cluster create [--name <name>] [--backend auto|kind|k3d|minikube] [--k8s-version <v>] [--nodes <n>] [--json]"
    behavior:
      - "Resolves the backend (auto prefers the first installed of kind, k3d, minikube whose Docker daemon is reachable)."
      - "Creates a standalone cluster with an isolated kubeconfig under <repo>/.vat/clusters/<name>/."
      - "Auto-generates a unique name when --name is omitted; rejects a name that collides with the registry or the backend."
      - "Fails with a structured cluster_backend_unavailable error and a non-zero exit when no backend is usable."
  - name: vat cluster ls
    usage: "vat cluster ls [--json]"
    behavior:
      - "Lists vat-managed clusters from the registry."
      - "Reconciles against the backend cluster list and marks entries missing from the backend as stale."
  - name: vat cluster kubeconfig
    usage: "vat cluster kubeconfig <name> [--json]"
    behavior:
      - "Prints the isolated kubeconfig path for the named cluster (or its contents in --json form)."
  - name: vat cluster delete
    usage: "vat cluster delete <name> [--json]"
    behavior:
      - "Deletes the cluster via its backend driver, then removes the registry directory."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-kind-like-local-kubernetes-clusters-unit-tests
---
requirementDiagram
    requirement cluster_service_exclusivity {
      id: UT1
      text: "A cluster service is mutually exclusive with cmd/preset/image; validation rejects any combination and rejects an empty backing."
      risk: high
      verifymethod: test
    }
    requirement cluster_backend_enum {
      id: UT2
      text: "ClusterBackend round-trips auto/kind/k3d/minikube via serde with the k3d/minikube tokens preserved."
      risk: medium
      verifymethod: test
    }
    requirement cluster_knob_rejection {
      id: UT3
      text: "A cluster service rejects container_port/image_env/seed and rejects nodes outside 1..9."
      risk: medium
      verifymethod: test
    }
    requirement cluster_name_sanitized {
      id: UT4
      text: "The run-scoped cluster name builder produces a collision-resistant, backend-safe name from vat id and service id."
      risk: medium
      verifymethod: test
    }
    requirement backend_unavailable_no_panic {
      id: UT5
      text: "resolve_backend with no cluster backend on PATH returns a structured cluster_backend_unavailable error and never panics."
      risk: high
      verifymethod: test
    }
    test config_cluster_validation_tests {
      type: functional
      verifies: cluster_service_exclusivity
    }
    test config_cluster_knob_tests {
      type: functional
      verifies: cluster_knob_rejection
    }
    test cluster_backend_serde_tests {
      type: functional
      verifies: cluster_backend_enum
    }
    test cluster_name_tests {
      type: functional
      verifies: cluster_name_sanitized
    }
    test resolve_backend_unavailable_tests {
      type: functional
      verifies: backend_unavailable_no_panic
    }
```
