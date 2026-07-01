---
id: projects-jet-logic-jet-pack-publish-workspace-dependencies-are-not-rewritten-to-rea-md
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: publish-and-private-registry
    coverage: partial
    rationale: "jet pack/publish must rewrite workspace protocol dependency ranges before writing package.json into the tarball or registry publish payload."
---

# jet pack/publish: Workspace Dependencies Rewrite Before Packaging

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-pack-publish-workspace-deps-contract
entry: read_manifest
nodes:
  read_manifest: { kind: start, label: "Read publish package.json" }
  scan_ancestor: { kind: process, label: "Try WorkspaceManager::discover on package dir and ancestors" }
  usable_ws: { kind: decision, label: "Ancestor contains this package and dependency targets?" }
  no_ws: { kind: process, label: "Leave manifest unchanged when no workspace exists" }
  field_loop: { kind: process, label: "Visit dependencies/devDependencies/peerDependencies/optionalDependencies" }
  workspace_spec: { kind: decision, label: "String value starts with workspace:?" }
  resolve: { kind: process, label: "Resolve target package version and range operator" }
  replace: { kind: process, label: "Replace workspace spec with npm-compatible range" }
  emit: { kind: terminal, label: "Emit transformed manifest to tarball and publish JSON" }
edges:
  - { from: read_manifest, to: scan_ancestor }
  - { from: scan_ancestor, to: usable_ws }
  - { from: usable_ws, to: field_loop, label: "yes" }
  - { from: usable_ws, to: no_ws, label: "no" }
  - { from: no_ws, to: emit }
  - { from: field_loop, to: workspace_spec }
  - { from: workspace_spec, to: resolve, label: "yes" }
  - { from: workspace_spec, to: emit, label: "no more matches" }
  - { from: resolve, to: replace }
  - { from: replace, to: field_loop }
---
flowchart TD
    read_manifest([Read package.json]) --> scan_ancestor[Discover workspace from package dir and ancestors]
    scan_ancestor --> usable_ws{Workspace can resolve this package's deps?}
    usable_ws -->|no| no_ws[No workspace transform]
    no_ws --> emit([Emit manifest])
    usable_ws -->|yes| field_loop[Visit dependencies, devDependencies, peerDependencies, optionalDependencies]
    field_loop --> workspace_spec{workspace: string?}
    workspace_spec -->|yes| resolve[Resolve target package version/range]
    resolve --> replace[Replace with registry-compatible semver range]
    replace --> field_loop
    workspace_spec -->|no more matches| emit
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-pack-publish-workspace-deps-contract-tests
requirements:
  R1:
    text: "A Publisher created for packages/widget discovers the pnpm workspace declared at the temp repository root."
    risk: high
    verify: unit
  R2:
    text: "Publisher::pack writes package/package.json with dependencies exact, peerDependencies caret, and optionalDependencies tilde ranges resolved from sibling package versions."
    risk: high
    verify: unit
  R3:
    text: "The packed manifest serialization contains no workspace: literal after transformation."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Publisher walks to workspace root"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Packed manifest rewrites all publish dependency fields"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "No workspace protocol leaks to tarball"
  risk: High
  verifymethod: Test
}
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
      Discover workspace context by walking ancestors from the publishing
      package directory, then rewrite workspace protocol ranges in
      dependencies, devDependencies, peerDependencies, and
      optionalDependencies before package.json is written into the tarball or
      publish body.
    impl_mode: hand-written
  - path: "projects/jet/tests/publish/library_publish_e2e.rs"
    action: modify
    section: unit-test
    description: |
      Add a pack regression with a package nested under a pnpm workspace root.
      Assert the packed package.json rewrites workspace:*, workspace:^, and
      workspace:~ ranges across publish-relevant dependency fields and contains
      no workspace: protocol literals.
    impl_mode: hand-written
```
