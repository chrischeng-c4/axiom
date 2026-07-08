---
id: preview-base-workload-discovery
summary: >
  Add a local base workload discovery path for Preview. The CLI reads a base
  namespace Deployment/Service through kubectl, normalizes the cloneable
  workload contract, excludes runtime identity, and lets render embed that
  discovered contract into the workload clone plan before kind/GKE validation.
capability_refs:
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    gap: "base-workload-discovery"
    claim: "base-workload-discovery"
    coverage: partial
    rationale: >
      Work item #1108 extends the base namespace clone model from manual
      configuration to local discovery against a kind/base namespace fixture.
fill_sections: [logic, schema, cli, unit-test, e2e-test, changes]
---

# TD: Preview Base Workload Discovery

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: preview-base-workload-discovery-flow
entry: discover_base
nodes:
  discover_base: { kind: start, label: "preview discover-base --namespace uat-base --app checkout" }
  kubectl_deployment: { kind: process, label: "kubectl get deployment checkout -n uat-base -o json" }
  kubectl_service: { kind: process, label: "kubectl get service checkout -n uat-base -o json" }
  normalize: { kind: process, label: "normalize Deployment/Service into BaseWorkloadContract" }
  exclude_runtime: { kind: process, label: "drop runtime identity: uid resourceVersion generation managedFields status clusterIP nodePort loadBalancer ownerReferences secrets" }
  write_contract: { kind: terminal, label: "emit base contract JSON to stdout or --out" }
  render_contract: { kind: process, label: "preview render --base-contract embeds discoveredBase into plans/workload-clone.json" }
  kind_gate: { kind: terminal, label: "kind lifecycle builds base fixture, runs discover-base, renders preview, applies and rolls out" }
edges:
  - { from: discover_base, to: kubectl_deployment }
  - { from: discover_base, to: kubectl_service }
  - { from: kubectl_deployment, to: normalize }
  - { from: kubectl_service, to: normalize }
  - { from: normalize, to: exclude_runtime }
  - { from: exclude_runtime, to: write_contract }
  - { from: write_contract, to: render_contract }
  - { from: render_contract, to: kind_gate }
---
flowchart TD
    discover_base([preview discover-base]) --> kubectl_deployment[kubectl get Deployment JSON]
    discover_base --> kubectl_service[kubectl get Service JSON]
    kubectl_deployment --> normalize[Normalize cloneable workload contract]
    kubectl_service --> normalize
    normalize --> exclude_runtime[Exclude runtime and cluster-assigned identity]
    exclude_runtime --> write_contract[Write BaseWorkloadContract JSON]
    write_contract --> render_contract[preview render --base-contract embeds discoveredBase]
    render_contract --> kind_gate[Kind lifecycle applies discovered contract preview]
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
types:
  BaseWorkloadContract:
    fields:
      namespace: string
      app: string
      deployment: string
      service: string
      selector: map<string,string>
      podLabels: map<string,string>
      container: BaseContainerContract
      servicePorts: list<BaseServicePort>
      excludedRuntimeFields: list<string>
  BaseContainerContract:
    fields:
      name: string
      image: string
      ports: list<BaseContainerPort>
      env: list<BaseEnvVar>
      resources: json
      readinessPath: string?
      livenessPath: string?
  BaseEnvVar:
    fields:
      name: string
      value: string?
      valueFromKind: string?
normalization_rules:
  include:
    - deployment selector and pod labels
    - selected container name/image/ports/env/resources/probe paths
    - service ports and target ports
  exclude:
    - metadata.uid
    - metadata.resourceVersion
    - metadata.generation
    - metadata.managedFields
    - metadata.ownerReferences
    - status
    - spec.clusterIP
    - spec.clusterIPs
    - spec.ports[].nodePort
    - status.loadBalancer
    - secrets by default
failure_modes:
  ambiguous_container: "more than one container and none named like --app"
  selector_mismatch: "Deployment selector is not present in pod labels"
  identity_mismatch: "Deployment/Service namespace or name does not match request"
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: preview discover-base
    args:
      --namespace: "base namespace to inspect"
      --app: "base Deployment and Service name"
      --context: "optional kubectl context"
      --out: "optional JSON output path; stdout when omitted"
    behavior:
      - reads Deployment and Service JSON using kubectl
      - normalizes to BaseWorkloadContract
      - writes actionable kubectl errors when discovery fails
  - name: preview render
    added_args:
      --base-contract: "optional BaseWorkloadContract JSON emitted by discover-base"
    behavior:
      - embeds discoveredBase into plans/workload-clone.json
      - preserves existing manual base namespace path when omitted
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: preview-base-workload-discovery-unit-tests
requirements:
  fixture_normalization:
    id: R1
    text: "normalize_base_workload reads Kubernetes Deployment/Service JSON fixtures, preserves selectors, ports, env, probes, and resources, and excludes runtime identity values."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test base_discovery_contract"
  ambiguous_rejection:
    id: R2
    text: "base discovery refuses ambiguous multi-container Deployments without a container named like the app."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test base_discovery_contract"
  render_integration:
    id: R3
    text: "preview render --base-contract consumes discovered JSON and embeds discoveredBase in plans/workload-clone.json."
    kind: behavior
    risk: high
    verify: "cargo test -p preview local_ci_render_consumes_discovered_base_contract_file"
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "normalize Kubernetes JSON fixture"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "reject ambiguous base workloads"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "render consumes discovered contract"
      risk: high
      verifymethod: test
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: preview-kind-base-discovery
    command: "PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture"
    assertions:
      - "kind creates a uat-base Deployment and Service fixture."
      - "`preview discover-base --namespace uat-base --app checkout --out base-contract.json` reads the fixture through kubectl."
      - "The discovered contract drives preview render before preview apply/rollout."
      - "The test cleans the preview, control, and test-created base namespaces."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "apps/preview/src/discover.rs"
    action: add
    section: logic
    description: "Implement the discovery flow from kubectl JSON fetch through normalization and discovered contract output."
    impl_mode: hand-written
  - path: "apps/preview/src/discover.rs"
    action: add
    section: schema
    description: "Add BaseWorkloadContract data model plus kubectl discovery and normalization helpers."
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-discover-rs>"
  - path: "apps/preview/src/lib.rs"
    action: modify
    section: schema
    description: "Export discovery models and helpers."
    impl_mode: hand-written
  - path: "apps/preview/src/render.rs"
    action: modify
    section: schema
    description: "Allow RenderInput to carry an optional discovered base contract and embed it in the workload clone plan."
    impl_mode: hand-written
  - path: "apps/preview/src/main.rs"
    action: modify
    section: cli
    description: "Add preview discover-base and render --base-contract CLI surfaces."
    impl_mode: hand-written
  - path: "apps/preview/tests/base_discovery_contract.rs"
    action: add
    section: unit-test
    description: "Cover no-cluster Kubernetes JSON fixture normalization, runtime field exclusion, and ambiguous workload rejection."
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-tests-base-discovery-contract-rs>"
  - path: "apps/preview/tests/local_cicd_contract.rs"
    action: modify
    section: unit-test
    description: "Cover preview render --base-contract in the local CI lifecycle lane."
    impl_mode: hand-written
  - path: "apps/preview/tests/kind_lifecycle.rs"
    action: modify
    section: e2e-test
    description: "Create a base Deployment/Service fixture and run preview discover-base before rendering the preview lifecycle."
    impl_mode: hand-written
```
