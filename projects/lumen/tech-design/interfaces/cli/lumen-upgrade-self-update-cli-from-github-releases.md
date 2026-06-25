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
