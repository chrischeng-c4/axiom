---
id: '1887'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-workspace-contract-convergence
entry: load_workspace
nodes:
  load_workspace:
    kind: start
    label: "Cargo loads projects/sift/Cargo.toml as a workspace member"
  aliases_current:
    kind: decision
    label: "all dependency package names and paths resolve to current shared libraries?"
  fail_manifest:
    kind: terminal
    label: "workspace load fails before any package can build"
  preserve_aliases:
    kind: process
    label: "map existing Rust aliases to service-k8s, storage-durable, metrics-prometheus, and raft-runtime"
  requests_current:
    kind: decision
    label: "Sift SearchRequest constructors include the current required offset?"
  preserve_first_page:
    kind: process
    label: "set offset to zero in logging and embedded-Lumen projections"
  metadata:
    kind: process
    label: "run root cargo metadata, the alias drift test, and Sift library check"
  buildable:
    kind: terminal
    label: "workspace manifest is valid and focused package checks can run"
edges:
  - { from: load_workspace, to: aliases_current }
  - { from: aliases_current, to: fail_manifest, label: "no" }
  - { from: aliases_current, to: preserve_aliases, label: "rename-only fix" }
  - { from: preserve_aliases, to: requests_current }
  - { from: requests_current, to: preserve_first_page, label: "no" }
  - { from: requests_current, to: metadata, label: "yes" }
  - { from: preserve_first_page, to: metadata }
  - { from: metadata, to: buildable }
---
flowchart TD
    load[load Sift workspace manifest] --> current{shared package paths current?}
    current -->|no| aliases[preserve crate aliases; update package/path]
    current -->|yes| request{SearchRequest offset explicit?}
    aliases --> request
    request -->|no| offset[set offset to zero; preserve first-page behavior]
    request -->|yes| metadata[cargo metadata plus drift test]
    offset --> metadata
    metadata --> ready([workspace can build])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/tests/shared_library_manifest_aliases.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Lock the four current package names and paths while proving the legacy Rust dependency aliases remain stable.
  - path: projects/sift/src/projection/logging.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Supply the current Lumen SearchRequest offset with zero to preserve first-page logging queries.
  - path: projects/sift/src/projection/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Supply the current Lumen SearchRequest offset with zero to preserve first-page embedded searches.
```

The bounded implementation also corrects the four corresponding dependency entries in `projects/sift/Cargo.toml`. Cargo manifests are declarative build inputs rather than Rust codegen targets; the generated regression test is the executable drift gate for that hand-authored manifest change. The two projection changes are already owned by their established HANDWRITE regions and only make the current Lumen first-page default explicit.
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-workspace-contract-convergence-verification
requirements:
  current_packages_preserve_aliases:
    id: R1
    text: "Sift maps its four established Rust dependency aliases to the current service-k8s, storage-durable, metrics-prometheus, and raft-runtime package paths."
    kind: regression
    risk: high
    verify: cargo test -p sift --test shared_library_manifest_aliases manifest_uses_current_shared_library_packages -- --exact
  sift_library_resolves:
    id: R4
    text: "Sift library source continues to resolve its existing crate aliases and current Lumen request contract without API changes."
    kind: regression
    risk: medium
    verify: cargo check -p sift --lib
  workspace_manifest_loads:
    id: R3
    text: "The repository workspace loads every manifest after the package/path convergence and no retired shared-library directory is required."
    kind: functional
    risk: high
    verify: cargo metadata --no-deps
  search_requests_preserve_first_page:
    id: R2
    text: "Both Sift projections set offset to zero so pagination behavior remains on the first result page."
    kind: regression
    risk: medium
    verify: cargo check -p sift --lib
---
flowchart TD
    r1[R1 current packages preserve aliases] --> cargo_test_p_sift_test_shared_library_manifest_aliases_manifest_uses_current_shared_library_packages_exact[cargo test -p sift --test shared_library_manifest_aliases manifest_uses_current_shared_library_packages -- --exact]
    r2[R2 search requests preserve first page] --> cargo_check_p_sift_lib[cargo check -p sift --lib]
    r3[R3 workspace manifest loads] --> cargo_metadata_no_deps[cargo metadata --no-deps]
    r4[R4 sift library resolves] --> cargo_check_p_sift_lib
```
