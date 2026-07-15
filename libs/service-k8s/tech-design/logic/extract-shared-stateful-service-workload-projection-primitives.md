---
id: '1644'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-stateful-workload-projection
entry: app_policy
nodes:
  app_policy: { kind: start, label: "App-owned CRD and workload policy" }
  typed_input: { kind: process, label: "Build service-k8s typed projection input" }
  statefulset: { kind: process, label: "Render common StatefulSet with downward API, storage, probes, and security defaults" }
  services: { kind: process, label: "Render headless/client Services and PDB" }
  hpa: { kind: decision, label: "App policy enables HPA?" }
  render_hpa: { kind: process, label: "Render optional HPA without changing durable shard topology" }
  preserve: { kind: terminal, label: "Lumen and Tape retain distinct CRDs and domain policy" }
edges:
  - { from: app_policy, to: typed_input }
  - { from: typed_input, to: statefulset }
  - { from: statefulset, to: services }
  - { from: services, to: hpa }
  - { from: hpa, to: render_hpa, label: "yes" }
  - { from: hpa, to: preserve, label: "no" }
  - { from: render_hpa, to: preserve }
---
flowchart TD
    app_policy([App-owned CRD and workload policy]) --> typed_input[Build service-k8s typed projection input]
    typed_input --> statefulset[Render common StatefulSet]
    statefulset --> services[Render Services and PDB]
    services --> hpa{App policy enables HPA?}
    hpa -->|yes| render_hpa[Render optional HPA]
    hpa -->|no| preserve([Preserve distinct CRDs and domain policy])
    render_hpa --> preserve
```
