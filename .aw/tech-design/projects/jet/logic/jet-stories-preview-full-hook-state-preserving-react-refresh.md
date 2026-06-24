---
id: projects-jet-logic-jet-stories-preview-full-hook-state-preserving-react-refresh-md
fill_sections: [logic, changes]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-preview-hmr
    coverage: partial
    rationale: "Hook-state-preserving React Refresh in the isolated story preview completes the live-edit loop of the component workbench."
---

# jet stories preview: Full Hook-State-Preserving React Refresh

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-refresh
entry: edit
nodes:
  edit:     { kind: start,    label: "preview module edited (HMR update msg)" }
  instr:    { kind: process,  label: "preview modules transformed with React Refresh reg ($RefreshReg$/$RefreshSig$)" }
  boundary: { kind: decision, label: "refresh-compatible (only component exports changed)?" }
  refresh:  { kind: process,  label: "performReactRefresh: re-register + reconcile, preserve hook state" }
  reload:   { kind: process,  label: "incompatible: full preview-frame reload" }
  done:     { kind: terminal, label: "preview updated; hook state preserved on compatible edit" }
edges:
  - { from: edit,     to: instr }
  - { from: instr,    to: boundary }
  - { from: boundary, to: refresh, label: "compatible" }
  - { from: boundary, to: reload,  label: "incompatible" }
  - { from: refresh,  to: done }
  - { from: reload,   to: done }
---
flowchart TD
    edit([preview module edited]) --> instr[modules carry React Refresh registration]
    instr --> boundary{refresh-compatible?}
    boundary -->|compatible| refresh[performReactRefresh, preserve hook state]
    boundary -->|incompatible| reload[full preview-frame reload]
    refresh --> done([preview updated])
    reload --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/hmr.rs"
    action: modify
    section: logic
    description: |
      Drive a React-Refresh update on a compatible edit: serve the refresh
      runtime and, on an update message, call performReactRefresh instead of a
      blind re-render; keep the full-reload fallback for incompatible edits.
    impl_mode: hand-written
  - path: "projects/jet/src/stories/server.rs"
    action: modify
    section: logic
    description: |
      Transform preview modules with React-Refresh registration ($RefreshReg$/
      $RefreshSig$), reusing the dev_server react_refresh transform, and serve
      the react-refresh runtime endpoint for the preview frame.
    impl_mode: hand-written
  - path: "projects/jet/src/stories/manager.rs"
    action: modify
    section: logic
    description: |
      Inject the react-refresh runtime + refresh-aware update handling into the
      preview client (so a compatible edit reconciles in place, preserving hook
      state); manager shell unchanged.
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/preview_hmr.rs"
    action: modify
    section: unit-test
    description: |
      Tests: preview modules are emitted with React-Refresh registration; the
      preview client wires performReactRefresh on update and full reload on
      incompatible; existing preview_hmr tests pass; manager not reloaded.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (jet-stories-refresh) complete + deterministic: edit -> instrumented modules -> compatibility decision (both labeled) -> performReactRefresh (state-preserving) vs full reload -> terminal. All nodes reachable; terminal real. Extends B2b.
