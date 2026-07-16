---
id: '1817'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-dx-readme-surface-contract
entry: cli
nodes:
  cli: { kind: start, label: "built vat --help is the command inventory authority" }
  docs: { kind: process, label: "README and llm describe the same agent-facing command surface" }
  boundaries: { kind: process, label: "state Apple Container support and Docker Engine, generic Compose, persistent K8s exclusions" }
  test: { kind: process, label: "built-binary test asserts command inventory and boundary phrases" }
  done: { kind: terminal, label: "documentation drift is a failing test" }
edges:
  - { from: cli, to: docs }
  - { from: docs, to: boundaries }
  - { from: boundaries, to: test }
  - { from: test, to: done }
---
```

The public contract is documentation-only: no command behavior changes. The README and offline `vat llm` guide enumerate `build`, `compose`, `docker`, and `k8s` as shipped agent-facing commands, state their Apple Container limits, and avoid promising Docker Engine/API emulation, general Compose compatibility, or persistent Kubernetes. The test obtains the actual binary help and fails if the documented inventory gets ahead of the executable surface.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/src/commands/llm.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: exec
  - path: apps/vat/tests/vat_cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: cli_convention_help_lists_all_three
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-dx-readme-surface-contract-verification
requirements:
  boundary_text:
    id: AC2
    text: "The agent docs distinguish Apple Container support from unsupported Docker Engine/API, generic Compose, and persistent Kubernetes claims."
    kind: regression
    risk: medium
    verify: vat_cli_convention::dx_docs_state_supported_boundaries
  command_inventory:
    id: AC1
    text: "The README's agent command inventory is a subset of the top-level commands exposed by the built vat binary and includes build, compose, docker, and k8s."
    kind: regression
    risk: high
    verify: vat_cli_convention::documented_agent_commands_match_help
---
flowchart TD
    ac1[AC1 command inventory] --> vat_cli_convention_documented_agent_commands_match_help[vat_cli_convention::documented_agent_commands_match_help]
    ac2[AC2 boundary text] --> vat_cli_convention_dx_docs_state_supported_boundaries[vat_cli_convention::dx_docs_state_supported_boundaries]
```
