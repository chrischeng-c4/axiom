---
id: projects-jet-logic-jet-stories-controls-generic-cross-file-and-intersection-prop-ty-md
fill_sections: [logic, changes]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-controls-panel
    coverage: partial
    rationale: "Resolving generic, cross-file, and intersection prop types lets controls render for far more real components, completing the controls panel."
---

# jet stories controls: Generic, Cross-File, and Intersection Prop Types

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-controls-advanced-props
entry: locate
nodes:
  locate:  { kind: start,    label: "controls needs the component prop type" }
  shape:   { kind: decision, label: "prop type shape" }
  same:    { kind: process,  label: "same-file interface/type (existing)" }
  generic: { kind: process,  label: "generic Props<T> / FC<Props>: resolve type arg/base where determinable" }
  cross:   { kind: process,  label: "imported from sibling file: parse that file's decl" }
  inter:   { kind: process,  label: "intersection A & B: merge members" }
  unres:   { kind: terminal, label: "unresolvable: no controls (graceful)" }
  members: { kind: process,  label: "collect prop members -> PropDef list" }
  done:    { kind: terminal, label: "PropDefs for control inference" }
edges:
  - { from: locate,  to: shape }
  - { from: shape,   to: same,    label: "same-file" }
  - { from: shape,   to: generic, label: "generic" }
  - { from: shape,   to: cross,   label: "imported" }
  - { from: shape,   to: inter,   label: "intersection" }
  - { from: shape,   to: unres,   label: "unresolvable" }
  - { from: same,    to: members }
  - { from: generic, to: members }
  - { from: cross,   to: members }
  - { from: inter,   to: members }
  - { from: members, to: done }
---
flowchart TD
    locate([need component prop type]) --> shape{prop type shape}
    shape -->|same-file| same[same-file decl]
    shape -->|generic| generic[resolve type arg/base]
    shape -->|imported| cross[parse sibling file decl]
    shape -->|intersection| inter[merge members]
    shape -->|unresolvable| unres([no controls])
    same --> members[collect PropDefs]
    generic --> members
    cross --> members
    inter --> members
    members --> done([PropDefs for inference])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/prop_extractor.rs"
    action: modify
    section: logic
    description: |
      Extend extract_props to resolve: generic prop types (React.FC<Props>,
      Props<Variant> where the type arg/base is statically determinable),
      cross-file imported prop types (follow the import to the sibling file and
      parse its interface/type decl), and intersection types (A & B -> union of
      members). Unresolvable shapes degrade gracefully to no props (no error).
    impl_mode: hand-written
  - path: "projects/jet/src/stories/controls.rs"
    action: modify
    section: logic
    description: |
      Consume the richer PropDef set (merged/dedup members) unchanged downstream;
      only adjust if member-merge ordering needs dedup.
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/controls.rs"
    action: modify
    section: unit-test
    description: |
      Tests: imported prop type from a sibling file yields controls; intersection
      (Base & Extra) yields the union of members; a simple generic resolves;
      unresolvable generic -> no controls, no error; existing controls tests pass.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (jet-controls-advanced-props) complete + deterministic: locate -> shape decision (5 labeled branches incl unresolvable terminal) -> per-shape member collection -> PropDefs -> terminal. All nodes reachable; both terminals real. Extends B3 prop_extractor.
