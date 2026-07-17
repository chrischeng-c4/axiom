---
id: '1887'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-shared-library-manifest-convergence
entry: load_workspace
nodes:
  load_workspace:
    kind: start
    label: "Cargo loads projects/sift/Cargo.toml as a workspace member"
  aliases_current:
    kind: decision
    label: "all dependency package names and paths resolve to current shared libraries?"
  fail_manifest:
    kind: terminal
    label: "workspace load fails before any package can build"
  preserve_aliases:
    kind: process
    label: "map existing Rust aliases to service-k8s, storage-durable, metrics-prometheus, and raft-runtime"
  metadata:
    kind: process
    label: "run root cargo metadata without changing runtime code"
  buildable:
    kind: terminal
    label: "workspace manifest is valid and focused package checks can run"
edges:
  - { from: load_workspace, to: aliases_current }
  - { from: aliases_current, to: fail_manifest, label: "no" }
  - { from: aliases_current, to: preserve_aliases, label: "rename-only fix" }
  - { from: preserve_aliases, to: metadata }
  - { from: metadata, to: buildable }
---
flowchart TD
    load[load Sift workspace manifest] --> current{shared package paths current?}
    current -->|no| aliases[preserve crate aliases; update package/path]
    current -->|yes| metadata[cargo metadata]
    aliases --> metadata
    metadata --> ready([workspace can build])
```
