---
id: "1589"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-restart-endurance
entry: cycle
nodes:
  cycle:
    kind: start
    label: "Start a real single-node TapeRaft on the same durable directory"
  write:
    kind: process
    label: "Commit a bounded batch of appends and one monotonic checkpoint"
  restart:
    kind: process
    label: "Drop the host and reopen a fresh journal from the same durable directory"
  verify:
    kind: decision
    label: "Did recovery preserve all events, applied floor, and checkpoint?"
  fail:
    kind: terminal
    label: "Fail on loss, duplicate offset, or checkpoint regression"
  next:
    kind: terminal
    label: "Repeat bounded cycles; prove sustained restart recovery"
edges:
  - { from: cycle, to: write }
  - { from: write, to: restart }
  - { from: restart, to: verify }
  - { from: verify, to: fail, label: "no" }
  - { from: verify, to: next, label: "yes" }
---
flowchart TD
 cycle[Start real TapeRaft] --> write[Commit appends and checkpoint]
 write --> restart[Restart from same durable directory]
 restart --> verify{State preserved?}
 verify -->|no| fail([Fail])
 verify -->|yes| next([Repeat bounded cycles])
```
