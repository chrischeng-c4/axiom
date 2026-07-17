---
id: '1883'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: prestop-get-drain
entry: prestop
nodes:
  prestop: { kind: start, label: "Kubernetes preStop HTTP GET /drain" }
  handler: { kind: process, label: "shared idempotent drain handler" }
  ready: { kind: process, label: "DrainController flips readiness" }
  done: { kind: terminal, label: "SIGTERM arrives after drain starts" }
edges:
  - { from: prestop, to: handler }
  - { from: handler, to: ready }
  - { from: ready, to: done }
---
flowchart TD
  prestop[preStop GET /drain] --> handler[shared drain handler]
  handler --> ready[DrainController flips readiness]
  ready --> done[SIGTERM after drain starts]
```
