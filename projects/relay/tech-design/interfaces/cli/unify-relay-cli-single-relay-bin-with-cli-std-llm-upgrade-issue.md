---
id: relay-cli-std-surface
summary: >
  Replace relay's bare relay-server binary with a single `relay` CLI bin
  (clap): bare `relay` serves the h2c transport (env-fallback flags), and the
  standard agent-facing ops ship alongside it — `relay llm` (cli_std::llm over
  relay-supplied topics + build-stamp ToolInfo), `relay upgrade`, and
  `relay issue <search|view|create>` (cli_std::issue, auto-tagged
  project:relay). build.rs delegates to libs/build-stamp for
  RELAY_GIT_SHA/RELAY_BUILT_AT/RELAY_TARGET. relay-raft stays untouched (its
  collapse is the raft-host adoption WI).
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-cli-std-surface-contract
entry: parse
nodes:
  parse: { kind: start, label: "relay CLI parses argv with clap (single relay bin)" }
  branch: { kind: decision, label: "which subcommand" }
  serve: { kind: process, label: "bare relay runs the h2c server with ServeArgs flags falling back to RELAY_BIND and RELAY_DATA_DIR env" }
  llm: { kind: process, label: "llm renders relay topics via cli_std llm render with the build-stamp ToolInfo" }
  upgrade: { kind: process, label: "upgrade checks or installs the latest relay release asset via cli_std (network path behind self-update feature)" }
  issue: { kind: process, label: "issue search view create dispatch cli_std issue filtered and auto-tagged project:relay (network path behind issue feature)" }
  out: { kind: terminal, label: "serve runs until shutdown; ops verbs print to stdout and exit" }
edges:
  - { from: parse, to: branch }
  - { from: branch, to: serve, label: "none (default)" }
  - { from: branch, to: llm, label: "llm" }
  - { from: branch, to: upgrade, label: "upgrade" }
  - { from: branch, to: issue, label: "issue" }
  - { from: serve, to: out }
  - { from: llm, to: out }
  - { from: upgrade, to: out }
  - { from: issue, to: out }
---
flowchart TD
    parse([relay CLI parses argv with clap single relay bin]) --> branch{which subcommand}
    branch -->|none default| serve[bare relay runs the h2c server with ServeArgs flags falling back to RELAY_BIND and RELAY_DATA_DIR env]
    branch -->|llm| llm[llm renders relay topics via cli_std llm render with the build-stamp ToolInfo]
    branch -->|upgrade| upgrade[upgrade checks or installs the latest relay release asset via cli_std network path behind self-update feature]
    branch -->|issue| issue[issue search view create dispatch cli_std issue filtered and auto-tagged project relay network path behind issue feature]
    serve --> out([serve runs until shutdown and ops verbs print to stdout and exit])
    llm --> out
    upgrade --> out
    issue --> out
```
