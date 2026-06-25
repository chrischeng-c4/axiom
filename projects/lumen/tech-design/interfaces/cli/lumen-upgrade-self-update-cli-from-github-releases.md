---
id: lumen-cli-upgrade
summary: >
  Add a top-level `lumen upgrade` subcommand that moves the installed lumen
  binary to a published GitHub release. It resolves the current build target and
  version, queries the `lumen@*` releases, selects the latest stable (or
  `--tag`), downloads the matching `lumen-<target>.tar.gz`, verifies it against
  the published `.sha256`, extracts the inner `lumen`, and atomically replaces
  the running executable. `--check` reports without changing anything; failures
  (no asset, sha mismatch, permission denied) abort with the existing binary
  intact. Implemented with workspace-locked crates only (reqwest, flate2, tar,
  sha2, semver) — no new top-level dependency.
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "service-process-interface"
    coverage: partial
    rationale: >
      Extends lumen's command surface with an operator-facing self-update path
      that consumes lumen's own release artifacts, keeping a deployed binary
      current without an external package manager.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-upgrade-dispatch
entry: start
nodes:
  start:     { kind: start,    label: "lumen upgrade [--check|--tag|--force|-y]" }
  resolve:   { kind: process,  label: "resolve current target + current version" }
  query:     { kind: process,  label: "GitHub Releases API: list lumen@* releases" }
  select:    { kind: process,  label: "pick target version (latest stable semver | --tag)" }
  is_check:  { kind: decision, label: "--check?" }
  report:    { kind: terminal, label: "print current vs latest; exit 0 (no change)" }
  current:   { kind: decision, label: "already current and not --force?" }
  uptodate:  { kind: terminal, label: "already up to date; exit 0" }
  asset:     { kind: decision, label: "asset lumen-<target>.tar.gz exists?" }
  noasset:   { kind: terminal, label: "no asset for target; exit non-zero" }
  download:  { kind: process,  label: "download tarball + .sha256 to temp" }
  verify:    { kind: decision, label: "sha256 matches?" }
  shafail:   { kind: terminal, label: "sha mismatch; abort, binary intact; exit non-zero" }
  extract:   { kind: process,  label: "extract inner lumen from tarball" }
  replace:   { kind: decision, label: "atomic replace running exe (temp + rename)?" }
  permfail:  { kind: terminal, label: "permission denied; binary intact; remediation msg" }
  done:      { kind: terminal, label: "installed <new version>; exit 0" }
edges:
  - { from: start,    to: resolve }
  - { from: resolve,  to: query }
  - { from: query,    to: select }
  - { from: select,   to: is_check }
  - { from: is_check, to: report,   label: "yes" }
  - { from: is_check, to: current,  label: "no" }
  - { from: current,  to: uptodate, label: "yes" }
  - { from: current,  to: asset,    label: "no" }
  - { from: asset,    to: noasset,  label: "no" }
  - { from: asset,    to: download, label: "yes" }
  - { from: download, to: verify }
  - { from: verify,   to: shafail,  label: "no" }
  - { from: verify,   to: extract,  label: "yes" }
  - { from: extract,  to: replace }
  - { from: replace,  to: permfail, label: "no" }
  - { from: replace,  to: done,     label: "yes" }
---
flowchart TD
    start([lumen upgrade]) --> resolve[resolve target + version]
    resolve --> query[list lumen@* releases]
    query --> select[pick target version]
    select --> is_check{--check?}
    is_check -->|yes| report([print current vs latest])
    is_check -->|no| current{already current and not --force?}
    current -->|yes| uptodate([already up to date])
    current -->|no| asset{asset for target?}
    asset -->|no| noasset([no asset; error])
    asset -->|yes| download[download tarball + .sha256]
    download --> verify{sha256 matches?}
    verify -->|no| shafail([sha mismatch; abort])
    verify -->|yes| extract[extract inner lumen]
    extract --> replace{atomic replace exe?}
    replace -->|no| permfail([permission denied; intact])
    replace -->|yes| done([installed new version])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-upgrade-verification
requirements:
  check_reports_no_change:
    id: R1
    text: "upgrade --check prints current vs latest and exits 0 without replacing the binary"
    kind: functional
    risk: high
    verify: test
  target_asset_selected:
    id: R2
    text: "the release asset name for the running target resolves to lumen-<target>.tar.gz (+ .sha256)"
    kind: functional
    risk: high
    verify: test
  sha_mismatch_aborts:
    id: R3
    text: "a tarball whose sha256 differs from the published .sha256 aborts the install and leaves the binary intact"
    kind: functional
    risk: high
    verify: test
  version_select_latest_or_tag:
    id: R4
    text: "version selection picks the highest stable semver across lumen@* tags, or the exact --tag when given"
    kind: functional
    risk: medium
    verify: test
  already_current_noop:
    id: R5
    text: "when the installed version equals the selected version and --force is absent, no download or replace occurs"
    kind: functional
    risk: medium
    verify: test
---
flowchart TD
    r1[R1 --check reports, no change] --> v1{exit 0 and binary untouched?}
    r2[R2 target -> asset name] --> v2{lumen-target.tar.gz resolved?}
    r3[R3 sha mismatch] --> v3{abort and binary intact?}
    r4[R4 version select] --> v4{latest stable or --tag?}
    r5[R5 already current] --> v5{no-op without --force?}
```
