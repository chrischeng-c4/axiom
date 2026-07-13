---
id: '1626'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-immediate-idle-liveness-probe
entry: acquire_idle
nodes:
  acquire_idle: { kind: start, label: "Acquire pops an idle backend" }
  immediate_probe: { kind: process, label: "Poll TcpStream peek once without a timer" }
  pending: { kind: decision, label: "No readability event yet?" }
  live: { kind: process, label: "Hand out unchanged stream" }
  dead: { kind: process, label: "Drop dead backend and retry acquire" }
  readable: { kind: process, label: "Leave bytes unconsumed for relay" }
edges:
  - { from: acquire_idle, to: immediate_probe }
  - { from: immediate_probe, to: pending }
  - { from: pending, to: live, label: "yes" }
  - { from: pending, to: readable, label: "readable bytes" }
  - { from: readable, to: live }
  - { from: immediate_probe, to: dead, label: "EOF or I/O error" }
---
flowchart LR
  acquire_idle([pop idle backend]) --> immediate_probe[poll peek once: no timer]
  immediate_probe --> pending{pending?}
  pending -->|yes| live[reuse unchanged stream]
  pending -->|readable bytes| readable[leave bytes queued]
  readable --> live
  immediate_probe -->|EOF or error| dead[drop and retry]
```
