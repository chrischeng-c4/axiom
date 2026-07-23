---
id: '2461'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-switching-state
entry: project_select
nodes:
  project_select: { kind: start, label: select-project }
  workspace: { kind: process, label: update-selected-workspace }
  files: { kind: process, label: refresh-files-listing }
  no_pty_change: { kind: terminal, label: preserve-existing-pty }
  tab_select: { kind: start, label: select-tab }
  show_layer: { kind: process, label: show-tab-layer }
  retained_view: { kind: process, label: retain-terminal-view }
  incremental: { kind: terminal, label: feed-new-bytes-only }
edges:
  - { from: project_select, to: workspace }
  - { from: workspace, to: files }
  - { from: files, to: no_pty_change }
  - { from: tab_select, to: show_layer }
  - { from: show_layer, to: retained_view }
  - { from: retained_view, to: incremental }
---
flowchart LR
  project_select([Select project]) --> workspace[Update selected workspace]
  workspace --> files[Refresh Files and launch root]
  files --> no_pty_change([Preserve existing PTY])
  tab_select([Select tab]) --> show_layer[Show selected terminal layer]
  show_layer --> retained_view[Retain tab-keyed TerminalView]
  retained_view --> incremental([Feed new bytes only])
```

Project selection updates the selected project id, launch root, and file listing together on the main actor. This selection is for future terminal launches; it never sends a lifecycle request or rewrites an existing tab's cwd.

Every non-idle and non-failed tab owns a mounted `TerminalSurface` keyed by tab id. Inactive layers are visually hidden and non-interactive instead of destroyed. Their coordinators retain both the SwiftTerm terminal buffer and fed-byte cursor, so a tab switch only exposes its existing renderer; polling supplies subsequent incremental bytes.
