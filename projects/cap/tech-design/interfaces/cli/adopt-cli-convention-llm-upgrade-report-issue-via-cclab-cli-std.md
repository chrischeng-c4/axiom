---
id: adopt-cli-convention-llm-upgrade-report-issue-via-cclab-cli-std
summary: Adopt the current shared CLI convention for cap through cli-std, exposing llm, upgrade, and issue commands.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: daemon-lifecycle-and-status
    role: primary
    gap: cli-status-and-wait-surfaces
    claim: cli-status-and-wait-surfaces
    coverage: partial
    rationale: "The standard agent-facing CLI verbs are part of cap's CLI surface and must coexist with command wrapping and status entrypoints."
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: partial
    rationale: "Agents use cap directly, so cap must expose the repo-wide llm/upgrade/issue control surface."
---

# Adopt CLI Convention Llm Upgrade Issue Via Cli Std

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-cli-std-convention-contract
entry: cap_cli
nodes:
  cap_cli: { kind: start, label: "cap CLI receives argv" }
  clap_dispatch: { kind: decision, label: "standard agent command?" }
  llm: { kind: process, label: "llm renders cap topics through cli_std::llm" }
  upgrade: { kind: process, label: "upgrade delegates to cli_std::upgrade::run" }
  issue: { kind: process, label: "issue search/view/create delegates to cli_std::issue" }
  legacy_alias: { kind: process, label: "report-issue compatibility forwards to issue create behavior" }
  domain: { kind: process, label: "existing cap run/status/init/hook behavior unchanged" }
  terminal: { kind: terminal, label: "standard help, docs, and diagnostics-rich issue surface" }
edges:
  - { from: cap_cli, to: clap_dispatch, label: "parse subcommand" }
  - { from: clap_dispatch, to: llm, label: "llm" }
  - { from: clap_dispatch, to: upgrade, label: "upgrade" }
  - { from: clap_dispatch, to: issue, label: "issue" }
  - { from: clap_dispatch, to: legacy_alias, label: "report-issue compatibility" }
  - { from: clap_dispatch, to: domain, label: "run/status/config/init/hook/etc." }
  - { from: llm, to: terminal, label: "offline topic output" }
  - { from: upgrade, to: terminal, label: "release check/update path" }
  - { from: issue, to: terminal, label: "project:cap issue operations" }
  - { from: legacy_alias, to: terminal, label: "deprecated dry-run/create compatibility" }
  - { from: domain, to: terminal, label: "existing cap semantics preserved" }
---
flowchart TB
  cap_cli["cap CLI receives argv"] --> clap_dispatch{"standard agent command?"}
  clap_dispatch -->|llm| llm["cli_std::llm renders cap topics"]
  clap_dispatch -->|upgrade| upgrade["cli_std::upgrade::run"]
  clap_dispatch -->|issue| issue["cli_std::issue search/view/create"]
  clap_dispatch -->|report-issue compatibility| legacy_alias["deprecated alias forwards to issue create"]
  clap_dispatch -->|domain commands| domain["existing cap run/status/init/hook behavior"]
  llm --> terminal["standard agent-facing CLI surface"]
  upgrade --> terminal
  issue --> terminal
  legacy_alias --> terminal
  domain --> terminal
```

Cap adopts the current repo-wide CLI convention through `cli-std`: `llm`,
`upgrade`, and `issue search/view/create` are the primary standard surface.
The older `report-issue` spelling in the WI body is implemented only as a
deprecated compatibility entrypoint that forwards to the same diagnostics-rich
create path. Existing cap domain commands and passthrough wrapping keep their
current parse behavior.
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cap-cli-std-convention-tests
requirements:
  help_surface:
    id: CLI-STD-UT-1
    text: "cap --help lists llm, upgrade, and issue as standard agent-facing commands."
    kind: functional
    risk: high
    verify: test
  llm_offline:
    id: CLI-STD-UT-2
    text: "cap llm renders cap-specific offline docs through cli_std::llm and includes the standard-command footer."
    kind: functional
    risk: medium
    verify: test
  issue_create:
    id: CLI-STD-UT-3
    text: "cap issue create --dry-run builds a diagnostics-rich issue body tagged project:cap without network submission."
    kind: functional
    risk: high
    verify: test
  legacy_report_issue:
    id: CLI-STD-UT-4
    text: "cap report-issue --dry-run remains a deprecated compatibility entrypoint for the stale WI acceptance text."
    kind: compatibility
    risk: medium
    verify: test
  build_features:
    id: CLI-STD-UT-5
    text: "cap builds in the default offline configuration and with the online release feature that enables cli-std network paths."
    kind: functional
    risk: medium
    verify: test
elements:
  cli_unit_tests:
    kind: test
    type: "cargo test -p cap cli_std_convention"
  cap_package_tests:
    kind: test
    type: "cargo test -p cap"
  cap_online_build:
    kind: test
    type: "cargo build -p cap --features release"
relations:
  - { from: cli_unit_tests, verifies: help_surface }
  - { from: cli_unit_tests, verifies: llm_offline }
  - { from: cli_unit_tests, verifies: issue_create }
  - { from: cli_unit_tests, verifies: legacy_report_issue }
  - { from: cap_package_tests, verifies: help_surface }
  - { from: cap_online_build, verifies: build_features }
---
requirementDiagram
  requirement help_surface {
    id: CLI-STD-UT-1
    text: "help lists llm upgrade issue"
    risk: high
    verifymethod: test
  }
  requirement llm_offline {
    id: CLI-STD-UT-2
    text: "llm uses cli_std offline renderer"
    risk: medium
    verifymethod: test
  }
  requirement issue_create {
    id: CLI-STD-UT-3
    text: "issue create dry-run emits diagnostics and project label"
    risk: high
    verifymethod: test
  }
  requirement legacy_report_issue {
    id: CLI-STD-UT-4
    text: "report-issue dry-run compatibility remains"
    risk: medium
    verifymethod: test
  }
  requirement build_features {
    id: CLI-STD-UT-5
    text: "default and release feature builds work"
    risk: medium
    verifymethod: test
  }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cap/Cargo.toml
    action: modify
    section: dependencies
    impl_mode: hand-written
    description: >
      Add cli-std with default features disabled and expose a release feature
      that enables cli-std/online for upgrade and issue network paths.

  - path: projects/cap/build.rs
    action: create
    section: build-provenance
    impl_mode: hand-written
    description: >
      Stamp CAP_TARGET, CAP_GIT_SHA, and CAP_BUILT_AT for cli_std::ToolInfo
      release asset and diagnostics metadata.

  - path: projects/cap/src/cli.rs
    action: modify
    section: cli-surface
    impl_mode: hand-written
    description: >
      Register and dispatch llm, upgrade, issue search/view/create, and a
      deprecated report-issue compatibility entrypoint through cli-std while
      preserving existing cap domain command behavior.

  - path: projects/cap/README.md
    action: modify
    section: cli-convention
    impl_mode: hand-written
    description: >
      Document the standard agent-facing commands and clarify that issue is the
      current surface while report-issue is legacy compatibility.

  - path: projects/cap/tech-design/semantic/cap-src.md
    action: modify
    section: source-metadata
    impl_mode: hand-written
    description: >
      Keep the semantic source manifest aligned with cap's CLI exports and any
      newly introduced build provenance script.
```
