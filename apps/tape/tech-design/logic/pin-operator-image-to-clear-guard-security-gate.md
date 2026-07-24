---
id: '1703'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-operator-image-pin-contract
entry: operator_fixture
nodes:
  operator_fixture:
    kind: start
    label: "apps/tape/k8s/operator/deployment.yaml"
  pinned_release:
    kind: process
    label: "image: ghcr.io/chrischeng-c4/tape:0.4.10"
  namespace_render:
    kind: terminal
    label: "namespace substitution preserves the pinned image"
  guard:
    kind: terminal
    label: "K8003 sees no latest or untagged production image"
edges:
  - { from: operator_fixture, to: pinned_release }
  - { from: pinned_release, to: namespace_render }
  - { from: pinned_release, to: guard }
---
flowchart TD
    fixture[operator Deployment fixture] --> release[ghcr.io/chrischeng-c4/tape:0.4.10]
    release --> render[operator render keeps image]
    release --> guard[K8003 clear]
```

`apps/tape/k8s/operator/deployment.yaml` is the exact Deployment fragment consumed by `render_operator_yaml`. Its operator image is pinned to the workspace's concrete Tape release tag (`ghcr.io/chrischeng-c4/tape:0.4.10`). Rendering may replace only `tape-system` namespace fields and therefore cannot downgrade that image reference. The data-plane instance renderer remains separately configurable through its existing `--image` option; this change does not turn a local dev override into a production operator default.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/k8s/operator/deployment.yaml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/tape/tests/deploy_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-operator-image-pin-contract-verification
requirements:
  pinned_operator_image:
    id: R1
    text: "The offline operator render emits tape:0.4.5 for its Deployment and never emits tape:latest for the operator control plane."
    kind: regression
    risk: high
    verify: render_verbs_emit_parseable_yaml_offline
---
flowchart TD
    r1[R1 pinned operator image] --> render_verbs_emit_parseable_yaml_offline[render_verbs_emit_parseable_yaml_offline]
```
