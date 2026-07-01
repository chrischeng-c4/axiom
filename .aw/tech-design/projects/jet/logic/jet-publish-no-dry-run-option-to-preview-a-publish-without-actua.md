---
id: projects-jet-logic-jet-publish-no-dry-run-option-to-preview-a-publish-without-actua-md
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: publish-and-private-registry
    coverage: partial
    rationale: "jet publish needs a dry-run preview path that exercises pack/metadata/auth preparation without uploading to the registry."
---

# jet publish: Dry-Run Preview Without Registry Upload

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-publish-dry-run-contract
entry: cli
nodes:
  cli: { kind: start, label: "CLI parses publish --dry-run" }
  common_prep: { kind: process, label: "Run common publish preparation" }
  auth_check: { kind: process, label: "Resolve registry and require local auth token" }
  tarball: { kind: process, label: "Create tarball bytes without writing or uploading" }
  preview: { kind: process, label: "Build preview report from manifest, registry, tag, access, files" }
  put_gate: { kind: decision, label: "dry-run flag set?" }
  print: { kind: terminal, label: "Print preview and return success" }
  put: { kind: process, label: "Real publish PUT path remains unchanged" }
edges:
  - { from: cli, to: common_prep }
  - { from: common_prep, to: auth_check }
  - { from: auth_check, to: tarball }
  - { from: tarball, to: preview }
  - { from: preview, to: put_gate }
  - { from: put_gate, to: print, label: "yes" }
  - { from: put_gate, to: put, label: "no" }
---
flowchart TD
    cli([publish --dry-run parsed]) --> common_prep[Common publish prep: transform, optional build, metadata validation]
    common_prep --> auth_check[Resolve registry and require local auth token]
    auth_check --> tarball[Create tarball bytes in memory]
    tarball --> preview[Build preview report]
    preview --> put_gate{dry-run?}
    put_gate -->|yes| print([Print preview; no PUT])
    put_gate -->|no| put[Existing registry PUT publish]
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-publish-dry-run-tests
requirements:
  R1:
    text: "publish command accepts --dry-run with existing --tag, --access, and --build flags."
    risk: high
    verify: unit
  R2:
    text: "Publisher dry-run returns transformed package.json, target registry/tag/access, tarball file name, and tarball entry list."
    risk: high
    verify: unit
  R3:
    text: "Dry-run does not execute the registry PUT upload path."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "CLI accepts --dry-run"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Preview contains manifest, registry, and files"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Dry-run performs no upload"
  risk: High
  verifymethod: Test
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/cli.rs"
    action: modify
    section: logic
    description: |
      Add `jet publish --dry-run` to the CLI and route it to the publisher
      dry-run preview path instead of the registry upload path.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/publish.rs"
    action: modify
    section: logic
    description: |
      Add a publish dry-run preview that performs the existing package
      transformation, optional build, metadata validation, registry/auth
      lookup, and in-memory tarball creation, then returns a printable preview
      without issuing the registry PUT request.
    impl_mode: hand-written
  - path: "projects/jet/tests/publish/library_publish_e2e.rs"
    action: modify
    section: unit-test
    description: |
      Add a dry-run regression that points publish at the in-process mock
      registry, asserts the preview shape, and verifies the mock store remains
      empty because no upload happened.
    impl_mode: hand-written
  - path: "projects/jet/src/cli.rs"
    action: modify
    section: unit-test
    description: |
      Add command parser coverage proving `publish --dry-run --tag --access`
      is accepted and sets the dry-run flag.
    impl_mode: hand-written
```
