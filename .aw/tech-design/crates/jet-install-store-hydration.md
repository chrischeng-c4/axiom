---
id: jet-install-store-hydration
summary: Ensure Jet install cannot hide missing store packages or broken root node_modules links behind the lockfile marker.
fill_sections: [scenarios, logic, test-plan, changes]
---

# Jet Install Store Hydration

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: S1
    given: jet-lock yaml contains @types/prop-types and the node_modules marker matches package json deps
    and: the global store path for @types/prop-types is missing
    when: jet install runs
    then: Jet does not print Already up to date and proceeds through lockfile installation
  - id: S2
    given: node_modules contains a package with the expected package json version
    and: the global store no longer has the package or matching integrity marker
    when: install_resolved processes the lockfile entry
    then: Jet hydrates the store before deciding whether node_modules can be skipped
  - id: S3
    given: node_modules contains a dangling scoped-package symlink
    when: the marker fast path checks root links
    then: the link is treated as invalid and lockfile install repairs it
  - id: S4
    given: a lockfile key is scoped like /@types/prop-types@15.7.15
    when: lockfile validation checks the store
    then: Jet checks @types/prop-types rather than prop-types or @types/prop-types@15.7.15
```

## Install Pipeline
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-install-store-hydration-logic
entry: A
nodes:
  A: {kind: start, label: install_with_options}
  B: {kind: decision, label: lockfile valid in store}
  C: {kind: decision, label: node_modules marker hash matches}
  D: {kind: decision, label: root node_modules links match lockfile}
  E: {kind: terminal, label: report Already up to date}
  F: {kind: process, label: install from lockfile}
  G: {kind: process, label: ensure store package exists}
  H: {kind: decision, label: package installed at link target}
  I: {kind: terminal, label: skip relink}
  J: {kind: process, label: link package from store}
  K: {kind: terminal, label: write marker}
edges:
  - {from: A, to: B}
  - {from: B, to: C, label: yes}
  - {from: C, to: D, label: yes}
  - {from: D, to: E, label: yes}
  - {from: B, to: F, label: no or invalid}
  - {from: C, to: F, label: no}
  - {from: D, to: F, label: no}
  - {from: F, to: G}
  - {from: G, to: H}
  - {from: H, to: I, label: yes}
  - {from: H, to: J, label: no}
  - {from: I, to: K}
  - {from: J, to: K}
---
flowchart TD
    A[install_with_options] --> B{lockfile valid in store}
    B -->|yes| C{node_modules marker hash matches}
    C -->|yes| D{root node_modules links match lockfile}
    D -->|yes| E[report Already up to date]
    B -->|no or invalid| F[install from lockfile]
    C -->|no| F
    D -->|no| F
    F --> G[ensure store package exists]
    G --> H{package installed at link target}
    H -->|yes| I[skip relink]
    H -->|no| J[link package from store]
    I --> K[write marker]
    J --> K
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: jet-install-store-hydration-test-plan
entry: T1
---
requirementDiagram
    requirement R1 {
        id: R1
        text: marker fast path validates root links
        risk: high
        verifymethod: unit-test
    }
    requirement R2 {
        id: R2
        text: store is hydrated before installed skip
        risk: high
        verifymethod: unit-test
    }
    requirement R3 {
        id: R3
        text: scoped lockfile names validate store paths
        risk: medium
        verifymethod: unit-test
    }
    element T1 {
        type: test
        docref: cargo test -p jet pkg_manager::tests::test_lockfile_root_links_valid_detects_missing_scoped_link
    }
    element T2 {
        type: test
        docref: cargo test -p jet pkg_manager::tests::test_lockfile_root_links_valid_accepts_scoped_package_link
    }
    element T3 {
        type: test
        docref: cargo test -p jet pkg_manager::lockfile::tests::test_lockfile_scoped_non_workspace_entry_not_skipped
    }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/crates/jet-install-store-hydration.md
    action: CREATE
    impl_mode: hand-written
    desc: Focused TD for lockfile marker and store hydration repair behavior.
  - path: projects/jet/src/pkg_manager/mod.rs
    action: MODIFY
    impl_mode: hand-written
    desc: Validate root node_modules links before marker fast-path success and ensure store hydration precedes installed-package skip.
  - path: projects/jet/src/pkg_manager/lockfile.rs
    action: MODIFY
    impl_mode: hand-written
    desc: Add scoped-package lockfile validation coverage for missing store entries.
```

# Reviews

### Review 1
**Verdict:** approved

- [scenarios] Scenario set covers the missing scoped store package, stale marker, dangling symlink, and scoped lockfile key parsing paths from the issue.
- [logic] Pipeline explicitly gates `Already up to date` behind both store validation and root link validation, and moves store hydration before installed-package skip.
- [test-plan] Unit tests target the high-risk marker fast path and scoped package validation without requiring registry network access.
- [changes] File list is scoped to the package manager install path, lockfile coverage, and this TD.
