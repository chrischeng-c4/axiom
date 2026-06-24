---
id: projects-jet-logic-jet-stories-build-static-export-of-the-component-workbench-for-h-md
fill_sections: [logic, changes]
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

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/build.rs"
    action: create
    section: logic
    description: |
      Static exporter: build_stories_static(root, out_dir) -> discover StoryIndex,
      clean out_dir, write manager index.html (reuse manager::render_manager_html),
      and per story write preview/{id}.html (render_preview_html) + transform and
      emit each imported module to out_dir with relative URLs; copy referenced
      assets. Output is servable by any static host with no jet process.
    impl_mode: hand-written
  - path: "projects/jet/src/stories/manager.rs"
    action: modify
    section: logic
    description: |
      Allow the manager/preview renderers to emit relative (static) URLs in
      addition to the dev-server absolute routes, so the same HTML works in the
      static build.
    impl_mode: hand-written
  - path: "projects/jet/src/cli.rs"
    action: modify
    section: cli
    description: |
      Add  subcommand dispatching to
      stories::build_stories_static; dev  unchanged.
    impl_mode: hand-written
  - path: "projects/jet/src/stories/mod.rs"
    action: modify
    section: logic
    description: |
      Register  and re-export build_stories_static.
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/stories_build.rs"
    action: create
    section: unit-test
    description: |
      Tests: build to a temp out_dir emits index.html + one preview per story +
      the transformed modules they import; emitted URLs are relative and resolve
      to files present in the output; dev jet stories behavior unaffected.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (id jet-stories-build) is complete and deterministic: cmd -> discover -> clean out_dir -> render manager -> per-story loop (preview HTML + emit transformed modules + rewrite to relative URLs) -> decision more -> copy assets -> terminal static workbench. All nodes reachable; the more decision carries both labeled branches; terminal done is a real end. Reuses B2 render fns; dev-only HMR excluded. Scope correct.
