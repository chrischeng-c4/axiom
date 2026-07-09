---
id: preview-local-router-adapter
summary: >
  Add a local router adapter for Preview. The adapter loads route-binding data
  from rendered files or kind ConfigMaps and resolves requests into explicit
  base, preview, or not-found decisions so GKE ingress work starts from a
  proven traffic rule.
capability_refs:
  - id: "preview-external-contracts"
    role: primary
    gap: "local-router-adapter"
    claim: "local-router-adapter"
    coverage: partial
    rationale: >
      Work item #1110 adds local adapter proof for base fallback, preview target
      routing, and invalid-target fail-closed behavior.
fill_sections: [logic, schema, cli, unit-test, e2e-test, changes]
---

# TD: Preview Local Router Adapter

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: preview-local-router-adapter-flow
entry: preview_router_resolve
nodes:
  load_table: { kind: start, label: "load route table from rendered YAML or ConfigMaps" }
  request: { kind: process, label: "read host, X-UAT-Target header, uat_target cookie" }
  no_target: { kind: decision, label: "no header/cookie target" }
  known_target: { kind: decision, label: "target exists and host matches" }
  base: { kind: terminal, label: "base decision" }
  preview: { kind: terminal, label: "preview decision" }
  not_found: { kind: terminal, label: "not-found decision" }
edges:
  - { from: load_table, to: request }
  - { from: request, to: no_target }
  - { from: no_target, to: base }
  - { from: request, to: known_target }
  - { from: known_target, to: preview }
  - { from: known_target, to: not_found }
---
flowchart TD
    load_table([Load route table]) --> request[RouteRequest]
    request --> no_target{No target?}
    no_target --> base[Base route]
    request --> known_target{Known target and host?}
    known_target --> preview[Preview route]
    known_target --> not_found[Fail closed]
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
types:
  BaseRoute:
    fields:
      host: string
      namespace: string
      service: string
      servicePort: u16
  RouteDecision:
    fields:
      outcome: base | preview | not-found
      target: string?
      namespace: string?
      service: string?
      servicePort: u16?
      reason: string
rules:
  - "X-UAT-Target has precedence over uat_target cookie."
  - "No target routes to BaseRoute only when host matches."
  - "Known target with matching host routes to preview namespace/service."
  - "Unknown target or host mismatch returns not-found with no namespace/service."
sources:
  rendered_file: "router/route-binding.yaml"
  kind_configmaps: "ConfigMaps labeled preview.cclab.dev/kind=route-binding"
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: preview router resolve
    args:
      --dir: "optional rendered preview directory; when omitted load ConfigMaps through kubectl"
      --context: "optional kubectl context"
      --control-namespace: "route-binding ConfigMap namespace"
      --host: "request host"
      --base-namespace: "base fallback namespace"
      --base-service: "base fallback service"
      --base-service-port: "base fallback service port"
      --header-target: "optional X-UAT-Target value"
      --cookie-target: "optional uat_target value"
    behavior:
      - returns JSON RouteDecision
      - proves base fallback without a live ingress
      - fails closed for invalid targets
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: preview-local-router-adapter-unit-tests
requirements:
  base_fallback:
    id: R1
    text: "No header/cookie target returns a base route decision."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test router_contract"
  target_precedence:
    id: R2
    text: "Header target overrides cookie target and valid targets route to preview."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test router_contract"
  fail_closed:
    id: R3
    text: "Unknown target and host mismatch return not-found without namespace/service."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test router_contract"
  cli_adapter:
    id: R4
    text: "preview router resolve loads rendered files and emits base/preview/not-found JSON decisions."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test local_cicd_contract local_router_resolve_proves_base_preview_and_fail_closed"
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "base fallback"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "target precedence"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "fail closed"
      risk: high
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "CLI adapter decisions"
      risk: high
      verifymethod: test
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: preview-kind-router-adapter
    command: "PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture"
    assertions:
      - "kind applies the route-binding ConfigMap into the control namespace."
      - "load_route_table_from_kubectl reads route-binding ConfigMaps."
      - "resolve_route_with_base maps X-UAT-Target=mr-123 to uat-mr-123/checkout."
      - "cleanup still removes preview/control/base namespaces created by the test."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "apps/preview/src/router.rs"
    action: modify
    section: logic
    description: "Add BaseRoute, RouteDecision, base fallback, fail-closed resolution, and route table loaders for rendered files and kubectl ConfigMaps."
    impl_mode: hand-written
  - path: "apps/preview/src/main.rs"
    action: modify
    section: cli
    description: "Add preview router resolve CLI outputting JSON route decisions."
    impl_mode: hand-written
  - path: "apps/preview/src/lib.rs"
    action: modify
    section: schema
    description: "Export router adapter types and loaders."
    impl_mode: hand-written
  - path: "apps/preview/tests/router_contract.rs"
    action: modify
    section: unit-test
    description: "Cover base fallback, header override, cookie routing, host mismatch, invalid-target fail-closed, and rendered-file route table loading."
    impl_mode: hand-written
  - path: "apps/preview/tests/local_cicd_contract.rs"
    action: modify
    section: unit-test
    description: "Cover preview router resolve CLI decisions from rendered files."
    impl_mode: hand-written
  - path: "apps/preview/tests/kind_lifecycle.rs"
    action: modify
    section: e2e-test
    description: "Load route-binding ConfigMaps from kind and resolve a preview target through the local adapter."
    impl_mode: hand-written
```
