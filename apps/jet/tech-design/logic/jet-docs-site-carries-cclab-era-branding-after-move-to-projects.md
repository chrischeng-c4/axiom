---
id: jet-docs-site-rebrand-and-orphan-nav-linking
summary: >
  jet's VitePress docs site under `projects/jet/docs/` still carries
  cclab-era branding left over from the repo-root-to-projects/jet/docs
  relocation (41f4bf9be): `index.md` hero.name is `cclab` plus stray
  Mamba/SDD "(Docs coming soon)" placeholder features that are not jet
  content, `.vitepress/config.mjs` title is `cclab`, and `package.json`
  name is `cclab-docs`. Separately, that same relocation merged jet's
  pre-existing hand-written design-note markdown files into
  `projects/jet/docs/` without adding them to the VitePress nav/sidebar,
  so they render as unlinked static pages. This TD rebrands the site to
  jet's identity, removes the non-jet placeholder features, and adds
  nav/sidebar entries for every orphaned design-note file, closing WI
  #1083.
capability_refs:
  - id: jet-project-architecture-and-authoring-clarity
    role: primary
    gap: rebrand-jet-docs-site-and-nav-link-orphaned-design-notes
    claim: rebrand-jet-docs-site-and-nav-link-orphaned-design-notes
    coverage: full
    rationale: "Defines the bounded docs-site rebrand + orphaned-design-note nav-linking requested by WI #1083, pinned to the 'Rebrand jet docs site and nav-link orphaned design notes' work root under the existing Jet Project Architecture And Authoring Clarity capability."
fill_sections: [logic, config, unit-test, changes]
---

# jet: docs site carries cclab-era branding after move to projects/jet/docs

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-docs-site-rebrand-and-orphan-nav-linking-flow
entry: rebrand
nodes:
  rebrand:
    kind: start
    label: "Rebrand projects/jet/docs/index.md hero.name,\n.vitepress/config.mjs title, and package.json name\nfrom cclab-era branding to jet identity"
  strip_placeholders:
    kind: process
    label: "Remove the Mamba and SDD '(Docs coming soon)'\nplaceholder features from index.md;\nkeep only the Jet feature card"
  enumerate_docs:
    kind: process
    label: "Glob every markdown file under projects/jet/docs/\n(excluding node_modules/.vitepress/dist/.vitepress/cache)"
  diff_against_nav:
    kind: process
    label: "Grep .vitepress/config.mjs nav+sidebar link targets;\ndiff against the enumerated markdown file set"
  orphan_found:
    kind: decision
    label: "Is this markdown file's link path\nabsent from nav/sidebar?"
  add_nav_entry:
    kind: process
    label: "Add a sidebar item (grouped by topic:\nArchitecture or Design Notes) linking to the\norphaned file's path"
  linked_already:
    kind: process
    label: "Leave the existing nav/sidebar entry unchanged"
  more_files:
    kind: decision
    label: "More enumerated markdown files left to classify?"
  verify_no_cclab:
    kind: process
    label: "Grep projects/jet/docs/ for residual 'cclab' branding\nand for the retired Mamba/SDD placeholder text"
  verify_all_linked:
    kind: process
    label: "Re-diff nav/sidebar link targets against the full\nmarkdown file set; confirm zero orphans remain"
  done:
    kind: terminal
    label: "index.md, .vitepress/config.mjs, and package.json\ncarry jet identity with no cclab branding or stray\nplaceholder features; every design-note markdown file\nis reachable from the VitePress nav/sidebar"
edges:
  - from: rebrand
    to: strip_placeholders
    label: "branding fields rewritten"
  - from: strip_placeholders
    to: enumerate_docs
    label: "placeholder features removed"
  - from: enumerate_docs
    to: diff_against_nav
    label: "file set enumerated"
  - from: diff_against_nav
    to: orphan_found
    label: "per-file classification"
  - from: orphan_found
    to: add_nav_entry
    label: "yes"
  - from: orphan_found
    to: linked_already
    label: "no"
  - from: add_nav_entry
    to: more_files
    label: "entry added"
  - from: linked_already
    to: more_files
    label: "already discoverable"
  - from: more_files
    to: diff_against_nav
    label: "yes"
  - from: more_files
    to: verify_no_cclab
    label: "no"
  - from: verify_no_cclab
    to: verify_all_linked
    label: "no residual branding found"
  - from: verify_all_linked
    to: done
    label: "zero orphans remain"
---
flowchart TD
    rebrand([Rebrand index.md hero, vitepress config title, package.json name to jet identity]) --> strip_placeholders[Remove Mamba/SDD Docs coming soon placeholder features]
    strip_placeholders --> enumerate_docs[Enumerate every markdown file under projects/jet/docs/]
    enumerate_docs --> diff_against_nav[Diff enumerated files against vitepress nav/sidebar link targets]
    diff_against_nav --> orphan_found{Is file's link path absent from nav/sidebar?}
    orphan_found -- yes --> add_nav_entry[Add grouped sidebar entry for the orphaned file]
    orphan_found -- no --> linked_already[Leave existing nav/sidebar entry unchanged]
    add_nav_entry --> more_files{More files to classify?}
    linked_already --> more_files
    more_files -- yes --> diff_against_nav
    more_files -- no --> verify_no_cclab[Grep docs tree for residual cclab branding and retired placeholder text]
    verify_no_cclab --> verify_all_linked[Re-diff nav/sidebar against full file set]
    verify_all_linked --> done([Site carries jet identity; every design-note file is nav-discoverable])
```
## Config
<!-- type: config lang: yaml -->

```yaml
docs_site_rebrand:
  project: jet
  site_root: projects/jet/docs
  branding_fixes:
    - path: projects/jet/docs/index.md
      field: "frontmatter hero.name"
      from: "cclab"
      to: "Jet"
    - path: projects/jet/docs/.vitepress/config.mjs
      field: "defineConfig({ title })"
      from: "cclab"
      to: "Jet"
    - path: projects/jet/docs/package.json
      field: "name"
      from: "cclab-docs"
      to: "jet-docs"
  placeholder_features_removed:
    - path: projects/jet/docs/index.md
      field: "frontmatter features[]"
      removed:
        - title: "Mamba"
          details: "Force-typed Python compiler with Cranelift backend. (Docs coming soon)"
        - title: "SDD"
          details: "Spec-Driven Development workflow engine. (Docs coming soon)"
      retained:
        - title: "Jet"
  nav_sidebar_additions:
    config_path: projects/jet/docs/.vitepress/config.mjs
    sidebar_root: "themeConfig.sidebar['/']"
    existing_group: "Jet"
    new_groups:
      - group: "Architecture"
        items:
          - text: "Project Layout"
            link: "/architecture/layout"
            source_file: projects/jet/docs/architecture/layout.md
          - text: "Source-Tree Reorg Plan"
            link: "/architecture/reorg-plan"
            source_file: projects/jet/docs/architecture/reorg-plan.md
      - group: "Design Notes"
        items:
          - text: "Build Fails Loudly on Unresolved Bare Specifiers"
            link: "/build-fails-loudly-on-unresolved-bare-specifiers"
            source_file: projects/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
          - text: "Check Exits Non-Zero While Unimplemented"
            link: "/check-exits-non-zero-while-unimplemented"
            source_file: projects/jet/docs/check-exits-non-zero-while-unimplemented.md
          - text: "Dev Server Source Analysis UTF-8 Safety"
            link: "/dev-server-source-analysis-utf8-safety"
            source_file: projects/jet/docs/dev-server-source-analysis-utf8-safety.md
          - text: "Layout Box Model -- Slice 7a"
            link: "/layout-box-slice-7a"
            source_file: projects/jet/docs/layout-box-slice-7a.md
          - text: "Wasm Config Accepts Shared Jet Sections"
            link: "/wasm-config-accept-shared-jet-sections"
            source_file: projects/jet/docs/wasm-config-accept-shared-jet-sections.md
          - text: "Wasm Transpiler Boolean useState Literals"
            link: "/wasm-transpiler-boolean-usestate-literals"
            source_file: projects/jet/docs/wasm-transpiler-boolean-usestate-literals.md
    existing_group_additions:
      - text: "OpenAPI Codegen"
        link: "/openapi-codegen"
        source_file: projects/jet/docs/openapi-codegen.md
      - text: "Library Publishing"
        link: "/library-publishing"
        source_file: projects/jet/docs/library-publishing.md
      - text: "Migration from Playwright"
        link: "/migration-from-playwright"
        source_file: projects/jet/docs/migration-from-playwright.md
  out_of_scope:
    - "projects/jet/docs/.vitepress/config.mjs themeConfig.socialLinks github URL still points at the retired 'cclab' repo slug; WI #1083 scopes only hero.name, vitepress title, and package.json name, so this stale link is left for a follow-up WI rather than re-litigated here."
  verification:
    tooling_availability:
      node: "present via nvm (node v22.18.0, npm bundled)"
      vitepress_install: "absent -- no node_modules under projects/jet/docs or repo root; installing vitepress requires network access out of scope for this hand-written content/config change"
      acceptance_scope: "content and config correctness verified via grep/read, not a live `npm run build`; optional manual verification once deps are installed: npm --prefix projects/jet/docs install && npm --prefix projects/jet/docs run build"
    local_checks:
      - "aw td check projects/jet/tech-design/logic/jet-docs-site-carries-cclab-era-branding-after-move-to-projects.md"
      - "aw capability report --project jet"
    branding_scan:
      command: "grep -rn 'cclab' projects/jet/docs/index.md projects/jet/docs/.vitepress/config.mjs projects/jet/docs/package.json"
      expected: "no matches for hero.name, title, or package.json name fields"
    placeholder_scan:
      command: "grep -n 'Mamba\\|SDD' projects/jet/docs/index.md"
      expected: "no matches"
    orphan_scan:
      command: "for f in $(find projects/jet/docs -name '*.md' -not -path '*/node_modules/*'); do link=\"/${f#projects/jet/docs/}\"; link=\"${link%.md}\"; grep -q -- \"$link\" projects/jet/docs/.vitepress/config.mjs || echo \"orphan: $f\"; done"
      expected: "no output (every markdown file's link path appears in the nav/sidebar config)"
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-docs-site-rebrand-and-orphan-nav-linking-verification
requirements:
  every_design_note_nav_linked:
    id: R6
    text: "Every pre-existing hand-written design-note markdown file under projects/jet/docs/ (the 11 files the doc-relocation left unlinked: architecture/layout.md, architecture/reorg-plan.md, build-fails-loudly-on-unresolved-bare-specifiers.md, check-exits-non-zero-while-unimplemented.md, dev-server-source-analysis-utf8-safety.md, layout-box-slice-7a.md, library-publishing.md, migration-from-playwright.md, openapi-codegen.md, wasm-config-accept-shared-jet-sections.md, wasm-transpiler-boolean-usestate-literals.md) has a corresponding nav/sidebar entry in .vitepress/config.mjs."
    kind: functional
    risk: medium
    verify: for f in architecture/layout architecture/reorg-plan build-fails-loudly-on-unresolved-bare-specifiers check-exits-non-zero-while-unimplemented dev-server-source-analysis-utf8-safety layout-box-slice-7a library-publishing migration-from-playwright openapi-codegen wasm-config-accept-shared-jet-sections wasm-transpiler-boolean-usestate-literals; do grep -q -- "/$f" projects/jet/docs/.vitepress/config.mjs || exit 1; done
  hero_name_rebranded:
    id: R1
    text: "projects/jet/docs/index.md frontmatter hero.name is 'Jet', not 'cclab'."
    kind: functional
    risk: low
    verify: grep -q "name: Jet" projects/jet/docs/index.md && ! grep -q "name: cclab" projects/jet/docs/index.md
  no_orphaned_markdown_remains:
    id: R7
    text: "No markdown file under projects/jet/docs/ (excluding node_modules/.vitepress build output) is absent from the nav/sidebar link set."
    kind: regression
    risk: medium
    verify: for f in $(find projects/jet/docs -name '*.md' -not -path '*/node_modules/*'); do link="/${f#projects/jet/docs/}"; link="${link%.md}"; grep -q -- "$link" projects/jet/docs/.vitepress/config.mjs || exit 1; done
  no_residual_cclab_branding:
    id: R5
    text: "No residual 'cclab' string remains in index.md, .vitepress/config.mjs, or package.json."
    kind: regression
    risk: medium
    verify: ! grep -rn 'cclab' projects/jet/docs/index.md projects/jet/docs/.vitepress/config.mjs projects/jet/docs/package.json
  package_json_name_rebranded:
    id: R4
    text: "projects/jet/docs/package.json name is a jet-scoped docs package name, not 'cclab-docs'."
    kind: functional
    risk: low
    verify: grep -q '"name": "jet-docs"' projects/jet/docs/package.json && ! grep -q 'cclab-docs' projects/jet/docs/package.json
  placeholder_features_removed:
    id: R2
    text: "projects/jet/docs/index.md no longer lists the Mamba or SDD '(Docs coming soon)' placeholder features; only the Jet feature entry remains."
    kind: functional
    risk: low
    verify: ! grep -Eq 'title: Mamba|title: SDD' projects/jet/docs/index.md
  vitepress_title_rebranded:
    id: R3
    text: "projects/jet/docs/.vitepress/config.mjs defineConfig title is jet-branded, not 'cclab'."
    kind: functional
    risk: low
    verify: grep -q "title: 'Jet'" projects/jet/docs/.vitepress/config.mjs && ! grep -q "title: 'cclab'" projects/jet/docs/.vitepress/config.mjs
---
flowchart TD
    r1[R1 hero name rebranded] --> grep_q_name_jet_projects_jet_docs_index_md_grep_q_name_cclab_projects_jet_docs_index_md[grep -q "name: Jet" projects/jet/docs/index.md && ! grep -q "name: cclab" projects/jet/docs/index.md]
    r2[R2 placeholder features removed] --> grep_eq_title_mamba_title_sdd_projects_jet_docs_index_md[! grep -Eq 'title: Mamba|title: SDD' projects/jet/docs/index.md]
    r3[R3 vitepress title rebranded] --> grep_q_title_jet_projects_jet_docs_vitepress_config_mjs_grep_q_title_cclab_projects_jet_docs_vitepress_config_mjs[grep -q "title: 'Jet'" projects/jet/docs/.vitepress/config.mjs && ! grep -q "title: 'cclab'" projects/jet/docs/.vitepress/config.mjs]
    r4[R4 package json name rebranded] --> grep_q_name_jet_docs_projects_jet_docs_package_json_grep_q_cclab_docs_projects_jet_docs_package_json[grep -q '"name": "jet-docs"' projects/jet/docs/package.json && ! grep -q 'cclab-docs' projects/jet/docs/package.json]
    r5[R5 no residual cclab branding] --> grep_rn_cclab_projects_jet_docs_index_md_projects_jet_docs_vitepress_config_mjs_projects_jet_docs_package_json[! grep -rn 'cclab' projects/jet/docs/index.md projects/jet/docs/.vitepress/config.mjs projects/jet/docs/package.json]
    r6[R6 every design note nav linked] --> for_f_in_architecture_layout_architecture_reorg_plan_build_fails_loudly_on_unresolved_bare_specifiers_check_exits_non_zero_while_unimplemented_dev_server_source_analysis_utf8_safety_layout_box_slice_7a_library_publishing_migration_from_playwright_openapi_codegen_wasm_config_accept_shared_jet_sections_wasm_transpiler_boolean_usestate_literals_do_grep_q_f_projects_jet_docs_vitepress_config_mjs_exit_1_done[for f in architecture/layout architecture/reorg-plan build-fails-loudly-on-unresolved-bare-specifiers check-exits-non-zero-while-unimplemented dev-server-source-analysis-utf8-safety layout-box-slice-7a library-publishing migration-from-playwright openapi-codegen wasm-config-accept-shared-jet-sections wasm-transpiler-boolean-usestate-literals; do grep -q -- "/$f" projects/jet/docs/.vitepress/config.mjs || exit 1; done]
    r7[R7 no orphaned markdown remains] --> for_f_in_find_projects_jet_docs_name_md_not_path_node_modules_do_link_f_projects_jet_docs_link_link_md_grep_q_link_projects_jet_docs_vitepress_config_mjs_exit_1_done[for f in $(find projects/jet/docs -name '*.md' -not -path '*/node_modules/*'); do link="/${f#projects/jet/docs/}"; link="${link%.md}"; grep -q -- "$link" projects/jet/docs/.vitepress/config.mjs || exit 1; done]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/jet/docs/index.md
    action: update
    section: config
    impl_mode: hand-written
    reason: "Rebrands frontmatter hero.name from 'cclab' to 'Jet' and removes the Mamba/SDD '(Docs coming soon)' placeholder features that are not jet content, keeping only the Jet feature entry, per docs_site_rebrand.branding_fixes and .placeholder_features_removed in the Config section."
  - path: projects/jet/docs/.vitepress/config.mjs
    action: update
    section: config
    impl_mode: hand-written
    reason: "Rebrands defineConfig title from 'cclab' to 'Jet' and adds the Architecture and Design Notes sidebar groups plus three existing-group items so every orphaned design-note markdown file becomes nav-discoverable, per docs_site_rebrand.branding_fixes and .nav_sidebar_additions in the Config section."
  - path: projects/jet/docs/package.json
    action: update
    section: config
    impl_mode: hand-written
    reason: "Renames the package from 'cclab-docs' to the jet-scoped 'jet-docs', per docs_site_rebrand.branding_fixes in the Config section."
  - path: projects/jet/README.md
    action: update
    section: config
    impl_mode: hand-written
    reason: "Registers the rebrand-jet-docs-site-and-nav-link-orphaned-design-notes work root (WI #1083) under the existing Jet Project Architecture And Authoring Clarity capability so this migration is capability-tracked; already applied ahead of this TD per standard aw-td-writer capability-registration practice."
```
