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
  - path: apps/vat/README.md
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/vat/src/commands/llm.rs
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/vat/tests/vat_cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-dx-readme-surface-verification
requirements:
  boundaries_are_explicit:
    id: AC2
    text: "README and offline llm guide state the bounded Apple Container support and reject Docker Engine/API, generic Compose, and persistent Kubernetes claims."
    kind: regression
    risk: medium
    verify: vat_cli_convention::dx_docs_state_supported_boundaries
  documented_commands_match_help:
    id: AC1
    text: "The README agent command inventory names build, compose, docker, and k8s only when each is exposed by the built vat binary's top-level help."
    kind: regression
    risk: high
    verify: vat_cli_convention::documented_agent_commands_match_help
---
flowchart TD
    ac1[AC1 documented commands match help] --> vat_cli_convention_documented_agent_commands_match_help[vat_cli_convention::documented_agent_commands_match_help]
    ac2[AC2 boundaries are explicit] --> vat_cli_convention_dx_docs_state_supported_boundaries[vat_cli_convention::dx_docs_state_supported_boundaries]
```
