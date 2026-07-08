---
id: defer-app-source-root-taxonomy
summary: >
  Defer moves its app-facing source root from projects/defer to apps/defer while
  preserving project identity, GitHub labels, persistent branch conventions, and
  the existing TD bucket identity.
capability_refs:
  - id: defer-app-source-root-taxonomy
    role: primary
    gap: defer-app-source-root-taxonomy
    claim: defer-app-source-root-taxonomy
    coverage: full
    rationale: "Defines the bounded source-root taxonomy migration requested by WI #1217."
fill_sections: [logic, config, unit-test, changes]
---

# defer app source-root taxonomy

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-app-source-root-taxonomy-flow
entry: inventory
nodes:
  inventory:
    kind: start
    label: "Inventory every projects/defer reference"
  classify:
    kind: decision
    label: "Is this reference the live source root contract?"
  preserve:
    kind: process
    label: "Preserve TD bucket, historical notes, and project identity references"
  migrate:
    kind: process
    label: "Rewrite live source-root references to apps/defer"
  move_tree:
    kind: process
    label: "Move the Defer source tree to apps/defer"
  route:
    kind: process
    label: "Update Cargo, AW config, README inventory, scripts, tests, and evidence paths"
  smoke:
    kind: process
    label: "Run AW project resolution and targeted Defer verification"
  stale:
    kind: decision
    label: "Does a live command still emit projects/defer as source root?"
  fix:
    kind: process
    label: "Fix the stale runtime source-root reference"
  done:
    kind: terminal
    label: "Defer resolves through apps/defer while project identity remains project:defer"
edges:
  - from: inventory
    to: classify
    label: "reference classified"
  - from: classify
    to: preserve
    label: "not source root"
  - from: classify
    to: migrate
    label: "source root"
  - from: preserve
    to: smoke
    label: "kept intentionally"
  - from: migrate
    to: move_tree
    label: "paths rewritten"
  - from: move_tree
    to: route
    label: "tree moved"
  - from: route
    to: smoke
    label: "routing updated"
  - from: smoke
    to: stale
    label: "checks complete"
  - from: stale
    to: fix
    label: "yes"
  - from: fix
    to: smoke
    label: "retry"
  - from: stale
    to: done
    label: "no"
---
flowchart TD
    inventory([Inventory every projects/defer reference]) --> classify{Live source root contract?}
    classify -- no --> preserve[Preserve TD bucket and historical/project identity references]
    classify -- yes --> migrate[Rewrite live source-root references to apps/defer]
    migrate --> move_tree[Move source tree to apps/defer]
    move_tree --> route[Update Cargo, AW config, README inventory, scripts, tests, and evidence paths]
    preserve --> smoke[Run AW project resolution and targeted Defer verification]
    route --> smoke
    smoke --> stale{Live command still emits projects/defer as source root?}
    stale -- yes --> fix[Fix stale runtime source-root reference]
    fix --> smoke
    stale -- no --> done([Defer resolves through apps/defer while identity remains project:defer])
```
## Config
<!-- type: config lang: yaml -->

```yaml
repo_taxonomy_migration:
  project: defer
  canonical_source_root: apps/defer
  legacy_source_root: projects/defer
  preserved_identity:
    aw_project: defer
    github_label: project:defer
    persistent_branch: project-defer
    td_bucket: projects/defer/tech-design
  rewrite_classes:
    - root_readme_inventory_link
    - cargo_workspace_member
    - aw_project_path
    - aw_cap_path
    - project_local_aw_toml_path
    - install_or_build_script_path
    - test_fixture_or_manifest_path
    - generated_evidence_source_root_path
  preserve_classes:
    - historical_release_notes
    - issue_body_reference_context
    - td_bucket_path
    - git_branch_name
    - github_label
  verification:
    aw_resolution:
      - aw wi show 1217
      - aw wi list --project defer --state open
      - aw capability check --project defer
    local_checks:
      - cargo test -p defer
      - aw td check projects/defer/tech-design
    stale_source_root_scan:
      command: rg -n "projects/defer" README.md CONTRIBUTING.md aw.toml .aw/config.toml Cargo.toml apps projects .github
      allowed_contexts:
        - projects/defer/tech-design
        - historical text that explicitly names the retired source root
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: defer-app-source-root-taxonomy-verification
requirements:
  defer_local_checks_still_run:
    id: R4
    text: "Defer's targeted local verification continues to run after the source-root move."
    kind: functional
    risk: medium
    verify: cargo test -p defer
  project_identity_is_preserved:
    id: R2
    text: "The migration preserves AW project name defer, GitHub label project:defer, persistent branch project-defer, and the projects/defer/tech-design TD bucket."
    kind: regression
    risk: medium
    verify: aw wi list --project defer --state open
  source_root_routes_to_apps_defer:
    id: R1
    text: "Repository routing resolves Defer's live app source root through apps/defer rather than projects/defer."
    kind: functional
    risk: high
    verify: aw capability check --project defer
  stale_source_root_references_are_bounded:
    id: R3
    text: "No live source-root command or routing artifact emits projects/defer except intentionally preserved TD or historical references."
    kind: regression
    risk: medium
    verify: rg stale projects/defer source-root scan
---
flowchart TD
    r1[R1 source root routes to apps defer] --> aw_capability_check_project_defer[aw capability check --project defer]
    r2[R2 project identity is preserved] --> aw_wi_list_project_defer_state_open[aw wi list --project defer --state open]
    r3[R3 stale source root references are bounded] --> rg_stale_projects_defer_source_root_scan[rg stale projects/defer source-root scan]
    r4[R4 defer local checks still run] --> cargo_test_p_defer[cargo test -p defer]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/defer
    action: move
    target: apps/defer
    section: logic
    impl_mode: hand-written
    reason: "Defer's live app source root moves to the apps/ taxonomy."
  - path: README.md
    action: update
    section: config
    impl_mode: hand-written
    reason: "Root project inventory and install/discovery links should point at apps/defer for Defer."
  - path: Cargo.toml
    action: update
    section: config
    impl_mode: hand-written
    reason: "Workspace membership and package routing must resolve the Defer crate under apps/defer."
  - path: aw.toml
    action: update
    section: config
    impl_mode: hand-written
    reason: "Project discovery must include app-local aw.toml files so defer resolves through apps/defer."
  - path: .aw/config.toml
    action: update
    section: config
    impl_mode: hand-written
    reason: "AW project path and cap_path for project defer must point at apps/defer while retaining name defer and label project:defer."
  - path: apps/defer/README.md
    action: update
    section: config
    impl_mode: hand-written
    reason: "Project-local capability docs and verification paths should use apps/defer or path-relative references for the live source root."
  - path: apps/defer/aw.toml
    action: update
    section: config
    impl_mode: hand-written
    reason: "Project-local AW metadata should describe the moved app path without changing project identity."
  - path: apps/defer/tests
    action: update
    section: unit-test
    impl_mode: hand-written
    reason: "Any project-local tests or manifests that hard-code projects/defer as source root should use apps/defer or relative paths."
  - path: projects/defer/tech-design
    action: preserve
    section: config
    impl_mode: hand-written
    reason: "The issue explicitly keeps the TD project bucket under projects/defer/tech-design until a separate TD-platform migration exists."
```
