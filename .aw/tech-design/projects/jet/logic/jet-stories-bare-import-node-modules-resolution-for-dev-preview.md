---
id: projects-jet-logic-jet-stories-bare-import-node-modules-resolution-for-dev-preview-md
fill_sections: [logic, changes]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-dev-manager
    coverage: partial
    rationale: "Resolving node_modules bare imports lets real components with third-party deps render in the workbench (dev + static), completing the manager surface."
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-bare-import-resolution
    coverage: partial
    rationale: "This TD verifies node_modules bare-import resolution for the stories dev preview and static build."
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

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: stories_bare_import_resolution
    capability_id: component-workbench
    claim_id: stories-bare-import-resolution
    name: "Stories bare-import resolution"
    command: "cargo test -p jet --test manager -- --nocapture"
    proves: "node_modules bare imports resolve for stories dev preview and static export."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/server.rs"
    action: modify
    section: logic
    description: |
      Dev module route: when a served module imports a bare specifier, resolve it
      in the project node_modules via the existing resolver / pkg_manager, then
      transform + serve the resolved dependency module(s) transitively (rewriting
      bare imports to the served paths). Unresolved/CDN-only deps fall back to the
      esm.sh importmap. React-class deps keep working.
    impl_mode: hand-written
  - path: "projects/jet/src/stories/build.rs"
    action: modify
    section: logic
    description: |
      Static build: emit the resolved node_modules dependency modules into the
      out dir with relative URLs (same resolution as dev), so the static preview
      loads them without a server / without esm.sh for resolved deps.
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/manager.rs"
    action: modify
    section: unit-test
    description: |
      Dev test: a fixture component importing a small node_modules package resolves
      + serves the dependency through the module route.
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/stories_build.rs"
    action: modify
    section: unit-test
    description: |
      Static test: jet stories build emits the resolved dependency module(s) and
      the static preview references them via relative URLs.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (jet-stories-bare-import) complete + deterministic: import -> specifier-kind decision -> relative(existing) / bare->resolve->resolved decision -> serve/emit dep vs esm.sh fallback -> terminal resolves in dev+static. All nodes reachable; both decisions labeled; terminal real. Builds on B2/B4.
