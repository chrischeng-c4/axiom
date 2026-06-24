---
id: projects-jet-logic-jet-stories-build-static-export-of-the-component-workbench-for-h-md
fill_sections: [logic]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-static-export
    coverage: partial
    rationale: "jet stories build renders the manager + per-story previews + modules to a static hostable directory — the deferred static-export mode of component-workbench."
---

# jet stories build: Static Export of the Component Workbench

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-build
entry: cmd
nodes:
  cmd:      { kind: start,    label: "jet stories build [--out-dir]" }
  discover: { kind: process,  label: "stories::discover(root) -> StoryIndex (B1)" }
  clean:    { kind: process,  label: "prepare/clean out_dir" }
  manager:  { kind: process,  label: "render manager HTML (B2) -> out_dir/index.html" }
  loop:     { kind: process,  label: "for each story" }
  preview:  { kind: process,  label: "render isolated preview HTML (B2) -> out_dir/preview/{id}.html" }
  modules:  { kind: process,  label: "transform + emit imported modules -> out_dir (relative URLs)" }
  rewrite:  { kind: process,  label: "rewrite preview/module URLs to relative static paths" }
  more:     { kind: decision, label: "more stories?" }
  assets:   { kind: process,  label: "copy referenced assets" }
  done:     { kind: terminal, label: "static workbench in out_dir (servable, no jet process)" }
edges:
  - { from: cmd,      to: discover }
  - { from: discover, to: clean }
  - { from: clean,    to: manager }
  - { from: manager,  to: loop }
  - { from: loop,     to: preview }
  - { from: preview,  to: modules }
  - { from: modules,  to: rewrite }
  - { from: rewrite,  to: more }
  - { from: more,     to: loop,   label: "yes" }
  - { from: more,     to: assets, label: "no" }
  - { from: assets,   to: done }
---
flowchart TD
    cmd([jet stories build]) --> discover[discover StoryIndex]
    discover --> clean[prepare out_dir]
    clean --> manager[render manager -> index.html]
    manager --> loop[for each story]
    loop --> preview[render preview -> preview/id.html]
    preview --> modules[transform+emit modules]
    modules --> rewrite[rewrite URLs to relative]
    rewrite --> more{more stories?}
    more -->|yes| loop
    more -->|no| assets[copy assets]
    assets --> done([static workbench, servable])
```
