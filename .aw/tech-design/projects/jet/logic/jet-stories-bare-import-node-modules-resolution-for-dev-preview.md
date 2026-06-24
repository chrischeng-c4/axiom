---
id: projects-jet-logic-jet-stories-bare-import-node-modules-resolution-for-dev-preview-md
fill_sections: [logic]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-dev-manager
    coverage: partial
    rationale: "Resolving node_modules bare imports lets real components with third-party deps render in the workbench (dev + static), completing the manager surface."
---

# jet stories: Bare-Import (node_modules) Resolution for Dev Preview + Static Build

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-bare-import
entry: imp
nodes:
  imp:      { kind: start,    label: "preview module imports a specifier" }
  cls:      { kind: decision, label: "specifier kind" }
  relative: { kind: process,  label: "relative: transform + serve/emit sibling (existing)" }
  bare:     { kind: process,  label: "bare: resolve in node_modules via pkg resolver" }
  res:      { kind: decision, label: "resolved in node_modules?" }
  servedep: { kind: process,  label: "transform + serve/emit the dependency module(s)" }
  cdn:      { kind: process,  label: "fallback: esm.sh importmap (React-class / unresolved)" }
  done:     { kind: terminal, label: "import resolves in dev preview + static build" }
edges:
  - { from: imp,      to: cls }
  - { from: cls,      to: relative, label: "relative" }
  - { from: cls,      to: bare,     label: "bare" }
  - { from: bare,     to: res }
  - { from: res,      to: servedep, label: "yes" }
  - { from: res,      to: cdn,      label: "no" }
  - { from: relative, to: done }
  - { from: servedep, to: done }
  - { from: cdn,      to: done }
---
flowchart TD
    imp([preview imports specifier]) --> cls{specifier kind}
    cls -->|relative| relative[serve/emit sibling]
    cls -->|bare| bare[resolve in node_modules]
    bare --> res{resolved?}
    res -->|yes| servedep[transform + serve/emit dep]
    res -->|no| cdn[esm.sh importmap fallback]
    relative --> done([import resolves dev + static])
    servedep --> done
    cdn --> done
```
