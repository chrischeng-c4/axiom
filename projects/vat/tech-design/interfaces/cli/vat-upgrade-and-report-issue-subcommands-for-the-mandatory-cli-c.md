---
id: vat-upgrade-and-report-issue-subcommands-for-the-mandatory-cli-c
summary: Keep vat aligned with the mandatory CLI convention — `vat llm`, `vat upgrade`, and `vat issue search/view/create` — so vat satisfies the ecosystem contract that every CLI ships llm/upgrade/issue.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: vat-llm-vat-upgrade-vat-issue-mandatory-cli-convention
    claim: vat-llm-vat-upgrade-vat-issue-mandatory-cli-convention
    coverage: partial
    rationale: "vat is the agent's dev-container CLI; the mandatory agent-facing contract (self-document, self-update, and tracker issue search/read/create) is incomplete unless the binary exposes llm, upgrade, and issue."
---

# Vat standard CLI convention subcommands

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-upgrade-and-report-issue-subcommands-logic
entry: start
nodes:
  start: { kind: start, label: "vat llm, vat upgrade, or vat issue invoked" }
  which: { kind: decision, label: "which verb" }
  detect: { kind: process, label: "upgrade detect target from VAT_TARGET stamped at build" }
  releases: { kind: process, label: "query github releases filter vat@ tags" }
  select: { kind: process, label: "select latest stable or pinned --version tag" }
  checkonly: { kind: decision, label: "--check" }
  reportver: { kind: terminal, label: "print current vs latest and exit 0" }
  download: { kind: process, label: "download vat-target tar.gz and .sha256 sidecar" }
  verify: { kind: decision, label: "sha256 matches" }
  abort: { kind: terminal, label: "fail loudly do not replace binary" }
  extract: { kind: process, label: "gunzip untar find vat-target/vat inner binary" }
  install: { kind: process, label: "write temp sibling chmod 0755 atomic rename over self" }
  upok: { kind: terminal, label: "upgraded" }
  diag: { kind: process, label: "issue create collect version os arch context diagnostics" }
  body: { kind: process, label: "assemble body user message plus diagnostics block" }
  dry: { kind: decision, label: "--dry-run" }
  preview: { kind: terminal, label: "print body and exit without submitting" }
  submit: { kind: process, label: "gh issue create else print prefilled issues/new url" }
  filed: { kind: terminal, label: "issue filed or url printed" }
edges:
  - { from: start, to: which }
  - { from: which, to: detect, label: "upgrade" }
  - { from: detect, to: releases }
  - { from: releases, to: select }
  - { from: select, to: checkonly }
  - { from: checkonly, to: reportver, label: "yes" }
  - { from: checkonly, to: download, label: "no" }
  - { from: download, to: verify }
  - { from: verify, to: abort, label: "no" }
  - { from: verify, to: extract, label: "yes" }
  - { from: extract, to: install }
  - { from: install, to: upok }
  - { from: which, to: diag, label: "issue create" }
  - { from: diag, to: body }
  - { from: body, to: dry }
  - { from: dry, to: preview, label: "yes" }
  - { from: dry, to: submit, label: "no" }
  - { from: submit, to: filed }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cli-convention-upgrade-report-issue.schema.json"
title: "vat upgrade / issue evidence shapes"
type: object
properties:
  release_asset:
    type: object
    description: "GitHub release asset naming the upgrade path consumes."
    properties:
      tag: { type: string, description: "vat@X.Y.Z" }
      tarball: { type: string, description: "vat-<arch>-<os>.tar.gz" }
      checksum: { type: string, description: "<tarball>.sha256 sidecar (one-line hex digest)" }
      inner_path: { type: string, description: "vat-<target>/vat inside the tarball" }
    required: [tag, tarball, checksum]
  upgrade_result:
    type: object
    properties:
      current: { type: string }
      latest: { type: string }
      action: { type: string, enum: [checked, upgraded, already-current, aborted-checksum] }
  report_diagnostics:
    type: object
    description: "Auto-attached to every issue create body."
    properties:
      version: { type: string }
      target: { type: string }
      os: { type: string }
      arch: { type: string }
    required: [version, os, arch]
additionalProperties: true
```
## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cli-convention-cargo-features.schema.json"
title: "vat Cargo features for the self-update / issue HTTP paths"
type: object
properties:
  features:
    type: object
    description: "Network paths are feature-gated so the lean build stays HTTP-client-free; pure decode/verify deps (flate2/tar/sha2/semver) stay non-optional."
    properties:
      self-update:
        type: array
        items: { type: string }
        description: "[dep:reqwest] — gates vat upgrade's HTTPS download path."
      issue:
        type: array
        items: { type: string }
        description: "[cli-std/online] — gates issue search/view/create online GitHub paths."
      report-issue:
        type: array
        items: { type: string }
        description: "Deprecated compatibility alias for the issue feature; not a CLI verb."
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat upgrade
    usage: "vat upgrade [--version <tag>] [--check] [--force] [--yes]"
    behavior:
      - "Self-update the running binary to the latest vat@* GitHub release."
      - "Detect target from VAT_TARGET; download vat-<target>.tar.gz + .sha256; verify checksum; extract the inner vat binary; atomically replace the running binary (temp sibling + rename)."
      - "--check reports current vs latest and exits 0 without modifying the binary."
      - "--version <tag> pins an exact release (bare or vat@ prefixed); --force reinstalls the selected version; --yes skips the confirmation prompt."
      - "Fail loudly on checksum mismatch; never leave a half-written binary. The HTTPS download path is behind the self-update Cargo feature."
  - name: vat issue
    usage: "vat issue search [query] | vat issue view <n> | vat issue create [--title <t>] [--dry-run] [message...]"
    behavior:
      - "Search and view project:vat issues through read-only GitHub API calls."
      - "Create a structured GitHub issue against the axiom repo; without a token, print a prefilled issues/new URL — never silent-fail."
      - "Auto-attach diagnostics: vat version, target, OS/arch, git sha, and build timestamp."
      - "`create --title` sets the title; trailing args add a description above the diagnostics block; `--dry-run` assembles and prints without submitting."
  - name: vat --help
    behavior:
      - "List llm, upgrade, and issue so all three mandatory CLI-convention verbs are discoverable."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-upgrade-and-report-issue-subcommands-unit-tests
---
requirementDiagram
    requirement select_version {
      id: UT1
      text: "Version selection picks the latest stable vat@ release, ignores prereleases, and honors a pinned --version tag."
      risk: medium
      verifymethod: test
    }
    requirement verify_sha256 {
      id: UT2
      text: "sha256 verify is a pure case-insensitive hex comparison; a mismatch returns an error (upgrade aborts, binary untouched)."
      risk: high
      verifymethod: test
    }
    requirement extract_binary {
      id: UT3
      text: "Tarball extraction finds the vat-<target>/vat inner binary entry."
      risk: medium
      verifymethod: test
    }
    requirement assemble_body {
      id: UT4
      text: "issue create assembles a body = optional message + separator + diagnostics block (version, target, os, arch)."
      risk: low
      verifymethod: test
    }
    requirement prefilled_url {
      id: UT5
      text: "The fallback prefilled issues/new URL percent-encodes title and body."
      risk: low
      verifymethod: test
    }
    test upgrade_select_version_tests {
      type: functional
      verifies: select_version
    }
    test upgrade_verify_sha256_tests {
      type: functional
      verifies: verify_sha256
    }
    test upgrade_extract_binary_tests {
      type: functional
      verifies: extract_binary
    }
    test issue_assemble_body_tests {
      type: functional
      verifies: assemble_body
    }
    test issue_prefilled_url_tests {
      type: functional
      verifies: prefilled_url
    }
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-cli-convention-help-lists-all-three
    name: "vat --help lists llm, upgrade, issue"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: vat-llm-vat-upgrade-vat-issue-mandatory-cli-convention
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_cli_convention -- --nocapture"
    assertions:
      - "`vat --help` output contains `llm`, `upgrade`, and `issue`."
      - "`vat upgrade --check` exits 0 and reports current vs latest without writing the binary (network-permitting; offline it errors cleanly, never panics)."
      - "`vat issue create --title X --dry-run` prints a body containing the vat version and OS/arch and submits nothing."
  - id: vat-cli-convention-lean-build
    name: "lean build still compiles"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: vat-llm-vat-upgrade-vat-issue-mandatory-cli-convention
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo build -p vat --no-default-features"
    assertions:
      - "vat compiles without default features; the self-update HTTPS path is absent but the verbs still parse and the pure helpers remain testable."
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/cli.rs
    action: modify
    section: cli
    impl_mode: hand-written
    reason: "CLI section edge: expose the mandatory llm, upgrade, and issue subcommands in vat's top-level parser."
  - path: projects/vat/Cargo.toml
    action: modify
    section: config
    impl_mode: hand-written
    reason: "Config section edge: feature-gate online upgrade/issue behavior while keeping lean builds parseable."
  - path: projects/vat/src/cli.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Logic section edge: select upgrade versions, assemble issue diagnostics, and dispatch the standard verbs."
  - path: projects/vat/build.rs
    action: modify
    section: schema
    impl_mode: hand-written
    reason: "Schema section edge: stamp VAT_TARGET, VAT_GIT_SHA, and VAT_BUILT_AT used by upgrade and issue diagnostics."
  - path: projects/vat/tests/vat_cli_convention.rs
    action: validate
    section: unit-test
    impl_mode: hand-written
    reason: "Unit-test section edge: standard CLI smoke tests cover help, upgrade check, and issue dry-run contracts."
  - path: projects/vat/src/commands/upgrade.rs
    action: create
    section: source
    impl_mode: hand-written
    reason: "Port lumen's upgrade self-update implementation (target detect, release query, version select, download, sha256 verify, extract, atomic replace) to vat."
  - path: libs/cli-std/src/issue.rs
    action: reuse
    section: source
    impl_mode: hand-written
    reason: "Use the shared issue implementation for search/view/create, diagnostics, body assembly, GitHub API submit, and prefilled URL fallback."
  - path: projects/vat/src/commands/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "No vat-local report_issue module is exported; issue routes through cli-std from cli.rs."
  - path: projects/vat/src/cli.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add Upgrade and Issue subcommand variants + flags and dispatch them to cli-std-backed handlers."
  - path: projects/vat/build.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Stamp VAT_TARGET (and VAT_GIT_SHA / VAT_BUILT_AT) for upgrade target detection and issue diagnostics."
  - path: projects/vat/Cargo.toml
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Route self-update / issue features through cli-std/online."
  - path: projects/vat/tests/vat_cli_convention.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "Smoke test: --help lists all three mandatory verbs; upgrade --check and issue create --dry-run behave per contract."
```
