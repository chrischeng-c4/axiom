---
id: '1819'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-developer-agent-experience-capability
entry: readme
nodes:
  registry: { kind: start, label: "README capability index names Developer and Agent Experience" }
  roots: { kind: process, label: "four subdomains map to bounded work roots and evidence" }
  verify: { kind: process, label: "CLI convention tests prove offline onboarding and command inventory" }
  done: { kind: terminal, label: "agents can find commands, boundaries, and integration contracts offline" }
edges:
  - { from: registry, to: roots }
  - { from: roots, to: verify }
  - { from: verify, to: done }
---
```

The canonical README capability registry adds `developer-agent-experience` as an AgentFirst capability. Its work-root table covers offline command contract (#1817), agent onboarding (#1818), explicit local-only interactive-tooling n/a, and integration contract through the existing production-like runner work (#701). Each row links a deterministic command or test so the capability is checkable rather than narrative-only.
