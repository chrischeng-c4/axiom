---
id: migrate-upgrade-and-report-issue-to-the-shared-cli-std-crate
summary: Replace vat's hand-ported CLI-convention verbs with the shared libs/cli-std crate — vat depends on cli-std, defines a ToolInfo, and dispatches `upgrade` plus `issue search/view/create` to cli_std::upgrade / cli_std::issue while `llm` uses cli_std::llm.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "vat must keep the mandatory llm/upgrade/issue verbs in lockstep with the ecosystem; consuming the shared cli-std crate prevents drift from the repo-wide CLI convention."
---

# Migrate standard CLI verbs to the shared cli-std crate

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-adopt-cli-std-logic
entry: start
nodes:
  start: { kind: start, label: "vat llm, vat upgrade, or vat issue invoked" }
  parse: { kind: process, label: "cli.rs parses the convention's flag shape" }
  tool: { kind: process, label: "build cli_std ToolInfo for vat name vat tag vat@ repo axiom stamps" }
  which: { kind: decision, label: "which verb" }
  llm: { kind: process, label: "cli_std llm render topic/format" }
  up: { kind: process, label: "cli_std upgrade run Options from flags" }
  iss: { kind: process, label: "cli_std issue search/view/create from subcommand flags" }
  done: { kind: terminal, label: "standard behaviour backed by shared crate" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: tool }
  - { from: tool, to: which }
  - { from: which, to: llm, label: "llm" }
  - { from: which, to: up, label: "upgrade" }
  - { from: which, to: iss, label: "issue" }
  - { from: llm, to: done }
  - { from: up, to: done }
  - { from: iss, to: done }
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
description: "Fields cli-std needs to drive upgrade release discovery/assets and issue diagnostics. Exact field names follow cli_std::ToolInfo."
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
      issue: { type: array, items: { type: string }, description: "[cli-std/online]" }
      report-issue: { type: array, items: { type: string }, description: "deprecated alias for issue" }
    description: "The online HTTP path comes from cli-std/online, not vat's own reqwest gate."
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat llm
    behavior:
      - "Supports the convention shape `vat llm [--topic <topic>] [--format md|json]`; default topic is outline and `--topic guide` prints vat's detailed agent guide."
  - name: vat upgrade
    behavior:
      - "Same flags (--check/--version/--force/--yes); dispatches to cli_std::upgrade::run with a vat ToolInfo and Options mapped from the flags. Self-updates from the latest vat@* GitHub release exactly as before."
  - name: vat issue
    behavior:
      - "`search [query]`, `view <n>`, and `create [--title <t>] [message...]` dispatch to cli_std::issue and are scoped to the project:vat label."
  - name: vat --help
    behavior:
      - "Lists llm, upgrade, and issue (all three mandatory verbs)."
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
    name: "llm/upgrade/issue behave via cli-std"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_cli_convention -- --nocapture"
    assertions:
      - "vat --help lists llm, upgrade, issue."
      - "vat llm --topic outline --format json prints the cli-std JSON shape."
      - "vat issue create --title X --dry-run prints the diagnostics body incl. the vat version and OS."
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
    reason: "Add cli-std path dep; route self-update/issue features through cli-std/online."
  - path: projects/vat/src/cli.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Define vat's cli_std::ToolInfo; dispatch Upgrade/Issue to cli_std::{upgrade,issue} and route llm topic rendering through cli_std::llm."
  - path: projects/vat/src/commands/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Remove the old hand-rolled upgrade/report_issue module declarations."
  - path: projects/vat/src/commands/upgrade.rs
    action: delete
    section: source
    impl_mode: hand-written
    reason: "Replaced by cli_std::upgrade."
  - path: projects/vat/src/commands/report_issue.rs
    action: delete
    section: source
    impl_mode: hand-written
    reason: "Replaced by cli_std::issue."
  - path: projects/vat/tests/vat_cli_convention.rs
    action: validate
    section: e2e-test
    impl_mode: hand-written
    reason: "The existing CLI-convention smoke test proves behaviour parity through cli-std."
```
