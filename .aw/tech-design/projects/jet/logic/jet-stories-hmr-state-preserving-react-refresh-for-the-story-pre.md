---
id: projects-jet-logic-jet-stories-hmr-state-preserving-react-refresh-for-the-story-pre-md
fill_sections: [logic, changes]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-dev-manager
    coverage: partial
    rationale: "Wiring HMR + state-preserving React refresh into the isolated story preview completes the live-edit workbench loop of component-workbench."
---

# jet stories: Preview HMR + State-Preserving React Refresh

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-hmr
entry: watch
nodes:
  watch:       { kind: start,    label: "stories server watches story/component files" }
  change:      { kind: process,  label: "file change event (story or component source)" }
  graph:       { kind: process,  label: "module_graph.dependents_of(changed) -> affected modules" }
  invalidate:  { kind: process,  label: "invalidate affected module cache" }
  notify:      { kind: process,  label: "push HMR message to preview frame over WS" }
  refreshable: { kind: decision, label: "edit react-refresh compatible?" }
  patch:       { kind: process,  label: "state-preserving react-refresh update of component" }
  reload:      { kind: process,  label: "full preview reload (manager shell stays)" }
  done:        { kind: terminal, label: "preview updated; manager shell untouched" }
edges:
  - { from: watch,       to: change }
  - { from: change,      to: graph }
  - { from: graph,       to: invalidate }
  - { from: invalidate,  to: notify }
  - { from: notify,      to: refreshable }
  - { from: refreshable, to: patch,  label: "yes" }
  - { from: refreshable, to: reload, label: "no" }
  - { from: patch,       to: done }
  - { from: reload,      to: done }
---
flowchart TD
    watch([watch story/component files]) --> change[file change event]
    change --> graph[dependents_of -> affected modules]
    graph --> invalidate[invalidate module cache]
    invalidate --> notify[push HMR msg to preview WS]
    notify --> refreshable{react-refresh compatible?}
    refreshable -->|yes| patch[state-preserving refresh]
    refreshable -->|no| reload[full preview reload]
    patch --> done([preview updated, manager untouched])
    reload --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/hmr.rs"
    action: create
    section: logic
    description: |
      Preview HMR: a file watcher + WS endpoint that, on a story/component edit,
      computes the affected module set (dependents_of), invalidates the module
      cache, and pushes an HMR message; reuses the dev_server hmr_client +
      react_refresh runtime so compatible edits are state-preserving and
      incompatible edits fall back to a full preview reload.
    impl_mode: hand-written
  - path: "projects/jet/src/stories/server.rs"
    action: modify
    section: logic
    description: |
      Wire the HMR WS route + watcher into the stories server and inject the
      HMR client runtime into the preview HTML so the preview frame (not the
      manager shell) hot-updates.
    impl_mode: hand-written
  - path: "projects/jet/src/stories/manager.rs"
    action: modify
    section: logic
    description: |
      Inject the HMR client script into render_preview_html; keep the manager
      shell free of HMR reloads.
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/preview_hmr.rs"
    action: create
    section: unit-test
    description: |
      Tests: a changed module yields the correct invalidation set; the preview
      HTML includes the HMR client; a react-refresh-compatible vs incompatible
      edit routes to patch vs full reload; the manager shell is not reloaded.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (id jet-stories-hmr) is complete and deterministic: watch -> change -> dependents_of -> invalidate -> WS notify -> decision react-refresh-compatible (state-preserving patch) vs not (full preview reload) -> terminal preview-updated with manager untouched. All nodes reachable; the refreshable decision carries both labeled branches; terminal done is a real end. Scope correct: reuses existing HMR substrate; builds on B2.
