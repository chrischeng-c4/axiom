---
id: keep-operator-adoption
summary: >
  Keep adopts the shared libs/operator scaffold + render toolkit: a KeepSpec CRD
  that flattens operator::ClusterSpec, a ManagedService impl that renders the
  sharded HA StatefulSet (with the downward-API env raft-host consumes) plus its
  ServiceAccount/ConfigMap/Services/PDB, and a `keep k8s crd/operator/instance`
  CLI gated behind an `operator` feature. The generated CRD is normalized to be
  Kubernetes-OpenAPI compatible and the binary installs a process-level rustls
  crypto provider at startup, replacing hand-maintained static HA Kustomize YAML
  with an operator-driven CR path.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-operator-adoption-contract
entry: cli_start
nodes:
  cli_start: { kind: start, label: "keep main installs the rustls crypto provider then parses the CLI" }
  k8s_dispatch: { kind: process, label: "keep k8s dispatches crd / operator / instance subcommands" }
  crd_render: { kind: process, label: "crd render serializes Keep::crd and normalizes uint32/uint64 to integer with minimum 0" }
  operator_render: { kind: process, label: "operator render emits keep-system namespace RBAC and controller Deployment YAML" }
  operator_run: { kind: process, label: "operator run drives operator::run::Keep reconcile loop behind the operator feature" }
  instance_render: { kind: process, label: "instance render emits a keep.dev v1alpha1 Keep custom resource per profile" }
  reconcile: { kind: process, label: "ManagedService::render builds keep child objects" }
  sharded_sts: { kind: process, label: "operator::render::sharded_statefulset injects the POD_NAME POD_NAMESPACE SHARD_COUNT REPLICAS_PER_SHARD VOTER_COUNT and headless env" }
  children: { kind: process, label: "service account configmap headless and client service pdb plus the hardened durable StatefulSet" }
  readiness: { kind: process, label: "readiness_targets is the StatefulSet and status_patch derives phase from readyReplicas" }
  apply: { kind: process, label: "controller server-side-applies children and writes status when it holds the lease" }
  stop: { kind: terminal, label: "operator converges the Keep custom resource to the sharded HA topology" }
edges:
  - { from: cli_start, to: k8s_dispatch }
  - { from: k8s_dispatch, to: crd_render }
  - { from: k8s_dispatch, to: operator_render }
  - { from: k8s_dispatch, to: operator_run }
  - { from: k8s_dispatch, to: instance_render }
  - { from: operator_run, to: reconcile }
  - { from: reconcile, to: sharded_sts }
  - { from: sharded_sts, to: children }
  - { from: children, to: readiness }
  - { from: readiness, to: apply }
  - { from: apply, to: stop }
---
flowchart TD
    cli_start([keep main installs rustls crypto provider then parses CLI]) --> k8s_dispatch[keep k8s dispatches crd operator instance]
    k8s_dispatch --> crd_render[crd render serializes Keep crd and normalizes uint to integer with minimum 0]
    k8s_dispatch --> operator_render[operator render emits keep-system namespace RBAC and Deployment YAML]
    k8s_dispatch --> operator_run[operator run drives operator run Keep reconcile loop behind operator feature]
    k8s_dispatch --> instance_render[instance render emits keep.dev v1alpha1 Keep custom resource per profile]
    operator_run --> reconcile[ManagedService render builds keep child objects]
    reconcile --> sharded_sts[sharded_statefulset injects POD_NAME POD_NAMESPACE SHARD_COUNT REPLICAS_PER_SHARD VOTER_COUNT headless env]
    sharded_sts --> children[service account configmap headless and client service pdb plus hardened StatefulSet]
    children --> readiness[readiness_targets StatefulSet and status_patch phase from readyReplicas]
    readiness --> apply[controller server-side-applies children and writes status when lease holder]
    apply --> stop([operator converges Keep custom resource to sharded HA topology])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-operator-adoption-tests
requirements:
  crd_flatten:
    id: R1
    text: "KeepSpec flattens operator::ClusterSpec so the CRD schema carries image, imagePullPolicy, shardCount, replicasPerShard, voterCount, and resources."
    kind: behavior
    risk: medium
    verify: test
  render_downward_api:
    id: R2
    text: "ManagedService for Keep renders the sharded StatefulSet whose container env carries the downward-API quartet plus headless service env raft-host consumes, with replicas = shardCount * replicasPerShard, and status_patch derives phase from readyReplicas."
    kind: behavior
    risk: high
    verify: test
  k8s_cli:
    id: R3
    text: "keep k8s crd render, operator render and run, and instance render --profile parse and dispatch, with operator run gated behind the operator feature."
    kind: behavior
    risk: medium
    verify: test
  crd_openapi_compat:
    id: R4
    text: "The generated CRD is Kubernetes-OpenAPI compatible: no format uint32 or uint64 remains and unsigned integer fields keep minimum 0."
    kind: behavior
    risk: high
    verify: test
  rustls_provider:
    id: R5
    text: "The process-level rustls crypto provider install is idempotent and safe to call before command parsing."
    kind: behavior
    risk: medium
    verify: test
elements:
  operator_tests:
    kind: test
    path: projects/keep/tests/operator.rs
  keep_cli:
    kind: test
    path: projects/keep/src/bin/keep.rs
relations:
  - { from: operator_tests, verifies: crd_flatten }
  - { from: operator_tests, verifies: render_downward_api }
  - { from: operator_tests, verifies: crd_openapi_compat }
  - { from: operator_tests, verifies: rustls_provider }
  - { from: keep_cli, verifies: k8s_cli }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "KeepSpec flattens ClusterSpec"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "render downward-api statefulset"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "keep k8s cli verbs"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "crd openapi compatible"
      risk: high
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: "rustls crypto provider install"
      risk: medium
      verifymethod: test
    }
    element operator_tests {
      type: "rs/#[test]"
    }
    element keep_cli {
      type: "rs/#[test]"
    }
    operator_tests - verifies -> R1
    operator_tests - verifies -> R2
    operator_tests - verifies -> R4
    operator_tests - verifies -> R5
    keep_cli - verifies -> R3
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the operator feature and its optional deps (kube, k8s-openapi, schemars, operator, serde_yaml, rustls); wire a private rustls-provider feature enabled by operator/raft/client/self-update/issue."
  - path: projects/keep/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Declare pub mod tls (always) and #[cfg(feature = operator)] pub mod operator."
  - path: projects/keep/src/tls.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "install_default_crypto_provider: once-guarded aws_lc_rs rustls provider install, compiled real behind the rustls-provider feature and a no-op otherwise."
  - path: projects/keep/src/operator/mod.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "operator module root: crd_yaml() that serializes the Keep CRD and normalizes uint formats, plus re-exports of Keep/KeepSpec/KeepStatus and run."
  - path: projects/keep/src/operator/crd.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "KeepSpec (#[serde(flatten)] cluster: operator::ClusterSpec + keep engineShards/logLevel/graceSecs/storage/storageClass), the Keep CustomResource (keep.dev/v1alpha1), and KeepStatus."
  - path: projects/keep/src/operator/render.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "render(&Keep) -> Vec<Value>: service_account, keep ConfigMap, sharded_statefulset (downward-API env + /data PVC) hardened with probes/securityContext/tmp, headless + client Service, and PDB via operator::render helpers."
  - path: projects/keep/src/operator/reconcile.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "impl operator::ManagedService for Keep (MANAGER, render, readiness_targets = StatefulSet, status_patch) and pub async fn run() = operator::run::<Keep>()."
  - path: projects/keep/src/bin/keep.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Call keep::tls::install_default_crypto_provider() before Cli::parse(); add the K8s command tree (crd render, operator render|run, instance render --profile dev|staging|prod|template) with operator run gated behind the operator feature; add CLI parse tests for the new verbs."
  - path: projects/keep/k8s/operator/crd.yaml
    action: create
    section: logic
    impl_mode: hand-written
    description: "Generated Keep CustomResourceDefinition, checked in so `keep k8s crd render` works without the operator feature (include_str fallback)."
  - path: projects/keep/k8s/operator/rbac.yaml
    action: create
    section: logic
    impl_mode: hand-written
    description: "keep-system Namespace, operator ServiceAccount, ClusterRole (keeps + status, statefulsets, services/configmaps/serviceaccounts, poddisruptionbudgets, leases) and ClusterRoleBinding."
  - path: projects/keep/k8s/operator/deployment.yaml
    action: create
    section: logic
    impl_mode: hand-written
    description: "Operator controller Deployment running `keep k8s operator run` from the keep image with downward-API POD_NAME/POD_NAMESPACE for leader election."
  - path: projects/keep/k8s/operator/kustomization.yaml
    action: create
    section: logic
    impl_mode: hand-written
    description: "Kustomization installing crd.yaml + rbac.yaml + deployment.yaml for the operator control plane."
  - path: projects/keep/tests/operator.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "operator-feature integration tests: R1 CRD flatten schema, R2 render downward-API StatefulSet + replicas + status phase, R4 no uint32/uint64 with minimum 0, R5 idempotent rustls provider install."
```
