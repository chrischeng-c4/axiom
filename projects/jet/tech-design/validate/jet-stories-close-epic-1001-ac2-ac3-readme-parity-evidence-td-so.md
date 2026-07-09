---
id: jet-stories-close-epic-1001-ac2-ac3-readme-parity-evidence-td-so
summary: >
  Epic #1001's 15 member WIs (#981, #987-#1000) are all closed via PR #1070,
  but two of the epic's own acceptance criteria were never actually
  satisfied: `projects/jet/README.md`'s Component Workbench row/detailed
  block still describes only pre-epic scope with a stale 5-test Gate
  Inventory, and `projects/jet/tech-design/semantic/jet-stories.md`'s
  `source_units` list omits `mdx.rs`/`optimizer.rs`. No in-repo evidence
  exists that decorators, a `play()` interaction, `argTypes` overrides, and
  an MDX docs page work together — only a manual, opt-in external Storybook
  oracle harness does. This TD closes WI #1343: it rewrites the README
  capability row/detailed block with concrete test-path citations for every
  shipped feature area, adds the two missing `source_units` entries, and
  adds a small in-repo fixture (decorators + `play()` + `argTypes` override
  + MDX docs page) with a passing static-build-verifying test, giving
  AC2-equivalent in-repo evidence without depending on an external
  third-party Storybook install.
capability_refs:
  - id: "component-workbench"
    role: primary
    gap: "close-stories-parity-evidence-gaps-from-epic-1001"
    claim: "close-stories-parity-evidence-gaps-from-epic-1001"
    coverage: partial
    rationale: "Pins WI #1343's closeout of epic #1001's AC2/AC3 gap under the existing Component Workbench work root: README row/detailed-block parity evidence, jet-stories.md schema source_units completeness, and an in-repo decorators+play+argTypes+MDX fixture suite."
fill_sections: [logic, config, unit-test, changes]
---

# jet stories: close epic #1001 AC2/AC3 — README parity evidence + TD source_units + fixture suite

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-close-epic-1001-ac2-ac3-flow
entry: audit_closed_wis
nodes:
  audit_closed_wis:
    kind: start
    label: "Audit epic #1001's 15 closed member WIs (#981,\n#987-#1000, landed via PR #1070) for the concrete\ntest paths each one actually added"
  classify_area:
    kind: decision
    label: "Does an existing cargo test already prove this\nfeature area (decorators/parameters/globals/loaders,\ncontrols, toolbar, measure/outline/highlight, actions,\ninteractions/play, a11y, story source, autodocs, MDX,\nmanager UX, index.json, headless test runner)?"
  cite_existing:
    kind: process
    label: "Cite the concrete test path: csf.rs::tests::\nparses_render_path_core_fields (decorators/\nparameters/globals/globalTypes/loaders/autodocs tag);\nstories_build.rs::static_manager_keeps_dev_feature_\nparity_checklist (toolbar/controls/actions/\ninteractions/a11y/source/docs/search/theme/index.json);\nmanager.rs::tests::manager_toolbar_renders_viewport_\nbackground_zoom_and_custom_parameters (toolbar+\nmeasure+outline+shortcuts); mdx.rs::tests::\ncompiles_core_doc_blocks + stories_build.rs::\nmdx_docs_pages_render_core_blocks_in_static_export\n(MDX); cli.rs run_stories_smoke_tests / `jet test\n--stories` (headless-Chromium play() runner)"
  gap_found:
    kind: process
    label: "GAP: no in-repo evidence proves decorators + a\nplay() interaction + argTypes overrides + an MDX\ndocs page compile and wire together for ONE story -\nonly the external, manually-run Storybook oracle\nharness (compare_storybook_oracle.mjs) exercises\nthis combination"
  more_areas:
    kind: decision
    label: "More feature areas left to classify?"
  rewrite_readme:
    kind: process
    label: "Rewrite README.md's Component Workbench summary\nrow + detailed Promise/Gate Inventory/EC Dimensions\nto enumerate the full shipped surface with the\ncataloged test-path citations"
  update_schema:
    kind: process
    label: "Add mdx.rs and optimizer.rs entries (symbols +\nownership_state) to jet-stories.md's schema\nsource_units list"
  design_fixture:
    kind: process
    label: "Design one small in-repo fixture (Widget.tsx +\nWidget.stories.tsx with decorators + argTypes\noverride + play() + Widget.mdx docs page) under\ntests/stories/fixtures/parity/, closing the gap\nfound above"
  add_fixture_test:
    kind: process
    label: "Author tests/stories/parity_fixture.rs asserting\nbuild_stories_static compiles the fixture with zero\ndiagnostics, the emitted module preserves the\ndecorators/argTypes/play source text, and the MDX\ndocs page wires to the Interactive story"
  verify:
    kind: process
    label: "cargo test -p jet --test stories_parity_fixture;\ngrep source_units for mdx.rs/optimizer.rs; aw td\ncheck; aw capability report --project jet"
  done:
    kind: terminal
    label: "README enumerates the full parity surface with\nconcrete citations; jet-stories.md source_units\nlist is complete; a passing in-repo fixture proves\ndecorators+play+argTypes+MDX work together without\nan external Storybook install"
edges:
  - { from: audit_closed_wis, to: classify_area }
  - { from: classify_area, to: cite_existing, label: "yes" }
  - { from: classify_area, to: gap_found, label: "no (combined play scenario)" }
  - { from: cite_existing, to: more_areas }
  - { from: gap_found, to: more_areas }
  - { from: more_areas, to: classify_area, label: "yes" }
  - { from: more_areas, to: rewrite_readme, label: "no" }
  - { from: rewrite_readme, to: update_schema }
  - { from: update_schema, to: design_fixture }
  - { from: design_fixture, to: add_fixture_test }
  - { from: add_fixture_test, to: verify }
  - { from: verify, to: done }
---
flowchart TD
    audit_closed_wis([Audit epic #1001's 15 closed member WIs for concrete test paths added]) --> classify_area{Does an existing cargo test already prove this feature area?}
    classify_area -->|yes| cite_existing[Cite the concrete existing test path per feature area]
    classify_area -->|no combined play scenario| gap_found[GAP: no in-repo evidence proves decorators+play+argTypes+MDX together]
    cite_existing --> more_areas{More feature areas left to classify?}
    gap_found --> more_areas
    more_areas -->|yes| classify_area
    more_areas -->|no| rewrite_readme[Rewrite README Component Workbench row + detailed block with cataloged citations]
    rewrite_readme --> update_schema[Add mdx.rs and optimizer.rs to jet-stories.md source_units]
    update_schema --> design_fixture[Design decorators+argTypes+play+MDX fixture under tests/stories/fixtures/parity/]
    design_fixture --> add_fixture_test[Author tests/stories/parity_fixture.rs static-build verification]
    add_fixture_test --> verify[cargo test; grep source_units; aw td check; aw capability report]
    verify --> done([README enumerates full surface; source_units complete; fixture passes])
```

## Config
<!-- type: config lang: yaml -->

```yaml
(fill)
```
