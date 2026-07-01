---
id: keep-cli-issue-group-migration
summary: >
  Replace Keep's write-only `keep report-issue` clap command with the standard
  `keep issue <search|view|create>` group, dispatching to
  cli_std::issue::{search, view, create} while keeping the convention's
  project:keep auto-tag on create and project:keep filtering on search.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-cli-issue-group-migration-contract
entry: parse
nodes:
  parse: { kind: start, label: "keep CLI parses the issue subcommand group" }
  branch: { kind: decision, label: "which issue verb" }
  search: { kind: process, label: "issue search dispatches cli_std::issue::search filtered to project:keep" }
  view: { kind: process, label: "issue view dispatches cli_std::issue::view by number" }
  create: { kind: process, label: "issue create dispatches cli_std::issue::create auto-tagged project:keep" }
  out: { kind: terminal, label: "print results or filed/preview issue" }
edges:
  - { from: parse, to: branch }
  - { from: branch, to: search, label: "search" }
  - { from: branch, to: view, label: "view" }
  - { from: branch, to: create, label: "create" }
  - { from: search, to: out }
  - { from: view, to: out }
  - { from: create, to: out }
---
flowchart TD
    parse([keep CLI parses the issue subcommand group]) --> branch{which issue verb}
    branch -->|search| search[issue search dispatches cli_std issue search filtered to project keep]
    branch -->|view| view[issue view dispatches cli_std issue view by number]
    branch -->|create| create[issue create dispatches cli_std issue create auto-tagged project keep]
    search --> out([print results or filed or preview issue])
    view --> out
    create --> out
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-cli-issue-group-migration-tests
requirements:
  issue_group_parses:
    id: R1
    text: "The keep CLI parses issue search, issue view, and issue create with their convention flags."
    kind: behavior
    risk: medium
    verify: test
  report_issue_gone:
    id: R2
    text: "The removed keep report-issue command no longer parses."
    kind: behavior
    risk: medium
    verify: test
elements:
  keep_cli_tests:
    kind: test
    path: projects/keep/src/bin/keep.rs
relations:
  - { from: keep_cli_tests, verifies: issue_group_parses }
  - { from: keep_cli_tests, verifies: report_issue_gone }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "issue group parses"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "report-issue gone"
      risk: medium
      verifymethod: test
    }
    element keep_cli_tests {
      type: "rs/#[test]"
    }
    keep_cli_tests - verifies -> R1
    keep_cli_tests - verifies -> R2
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Replace the report-issue feature with an issue feature gating cli-std/online so the issue group's network paths build, mirroring jet."
  - path: projects/keep/src/bin/keep.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Replace the ReportIssue clap command + ReportIssueArgs with an Issue subcommand group (search/view/create) dispatching to cli_std::issue::{search,view,create}; auto-tag project:keep on create and filter search to it; update the module doc comment."
  - path: projects/keep/src/bin/keep.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add a #[cfg(test)] mod asserting keep issue search/view/create parse with their flags and that keep report-issue no longer parses."
```
