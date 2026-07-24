---
id: '2511'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-native-accessibility-identities
entry: container
nodes:
  container: { kind: start, label: "SwiftUI container" }
  contain: { kind: process, label: "contain accessibility children" }
  identity: { kind: process, label: "container identifier" }
  child: { kind: terminal, label: "independent descendant identifier" }
  file: { kind: process, label: "bounded top-level file name" }
edges:
  - { from: container, to: contain }
  - { from: contain, to: identity }
  - { from: contain, to: child }
  - { from: file, to: child, label: "auxiliary.file.<name>" }
---
flowchart LR
  container([SwiftUI container]) --> contain[Contain accessibility children]
  contain --> identity[Container identifier]
  contain --> child([Independent descendant identifier])
  file[Bounded top-level file name] -->|auxiliary.file.name| child
```

Each terminal container becomes an explicit accessibility element with contained children before its identifier is assigned. SwiftUI then exposes both the container and descendant controls instead of propagating the parent identifier onto every accessible leaf. Auxiliary rows use the top-level entry name, which is unique within one project root and bounded independently of the absolute fixture path.
