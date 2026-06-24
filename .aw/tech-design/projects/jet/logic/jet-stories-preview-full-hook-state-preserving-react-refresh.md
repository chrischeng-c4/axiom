---
id: projects-jet-logic-jet-stories-preview-full-hook-state-preserving-react-refresh-md
fill_sections: [logic]
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

# Reviews

### Review 1
**Verdict:** approved

- [logic] Applicability sound: preview modules carry React Refresh registration; on edit, branch refresh-compatible (performReactRefresh preserving hook state) vs incompatible (full preview reload). Extends B2b HMR; manager shell + bare-import (SF2) out of scope.
