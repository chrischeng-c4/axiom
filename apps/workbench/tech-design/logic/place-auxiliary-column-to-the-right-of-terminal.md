---
id: '2459'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-auxiliary-right-order
entry: profile
nodes:
  profile: { kind: decision, label: "Beta profile?" }
  stable: { kind: terminal, label: "Projects | Terminal" }
  terminal: { kind: process, label: "render primary terminal workspace" }
  auxiliary: { kind: process, label: "render Files auxiliary after terminal" }
  beta: { kind: terminal, label: "Projects | Terminal | Auxiliary" }
edges:
  - { from: profile, to: stable, label: "stable" }
  - { from: profile, to: terminal, label: "beta" }
  - { from: terminal, to: auxiliary }
  - { from: auxiliary, to: beta }
---
flowchart LR
    profile{Beta profile?} -->|Stable| stable([Projects | Terminal])
    profile -->|Beta| terminal[Primary terminal workspace]
    terminal --> auxiliary[Files auxiliary]
    auxiliary --> beta([Projects | Terminal | Auxiliary])
```

`WorkbenchView.body` remains a `NavigationSplitView` with the registered Projects sidebar as its leading column. Within the detail `HStack`, `terminalWorkspace` is always rendered first and remains the only flexible workspace. When `WorkbenchRuntimeProfile` is `beta`, a divider and the existing bounded `auxiliaryColumn` follow it, yielding the visible order Projects | Terminal | Auxiliary. Stable does not render the divider or Auxiliary column and therefore remains Projects | Terminal.

The implementation changes only detail-child ordering and the terminal workspace sizing priority. It does not change file listing, project selection, tab lifecycle, PTY launch, or the native sidebar toggle. Native UI coverage asserts that terminal controls appear horizontally before the Auxiliary Files element in Beta, while the Stable profile has no Auxiliary element.
