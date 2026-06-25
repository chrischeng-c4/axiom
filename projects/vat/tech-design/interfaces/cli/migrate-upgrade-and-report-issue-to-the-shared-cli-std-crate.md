---
id: migrate-upgrade-and-report-issue-to-the-shared-cli-std-crate
summary: Replace vat's hand-ported upgrade/report-issue (#491) with the shared libs/cli-std crate — vat depends on cli-std, defines a ToolInfo, and dispatches the two verbs to cli_std::upgrade::run / report_issue::run, matching how loom and lumen consume it. Pure de-duplication, no behaviour change.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "vat is the lone CLI hand-rolling the mandatory upgrade/report-issue verbs; consuming the shared cli-std crate removes ~800 lines of duplication and keeps vat's CLI-convention behaviour in lockstep with the ecosystem."
---

# Migrate upgrade + report-issue to the shared cli-std crate

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-adopt-cli-std-logic
entry: start
nodes:
  start: { kind: start, label: "vat upgrade or vat report-issue invoked" }
  parse: { kind: process, label: "cli.rs parses the same flags as before" }
  tool: { kind: process, label: "build cli_std ToolInfo for vat name vat tag vat@ repo axiom stamps" }
  which: { kind: decision, label: "which verb" }
  up: { kind: process, label: "cli_std upgrade run Options from flags" }
  rep: { kind: process, label: "cli_std report_issue run Options from flags" }
  done: { kind: terminal, label: "identical behaviour backed by shared crate" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: tool }
  - { from: tool, to: which }
  - { from: which, to: up, label: "upgrade" }
  - { from: which, to: rep, label: "report-issue" }
  - { from: up, to: done }
  - { from: rep, to: done }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cli-std-toolinfo.schema.json"
title: "vat ToolInfo for cli-std"
type: object
properties:
  tool_name: { type: string, const: "vat" }
  tag_prefix: { type: string, const: "vat@" }
  repo: { type: string, const: "chrischeng-c4/axiom" }
  version: { type: string, description: "env CARGO_PKG_VERSION" }
  target: { type: string, description: "env VAT_TARGET (build stamp)" }
  git_sha: { type: string, description: "env VAT_GIT_SHA" }
  built_at: { type: string, description: "env VAT_BUILT_AT" }
description: "Fields cli-std needs to drive upgrade (release discovery/asset) + report-issue (diagnostics). Exact field names follow cli_std::ToolInfo."
additionalProperties: true
```
## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-cli-std-cargo.schema.json"
title: "Cargo wiring for cli-std"
type: object
properties:
  dependency:
    type: string
    const: "cli-std = { path = \"../../libs/cli-std\", default-features = false }"
  features:
    type: object
    properties:
      self-update: { type: array, items: { type: string }, description: "[cli-std/online]" }
      report-issue: { type: array, items: { type: string }, description: "[cli-std/online]" }
    description: "Mirror loom/lumen: the online HTTP path comes from cli-std/online, not vat's own reqwest gate. Drop vat-direct deps (sha2/tar/flate2/semver and the upgrade-only reqwest gating) if cli-std now provides them."
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat upgrade
    behavior:
      - "Same flags (--check/--version/--force/--yes); dispatches to cli_std::upgrade::run with a vat ToolInfo and Options mapped from the flags. Self-updates from the latest vat@* GitHub release exactly as before."
  - name: vat report-issue
    behavior:
      - "Same flags (--title/--message/--repo/--label/--dry-run/--yes/[message...]); dispatches to cli_std::report_issue::run. Same diagnostics (version + target + OS/arch) and gh-API/prefilled-URL fallback."
  - name: vat --help
    behavior:
      - "Still lists llm, upgrade, report-issue (all three mandatory verbs)."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-adopt-cli-std-unit-tests
---
requirementDiagram
    requirement toolinfo_correct {
      id: UT1
      text: "vat's ToolInfo carries name=vat, tag prefix vat@, repo chrischeng-c4/axiom, and the build-stamp provenance."
      risk: low
      verifymethod: test
    }
    test vat_toolinfo_tests {
      type: functional
      verifies: toolinfo_correct
    }
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-cli-std-parity
    name: "upgrade/report-issue behave identically via cli-std"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat cli_convention -- --nocapture"
    assertions:
      - "vat --help lists llm, upgrade, report-issue."
      - "vat report-issue --title X --dry-run prints the diagnostics body incl. the vat version and OS."
      - "vat upgrade --check exits cleanly."
  - id: vat-cli-std-build
    name: "default + lean build compile"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo build -p vat --no-default-features"
    assertions:
      - "vat compiles with and without default features; the hand-rolled modules are gone."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/Cargo.toml
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add cli-std path dep; route self-update/report-issue features through cli-std/online; drop the now-unused upgrade-only direct deps if cli-std covers them."
  - path: projects/vat/src/cli.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Define vat's cli_std::ToolInfo; dispatch Upgrade/ReportIssue to cli_std::{upgrade,report_issue}::run mapping the existing flags onto cli-std Options."
  - path: projects/vat/src/commands/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Remove the upgrade / report_issue module declarations."
  - path: projects/vat/src/commands/upgrade.rs
    action: delete
    section: source
    impl_mode: hand-written
    reason: "Replaced by cli_std::upgrade."
  - path: projects/vat/src/commands/report_issue.rs
    action: delete
    section: source
    impl_mode: hand-written
    reason: "Replaced by cli_std::report_issue."
  - path: projects/vat/tests/vat_cli_convention.rs
    action: validate
    section: e2e-test
    impl_mode: hand-written
    reason: "The existing CLI-convention smoke test proves behaviour parity through cli-std."
```
