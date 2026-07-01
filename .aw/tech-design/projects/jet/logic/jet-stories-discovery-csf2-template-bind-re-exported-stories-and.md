---
id: projects-jet-logic-jet-stories-discovery-csf2-template-bind-re-exported-stories-and-md
fill_sections: [logic, changes]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: csf-story-discovery
    coverage: partial
    rationale: "Supporting CSF2 Template.bind, re-exported stories, and spread args lets discovery capture the full story set from real story files."
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: csf2-template-bind-re-exports
    coverage: partial
    rationale: "This TD verifies CSF2 Template.bind, re-exported stories, and spread args discovery."
---

# jet stories discovery: CSF2 Template.bind, Re-exported Stories, and Spread Args

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-csf2-discovery
entry: scan
nodes:
  scan:     { kind: start,    label: "parse_csf scans a named export" }
  exp:      { kind: decision, label: "export shape" }
  csf3:     { kind: process,  label: "CSF3 object: { args, render } (existing)" }
  bind:     { kind: process,  label: "CSF2: const S = Template.bind({}); S.args = {} -> story" }
  reexport: { kind: process,  label: "export {X} from ./other: resolve sibling, pull story" }
  resargs:  { kind: process,  label: "resolve args: merge spread { ...base, x } (statically known)" }
  index:    { kind: process,  label: "add to StoryIndex" }
  done:     { kind: terminal, label: "stories captured incl CSF2 / re-export / spread" }
edges:
  - { from: scan,     to: exp }
  - { from: exp,      to: csf3,     label: "csf3-object" }
  - { from: exp,      to: bind,     label: "template-bind" }
  - { from: exp,      to: reexport, label: "re-export" }
  - { from: csf3,     to: resargs }
  - { from: bind,     to: resargs }
  - { from: reexport, to: index }
  - { from: resargs,  to: index }
  - { from: index,    to: done }
---
flowchart TD
    scan([parse_csf named export]) --> exp{export shape}
    exp -->|csf3-object| csf3[CSF3 object args/render]
    exp -->|template-bind| bind[CSF2 Template.bind + .args]
    exp -->|re-export| reexport[resolve sibling, pull story]
    csf3 --> resargs[merge spread args statically]
    bind --> resargs
    reexport --> index[add to StoryIndex]
    resargs --> index
    index --> done([stories captured])
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: csf2_template_bind_re_exports
    capability_id: component-workbench
    claim_id: csf2-template-bind-re-exports
    name: "CSF2 Template.bind and re-exports"
    command: "cargo test -p jet --test csf_discovery -- --nocapture"
    proves: "CSF2 Template.bind, re-exported stories, and spread args are discovered."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/csf.rs"
    action: modify
    section: logic
    description: |
      Extend parse_csf to capture CSF2 stories (const S = Template.bind({}); S.args
      = {...} assigned after declaration), re-exported stories (export { X } from
      "./other" -> resolve sibling file + pull that story), and spread args
      (args: { ...base, x } -> merge the statically-known base where resolvable).
    impl_mode: hand-written
  - path: "projects/jet/src/stories/mod.rs"
    action: modify
    section: logic
    description: |
      discover() resolves re-exported story sources relative to the file when
      assembling the index; pass-through if unresolvable.
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/csf_discovery.rs"
    action: modify
    section: unit-test
    description: |
      Tests: CSF2 Template.bind + .args surfaces stories with args; re-exported
      story appears in the discovering file set; spread args merge the static
      base; existing csf_discovery tests pass.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (jet-csf2-discovery) complete + deterministic: scan named export -> shape decision (csf3/template-bind/re-export, all labeled) -> args resolution (spread merge) for csf3+bind / sibling pull for re-export -> add to index -> terminal. All nodes reachable; terminal real. Extends B1.
