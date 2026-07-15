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
