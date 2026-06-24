---
id: projects-jet-logic-jet-pack-publish-honor-package-json-files-allowlist-npmignore-md
fill_sections: [logic, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: publish-and-private-registry
    coverage: partial
    rationale: "jet pack/publish honors package.json files allowlist"
---

# jet pack/publish honors package.json files allowlist

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-pack-files
entry: collect
nodes:
  collect: { kind: start,    label: collect_publish_files gathers tarball entries }
  hasfiles: { kind: decision, label: package.json files allowlist present? }
  filesonly: { kind: process,  label: include ONLY files-glob matches }
  npmignore: { kind: decision, label: .npmignore present? }
  ignore: { kind: process,  label: default tree minus .npmignore patterns }
  deflist: { kind: process,  label: default skip-list (node_modules/.git/...) }
  always: { kind: process,  label: always add package.json + README/LICENSE }
  done: { kind: terminal, label: final tarball file list }
edges:
  - { from: collect,   to: hasfiles }
  - { from: hasfiles,  to: filesonly, label: yes }
  - { from: hasfiles,  to: npmignore, label: no }
  - { from: npmignore, to: ignore,    label: yes }
  - { from: npmignore, to: deflist,   label: no }
  - { from: filesonly, to: always }
  - { from: ignore,    to: always }
  - { from: deflist,   to: always }
  - { from: always,    to: done }
---
flowchart TD
    collect([collect_publish_files]) --> hasfiles{files allowlist?}
    hasfiles -->|yes| filesonly[only files-glob matches]
    hasfiles -->|no| npmignore{.npmignore?}
    npmignore -->|yes| ignore[tree minus ignored]
    npmignore -->|no| deflist[default skip-list]
    filesonly --> always[always add package.json + README/LICENSE]
    ignore --> always
    deflist --> always
    always --> done([tarball file list])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/pkg_manager/publish.rs"
    action: modify
    section: logic
    description: |
      Honor package.json files allowlist (glob) in collect_publish_files; .npmignore fallback when files absent; always include package.json + README/LICENSE; keep default skip-list as last resort.
  - path: "projects/jet/src/resolver/package.rs"
    action: modify
    section: logic
    description: |
      Add a files: Option<Vec<String>> field to PackageJson if not present, for the allowlist.
  - path: "projects/jet/src/pkg_manager/publish.rs"
    action: modify
    section: unit-test
    description: |
      Tests: files=[dist] packs only dist+package.json+README; .npmignore excludes patterns; always-include files present.
```

