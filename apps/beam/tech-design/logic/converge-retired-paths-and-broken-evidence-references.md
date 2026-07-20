---
id: '2147'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: converge-paths
entry: start
nodes:
  start: { kind: start, label: "Identify files in projects/beam and .aw" }
  relocate: { kind: process, label: "Relocate benchmark evidence to apps/beam/" }
  remove_aw: { kind: process, label: "Remove retired repo-root .aw/ artifact" }
  update_refs: { kind: terminal, label: "Update README and CAPABILITIES.md refs" }
edges:
  - { from: start, to: relocate }
  - { from: relocate, to: remove_aw }
  - { from: remove_aw, to: update_refs }
---
flowchart TD
    start([Identify files]) --> relocate[Relocate evidence]
    relocate --> remove_aw[Remove .aw/]
    remove_aw --> update_refs([Update refs])
```
