---
id: jet-project-architecture-and-authoring-clarity
summary: >
  Jet moves the project-root uppercase meta doc projects/jet/LAYOUT.md into
  the scoped project doc projects/jet/docs/architecture/layout.md, preserving
  the top-level path map and crate/package naming conventions, and relinks
  the Jet README so the layout guide stays discoverable without a
  project-root uppercase meta doc.
capability_refs:
  - id: jet-project-architecture-and-authoring-clarity
    role: primary
    gap: move-root-layout-meta-doc-into-scoped-architecture-documentation
    claim: move-root-layout-meta-doc-into-scoped-architecture-documentation
    coverage: full
    rationale: "Defines the bounded root-layout-meta-doc-to-scoped-doc migration requested by WI #1169, pinned to the 'Move root layout meta doc into scoped architecture documentation' work root."
fill_sections: [logic, config, unit-test, changes]
---

# jet: move root layout meta doc into scoped architecture documentation

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-project-architecture-and-authoring-clarity-flow
entry: inventory
nodes:
  inventory:
    kind: start
    label: "Inventory every reference to projects/jet/LAYOUT.md\n(the root file itself + the README Source map pointer row)"
  classify:
    kind: decision
    label: "Is this reference the live root meta-doc file content,\nor a pointer to it (e.g. README Source map row)?"
  migrate_content:
    kind: process
    label: "Copy the top-level path map + crate/package naming\nconventions verbatim into\nprojects/jet/docs/architecture/layout.md"
  remove_root_doc:
    kind: process
    label: "Delete projects/jet/LAYOUT.md so no project-root\nuppercase meta doc remains for jet"
  update_pointer:
    kind: process
    label: "Update projects/jet/README.md Source map row to\nreference projects/jet/docs/architecture/layout.md"
  verify_scope:
    kind: process
    label: "Grep-scan the repo for LAYOUT.md references and confirm\nREADME.md/CAPABILITIES.md remain the only jet\nproject-root uppercase meta docs"
  stale:
    kind: decision
    label: "Does any live reference still point at the retired\nprojects/jet/LAYOUT.md path?"
  fix:
    kind: process
    label: "Fix the stale projects/jet/LAYOUT.md reference"
  done:
    kind: terminal
    label: "projects/jet/LAYOUT.md no longer exists; the layout guide\nis discoverable at projects/jet/docs/architecture/layout.md\nand linked from README"
edges:
  - from: inventory
    to: classify
    label: "reference classified"
  - from: classify
    to: migrate_content
    label: "root doc content"
  - from: classify
    to: update_pointer
    label: "pointer to root doc"
  - from: migrate_content
    to: remove_root_doc
    label: "content preserved in scoped doc"
  - from: remove_root_doc
    to: verify_scope
    label: "root doc removed"
  - from: update_pointer
    to: verify_scope
    label: "pointer updated"
  - from: verify_scope
    to: stale
    label: "scan complete"
  - from: stale
    to: fix
    label: "yes"
  - from: fix
    to: verify_scope
    label: "retry"
  - from: stale
    to: done
    label: "no"
---
flowchart TD
    inventory([Inventory every projects/jet/LAYOUT.md reference]) --> classify{Root doc content or pointer to it?}
    classify -- root doc content --> migrate_content[Copy path map + naming conventions into docs/architecture/layout.md]
    classify -- pointer to root doc --> update_pointer[Update README Source map row]
    migrate_content --> remove_root_doc[Delete projects/jet/LAYOUT.md]
    remove_root_doc --> verify_scope[Scan repo for LAYOUT.md references; confirm only README/CAPABILITIES remain root uppercase meta docs]
    update_pointer --> verify_scope
    verify_scope --> stale{Any live reference still points at retired path?}
    stale -- yes --> fix[Fix the stale reference]
    fix --> verify_scope
    stale -- no --> done([Layout guide discoverable at docs/architecture/layout.md, linked from README])
```

## Config
<!-- type: config lang: yaml -->

```yaml
meta_doc_migration:
  project: jet
  legacy_doc_path: projects/jet/LAYOUT.md
  canonical_doc_path: projects/jet/docs/architecture/layout.md
  preserved_content:
    - top_level_path_map
    - crate_package_naming_conventions
    - two_halves_of_jet_table
    - parity_workspace_table
  pointer_updates:
    - path: projects/jet/README.md
      location: "Source map table"
      from: "`projects/jet/LAYOUT.md`"
      to: "`projects/jet/docs/architecture/layout.md`"
  remaining_root_uppercase_meta_docs:
    - projects/jet/README.md
  verification:
    local_checks:
      - aw td check projects/jet/tech-design/logic/move-root-layout-meta-doc-into-scoped-architecture-documentation.md
      - aw capability report --project jet
    stale_source_scan:
      command: rg -n "projects/jet/LAYOUT.md" --glob '!projects/jet/tech-design/**'
      allowed_contexts:
        - projects/jet/tech-design (historical TD text that explicitly names the retired root doc)
```
