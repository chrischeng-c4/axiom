---
id: projects-jet-logic-jet-stories-controls-args-panel-derived-from-ts-prop-types-argty-md
fill_sections: [logic, changes]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-controls-panel
    coverage: partial
    rationale: "Deriving a live Controls panel from component prop types + argTypes and feeding edited args back into the preview is the defining interactive feature of component-workbench."
---

# jet stories: Controls Panel Derived from Prop Types + argTypes

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-controls
entry: start
nodes:
  start:    { kind: start,    label: "manager renders a selected story (B2)" }
  locate:   { kind: process,  label: "locate the story component prop type (interface/type alias)" }
  found:    { kind: decision, label: "prop type found?" }
  no_ctrl:  { kind: terminal, label: "no controls (component props untyped)" }
  infer:    { kind: process,  label: "per prop infer control: bool->toggle, string->text, number->number, union->select" }
  override: { kind: process,  label: "apply meta.argTypes overrides (control type/options/disable)" }
  panel:    { kind: process,  label: "render Controls panel seeded from merged story args" }
  edit:     { kind: process,  label: "user edits a control -> update args" }
  rerender: { kind: process,  label: "re-render preview with new args (live)" }
  done:     { kind: terminal, label: "controls live; preview reflects edited args" }
edges:
  - { from: start,    to: locate }
  - { from: locate,   to: found }
  - { from: found,    to: no_ctrl,  label: "no" }
  - { from: found,    to: infer,    label: "yes" }
  - { from: infer,    to: override }
  - { from: override, to: panel }
  - { from: panel,    to: edit }
  - { from: edit,     to: rerender }
  - { from: rerender, to: done }
---
flowchart TD
    start([story selected in manager]) --> locate[locate component prop type]
    locate --> found{prop type found?}
    found -->|no| no_ctrl([no controls])
    found -->|yes| infer[infer control kind per prop]
    infer --> override[apply meta.argTypes overrides]
    override --> panel[render Controls panel from merged args]
    panel --> edit[user edits control -> update args]
    edit --> rerender[re-render preview live]
    rerender --> done([controls live, preview reflects args])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/prop_extractor.rs"
    action: create
    section: logic
    description: |
      Tree-sitter walk of a component file: locate the component's props type
      (interface or type alias referenced by the component's first param) and
      return an ordered list of (prop name, type text, optional flag).
    impl_mode: hand-written
  - path: "projects/jet/src/stories/controls.rs"
    action: create
    section: logic
    description: |
      Map a prop type to a control kind (bool->toggle, string->text,
      number->number, string-literal union->select with options, else text),
      then apply meta.argTypes overrides (control type/options/disable). Returns
      the resolved control descriptors for a story.
    impl_mode: hand-written
  - path: "projects/jet/src/stories/manager.rs"
    action: modify
    section: logic
    description: |
      Render a Controls panel from the resolved controls seeded with the story's
      merged args, and an args channel so editing a control posts updated args to
      the preview frame for a live re-render (reusing the B2b preview render hook).
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/controls.rs"
    action: create
    section: unit-test
    description: |
      Tests: prop-type extraction for a typed component; control inference for
      bool/string/number/string-literal-union; meta.argTypes override wins; the
      Controls panel HTML seeds current arg values; editing posts new args.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (id jet-stories-controls) is complete and deterministic: locate prop type -> decision found (terminal no_ctrl vs infer) -> infer per-prop control -> apply argTypes overrides -> render panel from merged args -> edit -> live re-render -> terminal done. All nodes reachable; the found decision carries both labeled branches; terminals (no_ctrl, done) are real ends. Scope correct: builds on B1 args/argTypes + B2 manager/preview.
