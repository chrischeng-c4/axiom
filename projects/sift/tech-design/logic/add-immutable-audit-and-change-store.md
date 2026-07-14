---
id: "1668"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-audit-change-store
entry: event
nodes:
  event: { kind: start, label: "committed audit or change event" }
  normalize: { kind: process, label: "normalize actor subject action target and correlations" }
  chain: { kind: process, label: "append project hash-chain record" }
  hold: { kind: process, label: "apply retention and legal-hold metadata" }
  checkpoint: { kind: terminal, label: "independent immutable timeline checkpoint" }
  query: { kind: start, label: "authorized query or export" }
  authorize: { kind: decision, label: "project read or admin export access" }
  page: { kind: terminal, label: "stable timeline page or controlled manifest" }
  deny: { kind: terminal, label: "explicit denial" }
edges:
  - { from: event, to: normalize }
  - { from: normalize, to: chain }
  - { from: chain, to: hold }
  - { from: hold, to: checkpoint }
  - { from: query, to: authorize }
  - { from: authorize, to: page, when: allowed }
  - { from: authorize, to: deny, when: denied }
---
flowchart LR
    event([audit/change]) --> normalize[normalize] --> chain[hash chain] --> hold[retention/hold] --> checkpoint([checkpoint])
    query([query/export]) --> authorize{authorized}
    authorize -->|yes| page([timeline/manifest])
    authorize -->|no| deny([denied])
```
