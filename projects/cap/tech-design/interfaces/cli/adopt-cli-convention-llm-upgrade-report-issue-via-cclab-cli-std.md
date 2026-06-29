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
  legacy_alias: { kind: process, label: "optional report-issue alias forwards to issue create behavior" }
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

The current repository contract names the mandatory issue surface `issue`, not
the older `report-issue` spelling still present in this WI body. Cap should
therefore make `llm`, `upgrade`, and `issue` visible in `cap --help` and route
their behavior through `cli-std`. A deprecated `report-issue` compatibility
entrypoint may remain for the issue's old acceptance text, but it must not
replace the current `issue search/view/create` surface.

`llm` is offline and owns only agent-facing cap topics. `upgrade` and `issue`
use `cli_std::ToolInfo` for release asset identity, build provenance, repo
routing, diagnostics, and the `project:cap` issue label. Existing cap domain
commands (`run`, passthrough wrapping, daemon, status, init, hook, config,
ping, wait) keep their current behavior and parse precedence.
