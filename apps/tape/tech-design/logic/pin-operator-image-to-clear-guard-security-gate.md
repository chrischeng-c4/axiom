---
id: '1703'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-operator-image-pin
entry: render_operator
nodes:
  render_operator:
    kind: start
    label: "tape k8s operator render"
  fixture:
    kind: process
    label: "checked-in control-plane Deployment fixture"
  pinned:
    kind: terminal
    label: "operator image uses a concrete Tape release tag"
  development:
    kind: terminal
    label: "instance dev profile remains an explicit local override"
edges:
  - { from: render_operator, to: fixture }
  - { from: fixture, to: pinned, label: "render preserves release-pinned image" }
---
flowchart TD
    render_operator([k8s operator render]) --> fixture[operator deployment fixture]
    fixture --> pinned([tape:0.4.5 release tag])
    development([instance dev image override])
```

The operator control-plane fixture is the renderer-owned production artifact. Its image must use the current concrete Tape release tag rather than `latest`, so the checked-in manifest and rendered output are reproducible and pass K8003. Namespace replacement remains image-neutral. The explicit image argument for instance rendering, including the dev profile's local image default, is a separate data-plane contract and remains unchanged.
