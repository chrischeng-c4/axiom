---
id: '1224'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-source-root-convergence
entry: scan
nodes:
  scan: { kind: start, label: "Scan repository for literal projects/lumen" }
  classify: { kind: decision, label: "Live source-root fact or fixture?" }
  rewrite: { kind: process, label: "Rewrite to apps/lumen" }
  retain: { kind: process, label: "Retain intentional historical TD identity" }
  verify: { kind: process, label: "Verify workspace and AW discovery resolve apps/lumen" }
  done: { kind: terminal, label: "No stale live source-root reference remains" }
edges:
  - { from: scan, to: classify }
  - { from: classify, to: rewrite, label: "yes" }
  - { from: classify, to: retain, label: "historical only" }
  - { from: rewrite, to: verify }
  - { from: retain, to: verify }
  - { from: verify, to: done }
---
flowchart TD
    scan([Scan literal projects/lumen references]) --> classify{Live source-root fact or fixture?}
    classify -->|yes| rewrite[Rewrite to apps/lumen]
    classify -->|historical only| retain[Retain intentional TD identity]
    rewrite --> verify[Verify workspace and AW discovery]
    retain --> verify
    verify --> done([No stale live reference remains])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/mamba/src/runtime/stdlib/mmap_mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Correct the workspace dependency note to name the canonical apps/lumen source root."
  - path: apps/agentic-workflow/src/cli/capability.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Make the Lumen capability-profile discovery fixture exercise apps/lumen rather than the retired projects/lumen root."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-source-root-convergence-verification
requirements:
  canonical_source_root:
    id: R1
    text: "Every live Lumen source-root fact and discovery fixture uses apps/lumen while AW project identity remains lumen."
    kind: regression
    risk: medium
    verify: cargo test -p agentic-workflow lumen_agent_facing_trait_adds_dx_without_expanding_service_obligations && cargo build -p lumen
---
flowchart TD
    r1[R1 canonical source root] --> cargo_test_p_agentic_workflow_lumen_agent_facing_trait_adds_dx_without_expanding_service_obligations_cargo_build_p_lumen[cargo test -p agentic-workflow lumen_agent_facing_trait_adds_dx_without_expanding_service_obligations && cargo build -p lumen]
```
