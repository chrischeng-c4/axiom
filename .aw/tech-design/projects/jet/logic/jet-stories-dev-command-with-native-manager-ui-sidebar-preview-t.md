---
id: projects-jet-logic-jet-stories-dev-command-with-native-manager-ui-sidebar-preview-t-md
fill_sections: [logic]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-dev-manager
    coverage: partial
    rationale: "jet stories serves a native manager UI (sidebar/toolbar/preview) and renders each story in isolation on the dev server — the workbench surface of component-workbench."
---

# jet stories: Dev Command, Native Manager UI, Isolated Render

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-manager
entry: cmd
nodes:
  cmd:         { kind: start,    label: "jet stories [--port --host]" }
  discover:    { kind: process,  label: "stories::discover(root) -> StoryIndex (B1)" }
  serve:       { kind: process,  label: "start dev server on host:port (reuse dev_server)" }
  routes:      { kind: process,  label: "register manager + preview + module routes" }
  ready:       { kind: process,  label: "manager listening; handle each request" }
  req:         { kind: decision, label: "request kind" }
  manager:     { kind: process,  label: "GET manager: sidebar(StoryIndex)+toolbar+preview frame" }
  preview:     { kind: process,  label: "GET preview/{id}: isolated render of one story, no app shell" }
  build_entry: { kind: process,  label: "build per-story entry module via module graph" }
  module:      { kind: process,  label: "GET module/asset: serve transformed module" }
  served:      { kind: terminal, label: "response served; previews isolated per story" }
edges:
  - { from: cmd,         to: discover }
  - { from: discover,    to: serve }
  - { from: serve,       to: routes }
  - { from: routes,      to: ready }
  - { from: ready,       to: req }
  - { from: req,         to: manager,     label: "manager" }
  - { from: req,         to: preview,     label: "preview" }
  - { from: req,         to: module,      label: "module" }
  - { from: preview,     to: build_entry }
  - { from: build_entry, to: served }
  - { from: manager,     to: served }
  - { from: module,      to: served }
---
flowchart TD
    cmd([jet stories]) --> discover[discover StoryIndex]
    discover --> serve[start dev server]
    serve --> routes[register manager/preview/module routes]
    routes --> ready[manager listening]
    ready --> req{request kind}
    req -->|manager| manager[sidebar+toolbar+preview frame]
    req -->|preview| preview[isolated story render]
    req -->|module| module[serve transformed module]
    preview --> build_entry[build per-story entry]
    build_entry --> served([served, isolated])
    manager --> served
    module --> served
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Applicability is correct: jet stories discovers the StoryIndex (B1), starts a dev-server variant, registers manager/preview/module routes, and on each request branches to serve the manager shell (sidebar+toolbar+preview frame), an isolated per-story preview (built via the module graph, no app shell), or a transformed module. Scoped to command+manager+isolated render; HMR is B2b and controls are B3.
