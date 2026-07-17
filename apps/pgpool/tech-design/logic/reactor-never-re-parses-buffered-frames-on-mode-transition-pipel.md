---
id: '1878'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: reactor-buffered-frame-resume
entry: readable_or_resumed_client
nodes:
  ingest: { kind: process, label: "drain socket into FrameReader when readable" }
  parse: { kind: process, label: "drain buffered frames while client_can_read" }
  stop: { kind: terminal, label: "stop parsing and retain socket backpressure" }
  resume: { kind: process, label: "mode transition resumes buffered parsing without new socket readiness" }
edges:
  - { from: readable_or_resumed_client, to: ingest, label: socket readable }
  - { from: ingest, to: parse }
  - { from: readable_or_resumed_client, to: resume, label: read-enabled transition }
  - { from: resume, to: parse }
  - { from: parse, to: stop, label: waiting or pending }
  - { from: parse, to: parse, label: next buffered frame remains permitted }
---
flowchart TD
  readable["socket readable"] --> ingest["drain socket into FrameReader"]
  ingest --> parse["drain buffered frames while client_can_read"]
  parse --> waiting{"frame makes client Waiting or pending?"}
  waiting -->|yes| stop["stop parsing and keep socket read backpressure"]
  waiting -->|no| parse

  transition["mode transition enables reads"] --> buffered["resume buffered parse without socket readiness"]
  buffered --> parse
  transition --- events["ReadyForQuery Idle, startup replay, auth challenge"]
```
