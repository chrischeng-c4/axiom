---
id: '1637'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-vectored-buffered-relay
entry: read
nodes:
  read: { kind: start, label: "Read first complete validated backend frame" }
  drain: { kind: process, label: "Drain only complete buffered frames until ReadyForQuery or incomplete suffix" }
  segments: { kind: process, label: "Retain ordered Bytes segments without concatenation" }
  writev: { kind: process, label: "writev segments; advance partial write cursor" }
  ready: { kind: process, label: "Apply ReadyForQuery status after all prior bytes are sent" }
  prefix: { kind: process, label: "Forward valid prefix once then terminate on malformed suffix" }
  wait: { kind: terminal, label: "Await next backend frame" }
edges:
  - { from: read, to: drain }
  - { from: drain, to: segments, label: "complete frames" }
  - { from: segments, to: writev }
  - { from: writev, to: ready, label: "batch contains ReadyForQuery" }
  - { from: writev, to: wait, label: "no ReadyForQuery" }
  - { from: drain, to: prefix, label: "malformed suffix after valid prefix" }
---
flowchart LR
  read([first validated frame]) --> drain[drain complete buffered frames]
  drain --> segments[ordered Bytes segments\nno concatenation]
  segments --> writev[writev + partial cursor]
  writev -->|ReadyForQuery| ready[apply ownership status]
  writev -->|no Ready| wait([await next frame])
  drain -->|malformed suffix| prefix[forward valid prefix then end]
```
