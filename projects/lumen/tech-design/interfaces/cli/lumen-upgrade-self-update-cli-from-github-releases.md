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
id: lumen-upgrade-contract
entry: start
nodes:
  start:    { kind: start,    label: "lumen upgrade [--check] [--tag T] [--force] [-y]" }
  selfexe:  { kind: process,  label: "current_exe() -> install_path; env!(CARGO_PKG_VERSION) -> cur" }
  triple:   { kind: process,  label: "target = compile-time target triple (e.g. aarch64-apple-darwin)" }
  list:     { kind: process,  label: "GET api.github.com/repos/{repo}/releases (UA + optional GITHUB_TOKEN)" }
  pick:     { kind: process,  label: "tags lumen@X.Y.Z -> semver; pick = --tag T else max stable" }
  ckflag:   { kind: decision, label: "--check?" }
  report:   { kind: terminal, label: "print cur vs pick; exit 0" }
  cmp:      { kind: decision, label: "pick == cur and not --force?" }
  noop:     { kind: terminal, label: "'already up to date (X.Y.Z)'; exit 0" }
  findasset:{ kind: decision, label: "asset 'lumen-{target}.tar.gz' in release?" }
  noasset:  { kind: terminal, label: "err 'no asset for {target}'; exit 1" }
  confirm:  { kind: decision, label: "tty and not -y -> confirm cur->pick?" }
  abort:    { kind: terminal, label: "'aborted'; exit 0" }
  dl:       { kind: process,  label: "download tarball + '.sha256' into temp dir beside install_path" }
  sha:      { kind: decision, label: "sha256(tarball) == published sha?" }
  shabad:   { kind: terminal, label: "err 'checksum mismatch'; drop temp; exit 1" }
  untar:    { kind: process,  label: "gz-decode + untar; read inner 'lumen-{target}/lumen' -> temp bin; chmod 0755" }
  persist:  { kind: process,  label: "write temp bin next to install_path (same dir = same fs)" }
  rename:   { kind: decision, label: "rename(temp_bin, install_path) ok?" }
  permbad:  { kind: terminal, label: "err 'cannot replace {path}: permission denied; re-run with sudo'; binary intact; exit 1" }
  done:     { kind: terminal, label: "'upgraded cur -> pick'; exit 0" }
edges:
  - { from: start,    to: selfexe }
  - { from: selfexe,  to: triple }
  - { from: triple,   to: list }
  - { from: list,     to: pick }
  - { from: pick,     to: ckflag }
  - { from: ckflag,   to: report,    label: "yes" }
  - { from: ckflag,   to: cmp,       label: "no" }
  - { from: cmp,      to: noop,      label: "yes" }
  - { from: cmp,      to: findasset, label: "no" }
  - { from: findasset,to: noasset,   label: "no" }
  - { from: findasset,to: confirm,   label: "yes" }
  - { from: confirm,  to: abort,     label: "declined" }
  - { from: confirm,  to: dl,        label: "yes/-y" }
  - { from: dl,       to: sha }
  - { from: sha,      to: shabad,    label: "no" }
  - { from: sha,      to: untar,     label: "yes" }
  - { from: untar,    to: persist }
  - { from: persist,  to: rename }
  - { from: rename,   to: permbad,   label: "no" }
  - { from: rename,   to: done,      label: "yes" }
---
flowchart TD
    start([lumen upgrade]) --> selfexe[current_exe + cur version]
    selfexe --> triple[compile-time target triple]
    triple --> list[GET releases]
    list --> pick[pick --tag or max stable semver]
    pick --> ckflag{--check?}
    ckflag -->|yes| report([print cur vs pick])
    ckflag -->|no| cmp{pick == cur and not --force?}
    cmp -->|yes| noop([already up to date])
    cmp -->|no| findasset{asset for target?}
    findasset -->|no| noasset([no asset; exit 1])
    findasset -->|yes| confirm{confirm unless -y?}
    confirm -->|declined| abort([aborted])
    confirm -->|yes| dl[download tarball + .sha256 to temp]
    dl --> sha{sha256 matches?}
    sha -->|no| shabad([checksum mismatch; exit 1])
    sha -->|yes| untar[untar inner lumen to temp bin]
    untar --> persist[place temp bin in install dir]
    persist --> rename{rename over install_path?}
    rename -->|no| permbad([permission denied; intact; exit 1])
    rename -->|yes| done([upgraded])
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

# Reviews

### Review 1
**Verdict:** approved

- [logic] Dispatch flow covers the full upgrade path and every documented branch: `--check` short-circuit, already-current/`--force` no-op, missing per-target asset, sha256 verify gate, and the atomic-replace permission gate — each terminating safely with the binary intact on failure.
- [unit-test] Requirements R1–R5 map onto the acceptance criteria (check no-op, target→asset resolution, sha-mismatch abort, latest-stable/`--tag` selection, already-current no-op) and are all `verify: test`, matching the code-artifact testability gate.
