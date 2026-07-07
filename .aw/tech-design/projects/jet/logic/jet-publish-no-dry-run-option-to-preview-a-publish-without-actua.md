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
id: jet-publish-dry-run-contract-tests
requirements:
  R1:
    text: "Command parser exposes publish --dry-run and it composes with --tag and --access."
    risk: high
    verify: unit
  R2:
    text: "Dry-run preview includes package name/version, registry URL, tag, access, tarball file name, tarball byte length, entries, and transformed package.json."
    risk: high
    verify: unit
  R3:
    text: "Dry-run against the mock registry leaves the mock publish store empty."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "CLI dry-run flag parsed"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Preview report is complete"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Mock registry receives no PUT"
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
      Register `publish --dry-run`, parse it with existing publish options, and
      print the publisher preview report instead of awaiting the upload method
      when the flag is set.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/publish.rs"
    action: modify
    section: logic
    description: |
      Factor common publish preparation into a dry-run-capable path that reads
      and transforms package.json, runs optional build/metadata validation,
      resolves registry/auth, creates tarball bytes, lists tarball entries, and
      formats a deterministic preview without sending an HTTP PUT.
    impl_mode: hand-written
  - path: "projects/jet/tests/publish/library_publish_e2e.rs"
    action: modify
    section: unit-test
    description: |
      Add mock-registry dry-run coverage: preview fields are populated and the
      mock registry store remains empty, proving no upload occurred.
    impl_mode: hand-written
  - path: "projects/jet/src/cli.rs"
    action: modify
    section: unit-test
    description: |
      Add parser-level coverage for `jet publish --dry-run --tag beta --access restricted`.
    impl_mode: hand-written
```
